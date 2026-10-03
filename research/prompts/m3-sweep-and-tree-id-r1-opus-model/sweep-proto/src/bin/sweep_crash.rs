//! S1 Q2 / Q4 / Q5、C333：专用清扫发布（零轮原型）连同之前的抹头，在合法前缀上逐崩溃状态判。
//! 前缀：mkfs → 实例 1 第一个文件、覆盖写 2 次 → 一次顺序写（N 个数据单元）切在第一条记录写之前（实例 1 留下孤儿）
//!       → 可写挂载实例 2（写行 (1, T1, W1)）→ 实例 2 覆盖写到根环里没有实例 1 的根。
//! 清扫：判候选行与它们的未发布单元 → 抹头（FUA）+ 屏障 → 清扫发布（删候选行）。
//! 用法：sweep_crash <S> <N> <臂> <probe|enumerate> [下一次挂载取样步长]
//!   臂 sweep：抹头 + 删行；noerase：不抹头就删行（判别力对照）；carry：抹头 + 照抄实例表的空发布。
use std::collections::BTreeMap;

use singlefs_checker_tier::crash::{enumerate_layer0_selecting_versions_observing_each_state, layer0_state_count_with_torn_in_place_overwrites, full_expansion};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::recovery::instance_table_chain_of_root;
use singlefs_core::transaction::{publish_first_file, publish_overwrite, publish_sequential_write, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, PublishedVersion};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;
use sweep_proto::*;

fn overwrite(parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters, devices: &mut Devices, allocator: &mut singlefs_core::allocator::PoolAllocator, current: &TransactionOutput, salt: u64, inst: u32) -> TransactionOutput {
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    publish_overwrite(&mut pool, allocator, current, FirstFile { content: &content(salt, 3000), write_time_seconds: WRITE_TIME + salt }, instance(inst)).expect("覆盖写")
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let n: usize = a[2].parse().expect("N");
    let arm = a[3].clone();
    let enumerate = a[4] == "enumerate";
    let stride: u64 = a.get(5).map_or(1, |v| v.parse().expect("步长"));
    let parameters = parameters(s);
    let mut devices = fresh_devices();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let Mounted { output, mut allocator, current } = mount_writable(&parameters, &mut devices).expect("挂载 1");
    let i1 = output.instance.0;
    let mut cur = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(&mut pool, &mut allocator, current.root(), FirstFile { content: &content(0, 3000), write_time_seconds: WRITE_TIME }, instance(i1), current.record_bytes()).expect("第一个文件")
    };
    for step in 1..=2 {
        cur = overwrite(&parameters, &mut devices, &mut allocator, &cur, step, i1);
    }
    // 实例 1 的一次顺序写，切在第一条记录写之前：单元都落了、记录与根没落 ⇒ 孤儿。
    let before = pool_of(&devices);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&before, &stream);
    if n > 0 {
        let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
        let _ = publish_sequential_write(&mut pool, &mut allocator.clone(), &cur, FirstFile { content: &content(9, n * 32_634), write_time_seconds: WRITE_TIME + 9 }, instance(i1)).expect("顺序写");
    }
    let ops = stream.retained_operations();
    let g = geometry(&parameters);
    let cut = ops.iter().position(|o| g.classify(&o.operation) == StepKind::JournalRecord).unwrap_or(ops.len());
    let mut image = before;
    image.apply(&ops[..cut]);
    let orphans_written = ops[..cut].iter().filter(|o| g.classify(&o.operation) == StepKind::UnitWrite).count();
    // 实例 2。
    let mut devices = devices_of(&image);
    let m2 = mount_writable(&parameters, &mut devices).expect("挂载 2");
    let i2 = m2.output.instance.0;
    let mut alloc2 = m2.allocator.clone();
    let mut cur2 = m2.current.clone().into_file_version().expect("带文件");
    let rows_after_mount2: Vec<(u32, u64, u64)> = m2.output.rows_written.iter().map(|r| (r.instance.0, r.selected_root_txg.0, r.applied_transaction_high_water)).collect();
    let count_unpublished_of_i1 = |pool: &singlefs_harness::memory_pool::MemoryPool, row_txg: u64, row_w: u64| {
        every_unit_header(pool).values().filter(|h| h.instance == i1 && h.fsid_low == fsid_low() && unpublished_by_row(h, row_txg, row_w)).count()
    };
    let (t1, w1) = (rows_after_mount2[0].1, rows_after_mount2[0].2);
    let orphans_after_mount2 = count_unpublished_of_i1(&pool_of(&devices), t1, w1);
    let mut steps = 0u64;
    loop {
        let (roots, _) = ring_roots(&pool_of(&devices));
        if !roots.iter().any(|r| r.instance.0 == i1) {
            break;
        }
        steps += 1;
        cur2 = overwrite(&parameters, &mut devices, &mut alloc2, &cur2, 100 + steps, i2);
    }
    let base = pool_of(&devices);
    let judgement = judge_sweep(&base, &cur2.root, None);
    let targets: Vec<(u32, u64)> = judgement.unpublished_by_instance.values().flatten().copied().collect();
    println!("PREFIX s={s} n={n} i1={i1} i2={i2} orphan_unit_writes={orphans_written} rows_after_mount2={rows_after_mount2:?} unpublished_of_i1_after_mount2={orphans_after_mount2} overwrites_until_no_i1_root={steps} current=({},{}) every_ring_slot_self_verified={} candidate_rows={:?} unpublished_targets={}",
        cur2.root.instance.0, cur2.root.checkpoint_txg.0, judgement.every_ring_slot_self_verified,
        judgement.candidate_rows.iter().map(|r| (r.instance.0, r.selected_root_txg.0)).collect::<Vec<_>>(), targets.len());
    // 清扫流：抹头 + 清扫发布，录下来。
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&base, &stream);
    if arm != "noerase" {
        erase_headers(&mut recording, &targets);
    }
    let chain = instance_table_chain_of_root(&base, &cur2.root).expect("链");
    let mut alloc_copy = alloc2.clone();
    let swept = sweep_publish(&parameters, &mut recording, &mut alloc_copy, &cur2, judgement.kept_rows.clone(), chain.page_pointers.clone(), arm != "carry").expect("清扫发布");
    let operations = stream.retained_operations();
    let (writes, segments) = writes_and_segments(&operations, &g);
    let root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("根槽写");
    let layer0 = layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &full_expansion);
    println!("STREAM arm={arm} writes={} segments={} segment_lengths={:?} closed_form_states={} layer0_states={layer0} swept_root=({},{}) rows_after={:?}",
        writes.len(), segments.len(), segments.iter().map(Vec::len).collect::<Vec<_>>(), closed_form_state_count(&segments),
        swept.root.instance.0, swept.root.checkpoint_txg.0, rows_of(&{ let mut p = base.clone(); p.apply_writes(&writes); p }, &swept.root).iter().map(|r| (r.instance.0, r.selected_root_txg.0)).collect::<Vec<_>>());
    let content_now0 = content(100 + steps, 3000);
    let versions0 = vec![
        PublishedVersion { instance: instance(i2), checkpoint_txg: cur2.root.checkpoint_txg, content: content_now0.clone() },
        PublishedVersion { instance: instance(i2), checkpoint_txg: swept.root.checkpoint_txg, content: content_now0.clone() },
    ];
    if a[4] == "states" {
        // 手摆的几个状态（不是层 0 枚举）：每个都跑层 0 同一套判定（两种 journal 政策的恢复、oracle、池级 checker、记录核对器），再做一次可写挂载。
        let first_publish_write = writes.iter().position(|w| w.kind != StepKind::UnitWrite || !targets.iter().any(|(d, sl)| w.device.0 == *d && w.offset.0 == sl * singlefs_format::SLOT_BYTES)).unwrap_or(0);
        let erase_count = if arm == "noerase" { 0 } else { targets.len() };
        let sysconf_first = writes.iter().position(|w| w.kind == StepKind::SystemConfigurationSlot).unwrap_or(writes.len());
        let mk = |upto: usize| -> Vec<bool> { (0..writes.len()).map(|i| i < upto).collect() };
        let mut cases: Vec<(String, Vec<bool>)> = vec![
            ("erase_half".into(), mk(erase_count / 2)),
            ("erase_all_publish_none".into(), mk(erase_count)),
            ("record_persisted_root_not".into(), mk(root_index)),
            ("root_persisted_sysconf_not".into(), mk(root_index + 1)),
            ("all".into(), mk(writes.len())),
        ];
        if sysconf_first < writes.len() { cases.push(("one_sysconf".into(), mk(sysconf_first + 1))); }
        println!("STATES first_publish_write={first_publish_write} root_index={root_index} sysconf_first={sysconf_first} kinds={:?}", writes.iter().map(|w| w.kind.name()).collect::<Vec<_>>());
        for (name, persisted) in cases {
            let mut tally = singlefs_checker_tier::crash::Layer0Tally::default();
            let report = singlefs_checker_tier::crash::evaluate_state_for_versions(&base, &writes, persisted.clone(), root_index, &versions0, &mut tally);
            let pool = materialize(&base, &writes, &persisted);
            let left = every_unit_header(&pool).values().filter(|h| h.fsid_low == fsid_low()
                && judgement.candidate_rows.iter().any(|r| r.instance.0 == h.instance && unpublished_by_row(h, r.selected_root_txg.0, r.applied_transaction_high_water))).count();
            let red_crash = violated(&pool);
            let mut next = devices_of(&pool);
            let after = match mount_writable(&parameters, &mut next) {
                Ok(m) => format!("ok chosen=({},{}) effective=({},{}) rows_written={:?} table_rows_after={:?} red_after={:?}",
                    m.output.chosen_root.instance.0, m.output.chosen_root.checkpoint_txg.0, m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0,
                    m.output.rows_written.iter().map(|r| (r.instance.0, r.selected_root_txg.0, r.applied_transaction_high_water)).collect::<Vec<_>>(),
                    rows_of(&pool_of(&next), m.current.root()).iter().map(|r| (r.instance.0, r.selected_root_txg.0)).collect::<Vec<_>>(), violated(&pool_of(&next))),
                Err(e) => format!("err={}", short_err(&e)),
            };
            println!("STATE {name} effective={:?} oracle_violations={} first_violation={:?} checker_violated={:?} red_crash={red_crash:?} leftover_unpublished_of_candidate_rows={left} NEXT_MOUNT {after}",
                report.effective_root.map(|(i, t)| (i.0, t.0)), tally.violations, tally.first_violation, tally.checker_violated_states);
        }
        return;
    }
    if !enumerate {
        return;
    }
    let content_now = content(100 + steps, 3000);
    let versions = vec![
        PublishedVersion { instance: instance(i2), checkpoint_txg: cur2.root.checkpoint_txg, content: content_now.clone() },
        PublishedVersion { instance: instance(i2), checkpoint_txg: swept.root.checkpoint_txg, content: content_now.clone() },
    ];
    let mut ordinal = 0u64;
    let mut next_mount: BTreeMap<String, u64> = BTreeMap::new();
    let mut leftover_orphans_with_row_gone = 0u64;
    let mut first_bad: Option<String> = None;
    let deleted: Vec<u32> = judgement.candidate_rows.iter().map(|r| r.instance.0).collect();
    let mut observe = |image: &singlefs_harness::memory_pool::CrashImage<'_>, report: &singlefs_core::recovery::RecoveryReport| {
        ordinal += 1;
        if ordinal % stride != 0 {
            return;
        }
        let pool = materialize(image.base, image.writes, &image.persisted);
        // 行被删的那一版是有效根时，被删实例的未发布单元（按删之前的行判）还在不在。
        if let Some((_, txg)) = report.effective_root {
            if txg == swept.root.checkpoint_txg && arm != "carry" {
                let left = every_unit_header(&pool).values().filter(|h| deleted.contains(&h.instance) && h.fsid_low == fsid_low()
                    && judgement.candidate_rows.iter().any(|r| r.instance.0 == h.instance && unpublished_by_row(h, r.selected_root_txg.0, r.applied_transaction_high_water))).count();
                if left > 0 { leftover_orphans_with_row_gone += 1; }
            }
        }
        let mut devices = devices_of(&pool);
        let key = match mount_writable(&parameters, &mut devices) {
            Ok(m) => {
                let red = violated(&pool_of(&devices));
                let rows: Vec<u32> = m.output.rows_written.iter().map(|r| r.instance.0).collect();
                let table_rows = rows_of(&pool_of(&devices), m.current.root()).len();
                let k = format!("ok effective=({},{}) rows_written={rows:?} table_rows={table_rows} red_after={red:?}", m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0);
                if !red.is_empty() && first_bad.is_none() { first_bad = Some(format!("ordinal={ordinal} {k}")); }
                k
            }
            Err(e) => {
                let k = format!("err={}", short_err(&e));
                if first_bad.is_none() { first_bad = Some(format!("ordinal={ordinal} {k}")); }
                k
            }
        };
        *next_mount.entry(key).or_insert(0) += 1;
    };
    let tally = enumerate_layer0_selecting_versions_observing_each_state(&base, &writes, &segments, root_index, &versions, &|_, _| true, &mut observe);
    println!("ENUM states={} violations={} first_violation={:?} checker_violated_states={:?} checker_first={:?}", tally.states, tally.violations, tally.first_violation, tally.checker_violated_states, tally.checker_first_violation);
    println!("NEXT_MOUNT_SAMPLED stride={stride} sampled={} leftover_unpublished_with_row_deleted_states={leftover_orphans_with_row_gone} first_bad={first_bad:?}", next_mount.values().sum::<u64>());
    for (k, v) in &next_mount {
        println!("NEXT_MOUNT {v} {k}");
    }
}
