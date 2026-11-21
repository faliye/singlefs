//! 本包真卡对拍两侧（参照一侧、GPU 一侧）的判法单元入口登记表（`tests/pairing_units_entries.tsv`，编进测试二进制）的读法：
//! 测试按侧取组名与入口，逐组对测试支撑包（`singlefs-background-gpu-pairing-keys`）编进来的键表核。算键的是那个包的构建脚本，
//! 它按路径读同一份登记表、自己解；这里另写一份解法，不与它共用：登记表写坏了、构建脚本解错了，两边对不上，核的那几条用例判红。
//! 两侧各是什么、各拼进哪一把键写在登记表的文件头。
#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]

/// 登记表的原文。
pub const PAIRING_UNITS_ENTRIES_TEXT: &str = include_str!("../pairing_units_entries.tsv");

/// 登记表一行是对拍的哪一侧。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairingSideOfTheEntries {
    /// `reference`：在 CPU 上算、只录那一段写进判定库的参照。
    Reference,
    /// `gpu-side`：判段持卡那一段，组名就是真卡对拍用例的名字。
    GpuSide,
}

/// 登记表里的一组。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairingSideUnitsEntryGroup {
    pub side: PairingSideOfTheEntries,
    /// 组名：测试取键时点的名字。
    pub group_name: &'static str,
    /// 编译目标：`test:<测试目标>`。
    pub target: &'static str,
    /// 收闭包时请求的特性。
    pub features: Vec<&'static str>,
    /// 入口 fn 路径：`singlefs_background_gpu::<模块路径>::<fn>`（测试目标里的路径头一段写包名）。
    pub entry_functions: Vec<&'static str>,
}

/// 登记表里的每一组，按登记的次序。
///
/// # Panics
/// 有一行不是五列、侧认不得：登记表写坏了。
#[must_use]
pub fn every_registered_group() -> Vec<PairingSideUnitsEntryGroup> {
    PAIRING_UNITS_ENTRIES_TEXT
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let [side_word, group_name, target, features, entry_functions]: [&'static str; 5] =
                line.split('\t')
                    .collect::<Vec<&'static str>>()
                    .try_into()
                    .unwrap_or_else(|columns: Vec<&'static str>| {
                        panic!(
                            "登记表一行不是五列（侧、组名、目标、特性、入口），读到 {} 列：{line:?}",
                            columns.len()
                        )
                    });
            let side = match side_word {
                "reference" => PairingSideOfTheEntries::Reference,
                "gpu-side" => PairingSideOfTheEntries::GpuSide,
                unrecognized => panic!("登记表一行的侧「{unrecognized}」认不得：{line:?}"),
            };
            PairingSideUnitsEntryGroup {
                side,
                group_name,
                target,
                features: features.split(',').map(str::trim).collect(),
                entry_functions: entry_functions.split(',').map(str::trim).collect(),
            }
        })
        .collect()
}

/// 登记表里 `side` 那一侧的组，按登记的次序。
#[must_use]
pub fn registered_groups_of_the_side(
    side: PairingSideOfTheEntries,
) -> Vec<PairingSideUnitsEntryGroup> {
    every_registered_group()
        .into_iter()
        .filter(|group| group.side == side)
        .collect()
}
