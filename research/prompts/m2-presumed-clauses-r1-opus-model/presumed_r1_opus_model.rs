//! m2-presumed-clauses-r1 云端攻方的纯 std 模型：与副本装置（`crates/` 真代码）不共享一行代码的第二条路。
//! K2：两盘、根环区域归属 0 / 1 / 0、区域 = txg mod 3。按写行 txg 现算暖机（今天：逐个加一推到覆盖两块盘；候选：写行 txg 往后挪到只推一次），
//!     数白推次数；挂载后发一版数据，丢一块盘时这一版还在不在——点名单元验证按「两份都得过」（今天）与「任一份过」两种口径各算一遍。
//! K4：所选根 txg 10，三次发布 P0 / P1 / P2 各 n0 / n1 / n2 条记录（同一次发布共享 txg），读不出的记录取遍全部子集；
//!     链首按今天的规则（锚 = 第一条 txg 相等的记录；没锚时只接 txg = 根 + 1 的那条）与三个候选各跑一遍，与「锚点已知时的条款链」比。
//! 复跑：rustc -O --edition 2021 presumed_r1_opus_model.rs -o /tmp/claude-1000/presumed-r1-opus/model && /tmp/claude-1000/presumed-r1-opus/model
use std::collections::BTreeMap;

const REGION_DEVICES: [u32; 3] = [0, 1, 0];

fn device_of(txg: u64) -> u32 {
    REGION_DEVICES[(txg % 3) as usize]
}

/// 今天的现算：从写行 txg 起逐个加一，推到覆盖两块盘为止，至多 3 次。
fn warm_up_txgs(row_txg: u64) -> Vec<u64> {
    let mut covered = vec![device_of(row_txg)];
    let mut warm = Vec::new();
    let mut latest = row_txg;
    while covered.len() < 2 && warm.len() < 3 {
        latest += 1;
        if !covered.contains(&device_of(latest)) {
            covered.push(device_of(latest));
        }
        warm.push(latest);
    }
    warm
}

fn white_count(row_txg: u64, warm: &[u64]) -> usize {
    let mut covered = vec![device_of(row_txg)];
    let mut white = 0;
    for txg in warm {
        if covered.contains(&device_of(*txg)) {
            white += 1;
        } else {
            covered.push(device_of(*txg));
        }
    }
    white
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WarmArm {
    Today,
    Cap(usize),
    Jump,
}

/// 挂载：给出写行 txg 与暖机 txg 列表。
fn mount(first_txg: u64, arm: WarmArm) -> (u64, Vec<u64>) {
    match arm {
        WarmArm::Today => (first_txg, warm_up_txgs(first_txg)),
        WarmArm::Cap(cap) => {
            let mut warm = warm_up_txgs(first_txg);
            warm.truncate(cap);
            (first_txg, warm)
        }
        WarmArm::Jump => {
            let mut row = first_txg;
            let mut jumps = 0;
            while warm_up_txgs(row).len() > 1 && jumps < 3 {
                row += 1;
                jumps += 1;
            }
            (row, warm_up_txgs(row))
        }
    }
}

/// 挂载后发一版数据（txg = 最后一次暖机 + 1），丢掉 lost 那块盘：这一版还在不在。
/// 两份都得过（今天）：只有这一版自己的根在幸存盘上才在——从别的根重放要验点名单元，丢掉的那一份验不过。
/// 任一份过：幸存盘上有这个实例的任何一条根就在（记录两盘镜像，从那条根顺着同实例的链重放到这一版）。
fn data_survives(row: u64, warm: &[u64], lost: u32, named_any: bool) -> bool {
    let data_txg = warm.last().copied().unwrap_or(row) + 1;
    if device_of(data_txg) != lost {
        return true;
    }
    if !named_any {
        return false;
    }
    std::iter::once(row)
        .chain(warm.iter().copied())
        .any(|txg| device_of(txg) != lost)
}

fn k2() {
    println!("# K2 纯模型");
    for arm in [WarmArm::Today, WarmArm::Cap(0), WarmArm::Cap(1), WarmArm::Jump] {
        for named_any in [false, true] {
            let mut lost_count = [0usize; 3];
            let mut mounts = [0usize; 3];
            let mut white = [0usize; 3];
            let mut publishes = [0usize; 3];
            for first_txg in 4..=30u64 {
                let (row, warm) = mount(first_txg, arm);
                let residue = (first_txg % 3) as usize;
                mounts[residue] += 1;
                white[residue] += white_count(row, &warm);
                publishes[residue] += 1 + warm.len();
                for lost in [0u32, 1] {
                    if !data_survives(row, &warm, lost, named_any) {
                        lost_count[residue] += 1;
                    }
                }
            }
            println!(
                "K2-MODEL arm={arm:?} named_any={named_any} by_first_txg_mod3: mounts={mounts:?} publishes_per_mount_total={publishes:?} white={white:?} disk_loss_LOST={lost_count:?}"
            );
        }
    }
    // 只重开、不写：稳态。
    for arm in [WarmArm::Today, WarmArm::Jump] {
        for start_last_txg in [3u64, 4, 5] {
            let mut last = start_last_txg;
            let mut sequence = Vec::new();
            let mut whites = Vec::new();
            for _ in 0..6 {
                let (row, warm) = mount(last + 1, arm);
                sequence.push(1 + warm.len());
                whites.push(white_count(row, &warm));
                last = warm.last().copied().unwrap_or(row);
            }
            println!(
                "K2-MODEL-CHAIN arm={arm:?} previous_last_txg={start_last_txg} publishes_per_mount={sequence:?} white_per_mount={whites:?}"
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Record {
    publish: usize,
    counter: u64,
    txg: u64,
    last_in_publish: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeadArm {
    Today,
    Max,
    Stop,
    MaxStop,
}

/// 今天的 `replay_journal` 链首与续接，照它的控制流另写一遍（只管链，不管施加）。
fn chain(records: &[Record], readable: &[bool], arm: HeadArm) -> Vec<u64> {
    const ROOT: u64 = 10;
    let readable_records: Vec<Record> = records
        .iter()
        .zip(readable)
        .filter(|(_, is_readable)| **is_readable)
        .map(|(record, _)| *record)
        .collect();
    let same_txg = readable_records.iter().filter(|record| record.txg == ROOT);
    let anchor = match arm {
        HeadArm::Today | HeadArm::Stop => same_txg.map(|record| record.counter).min(),
        HeadArm::Max | HeadArm::MaxStop => same_txg.map(|record| record.counter).max(),
    };
    let mut expected = anchor.map(|counter| counter + 1);
    let mut applied = Vec::new();
    for record in readable_records.iter().filter(|record| record.txg > ROOT) {
        match expected {
            Some(next) => {
                if record.counter != next {
                    break;
                }
            }
            None => {
                if matches!(arm, HeadArm::Stop | HeadArm::MaxStop) || record.txg != ROOT + 1 {
                    break;
                }
            }
        }
        expected = Some(record.counter + 1);
        applied.push(record.counter);
    }
    applied
}

fn k4() {
    println!("# K4 纯模型");
    for arm in [HeadArm::Today, HeadArm::Max, HeadArm::Stop, HeadArm::MaxStop] {
        let mut tally: BTreeMap<String, usize> = BTreeMap::new();
        let mut cells = 0;
        for n0 in 1..=3usize {
            for n1 in 1..=3usize {
                for n2 in 0..=2usize {
                    let mut records = Vec::new();
                    let mut counter = 100;
                    for (publish, count) in [n0, n1, n2].iter().enumerate() {
                        for index in 0..*count {
                            records.push(Record {
                                publish,
                                counter,
                                txg: 10 + publish as u64,
                                last_in_publish: index + 1 == *count,
                            });
                            counter += 1;
                        }
                    }
                    for mask in 0..(1u64 << records.len()) {
                        cells += 1;
                        let readable: Vec<bool> =
                            (0..records.len()).map(|position| mask & (1 << position) == 0).collect();
                        let today = chain(&records, &readable, arm);
                        let anchor = records
                            .iter()
                            .filter(|record| record.publish == 0)
                            .map(|record| record.counter)
                            .max()
                            .unwrap();
                        let mut clause = Vec::new();
                        for (record, is_readable) in records.iter().zip(&readable) {
                            if record.counter <= anchor {
                                continue;
                            }
                            if !is_readable {
                                break;
                            }
                            clause.push(record.counter);
                        }
                        let anchor_readable = records.iter().zip(&readable).any(|(record, is_readable)| {
                            record.publish == 0 && record.last_in_publish && *is_readable
                        });
                        let other_readable = records
                            .iter()
                            .zip(&readable)
                            .any(|(record, is_readable)| record.publish == 0 && *is_readable);
                        let branch = if anchor_readable {
                            "anchor-readable"
                        } else if other_readable {
                            "anchor-torn-other-P0-readable"
                        } else {
                            "all-P0-torn(no-anchor)"
                        };
                        let verdict = match (today.first(), clause.first()) {
                            (None, None) => "match-none",
                            (Some(a), Some(b)) if a == b => "match",
                            (Some(_), _) => "OVER(接上不该接的)",
                            (None, Some(_)) => "UNDER(少施加)",
                        };
                        *tally.entry(format!("{branch} {verdict}")).or_default() += 1;
                    }
                }
            }
        }
        let arm_name = match arm {
            HeadArm::Today => "today",
            HeadArm::Max => "max",
            HeadArm::Stop => "stop",
            HeadArm::MaxStop => "max-stop",
        };
        for (key, count) in &tally {
            println!("K4-MODEL-TALLY [{arm_name}] {key} = {count}");
        }
        println!("K4-MODEL-SUMMARY [{arm_name}] cells={cells}");
    }
}

fn main() {
    k2();
    k4();
}
