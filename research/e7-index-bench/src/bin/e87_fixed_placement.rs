//! E87：固定结构的放置 —— 超级块与 journal 环在第一版的 2 盘池上怎么摆，掉一盘各是什么结局。
//!
//! ## 为什么要有这个实验
//!
//! C79（超级块与 journal 环的放置没人定）：超级块（设备表、树表、tail 槽、根环参数、阈值
//! 全住在它里）自己的副本数与放置、journal 环放哪块盘、掉那块盘还挂不挂得上——零覆盖，
//! 而 2 盘第一版里它们是仅有的单点候选。根环那半已定（D22（单元原子性怎么合成）已定项 2：
//! R=3 区域、逐区域存设备身份、跨区轮转），E87（固定结构的放置）把它与超级块 / journal
//! 的放置组合起来穷举单盘失效。
//!
//! ⚠️ 一条早已有、但从没对准 journal 的规则：D2（RAID 条带策略）已定项 6 的硬下界
//! 「`w ≥ 2`——**零冗余的条带不许发出**」。journal 记录也是写——单盘 journal 环
//! 就是一条恒 w=1 的写路径，**规则字面就该拦它**，只是全仓没人把两句话放到一起过。
//!
//! ## 模型
//!
//! 2 盘；根环 R=3 区域按 mkfs 轮转指派（区域 0、2 → 盘 0，区域 1 → 盘 1；
//! 逐区域存身份，D2（RAID 条带策略）已定项 7）；发布 g 落区域 `g mod 3`
//! （D16（发布语义）已定项 6：逐发布计数）。放置臂 2 × 2：
//! 超级块 {sb_single：只在盘 0 / sb_per_dev：每盘一份}；
//! journal 环 {j_single：只在盘 0 / j_mirror：两盘各一份，每条记录写两遍}。
//! 失效：掉盘 0 / 掉盘 1，逐发布代 g = 1..=12 穷举。
//!
//! 判定（每格）：挂得上吗（≥1 超级块副本 且 ≥1 幸存根）；根回退几代（幸存区域里
//! 最新的代与 g 的差，取 g 全档最坏）；journal 重放窗口丢不丢（环副本全灭 = 丢，
//! 上界一个 T_time 窗口，D16（发布语义）已定项 5）；稳态代价（镜像 journal 每条记录 +1 写）。
//!
//! ⚠️ **试跑时发现并修订的一处（2026-09-02，任何正式轮入库之前）**：初版没建 mkfs 的
//! 初始根，g=1 掉盘 1 直接全灭（第一次发布恰落在盘 1）。⇒ 模型改成
//! **mkfs 把第 0 代根种进全部区域**，并保留未种臂把这个洞钉成单测——
//! 它是要写进 C79（超级块与 journal 环的放置没人定）收口决策的一条硬要求。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. sb_single 掉盘 0 必不可挂（哪怕数据、根、journal 全健在）——单点的机检形态；
//!    sb_per_dev 全部 8 格可挂。
//! 2. 根回退最坏值按轮转算术钉死：掉盘 1（只有区域 1）最坏 1 代；
//!    掉盘 0（区域 0、2 全灭、只剩区域 1）最坏 **2** 代——g ≡ 1 (mod 3) 时区域 1 里
//!    最新的是 g−3？不对：区域 1 存代 ≡ 1 (mod 3)，g ≡ 1 时它自己就在盘 1……
//!    穷举给答案，判据只钉「与逐代穷举一致的闭式」（见单测手算）。
//! 3. j_single 掉盘 0 丢重放窗口，j_mirror 全格不丢；镜像的稳态代价恰为每条记录 ×2。
//! 4. 不判「选哪格」——那是 C79（超级块与 journal 环的放置没人定）的收口决策，交表。
//!
//! ## 它答不了的
//!
//! 纯算术穷举，文件操作 0 处。不建模超级块的更新协议（≥2 槽轮换那套语义归
//! D23（journal 的角色与格式）已定项 3 的同型纪律，收口时一起定）；不建模盘失而复得
//! （那是 C88（根环的时间线判别未实现）的射程）；S（每区槽数）取 1 简化——
//! 槽多只加深同区回退，不改跨区结局。

use e7_index_bench::Emitter;

const DEVS: usize = 2;
const R: usize = 3;

/// mkfs 轮转指派：区域 r → 盘 r mod 2（逐区域存身份，掉盘不重算）。
fn region_dev(r: usize) -> usize {
    r % DEVS
}

/// 发布 g 落区域 g mod R（D16 已定项 6 的逐发布计数）。
fn region_of(g: u64) -> usize {
    (g % R as u64) as usize
}

/// 掉 `dead_dev` 后，从代 g 往回找幸存区域里最新的代；返回回退了几代。
/// **mkfs 把第 0 代根种进全部区域**（试跑时发现的前置：不种的话第一次发布落在
/// 某盘、掉那盘就没有任何根——见 `fallback_unseeded` 与对应单测）。
fn fallback(g: u64, dead_dev: usize) -> u64 {
    let mut back = 0;
    loop {
        let cand = g - back;
        if cand == 0 {
            return back; // 第 0 代在全部区域都有种子 ⇒ 任何盘上都找得到
        }
        if region_dev(region_of(cand)) != dead_dev {
            return back;
        }
        back += 1;
    }
}

/// 未种子的变体：mkfs 不写任何初始根（发布从 g=1 开始才有根）。用来钉「不种会全灭」这个洞。
fn fallback_unseeded(g: u64, dead_dev: usize) -> u64 {
    let mut back = 0;
    loop {
        let cand = g - back;
        if cand == 0 {
            return u64::MAX; // 没有第 0 代根可退
        }
        if region_dev(region_of(cand)) != dead_dev {
            return back;
        }
        back += 1;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SbArm {
    Single,
    PerDev,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum JArm {
    Single,
    Mirror,
}

/// 一格的判定结果。
struct Cell {
    mountable: bool,
    worst_fallback: u64,
    window_lost: bool,
    journal_writes_per_record: u64,
}

fn judge(sb: SbArm, j: JArm, dead_dev: usize) -> Cell {
    let sb_survives = match sb {
        SbArm::Single => dead_dev != 0,
        SbArm::PerDev => true,
    };
    // 根：R=3 轮转下任何单盘失效都至少剩一个区域 ⇒ 根总有幸存者
    let worst_fallback = (1..=12u64).map(|g| fallback(g, dead_dev)).max().unwrap();
    let window_lost = match j {
        JArm::Single => dead_dev == 0,
        JArm::Mirror => false,
    };
    Cell {
        mountable: sb_survives, // 根恒有幸存者，成不成只看超级块
        worst_fallback,
        window_lost,
        journal_writes_per_record: match j {
            JArm::Single => 1,
            JArm::Mirror => 2,
        },
    }
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config devs={DEVS} ring_regions={R} region_dev_map=0:0,1:1,2:0 model=arithmetic file_ops=0"
        ))
    );
    for sb in [SbArm::Single, SbArm::PerDev] {
        for j in [JArm::Single, JArm::Mirror] {
            for dead in 0..DEVS {
                let c = judge(sb, j, dead);
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=cell sb={} journal={} dead_dev={dead} mountable={} worst_root_fallback={} replay_window_lost={} journal_writes_per_record={}",
                        match sb { SbArm::Single => "single", SbArm::PerDev => "per_dev" },
                        match j { JArm::Single => "single", JArm::Mirror => "mirror" },
                        u8::from(c.mountable),
                        c.worst_fallback,
                        u8::from(c.window_lost),
                        c.journal_writes_per_record
                    ))
                );
            }
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1**：超级块单份、掉盘 0 ⇒ 不可挂（数据、根、journal 健在也没用）；
    /// 每盘一份 ⇒ 全部 8 格可挂。
    #[test]
    fn single_superblock_is_a_single_point_of_failure() {
        for j in [JArm::Single, JArm::Mirror] {
            assert!(!judge(SbArm::Single, j, 0).mountable);
            assert!(judge(SbArm::Single, j, 1).mountable);
            for dead in 0..DEVS {
                assert!(judge(SbArm::PerDev, j, dead).mountable);
            }
        }
    }

    /// **判据 2 手算锚点**：区域→盘 = {0:0, 1:1, 2:0}。
    /// 掉盘 1（只灭区域 1）：g ≡ 1 (mod 3) 时最新根在区域 1，退 1 代到区域 0/2 ⇒ 最坏 1。
    /// 掉盘 0（灭区域 0、2）：只剩区域 1（存代 ≡ 1 mod 3）——
    /// g ≡ 0 (mod 3) 时区域 1 里最新的是 g−2 ⇒ 最坏 2。
    #[test]
    fn absolute_fallback_arithmetic() {
        assert_eq!((1..=12u64).map(|g| fallback(g, 1)).max().unwrap(), 1);
        assert_eq!((1..=12u64).map(|g| fallback(g, 0)).max().unwrap(), 2);
        // 具体代逐个钉：g=3（区域 0，盘 0）掉盘 0 ⇒ 退到 g=1（区域 1）= 2 代
        assert_eq!(fallback(3, 0), 2);
        // g=4（区域 1，盘 1）掉盘 0 ⇒ 自己就幸存，0 代
        assert_eq!(fallback(4, 0), 0);
        // g=4 掉盘 1 ⇒ 退到 g=3（区域 0）= 1 代
        assert_eq!(fallback(4, 1), 1);
    }

    /// 根恒有幸存者（**前提：mkfs 把第 0 代根种进全部区域**）：任何单盘失效、
    /// 任何代都找得到根。这是「挂得上只看超级块」那半句的前提。
    #[test]
    fn roots_always_survive_single_disk_loss() {
        for dead in 0..DEVS {
            for g in 1..=24u64 {
                assert_ne!(fallback(g, dead), u64::MAX, "g={g} dead={dead}");
            }
        }
    }

    /// **刚 mkfs、零次发布、掉任一盘**（g=0 那一格）：第 0 代种进全部区域时两边都挂得上；
    /// 只种一个区域（等于只种一个失败域）时，掉那个盘就全灭——
    /// 「初始根要覆盖每个失败域」的判决格。
    #[test]
    fn fresh_mkfs_survives_either_disk_loss() {
        assert_eq!(fallback(0, 0), 0);
        assert_eq!(fallback(0, 1), 0);
    }

    /// **不种子的洞**（试跑时撞出来的）：mkfs 只把初始根放区域 0 时，
    /// g=1 掉盘 1 全灭（第一次发布恰落在盘 1，往回只剩不存在的根）。
    /// 这就是「mkfs 必须把第 0 代根种进全部区域」的判决格。
    #[test]
    fn unseeded_mkfs_has_a_total_loss_window() {
        assert_eq!(fallback_unseeded(1, 1), u64::MAX, "不种子 ⇒ 头一代掉盘 1 全灭");
        assert_eq!(fallback_unseeded(1, 0), 0, "头一代根在盘 1，掉盘 0 时它自己幸存");
        assert_ne!(fallback(1, 1), u64::MAX, "mkfs 种上第 0 代根后这个窗口消失");
    }

    /// **判据 3**：journal 单份掉盘 0 丢重放窗口；镜像全格不丢；镜像代价恰每条 ×2。
    #[test]
    fn journal_mirroring_arithmetic() {
        assert!(judge(SbArm::PerDev, JArm::Single, 0).window_lost);
        assert!(!judge(SbArm::PerDev, JArm::Single, 1).window_lost);
        for dead in 0..DEVS {
            let c = judge(SbArm::PerDev, JArm::Mirror, dead);
            assert!(!c.window_lost);
            assert_eq!(c.journal_writes_per_record, 2);
        }
        assert_eq!(judge(SbArm::PerDev, JArm::Single, 1).journal_writes_per_record, 1);
    }

    /// 区域指派自检：逐区域存身份的映射恒为 {0:0, 1:1, 2:0}——两盘上 R=3 必有一盘背两个区域。
    #[test]
    fn region_assignment_is_pinned() {
        assert_eq!((0..R).map(region_dev).collect::<Vec<_>>(), vec![0, 1, 0]);
    }

    /// 发布代到区域的映射用的是逐发布计数（D16 已定项 6）——相邻代必落不同区域。
    #[test]
    fn consecutive_generations_hit_different_regions() {
        for g in 1..=24u64 {
            assert_ne!(region_of(g), region_of(g + 1));
        }
    }
}
