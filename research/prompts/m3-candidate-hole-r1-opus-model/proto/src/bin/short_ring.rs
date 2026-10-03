//! H2 来源二（短 journal 环上记录被盖掉）：mkfs（journal 环 `journal` 字节）→ 实例 1 写到 txg `last` → 覆盖写 last + 1，崩在根槽写之前
//! （记录已落、根槽没落）→ 实例 2 可写挂载（恢复把 last + 1 由记录施加出来）→ 覆盖写 `k` 次，每次之后每组开关各判一次。
//! 用法：short_ring <S> <journal 环字节> <last> <k>
use hole_attack_proto::{build_plain, devices_of, geometry, overwrite, parameters, pool_of, recording_of, show_all};
use singlefs_core::mount::mount_writable;
use singlefs_harness::memory_pool::writes_and_segments;
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let journal: u64 = a[2].parse().expect("journal");
    let last: u64 = a[3].parse().expect("last");
    let k: u64 = a[4].parse().expect("k");
    let parameters = parameters(s, journal);
    let geo = geometry(&parameters);
    let (devices, mut allocator, current) = build_plain(&parameters, last);
    let image = pool_of(&devices);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&image, &stream);
    let crashed_version = overwrite(&parameters, &mut recording, &mut allocator, &current);
    drop(recording);
    let (writes, _) = writes_and_segments(&stream.retained_operations(), &geo);
    let cut = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("根槽写");
    let mut crashed = image.clone();
    crashed.apply_writes(&writes[..cut]);
    println!("CRASHED txg={} writes={} cut_before_root_fua={cut}", crashed_version.root.checkpoint_txg.0, writes.len());
    let mut devices = devices_of(&crashed);
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 2");
    println!(
        "INSTANCE_2 instance={} row_txg={} bad_writer={:?}",
        mounted.output.instance.0,
        mounted.output.row_publish.root().checkpoint_txg.0,
        std::env::var("HOLE_BAD_WRITER").ok()
    );
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted.current.clone().into_file_version().expect("有文件");
    show_all(&pool_of(&devices), "k=0");
    for step in 1..=k {
        current = overwrite(&parameters, &mut devices, &mut allocator, &current);
        show_all(&pool_of(&devices), &format!("k={step} newest={}", current.root.checkpoint_txg.0));
    }
}
