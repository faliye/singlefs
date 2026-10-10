//! 合法的「取号跳过」（K1 类）：mkfs → 实例 1 写到 txg `last` → 实例 2 的可写挂载崩在写行那次根槽写之前（记录已落、根槽没落）
//! → 实例 3 可写挂载（不施加实例 2 那条记录，取号越过它）→ 覆盖写 `k` 次，每次之后每组开关各判一次。
//! 写法取自 `research/prompts/m3-root-ring-code-r2-opus-model/proto/src/bin/skipped_instance_rows.rs` 的 row 那一格。
//! 用法：skip_row <S> <journal 环字节> <last> <k>
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
    let (devices, _, _) = build_plain(&parameters, last);
    let image = pool_of(&devices);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&image, &stream);
    let mounted = mount_writable(&parameters, &mut recording).expect("实例 2");
    println!("INSTANCE_2 instance={} row_txg={}", mounted.output.instance.0, mounted.output.row_publish.root().checkpoint_txg.0);
    drop(recording);
    let (writes, _) = writes_and_segments(&stream.retained_operations(), &geo);
    let cut = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("根槽写");
    let mut crashed = image.clone();
    crashed.apply_writes(&writes[..cut]);
    let mut devices = devices_of(&crashed);
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 3");
    println!("INSTANCE_3 instance={} row_txg={}", mounted.output.instance.0, mounted.output.row_publish.root().checkpoint_txg.0);
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted.current.clone().into_file_version().expect("有文件");
    show_all(&pool_of(&devices), "k=0");
    for step in 1..=k {
        current = overwrite(&parameters, &mut devices, &mut allocator, &current);
        show_all(&pool_of(&devices), &format!("k={step} newest={}", current.root.checkpoint_txg.0));
    }
}
