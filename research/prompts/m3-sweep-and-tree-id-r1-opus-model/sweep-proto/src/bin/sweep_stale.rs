//! S1 Q1 / Q2：判定与抹头之间隔着分配（后台续做、或取号之前判、写行之后抹）时，抹的是一张过期的单子。
//! 用法：sweep_stale <K> <stale|reverify>
//!   在「死实例」前缀上先判（单子 A），之后实例 2 再覆盖写 K 次（分配器照常发槽），再按单子 A 抹头、清扫发布。
//!   stale：照单子抹；reverify：抹之前逐个重读那一槽的头，仍是候选行的未发布单元才抹。
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::instance_table_chain_of_root;
use sweep_proto::*;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let k: u64 = a[1].parse().expect("K");
    let arm = a[2].clone();
    let mut p = dead_instance_prefix(4, 5);
    let base0 = pool_of(&p.devices);
    let judgement = judge_sweep(&base0, &p.current.root, None);
    let list_a: Vec<(u32, u64)> = judgement.unpublished_by_instance.values().flatten().copied().collect();
    let i2 = p.i2;
    for step in 0..k {
        p.current = overwrite_with(&p.parameters, &mut p.devices, &mut p.allocator, &p.current, &content(300 + step, 3000), 300 + step, i2);
    }
    let base = pool_of(&p.devices);
    let allocated_now = allocated_slots(&p.current);
    let ring_ref = slots_referenced_by_any_ring_root(&base);
    let stale_live = list_a.iter().filter(|s| allocated_now.contains(s)).count();
    let stale_ring = list_a.iter().filter(|s| ring_ref.contains(s)).count();
    let targets: Vec<(u32, u64)> = if arm == "reverify" {
        let rows = judgement.candidate_rows.clone();
        list_a.iter().copied().filter(|(d, s)| {
            parse_unit_header(&read_slot_head(&base, singlefs_core::address::DeviceIdentity(*d), *s, 4096)).is_some_and(|h| h.fsid_low == fsid_low()
                && rows.iter().any(|r| r.instance.0 == h.instance && unpublished_by_row(&h, r.selected_root_txg.0, r.applied_transaction_high_water)))
        }).collect()
    } else {
        list_a.clone()
    };
    println!("SETUP k={k} arm={arm} list_a={} list_a_now_in_current_ledger={stale_live} list_a_now_referenced_by_a_ring_root={stale_ring} targets={}", list_a.len(), targets.len());
    erase_headers(&mut p.devices, &targets);
    // 清扫条件在抹头这一刻仍成立（环里仍没有实例 1 的根），按单子 A 那一刻判的候选行删。
    let chain = instance_table_chain_of_root(&base, &p.current.root).expect("链");
    let kept: Vec<_> = rows_of(&base, &p.current.root).into_iter().filter(|r| !judgement.candidate_rows.iter().any(|c| c.instance == r.instance)).collect();
    let mut alloc = p.allocator.clone();
    let swept = sweep_publish(&p.parameters, &mut p.devices, &mut alloc, &p.current, kept, chain.page_pointers.clone(), true);
    match swept {
        Err(e) => println!("SWEEP_PUBLISH err={}", short_err(&e)),
        Ok(sw) => {
            let after = pool_of(&p.devices);
            println!("AFTER_SWEEP root=({},{}) red={:?}", sw.root.instance.0, sw.root.checkpoint_txg.0, violated(&after));
            let mut devices = devices_of(&after);
            match mount_writable(&p.parameters, &mut devices) {
                Err(e) => println!("REMOUNT err={}", short_err(&e)),
                Ok(m) => println!("REMOUNT ok effective=({},{}) red={:?}", m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0, violated(&pool_of(&devices))),
            }
            println!("READ_ONLY_MOUNT ok={}", singlefs_core::mounted_read::mount_read_only(&after).is_ok());
        }
    }
}
