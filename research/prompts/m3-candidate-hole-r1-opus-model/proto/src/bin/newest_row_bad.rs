//! H1 行证据误认「取号跳过」：mkfs → 实例 1 写到 txg `last` → 实例 2 可写挂载（写行 txg last + 1）→ 覆盖写 `k` 次
//! → 把实例 2 的前 `bad` 条根（txg last + 1 起）的根槽各翻一个字节 → 每组开关各判一次。
//! 用法：newest_row_bad <S> <journal 环字节> <last> <k> <bad>
use hole_attack_proto::{build_plain, corrupt_root_slot, devices_of, overwrite, parameters, pool_of, show_all};
use singlefs_core::mount::mount_writable;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let journal: u64 = a[2].parse().expect("journal");
    let last: u64 = a[3].parse().expect("last");
    let k: u64 = a[4].parse().expect("k");
    let bad: u64 = a[5].parse().expect("bad");
    let parameters = parameters(s, journal);
    let (devices, _, _) = build_plain(&parameters, last);
    let mut devices = devices_of(&pool_of(&devices));
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 2");
    let row_txg = mounted.output.row_publish.root().checkpoint_txg.0;
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted.current.clone().into_file_version().expect("有文件");
    for _ in 0..k {
        current = overwrite(&parameters, &mut devices, &mut allocator, &current);
    }
    let mut image = pool_of(&devices);
    println!("BUILT instance=2 row_txg={row_txg} newest_txg={} bad_writer={:?}", current.root.checkpoint_txg.0, std::env::var("HOLE_BAD_WRITER").ok());
    show_all(&image, "clean");
    let bad_txgs: Vec<u64> = (row_txg..row_txg + bad).collect();
    for txg in &bad_txgs {
        corrupt_root_slot(&mut image, &parameters, *txg);
    }
    show_all(&image, &format!("bad{bad_txgs:?}"));
}
