//! m4-validate-code-r2 攻方腿：HEAD（841b3beab，隔离记号那一批之前）那一版的读者读这一批写者写出的镜像（只在 HEAD 的拷贝里跑，不进仓）。
//! 镜像由冻结副本拷贝里的 `t5_dump_images_for_the_old_readers` 落进 `ATTACK_IMAGE_DUMP_DIR`：每条是 设备 4 + 扇区号 8 + 512 字节。
mod common;

use std::panic::{catch_unwind, AssertUnwindSafe};

use common::parameters;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{allocation_records_under_root, choose_root, choose_system_configuration, recover, walk_to_file, JournalPolicy};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

fn short(text: impl std::fmt::Debug) -> String {
    format!("{text:?}").chars().take(200).collect()
}

fn load(path: &std::path::Path) -> MemoryPool {
    let bytes = std::fs::read(path).expect("读镜像");
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], 4 << 30);
    for record in bytes.chunks(4 + 8 + 512) {
        let device = DeviceIdentity(u32::from_le_bytes(record[0..4].try_into().expect("4")));
        let sector = u64::from_le_bytes(record[4..12].try_into().expect("8"));
        image.devices.get_mut(&device).expect("盘").write(DeviceOffsetInBytes(sector * 512), &record[12..]);
    }
    image
}

fn probe(name: &str, image: &MemoryPool) {
    let guarded = |f: &dyn Fn() -> String| catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|_| "panic".to_string());
    let writable = guarded(&|| {
        let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
            .devices
            .iter()
            .map(|(identity, sparse)| {
                let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
                device.image = sparse.clone();
                (*identity, device)
            })
            .collect();
        mount_writable(&parameters(), &mut devices).map(|_| "Ok".to_string()).unwrap_or_else(|refusal| format!("Err({})", short(refusal)))
    });
    let read_only = guarded(&|| mount_read_only(image).map(|_| "Ok".to_string()).unwrap_or_else(|refusal| format!("Err({})", short(refusal))));
    let tree = guarded(&|| {
        let configuration = choose_system_configuration(image).expect("系统配置");
        let root = choose_root(image, &configuration).expect("根");
        allocation_records_under_root(image, &root).map(|records| format!("Ok({} 条)", records.len())).unwrap_or_else(|failure| format!("Err({})", short(failure)))
    });
    let walk = guarded(&|| {
        let configuration = choose_system_configuration(image).expect("系统配置");
        let root = choose_root(image, &configuration).expect("根");
        let mut fallbacks = 0;
        walk_to_file(image, &root, &mut fallbacks).map(|content| format!("Ok({:?} 字节)", content.map(|bytes| bytes.len()))).unwrap_or_else(|failure| format!("Err({})", short(failure)))
    });
    let recovered = guarded(&|| short(recover(image, JournalPolicy::Consult).outcome).chars().take(120).collect());
    let checker = guarded(&|| {
        check_pool_image(image)
            .iter()
            .filter_map(|(name, verdict)| match verdict {
                InvariantVerdict::Violated(why) => Some(format!("{name}:{}", why.chars().take(160).collect::<String>())),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" | ")
    });
    eprintln!("ATTACK-T5旧读者 {name}：可写挂载={writable} 只读挂载={read_only} 读分配记录树={tree} 冷走读={walk} 冷恢复={recovered} 池级checker判红=[{checker}]");
}

#[test]
fn old_readers_read_the_images_written_by_the_quarantine_mark_writer() {
    let directory = std::path::PathBuf::from(std::env::var_os("ATTACK_IMAGE_DUMP_DIR").expect("ATTACK_IMAGE_DUMP_DIR"));
    probe("不带记号的对照", &load(&directory.join("unmarked.sectors")));
    probe("带记号", &load(&directory.join("marked.sectors")));
}
