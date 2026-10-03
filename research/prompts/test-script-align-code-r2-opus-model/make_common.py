"""从冻结副本的 harness 测试公共模块生成原型用的那一份：盘换成内存稀疏盘（SparseBlockDevice），重开按录制流在内存里重建，
不在磁盘上建镜像文件。用法：python3 make_common.py <仓副本根> <输出文件>"""
import sys
tree, out = sys.argv[1], sys.argv[2]
s = open(f"{tree}/crates/singlefs-harness/tests/common/mod.rs", encoding="utf-8").read()
def once(old, new):
    global s
    assert s.count(old) == 1, old
    s = s.replace(old, new, 1)
once('#![allow(dead_code, reason = "每个测试文件各自只用到其中一部分")]',
     '#![allow(dead_code, unused_imports, reason = "原型：每个程序只用到其中一部分")]')
once("pub type Recorded = RecordingBlockDevice<FileBackedBlockDevice>;",
     "// 原型改动：盘换成内存稀疏盘\npub type Recorded = RecordingBlockDevice<singlefs_harness::memory_pool::SparseBlockDevice>;")
old_reopen = s[s.index("fn reopen_recorded_images("):s.index("/// 按路径重开镜像，不录。")]
once(old_reopen, '''fn reopen_recorded_images(
    paths: &[PathBuf],
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, Recorded)> {
    // 原型改动：按录制流把整份镜像在内存里重建（与文件镜像同一份字节），外面接着录进同一条流
    let _ = paths;
    let pool = memory_pool_of(&stream.retained_operations());
    pool.devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = singlefs_harness::memory_pool::SparseBlockDevice::new(
                IMAGE_BYTES,
                PhysicalBlockSizeInBytes(512),
            );
            device.image = sparse.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect()
}

''')
once('''        let file = FileBackedBlockDevice::create_image_file_exclusively(
            &path,
            IMAGE_BYTES,
            PhysicalBlockSizeInBytes(512),
        )
        .expect("建镜像");''', '''        let file = singlefs_harness::memory_pool::SparseBlockDevice::new(
            IMAGE_BYTES,
            PhysicalBlockSizeInBytes(512),
        );''')
open(out, "w", encoding="utf-8").write(s)
