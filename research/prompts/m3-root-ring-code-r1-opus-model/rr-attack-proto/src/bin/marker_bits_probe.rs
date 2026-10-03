//! m3-root-ring-code-r1 攻方原型 K1：新根段卸载记号「位 0 之外有位为 1 当记录损坏」在读者 / 恢复 / 池级 checker 三处是不是一个口径。
//! 造法：mkfs → 可写挂载 → 第一个文件 → 覆盖写到 P；txg P+1 的覆盖写崩在根槽写之前（记录已持久、根槽没持久）。
//! 臂 clean：照原样；臂 bit1：把这次发布每一条 journal 记录（两块盘各一份）偏移 459..463 那 4 字节改成 `marker`、重封整条的头部校验和。
//! 每臂打印：core 的 JournalRecord::parse 收不收、checker 的 check_journal_record 收不收、core 的恢复走到哪一版、池级 checker 违例、
//! 之后一次可写挂载的结局与它之后盘面上 checker 的违例。
//! 用法：marker_bits_probe <P> <marker 十六进制，如 2>
use rr_attack_proto::{build_plain, content, devices_of, geometry, parameters, pool_of, recording_of, FILESYSTEM_IDENTIFIER, WRITE_TIME};
use singlefs_core::journal::{JournalRecord, JOURNAL_HEADER_CHECKSUM_OFFSET};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{MemoryPool, WrittenContents};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn red(image: &MemoryPool) -> Vec<String> {
    singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .filter_map(|(name, verdict)| match verdict {
            singlefs_checker::image::InvariantVerdict::Violated(text) => Some(format!("{name}：{}", text.chars().take(120).collect::<String>())),
            _ => None,
        })
        .collect()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let p: u64 = a[1].parse().expect("P");
    let marker = u32::from_str_radix(&a[2], 16).expect("marker");
    let parameters = parameters(4);
    let geo = geometry(&parameters);
    let (image, _) = build_plain(4, p);
    let mut devices = devices_of(&image);
    let singlefs_core::mount::Mounted { mut allocator, current, .. } = mount_writable(&parameters, &mut devices).expect("重开");
    let current = current.into_file_version().expect("有文件");
    let before = pool_of(&devices);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&before, &stream);
    let crashed_txg;
    {
        let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
        crashed_txg = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(99), write_time_seconds: WRITE_TIME + 99 }, current.root.instance).expect("被截的那次").root.checkpoint_txg.0;
    }
    let operations = stream.retained_operations();
    let (writes, _) = singlefs_harness::memory_pool::writes_and_segments(&operations, &geo);
    let root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("根槽写");
    let fsid_low = singlefs_core::unit::unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER);
    for arm in ["clean", "bit1"] {
        let mut prefix: Vec<_> = writes[..root_index].to_vec();
        let mut parse_core = Vec::new();
        let mut parse_checker = Vec::new();
        for w in prefix.iter_mut().filter(|w| w.kind == StepKind::JournalRecord) {
            let WrittenContents::Bytes(bytes) = &mut w.contents else { continue };
            if arm == "bit1" {
                bytes[459..463].copy_from_slice(&marker.to_le_bytes());
                let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(bytes, 4096, JOURNAL_HEADER_CHECKSUM_OFFSET);
                bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
            }
            parse_core.push(JournalRecord::parse(bytes, fsid_low).is_some());
            parse_checker.push(singlefs_checker::check_journal_record(bytes, fsid_low).is_ok());
        }
        let mut crashed = before.clone();
        crashed.apply_writes(&prefix);
        let report = recover(&crashed, JournalPolicy::Consult);
        println!("ARM {arm} crashed_txg={crashed_txg} core_parse_accepts={parse_core:?} checker_parse_accepts={parse_checker:?} recovery_effective_root={:?} checker_red={:?}", report.effective_root.map(|(i, t)| (i.0, t.0)), red(&crashed));
        let mut devices = devices_of(&crashed);
        match mount_writable(&parameters, &mut devices) {
            Ok(m) => {
                let after = pool_of(&devices);
                println!("ARM {arm} mount=Ok effective_root=({}, {}) row_txg={} after_checker_red={:?}", m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0, m.output.row_publish.root().checkpoint_txg.0, red(&after));
            }
            Err(e) => println!("ARM {arm} mount=Err {}", format!("{e:?}").chars().take(200).collect::<String>()),
        }
    }
}
