//! m3-root-ring-code-r1 攻方原型 K3：R2 与留着的「某块盘一份自证过的都没有就拒」合起来。
//! 造法：mkfs → 可写挂载 → 第一个文件 → 覆盖写到 P；再录一次覆盖写，崩在发布末尾系统配置轮换那一写的中途：
//! 盘 `torn_device` 那一槽只落了头 512 字节（层 0 的撕裂粒度是物理块 512），之后的写都没落。
//! 然后给盘上指定的系统配置槽挂一个一直读不出的故障（`faults`：dev:slot 列表，slot 0/1），连做 `tries` 次可写挂载，
//! 每次之间不改故障；打印每次的结局、只读挂载的结局，与 R2 两半、每块盘自证过几份。
//! 用法：sysconf_r2_forever <P> <torn_device：0|1|none> <faults：如 0:1 或 1:0,0:1 或 -> <tries>
use std::cell::RefCell;
use std::rc::Rc;

use rr_attack_proto::{build_plain, content, devices_of, geometry, parameters, pool_of, recording_of, HidingDevice, WRITE_TIME};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{system_configuration_slots_of_one_device_read_once, system_configuration_witness_is_undecidable, PoolReader};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{MemoryPool, WrittenContents};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

struct Faulty<'a> { image: &'a MemoryPool, hidden: Vec<(u32, u64)> }
impl PoolReader for Faulty<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> { self.image.device_identities() }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> { self.image.device_size_in_bytes(device) }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: usize) -> Option<Vec<u8>> {
        let end = offset.0 + length as u64;
        if self.hidden.iter().any(|(d, o)| *d == device.0 && offset.0 < o + 4096 && *o < end) { return None; }
        self.image.read(device, offset, length)
    }
    fn journal_record_offsets_hint(&self, device: DeviceIdentity, ring_start: DeviceOffsetInBytes, ring_bytes: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        self.image.journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let p: u64 = a[1].parse().expect("P");
    let torn: Option<u32> = a[2].parse().ok();
    let faults: Vec<(u32, u64)> = if a[3] == "-" { Vec::new() } else { a[3].split(',').map(|f| { let (d, s) = f.split_once(':').expect("dev:slot"); (d.parse().expect("dev"), s.parse().expect("slot")) }).collect() };
    let tries: u64 = a[4].parse().expect("tries");
    let parameters = parameters(4);
    let geo = geometry(&parameters);
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let (image, _) = build_plain(4, p);
    let image = if let Some(torn_device) = torn {
        let mut devices = devices_of(&image);
        let singlefs_core::mount::Mounted { mut allocator, current, .. } = mount_writable(&parameters, &mut devices).expect("重开");
        let current = current.into_file_version().expect("有文件");
        let before = pool_of(&devices);
        let stream = SharedStream::retaining_contents();
        let mut recording = recording_of(&before, &stream);
        {
            let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
            publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(77), write_time_seconds: WRITE_TIME + 77 }, current.root.instance).expect("被截的那次");
        }
        let operations = stream.retained_operations();
        let (writes, _) = singlefs_harness::memory_pool::writes_and_segments(&operations, &geo);
        let cut = writes.iter().position(|w| w.kind == StepKind::SystemConfigurationSlot && w.device.0 == torn_device).expect("那块盘的轮换写");
        let mut crashed = before;
        crashed.apply_writes(&writes[..cut]);
        let w = &writes[cut];
        let WrittenContents::Bytes(bytes) = &w.contents else { panic!("系统配置写是普通写") };
        let mut torn_write = w.clone();
        torn_write.contents = WrittenContents::Bytes(bytes[..512].to_vec());
        crashed.apply_writes(std::slice::from_ref(&torn_write));
        println!("TORN device={torn_device} slot_offset={} (写之前这一槽是另一份，之后只落了头 512 字节)", w.offset.0);
        crashed
    } else {
        image
    };
    let hidden_offsets: Vec<(u32, u64)> = faults.iter().map(|(d, s)| (*d, s * spacing)).collect();
    let faulty = Faulty { image: &image, hidden: hidden_offsets.clone() };
    let per_device: Vec<_> = faulty.device_identities().into_iter().map(|d| system_configuration_slots_of_one_device_read_once(&faulty, d, spacing, &parameters.filesystem_identifier)).collect();
    for d in &per_device {
        println!("DEVICE {} self_verified={} read_errors={}", d.device.0, d.self_verified.len(), d.slots_reporting_a_read_error);
    }
    println!("R2_UNDECIDABLE {}", system_configuration_witness_is_undecidable(&per_device));
    let mut current_image = image.clone();
    for attempt in 0..tries {
        let hidden: Rc<RefCell<Vec<(DeviceIdentity, u64, u64)>>> = Rc::new(RefCell::new(hidden_offsets.iter().map(|(d, o)| (DeviceIdentity(*d), *o, *o + 4096)).collect()));
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<_> = recording_of(&current_image, &stream).into_iter().map(|(id, rec)| (id, HidingDevice { device: id, inner: rec, hidden: hidden.clone() })).collect();
        let outcome = mount_writable(&parameters, &mut devices);
        let after = { let mut i = current_image.clone(); i.apply(&stream.retained_operations()); i };
        match &outcome {
            Ok(m) => println!("ATTEMPT {attempt} writable=Ok instance={}", m.output.instance.0),
            Err(e) => {
                let text = format!("{e:?}");
                let reread_comparison = text.rfind("comparison: ").map(|at| text[at..].chars().take(90).collect::<String>());
                println!("ATTEMPT {attempt} writable=Err {} reread_{reread_comparison:?} disk_unchanged={}", text.chars().take(120).collect::<String>(), after == current_image);
            }
        }
        drop(devices);
        current_image = after;
    }
    let faulty = Faulty { image: &current_image, hidden: hidden_offsets };
    println!("READ_ONLY {}", match singlefs_core::mounted_read::mount_read_only(&faulty) { Ok(_) => "Ok".to_string(), Err(e) => format!("{e:?}").chars().take(200).collect() });
}
