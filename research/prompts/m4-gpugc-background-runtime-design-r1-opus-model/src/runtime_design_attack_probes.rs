//! 运行库设计轮攻方腿的探针（只在仓副本里跑，不进主工作区）：产品路径的后台入口 `run_compaction_in_the_background` 在「一把锁加周期线程」会走到的调用次序下
//! 做什么。盘是内存稀疏盘，起点状态只建一次，每一格从它拷。每一格打 `name=` 开头的一行，由报告引；用例不断言「应当怎样」，只钉前提、打读数：
//! 判的人看读数。用户决定的动作（一次用户改动之间醒几次、崩几次、卡住之后做哪个动作、批大小）全放开扫，不写死。
mod common;

use std::cell::Cell;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use common::{parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, SlotNumber};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{
    CallerMonotonicClockReading, CallerOfTheBackgroundCompaction, CompactionBatchSize,
    CompactionBatchUnits, CompactionPacing, CompactionRunForcing, CompactionSwitches,
    CompactionTimerFallback, CompactionWriteBandwidthLimit, EmptyClusterSegmentsFloor,
    ForegroundFsync,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{BackgroundCompactionCall, MountedSession, UserChange};
use singlefs_core::recovery::{effective_rollback_floor, readable_roots};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_verification::memory_pool::SparseBlockDevice;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

/// 定时兜底间隔：取用户定的默认 30 秒。
const INTERVAL_IN_MILLISECONDS: u64 = 30_000;
/// 起点状态那一次顺序写的数据单元数：同 `compaction_off_the_calling_thread_and_admission.rs` 闲置池那一条（数据单元最多的一段里 29 个）。
const DATA_UNITS_OF_THE_STARTING_POINT: usize = 40;

struct CallerForProbes {
    clock_in_milliseconds: Cell<u64>,
    foreground_fsync: Cell<ForegroundFsync>,
}

impl CallerForProbes {
    fn at_the_start() -> Self {
        CallerForProbes {
            clock_in_milliseconds: Cell::new(0),
            foreground_fsync: Cell::new(ForegroundFsync::NoneWaiting),
        }
    }
}

impl CallerOfTheBackgroundCompaction for CallerForProbes {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(Duration::from_millis(self.clock_in_milliseconds.get()))
    }
    fn foreground_fsync(&self) -> ForegroundFsync {
        self.foreground_fsync.get()
    }
}

fn pacing_of_the_default_interval(write_bandwidth_limit: CompactionWriteBandwidthLimit) -> CompactionPacing {
    CompactionPacing {
        timer_fallback: CompactionTimerFallback::EveryMilliseconds(
            NonZeroU64::new(INTERVAL_IN_MILLISECONDS).expect("不是 0"),
        ),
        write_bandwidth_limit,
    }
}

/// mkfs、取号、暖机、第一个文件，都在内存稀疏盘上。
fn devices_of_a_new_pool() -> Devices {
    let pool_parameters = parameters();
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&pool_parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&pool_parameters, &devices, &genesis);
    let first_content = b"first file of the runtime design probes".to_vec();
    {
        let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let _first_version = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机写了两条根"),
            FirstFile {
                content: &first_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    devices
}

/// 从内存里那一份拷一份盘（不重跑 mkfs 与起点历史）。
fn copy_of(devices: &Devices) -> Devices {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copied = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            copied.image = device.image.clone();
            (*identity, copied)
        })
        .collect()
}

fn session_of(devices: &mut Devices) -> MountedSession {
    let mounted = mount_writable(&parameters(), devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    session
}

/// 顺序写 `data_units` 个数据单元，再丢掉会话（不卸载）：起点状态。
fn devices_after_a_sequential_write_of(data_units: usize) -> Devices {
    let mut devices = devices_of_a_new_pool();
    let mut session = session_of(&mut devices);
    let content: Vec<u8> = (0..data_units * data_unit_payload_capacity())
        .map(|index| u8::try_from(index % 241).expect("小于 256"))
        .collect();
    session
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
        )
        .expect("顺序写第一个文件");
    devices
}

fn current_of(session: &MountedSession) -> &TransactionOutput {
    session.current.version_of_trees().expect("有树的一版")
}

/// 数据单元最多的那一段。
fn fullest_data_segment_of(session: &MountedSession) -> (CompactionRegion, u64) {
    let mut data_units_by_segment: std::collections::BTreeMap<u64, u64> =
        std::collections::BTreeMap::new();
    for record in current_of(session).allocation_records.iter().filter(|record| {
        record.device == DeviceIdentity(0) && !record.is_released && record.span_slots == 2
    }) {
        *data_units_by_segment
            .entry(record.slot.0 / 64 * 64)
            .or_default() += 1;
    }
    let (segment, data_units) = data_units_by_segment
        .into_iter()
        .max_by_key(|(_, data_units)| *data_units)
        .expect("写了数据单元");
    (
        CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(segment))
            .expect("按 64 取整的段起点"),
        data_units,
    )
}

/// 会话装上只供测试的开关：批大小、选段、起跑强制、`N₀`，其余照产品值。水位越线的池子在核心层今天只能这样造（`compaction_starts_by_the_watermarks` 只看开关）。
fn force_the_switches(
    session: &mut MountedSession,
    batch_units: usize,
    forced_region: Option<CompactionRegion>,
    empty_cluster_segments_floor: Option<u64>,
) {
    let CompactionSwitches {
        batch_size: _,
        forced_region: _,
        run_forcing: _,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor: floor_of_the_product,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size: CompactionBatchSize::ForcedForTests(
                CompactionBatchUnits::new(batch_units).expect("至少 1"),
            ),
            forced_region,
            run_forcing: CompactionRunForcing::ForcedToStartForTests,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor: empty_cluster_segments_floor
                .map_or(floor_of_the_product, EmptyClusterSegmentsFloor::ForcedForTests),
            relocation_publish_order,
        });
}

fn create_one_inode(session: &mut MountedSession, devices: &mut Devices) -> CheckpointTxg {
    let _published = session
        .publish_user_change(
            devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
        )
        .expect("建一个 inode");
    current_of(session).root.checkpoint_txg
}

/// 一次叫起的读数：起没起跑、为什么、发了几次布、几批、停在哪。
struct WakeReading {
    text: String,
    is_due_run: bool,
    is_timer_run: bool,
    publishes: u64,
    batches: u64,
}

fn wake(
    session: &mut MountedSession,
    devices: &mut Devices,
    caller: &CallerForProbes,
    write_bandwidth_limit: CompactionWriteBandwidthLimit,
) -> WakeReading {
    let call = session
        .run_compaction_in_the_background(
            devices,
            pacing_of_the_default_interval(write_bandwidth_limit),
            caller,
        )
        .expect("盘表是挂载的那一份");
    match call {
        BackgroundCompactionCall::NotStarted { reason, .. } => WakeReading {
            text: format!("not_started={reason:?}"),
            is_due_run: false,
            is_timer_run: false,
            publishes: 0,
            batches: 0,
        },
        BackgroundCompactionCall::Ran { trigger, run } => {
            let trigger_text = format!("{trigger:?}");
            WakeReading {
                is_due_run: trigger_text == "DueAfterAUserChange",
                is_timer_run: trigger_text == "TimerFallback",
                publishes: u64::try_from(run.versions_published.len()).expect("发布数"),
                batches: run.batches_published,
                text: format!(
                    "trigger={trigger_text} publishes={} batches={} outcome={:?}",
                    run.versions_published.len(),
                    run.batches_published,
                    run.outcome
                ),
            }
        }
    }
}

/// 根环里读得出、不低于 F 的根，按用户改动分到它属于哪一个用户状态（某次用户改动的 txg 起到下一次用户改动之前）：环里还留得下几个用户状态。
fn user_states_reachable(devices: &Devices, txgs_of_the_user_changes: &[CheckpointTxg]) -> usize {
    let pool_parameters = parameters();
    let roots = readable_roots(
        devices,
        &pool_parameters.region_devices,
        &pool_parameters.geometry,
        &pool_parameters.filesystem_identifier,
    );
    let floor = effective_rollback_floor(
        devices,
        &pool_parameters.region_devices,
        &pool_parameters.geometry,
        &pool_parameters.filesystem_identifier,
    );
    txgs_of_the_user_changes
        .iter()
        .enumerate()
        .filter(|(index, txg)| {
            let next = txgs_of_the_user_changes.get(index + 1);
            roots.iter().any(|root| {
                root.checkpoint_txg >= floor
                    && root.checkpoint_txg >= **txg
                    && next.map_or(true, |next_txg| root.checkpoint_txg < *next_txg)
            })
        })
        .count()
}

/// 打中「有用户改动时定时兜底那一支轮不到」：每隔 n 次叫起来一次用户改动（n = 0 是一次都没有），一批一个，钟每次走一个间隔，数两种起跑各几趟、几批。
#[test]
fn the_branch_after_a_user_change_shadows_the_timer_fallback() {
    let starting_point = devices_after_a_sequential_write_of(DATA_UNITS_OF_THE_STARTING_POINT);
    for wakes_per_user_change in [0_u64, 1, 2, 3] {
        let mut devices = copy_of(&starting_point);
        let mut session = session_of(&mut devices);
        let (region, _) = fullest_data_segment_of(&session);
        force_the_switches(&mut session, 1, Some(region), None);
        let caller = CallerForProbes::at_the_start();
        let (mut due_runs, mut timer_runs, mut batches, mut publishes) = (0_u64, 0_u64, 0_u64, 0_u64);
        for wake_number in 1..=12_u64 {
            caller.clock_in_milliseconds.set(wake_number * INTERVAL_IN_MILLISECONDS);
            if wakes_per_user_change > 0 && wake_number % wakes_per_user_change == 0 {
                let _txg = create_one_inode(&mut session, &mut devices);
            }
            let reading = wake(&mut session, &mut devices, &caller, CompactionWriteBandwidthLimit::Unlimited);
            due_runs += u64::from(reading.is_due_run);
            timer_runs += u64::from(reading.is_timer_run);
            batches += reading.batches;
            publishes += reading.publishes;
            println!("name=due_shadow_wake wakes_per_user_change={wakes_per_user_change} wake={wake_number} {}", reading.text);
        }
        println!("name=due_shadow wakes_per_user_change={wakes_per_user_change} wakes=12 due_runs={due_runs} timer_runs={timer_runs} batches={batches} compaction_publishes={publishes}");
    }
}

/// 打中「回退候选集被整理发布挤掉」：同一串用户改动，放开一次用户改动之后醒几次（0 次是对照臂），每一步之后数环里还留几个用户状态，报最小值。
#[test]
fn compaction_publishes_of_the_periodic_wakes_squeeze_the_rollback_candidates() {
    let starting_point = devices_after_a_sequential_write_of(DATA_UNITS_OF_THE_STARTING_POINT);
    for wakes_after_each_change in [0_u64, 1, 2, 3] {
        let mut devices = copy_of(&starting_point);
        let mut session = session_of(&mut devices);
        let (region, _) = fullest_data_segment_of(&session);
        force_the_switches(&mut session, 1, Some(region), None);
        let mut txgs = Vec::new();
        for _ in 0..4 {
            txgs.push(create_one_inode(&mut session, &mut devices));
        }
        let caller = CallerForProbes::at_the_start();
        let mut clock = 0_u64;
        let mut smallest = user_states_reachable(&devices, &txgs);
        let mut compaction_publishes = 0_u64;
        for _ in 0..4 {
            txgs.push(create_one_inode(&mut session, &mut devices));
            smallest = smallest.min(user_states_reachable(&devices, &txgs));
            for _ in 0..wakes_after_each_change {
                clock += INTERVAL_IN_MILLISECONDS;
                caller.clock_in_milliseconds.set(clock);
                let reading = wake(&mut session, &mut devices, &caller, CompactionWriteBandwidthLimit::Unlimited);
                compaction_publishes += reading.publishes;
                smallest = smallest.min(user_states_reachable(&devices, &txgs));
                println!("name=rollback_states_step wakes_after_each_change={wakes_after_each_change} {} states_reachable_now={}", reading.text, user_states_reachable(&devices, &txgs));
            }
        }
        println!("name=rollback_states wakes_after_each_change={wakes_after_each_change} user_changes={} compaction_publishes={compaction_publishes} smallest_states_reachable={smallest} states_reachable_at_the_end={}", txgs.len(), user_states_reachable(&devices, &txgs));
    }
}

/// 打中「重挂给一份新预算」：六次用户改动之后不再有用户改动，崩了重挂（丢会话、不卸载）几次，放开每次挂上之后醒几次（0 次是对照臂）。
#[test]
fn each_mount_grants_a_fresh_budget_without_a_user_change() {
    let starting_point = devices_after_a_sequential_write_of(DATA_UNITS_OF_THE_STARTING_POINT);
    for wakes_after_each_mount in [0_u64, 1, 2] {
        let mut devices = copy_of(&starting_point);
        let mut session = session_of(&mut devices);
        let (region, _) = fullest_data_segment_of(&session);
        let mut txgs = Vec::new();
        for _ in 0..6 {
            txgs.push(create_one_inode(&mut session, &mut devices));
        }
        drop(session);
        println!("name=crash_loop_step wakes_after_each_mount={wakes_after_each_mount} mount=0 states_reachable={}", user_states_reachable(&devices, &txgs));
        for mount_number in 1..=3_u64 {
            let mut session = session_of(&mut devices);
            force_the_switches(&mut session, 1, Some(region), None);
            let caller = CallerForProbes::at_the_start();
            let mut publishes_of_this_mount = 0_u64;
            for wake_number in 0..wakes_after_each_mount {
                caller.clock_in_milliseconds.set(wake_number * INTERVAL_IN_MILLISECONDS);
                let reading = wake(&mut session, &mut devices, &caller, CompactionWriteBandwidthLimit::Unlimited);
                publishes_of_this_mount += reading.publishes;
                println!("name=crash_loop_wake wakes_after_each_mount={wakes_after_each_mount} mount={mount_number} wake={wake_number} {}", reading.text);
            }
            drop(session);
            println!("name=crash_loop_step wakes_after_each_mount={wakes_after_each_mount} mount={mount_number} compaction_publishes_of_this_mount={publishes_of_this_mount} states_reachable={}", user_states_reachable(&devices, &txgs));
        }
    }
}

/// 打中「一份预算做不完一条意图时，底额拒掉前台、预算再也回不来」：放开批大小 K，`N₀` 压在此刻全空段数加一（前台写从头就被合取 7 拒，等于全空段用完的盘），
/// 每次叫起之后前台试写一次，数卡住之前 GC 发了几次布、前台头一次写成在第几次叫起。
#[test]
fn a_foreground_refused_by_the_floor_never_refills_the_budget() {
    let starting_point = devices_after_a_sequential_write_of(DATA_UNITS_OF_THE_STARTING_POINT);
    let content = b"foreground write while the empty segments are below the floor".to_vec();
    for batch_units in [1_usize, 2, 3, 4, 8, 16, 32] {
        let mut devices = copy_of(&starting_point);
        let mut session = session_of(&mut devices);
        let (region, data_units_in_the_region) = fullest_data_segment_of(&session);
        let empty_now = session.allocator.devices[0].empty_segments();
        force_the_switches(&mut session, batch_units, Some(region), Some(empty_now + 1));
        let caller = CallerForProbes::at_the_start();
        let (mut publishes, mut refused_by_the_floor, mut first_accepted_at_wake) = (0_u64, 0_u64, None);
        let mut last_reading = String::new();
        let mut other_refusal = String::new();
        for wake_number in 1..=40_u64 {
            caller.clock_in_milliseconds.set(wake_number * INTERVAL_IN_MILLISECONDS);
            let reading = wake(&mut session, &mut devices, &caller, CompactionWriteBandwidthLimit::Unlimited);
            publishes += reading.publishes;
            last_reading = reading.text;
            let attempt = session.publish_user_change(
                &mut devices,
                UserChange::OverwriteOfAnInode {
                    inode: InodeNumber(FIRST_INODE_NUMBER),
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
                },
            );
            match attempt {
                Ok(_published) => {
                    first_accepted_at_wake = Some(wake_number);
                    break;
                }
                Err(refusal) => {
                    let text = format!("{refusal:?}");
                    if text.contains("EmptyClusterSegmentsBelowTheFloor") {
                        refused_by_the_floor += 1;
                    } else {
                        other_refusal = text.chars().take(160).collect();
                    }
                }
            }
        }
        println!("name=floor_and_budget batch_units={batch_units} data_units_in_the_region={data_units_in_the_region} floor={} empty_segments_at_the_end={} compaction_publishes={publishes} foreground_refused_by_the_floor={refused_by_the_floor} first_foreground_accepted_at_wake={first_accepted_at_wake:?} other_refusal=[{other_refusal}] last_wake=[{last_reading}]", empty_now + 1, session.allocator.devices[0].empty_segments());
        if first_accepted_at_wake.is_none() {
            ways_out_after_the_foreground_and_the_compaction_are_both_stuck(&devices, region, batch_units, empty_now + 1);
        }
    }
}

/// 卡住之后放开用户动作：建 inode、建快照、重挂，各从同一份盘拷一份试。批大小与 `N₀` 照原样装回；前台与整理在各自拷出来的会话上重新走到预算扣完。
fn ways_out_after_the_foreground_and_the_compaction_are_both_stuck(
    stuck_devices: &Devices,
    region: CompactionRegion,
    batch_units: usize,
    floor: u64,
) {
    for way_out in ["new_inode", "create_snapshot"] {
        let mut devices = copy_of(stuck_devices);
        let mut session = session_of(&mut devices);
        force_the_switches(&mut session, batch_units, Some(region), Some(floor));
        // 重挂本身给了一份新预算（另一条用例的打中）：先把这一份扣完，才是「卡住」那个状态。
        let caller = CallerForProbes::at_the_start();
        let mut publishes_before = 0_u64;
        for wake_number in 1..=3_u64 {
            caller.clock_in_milliseconds.set(wake_number * INTERVAL_IN_MILLISECONDS);
            publishes_before += wake(&mut session, &mut devices, &caller, CompactionWriteBandwidthLimit::Unlimited).publishes;
        }
        let change = match way_out {
            "new_inode" => UserChange::NewInodes { count: 1, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3 },
            _ => UserChange::CreateSnapshot,
        };
        let outcome = session.publish_user_change(&mut devices, change);
        let outcome_text = match &outcome {
            Ok(_) => "published".to_string(),
            Err(refusal) => format!("{refusal:?}").chars().take(120).collect(),
        };
        caller.clock_in_milliseconds.set(4 * INTERVAL_IN_MILLISECONDS);
        let after = wake(&mut session, &mut devices, &caller, CompactionWriteBandwidthLimit::Unlimited);
        println!("name=floor_and_budget_way_out batch_units={batch_units} way_out={way_out} compaction_publishes_of_the_remount_budget={publishes_before} change=[{outcome_text}] wake_after=[{}]", after.text);
    }
}

/// 打中「不发布的那几种停不扣预算」：只有顺序写的池（满的段），放开池大小，一批 32 个、不强制选段；钟每次走一个间隔，每次叫起量挂钟，打出停在哪、发了几次布。
/// 先小后大，下一档按上一档建池时长乘 4 估，估出来超过 20 分钟就不跑，打一行 `name=wake_cost_skipped`。
#[test]
fn stops_that_publish_nothing_are_not_metered() {
    let mut build_of_the_last_size = Duration::ZERO;
    for data_units in [64_usize, 256, 1024, 4096] {
        if build_of_the_last_size * 4 > Duration::from_secs(20 * 60) {
            println!("name=wake_cost_skipped data_units={data_units} build_of_the_last_size_in_seconds={}", build_of_the_last_size.as_secs());
            continue;
        }
        let started = Instant::now();
        let mut devices = devices_after_a_sequential_write_of(data_units);
        build_of_the_last_size = started.elapsed();
        let mut session = session_of(&mut devices);
        force_the_switches(&mut session, 32, None, None);
        let caller = CallerForProbes::at_the_start();
        let records = current_of(&session).allocation_records.len();
        println!("name=wake_cost_pool data_units={data_units} allocation_records={records} build_in_seconds={}", build_of_the_last_size.as_secs());
        for wake_number in 1..=8_u64 {
            caller.clock_in_milliseconds.set(wake_number * INTERVAL_IN_MILLISECONDS);
            let started_the_wake = Instant::now();
            let reading = wake(&mut session, &mut devices, &caller, CompactionWriteBandwidthLimit::Unlimited);
            println!("name=wake_cost data_units={data_units} allocation_records={records} wake={wake_number} wall_microseconds={} budget_used={} {}", started_the_wake.elapsed().as_micros(), session.compaction.publishes_since_the_last_user_change, reading.text);
        }
    }
}

/// 打中「一把锁下，只有 fsync 让得了路」：真线程、一把 `Mutex` 串会话与盘。整理那一趟进了锁（第一次读钟时举旗）之后，前台线程去抢锁，放开前台是什么：
/// `not_an_fsync`（`accept`、读、切换、卸载这类，调用方答不出「有 fsync 在等」）与 `fsync`（调用方答有 fsync 在等）。数前台等锁期间整理发了几次布。
#[test]
fn only_a_waiting_fsync_makes_the_run_yield_under_one_lock() {
    struct CallerOnTheThread<'flags> {
        started: &'flags AtomicBool,
        fsync_waiting: &'flags AtomicBool,
    }
    impl CallerOfTheBackgroundCompaction for CallerOnTheThread<'_> {
        fn clock_reading(&self) -> CallerMonotonicClockReading {
            self.started.store(true, Ordering::SeqCst);
            CallerMonotonicClockReading(Duration::from_millis(INTERVAL_IN_MILLISECONDS))
        }
        fn foreground_fsync(&self) -> ForegroundFsync {
            if self.fsync_waiting.load(Ordering::SeqCst) {
                ForegroundFsync::Waiting
            } else {
                ForegroundFsync::NoneWaiting
            }
        }
    }
    let starting_point = devices_after_a_sequential_write_of(DATA_UNITS_OF_THE_STARTING_POINT);
    for foreground in ["not_an_fsync", "fsync"] {
        let mut devices = copy_of(&starting_point);
        let mut session = session_of(&mut devices);
        let (region, _) = fullest_data_segment_of(&session);
        force_the_switches(&mut session, 1, Some(region), None);
        let lock = Mutex::new((session, devices));
        let started = AtomicBool::new(false);
        let fsync_waiting = AtomicBool::new(false);
        let (held_by_the_run, publishes_seen_by_the_foreground, stop_text) = std::thread::scope(|scope| {
            let background = scope.spawn(|| {
                let caller = CallerOnTheThread { started: &started, fsync_waiting: &fsync_waiting };
                let mut guard = lock.lock().expect("锁没中毒");
                let (session, devices) = &mut *guard;
                let began = Instant::now();
                let call = session
                    .run_compaction_in_the_background(
                        devices,
                        pacing_of_the_default_interval(CompactionWriteBandwidthLimit::Unlimited),
                        &caller,
                    )
                    .expect("盘表是挂载的那一份");
                let stop_text = match &call {
                    BackgroundCompactionCall::Ran { run, .. } => format!("{:?}", run.outcome),
                    BackgroundCompactionCall::NotStarted { reason, .. } => format!("{reason:?}"),
                };
                (began.elapsed(), stop_text)
            });
            while !started.load(Ordering::SeqCst) {
                std::thread::yield_now();
            }
            if foreground == "fsync" {
                fsync_waiting.store(true, Ordering::SeqCst);
            }
            let guard = lock.lock().expect("锁没中毒");
            let publishes_seen = guard.0.compaction.publishes_since_the_last_user_change;
            drop(guard);
            let (held, stop_text) = background.join().expect("整理线程没 panic");
            (held, publishes_seen, stop_text)
        });
        println!("name=one_lock foreground={foreground} lock_held_by_the_run_in_microseconds={} compaction_publishes_before_the_foreground_got_the_lock={publishes_seen_by_the_foreground} outcome=[{stop_text}]", held_by_the_run.as_micros());
    }
}

/// 打中「写带宽上限按秒算，定时兜底按间隔醒」：放开上限，钟每次走一个间隔，一次用户改动都没有，数每次叫起发了几次布、停在哪。
#[test]
fn the_write_bandwidth_limit_stops_a_run_until_the_next_wake() {
    let starting_point = devices_after_a_sequential_write_of(DATA_UNITS_OF_THE_STARTING_POINT);
    for limit_in_bytes_per_second in [64_u64 << 10, 1 << 20, 16 << 20] {
        let mut devices = copy_of(&starting_point);
        let mut session = session_of(&mut devices);
        let (region, _) = fullest_data_segment_of(&session);
        force_the_switches(&mut session, 1, Some(region), None);
        let caller = CallerForProbes::at_the_start();
        let limit = CompactionWriteBandwidthLimit::BytesPerSecond(
            NonZeroU64::new(limit_in_bytes_per_second).expect("不是 0"),
        );
        for wake_number in 1..=4_u64 {
            caller.clock_in_milliseconds.set(wake_number * INTERVAL_IN_MILLISECONDS);
            let reading = wake(&mut session, &mut devices, &caller, limit);
            println!("name=bandwidth_per_wake limit_in_bytes_per_second={limit_in_bytes_per_second} seconds_between_wakes={} wake={wake_number} {}", INTERVAL_IN_MILLISECONDS / 1000, reading.text);
        }
    }
}

/// 打中「腾出空段」的两种读法：闲置池上整理搬空 R 之后，记录口径的全空段数涨没涨，R 里的槽真可分配了没有（开段看的是后者）；放开之后接着写几次用户改动。
#[test]
fn a_segment_emptied_by_the_records_is_not_yet_a_segment_that_can_be_opened() {
    let starting_point = devices_after_a_sequential_write_of(DATA_UNITS_OF_THE_STARTING_POINT);
    let mut devices = copy_of(&starting_point);
    let mut session = session_of(&mut devices);
    let (region, _) = fullest_data_segment_of(&session);
    let empty_before = session.allocator.devices[0].empty_segments();
    force_the_switches(&mut session, 32, Some(region), None);
    let caller = CallerForProbes::at_the_start();
    caller.clock_in_milliseconds.set(INTERVAL_IN_MILLISECONDS);
    let reading = wake(&mut session, &mut devices, &caller, CompactionWriteBandwidthLimit::Unlimited);
    println!("name=emptied_segment step=after_the_run {} empty_segments_before={empty_before}", reading.text);
    for user_changes in 0..=6_u64 {
        if user_changes > 0 {
            let _txg = create_one_inode(&mut session, &mut devices);
        }
        let device_map = &session.allocator.devices[0];
        let free_slots_of_the_region = (region.start_slot().0..region.end_slot().0)
            .filter(|slot| device_map.is_free(SlotNumber(*slot)))
            .count();
        println!("name=emptied_segment step=user_changes_after_the_run user_changes={user_changes} empty_segments_by_the_records={} free_slots_of_the_region={free_slots_of_the_region} of 64 lowest_empty_segment={:?} is_the_region_the_lowest_empty={}", device_map.empty_segments(), device_map.lowest_empty_segment(), device_map.lowest_empty_segment() == Some(region.start_slot()));
    }
}
