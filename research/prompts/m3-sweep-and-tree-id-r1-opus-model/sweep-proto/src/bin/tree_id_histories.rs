//! S2：几段历史上跑候选判法（T1 全环 / 同线、T2 三种排除、T3 单调）与今天的池级 checker。
//! 用法：tree_id_histories <orphan_ff|applied_ff|rollback|fake_live|fake_freed|c342>
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, Mounted, RollbackTarget};
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolWriter, TransactionOutput};
use singlefs_format::{DATA_UNIT_PAYLOAD_OFFSET, SLOT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;
use sweep_proto::*;

fn report(tag: &str, pool: &MemoryPool) {
    let (roots, _) = ring_roots(pool);
    let mut r: Vec<(u32, u64, u64)> = roots.iter().map(|r| (r.instance.0, r.checkpoint_txg.0, r.tree_identifier_watermark)).collect();
    r.sort_by_key(|x| x.1);
    println!("{tag} ring={r:?}");
    println!("{tag} {}", summarize(&judge_tree_identifiers(pool)));
    println!("{tag} checker_red={:?}", violated(pool));
}

fn first_file(parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters, devices: &mut Devices, m: &mut Mounted, bytes: &[u8]) -> TransactionOutput {
    let inst = m.output.instance;
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    publish_first_file(&mut pool, &mut m.allocator, m.current.root(), FirstFile { content: bytes, write_time_seconds: WRITE_TIME }, inst, m.current.record_bytes()).expect("第一个文件")
}

fn main() {
    let arm = std::env::args().nth(1).expect("臂");
    let parameters = parameters(4);
    let g = geometry(&parameters);
    let mut devices = fresh_devices();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut m1 = mount_writable(&parameters, &mut devices).expect("挂载 1");
    let i1 = m1.output.instance.0;
    match arm.as_str() {
        "orphan_ff" | "applied_ff" => {
            // 第一个文件那次发布切在第一条记录写之前（orphan_ff）或根槽写之前（applied_ff）。
            let before = pool_of(&devices);
            let stream = SharedStream::retaining_contents();
            let mut rec = recording_of(&before, &stream);
            {
                let mut pool = PoolWriter::new(&parameters, rec.as_mut_slice());
                let _ = publish_first_file(&mut pool, &mut m1.allocator.clone(), m1.current.root(), FirstFile { content: &content(0, 3000), write_time_seconds: WRITE_TIME }, instance(i1), m1.current.record_bytes()).expect("第一个文件");
            }
            let ops = stream.retained_operations();
            let kind = if arm == "orphan_ff" { StepKind::JournalRecord } else { StepKind::RootRecordFua };
            let cut = ops.iter().position(|o| g.classify(&o.operation) == kind).expect("切点");
            let mut image = before;
            image.apply(&ops[..cut]);
            report("CRASHED", &image);
            let mut devices = devices_of(&image);
            let mut m2 = mount_writable(&parameters, &mut devices).expect("挂载 2");
            println!("MOUNT2 rows={:?} effective=({},{})", m2.output.rows_written.iter().map(|r| (r.instance.0, r.selected_root_txg.0)).collect::<Vec<_>>(), m2.output.effective_root.instance.0, m2.output.effective_root.checkpoint_txg.0);
            if m2.current.file_version().is_none() {
                let ff = first_file(&parameters, &mut devices, &mut m2, &content(1, 3000));
                println!("REISSUE first_file=({},{}) wm={}", ff.root.instance.0, ff.root.checkpoint_txg.0, ff.root.tree_identifier_watermark);
                let i2 = m2.output.instance.0;
                let _ = overwrite_with(&parameters, &mut devices, &mut m2.allocator, &ff, &content(2, 3000), 2, i2);
            }
            report("AFTER", &pool_of(&devices));
        }
        "rollback" => {
            let mut cur = first_file(&parameters, &mut devices, &mut m1, &content(0, 3000));
            let target = RollbackTarget { instance: cur.root.instance, checkpoint_txg: cur.root.checkpoint_txg };
            for step in 1..=3 {
                cur = overwrite_with(&parameters, &mut devices, &mut m1.allocator, &cur, &content(step, 3000), step, i1);
            }
            let r = roll_back_by_a_forward_publish(&parameters, &mut devices, &mut m1.allocator, &mut cur, target).expect("回退");
            println!("ROLLBACK to=({},{}) new=({},{})", target.instance.0, target.checkpoint_txg.0, cur.root.instance.0, cur.root.checkpoint_txg.0);
            let _ = r;
            let _ = overwrite_with(&parameters, &mut devices, &mut m1.allocator, &cur, &content(9, 3000), 9, i1);
            report("AFTER", &pool_of(&devices));
        }
        "fake_live" | "fake_freed" => {
            let pool = pool_of(&devices);
            let ((dev, slot), _) = every_unit_header(&pool).into_iter().find(|(_, h)| h.class == 2).expect("码 2");
            let head = read_slot_head(&pool, DeviceIdentity(dev), slot, 4096);
            let header_end = header_end_of(&head).expect("头");
            let k = 2 * usize::from(head[51]);
            let mut fake = head[..header_end].to_vec();
            fake[42..50].copy_from_slice(&(1u64 << 40).to_le_bytes());
            fake[52 + k..60 + k].copy_from_slice(&1u64.to_le_bytes());
            fake[68 + k..72 + k].copy_from_slice(&i1.to_le_bytes());
            reseal_header(&mut fake, header_end);
            let mut file = content(1, 20_000);
            let at = usize::try_from(SLOT_BYTES - DATA_UNIT_PAYLOAD_OFFSET).expect("小");
            file[at..at + header_end].copy_from_slice(&fake);
            let mut cur = first_file(&parameters, &mut devices, &mut m1, &file);
            report("LIVE", &pool_of(&devices));
            if arm == "fake_freed" {
                for step in 1..=60u64 {
                    // 内容只占一个槽的数据单元交替写：被释放的两槽单元的第一个槽有机会被一槽的节点拿走。
                    cur = overwrite_with(&parameters, &mut devices, &mut m1.allocator, &cur, &content(step, 3000), step, i1);
                    let pool = pool_of(&devices);
                    let fakes: Vec<(u32, u64)> = every_unit_header(&pool).into_iter().filter(|(_, h)| h.class == 2 && h.tree == 1 << 40).map(|(k, _)| k).collect();
                    let claimed = claim_rule_scan(&pool);
                    let visible = fakes.iter().filter(|k| claimed.contains_key(k)).count();
                    let referenced = slots_referenced_by_any_ring_root(&pool);
                    let still_ref = fakes.iter().filter(|k| referenced.contains(k)).count();
                    if step % 5 == 0 || visible > 0 {
                        println!("STEP {step} txg={} fakes_on_disk={} visible_under_claim_rule={visible} fakes_in_slots_referenced_by_a_ring_root={still_ref}", cur.root.checkpoint_txg.0, fakes.len());
                    }
                    if visible > 0 {
                        report(&format!("STEP{step}"), &pool);
                        break;
                    }
                }
            }
        }
        "c342" => {
            // 实例 1：第一个文件（txg 3 发号）之后覆盖写到 txg 10；把这几次发布的根槽写与记录写全部抹成零（多重介质故障，读不出），
            // 实例 2 落回 txg 2 那一版、重发同一批树 ID；之后把没被实例 2 盖掉的那些根槽与记录字节放回去（故障消失）。
            let before = pool_of(&devices);
            let stream = SharedStream::retaining_contents();
            let mut rec = recording_of(&before, &stream);
            {
                let mut pool = PoolWriter::new(&parameters, rec.as_mut_slice());
                let mut cur = publish_first_file(&mut pool, &mut m1.allocator, m1.current.root(), FirstFile { content: &content(0, 3000), write_time_seconds: WRITE_TIME }, instance(i1), m1.current.record_bytes()).expect("第一个文件");
                drop(pool);
                for step in 1..=7u64 {
                    let mut pool = PoolWriter::new(&parameters, rec.as_mut_slice());
                    cur = singlefs_core::transaction::publish_overwrite(&mut pool, &mut m1.allocator, &cur, FirstFile { content: &content(step, 3000), write_time_seconds: WRITE_TIME + step }, instance(i1)).expect("覆盖写");
                }
                println!("INSTANCE1 last=({},{}) wm={}", cur.root.instance.0, cur.root.checkpoint_txg.0, cur.root.tree_identifier_watermark);
            }
            let ops = stream.retained_operations();
            let mut image = before.clone();
            image.apply(&ops);
            let hidden: Vec<(DeviceIdentity, u64, Vec<u8>)> = ops.iter().filter(|o| matches!(g.classify(&o.operation), StepKind::RootRecordFua | StepKind::JournalRecord)).filter_map(|o| {
                let bytes = o.contents.clone()?;
                Some((o.operation.device, o.operation.offset.0, bytes))
            }).collect();
            for (device, offset, bytes) in &hidden {
                image.devices.get_mut(device).expect("盘").write(DeviceOffsetInBytes(*offset), &vec![0u8; bytes.len()]);
            }
            // 系统配置也退回实例 1 发第一个文件之前那一份（否则乙-配置续的见证在取号之前拒可写）：系统配置槽写原样换回 before 里的字节。
            let mut sysconf_rolled_back = 0;
            if std::env::var("C342_ROLL_BACK_SYSCONF").is_ok_and(|v| v == "1") {
                for o in ops.iter().filter(|o| g.classify(&o.operation) == StepKind::SystemConfigurationSlot) {
                    let len = usize::try_from(o.operation.length).expect("长");
                    let old = before.devices[&o.operation.device].read(o.operation.offset, len);
                    image.devices.get_mut(&o.operation.device).expect("盘").write(o.operation.offset, &old);
                    sysconf_rolled_back += 1;
                }
            }
            println!("HIDE root_and_record_writes={} system_configuration_writes_rolled_back={sysconf_rolled_back}", hidden.len());
            report("HIDDEN", &image);
            let mut devices = devices_of(&image);
            let mut m2 = mount_writable(&parameters, &mut devices).expect("挂载 2");
            println!("MOUNT2 rows={:?} effective=({},{})", m2.output.rows_written.iter().map(|r| (r.instance.0, r.selected_root_txg.0)).collect::<Vec<_>>(), m2.output.effective_root.instance.0, m2.output.effective_root.checkpoint_txg.0);
            let ff = first_file(&parameters, &mut devices, &mut m2, &content(1, 3000));
            println!("REISSUE first_file=({},{}) wm={}", ff.root.instance.0, ff.root.checkpoint_txg.0, ff.root.tree_identifier_watermark);
            let extra: u64 = std::env::var("C342_EXTRA").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
            let i2 = m2.output.instance.0;
            let mut cur2 = ff;
            for step in 0..extra {
                cur2 = overwrite_with(&parameters, &mut devices, &mut m2.allocator, &cur2, &content(40 + step, 3000), 40 + step, i2);
            }
            println!("INSTANCE2 last=({},{})", cur2.root.instance.0, cur2.root.checkpoint_txg.0);
            let mut image = pool_of(&devices);
            let mut restored = 0;
            for (device, offset, bytes) in &hidden {
                let now = image.devices[device].read(DeviceOffsetInBytes(*offset), bytes.len());
                if now.iter().all(|b| *b == 0) {
                    image.devices.get_mut(device).expect("盘").write(DeviceOffsetInBytes(*offset), bytes);
                    restored += 1;
                }
            }
            println!("RESTORED {restored}/{}", hidden.len());
            report("AFTER", &image);
            let _ = CheckpointTxg(0);
        }
        _ => panic!("臂"),
    }
}
