//! m3-root-ring-code-r1 攻方原型 K4：写行之后第一个文件版本、崩在记录已持久根槽没持久；下一次可写挂载由记录施加出这一版。
//! mkfs → 可写挂载（实例 1，写行、暖机）→ 可写挂载（实例 2，写行）→ 第一个文件崩在根槽写之前 → 可写挂载（实例 3）录流，
//! 层 0 全量枚举这条流（每个状态恢复之后跑池级 checker），之后再覆盖写 `after` 次、每次之后盘面跑 checker。
//! 用法：first_file_record_only <S> <after> <enumerate：0|1>
use rr_attack_proto::{content, devices_of, geometry, parameters, pool_of, recording_of, WRITE_TIME};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{publish_first_file, publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{MemoryPool, PublishedVersion, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn red(image: &MemoryPool) -> Vec<String> {
    singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .filter_map(|(name, verdict)| match verdict {
            singlefs_checker::image::InvariantVerdict::Violated(text) => Some(format!("{name}：{}", text.chars().take(140).collect::<String>())),
            _ => None,
        })
        .collect()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let after: u64 = a[2].parse().expect("after");
    let enumerate = a[3] == "1";
    let parameters = parameters(s);
    let geo = geometry(&parameters);
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|n| (DeviceIdentity(n), SparseBlockDevice::new(rr_attack_proto::image_bytes(), PhysicalBlockSizeInBytes(512))))
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    mount_writable(&parameters, &mut devices).expect("实例 1");
    let image = pool_of(&devices);
    let mut devices = devices_of(&image);
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 2");
    let before = pool_of(&devices);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&before, &stream);
    let first = content(1);
    let file_txg;
    {
        let mut allocator = mounted.allocator.clone();
        let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
        file_txg = publish_first_file(&mut pool, &mut allocator, mounted.current.root(), FirstFile { content: &first, write_time_seconds: WRITE_TIME }, mounted.output.instance, mounted.current.record_bytes()).expect("第一个文件").root.checkpoint_txg;
    }
    let operations = stream.retained_operations();
    let (writes, _) = singlefs_harness::memory_pool::writes_and_segments(&operations, &geo);
    let root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("根槽写");
    let mut crashed = before.clone();
    crashed.apply_writes(&writes[..root_index]);
    println!("CRASHED first_file_txg={} instance={} red={:?}", file_txg.0, mounted.output.instance.0, red(&crashed));
    // 实例 3 的可写挂载：录流。
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&crashed, &stream);
    let m3 = mount_writable(&parameters, &mut recording).expect("实例 3");
    let operations = stream.retained_operations();
    let (writes, segments) = singlefs_harness::memory_pool::writes_and_segments(&operations, &geo);
    let mut after_mount = crashed.clone();
    after_mount.apply_writes(&writes);
    println!("MOUNT3 effective_root=({}, {}) row_txg={} red={:?}", m3.output.effective_root.instance.0, m3.output.effective_root.checkpoint_txg.0, m3.output.row_publish.root().checkpoint_txg.0, red(&after_mount));
    if enumerate {
        let count = singlefs_harness::memory_pool::closed_form_state_count(&segments);
        println!("ENUM mount3 segments={} closed_form_state_count={count}", segments.len());
        assert!(count <= 1_000_000, "一次全量不超过约 10^6 个状态");
        let root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("根槽写");
        let versions = vec![
            PublishedVersion { instance: mounted.output.instance, checkpoint_txg: file_txg, content: first.clone() },
            PublishedVersion { instance: m3.output.instance, checkpoint_txg: m3.output.row_publish.root().checkpoint_txg, content: first.clone() },
        ];
        let tally = singlefs_checker_tier::crash::enumerate_layer0_versions(&crashed, &writes, &segments, root_index, &versions);
        println!("ENUM_TALLY states={} oracle_violations={} first_oracle={:?}", tally.states, tally.violations, tally.first_violation.as_deref().map(|t| t.chars().take(160).collect::<String>()));
        let wanted = ["I-3.1", "I-3.10", "I-4.1", "I-3.11", "I-7.4", "I-2.1", "I-8.10"];
        for n in wanted {
            println!("ENUM_INV {n} evaluated={} violated={} not_applicable={}", tally.checker_evaluated_states.get(n).copied().unwrap_or(0), tally.checker_violated_states.get(n).copied().unwrap_or(0), tally.checker_not_applicable_states.get(n).copied().unwrap_or(0));
        }
        let all: Vec<String> = tally.checker_violated_states.iter().map(|(n, c)| format!("{n}:{c}")).collect();
        println!("ENUM_ALL_VIOLATED {all:?}");
    }
    let mut devices = devices_of(&after_mount);
    let mut allocator = m3.allocator.clone();
    let mut current = m3.current.clone().into_file_version().expect("施加出来的那一版有文件");
    for step in 0..after {
        let salt = 300 + step;
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(salt), write_time_seconds: WRITE_TIME + salt }, m3.output.instance).expect("覆盖写");
        println!("AFTER txg={} red={:?}", current.root.checkpoint_txg.0, red(&pool_of(&devices)));
    }
}
