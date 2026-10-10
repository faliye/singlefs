//! 攻方副本原型（m3-batch10-code-r2 opus-s2，K2）：rr2 重读那一遍整份换掉第一遍的读数时，第一遍读出、重读读坏的被抛弃根是否掉出影子账。
//! 只在攻方副本上跑；臂由环境变量 ATTACK_RR2_ARM 选（副本 mount.rs 里那一处开关）：空 = 冻结副本今天的 rr2，before = rr2 之前，merge = 重读按槽并两遍。
mod common_admission;

use common_admission::{checker_violations_on, plain_devices_on, start_plain, PoolUnderTest};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread, BeforeTheOneReread,
    ShadowLedger,
};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::recovery::{choose_system_configuration, every_root_ring_slot};
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::TransactionOutput;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fault {
    Error,
    Zeros,
    Garbage,
}

struct State {
    spacing: u64,
    slots: Vec<(RootRingSlot, DeviceIdentity, u64)>,
    /// (槽, 这个槽第几次读) → 读成什么。
    rules: BTreeMap<(RootRingSlot, u64), Fault>,
    reads: BTreeMap<RootRingSlot, u64>,
    fired: u64,
}

#[derive(Clone)]
struct Shared(Rc<RefCell<State>>);

impl Shared {
    fn note(&self, device: DeviceIdentity, offset: u64) -> Option<(Fault, u64)> {
        let mut state = self.0.borrow_mut();
        let spacing = state.spacing;
        let (slot, start) = state
            .slots
            .iter()
            .find(|(_, d, s)| *d == device && offset >= *s && offset < *s + spacing)
            .map(|(slot, _, s)| (*slot, *s))?;
        let n = {
            let r = state.reads.entry(slot).or_insert(0);
            *r += 1;
            *r
        };
        let fault = *state.rules.get(&(slot, n))?;
        state.fired += 1;
        Some((fault, start))
    }
}

struct FaultyDevice {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    shared: Shared,
}

impl BlockDevice for FaultyDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let Some((fault, start)) = self.shared.note(self.identity, offset.0) else {
            return self.inner.read_at(offset, buffer);
        };
        let byte = match fault {
            Fault::Error => {
                return Err(BlockDeviceError::InputOutput(std::io::Error::other("攻方点名的这一次读坏")))
            }
            Fault::Zeros => 0u8,
            Fault::Garbage => 0xA5u8,
        };
        self.inner.read_at(offset, buffer)?;
        let spacing = self.shared.0.borrow().spacing;
        let lo = usize::try_from(start.saturating_sub(offset.0)).unwrap();
        let hi = usize::try_from((start + spacing - offset.0).min(buffer.len() as u64)).unwrap();
        buffer[lo..hi].fill(byte);
        Ok(())
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], d: WriteDurability) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, d)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

fn shared_for(image: &MemoryPool, rules: BTreeMap<(RootRingSlot, u64), Fault>) -> Shared {
    let sc = choose_system_configuration(image).expect("系统配置");
    let im = sc.immutable;
    let slots = every_root_ring_slot(&im.sizes)
        .into_iter()
        .map(|s| {
            (
                s,
                im.region_devices[usize::try_from(s.region).unwrap()],
                slot_offset(s, im.sizes.fixed_structure_slot_spacing).0,
            )
        })
        .collect();
    Shared(Rc::new(RefCell::new(State {
        spacing: u64::from(im.sizes.fixed_structure_slot_spacing),
        slots,
        rules,
        reads: BTreeMap::new(),
        fired: 0,
    })))
}

struct Abandoned {
    pool: PoolUnderTest<SparseBlockDevice>,
    c: TransactionOutput,
    c_slot: RootRingSlot,
    /// C 写出的单元在盘上的落点与那时的字节（写回之后逐个核过等于 C 写的）。
    c_places: Vec<(DeviceIdentity, SlotNumber, Vec<u8>)>,
}

fn read_bytes(pool: &PoolUnderTest<SparseBlockDevice>, device: DeviceIdentity, offset: DeviceOffsetInBytes, len: usize) -> Vec<u8> {
    let index = pool.devices.iter().position(|(i, _)| *i == device).unwrap();
    let mut bytes = vec![0u8; len];
    pool.devices[index].1.read_at(offset, &mut bytes).unwrap();
    bytes
}

fn write_bytes(pool: &mut PoolUnderTest<SparseBlockDevice>, device: DeviceIdentity, offset: DeviceOffsetInBytes, bytes: &[u8]) {
    let index = pool.devices.iter().position(|(i, _)| *i == device).unwrap();
    pool.devices[index].1.write_at(offset, bytes, WriteDurability::Plain).unwrap();
}

/// 第一个文件、覆盖写 `overwrites_before` 次（实例 1）；崩了再挂（实例 2），覆盖写一次得 C；再照 `tests/common` 的
/// `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 那一形抛弃 C（藏 C 的根槽与数据单元、系统配置轮换停在 C 之前，
/// 可写挂载落到 C 之前那一条、实例 3 写行与暖机，再把藏的字节写回），进程退出。
fn pool_with_an_abandoned_root(width: HistoryDeviceWidth, overwrites_before: usize) -> Abandoned {
    let mut pool = start_plain(width);
    for _ in 0..overwrites_before {
        pool.overwrite(2999).expect("实例 1 覆盖写");
    }
    pool.crash_and_mount_writable().expect("取号 2");
    pool.overwrite(2500).expect("C");
    let c = pool.current_file_version().clone();
    let params = pool.parameters.clone();
    let target = target_for_publish(c.root.checkpoint_txg, params.geometry.root_ring_slots_per_region);
    let root_device = params.region_devices[usize::try_from(target.region).unwrap()];
    let root_offset = slot_offset(target, params.geometry.fixed_structure_slot_spacing);
    let root_len = usize::try_from(params.geometry.physical_block_size).unwrap();
    let unit_len = usize::try_from(singlefs_format::DATA_UNIT_BYTES).unwrap();
    let mut places: Vec<(DeviceIdentity, DeviceOffsetInBytes, usize)> = vec![(root_device, root_offset, root_len)];
    for pointer in &c.data_pointers {
        for location in pointer.locations {
            places.push((location.device, location.slot.to_device_offset(), unit_len));
        }
    }
    let mut saved = Vec::new();
    for (device, offset, len) in &places {
        let bytes = read_bytes(&pool, *device, *offset, *len);
        write_bytes(&mut pool, *device, *offset, &vec![0u8; *len]);
        saved.push((*device, *offset, bytes));
    }
    let spacing = u64::from(params.geometry.fixed_structure_slot_spacing);
    let rotations: Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> = pool
        .devices
        .iter()
        .map(|(identity, _)| {
            let mut verified = singlefs_core::recovery::verified_system_configuration_slots(
                pool.devices.as_slice(),
                *identity,
                spacing,
                &params.filesystem_identifier,
            );
            verified.sort_by_key(|slot| slot.quantities.slot_generation);
            let [older, newest] = <[_; 2]>::try_from(verified).expect("两槽都自证得过");
            let generation = newest.quantities.slot_generation;
            let mut not_written = older;
            not_written.quantities.slot_generation = generation.checked_sub(2).unwrap();
            (
                *identity,
                DeviceOffsetInBytes((generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing),
                not_written.to_slot(),
            )
        })
        .collect();
    for (device, offset, bytes) in &rotations {
        write_bytes(&mut pool, *device, *offset, bytes);
    }
    let abandoning = pool.crash_and_mount_writable().expect("择根落到 C 之前那一条，照常可写挂载");
    assert!(abandoning.effective_root.checkpoint_txg < c.root.checkpoint_txg, "前提：落到 C 之前");
    for (device, offset, bytes) in &saved {
        write_bytes(&mut pool, *device, *offset, bytes);
    }
    pool.session = None;
    let mut c_places = Vec::new();
    for unit in &c.units {
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let on_disk = read_bytes(&pool, device, unit.slot.to_device_offset(), unit.bytes.len());
            if on_disk == unit.bytes {
                c_places.push((device, unit.slot, on_disk));
            }
        }
    }
    for pointer in &c.data_pointers {
        for location in pointer.locations {
            let on_disk = read_bytes(&pool, location.device, location.slot.to_device_offset(), unit_len);
            if !c_places.iter().any(|(d, s, _)| *d == location.device && *s == location.slot) {
                c_places.push((location.device, location.slot, on_disk));
            }
        }
    }
    Abandoned { pool, c, c_slot: target, c_places }
}

struct Outcome {
    line: String,
    hook_counts: Vec<BTreeMap<RootRingSlot, u64>>,
    rereads: String,
}

/// 照 `rules` 再可写挂载一次（准入判着、影子账开），之后至多覆盖写 `continue_publishes` 次，看 C 的单元有没有被改写、checker 红什么。
fn remount_and_continue(ab: &Abandoned, rules: BTreeMap<(RootRingSlot, u64), Fault>, continue_publishes: usize) -> Outcome {
    let image = ab.pool.image();
    let shared = shared_for(&image, rules);
    let mut devices: Vec<(DeviceIdentity, FaultyDevice)> = plain_devices_on(&image)
        .into_iter()
        .map(|(identity, inner)| (identity, FaultyDevice { identity, inner, shared: shared.clone() }))
        .collect();
    let mut hook_counts = Vec::new();
    let mounted = {
        let seen = shared.clone();
        let mut hook = || hook_counts.push(seen.0.borrow().reads.clone());
        mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
            &ab.pool.parameters,
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut hook),
        )
    };
    let fired = shared.0.borrow().fired;
    let mounted = match mounted {
        Ok(mounted) => mounted,
        Err(error) => {
            return Outcome {
                line: format!("result=Err({error:?}) fired={fired} hooks={}", hook_counts.len()),
                hook_counts,
                rereads: String::new(),
            }
        }
    };
    let rereads = format!("{:?}", mounted.output.rereads);
    let compact = format!(
        "read_stage={} table={:?} ring={:?}",
        if rereads.contains("read_stage: OnTheFirstRead") { "First" } else { "Reread" },
        mounted.output.rereads.instance_table_of_the_newest_root,
        mounted.output.rereads.root_ring_of_the_rebuilt_allocator
    );
    let isolated = mounted.output.isolated_slots_per_device.clone();
    let unreadable = mounted.output.abandoned_roots_unreadable;
    let free_c_places = ab
        .c_places
        .iter()
        .filter(|(device, slot, _)| mounted.allocator.devices[usize::try_from(device.0).unwrap()].is_free(*slot))
        .count();
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pool = PoolUnderTest {
        parameters: ab.pool.parameters.clone(),
        device_bytes: ab.pool.device_bytes,
        devices: devices.into_iter().map(|(identity, d)| (identity, d.inner)).collect(),
        session: Some(session),
        write_time_seconds: ab.pool.write_time_seconds + 100,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let mut reuse = format!("no_reuse_after={continue_publishes}");
    let mut publishes = 0;
    while publishes < continue_publishes {
        if pool.overwrite(2999).is_err() {
            reuse = format!("overwrite_refused_at={publishes}");
            break;
        }
        publishes += 1;
        let overwritten: Vec<(u32, u64)> = ab
            .c_places
            .iter()
            .filter(|(device, slot, bytes)| read_bytes(&pool, *device, slot.to_device_offset(), bytes.len()) != *bytes)
            .map(|(device, slot, _)| (device.0, slot.0))
            .collect();
        if !overwritten.is_empty() {
            reuse = format!("reused_after_publishes={publishes} txg={} overwritten={overwritten:?}", pool.session().current.root().checkpoint_txg.0);
            break;
        }
    }
    let floor_after = pool.rollback_floor().0;
    let red: BTreeSet<String> = checker_violations_on(&pool.image())
        .into_iter()
        .map(|v| v.split(':').next().unwrap_or("").to_string())
        .collect();
    Outcome {
        line: format!(
            "result=Ok fired={fired} hooks={} {compact} isolated={isolated:?} abandoned_roots_unreadable={unreadable} c_places={} free_c_places_after_mount={free_c_places} {reuse} floor_after={floor_after} checker_red={red:?}",
            hook_counts.len(),
            ab.c_places.len()
        ),
        hook_counts,
        rereads,
    }
}

fn newest_self_verified_slot_on(ab: &Abandoned, slots: &[RootRingSlot]) -> Option<RootRingSlot> {
    let image = ab.pool.image();
    let im = choose_system_configuration(&image).expect("系统配置").immutable;
    slots
        .iter()
        .filter_map(|slot| match singlefs_core::recovery::read_root_ring_slot(&image, &im.region_devices, &im.sizes, &im.filesystem_identifier, *slot) {
            singlefs_core::recovery::RootRingSlotReading::SelfVerified(root) => Some((root.checkpoint_txg, *slot)),
            _ => None,
        })
        .max()
        .map(|(_, slot)| slot)
}

fn slots_of_region(ab: &Abandoned, region: u64) -> Vec<RootRingSlot> {
    every_root_ring_slot(&ab.pool.parameters.geometry)
        .into_iter()
        .filter(|s| s.region == region)
        .collect()
}

fn rules_of(entries: &[(Vec<RootRingSlot>, u64, Fault)]) -> BTreeMap<(RootRingSlot, u64), Fault> {
    let mut rules = BTreeMap::new();
    for (slots, nth, fault) in entries {
        for slot in slots {
            rules.insert((*slot, *nth), *fault);
        }
    }
    rules
}

/// 重建分配器读根环那一遍是每个槽的第几次读：整环在第 k 次读坏，钩子恰调一次、调的那一刻每个槽恰读过 k 次、读阶段与实例表都第一遍判完的那个 k。
fn rebuild_pass_read_number(ab: &Abandoned) -> u64 {
    let every = every_root_ring_slot(&ab.pool.parameters.geometry);
    let mut found = None;
    for k in 1..=10u64 {
        let outcome = remount_and_continue(ab, rules_of(&[(every.clone(), k, Fault::Error)]), 0);
        let exact = outcome.hook_counts.len() == 1 && outcome.hook_counts[0].values().all(|n| *n == k);
        println!("name=k2-calibrate k={k} exact={exact} {}", outcome.line);
        if exact
            && outcome.rereads.contains("read_stage: OnTheFirstRead")
            && outcome.rereads.contains("instance_table_of_the_newest_root: OnTheFirstRead")
            && outcome.rereads.contains("root_ring_of_the_rebuilt_allocator: OnTheOneReread")
            && found.is_none()
        {
            found = Some(k);
        }
    }
    found.expect("找得到重建那一遍")
}

#[test]
fn k2_attack_grid() {
    let arm = std::env::var("ATTACK_RR2_ARM").unwrap_or_default();
    let width = match std::env::var("ATTACK_WIDTH").as_deref() {
        Ok("384") => HistoryDeviceWidth::UnitAreaOf384Slots,
        _ => HistoryDeviceWidth::FourGibibytes,
    };
    let overwrites_before: usize = std::env::var("ATTACK_OVERWRITES_BEFORE").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let continue_publishes: usize = std::env::var("ATTACK_CONTINUE").ok().and_then(|v| v.parse().ok()).unwrap_or(40);
    let ab = pool_with_an_abandoned_root(width, overwrites_before);
    println!(
        "name=k2-start arm={arm:?} width={width:?} overwrites_before={overwrites_before} c=({},{}) c_slot={:?} c_places={}",
        ab.c.root.instance.0, ab.c.root.checkpoint_txg.0, ab.c_slot, ab.c_places.len()
    );
    let k = rebuild_pass_read_number(&ab);
    println!("name=k2-k arm={arm:?} k={k}");
    let device_one = slots_of_region(&ab, 1);
    let c_slot = vec![ab.c_slot];
    let mut cells: Vec<(String, BTreeMap<(RootRingSlot, u64), Fault>)> = vec![
        ("no-fault".into(), BTreeMap::new()),
        ("pass1:dev1-all-e".into(), rules_of(&[(device_one.clone(), k, Fault::Error)])),
        ("pass1:c-e (C554 control)".into(), rules_of(&[(c_slot.clone(), k, Fault::Error)])),
        (
            "pass1:dev1-all-e+c-e reread:-".into(),
            rules_of(&[(device_one.clone(), k, Fault::Error), (c_slot.clone(), k, Fault::Error)]),
        ),
    ];
    for fault in [Fault::Error, Fault::Zeros, Fault::Garbage] {
        cells.push((
            format!("pass1:dev1-all-e reread:c-{fault:?}"),
            rules_of(&[(device_one.clone(), k, Fault::Error), (c_slot.clone(), k + 1, fault)]),
        ));
    }
    // 重读那一遍把盘 0 上除最新那条根之外的每个槽读坏（第一遍读得出），看回收门槛、抬 F 上限与 checker。
    let device_zero: Vec<RootRingSlot> = slots_of_region(&ab, 0).into_iter().chain(slots_of_region(&ab, 2)).collect();
    let newest_zero = newest_self_verified_slot_on(&ab, &device_zero);
    let device_zero_but_newest: Vec<RootRingSlot> = device_zero.iter().copied().filter(|s| Some(*s) != newest_zero).collect();
    let device_zero_but_newest_and_c: Vec<RootRingSlot> = device_zero_but_newest.iter().copied().filter(|s| *s != ab.c_slot).collect();
    for fault in [Fault::Error, Fault::Zeros, Fault::Garbage] {
        cells.push((
            format!("pass1:dev1-all-e reread:dev0-but-newest-and-c-{fault:?}"),
            rules_of(&[(device_one.clone(), k, Fault::Error), (device_zero_but_newest_and_c.clone(), k + 1, fault)]),
        ));
    }
    for fault in [Fault::Error, Fault::Zeros, Fault::Garbage] {
        cells.push((
            format!("pass1:dev1-all-e reread:dev0-but-newest-{fault:?}"),
            rules_of(&[(device_one.clone(), k, Fault::Error), (device_zero_but_newest.clone(), k + 1, fault)]),
        ));
    }
    let every = every_root_ring_slot(&ab.pool.parameters.geometry);
    cells.push((
        "pass1:dev1-all-e reread:all-e".into(),
        rules_of(&[(device_one.clone(), k, Fault::Error), (every.clone(), k + 1, Fault::Error)]),
    ));
    cells.push((
        "pass1:dev1-all-e reread:dev1-all-e".into(),
        rules_of(&[(device_one.clone(), k, Fault::Error), (device_one.clone(), k + 1, Fault::Error)]),
    ));
    for (name, rules) in cells {
        let outcome = remount_and_continue(&ab, rules, continue_publishes);
        println!("name=k2-cell arm={arm:?} cell={name:?} {}", outcome.line);
    }
    let _ = CheckpointTxg(0);
}

/// 读法 ①：新池上哪几次可写挂载走根环重读。mkfs 之后第一次可写挂载在第 n 个写上掉电（n 扫到那次挂载写完为止），再可写挂载一次，
/// 记这一次是不是重读、钩子调几次、每个根环槽读了几次。
#[test]
fn k2_young_ring_rereads() {
    use singlefs_core::make_filesystem::make_filesystem;
    use singlefs_harness::fault_injection::{
        FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
        FaultSchedule, InjectedFault, SharedFaultPlan,
    };
    let width = HistoryDeviceWidth::FourGibibytes;
    let parameters = width.parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| (identity, SparseBlockDevice::new(width.device_bytes(), PhysicalBlockSizeInBytes(512))))
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let after_mkfs = MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: width.device_bytes(),
    };
    let observe = |image: &MemoryPool, label: &str| -> Option<MemoryPool> {
        let shared = shared_for(image, BTreeMap::new());
        let mut faulty: Vec<(DeviceIdentity, FaultyDevice)> = plain_devices_on(image)
            .into_iter()
            .map(|(identity, inner)| (identity, FaultyDevice { identity, inner, shared: shared.clone() }))
            .collect();
        let mut hooks = 0u32;
        let mounted = {
            let mut hook = || hooks += 1;
            mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
                &parameters,
                &mut faulty,
                SpaceAdmission::JudgedByTheFormula,
                ShadowLedger::On,
                BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut hook),
            )
        };
        let reads: BTreeSet<u64> = shared.0.borrow().reads.values().copied().collect();
        match mounted {
            Ok(mounted) => {
                println!(
                    "name=k2-young {label} ring={:?} hooks={hooks} reads_per_ring_slot={reads:?} effective_root=({},{})",
                    mounted.output.rereads.root_ring_of_the_rebuilt_allocator,
                    mounted.output.effective_root.instance.0,
                    mounted.output.effective_root.checkpoint_txg.0
                );
                Some(MemoryPool {
                    devices: faulty.iter().map(|(identity, d)| (*identity, d.inner.image.clone())).collect(),
                    device_size_in_bytes: image.device_size_in_bytes,
                })
            }
            Err(error) => {
                println!("name=k2-young {label} result=Err({error:?}) hooks={hooks} reads_per_ring_slot={reads:?}");
                None
            }
        }
    };
    let after_m1 = observe(&after_mkfs, "mount=M1-after-mkfs").expect("M1");
    let after_m2 = observe(&after_m1, "mount=M2-after-a-clean-M1").expect("M2");
    observe(&after_m2, "mount=M3");
    let mut rereading = 0u32;
    let mut n = 1u64;
    loop {
        let plan = SharedFaultPlan::armed(
            width.fixed_geometry(),
            FaultSchedule {
                fault: InjectedFault::WriteFails,
                device: FaultDeviceSelector::EveryDevice,
                placement: FaultPlacement::AnyOffset,
                counting: FaultCounting::AcrossThePool,
                occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(n),
            },
        );
        let mut crashing: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&after_mkfs)
            .into_iter()
            .map(|(identity, device)| (identity, FaultInjectingBlockDevice::new(identity, device, plan.clone())))
            .collect();
        let crashed = singlefs_core::mount::mount_writable(&parameters, &mut crashing).is_err();
        let image = MemoryPool {
            devices: crashing.iter().map(|(identity, d)| (*identity, d.wrapped_device().image.clone())).collect(),
            device_size_in_bytes: width.device_bytes(),
        };
        if !crashed {
            println!("name=k2-young-sweep M1 finished before write {n}: stop");
            break;
        }
        let shared = shared_for(&image, BTreeMap::new());
        let mut faulty: Vec<(DeviceIdentity, FaultyDevice)> = plain_devices_on(&image)
            .into_iter()
            .map(|(identity, inner)| (identity, FaultyDevice { identity, inner, shared: shared.clone() }))
            .collect();
        let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
            &parameters,
            &mut faulty,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            BeforeTheOneReread::RereadImmediately,
        );
        let ring = mounted.as_ref().map(|m| format!("{:?}", m.output.rereads.root_ring_of_the_rebuilt_allocator)).unwrap_or_else(|e| format!("Err({e:?})"));
        if ring == "OnTheOneReread" {
            rereading += 1;
        }
        println!("name=k2-young-sweep m1_crash_at_write={n} m2_ring={ring}");
        n += 1;
        if n > 400 {
            break;
        }
    }
    println!("name=k2-young-sweep-total crash_points={} m2_rereading={rereading}", n - 1);
}
