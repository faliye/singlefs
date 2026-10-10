//! 合法历史的取样：前缀固定（mkfs → 新池新建文件 → 正常可写挂载），之后由用户决定的几步放开扫。
use crate::driver::Op;

pub fn parse(text: &str) -> Vec<Op> {
    text.split(',')
        .filter(|token| !token.is_empty())
        .map(|token| {
            let (head, rest) = token.split_at(1);
            match head {
                "C" => Op::Create(rest.parse().expect("C<n>")),
                "U" => Op::Unlink(rest.parse().expect("U<k>")),
                "W" => Op::Write(rest.parse().expect("W<units>")),
                "R" => Op::Rollback(rest.parse().expect("R<k>")),
                "X" => Op::CrashRecovery,
                "Y" => {
                    let first = rest.ends_with('f');
                    Op::Crash(rest.trim_end_matches('f').parse().expect("Y<k>[f]"), first)
                }
                "M" => Op::Remount,
                "P" => {
                    let (p, n) = rest.split_once('x').expect("P<p>x<n>");
                    Op::PressBack(p.parse().expect("p"), n.parse().expect("n"))
                }
                _ => panic!("不认得 {token}"),
            }
        })
        .collect()
}

fn named(ops: &[Op]) -> String {
    ops.iter().map(Op::name).collect::<Vec<_>>().join(",")
}

/// 家族：f1 unlink → 回退到第 k 新 → 尾巴；f2 崩溃恢复落到旧根之后重发；f3 几个号的 unlink 次序与回退 / 崩溃恢复插在哪；r 随机。
pub fn plan(family: &str) -> Vec<(String, Vec<Op>, u64)> {
    let mut out: Vec<(String, Vec<Op>, u64)> = Vec::new();
    let mut push = |ops: Vec<Op>, batch: u64| {
        let label = format!("b{batch}:{}", named(&ops));
        out.push((label, ops, batch));
    };
    if family == "f1" || family == "all" {
        let prefixes: Vec<(Vec<Op>, Vec<u64>, u64)> = vec![
            (vec![], vec![1], 144),
            (vec![Op::Create(2)], vec![1, 2, 3], 144),
            (vec![Op::Write(3)], vec![1], 1),
            (vec![Op::Write(3), Op::Create(1)], vec![1, 2], 1),
        ];
        for (prefix, targets, batch) in &prefixes {
            for target in targets {
                let other = if *target == 1 { 2 } else { 1 };
                for k in 1..=8u64 {
                    let tails: Vec<Vec<Op>> = vec![
                        vec![],
                        vec![Op::Unlink(*target)],
                        vec![Op::Unlink(other)],
                        vec![Op::Create(1)],
                        vec![Op::CrashRecovery],
                        vec![Op::Remount],
                        vec![Op::CrashRecovery, Op::Unlink(*target)],
                        vec![Op::Rollback(1)],
                        vec![Op::Rollback(3)],
                        vec![Op::Create(1), Op::CrashRecovery, Op::Create(1)],
                        vec![Op::Unlink(*target), Op::Rollback(2)],
                    ];
                    for tail in tails {
                        let mut ops = prefix.clone();
                        ops.push(Op::Unlink(*target));
                        ops.push(Op::Rollback(k));
                        ops.extend(tail);
                        push(ops, *batch);
                    }
                }
            }
        }
    }
    if family == "f2" || family == "all" {
        for pre in [vec![Op::Create(2)], vec![Op::Create(1)], vec![Op::Write(3), Op::Create(2)]] {
            let batch = if pre.contains(&Op::Write(3)) { 1 } else { 144 };
            let bodies: Vec<Vec<Op>> = vec![
                vec![Op::Unlink(3), Op::CrashRecovery, Op::Create(1)],
                vec![Op::Unlink(2), Op::CrashRecovery, Op::Create(2)],
                vec![Op::Create(1), Op::CrashRecovery, Op::Create(1)],
                vec![Op::Create(1), Op::Unlink(4), Op::CrashRecovery, Op::Create(1)],
                vec![Op::Unlink(1), Op::Create(1), Op::CrashRecovery, Op::Create(2)],
                vec![Op::Unlink(2), Op::Create(1), Op::CrashRecovery, Op::Create(1), Op::Unlink(4)],
                vec![Op::CrashRecovery, Op::Create(1), Op::CrashRecovery, Op::Create(1)],
                vec![Op::Unlink(2), Op::Rollback(3), Op::CrashRecovery, Op::Create(1)],
                vec![Op::Unlink(2), Op::CrashRecovery, Op::Rollback(2), Op::Create(1)],
                vec![Op::Create(1), Op::CrashRecovery, Op::Create(1), Op::Rollback(2)],
                vec![Op::Create(1), Op::CrashRecovery, Op::Create(1), Op::Rollback(4)],
                vec![Op::Create(1), Op::CrashRecovery, Op::Create(1), Op::Rollback(6)],
                vec![Op::Unlink(2), Op::Create(1), Op::CrashRecovery, Op::Create(1), Op::Rollback(3), Op::Unlink(2)],
                vec![Op::Unlink(1), Op::CrashRecovery, Op::Unlink(1), Op::Rollback(4), Op::Create(1)],
            ];
            for body in bodies {
                let mut ops = pre.clone();
                ops.extend(body);
                push(ops, batch);
            }
        }
    }
    if family == "f3" || family == "all" {
        let orders: [[u64; 3]; 6] = [[1, 2, 3], [1, 3, 2], [2, 1, 3], [2, 3, 1], [3, 1, 2], [3, 2, 1]];
        for order in orders {
            for position in 0..=3usize {
                for insert in [vec![], vec![Op::Rollback(2)], vec![Op::Rollback(5)], vec![Op::CrashRecovery], vec![Op::CrashRecovery, Op::Create(1)]] {
                    if position > 0 && insert.is_empty() {
                        continue;
                    }
                    let mut ops = vec![Op::Create(2)];
                    for (index, target) in order.iter().enumerate() {
                        if index == position {
                            ops.extend(insert.clone());
                        }
                        ops.push(Op::Unlink(*target));
                    }
                    if position == 3 {
                        ops.extend(insert.clone());
                    }
                    ops.push(Op::Create(1));
                    push(ops, 144);
                }
            }
        }
    }
    if family == "fc" || family == "all" {
        // 崩在上一步写出的那一段的每个屏障 / FUA 段界（与每段的第一个写之后），重挂载，再接用户动作。
        let bodies: Vec<(Vec<Op>, u64, u64)> = vec![
            (vec![Op::Unlink(1)], 144, 8),
            (vec![Op::Write(3), Op::Unlink(1)], 1, 16),
            (vec![Op::Create(2), Op::Unlink(2)], 144, 8),
            (vec![Op::Create(2), Op::Unlink(3), Op::Rollback(2)], 144, 4),
            (vec![Op::Create(2), Op::Unlink(3), Op::Rollback(3)], 144, 4),
            (vec![Op::Create(2), Op::Unlink(3), Op::Rollback(3), Op::Unlink(3)], 144, 8),
            (vec![Op::Create(2), Op::Create(1)], 144, 4),
            (vec![Op::Create(2), Op::Unlink(2), Op::Rollback(4), Op::Rollback(1)], 144, 4),
        ];
        let tails: Vec<Vec<Op>> = vec![
            vec![],
            vec![Op::Create(1)],
            vec![Op::Rollback(2)],
            vec![Op::Rollback(4)],
            vec![Op::Unlink(2), Op::Rollback(3)],
            vec![Op::CrashRecovery, Op::Create(1)],
        ];
        for (body, batch, segment_count) in &bodies {
            for k in 0..*segment_count {
                for first in [false, true] {
                    if k == 0 && !first {
                        continue;
                    }
                    for tail in &tails {
                        let mut ops = body.clone();
                        ops.push(Op::Crash(k, first));
                        ops.extend(tail.clone());
                        push(ops, *batch);
                    }
                }
            }
        }
    }
    if family == "r" || family == "all" {
        let base: u64 = std::env::var("M3DCR2_SEED_BASE").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
        let seeds: u64 = std::env::var("M3DCR2_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(120);
        for seed in base..base + seeds {
            let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
            let mut next = |bound: u64| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state % bound
            };
            let mut ops = vec![Op::Create(1 + next(3))];
            let mut largest = 4u64;
            for _ in 0..(6 + next(5)) {
                let op = match next(12) {
                    10 | 11 => Op::Crash(next(8), next(2) == 0),
                    0 | 1 | 2 => Op::Unlink(1 + next(largest)),
                    3 | 4 => Op::Rollback(1 + next(8)),
                    5 | 6 => Op::CrashRecovery,
                    7 => Op::Remount,
                    _ => {
                        let n = 1 + next(2);
                        largest += n;
                        Op::Create(n)
                    }
                };
                ops.push(op);
            }
            push(ops, if seed % 3 == 0 { 1 } else { 144 });
        }
    }
    out
}
