//! c355-c363-r3 攻方腿（W2）计数模型，只用 std。
//! 问的是：一次发布 P 分不到落点（空闲 f < c_E + u）之后，「先推带新 F 的空发布抬 F 腾地方」在第几步卡住；
//! 以及把 P 的用户内容拆成几次发布时，能不能在不回收的前提下做完。
//! 输入（每一格都写来源，行号在报告里现查）：
//!   - 根环区域 = txg mod 3，区域归属盘 0 / 1 / 0（D16 已定项 8 引 D2 已定项 7；`mount::raise_rollback_floor` 的循环按
//!     `target_for_publish` 数盘，落满每块盘才放开扣住）。
//!   - 每次发布（含空发布）重写四个固定点单元（D16 已定项 9），今天每个 1 槽 ⇒ c_E = 4；覆盖写多 6 槽
//!     （数据单元 2、extent 根 1、inode 叶容器 2、inode 根 1，`transaction::publish_admitted` 在装单元之前取定全部落点）。
//!   - 抬 F 回收的槽扣住到生效（`ReclaimedReuse::HeldUntilFloorTakesEffect`），推空发布只能用扣住之外的空槽 f。
//!   - 每次发布放掉上一版的落点进 defer，释放代 = 这次的 txg > 新 F，这一轮抬 F 回收不到它们。
//! 用法：w2_count [--mutate-ignore-device-cover | --mutate-split-pays-no-fixed-point]（变异一：抬 F 只推一次，不看落没落满每块盘；
//! 变异二：拆开的每一份都不付固定点。两条变异各自必须让断言红、退出码非 0）。

fn device_of_region(region: u64) -> u8 {
    [0u8, 1, 0][usize::try_from(region).expect("0..3")]
}

/// 从 first_txg 起连推，几次之后每块盘上都有一条（两块盘）。
fn publishes_to_cover_both_devices(first_txg: u64, ignore_cover: bool) -> u64 {
    if ignore_cover {
        return 1;
    }
    let mut seen = [false, false];
    let mut count = 0;
    let mut txg = first_txg;
    while !(seen[0] && seen[1]) {
        seen[usize::from(device_of_region(txg % 3))] = true;
        count += 1;
        txg += 1;
    }
    count
}

#[derive(Default, Clone, Copy)]
struct Tally {
    circular_before_any_write: u64,
    stuck_after_writes: u64,
    raise_completes: u64,
    split_first_part_fits: u64,
    split_completes_without_reclaim: u64,
}

fn main() {
    let ignore_cover = std::env::args().any(|arg| arg == "--mutate-ignore-device-cover");
    let split_pays_no_fixed_point = std::env::args().any(|arg| arg == "--mutate-split-pays-no-fixed-point");
    println!("# w2_count ignore_cover={ignore_cover} split_pays_no_fixed_point={split_pays_no_fixed_point}");
    println!("# c_E\tu\tphase(txg mod 3)\tk(推几次落满两块盘)\tf 取值区间\t卡在第一次（写之前）\t写了之后卡住\t抬 F 做完\t拆开后第一份做得成\t拆开后不回收就做完");
    let mut rows: Vec<(u64, u64, u64, u64, Tally)> = Vec::new();
    for c_e in [4u64, 5, 6, 7, 8, 10, 12] {
        for u in [0u64, 6] {
            for phase in 0..3u64 {
                let k = publishes_to_cover_both_devices(phase, ignore_cover);
                let mut tally = Tally::default();
                // P 被拒 ⟺ f < c_E + u（扣住之外的空槽不够 P 的全部落点）。
                for f in 0..(c_e + u) {
                    if f < c_e {
                        tally.circular_before_any_write += 1;
                    } else if f < k * c_e {
                        tally.stuck_after_writes += 1;
                    } else {
                        tally.raise_completes += 1;
                    }
                    // 拆：P 的用户内容 u 分成两份 u1 + u2（u1, u2 ≥ 1），每份各自是一次发布、各付 c_E。
                    // 不回收：第二份只能用第一份之后剩下的 f − c_E − u1（第一份放掉的落点释放代 > F，拿不到）。
                    let mut first_fits = false;
                    let mut completes = false;
                    for u1 in 1..u {
                        let u2 = u - u1;
                        let fixed = if split_pays_no_fixed_point { 0 } else { c_e };
                        if fixed + u1 <= f {
                            first_fits = true;
                            if fixed + u2 <= f - fixed - u1 {
                                completes = true;
                            }
                        }
                    }
                    if first_fits {
                        tally.split_first_part_fits += 1;
                    }
                    if completes {
                        tally.split_completes_without_reclaim += 1;
                    }
                }
                println!(
                    "{c_e}\t{u}\t{phase}\t{k}\t[0,{})\t{}\t{}\t{}\t{}\t{}",
                    c_e + u,
                    tally.circular_before_any_write,
                    tally.stuck_after_writes,
                    tally.raise_completes,
                    tally.split_first_part_fits,
                    tally.split_completes_without_reclaim
                );
                rows.push((c_e, u, phase, k, tally));
            }
        }
    }
    // 绝对值断言（变异时同样跑，变异必须让它们红：判别力自证）：
    {
        // 今天的几何（c_E = 4、覆盖写 u = 6）逐相位钉住。
        let today: Vec<(u64, u64, u64, u64)> = rows
            .iter()
            .filter(|(c_e, u, _, _, _)| *c_e == 4 && *u == 6)
            .map(|(_, _, phase, k, t)| (*phase, *k, t.stuck_after_writes, t.raise_completes))
            .collect();
        assert_eq!(today, vec![(0, 2, 4, 2), (1, 2, 4, 2), (2, 3, 6, 0)]);
        // 拆开后第一份做得成的 f：[c_E + 1, c_E + u) 共 u − 1 = 5 个，每个相位都一样。
        for (c_e, u, _, _, tally) in &rows {
            if *c_e == 4 && *u == 6 {
                assert_eq!(tally.split_first_part_fits, 5);
            }
        }
        for (c_e, u, _phase, k, tally) in &rows {
            // 空发布（u = 0）：P 被拒 ⟺ f < c_E ⇒ 推空发布在第一次就拿不到，没有一格走得出去，也没有东西可拆。
            if *u == 0 {
                assert_eq!(tally.circular_before_any_write, *c_e);
                assert_eq!(tally.stuck_after_writes + tally.raise_completes, 0);
                assert_eq!(tally.split_first_part_fits, 0);
            }
            // 拆开之后，不回收就做完的一格都没有（每多一份多付一次 c_E）。
            assert_eq!(tally.split_completes_without_reclaim, 0);
            // k 只取 2 或 3（区域 0 / 1 / 0）。
            assert!(*k == 2 || *k == 3);
        }
        println!("# 断言全过");
    }
    let mut total = Tally::default();
    for (_, _, _, _, tally) in &rows {
        total.circular_before_any_write += tally.circular_before_any_write;
        total.stuck_after_writes += tally.stuck_after_writes;
        total.raise_completes += tally.raise_completes;
        total.split_first_part_fits += tally.split_first_part_fits;
        total.split_completes_without_reclaim += tally.split_completes_without_reclaim;
    }
    println!(
        "# 合计\t卡在第一次={}\t写了之后卡住={}\t抬 F 做完={}\t拆开第一份做得成={}\t拆开不回收做完={}",
        total.circular_before_any_write,
        total.stuck_after_writes,
        total.raise_completes,
        total.split_first_part_fits,
        total.split_completes_without_reclaim
    );
}
