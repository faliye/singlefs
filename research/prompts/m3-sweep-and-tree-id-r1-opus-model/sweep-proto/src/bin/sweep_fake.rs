//! S1 Q3：用户文件内容里恰在槽边界上带一个码 1 伪头，写序冒充死实例 1、按行判未发布。清扫按几种读法判、抹头、删行之后看后果。
//! 用法：sweep_fake <live|freed> <all|skipalloc|claim|ringref>
//!   live：伪头在现行那一版的数据单元第二个槽里；freed：再覆盖写一次，伪头所在单元已释放、仍被环里上一版的根引用（回退候选）。
//!   all：池中每一个写过的槽都判；skipalloc：跳过当前账已分配的槽；claim：认领规则扫描；ringref：跳过根环里任一可读根引用的槽。
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::recovery::instance_table_chain_of_root;
use singlefs_format::{DATA_UNIT_PAYLOAD_OFFSET, SLOT_BYTES};
use sweep_proto::*;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let place = a[1].clone();
    let reading = a[2].clone();
    let mut p = dead_instance_prefix(4, 5);
    let row1 = p.rows_after_mount2[0];
    // 模板：盘上一个真码 1 头。
    let pool = pool_of(&p.devices);
    let ((dev, slot), _) = every_unit_header(&pool).into_iter().find(|(_, h)| h.class == 1).expect("码 1");
    let template = read_slot_head(&pool, singlefs_core::address::DeviceIdentity(dev), slot, 4096);
    let fake = forged_data_unit_header(&template, p.i1, row1.1 + 1000, row1.2 + 1000);
    let mut file = content(77, 20_000);
    let at = usize::try_from(SLOT_BYTES - DATA_UNIT_PAYLOAD_OFFSET).expect("小");
    file[at..at + fake.len()].copy_from_slice(&fake);
    let i2 = p.i2;
    p.current = overwrite_with(&p.parameters, &mut p.devices, &mut p.allocator, &p.current, &file, 500, i2);
    let fake_version = p.current.root;
    if place == "freed" {
        p.current = overwrite_with(&p.parameters, &mut p.devices, &mut p.allocator, &p.current, &content(78, 3000), 501, i2);
    }
    let base = pool_of(&p.devices);
    let fake_slots: Vec<(u32, u64)> = every_unit_header(&base).into_iter().filter(|(_, h)| h.instance == p.i1 && h.birth == row1.1 + 1000).map(|(k, _)| k).collect();
    let allocated = allocated_slots(&p.current);
    let ring_ref = slots_referenced_by_any_ring_root(&base);
    let judgement = judge_sweep(&base, &p.current.root, None);
    let claimed = claim_rule_scan(&base);
    let targets: Vec<(u32, u64)> = judgement.unpublished_by_instance.values().flatten().copied().filter(|k| match reading.as_str() {
        "all" => true,
        "skipalloc" => !allocated.contains(k),
        "claim" => claimed.contains_key(k),
        "ringref" => !ring_ref.contains(k),
        _ => panic!("读法"),
    }).collect();
    let fake_targeted = fake_slots.iter().filter(|k| targets.contains(k)).count();
    println!("SETUP place={place} reading={reading} rows={:?} fake_slots={fake_slots:?} fake_in_current_ledger={} fake_referenced_by_ring_root={} fake_claimed_away={} targets={} fake_targeted={fake_targeted} candidate_rows={:?}",
        p.rows_after_mount2, fake_slots.iter().filter(|k| allocated.contains(k)).count(), fake_slots.iter().filter(|k| ring_ref.contains(k)).count(),
        fake_slots.iter().filter(|k| !claimed.contains_key(k)).count(), targets.len(), judgement.candidate_rows.iter().map(|r| (r.instance.0, r.selected_root_txg.0)).collect::<Vec<_>>());
    erase_headers(&mut p.devices, &targets);
    let chain = instance_table_chain_of_root(&base, &p.current.root).expect("链");
    let mut alloc = p.allocator.clone();
    let swept = sweep_publish(&p.parameters, &mut p.devices, &mut alloc, &p.current, judgement.kept_rows.clone(), chain.page_pointers.clone(), true).expect("清扫发布");
    let after = pool_of(&p.devices);
    println!("AFTER_SWEEP root=({},{}) rows={:?} red={:?}", swept.root.instance.0, swept.root.checkpoint_txg.0, rows_of(&after, &swept.root).iter().map(|r| (r.instance.0, r.selected_root_txg.0)).collect::<Vec<_>>(), violated(&after));
    let mut devices = devices_of(&after);
    match mount_writable(&p.parameters, &mut devices) {
        Err(e) => println!("REMOUNT err={}", short_err(&e)),
        Ok(m) => {
            println!("REMOUNT ok effective=({},{}) red={:?}", m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0, violated(&pool_of(&devices)));
            if place == "freed" {
                let mut alloc = m.allocator.clone();
                let mut cur = m.current.clone().into_file_version().expect("带文件");
                let target = RollbackTarget { instance: fake_version.instance, checkpoint_txg: fake_version.checkpoint_txg };
                match roll_back_by_a_forward_publish(&p.parameters, &mut devices, &mut alloc, &mut cur, target) {
                    Ok(r) => println!("ROLLBACK to=({},{}) ok resurrected={} red={:?}", target.instance.0, target.checkpoint_txg.0, r.resurrected_user_visible_copies.len(), violated(&pool_of(&devices))),
                    Err(e) => println!("ROLLBACK to=({},{}) err={}", target.instance.0, target.checkpoint_txg.0, short_err(&e)),
                }
            }
        }
    }
}
