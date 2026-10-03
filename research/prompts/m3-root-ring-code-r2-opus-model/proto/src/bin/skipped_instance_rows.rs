//! K3 看 8-2 自取读法 ③（「两者之间每个实例的行都是 (j, 0, 0)」读成每个中间实例都要有一行、缺行算接不上）在合法历史上碰不碰得到：
//! mkfs → 实例 1 写到 txg `last` → 实例 2 的可写挂载崩在 `cut` 处（`row`：写行那次根槽写之前；`zero`：一个写都没落）→ 实例 3 的可写挂载
//! 做完，再覆盖写一次。打印最新根的实例表行、候选集 txg 与池级 checker 对 I-3.5 的判定。
//! 用法：skipped_instance_rows <S> <last> <row|zero>
use e2_attack_proto::{build_plain, content, geometry, parameters, recording_of, rollback_targets, WRITE_TIME};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{choose_system_configuration, instance_table_of_root, readable_roots_with_ring_slots};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn show(image: &MemoryPool, tag: &str) {
    let c = choose_system_configuration(image).expect("系统配置");
    let i = &c.immutable;
    let newest = readable_roots_with_ring_slots(image, &i.region_devices, &i.sizes, &i.filesystem_identifier).into_iter().map(|(_, r)| r).max_by_key(|r| (r.checkpoint_txg, r.instance)).expect("有根");
    let table = instance_table_of_root(image, &newest).expect("实例表");
    let verdict = singlefs_checker::walk::check_pool_image(image).into_iter().find(|(n, _)| *n == "I-3.5").map(|(_, v)| format!("{v:?}"));
    println!("{tag} newest=(实例 {}, txg {}) rows={:?} candidates={:?} I-3.5={verdict:?}", newest.instance.0, newest.checkpoint_txg.0, table.rows, rollback_targets(image));
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let last: u64 = a[2].parse().expect("last");
    let cut = a[3].clone();
    let parameters = parameters(s);
    let geo = geometry(&parameters);
    let image = build_plain(s, last);
    show(&image, "AFTER_INSTANCE_1");
    let stream = SharedStream::retaining_contents();
    let mut devices = recording_of(&image, &stream);
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 2");
    println!("INSTANCE_2 instance={} row_txg={}", mounted.output.instance.0, mounted.output.row_publish.root().checkpoint_txg.0);
    drop(devices);
    let (writes, _) = writes_and_segments(&stream.retained_operations(), &geo);
    let k = if cut == "zero" { 0 } else { writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("根槽写") };
    let mut crashed = image.clone();
    crashed.apply_writes(&writes[..k]);
    show(&crashed, "AFTER_INSTANCE_2_CRASH");
    let mut devices = e2_attack_proto::devices_of(&crashed);
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 3");
    let instance = mounted.output.instance;
    println!("INSTANCE_3 instance={} row_txg={}", instance.0, mounted.output.row_publish.root().checkpoint_txg.0);
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted.current.clone().into_file_version().expect("有文件");
    let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
    let salt = current.root.checkpoint_txg.0 + 1;
    current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(salt), write_time_seconds: WRITE_TIME + salt }, instance).expect("覆盖写");
    let _ = current;
    show(&e2_attack_proto::pool_of(&devices), "AFTER_INSTANCE_3");
}
