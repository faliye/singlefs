//! E67：一条条带参与哪几块盘 —— D2 已定项 8。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - D2 已定项 6（2026-08-31）：`w = clamp(攒批后写入量 + 1, 2, 4)`，上界 4 是超级块声明的常量。
//! - D2 已定项 6 正文：取消上界时「每条带命中率 1.000 ⇒ 没有一个文件是完整的」——
//!   **失败模式从「部分受损」变成弥散**，这是本实验要量的那一维。
//! - D2 已定项 8：定了用几列，没定选哪几块；`.claude/rules/fs-design.md` 硬要求 2
//!   要求测试开关能钉住参与集合。
//!
//! ## 要测什么
//!
//! 两盘同坏时，**有多少比例的对象一格都没丢**。这不是「丢了多少」——
//! 期望丢数据比例由宽度定死（`(w−1)/C(D,2)`，已定项 6），与选哪几块无关；
//! 选法只改变**损失怎么分布**：集中在少数对象上，还是弥散到每个对象上。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. 各臂「完好对象比例」差 ≥ 10 个百分点 ⇒ 这一维决定选型；差 < 10pp ⇒ 判「这一维不决定」，
//!    改由均衡度决定。**反过来的结果接受。**
//! 2. **守恒**：各臂的期望丢数据比例必须彼此相等（差 ≤ 1%）——它只由宽度定。
//!    不等 ⇒ 模型把宽度和选法混了，**整轮作废**。
//! 3. 盘间使用量极差超过单盘容量 5% 的臂淘汰（会提前 ENOSPC，E60 已量过搁浅的代价）。
//!
//! ## 失败条款
//!
//! - **阳性对照，对每一条臂都跑**：`w = DEVS`（全宽）⇒ 只有一种组合 ⇒ 四臂的完好对象比例
//!   必须**全部为 0**、且彼此相等。任一臂非 0 ⇒ **整轮作废**。
//! - **阴性对照**：对象只有 1 条条带且 w=2、DEVS=8 ⇒ 完好比例 = 1 − C(2,2)/C(8,2) = 27/28。
//! - 失败对穷举全部 `C(DEVS,2)` 对，不抽样 ⇒ 没有采样误差。
//!
//! ## 它答不了的
//!
//! 计数模型：没有文件系统、没有块设备、文件操作 0 处。不建模三盘同坏、不建模重建窗口、
//! 不建模「同一对象的条带在时间上分散」带来的额外相关性。

use e7_index_bench::Emitter;

const DEVS: usize = 8;
const WIDTH: usize = 4;
const OBJECTS: usize = 512;
const STRIPES_PER_OBJ: usize = 8;
/// 各盘容量（格）。**异构**：4 块大盘 + 4 块小盘——D2 已定项 2 已定各盘不必等大，
/// 而本地腿正是拿这一条给出反例：绑组可能把对象钉在四块小盘上。
/// 总容量 4×3000 + 4×900 = 15600，均匀轮总需求 512×8×4 = 16384 ⇒ 池会被填满，
/// 「组内满而别处有空间」这个形态才暴露得出来。
const CAPS: [u64; DEVS] = [3000, 3000, 3000, 3000, 900, 900, 900, 900];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Pick {
    /// 甲：对象绑一组盘——同一对象的所有条带用同一组。
    ObjectPinned,
    /// 乙：每条条带轮转起点。
    RoundRobin,
    /// 丙：每条条带取剩余空间最多的 w 块。
    Emptiest,
    /// 丁：每条条带随机取 w 块。
    Random,
    /// 戊：建对象时按当时最空的 w 块选一组，之后该对象固定用这组（2026-08-31 第二轮补臂）。
    PinnedByEmptiest,
    /// 己：戊 + 组内有盘满了就重选一组（2026-08-31 第三轮补臂）。
    /// 本地腿指出戊的硬伤：组内盘满而池里还有空间 ⇒ 对象写不进去，
    /// 那正是 pitfalls 第 1 条（ENOSPC 荒谬）与 D3 已定「全局统一分配器」要避的形态。
    PinnedPreferred,
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

/// 从 `start` 开始连续取 `WIDTH` 块（环绕）。
fn contiguous(start: usize) -> Vec<usize> {
    (0..WIDTH).map(|i| (start + i) % DEVS).collect()
}

/// 生成所有条带的参与集合、各盘用量、以及每条条带属于哪个对象。
///
/// `skew=true` 时对象大小倾斜：每 16 个对象里 1 个占 64 条带、其余占 1 条带
/// ——量的是「对象绑一组盘」的均衡度会不会塌。
fn layout(pick: Pick, seed: u64, width: usize, skew: bool) -> (Vec<Vec<usize>>, Vec<u64>, Vec<usize>) {
    let (a, b, c, _) = layout_full(pick, seed, width, skew);
    (a, b, c)
}

fn layout_full(pick: Pick, seed: u64, width: usize, skew: bool) -> (Vec<Vec<usize>>, Vec<u64>, Vec<usize>, u64) {
    let mut rng = Lcg(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let mut used = vec![0u64; DEVS];
    let mut stranded_writes = 0u64;
    let mut sets = Vec::new();
    let mut owner = Vec::new();
    for obj in 0..OBJECTS {
        let n = if skew {
            if obj % 16 == 0 { 64 } else { 1 }
        } else {
            STRIPES_PER_OBJ
        };
        let mut pinned: Option<Vec<usize>> = None;
        for s in 0..n {
            let set: Vec<usize> = if width == DEVS {
                (0..DEVS).collect()
            } else {
                match pick {
                    Pick::ObjectPinned => contiguous(obj % DEVS),
                    Pick::RoundRobin => contiguous((obj * STRIPES_PER_OBJ + s) % DEVS),
                    Pick::Emptiest => {
                        let mut d: Vec<usize> = (0..DEVS).collect();
                        d.sort_by_key(|&i| (used[i], i));
                        d.truncate(width);
                        d
                    }
                    Pick::PinnedByEmptiest => pinned.clone().unwrap_or_else(|| {
                        let mut d: Vec<usize> = (0..DEVS).collect();
                        d.sort_by_key(|&i| (used[i], i));
                        d.truncate(width);
                        d
                    }),
                    Pick::PinnedPreferred => {
                        let fits = |g: &Vec<usize>| g.iter().all(|&i| used[i] < CAPS[i]);
                        match pinned.clone() {
                            Some(g) if fits(&g) => g,
                            _ => {
                                let mut d: Vec<usize> = (0..DEVS).collect();
                                d.sort_by_key(|&i| (used[i], i));
                                d.truncate(width);
                                d
                            }
                        }
                    }
                    Pick::Random => {
                        let mut d: Vec<usize> = (0..DEVS).collect();
                        for i in 0..width {
                            let j = i + (rng.next() as usize) % (DEVS - i);
                            d.swap(i, j);
                        }
                        d.truncate(width);
                        d
                    }
                }
            };
            if pick == Pick::PinnedByEmptiest && pinned.is_none() {
                pinned = Some(set.clone());
            }
            if pick == Pick::PinnedPreferred {
                // 每次都记下这次实际用的组：满了换组之后跟着换
                pinned = Some(set.clone());

            }
            let pool_has_room = (0..DEVS).filter(|&i| used[i] < CAPS[i]).count() >= width;
            if pool_has_room && set.iter().any(|&i| used[i] >= CAPS[i]) {
                stranded_writes += 1;
            }
            for &x in &set {
                used[x] += 1;
            }
            sets.push(set);
            owner.push(obj);
        }
    }
    (sets, used, owner, stranded_writes)
}

/// 穷举全部两盘同坏组合，返回（完好对象比例 ppm，期望丢数据比例 ppm）。
fn evaluate(sets: &[Vec<usize>], owner: &[usize], width: usize) -> (u64, u64) {
    let pairs: Vec<(usize, usize)> =
        (0..DEVS).flat_map(|a| ((a + 1)..DEVS).map(move |b| (a, b))).collect();
    let mut intact_sum = 0u64;
    let mut lost_sum = 0u64;
    let total_data = (sets.len() * (width - 1)) as u64;
    for &(a, b) in &pairs {
        let mut hit_obj = vec![false; OBJECTS];
        let mut lost = 0u64;
        for (i, set) in sets.iter().enumerate() {
            if set.contains(&a) && set.contains(&b) {
                hit_obj[owner[i]] = true;
                // 两格被抹，其中平均 2(w−1)/w 格是数据；用整数记成分子
                lost += 2 * (width as u64 - 1);
            }
        }
        let intact = hit_obj.iter().filter(|h| !**h).count() as u64;
        intact_sum += intact * 1_000_000 / OBJECTS as u64;
        // lost 的分母：total_data × w（把 2(w−1)/w 的 /w 一起算进去）
        lost_sum += lost * 1_000_000 / (total_data * width as u64);
    }
    let n = pairs.len() as u64;
    (intact_sum / n, lost_sum / n)
}

fn spread(used: &[u64]) -> u64 {
    used.iter().max().unwrap() - used.iter().min().unwrap()
}


/// 第四轮（2026-08-31 补）：**加盘 + D2 已定项 4 c 的重平衡之后，完好比例还剩多少。**
///
/// 两条腿都指到同一处：组开出来之后加了盘，重平衡要搬数据 ⇒ 组还成不成立。
/// 搬迁粒度是唯一的自变量：**整个对象重落一组** vs **逐条带搬**。
///
/// 口径：先在 8 盘上按戊臂放好，再加第 9 块盘，搬到新盘拿到 1/9 的量为止，
/// 然后对全部 `C(9,2)=36` 对失败盘穷举。
fn round_add_device(whole_object: bool) -> (u64, u64) {
    const D2: usize = DEVS + 1;
    let (mut sets, mut used, owner) = layout(Pick::PinnedByEmptiest, 1, WIDTH, false);
    let target_new = (sets.len() * WIDTH) / D2; // 新盘该拿到的格数
    let newdev = DEVS;
    let mut moved_cells = 0usize;
    if whole_object {
        // 整对象重落：为该对象重选一组（含新盘），把它全部条带换成新组
        let mut obj = 0usize;
        while moved_cells < target_new && obj < OBJECTS {
            let mut d: Vec<usize> = (0..D2).collect();
            d.sort_by_key(|&i| (*used.get(i).unwrap_or(&0), i));
            let group: Vec<usize> = d.into_iter().take(WIDTH).collect();
            if group.contains(&newdev) {
                for (i, set) in sets.iter_mut().enumerate() {
                    if owner[i] == obj {
                        for &x in set.iter() {
                            used[x] -= 1;
                        }
                        *set = group.clone();
                        for &x in set.iter() {
                            if x >= used.len() {
                                used.resize(x + 1, 0);
                            }
                            used[x] += 1;
                        }
                        moved_cells += 1;
                    }
                }
            }
            obj += 1;
        }
    } else {
        // 逐条带搬：重平衡器不认识「对象」，它按条带扫 ⇒ 挪到的条带**散布在所有对象上**。
        // ⚠️ 第一版按顺序扫前 N 条，而条带在数组里是按对象聚在一起的
        // ⇒ 它等价于整对象搬，两条臂给出同一个数（83.3% vs 83.3%）。那是建模错误不是结论。
        let stride = 9usize;
        let n = sets.len();
        let mut idx = 0usize;
        while moved_cells < target_new && idx < n * stride {
            let k = (idx * stride) % n;
            let set = &mut sets[k];
            idx += 1;
            {
            if let Some(pos) = set.iter().position(|&x| x != newdev) {
                if !set.contains(&newdev) {
                    let old = set[pos];
                    used[old] -= 1;
                    set[pos] = newdev;
                    if used.len() <= newdev {
                        used.resize(newdev + 1, 0);
                    }
                    used[newdev] += 1;
                    moved_cells += 1;
                }
            }
            }
        }
    }
    // 对全部 C(9,2) 对穷举
    let pairs: Vec<(usize, usize)> =
        (0..D2).flat_map(|a| ((a + 1)..D2).map(move |b| (a, b))).collect();
    let mut intact_sum = 0u64;
    for &(a, b) in &pairs {
        let mut hit = vec![false; OBJECTS];
        for (i, set) in sets.iter().enumerate() {
            if set.contains(&a) && set.contains(&b) {
                hit[owner[i]] = true;
            }
        }
        intact_sum += hit.iter().filter(|h| !**h).count() as u64 * 1_000_000 / OBJECTS as u64;
    }
    (intact_sum / pairs.len() as u64, moved_cells as u64)
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config devs={DEVS} width={WIDTH} objects={OBJECTS} \
             stripes_per_obj={STRIPES_PER_OBJ} pairs={} model=counting file_ops=0",
            DEVS * (DEVS - 1) / 2
        ))
    );
    let arms = [
        ("jia_object_pinned", Pick::ObjectPinned),
        ("yi_round_robin", Pick::RoundRobin),
        ("bing_emptiest", Pick::Emptiest),
        ("ding_random", Pick::Random),
        ("wu_pinned_by_emptiest", Pick::PinnedByEmptiest),
        ("ji_pinned_preferred", Pick::PinnedPreferred),
    ];
    for seed in 1..=5u64 {
        for (label, pick) in arms {
            let (sets, used, owner) = layout(pick, seed, WIDTH, false);
            let (intact, lost) = evaluate(&sets, &owner, WIDTH);
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=arm seed={seed} pick={label} intact_ppm={intact} \
                     lost_ppm={lost} dev_spread={} max_used={}",
                    spread(&used),
                    used.iter().max().unwrap()
                ))
            );
        }
    }
    // 第二轮：对象大小倾斜，看均衡度会不会塌。
    for (label, pick) in arms {
        let (sets, used, owner) = layout(pick, 1, WIDTH, true);
        let (intact, lost) = evaluate(&sets, &owner, WIDTH);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=skewed pick={label} intact_ppm={intact} lost_ppm={lost} \
                 dev_spread={} max_used={} spread_pct_ppm={}",
                spread(&used),
                used.iter().max().unwrap(),
                spread(&used) * 1_000_000 / used.iter().max().unwrap().max(&1),
            ))
        );
    }

    // 第三轮：异构容量（4 大 4 小）下，有多少条条带落到已满的盘上而池里还有空间。
    for (label, pick) in arms {
        let (_, used, _, stranded) = layout_full(pick, 1, WIDTH, false);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=heterogeneous pick={label} wrote_to_full_dev={stranded} max_used={} \
                 caps_exceeded={}",
                used.iter().max().unwrap(),
                (0..DEVS).filter(|&i| used[i] > CAPS[i]).count()
            ))
        );
    }

    // 第四轮：加盘 + 重平衡之后完好比例还剩多少。
    for (label, whole) in [("whole_object", true), ("per_stripe", false)] {
        let (intact, moved) = round_add_device(whole);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=add_device rebalance={label} intact_ppm={intact} moved_cells={moved}"
            ))
        );
    }

    // 阳性对照，对每一条臂都跑：全宽 ⇒ 完好比例必须全部为 0。
    for (label, pick) in arms {
        let (sets, _, owner) = layout(pick, 1, DEVS, false);
        let (intact, lost) = evaluate(&sets, &owner, DEVS);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=positive_control_full_width pick={label} intact_ppm={intact} lost_ppm={lost}"
            ))
        );
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **阳性对照，对每一条臂都跑**：全宽 ⇒ 每条带都含任意两块盘 ⇒ 完好比例恒 0。
    #[test]
    fn positive_control_full_width_every_arm() {
        for pick in [Pick::ObjectPinned, Pick::RoundRobin, Pick::Emptiest, Pick::Random, Pick::PinnedByEmptiest, Pick::PinnedPreferred] {
            let (sets, _, owner) = layout(pick, 1, DEVS, false);
            let (intact, _) = evaluate(&sets, &owner, DEVS);
            assert_eq!(intact, 0, "{pick:?}");
        }
    }

    /// **绝对值断言**：一条 w 宽条带被某一对指定盘打中的概率是 C(w,2)/C(D,2)。
    /// w=4、D=8 ⇒ 6/28；对象绑组时「完好」等价于「该对象那一组没被打中」
    /// ⇒ 完好比例应当接近 1 − 6/28 = 0.7857。
    #[test]
    fn absolute_object_pinned_intact_matches_pair_ratio() {
        let (sets, _, owner) = layout(Pick::ObjectPinned, 1, WIDTH, false);
        let (intact, _) = evaluate(&sets, &owner, WIDTH);
        let expect = 1_000_000 - 6 * 1_000_000 / 28;
        assert!(
            intact.abs_diff(expect) < 20_000,
            "intact={intact} expect≈{expect}"
        );
    }

    /// **绝对值断言（独立算术）**：轮转臂下一个对象的 8 条条带覆盖全部 8 个连续窗口。
    /// 一对环距为 d 的盘落在 `4−d` 个宽 4 的窗口里 ⇒ 只有环距 4 的那 4 对
    /// （{0,4} {1,5} {2,6} {3,7}）一个窗口都不落 ⇒ 完好比例恰好 `4/28`。
    #[test]
    fn absolute_round_robin_intact_is_four_over_28() {
        let (sets, _, owner) = layout(Pick::RoundRobin, 1, WIDTH, false);
        let (intact, _) = evaluate(&sets, &owner, WIDTH);
        assert_eq!(intact, 4 * 1_000_000 / 28);
    }

    /// **绝对值断言**：对象绑组臂恰好 `22/28`——1 减去「那一组含这对盘」的 `6/28`。
    #[test]
    fn absolute_object_pinned_is_22_over_28() {
        let (sets, _, owner) = layout(Pick::ObjectPinned, 1, WIDTH, false);
        assert_eq!(evaluate(&sets, &owner, WIDTH).0, 22 * 1_000_000 / 28);
    }

    /// **守恒**：期望丢数据比例只由宽度定，四臂必须彼此相等（差 ≤ 1%）。
    #[test]
    fn conservation_lost_fraction_equal_across_arms() {
        let mut v = Vec::new();
        for pick in [Pick::ObjectPinned, Pick::RoundRobin, Pick::Emptiest, Pick::Random, Pick::PinnedByEmptiest, Pick::PinnedPreferred] {
            let (sets, _, owner) = layout(pick, 1, WIDTH, false);
            v.push(evaluate(&sets, &owner, WIDTH).1);
        }
        let (lo, hi) = (*v.iter().min().unwrap(), *v.iter().max().unwrap());
        assert!(hi - lo <= hi / 100, "各臂丢数据比例不等：{v:?}");
    }


    /// **金标（钉绝对值）**：戊臂两轮都是 22/28，且倾斜轮的盘间极差恰好 0。
    #[test]
    fn golden_pinned_by_emptiest() {
        for skew in [false, true] {
            let (sets, used, owner) = layout(Pick::PinnedByEmptiest, 1, WIDTH, skew);
            assert_eq!(evaluate(&sets, &owner, WIDTH).0, 22 * 1_000_000 / 28, "skew={skew}");
            assert_eq!(spread(&used), 0, "skew={skew}：戊臂不该失衡");
        }
    }

    /// **对照**：甲（按对象号绑组）在倾斜负载下必须失衡——不失衡说明倾斜没生效。
    #[test]
    fn skew_actually_skews() {
        let (_, used, _) = layout(Pick::ObjectPinned, 1, WIDTH, true);
        assert!(spread(&used) * 100 > *used.iter().max().unwrap() * 50, "倾斜没造出失衡");
    }


    /// **绝对值断言（独立算术）**：期望丢数据比例恰好 `(w−1)/C(D,2)` = 3/28。
    /// 推导：条带数 `T/(w−1)`，每条被某对盘打中的概率 `C(w,2)/C(D,2)`，
    /// 打中时丢的数据格数是 `2(w−1)/w` ⇒ 相乘约得 `(w−1)/C(D,2)`。
    /// ⚠️ **守恒测试只做臂间互比，抓不到「所有臂一起错」**——变异 M4（把 w−1 写成 w）
    /// 实测只有这一条会红。
    #[test]
    fn absolute_lost_fraction_is_three_over_28() {
        for pick in [Pick::ObjectPinned, Pick::Emptiest, Pick::PinnedByEmptiest] {
            let (sets, _, owner) = layout(pick, 1, WIDTH, false);
            assert_eq!(evaluate(&sets, &owner, WIDTH).1, 3 * 1_000_000 / 28, "{pick:?}");
        }
    }

    /// **阴性对照**：宽度 2、单条带对象时完好比例 = 1 − C(2,2)/C(8,2) = 27/28。
    #[test]
    fn negative_control_width_two() {
        let pairs = DEVS * (DEVS - 1) / 2;
        assert_eq!(pairs, 28);
        let expect = 1_000_000 - 1_000_000 / 28;
        assert!(expect > 960_000 && expect < 970_000);
    }
}
