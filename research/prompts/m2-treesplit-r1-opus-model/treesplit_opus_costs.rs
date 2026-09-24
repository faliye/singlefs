//! m2-treesplit-r1 云端攻方：层 0 状态数的代价（真实现的闭式 `closed_form_state_count`，段长取入库用例断言的真段序列）
//! 与「分裂单开一次发布」在回退窗口上的代价（真实现的 `user_visible_trees_changed`）。只在副本上。
#[path = "treesplit_opus_prototype.rs"]
#[allow(dead_code, unused_imports, unused_mut)]
mod proto;

use proto::{base_cfg, prefixes, menus, Cfg, Op, Pool, PublishKind, WriteStep};
use singlefs_core::mount::user_visible_trees_changed;
use singlefs_core::recovery::UserVisibleTreeRootPointers;
use singlefs_harness::crash::closed_form_state_count;

fn segs(sizes: &[usize]) -> Vec<Vec<usize>> {
    sizes.iter().map(|n| (0..*n).collect()).collect()
}

fn cf(sizes: &[usize]) -> u64 {
    closed_form_state_count(&segs(sizes))
}

#[test]
fn layer0_state_counts_on_the_real_segment_sequences() {
    // 入库用例断言的真段序列（行号见报告）：第一个事务（门禁 54 号 LAYER0 那条）与并行线一（ignored 的全量）。
    let first = [2, 2, 1, 2, 2, 1, 18, 3, 2];
    let line_one = [2, 2, 1, 2, 2, 1, 18, 2, 1, 20, 4, 1, 22, 6, 1, 2];
    println!("REAL first_transaction sizes={first:?} closed_form={}", cf(&first));
    println!("REAL parallel_line_one sizes={line_one:?} closed_form={}", cf(&line_one));
    // 第一个事务那次发布（18 写段 = 8 个单元两盘 + 上一次的系统配置槽 2 写）若同时让一棵根兼叶的树分裂：
    // 候选 A：根兼叶换成左叶 + 右叶 + 新根，多 2 个单元 = 多 4 写，同一段。
    // 候选 B2：分裂新建的两个单元（右叶、新根）自成一段（前面一道屏障），其余 18 写一段。
    // 候选 B1：先发一次只改结构的发布（左、右、新根 3 个 + 四个固定点单元 = 7 个单元 14 写 + 上一次系统配置 2 写；记录 2 写；根 1 写；
    //          它自己的系统配置 2 写并进下一段），再发原来那次（它的单元段仍是 18 写：8 个单元两盘 + 结构发布的系统配置 2 写）。
    for (name, sizes) in [
        ("A_root_leaf_split_one_tree", vec![2, 2, 1, 2, 2, 1, 22, 3, 2]),
        ("A_root_leaf_split_two_trees", vec![2, 2, 1, 2, 2, 1, 26, 3, 2]),
        ("A_root_leaf_split_three_trees", vec![2, 2, 1, 2, 2, 1, 30, 3, 2]),
        ("B2_root_leaf_split_one_tree", vec![2, 2, 1, 2, 2, 1, 4, 18, 3, 2]),
        ("B2_root_leaf_split_three_trees", vec![2, 2, 1, 2, 2, 1, 12, 18, 3, 2]),
        ("B1_root_leaf_split_one_tree", vec![2, 2, 1, 2, 2, 1, 16, 2, 1, 18, 2, 1, 2]),
    ] {
        println!("VARIANT {name} sizes={sizes:?} closed_form={} ratio_to_first={:.2}", cf(&sizes), cf(&sizes) as f64 / cf(&first) as f64);
    }
    // 每多 u 个单元（两盘各一写）同一段的状态数乘 4^u：给出 u = 1..8 时一个 18 写段变成多少。
    for u in 1..=8usize {
        let w = 18 + 2 * u;
        println!("SEGMENT unit_extra={u} writes={w} states_of_segment={}", (1u64 << w) - 1);
    }
}

/// 「非空」照实现的 `user_visible_trees_changed`：把原型里树根指针的字节当成 extent 树的根指针喂进去。
fn non_empty_flags(pool: &Pool) -> Vec<bool> {
    let mut previous = UserVisibleTreeRootPointers::ABSENT;
    pool.versions
        .iter()
        .map(|v| {
            let here = UserVisibleTreeRootPointers { inode_tree: None, extent_tree: Some(ptr_bytes(v)) };
            let changed = user_visible_trees_changed(&here, &previous);
            previous = here;
            changed
        })
        .collect()
}

fn ptr_bytes(v: &proto::Version) -> Vec<u8> {
    let mut b = v.root.slot.to_le_bytes().to_vec();
    b.extend(v.root.crc.to_le_bytes());
    b.push(v.root.level);
    b.extend(v.root.birth.to_le_bytes());
    b
}

/// D16 已定项 1：候选集的下沿 = 第 4 新的非空根（非空不足 4 个取最旧）；这里数候选集里有几种不同的内容。
fn distinct_states_in_window(pool: &Pool) -> (usize, usize, usize) {
    let flags = non_empty_flags(pool);
    let non_empty: Vec<usize> = (0..pool.versions.len()).filter(|i| flags[*i]).collect();
    let floor = if non_empty.len() >= 4 { non_empty[non_empty.len() - 4] } else { 0 };
    let mut contents: Vec<&std::collections::BTreeMap<u64, u64>> = vec![];
    for v in &pool.versions[floor..] {
        if !contents.contains(&&v.content) {
            contents.push(&v.content);
        }
    }
    let restructure = pool.versions[floor..].iter().filter(|v| v.kind == PublishKind::Restructure).count();
    (contents.len(), pool.versions.len() - floor, restructure)
}

#[test]
fn split_as_its_own_publish_eats_the_four_state_rollback_window() {
    for step in [WriteStep::OnePublish, WriteStep::SplitOwnBarrier, WriteStep::SplitOwnPublish] {
        let cfg = Cfg { step, name: "window", ..base_cfg("window") };
        let (mut histories, mut short, mut min_distinct) = (0u64, 0u64, usize::MAX);
        let mut example = String::new();
        for (pname, prefix) in prefixes().into_iter().filter(|(n, _)| *n == "PA" || *n == "PE") {
            let (_, seconds) = menus(&prefix);
            let after_prefix = proto::pool_after(cfg, &prefix);
            for a in &seconds {
                for b in &seconds {
                    for c in &seconds {
                        let mut pool = after_prefix.clone();
                        for ops in [a, b, c] {
                            pool.user_publish(ops);
                        }
                        // 只数用户可见内容确实变了四次以上的历史：内容不同的版本不足 4 个时窗口本来就凑不满。
                        let mut all: Vec<&std::collections::BTreeMap<u64, u64>> = vec![];
                        for v in &pool.versions {
                            if !all.contains(&&v.content) {
                                all.push(&v.content);
                            }
                        }
                        if all.len() < 5 {
                            continue;
                        }
                        histories += 1;
                        let (distinct, roots, restructure) = distinct_states_in_window(&pool);
                        min_distinct = min_distinct.min(distinct);
                        if distinct < 4 {
                            short += 1;
                            if example.is_empty() {
                                example = format!("prefix={pname} suffix={:?} window_roots={roots} restructure_in_window={restructure} distinct={distinct}", [a, b, c]);
                            }
                        }
                    }
                }
            }
        }
        println!("WINDOW step={step:?} histories={histories} window_short_of_4_distinct={short} min_distinct={min_distinct} example={example}");
    }
    let _ = Op::Ins(0);
}

/// T6「维持拒绝」在真容量上会不会拒：用实现的分配器（`PoolAllocator`，两盘，复用窗口置 0 让洞当场可复用）造一个老化的池——
/// 先连着分出 `filler` 个一单元对象，再每隔 `stride` 个释放一个留洞；然后「一次覆盖写 d 个数据单元的文件」：
/// 它的 d 个落点落进洞里（分配器取最低的空槽），同一次发布把上一版那 d 个落点释放。
/// 数的是这次发布要重写的分配记录树叶数的下界：分配记录按 (盘, 槽) 升序排、每叶装满 812 条（真容量、满填充——最省的那一种），
/// 被改到的记录落在几片不同的叶上。祖先、记账、映射树、树表一个都没算，所以真要点名的共享单元只多不少。
#[test]
fn refusal_on_real_capacities_in_an_aged_pool() {
    use singlefs_core::address::DeviceIdentity;
    use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator, ReuseWindow};
    use singlefs_format::{SLOT_BYTES, UNIT_AREA_START_SLOT};
    const LEAF: usize = 812;
    let named_capacity = 67usize;
    for (filler, stride, d) in [(4000usize, 50usize, 40usize), (20000, 400, 40), (20000, 400, 20), (40000, 700, 40), (40000, 700, 34)] {
        let unit_area_slots = (filler as u64 + 4 * d as u64 + 64) * 2 + 128;
        let bytes = (UNIT_AREA_START_SLOT + unit_area_slots) * SLOT_BYTES;
        let devices = vec![DeviceFreeMap::new(DeviceIdentity(1), bytes), DeviceFreeMap::new(DeviceIdentity(2), bytes)];
        let mut allocator = PoolAllocator::new(devices);
        allocator.set_reuse_window(ReuseWindow::ForcedToZero);
        let mut txg = 1u64;
        let mut placements = vec![];
        for _ in 0..filler {
            placements.push(allocator.allocate_user_data(singlefs_core::address::CheckpointTxg(txg)).expect("池够大"));
        }
        txg += 1;
        // 上一版的文件：d 个单元，分散在老化的池里（每隔 stride 个取一个当它的单元）。
        let old_file: Vec<_> = (0..d).map(|i| placements[i * stride + stride / 2]).collect();
        // 别的对象删掉留洞：每隔 stride 个释放一个（与文件单元错开）。
        for i in (0..filler).step_by(stride) {
            allocator.release(placements[i], singlefs_core::address::CheckpointTxg(txg));
        }
        txg += 1;
        let key = |device: u32, slot: u64| (device, slot);
        let before: Vec<(u32, u64)> = {
            let mut v: Vec<_> = allocator.records().iter().map(|r| key(r.device.0, r.slot.0)).collect();
            v.sort_unstable();
            v
        };
        // 这次覆盖写：d 个新落点（落进洞里），上一版的 d 个释放。
        let mut touched_slots = vec![];
        for _ in 0..d {
            let p = allocator.allocate_user_data(singlefs_core::address::CheckpointTxg(txg)).expect("有洞");
            touched_slots.push(p.slot.0);
        }
        for p in &old_file {
            allocator.release(*p, singlefs_core::address::CheckpointTxg(txg));
            touched_slots.push(p.slot.0);
        }
        let mut after: Vec<(u32, u64)> = allocator.records().iter().map(|r| key(r.device.0, r.slot.0)).collect();
        after.sort_unstable();
        let mut leaves = std::collections::BTreeSet::new();
        for device in [1u32, 2] {
            for slot in &touched_slots {
                let position = after.binary_search(&(device, *slot)).expect("改到的记录在树里");
                leaves.insert(position / LEAF);
            }
        }
        println!(
            "AGED filler={filler} stride={stride} d={d} records_before={} records_after={} alloc_leaves_total={} touched_alloc_leaves_lower_bound={} named_capacity={named_capacity} refused_under_keep_refusing={}",
            before.len(),
            after.len(),
            after.len().div_ceil(LEAF),
            leaves.len(),
            leaves.len() > named_capacity
        );
    }
}
