//! E28：明文映射层能不能只靠单元自描述重建 —— [checks-owed.md](checks-owed.md) C34。
//!
//! **问题**：仓里三处对「明文映射层是权威态还是派生态」说法不一致：
//!
//! | 处 | 说的是 |
//! |---|---|
//! | D9 已定项 5 定案 | 把物理地址的**权威记录点**移出加密结构 ⇒ 明文映射是权威 |
//! | D21 的分类表 | 权威态 = 单元 + 记账 + 根；索引是派生态 ⇒ 明文映射是派生 |
//! | I-6.10（明文映射层可重建） | checker **从权威侧重建**明文映射 ⇒ 预设它是派生，且重建源存在 |
//!
//! **若第一条成立，第三条不可能成立**（权威的东西没有可供重建的来源）。
//! D9 已定项 7 的定案采用「派生态、重建源是单元自描述」这个读法，
//! 而那个读法有一个未满足的前置：**D20 已指出数据块目前零自描述**。
//!
//! ## 本实验判什么
//!
//! 删掉明文映射，只扫单元头，能不能逐条重建出同一张映射。
//! **判据分元数据与数据两类分开报**——这正是那个前置成不成立的分界。
//!
//! ⚠️ **不是「能不能重建出一张表」，是「重建出的表与原表逐条相同」。**
//! 只判条数会让「重建出一张全错的表」也通过。

use e7_index_bench::Emitter;
use std::collections::BTreeMap;

/// 单元的自描述头。D20（承重面：单元的原子性与自包含）要求任一单元单独捡起来能回答
/// 「我是谁、我属于谁、我是第几代」。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct SelfDescription { object_number: u64, tree: u64, generation: u64 }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UnitClass { Metadata, Data }

/// 单元。⚠️ **这里不再存 `kind`**：单元是元数据还是数据这件事只在 `build` 返回的
/// 那张分类表里记一份——存两份会漂移，而扫描重建只看得到 `self_description`。
#[derive(Clone)]
struct Unit {
    physical_address: u64,                 // 物理地址
    /// 自描述头。**数据块今天是 None**——那正是 D20 点名的承重面上的洞。
    self_description: Option<SelfDescription>,
}

/// 明文映射层：逻辑身份 → 物理地址。
type PlaintextMap = BTreeMap<(u64, u64, u64), u64>;

/// 返回 `(单元表, 明文映射, 每个 key 是元数据还是数据)`。
///
/// ⚠️ **第三项不是冗余**：缺自描述的数据块在重建表里根本不出现，
/// 所以「缺的那一条是元数据还是数据」只能从这里查。
/// 曾经靠 `对象号 >= 10_000` 这个约定去分类，而 `Unit.kind` 同时也记着同一件事，
/// 编译器为此一直报 `kind` 从没被读过——**两份表示会漂移**，现在只留一份。
fn build(metadata_unit_count: u64, data_unit_count: u64, data_units_carry_self_description: bool) -> (Vec<Unit>, PlaintextMap, BTreeMap<(u64, u64, u64), UnitClass>) {
    let mut units = Vec::new();
    let mut plaintext_map = PlaintextMap::new();
    let mut unit_classes: BTreeMap<(u64, u64, u64), UnitClass> = BTreeMap::new();
    let mut physical_address = 1000u64;
    for metadata_index in 0..metadata_unit_count {
        let unit_description = SelfDescription { object_number: metadata_index, tree: metadata_index % 4, generation: 1 };
        units.push(Unit { physical_address, self_description: Some(unit_description) });
        plaintext_map.insert((unit_description.tree, unit_description.object_number, unit_description.generation), physical_address);
        unit_classes.insert((unit_description.tree, unit_description.object_number, unit_description.generation), UnitClass::Metadata);
        physical_address += 1;
    }
    for data_index in 0..data_unit_count {
        let unit_description = SelfDescription { object_number: 10_000 + data_index, tree: data_index % 4, generation: 1 };
        units.push(Unit { physical_address, self_description: if data_units_carry_self_description { Some(unit_description) } else { None } });
        plaintext_map.insert((unit_description.tree, unit_description.object_number, unit_description.generation), physical_address);
        unit_classes.insert((unit_description.tree, unit_description.object_number, unit_description.generation), UnitClass::Data);
        physical_address += 1;
    }
    (units, plaintext_map, unit_classes)
}

/// 只扫单元头重建映射。**没有自描述头的单元贡献不了任何条目。**
fn rebuild(units: &[Unit]) -> PlaintextMap {
    let mut rebuilt_map = PlaintextMap::new();
    for unit in units {
        if let Some(unit_description) = unit.self_description { rebuilt_map.insert((unit_description.tree, unit_description.object_number, unit_description.generation), unit.physical_address); }
    }
    rebuilt_map
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct RebuildTally {
    total: u64,
    rebuilt: u64,
    /// 原表里有、重建表里没有（重建不出来）
    missing: u64,
    /// 重建表里有、但指向了不同的物理地址（重建错了）
    wrong: u64,
    missing_metadata: u64,
    missing_data: u64,
}

fn measure(metadata_unit_count: u64, data_unit_count: u64, data_units_carry_self_description: bool) -> RebuildTally {
    measure_with_moved(metadata_unit_count, data_unit_count, data_units_carry_self_description, 0)
}

/// `moved_unit_count` = 有几个单元被搬走了（物理地址变了、自描述不动）。
/// 搬运正是本工程要支持的事（D1），所以「重建出的地址跟着搬」是**正确行为**；
/// 本参数用来喂那条「判据必须逐条比地址、不能只比条数」的测试。
fn measure_with_moved(metadata_unit_count: u64, data_unit_count: u64, data_units_carry_self_description: bool, moved_unit_count: usize) -> RebuildTally {
    let (mut units, original_map, unit_classes) = build(metadata_unit_count, data_unit_count, data_units_carry_self_description);
    for unit in units.iter_mut().take(moved_unit_count) { unit.physical_address += 999_000; }
    let rebuilt_map = rebuild(&units);
    let mut tally = RebuildTally { total: original_map.len() as u64, rebuilt: rebuilt_map.len() as u64, ..Default::default() };
    for (key, original_address) in &original_map {
        match rebuilt_map.get(key) {
            None => {
                tally.missing += 1;
                match unit_classes.get(key) {
                    // 没有 `_ =>` —— 新增单元类不补这里就编译不过
                    Some(UnitClass::Data) => tally.missing_data += 1,
                    Some(UnitClass::Metadata) => tally.missing_metadata += 1,
                    None => unreachable!("原表里的 key 必须有单元类"),
                }
            }
            Some(rebuilt_address) if rebuilt_address != original_address => tally.wrong += 1,
            _ => {}
        }
    }
    tally
}

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw("name=config note=只扫单元头重建明文映射"));
    for (metadata_unit_count, data_unit_count) in [(64u64, 256u64), (256, 1024), (1024, 4096)] {
        for data_units_carry_self_description in [false, true] {
            let tally = measure(metadata_unit_count, data_unit_count, data_units_carry_self_description);
            println!("{}", emitter.emit_raw(&format!(
                "name=cell meta={metadata_unit_count} data={data_unit_count} data_self_desc={data_units_carry_self_description} total={} rebuilt={} \
                 missing={} wrong={} missing_meta={} missing_data={}",
                tally.total, tally.rebuilt, tally.missing, tally.wrong, tally.missing_metadata, tally.missing_data)));
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **元数据侧：自描述在，重建必须逐条相同。**
    /// 这条证实 D21（权威态与派生态的分界）与 I-6.10（可从权威侧重建）在元数据上成立。
    #[test]
    fn metadata_rebuilds_exactly_because_units_are_self_describing() {
        for (metadata_unit_count, data_unit_count) in [(64u64, 256u64), (1024, 4096)] {
            let tally = measure(metadata_unit_count, data_unit_count, false);
            assert_eq!(tally.missing_metadata, 0, "元数据侧本该一条都不缺");
            assert_eq!(tally.wrong, 0, "重建出的地址本该逐条相同");
        }
    }

    /// **数据块侧：今天零自描述 ⇒ 一条都重建不出来。**
    /// 这是 D20（承重面：单元的原子性与自包含）点名的那个洞的可量形态，也是 D9 已定项 7 定案的未满足前置。
    #[test]
    fn data_blocks_rebuild_nothing_when_they_carry_no_self_description() {
        let tally = measure(64, 256, false);
        assert_eq!(tally.missing_data, 256, "数据块零自描述时本该一条都重建不出来");
        assert_eq!(tally.missing, 256, "缺的应当恰好是数据块那些");
        assert_eq!(tally.rebuilt, 64, "重建出的只有元数据那 64 条");
    }

    /// **阳性对照：给数据块加上自描述之后，缺口必须归零。**
    /// 少了这条，「数据块重建不出来」分不清是缺自描述还是重建代码根本不工作。
    #[test]
    fn giving_data_blocks_self_description_closes_the_gap_completely() {
        for (metadata_unit_count, data_unit_count) in [(64u64, 256u64), (256, 1024), (1024, 4096)] {
            let tally = measure(metadata_unit_count, data_unit_count, true);
            assert_eq!(tally.missing, 0, "加了自描述之后本该一条都不缺");
            assert_eq!(tally.wrong, 0);
            assert_eq!(tally.rebuilt, tally.total, "重建条数应等于原表条数");
        }
    }

    /// **绝对值：缺口恰好等于数据块数**，不是「有缺口」这种相对判断。
    #[test]
    fn the_gap_equals_exactly_the_number_of_data_blocks() {
        for data_unit_count in [256u64, 1024, 4096] {
            assert_eq!(measure(64, data_unit_count, false).missing, data_unit_count);
        }
    }

    /// **判据必须是「逐条相同」，不是「条数相同」。**
    ///
    /// ⚠️ **本测试第一版自己算 `wrong`，绕开了被测代码**——把 `measure` 里
    /// 统计地址错的那一行删掉，一个测试都不红（变异测试实测）。
    /// 现在走 `measure_with_moved`，判的是被测代码给出的那个数。
    #[test]
    fn the_verdict_compares_addresses_not_just_counts() {
        for moved_unit_count in [1usize, 3, 7] {
            let tally = measure_with_moved(16, 16, true, moved_unit_count);
            assert_eq!(tally.rebuilt, tally.total, "搬运不改变条数");
            assert_eq!(tally.wrong as usize, moved_unit_count,
                "搬走 {moved_unit_count} 个单元，逐条比对该报出 {moved_unit_count} 处地址不同，实测 {}", tally.wrong);
            assert_eq!(tally.missing, 0, "搬运不该造成缺条");
        }
    }
}
