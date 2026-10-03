//! m3-ring-r1 攻方原型 H2 补充（不是层 0 的枚举域）：带文件的后洞那段历史的全量状态数超过 10⁶、不许全量跑，
//! 这里只取被测挂载那条流的每个前缀截点（写流按次序落到第 c 次写为止），每个截点跑池级 checker，再做下一次可写挂载 + 2 次覆盖写、各跑一次 checker。
//! 打中的截点是可达的崩溃状态（前缀截点是层 0 枚举域的子集）；没打中不算证据。
//! 用法：ring_hole_prefix_cuts <S> <front|back> <挂点 0..5>
use ring_hole_proto::{
    build_prefix, content, devices_of, disk_metrics, geometry, hole_txg, parameters, pool_of,
    recording_of, violated, HolePosition, FILESYSTEM_IDENTIFIER, WRITE_TIME,
};
use singlefs_core::mount::{mount_writable, set_hole_fill_point, HoleFillPoint};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::writes_and_segments;
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let slots_per_region: u64 = arguments[1].parse().expect("S");
    let position = if arguments[2] == "front" { HolePosition::Front } else { HolePosition::Back };
    let point = match arguments[3].as_str() {
        "0" => HoleFillPoint::NotFilled,
        "1" => HoleFillPoint::P1AfterAcquisition,
        "2" => HoleFillPoint::P2BeforeRowRoot,
        "3" => HoleFillPoint::P2AfterRowRoot,
        "4" => HoleFillPoint::P3AfterWarmUp,
        "5" => HoleFillPoint::P2RecordedBeforeRowPublish,
        other => panic!("挂点 {other}"),
    };
    let parameters = parameters(slots_per_region);
    let prefix = build_prefix(slots_per_region, hole_txg(slots_per_region, position));
    set_hole_fill_point(point);
    let base = prefix.image.clone();
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&base, &stream);
    mount_writable(&parameters, &mut recording).expect("被测的可写挂载");
    let operations = stream.retained_operations();
    let (writes, segments) = writes_and_segments(&operations, &geometry(&parameters));
    let segment_of = |index: usize| segments.iter().position(|s| s.contains(&index)).expect("段");
    let name_of = |index: usize| -> String {
        let write = &writes[index];
        if write.kind == StepKind::RootRecordFua {
            if let Some(root) = write.bytes().and_then(|b| RootRecord::parse_slot(b, &FILESYSTEM_IDENTIFIER)) {
                return format!("root{}", root.checkpoint_txg.0);
            }
        }
        write.kind.name().to_string()
    };
    let mut red_cuts = 0u64;
    let mut red_after_mount = 0u64;
    let mut red_in_suffix = 0u64;
    for cut in 0..=writes.len() {
        let mut image = base.clone();
        image.apply_writes(&writes[..cut]);
        let at_crash = violated(&image);
        let crash_metrics = disk_metrics(&image);
        let last = if cut == 0 { "none".to_string() } else { format!("{}@segment{}", name_of(cut - 1), segment_of(cut - 1)) };
        let mut devices = devices_of(&image);
        let (mount_text, after_mount, suffix) = match mount_writable(&parameters, &mut devices) {
            Err(error) => (format!("err:{}", format!("{error:?}").chars().take(50).collect::<String>()), Vec::new(), Vec::new()),
            Ok(next) => {
                let after_mount = violated(&pool_of(&devices));
                let text = format!(
                    "chosen={} effective={} row={}",
                    next.output.chosen_root.checkpoint_txg.0,
                    next.output.effective_root.checkpoint_txg.0,
                    next.output.row_publish.root().checkpoint_txg.0
                );
                let instance = next.output.instance;
                let mut allocator = next.allocator;
                let mut current = next.current.into_file_version().expect("带文件");
                let mut suffix = Vec::new();
                for step in 0..2u64 {
                    let bytes = content(5000 + step);
                    let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                    current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &bytes, write_time_seconds: WRITE_TIME + step }, instance).expect("覆盖写");
                    drop(pool);
                    suffix.extend(violated(&pool_of(&devices)));
                }
                (text, after_mount, suffix)
            }
        };
        red_cuts += u64::from(!at_crash.is_empty());
        red_after_mount += u64::from(!after_mount.is_empty());
        red_in_suffix += u64::from(!suffix.is_empty());
        println!(
            "PROTO name=cut point={point:?} cut={cut} last_write={last} crash_newest={} crash_floor={} crash_violated={at_crash:?} next_mount={mount_text} after_mount_violated={after_mount:?} suffix_violated={suffix:?}",
            crash_metrics.as_ref().map_or(0, |m| m.newest_txg),
            crash_metrics.as_ref().map_or(0, |m| m.floor),
        );
    }
    println!(
        "PROTO name=cuts_summary s={slots_per_region} position={position:?} point={point:?} cuts={} red_at_crash={red_cuts} red_after_next_mount={red_after_mount} red_in_suffix={red_in_suffix}",
        writes.len() + 1
    );
}
