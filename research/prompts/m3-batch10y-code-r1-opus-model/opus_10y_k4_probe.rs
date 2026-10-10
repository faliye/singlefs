//! 攻方原型（m3-batch10y-code-r1 云端攻方，只在冻结副本的拷贝上跑，不是入库用例）：K4，删除线第 33 批 T（C694 的 B、C）与 T1 甲的交界。
//! 起点照第 10i 批攻方原型的 `abandoned_start`（`research/prompts/m3-batch10i-code-r1-opus-model/main.rs`）重写：崩溃恢复落到 C 之前那一条、
//! 取新实例，C 留在根环里成被抛弃根。之后可写挂载（挂载那一刻 C 的账读得出，记下它的落点）；挂着时用户覆盖写 j 次（j 放开扫）；
//! 卸载之前 C 的账那几个单元（树表、中央映射树根、分配记录树根、实例表）从此读坏；正常卸载。臂：OPUS_K4_ARM 空 = 冻结副本今天；
//! `recorded` = 副本 mount.rs 抬 F 那一遍账读不出就照挂载时记在根环表里的落点算（opus-k4-arms.patch）。另一维 S1：挂载时把 C 的槽造成
//! 「两遍里只有一遍自证过」（第 10i 批 s1_plans 的 a 形：一块盘上的有效根在重建那一遍读坏、C 的槽从重读那一遍起一直坏）。
//! 输出每行以 `OPUS10Y-K4` 开头。
#![allow(dead_code, unused_imports, clippy::all)]

#[path = "common_admission/mod.rs"]
mod common_admission;

use common_admission::*;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread,
    ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by, unmount, BeforeTheOneReread,
    MountError, Mounted, RootRingOfTheRebuiltAllocatorRead, ShadowLedger,
};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot, instance_table_of_root, read_root_ring_slot,
    root_is_abandoned_by_the_instance_table, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

fn member(error: &MountError) -> String {
    let text = format!("{error:?}");
    let end = text.find(|c: char| c == '(' || c == '{' || c == ' ').unwrap_or(text.len());
    let inner = text.find("AccountOfAnAbandonedRoot").map(|_| "/AccountOfAnAbandonedRoot").unwrap_or("");
    let inner2 = text.find("RootRingSlotKnownToHoldARoot").map(|_| "/RootRingSlotKnownToHoldARoot").unwrap_or("");
    format!("{}{inner}{inner2}", &text[..end])
}

// ---------------------------------------------------------------- 读故障：根环槽按「第几次读」，C 的账那几个单元按开关

#[derive(Default)]
struct State {
    spacing: u64,
    slots: Vec<(RootRingSlot, DeviceIdentity, u64)>,
    /// (槽, 第几次读) → 这一次读坏。
    rules: BTreeSet<(RootRingSlot, u64)>,
    /// 槽 → 从第几次读起一直坏。
    from_onward: BTreeMap<RootRingSlot, u64>,
    reads: BTreeMap<RootRingSlot, u64>,
    written: BTreeSet<RootRingSlot>,
    /// C 的账那几个单元：(盘, 起点字节, 终点字节)；`account_bad` 为真时读到就报错。
    account: Vec<(DeviceIdentity, u64, u64)>,
    account_bad: bool,
    account_reads_refused: u64,
}

#[derive(Clone, Default)]
struct Shared(Rc<RefCell<State>>);

impl Shared {
    fn on(image: &MemoryPool) -> Shared {
        let sc = choose_system_configuration(image).expect("系统配置");
        let im = sc.immutable;
        let slots = every_root_ring_slot(&im.sizes)
            .into_iter()
            .map(|s| (s, im.region_devices[usize::try_from(s.region).unwrap()], slot_offset(s, im.sizes.fixed_structure_slot_spacing).0))
            .collect();
        Shared(Rc::new(RefCell::new(State { spacing: u64::from(im.sizes.fixed_structure_slot_spacing), slots, ..State::default() })))
    }
    fn refuse(&self, device: DeviceIdentity, offset: u64, len: u64) -> bool {
        let mut st = self.0.borrow_mut();
        if st.account_bad && st.account.iter().any(|(d, lo, hi)| *d == device && offset < *hi && offset + len > *lo) {
            st.account_reads_refused += 1;
            return true;
        }
        let spacing = st.spacing;
        let Some(slot) = st.slots.iter().find(|(_, d, s)| *d == device && offset >= *s && offset < *s + spacing).map(|(slot, _, _)| *slot) else {
            return false;
        };
        let n = {
            let r = st.reads.entry(slot).or_insert(0);
            *r += 1;
            *r
        };
        if st.written.contains(&slot) {
            return false;
        }
        st.rules.contains(&(slot, n)) || st.from_onward.get(&slot).map_or(false, |from| n >= *from)
    }
}

struct FaultDevice {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    shared: Shared,
}

impl BlockDevice for FaultDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if self.shared.refuse(self.identity, offset.0, buffer.len() as u64) {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("攻方点名的这一次读坏")));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], d: WriteDurability) -> Result<(), BlockDeviceError> {
        {
            let mut st = self.shared.0.borrow_mut();
            let spacing = st.spacing;
            let hit: Vec<RootRingSlot> = st
                .slots
                .iter()
                .filter(|(_, dev, start)| *dev == self.identity && offset.0 < *start + spacing && offset.0 + bytes.len() as u64 > *start)
                .map(|(slot, _, _)| *slot)
                .collect();
            st.written.extend(hit);
        }
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

impl DeviceOverASparseImage for FaultDevice {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.image
    }
}

fn fault_devices_on(image: &MemoryPool, shared: &Shared) -> Vec<(DeviceIdentity, FaultDevice)> {
    plain_devices_on(image)
        .into_iter()
        .map(|(identity, inner)| (identity, FaultDevice { identity, inner, shared: shared.clone() }))
        .collect()
}

// ---------------------------------------------------------------- 起点（第 10i 批 abandoned_start 照抄，after = 0）

struct StartPoint {
    name: String,
    parameters: singlefs_core::make_filesystem::MakeFilesystemParameters,
    device_bytes: u64,
    image: MemoryPool,
    write_time_seconds: u64,
    c: RootRecord,
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

/// 实例 1 第一个文件、覆盖写 `overwrites_before` 次；崩了再挂（实例 2）覆盖写一次得 C；藏 C 的根槽与数据单元、系统配置轮换停在 C 之前，
/// 可写挂载落到 C 之前那一条（实例 3 写行与暖机），再把藏的字节写回，进程退出。
fn abandoned_start(width: HistoryDeviceWidth, width_name: &str, overwrites_before: usize) -> StartPoint {
    let mut pool = start_plain(width);
    for _ in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES).expect("实例 1 覆盖写");
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
            let mut verified = singlefs_core::recovery::verified_system_configuration_slots(pool.devices.as_slice(), *identity, spacing, &params.filesystem_identifier);
            verified.sort_by_key(|slot| slot.quantities.slot_generation);
            let [older, newest] = <[_; 2]>::try_from(verified).expect("两槽都自证得过");
            let generation = newest.quantities.slot_generation;
            let mut not_written = older;
            not_written.quantities.slot_generation = generation.checked_sub(2).unwrap();
            (*identity, DeviceOffsetInBytes((generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing), not_written.to_slot())
        })
        .collect();
    for (device, offset, bytes) in &rotations {
        write_bytes(&mut pool, *device, *offset, bytes);
    }
    let abandoning = pool.crash_and_mount_writable().expect("择根落到 C 之前那一条");
    assert!(abandoning.effective_root.checkpoint_txg < c.root.checkpoint_txg);
    for (device, offset, bytes) in &saved {
        write_bytes(&mut pool, *device, *offset, bytes);
    }
    pool.session = None;
    StartPoint {
        name: format!("ab-{width_name}-{overwrites_before}"),
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        image: pool.image(),
        write_time_seconds: pool.write_time_seconds,
        c: c.root,
    }
}

/// 根环里每个槽读出的是有效根（txg）、被抛弃根，还是读不出。
fn valid_slots_on(image: &MemoryPool, device: DeviceIdentity) -> Vec<RootRingSlot> {
    let sc = choose_system_configuration(image).expect("系统配置");
    let im = sc.immutable.clone();
    let newest = choose_root(image, &sc).expect("择得到根");
    let table = instance_table_of_root(image, &newest).expect("实例表");
    every_root_ring_slot(&im.sizes)
        .into_iter()
        .filter(|slot| im.region_devices[usize::try_from(slot.region).unwrap()] == device)
        .filter(|slot| match read_root_ring_slot(image, &im.region_devices, &im.sizes, &im.filesystem_identifier, *slot) {
            RootRingSlotReading::SelfVerified(root) => !root_is_abandoned_by_the_instance_table(&root, &table),
            RootRingSlotReading::Bad(_) => false,
        })
        .collect()
}

struct MountRun {
    result: Result<Mounted, MountError>,
    devices: Vec<(DeviceIdentity, FaultDevice)>,
    hooks: Vec<BTreeMap<RootRingSlot, u64>>,
}

fn mount_with(sp: &StartPoint, shared: &Shared) -> MountRun {
    let mut devices = fault_devices_on(&sp.image, shared);
    let mut hooks = Vec::new();
    let result = {
        let seen = shared.clone();
        let mut hook = || hooks.push(seen.0.borrow().reads.clone());
        mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
            &sp.parameters,
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut hook),
        )
    };
    MountRun { result, devices, hooks }
}

/// 重建分配器读根环那一遍是每个槽的第几次读（第 10i 批 rebuild_pass_read_number 照抄）。
fn rebuild_pass_read_number(sp: &StartPoint) -> u64 {
    let every = every_root_ring_slot(&choose_system_configuration(&sp.image).unwrap().immutable.sizes);
    for k in 1..=20u64 {
        let shared = Shared::on(&sp.image);
        for s in &every {
            shared.0.borrow_mut().rules.insert((*s, k));
        }
        let run = mount_with(sp, &shared);
        let exact = run.hooks.len() == 1 && run.hooks[0].len() == every.len() && run.hooks[0].values().all(|n| *n == k);
        if let Ok(m) = &run.result {
            if exact && m.output.rereads.root_ring_of_the_rebuilt_allocator == RootRingOfTheRebuiltAllocatorRead::OnTheOneReread {
                return k;
            }
        }
    }
    panic!("{}：认不出重建那一遍", sp.name)
}

// ---------------------------------------------------------------- 一段历史

fn account_ranges(c: &RootRecord) -> Vec<(DeviceIdentity, u64, u64)> {
    [c.tree_table, c.mapping_root, c.allocation_record_tree_root]
        .into_iter()
        .filter(|pointer| *pointer != NodePointer::empty_root())
        .flat_map(|pointer| pointer.locations)
        .map(|location| {
            let lo = location.slot.to_device_offset().0;
            (location.device, lo, lo + singlefs_format::SLOT_BYTES)
        })
        .collect()
}

fn i74(image: &MemoryPool) -> usize {
    checker_violations_on(image).iter().filter(|v| v.starts_with("I-7.4")).count()
}

fn other_red(image: &MemoryPool) -> Vec<String> {
    checker_violations_on(image)
        .into_iter()
        .filter(|v| !v.starts_with("I-7.4") && !v.starts_with("I-3.1"))
        .map(|v| v.chars().take(80).collect())
        .collect()
}

fn refusal_of(error: &singlefs_core::mounted_session::UserChangeRefused) -> String {
    let text = format!("{error:?}");
    let head: String = text.chars().take_while(|c| c.is_alphanumeric()).collect();
    let inner = if text.contains("AccountOfAnAbandonedRoot") { "/AccountOfAnAbandonedRoot" } else { "" };
    format!("{head}{inner}")
}

/// 挂上、覆盖写 `j` 次、C 的账从此读坏、正常卸载；被拒之后照用户的几种下一步各走一遍。交回一行。
fn one_history(sp: &StartPoint, arm: &str, s1_device: Option<(DeviceIdentity, u64)>, j: usize) -> String {
    std::env::set_var("OPUS_K4_ARM", arm);
    let shared = Shared::on(&sp.image);
    let spr = sp.parameters.geometry.root_ring_slots_per_region;
    let c_slot = target_for_publish(sp.c.checkpoint_txg, spr);
    shared.0.borrow_mut().account = account_ranges(&sp.c);
    if let Some((device, k)) = s1_device {
        for slot in valid_slots_on(&sp.image, device) {
            shared.0.borrow_mut().rules.insert((slot, k));
        }
        shared.0.borrow_mut().from_onward.insert(c_slot, k + 1);
    }
    let run = mount_with(sp, &shared);
    let head = format!(
        "OPUS10Y-K4 sp={} arm={arm:?} s1={:?} j={j}",
        sp.name,
        s1_device.map(|(d, k)| format!("d{}@{k}", d.0))
    );
    let mounted = match run.result {
        Ok(mounted) => mounted,
        Err(error) => return format!("{head} mount=Err({})", member(&error)),
    };
    let c_is_s1 = ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(&mounted.allocator).contains_key(&c_slot);
    let isolated = format!("{:?}", mounted.output.isolated_slots_per_device);
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pool: PoolUnderTest<FaultDevice> = PoolUnderTest {
        parameters: sp.parameters.clone(),
        device_bytes: sp.device_bytes,
        devices: run.devices,
        session: Some(session),
        write_time_seconds: sp.write_time_seconds + 1000,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let mut writes = Vec::new();
    for _ in 0..j {
        if let Err(error) = pool.overwrite(OVERWRITE_BYTES) {
            writes.push(refusal_of(&error));
            break;
        }
    }
    shared.0.borrow_mut().account_bad = std::env::var_os("OPUS_K4_NO_ACCOUNT_FAULT").is_none();
    let c_left_before_unmount = shared.0.borrow().written.contains(&c_slot);
    let mut session = pool.session.take().unwrap();
    let first = unmount(&pool.parameters, &mut pool.devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
    let first_text = match &first {
        Ok(_) => "Ok".to_string(),
        Err(error) => format!("Err({})", member(error)),
    };
    let mut tail = String::new();
    if first.is_ok() {
        let image = pool.image();
        tail += &format!(" after_unmount_I-7.4={} other={:?}", i74(&image), other_red(&image));
    } else {
        // 用户下一步甲：账仍读不出，接着写到 C 那一槽被这个进程写掉为止（至多 60 次），再卸载
        pool.session = Some(session);
        let mut more = 0;
        let mut write_refusals = BTreeSet::new();
        while !shared.0.borrow().written.contains(&c_slot) && more < 60 {
            if let Err(error) = pool.overwrite(OVERWRITE_BYTES) {
                write_refusals.insert(refusal_of(&error));
            }
            more += 1;
        }
        let mut session = pool.session.take().unwrap();
        let second = unmount(&pool.parameters, &mut pool.devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
        tail += &format!(
            " then_writes={more} c_slot_written={} write_refusals={write_refusals:?} second_unmount={}",
            shared.0.borrow().written.contains(&c_slot),
            match &second { Ok(_) => "Ok".to_string(), Err(e) => format!("Err({})", member(e)) }
        );
        pool.session = None;
    }
    // 用户下一步乙：账仍读不出时崩了再可写挂载；丙：故障消失之后再挂、覆盖写 6 次，每次核 I-7.4
    let remount_with_fault = match pool.crash_and_mount_writable() {
        Ok(_) => "Ok".to_string(),
        Err(error) => format!("Err({})", member(&error)),
    };
    shared.0.borrow_mut().account_bad = false;
    shared.0.borrow_mut().from_onward.clear();
    shared.0.borrow_mut().rules.clear();
    let remount_clean = match pool.crash_and_mount_writable() {
        Ok(_) => "Ok".to_string(),
        Err(error) => format!("Err({})", member(&error)),
    };
    let mut worst_i74 = i74(&pool.image());
    let mut first_i74: Option<String> = checker_violations_on(&pool.image()).into_iter().find(|v| v.starts_with("I-7.4")).map(|v| format!("at=remount_clean {}", v.chars().take(300).collect::<String>()));
    if remount_clean == "Ok" {
        for step in 0..6 {
            if pool.overwrite(OVERWRITE_BYTES).is_err() {
                break;
            }
            let image = pool.image();
            worst_i74 = worst_i74.max(i74(&image));
            if first_i74.is_none() {
                first_i74 = checker_violations_on(&image).into_iter().find(|v| v.starts_with("I-7.4")).map(|v| format!("at=overwrite{step} {}", v.chars().take(300).collect::<String>()));
            }
        }
    }
    format!(
        "{head} c_is_s1={c_is_s1} isolated={isolated} writes_refused={writes:?} c_left_before_unmount={c_left_before_unmount} account_reads_refused={} unmount={first_text}{tail} remount_with_fault={remount_with_fault} remount_clean={remount_clean} worst_I-7.4_after={worst_i74} first_I-7.4={first_i74:?}",
        shared.0.borrow().account_reads_refused
    )
}

#[test]
fn opus_10y_k4_unmount_when_the_account_of_an_abandoned_root_becomes_unreadable() {
    let widths: Vec<(HistoryDeviceWidth, &str)> = match std::env::var("OPUS_K4_WIDTHS").as_deref() {
        Ok("384") => vec![(HistoryDeviceWidth::UnitAreaOf384Slots, "384")],
        Ok("4g") => vec![(HistoryDeviceWidth::FourGibibytes, "4g")],
        _ => vec![(HistoryDeviceWidth::UnitAreaOf384Slots, "384"), (HistoryDeviceWidth::FourGibibytes, "4g")],
    };
    let js: Vec<usize> = std::env::var("OPUS_K4_JS").ok().map_or_else(
        || vec![0, 1, 2, 3, 5, 8, 12, 20],
        |text| text.split(',').map(|j| j.parse().unwrap()).collect(),
    );
    for (width, width_name) in widths {
        let befores: Vec<usize> = std::env::var("OPUS_K4_BEFORES").ok().map_or_else(|| vec![1, 3], |t| t.split(',').map(|x| x.parse().unwrap()).collect());
        for overwrites_before in befores {
            let sp = abandoned_start(width, width_name, overwrites_before);
            let k = rebuild_pass_read_number(&sp);
            println!("OPUS10Y-K4START sp={} c=({},{}) rebuild_pass_read={k}", sp.name, sp.c.instance.0, sp.c.checkpoint_txg.0);
            for s1 in [None, Some((DeviceIdentity(0), k)), Some((DeviceIdentity(1), k))] {
                let arms: Vec<String> = std::env::var("OPUS_K4_ARMS").ok().map_or_else(|| vec![String::new(), "recorded".to_string()], |t| t.split(',').map(|x| if x == "prod" { String::new() } else { x.to_string() }).collect());
                for arm in arms.iter().map(String::as_str) {
                    for j in &js {
                        println!("{}", one_history(&sp, arm, s1, *j));
                    }
                }
            }
        }
    }
}
