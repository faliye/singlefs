//! K3 探针（正推腿草稿，不入库）：只做过 mkfs 的池，一块盘的一个系统配置槽是空白（读得出、自证不过）时，今天的实现怎么走。
mod common;
use common::{format_pool, parameters};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::transaction::{acquire_instance, PoolWriter};

fn blank(devices: &mut Vec<(DeviceIdentity, common::Recorded)>, who: u32, slot: u64) {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let (_, d) = devices.iter_mut().find(|(i, _)| *i == DeviceIdentity(who)).unwrap();
    d.write_at(DeviceOffsetInBytes(slot * spacing), &vec![0u8; 4096], WriteDurability::Plain).unwrap();
}

#[test]
fn k3_probe_mkfs_only_pool_with_one_blank_system_configuration_slot() {
    for (who, slot) in [(0u32, 0u64), (0, 1), (1, 0), (1, 1)] {
        let mut pool = format_pool(&format!("k3-probe-{who}-{slot}"));
        let mut devices = pool.reopen_recorded();
        blank(&mut devices, who, slot);
        let image = pool.memory_pool();
        let _ = image;
        let outcome = mount_writable(&parameters(), &mut devices);
        println!("K3 mount_writable dev{who} slot{slot} blank: {:?}", outcome.as_ref().map(|_| "Ok").map_err(|e| format!("{e:?}")));
        // 同一形上，起点段里的取号（mkfs 同一个进程）
        let params = parameters();
        let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
        let acquired = acquire_instance(&mut writer);
        println!("K3 acquire_instance dev{who} slot{slot} blank: {:?}", acquired.as_ref().map(|i| i.0).map_err(|e| format!("{e:?}")));
        pool.devices = Some(devices);
    }
    // 对照：没有损坏
    let mut pool = format_pool("k3-probe-control");
    let mut devices = pool.reopen_recorded();
    let outcome = mount_writable(&parameters(), &mut devices);
    println!("K3 control mount_writable: {:?}", outcome.as_ref().map(|_| "Ok").map_err(|e| format!("{e:?}")));
    pool.devices = Some(devices);
    let _ = mount_read_only;
}
