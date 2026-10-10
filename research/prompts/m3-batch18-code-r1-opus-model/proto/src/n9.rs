//! N9：E142 导出 bin 那条路（`singlefs_harness::scenario::run_new_pool_file_creation`，与 bin 同一个入口、同一组参数）在几种每盘字节数上各跑一遍，
//! 窗口（新池新建文件那一段）里每一个写的 (盘, 偏移, 长度, 内容) 与 4 GiB 那一遍逐个比：哪几个随盘大小变、哪几个不变、窗口形状变不变。
//! 不起 checker 档的 bin（直接执行它属于重型），走的是 bin 调的同一个库函数。
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::{e142_parameters, run_new_pool_file_creation, ScenarioPoint};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

type Window = Vec<(u32, u64, u64, Vec<u8>)>;

fn window_at(device_bytes: u64) -> Result<Window, String> {
    let parameters = e142_parameters(512, 512);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|number| {
            let identity = DeviceIdentity(number);
            (identity, RecordingBlockDevice::with_shared_stream(identity, SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)), stream.clone()))
        })
        .collect();
    let mut window_start = None;
    run_new_pool_file_creation(&parameters, &mut devices, &stream, |point, _devices| {
        if let ScenarioPoint::BeforeNewPoolFileCreation = point {
            window_start = Some(stream.operation_count());
        }
    })
    .map_err(|failure| failure.render())?;
    let start = window_start.ok_or("没走到 BeforeNewPoolFileCreation")?;
    let operations = stream.retained_operations();
    let mut window = Vec::new();
    for retained in &operations[start..] {
        let operation = &retained.operation;
        if operation.kind == RecordedOperationKind::Barrier {
            continue;
        }
        let mut image = singlefs_harness::memory_pool::SparseDevice::default();
        retained_apply(retained, &mut image);
        let bytes = image.read(operation.offset, usize::try_from(operation.length).expect("长度"));
        window.push((operation.device.0, operation.offset.0, operation.length, bytes));
    }
    Ok(window)
}

fn retained_apply(retained: &singlefs_harness::RetainedOperation, image: &mut singlefs_harness::memory_pool::SparseDevice) {
    let mut pool = singlefs_harness::memory_pool::MemoryPool { devices: [(retained.operation.device, image.clone())].into_iter().collect(), device_size_in_bytes: u64::MAX };
    pool.apply(std::slice::from_ref(retained));
    *image = pool.devices.remove(&retained.operation.device).expect("盘");
}

pub fn run() {
    let reference = window_at(4 << 30).expect("4 GiB 那一遍");
    for device_bytes in [3u64 << 30, (4u64 << 30) + 65_536, 6 << 30, 8 << 30, 16 << 30, 64 << 30, (4u64 << 30) + 256, 1 << 30, 0] {
        match window_at(device_bytes) {
            Err(refused) => println!("N9 device_bytes={device_bytes} outcome=refused reason={}", refused.lines().next().unwrap_or("")),
            Ok(window) => {
                let same_shape = window.len() == reference.len()
                    && window.iter().zip(&reference).all(|(a, b)| (a.0, a.1, a.2) == (b.0, b.1, b.2));
                let unequal: Vec<String> = window
                    .iter()
                    .zip(&reference)
                    .filter(|(a, b)| a.3 != b.3)
                    .map(|(a, _)| format!("dev{}@{}+{}", a.0, a.1, a.2))
                    .collect();
                println!(
                    "N9 device_bytes={device_bytes} outcome=ran regions={} same_offsets_and_lengths_as_4GiB={same_shape} regions_with_different_bytes={} differing={unequal:?}",
                    window.len(),
                    unequal.len()
                );
            }
        }
    }
}
