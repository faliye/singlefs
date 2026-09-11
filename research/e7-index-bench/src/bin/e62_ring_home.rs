//! E62：根环归属存身份 vs 算公式 —— D2 已定项 7。
//!
//! **用户定案（2026-08-31）取「逐区域显式存设备身份」，要求测性能与安全性。**
//!
//! ## 被引用条款逐字贴在这里
//!
//! - E48：`prime_stride` 落盘归属 = `(lba/chunk) % devs`，`lba_r = r × PRIME × chunk`，P = 8191
//!   ⇒ 归属 = `(r × PRIME) mod devs`。`gcd(PRIME, devs)=1` ⇒ 乘 P 是模 devs 的双射。
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

const PRIME: u64 = 8191;
const SEEDS: [u64; 5] = [1, 2, 3, 4, 5];
const REGION_COUNT: usize = 4;
const DEVICE_IDENTITY_BYTES: u64 = 1;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// 公式臂：归属按**当前** devs 算。
    Formula,
    /// 存身份臂：归属是 mkfs 时写下的设备身份，此后不再算。
    Stored,
}

fn formula_home(region_index: usize, device_count: u64) -> u64 {
    (region_index as u64 * PRIME) % device_count
}

/// 一轮设备集合变化序列的结果。
#[derive(Default, Debug)]
struct ArmOutcome {
    misdirect: u64,
    collisions: u64,
    extra_reads: u64,
}

/// `events`：每一步之后的设备数。存身份臂的归属恒等于 mkfs 那一刻算出来的那个。
fn run(arm: Arm, device_count_at_mkfs: u64, events: &[u64]) -> ArmOutcome {
    let placed: Vec<u64> = (0..REGION_COUNT).map(|region_index| formula_home(region_index, device_count_at_mkfs)).collect();
    let mut outcome = ArmOutcome::default();
    for &device_count in events {
        let home: Vec<u64> = match arm {
            Arm::Formula => (0..REGION_COUNT).map(|region_index| formula_home(region_index, device_count)).collect(),
            Arm::Stored => placed.clone(),
        };
        for region_index in 0..REGION_COUNT {
            // 指错盘：算出来的归属与数据实际所在的盘不同，且那块盘还在
            if home[region_index] != placed[region_index] && placed[region_index] < device_count {
                outcome.misdirect += 1;
            }
        }
        for first_region in 0..REGION_COUNT {
            for second_region in (first_region + 1)..REGION_COUNT {
                if home[first_region] == home[second_region] {
                    outcome.collisions += 1;
                }
            }
        }
        // 身份与超级块同址 ⇒ 定位根环不多读一次；公式臂同样不多读（它算一下就行）
        outcome.extra_reads += 0;
    }
    outcome
}

/// 种子决定设备集合怎么变：加盘 / 掉盘 / 换盘各若干步。
fn events_for(seed: u64, device_count_at_mkfs: u64) -> Vec<u64> {
    let mut device_counts_after_each_step = Vec::new();
    let mut current_device_count = device_count_at_mkfs;
    let mut xorshift_state = seed.wrapping_mul(0x9E3779B97F4A7C15) | 1;
    for _ in 0..8 {
        xorshift_state ^= xorshift_state << 13;
        xorshift_state ^= xorshift_state >> 7;
        xorshift_state ^= xorshift_state << 17;
        match xorshift_state % 3 {
            0 => current_device_count += 1,                    // 加盘
            1 => current_device_count = (current_device_count - 1).max(1),        // 掉盘
            _ => {}                          // 换盘：数目不变
        }
        device_counts_after_each_step.push(current_device_count);
    }
    device_counts_after_each_step
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config regions={REGION_COUNT} prime={PRIME} dev_id_bytes={DEVICE_IDENTITY_BYTES} \
             bytes_cost={} model=counting file_ops=0",
            REGION_COUNT as u64 * DEVICE_IDENTITY_BYTES
        ))
    );

    for &seed in SEEDS.iter() {
        let events = events_for(seed, 4);
        for (label, arm) in [("formula", Arm::Formula), ("stored", Arm::Stored)] {
            let outcome = run(arm, 4, &events);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=arm seed={seed} arm={label} steps={} misdirect={} \
                     collisions={} extra_reads={}",
                    events.len(),
                    outcome.misdirect,
                    outcome.collisions,
                    outcome.extra_reads
                ))
            );
        }
    }

    // 阳性对照，对每一条臂都跑：设备集合不变。
    for (label, arm) in [("formula", Arm::Formula), ("stored", Arm::Stored)] {
        let outcome = run(arm, 4, &[4, 4, 4, 4]);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=positive_control_no_change arm={label} misdirect={} collisions={}",
                outcome.misdirect, outcome.collisions
            ))
        );
    }
    // 阴性对照：mkfs 那一刻 R > devs ⇒ 两臂都撞。
    for (label, arm) in [("formula", Arm::Formula), ("stored", Arm::Stored)] {
        let outcome = run(arm, 3, &[3]);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=negative_control_pigeonhole arm={label} collisions={}",
                outcome.collisions
            ))
        );
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言**：R=4、devs 4→5，公式臂恰好 2 个区域指错盘。
    /// mkfs 归属 (0,3,2,1)；devs=5 时算出 (0,1,2,3) ⇒ r=1 与 r=3 变了。
    #[test]
    fn absolute_formula_misdirects_exactly_two_on_add() {
        assert_eq!(
            (0..4).map(|region_index| formula_home(region_index, 4)).collect::<Vec<_>>(),
            vec![0, 3, 2, 1]
        );
        assert_eq!(
            (0..4).map(|region_index| formula_home(region_index, 5)).collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(run(Arm::Formula, 4, &[5]).misdirect, 2);
        assert_eq!(run(Arm::Stored, 4, &[5]).misdirect, 0);
    }

    /// **绝对值断言**：字节代价 = 区域数 × 设备身份宽度 = 4 字节。
    #[test]
    fn absolute_bytes_cost() {
        assert_eq!(REGION_COUNT as u64 * DEVICE_IDENTITY_BYTES, 4);
    }

    /// **绝对值断言（穷举）**：`devs ≥ REGION_COUNT` 时公式臂归属两两不同；撞的全部是 `devs < REGION_COUNT`。
    #[test]
    fn absolute_collisions_only_when_device_count_below_region_count() {
        let mut colliding_cells = 0;
        for region_count in 2..=8usize {
            for device_count in 1..=64u64 {
                let homes: Vec<u64> = (0..region_count).map(|region_index| formula_home(region_index, device_count)).collect();
                let mut distinct_homes = homes.clone();
                distinct_homes.sort_unstable();
                distinct_homes.dedup();
                if distinct_homes.len() < region_count {
                    assert!(device_count < region_count as u64, "devs={device_count} ≥ R={region_count} 却撞了");
                    colliding_cells += 1;
                }
            }
        }
        assert_eq!(colliding_cells, 28, "撞的格子数变了");
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
