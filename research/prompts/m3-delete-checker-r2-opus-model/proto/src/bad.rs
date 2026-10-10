//! 改坏的镜像与只供测试的开关造的坏状态：
//! press：C479 那一问，水位开关压到哪（p）、建几个（n）、压之前做了什么（前缀）放开扫，各候选与池级 checker 抓不抓得到号被重发；
//! s4a：同一片 inode 叶里同号两条、记录互换（T11）；knob：容量开关压小的合法镜像上 I-9.21 产品读法（T5、T6）。
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_core::bytes::ByteReader;
use singlefs_core::pointer::NodePointer;
use singlefs_core::unit::parse_index_node;
use singlefs_harness::memory_pool::MemoryPool;

use crate::cand::V;
use crate::common_deletion_in_progress::change_the_unit_and_propagate_its_checksum;
use crate::common_pointer_head_corruption::{data_unit_bytes, node_bytes, the_slot_of, tree_table_entry_of_kind, unit_bytes_at};
use crate::driver::{switches_with_knobs, Op, Pool};
use crate::{judge, plans, threads, Tally};

const WATCH: [&str; 11] = [
    "T2a/each", "T2c-A/each", "T2c-A2/each", "T2a/sel-x-map", "C479yi/rec", "C479yi/rec+tombA", "C479bing/lineage", "I96/code", "I96/newest", "I96/each",
    "I919/A",
];

fn short(verdict: &V) -> char {
    match verdict {
        V::Holds => 'H',
        V::Red(_) => 'R',
        V::Na => '-',
    }
}

fn press(prefixes: &[&str]) {
    let mut jobs: Vec<(String, Vec<Op>, u64, u64, u64)> = Vec::new();
    for prefix in prefixes {
        let ops = plans::parse(prefix);
        let mut probe = Pool::build(4 << 30, 144);
        for op in &ops {
            probe.run(op);
        }
        let Some(watermark) = probe.watermark() else {
            println!("PRESSNA {prefix} 现行一版没有水位");
            continue;
        };
        for p in 1..watermark {
            for n in 1..=3u64 {
                let follows: Vec<u64> = std::env::var("M3DCR2_FOLLOWS").ok().map_or(vec![0, 1], |text| text.split(',').map(|n| n.parse().expect("数")).collect());
                for follow in follows {
                    jobs.push((prefix.to_string(), ops.clone(), p, n, follow));
                }
            }
        }
    }
    let next = AtomicUsize::new(0);
    let lines = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads() {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::SeqCst);
                let Some((prefix, ops, p, n, follow)) = jobs.get(index) else { break };
                let mut pool = Pool::build(4 << 30, 144);
                pool.keep_images = false;
                for op in ops {
                    pool.run(op);
                }
                let before = pool.watermark().unwrap_or(0);
                let outcome = pool.run(&Op::PressBack(*p, *n));
                for _ in 0..*follow {
                    pool.run(&Op::Create(1));
                }
                let reissue = *p < before && !outcome.starts_with("REFUSED") && !outcome.starts_with("NA");
                let judged = judge(&pool.image());
                let marks: String = WATCH
                    .iter()
                    .map(|name| judged.candidates.iter().find(|(candidate, _)| candidate == name).map_or('?', |(_, verdict)| short(verdict)))
                    .collect();
                let checker: Vec<&str> = judged.checker_reds.iter().map(|(name, _)| *name).collect();
                let detail: BTreeMap<&str, String> = judged
                    .candidates
                    .iter()
                    .filter(|(name, verdict)| WATCH.contains(name) && matches!(verdict, V::Red(_)))
                    .map(|(name, verdict)| (*name, if let V::Red(why) = verdict { why.chars().take(140).collect() } else { String::new() }))
                    .collect();
                lines.lock().expect("锁").push(format!(
                    "PRESS prefix={prefix} wm_before={before} p={p} n={n} follow={follow} reissue={reissue} outcome=[{}] marks={marks} checker={checker:?} detail={detail:?}",
                    outcome.chars().take(60).collect::<String>()
                ));
            });
        }
    });
    let mut lines = lines.into_inner().expect("锁");
    lines.sort();
    println!("PRESSLEGEND marks 次序：{WATCH:?}（H 成立、R 红、- 无对象）");
    for line in &lines {
        println!("{line}");
    }
}

/// 最新根 inode 树根第一条条目指的那片叶的槽。
fn first_inode_leaf_slot(image: &MemoryPool) -> singlefs_core::address::SlotNumber {
    let root = tree_table_entry_of_kind(image, singlefs_core::records::TREE_KIND_INODE).root;
    let node = parse_index_node(&unit_bytes_at(image, the_slot_of(&root), node_bytes())).expect("inode 树根");
    let child = NodePointer::read_from(&mut ByteReader::at(&node.entries[0], 34));
    child.locations[0].slot
}

const RECORDS_START: usize = 136;
const RECORD_BYTES: usize = 140;

fn s4a() {
    let shapes: Vec<(&str, Box<dyn Fn(&mut Vec<u8>)>)> = vec![
        (
            "S4a [1 nlink0, 1 nlink1, 3]：第二条整条换成第一条、第一条 nlink 0",
            Box::new(|leaf: &mut Vec<u8>| {
                let first: Vec<u8> = leaf[RECORDS_START..RECORDS_START + RECORD_BYTES].to_vec();
                leaf[RECORDS_START + RECORD_BYTES..RECORDS_START + 2 * RECORD_BYTES].copy_from_slice(&first);
                leaf[RECORDS_START + 36..RECORDS_START + 40].copy_from_slice(&0u32.to_le_bytes());
            }),
        ),
        (
            "S4b [1 nlink1, 1 nlink0, 3]：第二条整条换成第一条、第二条 nlink 0",
            Box::new(|leaf: &mut Vec<u8>| {
                let first: Vec<u8> = leaf[RECORDS_START..RECORDS_START + RECORD_BYTES].to_vec();
                leaf[RECORDS_START + RECORD_BYTES..RECORDS_START + 2 * RECORD_BYTES].copy_from_slice(&first);
                let second = RECORDS_START + RECORD_BYTES;
                leaf[second + 36..second + 40].copy_from_slice(&0u32.to_le_bytes());
            }),
        ),
        (
            "swap [2, 1, 3]：前两条互换",
            Box::new(|leaf: &mut Vec<u8>| {
                let first: Vec<u8> = leaf[RECORDS_START..RECORDS_START + RECORD_BYTES].to_vec();
                let second: Vec<u8> = leaf[RECORDS_START + RECORD_BYTES..RECORDS_START + 2 * RECORD_BYTES].to_vec();
                leaf[RECORDS_START..RECORDS_START + RECORD_BYTES].copy_from_slice(&second);
                leaf[RECORDS_START + RECORD_BYTES..RECORDS_START + 2 * RECORD_BYTES].copy_from_slice(&first);
            }),
        ),
        (
            "swap-tail [1, 3, 2]：后两条互换",
            Box::new(|leaf: &mut Vec<u8>| {
                let second: Vec<u8> = leaf[RECORDS_START + RECORD_BYTES..RECORDS_START + 2 * RECORD_BYTES].to_vec();
                let third: Vec<u8> = leaf[RECORDS_START + 2 * RECORD_BYTES..RECORDS_START + 3 * RECORD_BYTES].to_vec();
                leaf[RECORDS_START + RECORD_BYTES..RECORDS_START + 2 * RECORD_BYTES].copy_from_slice(&third);
                leaf[RECORDS_START + 2 * RECORD_BYTES..RECORDS_START + 3 * RECORD_BYTES].copy_from_slice(&second);
            }),
        ),
    ];
    let mut pool = Pool::build(4 << 30, 144);
    pool.run(&Op::Create(2));
    let clean = pool.image();
    let judged = judge(&clean);
    println!("S4A clean checker={:?}", judged.checker_reds.iter().map(|(name, _)| *name).collect::<Vec<_>>());
    for (name, change) in &shapes {
        let mut image = clean.clone();
        let slot = first_inode_leaf_slot(&image);
        change_the_unit_and_propagate_its_checksum(&mut image, slot, data_unit_bytes(), |leaf| change(leaf));
        let judged = judge(&image);
        let mine: Vec<String> = judged
            .candidates
            .iter()
            .filter(|(candidate, _)| candidate.starts_with("S4a"))
            .map(|(candidate, verdict)| format!("{candidate}={}", short(verdict)))
            .collect();
        println!(
            "S4A shape=「{name}」 checker_red={:?} candidates={mine:?}",
            judged.checker_reds.iter().map(|(name, detail)| format!("{name}: {}", detail.chars().take(80).collect::<String>())).collect::<Vec<_>>()
        );
    }
}

fn knob() {
    let histories = ["C3,U2,U3,U4", "C3,U2,U3,U4,R3", "C3,U2,U3,R4,U4,U3", "C3,U4,U3,U2,X,C1", "C4,U2,U3,U4,U5,R2,U1"];
    let mut tally = Tally::default();
    for text in histories {
        for (tombstones, entries) in [(1u64, 2usize), (2, 2), (2, 135)] {
            let mut pool = Pool::build_with(4 << 30, switches_with_knobs(144, tombstones, 1, entries));
            for op in plans::parse(text) {
                pool.run(&op);
            }
            for (index, step) in pool.steps.iter().enumerate() {
                let judged = judge(&step.after);
                tally.note("knob", &judged, &format!("knob t{tombstones} e{entries} {text} #{index}:{} [{}]", step.label, step.outcome));
            }
        }
    }
    tally.print("knob", false);
}

/// T7：墓碑树 / 谱系树根第一条条目的身份引用改一处（容器号 +1、类型段改成未登记的 7），重封、补校验和；看今天的 checker 红哪几条。
fn t7(history: &str) {
    let cases: Vec<(&str, u16, usize, Box<dyn Fn(&mut [u8])>)> = vec![
        ("墓碑树根条目容器号 +1", singlefs_core::records::TREE_KIND_TOMBSTONE, 18, Box::new(|field: &mut [u8]| {
            let value = u64::from_le_bytes(field[..8].try_into().expect("8")) + 1;
            field[..8].copy_from_slice(&value.to_le_bytes());
        })),
        ("墓碑树根条目类型段 1 → 7", singlefs_core::records::TREE_KIND_TOMBSTONE, 16, Box::new(|field: &mut [u8]| field[..2].copy_from_slice(&7u16.to_le_bytes()))),
        ("墓碑树根条目类型段 1 → 0", singlefs_core::records::TREE_KIND_TOMBSTONE, 16, Box::new(|field: &mut [u8]| field[..2].copy_from_slice(&0u16.to_le_bytes()))),
        ("谱系树根条目容器号 +1", singlefs_core::records::TREE_KIND_LINEAGE, 18, Box::new(|field: &mut [u8]| {
            let value = u64::from_le_bytes(field[..8].try_into().expect("8")) + 1;
            field[..8].copy_from_slice(&value.to_le_bytes());
        })),
        ("谱系树根条目类型段 5 → 7", singlefs_core::records::TREE_KIND_LINEAGE, 16, Box::new(|field: &mut [u8]| field[..2].copy_from_slice(&7u16.to_le_bytes()))),
    ];
    let mut pool = Pool::build(4 << 30, 144);
    for op in plans::parse(history) {
        pool.run(&op);
    }
    let clean = pool
        .steps
        .iter()
        .rev()
        .find(|step| step.label.ends_with("-image"))
        .map_or_else(|| pool.image(), |step| step.after.clone());
    println!("T7 history={history}");
    println!("T7 clean checker={:?}", judge(&clean).checker_reds.iter().map(|(name, _)| *name).collect::<Vec<_>>());
    for (name, kind, offset, change) in &cases {
        let mut image = clean.clone();
        let root = tree_table_entry_of_kind(&image, *kind).root;
        if root == NodePointer::empty_root() {
            println!("T7 case=「{name}」 skipped（这一版这棵树是零根）");
            continue;
        }
        let slot = the_slot_of(&root);
        let node = parse_index_node(&unit_bytes_at(&image, slot, node_bytes())).expect("根");
        let entry = node.entries[0].clone();
        change_the_unit_and_propagate_its_checksum(&mut image, slot, node_bytes(), |bytes| {
            let at = bytes.windows(entry.len()).position(|window| window == entry.as_slice()).expect("条目在节点里");
            change(&mut bytes[at + offset..]);
        });
        let judged = judge(&image);
        println!(
            "T7 case=「{name}」 checker_red={:?}",
            judged.checker_reds.iter().map(|(name, detail)| format!("{name}: {}", detail.chars().take(90).collect::<String>())).collect::<Vec<_>>()
        );
    }
}

/// 跨容器、跨子树的同号：建 n 个（一片叶装 233 条），把某片叶的某条记录号改成别处已有的号。
fn xleaf(count: u64) {
    let mut pool = Pool::build(4 << 30, 144);
    pool.keep_images = false;
    pool.run(&Op::Create(count));
    let clean = pool.image();
    let root = tree_table_entry_of_kind(&clean, singlefs_core::records::TREE_KIND_INODE).root;
    let root_node = parse_index_node(&unit_bytes_at(&clean, the_slot_of(&root), node_bytes())).expect("根");
    println!("XLEAF count={count} root_level={} root_entries={} checker={:?}", root_node.level, root_node.entries.len(), judge(&clean).checker_reds.iter().map(|(name, _)| *name).collect::<Vec<_>>());
    // 叶按叶序列出来（层级 1 或 2）。
    let mut leaves: Vec<singlefs_core::address::SlotNumber> = Vec::new();
    let mut pending = vec![(root_node, root.clone())];
    while let Some((node, _)) = pending.pop() {
        for entry in node.entries.iter().rev() {
            let child = NodePointer::read_from(&mut ByteReader::at(entry, 34));
            if u16::from_le_bytes([entry[16], entry[17]]) == 0 {
                let child_node = parse_index_node(&unit_bytes_at(&clean, child.locations[0].slot, node_bytes())).expect("内部节点");
                pending.push((child_node, child.clone()));
            } else {
                leaves.insert(0, child.locations[0].slot);
            }
        }
    }
    // 上面按栈倒序压，叶序要再按第一条记录号排一次。
    let first_number = |slot: singlefs_core::address::SlotNumber| -> u64 {
        let leaf = unit_bytes_at(&clean, slot, data_unit_bytes());
        u64::from_le_bytes(leaf[RECORDS_START..RECORDS_START + 8].try_into().expect("8"))
    };
    leaves.sort_by_key(|slot| first_number(*slot));
    println!("XLEAF leaves={}", leaves.len());
    let picks: Vec<(usize, usize, &str)> = vec![(0, leaves.len() - 1, "第一片叶的最后一条改成最后一片叶的第一条的号"), (leaves.len() - 1, 0, "最后一片叶的第一条改成第一片叶最后一条的号")];
    for (from_leaf, to_leaf, name) in picks {
        let target_leaf = unit_bytes_at(&clean, leaves[to_leaf], data_unit_bytes());
        let target_count = usize::from(u16::from_le_bytes([target_leaf[69], target_leaf[70]]));
        let number = if to_leaf == 0 {
            u64::from_le_bytes(target_leaf[RECORDS_START + (target_count - 1) * RECORD_BYTES..][..8].try_into().expect("8"))
        } else {
            u64::from_le_bytes(target_leaf[RECORDS_START..RECORDS_START + 8].try_into().expect("8"))
        };
        let mut image = clean.clone();
        change_the_unit_and_propagate_its_checksum(&mut image, leaves[from_leaf], data_unit_bytes(), |leaf| {
            let count = usize::from(u16::from_le_bytes([leaf[69], leaf[70]]));
            let index = if from_leaf == 0 { count - 1 } else { 0 };
            leaf[RECORDS_START + index * RECORD_BYTES..][..8].copy_from_slice(&number.to_le_bytes());
        });
        let judged = judge(&image);
        println!(
            "XLEAF case=「{name}」 number={number} checker_red={:?}",
            judged.checker_reds.iter().map(|(name, detail)| format!("{name}: {}", detail.chars().take(90).collect::<String>())).collect::<Vec<_>>()
        );
    }
}

/// 新条的坏镜像只红不红自己（调查报告第二、四、五、六节那几份，推的 → 这里量）：最新根墓碑容器第一条墓碑改一处，
/// 看今天的 checker 红什么、各候选读法红什么。
fn tomb() {
    type Change = Box<dyn Fn(&mut [u8], u64)>;
    let turned = format!("C1,C1,U3,{}", vec!["C1"; 30].join(","));
    let turned: &'static str = Box::leak(turned.into_boxed_str());
    let mut cases: Vec<(&str, &str, Change)> = Vec::new();
    for history in ["C1,C1,U3", turned] {
        cases.push(("I-9.10 那份：2、3 分两次建，删完 3，墓碑对象 ID 3 → 2", history, Box::new(|record: &mut [u8], _| record[0..8].copy_from_slice(&2u64.to_le_bytes()))));
        cases.push(("I-9.19 那份：同上，墓碑 (3, b3) → (2, b2)", history, Box::new(|record: &mut [u8], _| {
            record[0..8].copy_from_slice(&2u64.to_le_bytes());
            record[8..16].copy_from_slice(&6u64.to_le_bytes());
        })));
        cases.push(("I-9.20 区间那份：删完 3，区间止 → 5", history, Box::new(|record: &mut [u8], _| record[24..32].copy_from_slice(&5u64.to_le_bytes()))));
        cases.push(("死亡 txg → 0（早于对象出生代）", history, Box::new(|record: &mut [u8], _| record[32..40].copy_from_slice(&0u64.to_le_bytes()))));
    }
    cases.push(("I-9.18 那份：意图之后（崩在第一批之前），死亡 txg → 最新根 txg + 1", "C2,U3,Y4", Box::new(|record: &mut [u8], newest: u64| record[32..40].copy_from_slice(&(newest + 1).to_le_bytes()))));
    cases.push(("I-9.18 变体：意图之后，死亡 txg → 0", "C2,U3,Y4", Box::new(|record: &mut [u8], _| record[32..40].copy_from_slice(&0u64.to_le_bytes()))));
    cases.push(("I-9.18 变体：意图之后，死亡 txg → 对象出生代 − 1", "C2,U3,Y4", Box::new(|record: &mut [u8], _| record[32..40].copy_from_slice(&5u64.to_le_bytes()))));
    let watched = ["T2a/each", "T2c-A/each", "I918/A", "I918/A2", "I918/B", "I919/A", "I919/A2", "I920/A", "I920/interval-all-T14", "C479yi/rec+tombA", "TombDeath/birth-le-death-le-txg"];
    for (name, history, change) in &cases {
        let mut pool = Pool::build(4 << 30, 144);
        for op in plans::parse(history) {
            pool.run(&op);
        }
        let clean = pool
            .steps
            .iter()
            .rev()
            .find(|step| step.label.ends_with("-image"))
            .map_or_else(|| pool.image(), |step| step.after.clone());
        let newest = crate::common_pointer_head_corruption::newest_root(&clean).checkpoint_txg.0;
        let before = judge(&clean);
        let mut image = clean.clone();
        let slot = crate::common_deletion_in_progress::first_tombstone_container_slot_of_the_newest_root(&image);
        change_the_unit_and_propagate_its_checksum(&mut image, slot, data_unit_bytes(), |container| change(&mut container[RECORDS_START..RECORDS_START + 40], newest));
        let judged = judge(&image);
        let reds: Vec<&str> = judged.candidates.iter().filter(|(candidate, verdict)| watched.contains(candidate) && matches!(verdict, V::Red(_))).map(|(candidate, _)| *candidate).collect();
        let reds_before: Vec<&str> = before.candidates.iter().filter(|(candidate, verdict)| watched.contains(candidate) && matches!(verdict, V::Red(_))).map(|(candidate, _)| *candidate).collect();
        println!(
            "TOMB case=「{name}」 history={history} newest_txg={newest} clean_checker={:?} clean_cand={reds_before:?} checker_red={:?} cand_red={reds:?}",
            before.checker_reds.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
            judged.checker_reds.iter().map(|(n, d)| format!("{n}: {}", d.chars().take(70).collect::<String>())).collect::<Vec<_>>()
        );
    }
}

pub fn run(args: &[String]) {
    match args.first().map(String::as_str) {
        Some("press") => {
            let prefixes: Vec<&str> = args[1..].iter().map(String::as_str).collect();
            press(&prefixes);
        }
        Some("s4a") => s4a(),
        Some("knob") => knob(),
        Some("tomb") => tomb(),
        Some("t7") => t7(args.get(1).map_or("C2,U3,R2", String::as_str)),
        Some("xleaf") => xleaf(args.get(1).and_then(|n| n.parse().ok()).unwrap_or(240)),
        other => eprintln!("bad <press <前缀…>|s4a|knob>，收到 {other:?}"),
    }
}
