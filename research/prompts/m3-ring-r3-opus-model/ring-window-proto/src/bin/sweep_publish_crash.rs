//! m3-ring-r3 攻方原型 H4 × H2：第二轮判决采的「删行落在一次专用的清扫发布上」与用户第 1 条选的乙（新根段只加两位、不带实例表指针）
//! 合在一起时，清扫发布崩在「记录已持久、根槽没持久」那一格：清扫发布与所选根同一实例，恢复施加它的记录，
//! 施加出来的根的实例表指针照抄所选根（`recovery.rs` 施加处），与清扫发布当初会写的根不同。
//! 清扫发布在今天的代码里没有：这里按写行那次发布同一个计划形态（`InstanceTablePlan::Rewrite`）拼一次「同一实例、删一行」的空发布。
//! 对照臂 carry：同一位置换成照抄实例表的空发布。
//!
//! 用法：sweep_publish_crash <S> <sweep|carry> <挂载后覆盖写次数 K> <之后覆盖写次数 M> [enumerate]
#![allow(clippy::too_many_lines)]

use ring_window_proto::{build_prefix, content, devices_of, geometry, parameters, pool_of, recording_of, violated, WRITE_TIME};
use singlefs_checker_tier::crash::{enumerate_layer0_versions, layer0_state_count_with_torn_in_place_overwrites, full_expansion};
use singlefs_core::address::CheckpointTxg;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{instance_table_chain_of_root, choose_system_configuration, readable_roots_with_ring_slots};
use singlefs_core::transaction::{publish_overwrite, publish_version, FirstFile, InstanceTablePlan, InstanceTableRewrite, PoolWriter, PublishPlan};
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, PublishedVersion};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let arm = a[2].clone();
    let k: u64 = a[3].parse().expect("K");
    let m: u64 = a[4].parse().expect("M");
    let enumerate = a.get(5).is_some_and(|v| v == "enumerate");
    let parameters = parameters(s);
    // 前缀：K1（实例 2 写行那次崩），挂载实例 3（写行带 (2, 0) 那一行），覆盖写 K 次。
    // SWEEP_PREFIX=plain：不带洞的前缀（mkfs → 挂载 1 → 第一个文件 → 覆盖写到 3S + 4，正常结束），用来对照「洞」是不是起因。
    let plain = std::env::var("SWEEP_PREFIX").is_ok_and(|v| v == "plain");
    if !plain { std::env::set_var("CRASH_THE_ROW_PUBLISH", "1"); }
    let prefix = build_prefix(s, 3 * s + 4);
    std::env::remove_var("CRASH_THE_ROW_PUBLISH");
    let image0 = if plain { ring_window_proto::build_plain(s, 3 * s + 4).0 } else { prefix.image.clone() };
    let mut devices = devices_of(&image0);
    let mounted = mount_writable(&parameters, &mut devices).expect("挂载 3");
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted.current.clone().into_file_version().expect("带文件");
    for step in 0..k {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(600 + step), write_time_seconds: WRITE_TIME + 600 + step }, instance).expect("覆盖写");
    }
    let content_now = content(600 + k.saturating_sub(1));
    let content_now = if k == 0 { if plain { ring_window_proto::build_plain(s, 3 * s + 4).1 } else { prefix.versions.last().expect("一版").content.clone() } } else { content_now };
    let before = pool_of(&devices);
    let chain = instance_table_chain_of_root(&before, &current.root).expect("实例表");
    let rows_before: Vec<(u32, u64)> = chain.records.rows.iter().map(|r| (r.instance.0, r.selected_root_txg.0)).collect();
    // 清扫：删掉 selected_root_txg = 0 的那几行（连崩挂载留下的 (j, 0, 0)）。
    let kept: Vec<_> = chain.records.rows.iter().copied().filter(|r| r.selected_root_txg.0 != 0).collect();
    let plan_table = if arm == "sweep" {
        InstanceTablePlan::Rewrite(InstanceTableRewrite { rows: kept.clone(), replaced_chain: chain.page_pointers.clone() })
    } else {
        InstanceTablePlan::Carry(current.root.instance_table)
    };
    let txg = CheckpointTxg(current.root.checkpoint_txg.0 + 1);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&before, &stream);
    let mut alloc_copy = allocator.clone();
    let written = {
        let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
        publish_version(&mut pool, &mut alloc_copy, PublishPlan {
            txg,
            counter: current.record.counter + 1,
            transaction: 0,
            highest_transaction_number_before_this_publish: current.highest_transaction_number_in_this_instance,
            instance,
            back_chain: singlefs_core::journal::back_chain_of(&current.record_bytes),
            file: None,
            new_inode_records: &[],
            instance_table: plan_table,
            tree_birth_txg: current.tree_birth_txg(),
            tree_identifier_watermark: current.root.tree_identifier_watermark,
            rollback_floor: current.root.rollback_floor,
        }, Some(&current)).expect("清扫发布")
    };
    let operations = stream.retained_operations();
    let (writes, segments) = writes_and_segments(&operations, &geometry(&parameters));
    let root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("根槽写");
    println!("SWEEP s={s} arm={arm} k={k} instance={} txg={} rows_before={rows_before:?} rows_written={:?} writes={} segments={} closed_form_states={} layer0_states={}",
        instance.0, txg.0, kept.iter().map(|r| (r.instance.0, r.selected_root_txg.0)).collect::<Vec<_>>(), writes.len(), segments.len(),
        closed_form_state_count(&segments), layer0_state_count_with_torn_in_place_overwrites(&before, &writes, &segments, &full_expansion));
    if enumerate {
        let versions = vec![
            PublishedVersion { instance, checkpoint_txg: current.root.checkpoint_txg, content: content_now.clone() },
            PublishedVersion { instance, checkpoint_txg: txg, content: content_now.clone() },
        ];
        let tally = enumerate_layer0_versions(&before, &writes, &segments, root_index, &versions);
        println!("ENUM states={} violations={} first_violation={:?} checker_violated={:?} checker_first={:?} record_root_without_record={} record_claimed_state_missing_unit={}",
            tally.states, tally.violations, tally.first_violation, tally.checker_violated_states, tally.checker_first_violation, tally.record_root_without_record, tally.record_claimed_state_missing_unit);
    }
    // 代表态：记录已持久、根槽没持久（根槽写之前的写全落）；CRASH_ROOT_TORN=1 时根槽写也落、但槽里改坏一个字节（撕裂态：自证不过）。
    let mut crashed = before.clone();
    crashed.apply_writes(&writes[..root_index]);
    if std::env::var("CRASH_ROOT_TORN").is_ok_and(|v| v == "1") {
        crashed.apply_writes(std::slice::from_ref(&writes[root_index]));
        let w = &writes[root_index];
        crashed.flip_byte(w.device, w.offset, 100);
    }
    let red_crash = violated(&crashed);
    println!("CRASH_STATE red={red_crash:?}");
    let mut next = devices_of(&crashed);
    let outcome = mount_writable(&parameters, &mut next);
    match outcome {
        Err(e) => println!("NEXT_MOUNT err={}", format!("{e:?}").chars().take(200).collect::<String>()),
        Ok(mounted4) => {
            let eff = mounted4.output.effective_root;
            println!("NEXT_MOUNT ok chosen=({},{}) effective=({},{}) effective_instance_table_eq_written={} effective_instance_table_eq_chosen={} red_crash_state={red_crash:?} red_after_mount={:?}",
                mounted4.output.chosen_root.instance.0, mounted4.output.chosen_root.checkpoint_txg.0, eff.instance.0, eff.checkpoint_txg.0,
                eff.instance_table == written.root.instance_table, eff.instance_table == current.root.instance_table, violated(&pool_of(&next)));
            let image = pool_of(&next);
            let c = choose_system_configuration(&image).expect("系统配置");
            let newest = readable_roots_with_ring_slots(&image, &c.immutable.region_devices, &c.immutable.sizes, &c.immutable.filesystem_identifier)
                .into_iter().map(|(_, r)| r).max_by_key(|r| (r.checkpoint_txg, r.instance)).expect("根");
            let rows_now = instance_table_chain_of_root(&image, &newest).map(|ch| ch.records.rows.iter().map(|r| (r.instance.0, r.selected_root_txg.0)).collect::<Vec<_>>());
            println!("AFTER_MOUNT newest=({},{}) rows={rows_now:?}", newest.instance.0, newest.checkpoint_txg.0);
            let inst4 = mounted4.output.instance;
            let mut alloc4 = mounted4.allocator.clone();
            match mounted4.current.clone().into_file_version() {
                None => println!("AFTER_MOUNT current_not_file_version"),
                Some(mut cur4) => {
                    for step in 0..m {
                        let mut pool = PoolWriter::new(&parameters, next.as_mut_slice());
                        match publish_overwrite(&mut pool, &mut alloc4, &cur4, FirstFile { content: &content(700 + step), write_time_seconds: WRITE_TIME + 700 + step }, inst4) {
                            Ok(o) => {
                                cur4 = o;
                                let red = violated(&pool_of(&next));
                                let rows = instance_table_chain_of_root(&pool_of(&next), &cur4.root).map(|ch| ch.records.rows.len());
                                println!("STEP {step} txg={} red={red:?} instance_table_rows={rows:?}", cur4.root.checkpoint_txg.0);
                            }
                            Err(e) => { println!("STEP {step} publish_err={}", format!("{e:?}").chars().take(200).collect::<String>()); break; }
                        }
                    }
                    let mut again = devices_of(&pool_of(&next));
                    match mount_writable(&parameters, &mut again) {
                        Ok(m5) => println!("REMOUNT ok effective=({},{}) red={:?}", m5.output.effective_root.instance.0, m5.output.effective_root.checkpoint_txg.0, violated(&pool_of(&again))),
                        Err(e) => println!("REMOUNT err={}", format!("{e:?}").chars().take(200).collect::<String>()),
                    }
                }
            }
        }
    }
}
