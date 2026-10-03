//! m3-code-r2 攻方原型（Opus），Y2-③：长度 0 的写零在文件后端、`O_DIRECT` 后端（两臂）、内存稀疏盘上逐个对拍，
//! 偏移取在设备里的、末尾、越过末尾、没对齐、溢出 u64 的。只打印每格的结局，再统一比。

use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DirectInputOutputBlockDevice, FileBackedBlockDevice,
    PageCachePolicy, PhysicalBlockSizeInBytes, PhysicalBlockSizeSource, WriteDurability,
};
use singlefs_harness::memory_pool::SparseBlockDevice;

const DEVICE_BYTES: u64 = 1 << 20;
const DIRT: u8 = 0x5A;

struct Removed(std::path::PathBuf);
impl Drop for Removed {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn verdict(result: Result<(), BlockDeviceError>) -> String {
    match result {
        Ok(()) => "Ok".to_string(),
        Err(BlockDeviceError::OutOfRange { .. }) => "OutOfRange".to_string(),
        Err(BlockDeviceError::Unaligned { .. }) => "Unaligned".to_string(),
        Err(other) => format!("Other({other:?})"),
    }
}

fn whole(device: &dyn BlockDevice) -> Vec<u8> {
    let mut bytes = vec![0u8; usize::try_from(DEVICE_BYTES).expect("usize")];
    device
        .read_at(DeviceOffsetInBytes(0), &mut bytes)
        .expect("读回整块盘");
    bytes
}

#[test]
fn r2opus_zero_length_zero_fill_on_three_backends() {
    let offsets = [
        0,
        512,
        DEVICE_BYTES / 2,
        DEVICE_BYTES - 512,
        DEVICE_BYTES,
        DEVICE_BYTES + 512,
        4 * DEVICE_BYTES,
        1,
        256,
        u64::MAX - 511,
        u64::MAX,
    ];
    let lengths = [0u64, 512];
    let dirt = vec![DIRT; usize::try_from(DEVICE_BYTES).expect("usize")];
    let pid = std::process::id();
    let file_path = std::env::temp_dir().join(format!("r2opus-zero-length-file-{pid}.img"));
    let _file_removed = Removed(file_path.clone());
    let mut file_device = FileBackedBlockDevice::create_image_file_exclusively(
        &file_path,
        DEVICE_BYTES,
        PhysicalBlockSizeInBytes(512),
    )
    .expect("建文件后端镜像");
    file_device
        .write_at(DeviceOffsetInBytes(0), &dirt, WriteDurability::Plain)
        .expect("填脏");
    let mut sparse = SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
    sparse
        .write_at(DeviceOffsetInBytes(0), &dirt, WriteDurability::Plain)
        .expect("填脏");
    let mut directs = Vec::new();
    let mut guards = Vec::new();
    for (index, policy) in [
        PageCachePolicy::BypassWithDirectInputOutput,
        PageCachePolicy::GoThroughPageCache,
    ]
    .into_iter()
    .enumerate()
    {
        let path = std::env::temp_dir().join(format!("r2opus-zero-length-direct{index}-{pid}.img"));
        guards.push(Removed(path.clone()));
        std::fs::write(&path, &dirt).expect("建镜像");
        directs.push((
            policy,
            DirectInputOutputBlockDevice::open_existing(
                &path,
                policy,
                PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(512)),
            )
            .expect("打开"),
        ));
    }
    let mut disagreements = 0u32;
    let mut cells = 0u32;
    for &length in &lengths {
        for &offset in &offsets {
            let file_before = whole(&file_device);
            let file_counts_before = file_device.zero_fill_landing_counts();
            let file_verdict = verdict(file_device.write_zeroes_at(DeviceOffsetInBytes(offset), length));
            let file_changed = whole(&file_device) != file_before;
            let file_counts_after = file_device.zero_fill_landing_counts();
            let sparse_before = whole(&sparse);
            let sparse_verdict = verdict(sparse.write_zeroes_at(DeviceOffsetInBytes(offset), length));
            let sparse_changed = whole(&sparse) != sparse_before;
            let mut line = format!(
                "H5 offset={offset} length={length} file={file_verdict}/changed={file_changed}/counts {:?}->{:?} sparse={sparse_verdict}/changed={sparse_changed}",
                file_counts_before, file_counts_after
            );
            let mut same = file_verdict == sparse_verdict && file_changed == sparse_changed;
            for (policy, direct) in &mut directs {
                let before = whole(&*direct);
                let direct_verdict = verdict(direct.write_zeroes_at(DeviceOffsetInBytes(offset), length));
                let changed = whole(&*direct) != before;
                same &= direct_verdict == file_verdict && changed == file_changed;
                line.push_str(&format!(" {policy:?}={direct_verdict}/changed={changed}"));
                // 还原，下一格从同样的脏盘起
                direct
                    .write_at(DeviceOffsetInBytes(0), &dirt, WriteDurability::Plain)
                    .expect("还原");
            }
            file_device
                .write_at(DeviceOffsetInBytes(0), &dirt, WriteDurability::Plain)
                .expect("还原");
            sparse
                .write_at(DeviceOffsetInBytes(0), &dirt, WriteDurability::Plain)
                .expect("还原");
            cells += 1;
            if !same {
                disagreements += 1;
                line.push_str(" FLAG");
            }
            println!("{line}");
        }
    }
    println!("H5-SUMMARY cells={cells} disagreements={disagreements}");
}
