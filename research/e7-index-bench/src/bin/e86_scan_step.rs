//! E86：扫描步进与两种单元大小 —— D18 已定项 9 的「按 32768 步进」撞上 D4 已定项 1 的
//! 「元数据侧 16 KiB」，三种修法各收得回多少单元头、各付什么。
//!
//! ## 为什么要有这个实验
//!
//! C86（两种单元大小与扫描步进对不拢）：D18（块里携带什么信息）已定项 9 逐字
//! 「扫描器按单元步进（32768 字节）」，而 D4（校验和位置）已定项 1 逐字「元数据侧 16 KiB、
//! 数据侧 32 KiB」（节点自己就是一个单元）——16 KiB 对齐的节点里一半的头不落在
//! 32768 步进点上，扫描重建（D21（权威态与派生态的分界）的唯一后备）会漏掉它们。
//! 修法候选有三，各自的收全率与代价要算出来才能收口。
//!
//! ## 模型
//!
//! 64 MiB 区域 = 4096 个 16 KiB 槽。混布数据单元（32 KiB，占 2 连续槽）与节点（16 KiB，1 槽），
//! 按 (放置政策 × 扫描步进) 全组合：
//!
//! | 修法臂 | 放置 | 步进 | 预期 |
//! |---|---|---|---|
//! | step16k_any | 数据 32K 对齐（D18 已定项 9 的硬要求），节点任意 16K 槽 | **16384** | 全收 |
//! | step32k_pad | 节点独占 32K 对齐槽对（后半空着） | 32768 | 全收，**节点空间 ×2** |
//! | step32k_packed | 节点任意 16K 槽（现状文字的组合） | 32768 | **漏掉奇数槽上的节点** |
//!
//! 另报一列「内部探针数」：步进落在数据单元载荷内部的次数——那里没有头，
//! 靠 magic + 头校验和分辨（E76（载荷校验和的判别力）口径），步进越细这类探针越多。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. step16k_any 收全率恒 100%（两类单元、全部种子）。
//! 2. step32k_packed 漏掉的恰是**起点在奇数槽的节点**（漏数 == 独立统计的奇槽节点数，
//!    两条路径不共享代码）；随机混布下约half，但判据是恒等式不是比例。
//! 3. step32k_pad 收全率 100%，节点占用槽数恰为节点数 × 2（浪费恒 50%）。
//! 4. 内部探针数闭式：step16k 恰为数据单元数（每单元载荷内 1 个 16K 探针点）；
//!    step32k 恒 0（数据 32K 对齐时步进点全是起点或节点/空槽）。
//! 5. 不判修法——那是 C86（两种单元大小与扫描步进对不拢）的收口决策，交数字。
//!
//! ## 它答不了的
//!
//! 计数模型，文件操作 0 处。「内部探针会不会被 payload 伪装的 magic 骗到」是概率问题
//! （头校验和 32 字节把它压到可忽略，E76（载荷校验和的判别力）同型），不建模；
//! 混布比例取 1:2（节点:数据单元的槽数比），是选定的场景点。

use e7_index_bench::Emitter;

const SLOTS: usize = 4096; // 16 KiB 槽

/// C59（种子折叠成同一个状态）教训：乘法混淆。
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xA076_1D64_78BD_642F);
        if state == 0 {
            state = 0xDEAD_BEEF;
        }
        Rng(state)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, exclusive_upper_bound: u64) -> u64 {
        self.next() % exclusive_upper_bound
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Slot {
    Free,
    NodeStart,
    DataStart,
    DataTail, // 数据单元的后半（载荷内部，没有头）
    NodePad,  // step32k_pad 政策下节点槽对的空后半
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ScanStepArm {
    Step16kAny,
    Step32kPad,
    Step32kPacked,
}
impl ScanStepArm {
    fn tag(self) -> &'static str {
        match self {
            ScanStepArm::Step16kAny => "step16k_any",
            ScanStepArm::Step32kPad => "step32k_pad",
            ScanStepArm::Step32kPacked => "step32k_packed",
        }
    }
    fn step_slots(self) -> usize {
        match self {
            ScanStepArm::Step16kAny => 1,
            ScanStepArm::Step32kPad | ScanStepArm::Step32kPacked => 2,
        }
    }
    /// 节点要不要独占 32K 对齐槽对。
    fn pad_nodes(self) -> bool {
        matches!(self, ScanStepArm::Step32kPad)
    }
}

/// 造盘：交替随机放节点与数据单元直到装不下。数据单元恒 32K 对齐（D18 已定项 9 硬要求）。
fn build(arm: ScanStepArm, seed: u64) -> Vec<Slot> {
    let mut rng = Rng::new(seed);
    let mut disk = vec![Slot::Free; SLOTS];
    let mut placed = 0;
    let mut fails = 0;
    while fails < 200 {
        let want_node = rng.below(3) == 0; // 槽数比 节点:数据 ≈ 1:2 的场景点
        let ok = if want_node {
            if arm.pad_nodes() {
                // 节点独占 32K 槽对
                let base = (rng.below((SLOTS / 2) as u64) * 2) as usize;
                if disk[base] == Slot::Free && disk[base + 1] == Slot::Free {
                    disk[base] = Slot::NodeStart;
                    disk[base + 1] = Slot::NodePad;
                    true
                } else {
                    false
                }
            } else {
                let at = rng.below(SLOTS as u64) as usize;
                if disk[at] == Slot::Free {
                    disk[at] = Slot::NodeStart;
                    true
                } else {
                    false
                }
            }
        } else {
            let base = (rng.below((SLOTS / 2) as u64) * 2) as usize;
            if disk[base] == Slot::Free && disk[base + 1] == Slot::Free {
                disk[base] = Slot::DataStart;
                disk[base + 1] = Slot::DataTail;
                true
            } else {
                false
            }
        };
        if ok {
            placed += 1;
            fails = 0;
        } else {
            fails += 1;
        }
    }
    let _ = placed;
    disk
}

/// 扫描：按步进探针，命中起点算找到。返回 (找到节点, 找到数据, 内部探针数)。
fn scan(disk: &[Slot], step_slots: usize) -> (u64, u64, u64) {
    let (mut nodes, mut data, mut interior) = (0u64, 0u64, 0u64);
    let mut slot_index = 0;
    while slot_index < disk.len() {
        match disk[slot_index] {
            Slot::NodeStart => nodes += 1,
            Slot::DataStart => data += 1,
            Slot::DataTail => interior += 1,
            Slot::Free | Slot::NodePad => {}
        }
        slot_index += step_slots;
    }
    (nodes, data, interior)
}

/// 独立真值：全盘逐槽数（与 scan 不共享步进逻辑）。
fn census(disk: &[Slot]) -> (u64, u64, u64) {
    let mut nodes = 0u64;
    let mut data = 0u64;
    let mut odd_nodes = 0u64;
    for (slot_index, slot) in disk.iter().enumerate() {
        match slot {
            Slot::NodeStart => {
                nodes += 1;
                if slot_index % 2 == 1 {
                    odd_nodes += 1;
                }
            }
            Slot::DataStart => data += 1,
            _ => {}
        }
    }
    (nodes, data, odd_nodes)
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!("name=config slots={SLOTS} slot_bytes=16384 model=counting file_ops=0"))
    );
    for arm in [ScanStepArm::Step16kAny, ScanStepArm::Step32kPad, ScanStepArm::Step32kPacked] {
        for seed in [11u64, 22, 33, 44, 55] {
            let disk = build(arm, seed);
            let (census_nodes, census_data, odd_nodes) = census(&disk);
            let (found_nodes, found_data, interior) = scan(&disk, arm.step_slots());
            let node_slots: u64 = disk
                .iter()
                .filter(|slot| matches!(slot, Slot::NodeStart | Slot::NodePad))
                .count() as u64;
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=scan arm={} seed={seed} nodes={census_nodes} data={census_data} found_nodes={found_nodes} found_data={found_data} missed_nodes={} odd_nodes={odd_nodes} interior_probes={interior} node_slots={node_slots}",
                    arm.tag(),
                    census_nodes - found_nodes
                ))
            );
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1**：16 KiB 步进全收（两类单元、全部种子）。
    #[test]
    fn step16k_finds_everything() {
        for seed in [11u64, 22, 33, 44, 55] {
            let disk = build(ScanStepArm::Step16kAny, seed);
            let (census_nodes, census_data, _) = census(&disk);
            let (found_nodes, found_data, _) = scan(&disk, 1);
            assert_eq!((found_nodes, found_data), (census_nodes, census_data), "seed={seed}");
        }
    }

    /// **判据 2 恒等式**：32 KiB 步进 + 节点任意 16K 槽 ⇒ 漏数恰等于奇槽节点数
    /// （独立 census 统计，与扫描不共享步进逻辑）。
    #[test]
    fn step32k_packed_misses_exactly_odd_nodes() {
        for seed in [11u64, 22, 33, 44, 55] {
            let disk = build(ScanStepArm::Step32kPacked, seed);
            let (census_nodes, census_data, odd_nodes) = census(&disk);
            let (found_nodes, found_data, _) = scan(&disk, 2);
            assert_eq!(census_nodes - found_nodes, odd_nodes, "seed={seed}");
            assert!(odd_nodes > 0, "场景里必须真的有奇槽节点，否则这格什么也没证");
            assert_eq!(found_data, census_data, "数据单元 32K 对齐，32K 步进收得全");
        }
    }

    /// **判据 3**：节点独占 32K 槽对 ⇒ 全收，且节点占用槽数恰为节点数 × 2。
    #[test]
    fn step32k_pad_finds_all_at_double_cost() {
        for seed in [11u64, 22, 33, 44, 55] {
            let disk = build(ScanStepArm::Step32kPad, seed);
            let (census_nodes, census_data, odd) = census(&disk);
            assert_eq!(odd, 0, "独占槽对下节点起点恒偶");
            let (found_nodes, found_data, _) = scan(&disk, 2);
            assert_eq!((found_nodes, found_data), (census_nodes, census_data));
            let node_slots = disk
                .iter()
                .filter(|slot| matches!(slot, Slot::NodeStart | Slot::NodePad))
                .count() as u64;
            assert_eq!(node_slots, census_nodes * 2, "浪费恒 50%");
        }
    }

    /// **判据 4 闭式**：内部探针——16K 步进恰为数据单元数；32K 步进恒 0。
    #[test]
    fn interior_probe_arithmetic() {
        for seed in [11u64, 22, 33] {
            let disk = build(ScanStepArm::Step16kAny, seed);
            let (_, census_data, _) = census(&disk);
            let (_, _, interior16) = scan(&disk, 1);
            assert_eq!(interior16, census_data, "每个数据单元载荷内恰 1 个 16K 探针点");
            let (_, _, interior32) = scan(&disk, 2);
            assert_eq!(interior32, 0, "数据 32K 对齐时 32K 步进点全是起点或节点/空槽");
        }
    }

    /// 手算锚点：4 槽小盘手工摆——节点在槽 1（奇）、数据在槽 2..3。
    /// 32K 步进（探 0、2）：漏节点、收数据；16K 步进全收。
    #[test]
    fn absolute_hand_case() {
        let disk = vec![Slot::Free, Slot::NodeStart, Slot::DataStart, Slot::DataTail];
        assert_eq!(scan(&disk, 2), (0, 1, 0));
        assert_eq!(scan(&disk, 1), (1, 1, 1));
        let (census_node_count, census_data_count, odd) = census(&disk);
        assert_eq!((census_node_count, census_data_count, odd), (1, 1, 1));
    }

    /// 臂到步进的映射本身要钉住——统计测试都直接传步进常量，绕过了这层映射，
    /// 映射写反时只有 main 的输出错（2026-09-02 变异测试实测 M5 漏网，补此测）。
    #[test]
    fn step_mapping_is_correct() {
        assert_eq!(ScanStepArm::Step16kAny.step_slots(), 1);
        assert_eq!(ScanStepArm::Step32kPad.step_slots(), 2);
        assert_eq!(ScanStepArm::Step32kPacked.step_slots(), 2);
        assert!(ScanStepArm::Step32kPad.pad_nodes());
        assert!(!ScanStepArm::Step16kAny.pad_nodes());
        assert!(!ScanStepArm::Step32kPacked.pad_nodes());
    }

    /// 不同种子不同盘；(2,3) 不折叠（C59（种子折叠成同一个状态））。
    #[test]
    fn seeds_differ() {
        let seed_2_census = census(&build(ScanStepArm::Step32kPacked, 2));
        let seed_3_census = census(&build(ScanStepArm::Step32kPacked, 3));
        assert!(seed_2_census != seed_3_census, "种子 2 与 3 折叠成了同一个盘");
    }

    /// 数据单元恒 32K 对齐（D18 已定项 9 的硬要求在模型里被遵守）——起点全在偶槽。
    #[test]
    fn data_units_are_32k_aligned() {
        for arm in [ScanStepArm::Step16kAny, ScanStepArm::Step32kPad, ScanStepArm::Step32kPacked] {
            let disk = build(arm, 11);
            for (slot_index, slot) in disk.iter().enumerate() {
                if *slot == Slot::DataStart {
                    assert_eq!(slot_index % 2, 0, "{arm:?} 数据单元起点在奇槽");
                }
            }
        }
    }
}
