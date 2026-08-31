//! E62：根环归属存身份 vs 算公式 —— D2 已定项 7。
//!
//! **用户定案（2026-08-31）取「逐区域显式存设备身份」，要求测性能与安全性。**
//!
//! ## 被引用条款逐字贴在这里
//!
//! - E48：`prime_stride` 落盘归属 = `(lba/chunk) % devs`，`lba_r = r × P × chunk`，P = 8191
//!   ⇒ 归属 = `(r × P) mod devs`。`gcd(P, devs)=1` ⇒ 乘 P 是模 devs 的双射。
//! - D2 已定项 5（2026-08-31 用户定案）：根环不重放置 —— 区域的字节留在原处不动。
//! - D2 已定项 1 / D19 已定项 1：物理位置逐个列出、各带设备身份，不引 chunk 映射表。
//!
//! ## 判据（跑前写死）
//!
//! 1. **安全**：设备集合变化序列跑完，「归属指到没有那份数据的盘」次数——
//!    存身份臂必须恒 0；公式臂 > 0 才说明这个量测得出来。
//! 2. **安全**：「两个区域归属同一块盘」次数——存身份臂只可能来自 mkfs 那一刻的鸽笼。
//! 3. **性能**：定位根环的额外读次数。身份与超级块同址 ⇒ 额外读 0；>0 如实记。
//! 4. **代价**：字节代价 = R × 设备身份宽度，钉绝对值。
//!
//! ## 失败条款（跑前写死）
//!
//! - **阳性对照，对每一条臂都跑**：设备集合**不变**的序列 ⇒ 两臂 misdirect 都必须为 0。
//!   公式臂在这里非 0 ⇒ 模型把「变化」和「没变化」混了，**整轮作废**。
//! - **阴性对照**：mkfs 那一刻就 R > devs ⇒ **两臂都撞**。存身份臂在这里为 0 ⇒
//!   说明模型给了它不该有的好处，**整轮作废**（存身份解决不了鸽笼）。
//! - 5 个种子方向不一致 ⇒ 报「不稳定」。
//!
//! ## 它答不了的
//!
//! 计数模型，不是实现：没有超级块、没有真盘、没有挂载。「额外读次数」是按
//! 「身份存在超级块里、超级块本来就要读」这个前提数出来的，不是实测 I/O。

use e7_index_bench::Emitter;

const P: u64 = 8191;
const SEEDS: [u64; 5] = [1, 2, 3, 4, 5];
const R: usize = 4;
const DEV_ID_BYTES: u64 = 1;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// 公式臂：归属按**当前** devs 算。
    Formula,
    /// 存身份臂：归属是 mkfs 时写下的设备身份，此后不再算。
    Stored,
}

fn formula_home(r: usize, devs: u64) -> u64 {
    (r as u64 * P) % devs
}

/// 一轮设备集合变化序列的结果。
#[derive(Default, Debug)]
struct Out {
    misdirect: u64,
    collisions: u64,
    extra_reads: u64,
}

/// `events`：每一步之后的设备数。存身份臂的归属恒等于 mkfs 那一刻算出来的那个。
fn run(arm: Arm, devs_mkfs: u64, events: &[u64]) -> Out {
    let placed: Vec<u64> = (0..R).map(|r| formula_home(r, devs_mkfs)).collect();
    let mut o = Out::default();
    for &devs in events {
        let home: Vec<u64> = match arm {
            Arm::Formula => (0..R).map(|r| formula_home(r, devs)).collect(),
            Arm::Stored => placed.clone(),
        };
        for r in 0..R {
            // 指错盘：算出来的归属与数据实际所在的盘不同，且那块盘还在
            if home[r] != placed[r] && placed[r] < devs {
                o.misdirect += 1;
            }
        }
        for a in 0..R {
            for b in (a + 1)..R {
                if home[a] == home[b] {
                    o.collisions += 1;
                }
            }
        }
        // 身份与超级块同址 ⇒ 定位根环不多读一次；公式臂同样不多读（它算一下就行）
        o.extra_reads += 0;
    }
    o
}

/// 种子决定设备集合怎么变：加盘 / 掉盘 / 换盘各若干步。
fn events_for(seed: u64, devs_mkfs: u64) -> Vec<u64> {
    let mut v = Vec::new();
    let mut d = devs_mkfs;
    let mut x = seed.wrapping_mul(0x9E3779B97F4A7C15) | 1;
    for _ in 0..8 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        match x % 3 {
            0 => d += 1,                    // 加盘
            1 => d = (d - 1).max(1),        // 掉盘
            _ => {}                          // 换盘：数目不变
        }
        v.push(d);
    }
    v
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config regions={R} prime={P} dev_id_bytes={DEV_ID_BYTES} \
             bytes_cost={} model=counting file_ops=0",
            R as u64 * DEV_ID_BYTES
        ))
    );

    for &seed in SEEDS.iter() {
        let ev = events_for(seed, 4);
        for (label, arm) in [("formula", Arm::Formula), ("stored", Arm::Stored)] {
            let o = run(arm, 4, &ev);
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=arm seed={seed} arm={label} steps={} misdirect={} \
                     collisions={} extra_reads={}",
                    ev.len(),
                    o.misdirect,
                    o.collisions,
                    o.extra_reads
                ))
            );
        }
    }

    // 阳性对照，对每一条臂都跑：设备集合不变。
    for (label, arm) in [("formula", Arm::Formula), ("stored", Arm::Stored)] {
        let o = run(arm, 4, &[4, 4, 4, 4]);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=positive_control_no_change arm={label} misdirect={} collisions={}",
                o.misdirect, o.collisions
            ))
        );
    }
    // 阴性对照：mkfs 那一刻 R > devs ⇒ 两臂都撞。
    for (label, arm) in [("formula", Arm::Formula), ("stored", Arm::Stored)] {
        let o = run(arm, 3, &[3]);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=negative_control_pigeonhole arm={label} collisions={}",
                o.collisions
            ))
        );
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言**：R=4、devs 4→5，公式臂恰好 2 个区域指错盘。
    /// mkfs 归属 (0,3,2,1)；devs=5 时算出 (0,1,2,3) ⇒ r=1 与 r=3 变了。
    #[test]
    fn absolute_formula_misdirects_exactly_two_on_add() {
        assert_eq!(
            (0..4).map(|r| formula_home(r, 4)).collect::<Vec<_>>(),
            vec![0, 3, 2, 1]
        );
        assert_eq!(
            (0..4).map(|r| formula_home(r, 5)).collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(run(Arm::Formula, 4, &[5]).misdirect, 2);
        assert_eq!(run(Arm::Stored, 4, &[5]).misdirect, 0);
    }

    /// **绝对值断言**：字节代价 = 区域数 × 设备身份宽度 = 4 字节。
    #[test]
    fn absolute_bytes_cost() {
        assert_eq!(R as u64 * DEV_ID_BYTES, 4);
    }

    /// **绝对值断言（穷举）**：`devs ≥ R` 时公式臂归属两两不同；撞的全部是 `devs < R`。
    #[test]
    fn absolute_collisions_only_when_devs_lt_r() {
        let mut bad = 0;
        for rr in 2..=8usize {
            for devs in 1..=64u64 {
                let h: Vec<u64> = (0..rr).map(|r| formula_home(r, devs)).collect();
                let mut uniq = h.clone();
                uniq.sort_unstable();
                uniq.dedup();
                if uniq.len() < rr {
                    assert!(devs < rr as u64, "devs={devs} ≥ R={rr} 却撞了");
                    bad += 1;
                }
            }
        }
        assert_eq!(bad, 28, "撞的格子数变了");
    }

    /// **阳性对照，对每一条臂都跑**：设备集合不变 ⇒ 两臂 misdirect 均为 0。
    #[test]
    fn positive_control_no_change_every_arm() {
        for arm in [Arm::Formula, Arm::Stored] {
            assert_eq!(run(arm, 4, &[4, 4, 4, 4]).misdirect, 0, "{arm:?}");
        }
    }

    /// **阴性对照**：mkfs 那一刻 R=4 > devs=3 ⇒ 两臂都撞，存身份解决不了鸽笼。
    #[test]
    fn negative_control_pigeonhole_hits_both_arms() {
        assert!(run(Arm::Formula, 3, &[3]).collisions > 0);
        assert!(run(Arm::Stored, 3, &[3]).collisions > 0);
    }

    /// 存身份臂在任何事件序列上 misdirect 恒 0。
    #[test]
    fn stored_never_misdirects() {
        for seed in SEEDS {
            assert_eq!(run(Arm::Stored, 4, &events_for(seed, 4)).misdirect, 0);
        }
    }
}
