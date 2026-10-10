//! 攻方探针（m3-formal-code-b24-r1 opus）：H2 常数注释里「第 k 次挂载上的第一个文件版本 txg 3k − 1，从 k ≥ 2 起成立」逐个 k 量。
//! 搭法照 `pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs` 的 `mount_writable_repeatedly` 与
//! `publish_the_first_file_on_the_mount`：只做过 mkfs 的池接连可写挂载（每次只写行、暖机），第 k 次挂载之后把盘面拷一份、在拷的那份上写第一个文件；
//! 原盘面不写文件，接着挂第 k + 1 次（与「重做 mkfs、挂 k 次」逐字节同一条路：挂载只读写盘面，不读别的状态）。
#[path = "../../singlefs-harness/tests/common/mod.rs"]
#[allow(dead_code, unused_imports)]
mod common;

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolVersion, PoolWriter};
use singlefs_format::{ROOT_RING_REGIONS, ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM};
use singlefs_harness::memory_pool::SparseBlockDevice;

fn copy_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copy = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}

#[test]
fn opus_txg_of_the_first_file_on_the_kth_warm_up_only_mount() {
    let max_k: u64 = std::env::var("OPUS_H2_MAX_K").ok().and_then(|v| v.parse().ok()).unwrap_or(40);
    let s = ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM;
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|number| (DeviceIdentity(number), SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512))))
        .collect();
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let mut roots_after_each_mount = Vec::new();
    let mut lines = Vec::new();
    for k in 1..=max_k {
        let last = mount_writable(&parameters(), &mut devices).expect("可写挂载");
        roots_after_each_mount.push(last.current.root().checkpoint_txg.0);
        let PoolVersion::WithoutFile(version) = &last.current else {
            panic!("只暖机过的池上现行那一版树表 0 条")
        };
        let mut copy = copy_of(&devices);
        let mut allocator = last.allocator.clone();
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, copy.as_mut_slice());
        let first_file = publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile { content: &file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            last.output.instance,
            &version.record_bytes,
        )
        .expect("树表 0 条的一版上发第一个文件版本");
        let txg = first_file.root.checkpoint_txg.0;
        let formula = 3 * k - 1;
        let oldest = txg.checked_sub(ROOT_RING_REGIONS * s);
        lines.push(format!(
            "h2 S={s} k={k} first_file_txg={txg} three_k_minus_one={formula} holds={} oldest_valid_root_before={oldest:?} current_root_txg_after_this_mount={}",
            txg == formula,
            roots_after_each_mount[roots_after_each_mount.len() - 1]
        ));
    }
    println!("{}", lines.join("\n"));
}
