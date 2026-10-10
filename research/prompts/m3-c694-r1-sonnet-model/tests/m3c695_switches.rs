//! C695 复现（调查员草稿，不入库）：普通盘族写过第一个文件、覆盖写 0 次、没崩了再挂，卸载之前丢一块盘，
//! 剩下那块盘择到 txg 2 那条根，只读挂载报 `UnitUnreadable { slot: SlotNumber(0) }`。四种宽度各一条；丢盘 1 那一份作对照。

mod common_admission;

use std::cell::RefCell;

use common_admission::start_plain;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader,
};
use singlefs_core::root_record::RootRecord;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::MemoryPool;

fn set_k3() {
    use std::sync::atomic::Ordering;
    let names = std::env::var("C695_SWITCHES").unwrap_or_default();
    singlefs_core::mount::candidate_switches::K3_CONDITIONAL.store(names.split(',').any(|n| n == "k3c"), Ordering::SeqCst);
    singlefs_core::mount::candidate_switches::K3_REORDER.store(names.split(',').any(|n| n == "k3r"), Ordering::SeqCst);
    println!("SWITCHES {names}");
}

fn image_without(image: &MemoryPool, lost: DeviceIdentity) -> MemoryPool {
    MemoryPool {
        devices: image.devices.iter().filter(|(identity, _)| **identity != lost)
            .map(|(identity, device)| (*identity, device.clone())).collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    }
}

/// 读的时候记下每一次读（盘、偏移、长度、读没读到）。
struct Tracing<'a> {
    inner: &'a MemoryPool,
    reads: RefCell<Vec<(u32, u64, usize, bool)>>,
}

impl PoolReader for Tracing<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        PoolReader::device_identities(self.inner)
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        PoolReader::device_size_in_bytes(self.inner, device)
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: usize) -> Option<Vec<u8>> {
        let bytes = PoolReader::read(self.inner, device, offset, length);
        self.reads.borrow_mut().push((device.0, offset.0, length, bytes.is_some()));
        bytes
    }
    fn journal_record_offsets_hint(&self, device: DeviceIdentity, ring_start: DeviceOffsetInBytes, ring_bytes: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        PoolReader::journal_record_offsets_hint(self.inner, device, ring_start, ring_bytes)
    }
}

fn whole_read(reader: &dyn PoolReader) -> Result<(u64, usize), String> {
    let mounted = mount_read_only(reader).map_err(|failure| format!("只读挂载：{failure:?}"))?;
    let mut files = 0;
    for record in mounted.mounted.inode_records().to_vec() {
        if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
            continue;
        }
        let file = mounted.mounted.open_file(reader, InodeNumber(record.inode)).map_err(|f| format!("打开：{f:?}"))?;
        file.read_at(reader, FileOffsetInBytes(0), record.size).map_err(|f| format!("读：{f:?}"))?;
        files += 1;
    }
    Ok((mounted.effective_root.checkpoint_txg.0, files))
}

fn pointers(root: &RootRecord) -> String {
    let p = |name: &str, pointer: &singlefs_core::pointer::NodePointer| {
        format!("{name}=[d{}s{},d{}s{}]", pointer.locations[0].device.0, pointer.locations[0].slot.0,
            pointer.locations[1].device.0, pointer.locations[1].slot.0)
    };
    format!("{} {} {} {}", p("tree_table", &root.tree_table), p("instance_table", &root.instance_table),
        p("mapping_root", &root.mapping_root), p("alloc_tree_root", &root.allocation_record_tree_root))
}

#[test]
fn c695_each_width_after_the_first_file_with_one_device_lost() {
    set_k3();
    for (code, width) in [
        (384, HistoryDeviceWidth::UnitAreaOf384Slots),
        (256, HistoryDeviceWidth::UnitAreaOf256Slots),
        (240, HistoryDeviceWidth::UnitAreaOf240Slots),
        (4, HistoryDeviceWidth::FourGibibytes),
    ] {
        let pool = start_plain(width);
        let image = pool.image();
        println!("WIDTH {code} current_txg={} both_devices whole_read={:?}", pool.session().current.root().checkpoint_txg.0, whole_read(&image));
        for lost in [DeviceIdentity(0), DeviceIdentity(1)] {
            let survivor = image_without(&image, lost);
            let chosen = recover(&survivor, JournalPolicy::Consult).effective_root.map(|(instance, txg)| (instance.0, txg.0));
            let root = choose_system_configuration(&survivor).ok().and_then(|configuration| choose_root(&survivor, &configuration));
            let tracing = Tracing { inner: &survivor, reads: RefCell::new(Vec::new()) };
            let outcome = whole_read(&tracing);
            let slot_zero_reads: Vec<_> = tracing.reads.borrow().iter()
                .filter(|(_, offset, length, _)| *offset == 0 && *length as u64 == SLOT_BYTES).cloned().collect();
            println!("  W={code} lost=dev{} recover_effective={chosen:?} chosen_root={:?} {} outcome={outcome:?} unit_reads_at_slot_0={slot_zero_reads:?}",
                lost.0, root.map(|r| (r.instance.0, r.checkpoint_txg.0)), root.map(|r| pointers(&r)).unwrap_or_default());
        }
    }
}

/// 两块盘都在时，根环里每一条自证过的根各自 `open_pool_for_read`：分得开「是丢了盘」还是「这一版本身打不开」。
#[test]
fn c695_every_ring_root_opened_with_both_devices_present() {
    set_k3();
    use singlefs_core::mounted_read::open_pool_for_read;
    use singlefs_core::recovery::{every_root_ring_slot, read_root_ring_slot, RootRingSlotReading};
    let pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    let image = pool.image();
    let configuration = choose_system_configuration(&image).expect("系统配置");
    for ring_slot in every_root_ring_slot(&configuration.immutable.sizes) {
        if let RootRingSlotReading::SelfVerified(root) = read_root_ring_slot(&image, &configuration.immutable.region_devices,
            &configuration.immutable.sizes, &configuration.immutable.filesystem_identifier, ring_slot) {
            let device = configuration.immutable.region_devices[ring_slot.region as usize];
            let opened = open_pool_for_read(&image, &root).map(|_| "Ok".to_string()).unwrap_or_else(|f| format!("Err {f:?}"));
            println!("RING region={} slot={} on_dev{} root=({},{}) {} open_both_devices={opened}",
                ring_slot.region, ring_slot.slot, device.0, root.instance.0, root.checkpoint_txg.0, pointers(&root));
        }
    }
}
