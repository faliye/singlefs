//! 攻方腿 m3-formal-code-r1 的原型（只在拷贝里）：择根、扫环两段胶水改前（提交 fcc4d340 的写法，照原文重写在这里）与改后
//! （今天的 `choose_root`、`scan_journal`）在随机输入上逐个比结局与读盘序列；再量扫环改前改后的堆峰值。
mod common;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};

use common::{build_pool, format_pool};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::journal::{JournalRecord, NamedUnit};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot, read_root_ring_slot,
    scan_journal, PoolReader, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::slot_offset;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

struct Counting;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let live = LIVE.fetch_add(layout.size(), Ordering::SeqCst) + layout.size();
        PEAK.fetch_max(live, Ordering::SeqCst);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::SeqCst);
        System.dealloc(pointer, layout)
    }
}
#[global_allocator]
static GLOBAL: Counting = Counting;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

/// 读盘记一笔、指定的 (盘, 偏移) 读不出。
struct Logged<'a> {
    inner: &'a dyn PoolReader,
    unreadable: BTreeSet<(DeviceIdentity, u64)>,
    order: Vec<DeviceIdentity>,
    log: RefCell<Vec<(DeviceIdentity, u64, usize)>>,
}
impl PoolReader for Logged<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.order.clone()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.inner.device_size_in_bytes(device)
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: usize) -> Option<Vec<u8>> {
        self.log.borrow_mut().push((device, offset.0, length));
        if self.unreadable.contains(&(device, offset.0)) {
            return None;
        }
        self.inner.read(device, offset, length)
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner.journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
}

/// 改前的 choose_root（fcc4d340：visit_valid_roots 逐槽、严格大于才换）。
fn choose_root_before(reader: &dyn PoolReader, system_configuration: &SystemConfiguration) -> Option<RootRecord> {
    let immutable = &system_configuration.immutable;
    let mut best: Option<RootRecord> = None;
    for ring_slot in every_root_ring_slot(&immutable.sizes) {
        match read_root_ring_slot(
            reader,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            ring_slot,
        ) {
            RootRingSlotReading::SelfVerified(candidate) => {
                let candidate_key = (candidate.checkpoint_txg, candidate.instance);
                if best.is_none_or(|current| candidate_key > (current.checkpoint_txg, current.instance)) {
                    best = Some(candidate);
                }
            }
            RootRingSlotReading::Bad(_) => {}
        }
    }
    best
}

/// 改前的 scan_journal（fcc4d340：BTreeMap entry().or_insert 先到先得）。
fn scan_journal_before(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
) -> BTreeMap<(InstanceGeneration, u64), JournalRecord> {
    let expected_filesystem_identifier =
        unit_filesystem_identifier(&system_configuration.immutable.filesystem_identifier);
    let ring_start = DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES);
    let ring_bytes = system_configuration.immutable.sizes.journal_ring_bytes;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let mut records = BTreeMap::new();
    for device in reader.device_identities() {
        let ring_bytes_inside_the_device = reader
            .device_size_in_bytes(device)
            .map_or(0, |device_bytes| ring_bytes.min(device_bytes.saturating_sub(ring_start.0)));
        let offsets = reader
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
            .unwrap_or_else(|| {
                (0..ring_bytes_inside_the_device / JOURNAL_RECORD_BYTES)
                    .map(|record_index| DeviceOffsetInBytes(ring_start.0 + record_index * JOURNAL_RECORD_BYTES))
                    .collect()
            });
        for offset in offsets {
            let Some(bytes) = reader.read(device, offset, record_bytes) else {
                continue;
            };
            if let Some(record) = JournalRecord::parse(&bytes, expected_filesystem_identifier) {
                records.entry((record.instance, record.counter)).or_insert(record);
            }
        }
    }
    records
}

fn write_slot(image: &mut MemoryPool, device: DeviceIdentity, offset: DeviceOffsetInBytes, bytes: &[u8]) {
    image.devices.get_mut(&device).expect("有这块盘").write(offset, bytes);
}

#[test]
fn choose_root_glue_matches_the_fold_before_on_random_rings() {
    let base = format_pool("attack-choose-root-differential").memory_pool();
    let system_configuration = choose_system_configuration(&base).expect("系统配置");
    let mkfs_root = choose_root(&base, &system_configuration).expect("mkfs 根");
    let immutable = system_configuration.immutable;
    let ring_slots = every_root_ring_slot(&immutable.sizes);
    let root_slot_bytes = usize::try_from(immutable.sizes.physical_block_size).expect("槽宽");
    let cases: u64 = std::env::var("ATTACK_CASES").ok().and_then(|v| v.parse().ok()).unwrap_or(3000);
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let (mut same, mut ties_with_different_content, mut none_chosen) = (0u64, 0u64, 0u64);
    for _ in 0..cases {
        let mut image = base.clone();
        let mut unreadable = BTreeSet::new();
        let mut keys_written: BTreeMap<(CheckpointTxg, InstanceGeneration), BTreeSet<u64>> = BTreeMap::new();
        for ring_slot in &ring_slots {
            let device = immutable.region_devices[usize::try_from(ring_slot.region).expect("区域")];
            let offset = slot_offset(*ring_slot, immutable.sizes.fixed_structure_slot_spacing);
            match rng.below(10) {
                0..=3 => {
                    let mut root = mkfs_root;
                    root.checkpoint_txg = CheckpointTxg(rng.below(4));
                    root.instance = InstanceGeneration(u32::try_from(rng.below(3)).expect("小"));
                    root.tree_identifier_watermark = rng.below(3);
                    keys_written.entry((root.checkpoint_txg, root.instance)).or_default().insert(root.tree_identifier_watermark);
                    write_slot(&mut image, device, offset, &root.to_slot(root_slot_bytes));
                }
                4 => {
                    let mut bytes = mkfs_root.to_slot(root_slot_bytes);
                    bytes[100] ^= 0x5A;
                    write_slot(&mut image, device, offset, &bytes);
                }
                5 => write_slot(&mut image, device, offset, &vec![0u8; root_slot_bytes]),
                _ => {}
            }
            if rng.below(8) == 0 {
                unreadable.insert((device, offset.0));
            }
        }
        if keys_written.values().any(|watermarks| watermarks.len() > 1) {
            ties_with_different_content += 1;
        }
        let order = image.device_identities();
        let before_reader = Logged { inner: &image, unreadable: unreadable.clone(), order: order.clone(), log: RefCell::new(Vec::new()) };
        let after_reader = Logged { inner: &image, unreadable, order, log: RefCell::new(Vec::new()) };
        let before = choose_root_before(&before_reader, &system_configuration);
        let after = choose_root(&after_reader, &system_configuration);
        assert_eq!(before, after, "择根结局改前改后不同");
        assert_eq!(before_reader.log.into_inner(), after_reader.log.into_inner(), "择根读盘序列改前改后不同");
        same += 1;
        if after.is_none() {
            none_chosen += 1;
        }
    }
    println!("name=choose-root-differential cases={cases} same={same} cases_with_a_key_written_twice_with_different_watermarks={ties_with_different_content} none_chosen={none_chosen} ring_slots={}", ring_slots.len());
}

/// 只在 `slots` 里那些 (盘, 偏移) 上有字节；别处读出全 0（自证不过）；池外的盘读不出。
struct Synthetic {
    order: Vec<DeviceIdentity>,
    device_bytes: u64,
    slots: BTreeMap<(DeviceIdentity, u64), Vec<u8>>,
    hint: bool,
}
impl PoolReader for Synthetic {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.order.clone()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.order.contains(&device).then_some(self.device_bytes)
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: usize) -> Option<Vec<u8>> {
        if !self.order.contains(&device) || offset.0 + length as u64 > self.device_bytes {
            return None;
        }
        Some(self.slots.get(&(device, offset.0)).cloned().unwrap_or_else(|| vec![0u8; length]))
    }
    fn journal_record_offsets_hint(&self, device: DeviceIdentity, _ring_start: DeviceOffsetInBytes, _ring_bytes: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        self.hint.then(|| {
            self.slots.keys().filter(|(slot_device, _)| *slot_device == device).map(|(_, offset)| DeviceOffsetInBytes(*offset)).collect()
        })
    }
}

fn template_record() -> (JournalRecord, SystemConfiguration) {
    let pool = build_pool("attack-scan-journal-template");
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let template = scan_journal(&image, &system_configuration).into_values().next().expect("建池之后环里有记录");
    (template, system_configuration)
}

#[test]
fn scan_journal_glue_matches_the_entry_or_insert_before_on_random_rings() {
    let (template, mut system_configuration) = template_record();
    let ring_slots: u64 = 16;
    system_configuration.immutable.sizes.journal_ring_bytes = ring_slots * JOURNAL_RECORD_BYTES;
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let cases: u64 = std::env::var("ATTACK_CASES").ok().and_then(|v| v.parse().ok()).unwrap_or(3000);
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    let (mut duplicated_keys_with_different_content, mut total_records) = (0u64, 0usize);
    for _ in 0..cases {
        let mut pool_devices: Vec<DeviceIdentity> = (0..6).map(DeviceIdentity).collect();
        for index in (1..pool_devices.len()).rev() {
            let other = usize::try_from(rng.below(index as u64 + 1)).expect("小");
            pool_devices.swap(index, other);
        }
        pool_devices.truncate(usize::try_from(1 + rng.below(4)).expect("小"));
        let mut slots = BTreeMap::new();
        let mut unreadable = BTreeSet::new();
        let mut contents: BTreeMap<(u32, u64), BTreeSet<Vec<u8>>> = BTreeMap::new();
        for device in &pool_devices {
            for slot in 0..ring_slots {
                let offset = ring_start + slot * JOURNAL_RECORD_BYTES;
                if rng.below(2) == 0 {
                    let mut record = template.clone();
                    record.instance = InstanceGeneration(u32::try_from(rng.below(3)).expect("小"));
                    record.counter = 1 + rng.below(6);
                    record.is_commit = rng.below(2) == 0;
                    record.transaction = rng.below(3);
                    let mut bytes = record.to_bytes();
                    if rng.below(10) == 0 {
                        bytes[200] ^= 0x33;
                    } else {
                        contents.entry((record.instance.0, record.counter)).or_default().insert(bytes.clone());
                    }
                    slots.insert((*device, offset), bytes);
                }
                if rng.below(10) == 0 {
                    unreadable.insert((*device, offset));
                }
            }
        }
        duplicated_keys_with_different_content += contents.values().filter(|copies| copies.len() > 1).count() as u64;
        let synthetic = Synthetic { order: pool_devices.clone(), device_bytes: ring_start + (ring_slots + 2) * JOURNAL_RECORD_BYTES, slots, hint: rng.below(2) == 0 };
        let before_reader = Logged { inner: &synthetic, unreadable: unreadable.clone(), order: pool_devices.clone(), log: RefCell::new(Vec::new()) };
        let after_reader = Logged { inner: &synthetic, unreadable, order: pool_devices, log: RefCell::new(Vec::new()) };
        let before = scan_journal_before(&before_reader, &system_configuration);
        let after = scan_journal(&after_reader, &system_configuration);
        total_records += after.len();
        assert_eq!(before, after, "扫环建表改前改后不同");
        assert_eq!(before_reader.log.into_inner(), after_reader.log.into_inner(), "扫环读盘序列改前改后不同");
    }
    println!("name=scan-journal-differential cases={cases} same={cases} keys_with_two_or_more_different_self_verified_copies={duplicated_keys_with_different_content} records_in_tables={total_records}");
}

/// 环满：每块盘每个槽都是计数器 = 槽号 + 1 的那条记录（每条记录写到每块盘，与 `WriteJournalRecordToEveryDevice` 同形），读时现编。
struct FullRing {
    order: Vec<DeviceIdentity>,
    device_bytes: u64,
    template: JournalRecord,
}
impl PoolReader for FullRing {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.order.clone()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.order.contains(&device).then_some(self.device_bytes)
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: usize) -> Option<Vec<u8>> {
        if !self.order.contains(&device) || offset.0 + length as u64 > self.device_bytes {
            return None;
        }
        let slot = (offset.0 - JOURNAL_RING_START_SLOT * SLOT_BYTES) / JOURNAL_RECORD_BYTES;
        let mut record = self.template.clone();
        record.counter = slot + 1;
        Some(record.to_bytes())
    }
    fn journal_record_offsets_hint(&self, _device: DeviceIdentity, _ring_start: DeviceOffsetInBytes, _ring_bytes: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        None
    }
}

fn peak_of<T>(work: impl FnOnce() -> T) -> (usize, usize, T) {
    let base = LIVE.load(Ordering::SeqCst);
    PEAK.store(base, Ordering::SeqCst);
    let out = work();
    (PEAK.load(Ordering::SeqCst) - base, LIVE.load(Ordering::SeqCst).saturating_sub(base), out)
}

#[test]
fn scan_journal_heap_peak_before_and_after_on_full_mirrored_rings() {
    let (template, mut system_configuration) = template_record();
    println!(
        "name=sizes size_of_journal_record={} size_of_named_unit={} template_named={}",
        std::mem::size_of::<JournalRecord>(),
        std::mem::size_of::<NamedUnit>(),
        template.named.len()
    );
    let named_one = template.named.first().copied().unwrap_or(NamedUnit {
        locations: [LocationEntry { device: DeviceIdentity(0), slot: SlotNumber(4096), unit_checksum: 0 }; 2],
        unit_class: 0,
        birth_tree: TreeIdentifier(1),
        birth_txg: CheckpointTxg(1),
        key_tail: [0; 10],
    });
    let slot_counts: Vec<u64> = std::env::var("ATTACK_SLOTS")
        .ok()
        .map(|v| v.split(',').map(|s| s.parse().expect("槽数")).collect())
        .unwrap_or_else(|| vec![1024, 8192]);
    for named in [0usize, 67] {
        let mut record = template.clone();
        record.named = vec![named_one; named];
        let parsed = JournalRecord::parse(&record.to_bytes(), unit_filesystem_identifier(&system_configuration.immutable.filesystem_identifier));
        assert_eq!(parsed.as_ref().map(|r| r.named.len()), Some(named), "带 {named} 个点名项的记录编得出、解得回");
        for &slots in &slot_counts {
            system_configuration.immutable.sizes.journal_ring_bytes = slots * JOURNAL_RECORD_BYTES;
            for devices in [1u32, 2, 4] {
                let ring = FullRing {
                    order: (0..devices).map(DeviceIdentity).collect(),
                    device_bytes: JOURNAL_RING_START_SLOT * SLOT_BYTES + slots * JOURNAL_RECORD_BYTES,
                    template: record.clone(),
                };
                let (before_peak, before_kept, before) = peak_of(|| scan_journal_before(&ring, &system_configuration));
                let before_len = before.len();
                drop(before);
                let (after_peak, after_kept, after) = peak_of(|| scan_journal(&ring, &system_configuration));
                assert_eq!(after.len(), before_len);
                drop(after);
                println!(
                    "name=heap named={named} ring_slots={slots} devices={devices} records_kept={before_len} before_peak_bytes={before_peak} after_peak_bytes={after_peak} ratio={:.2} before_kept_bytes={before_kept} after_kept_bytes={after_kept}",
                    after_peak as f64 / before_peak as f64
                );
            }
        }
    }
}
