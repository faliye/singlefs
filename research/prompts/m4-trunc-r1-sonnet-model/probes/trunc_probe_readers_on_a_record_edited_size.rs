//! 草稿探针（m4-trunc-r1 正推腿，不入库）：只改 inode 记录的 size（不重写尾单元）之后，今天的读者与池级 checker 各说什么。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{build_pool, FILE_BYTES};
use common_deletion_in_progress::change_the_inode_record;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{FileOffsetInBytes, InodeNumber};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::unit::data_unit_payload_capacity;

fn edit_and_read(label: &str, new_size: u64, read_length: u64) {
    let pool = build_pool("trunc-probe-readers");
    let mut image = pool.memory_pool();
    change_the_inode_record(&mut image, 1, |record| {
        record[40..48].copy_from_slice(&new_size.to_le_bytes());
        record[48..56].copy_from_slice(&new_size.div_ceil(512).to_le_bytes());
    });
    let violated: Vec<String> = check_pool_image(&image)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| format!("{invariant:?}"))
        .collect();
    println!("{label}: 池级 checker 判违例的不变量 = {violated:?}");
    match mount_read_only(&image) {
        Err(error) => println!("{label}: mount_read_only 拒：{error:?}"),
        Ok(mounted) => match mounted.mounted.open_file(&image, InodeNumber(1)) {
            Err(error) => println!("{label}: open_file 拒：{error:?}"),
            Ok(file) => match file.read_at(&image, FileOffsetInBytes(0), read_length) {
                Err(error) => println!("{label}: read_at 拒：{error:?}"),
                Ok(output) => println!("{label}: read_at 读出 {} 字节", output.bytes.len()),
            },
        },
    }
}

#[test]
fn probe_readers_after_a_record_only_size_change() {
    let file_bytes = u64::try_from(FILE_BYTES).expect("3000");
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    edit_and_read("C 缩到 2990（尾单元没重写）", file_bytes - 10, file_bytes - 10);
    edit_and_read("D 变长到 3010（同一个单元内，尾单元没重写）", file_bytes + 10, 3000);
    edit_and_read("E 变长到三个净荷（尾部是洞）", 3 * payload, 3000);
}
