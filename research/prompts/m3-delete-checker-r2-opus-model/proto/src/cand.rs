//! 每个候选读法在一份镜像上的判定：数据取池级 checker 删除态扫描要判的那几版（副本 walk.rs 的 attack_stash，与 I-9.5 共用判法同一个版本清单）。
//! 名字：T2a 按号、T2c 按号且只取谱系上的；A = 每版自己的 txg 与谱系树（T3-a）、B = 一律用所选根（最新那条根）的（T3-b）；
//! pub = 再按已发布谓词滤（T4「滤」）；each = 每版各比自己的；sel×map = 所选根墓碑对 checker 跨版本的号→出生代表。
use std::collections::{BTreeMap, BTreeSet};

use singlefs_checker::walk::attack_stash::{Facts, Version};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum V {
    Holds,
    Red(String),
    Na,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tomb {
    pub object: u64,
    pub birth: u64,
    pub start: u64,
    pub end: u64,
    pub death: u64,
    pub published: bool,
}

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

pub fn tombs(version: &Version) -> Vec<Tomb> {
    version
        .tombstone_containers
        .iter()
        .flat_map(|container| {
            container.records.iter().map(move |record| Tomb {
                object: u64_at(record, 0),
                birth: u64_at(record, 8),
                start: u64_at(record, 16),
                end: u64_at(record, 24),
                death: u64_at(record, 32),
                published: container.published,
            })
        })
        .collect()
}

pub fn lineage(version: &Version) -> Vec<(u64, u64)> {
    version
        .lineage_containers
        .iter()
        .flat_map(|container| container.records.iter().map(|record| (u64_at(record, 0), u64_at(record, 8))))
        .collect()
}

/// 链：从 `txg` 往回走，有谱系记录 (T_rb, T_old) 的 T_rb 的父是 T_old，其余 txg 的父是 txg − 1。交回链上的闭区间；谱系不合法交回 None。
pub fn chain(txg: u64, records: &[(u64, u64)]) -> Option<Vec<(u64, u64)>> {
    let mut sorted = records.to_vec();
    sorted.sort_unstable_by(|left, right| right.cmp(left));
    let mut cur = txg;
    let mut segments = Vec::new();
    for (rb, old) in sorted {
        if rb > cur {
            continue;
        }
        if old >= rb {
            return None;
        }
        segments.push((rb, cur));
        cur = old;
    }
    segments.push((0, cur));
    Some(segments)
}

pub fn on_chain(segments: &[(u64, u64)], txg: u64) -> bool {
    segments.iter().any(|(lo, hi)| *lo <= txg && txg <= *hi)
}

fn fold(results: impl Iterator<Item = (bool, String)>) -> V {
    let mut seen = false;
    for (holds, why) in results {
        seen = true;
        if !holds {
            return V::Red(why);
        }
    }
    if seen {
        V::Holds
    } else {
        V::Na
    }
}

fn records_by_number(version: &Version) -> BTreeMap<u64, Vec<(u32, u64, u64)>> {
    let mut map: BTreeMap<u64, Vec<(u32, u64, u64)>> = BTreeMap::new();
    for record in &version.inode_records {
        map.entry(record.inode).or_default().push((record.nlink, record.size, record.birth));
    }
    map
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Root {
    Own,
    Selected,
    NoFilter,
    /// 每版自己的 txg 起算，谱系记录取所选根那一版的谱系树（攻方补的 A2，零轮）。
    OwnTxgSelectedLineage,
}

/// 一版的「谱系上」判定器：None 表示这一版谱系不合法（判不了）。
fn chain_of(facts: &Facts, version: &Version, root: Root) -> Option<Option<Vec<(u64, u64)>>> {
    match root {
        Root::Own => Some(chain(version.txg, &lineage(version))),
        Root::Selected => {
            let newest = facts.versions.iter().find(|candidate| candidate.is_newest)?;
            Some(chain(newest.txg, &lineage(newest)))
        }
        Root::NoFilter => Some(Some(vec![(0, u64::MAX)])),
        Root::OwnTxgSelectedLineage => {
            let newest = facts.versions.iter().find(|candidate| candidate.is_newest)?;
            Some(chain(version.txg, &lineage(newest)))
        }
    }
}

fn judged_versions(facts: &Facts) -> impl Iterator<Item = &Version> {
    facts.versions.iter().filter(|version| version.has_trees && version.gap.is_none())
}

fn i918(facts: &Facts, root: Root, published_only: bool) -> V {
    fold(judged_versions(facts).flat_map(|version| {
        let records = records_by_number(version);
        let tombs = tombs(version);
        let segments = chain_of(facts, version, root).flatten();
        let queue = version.queue.clone().unwrap_or_default();
        queue
            .into_iter()
            .filter_map(|number| {
                let record = records.get(&number)?.first().copied()?;
                let segments = segments.as_ref()?;
                let found = tombs.iter().any(|tomb| {
                    tomb.object == number
                        && tomb.birth == record.2
                        && tomb.start == 0
                        && tomb.end == u64::MAX
                        && on_chain(segments, tomb.death)
                        && (!published_only || tomb.published)
                });
                Some((found, format!("{}：队列号 {number}（出生代 {}）没有谱系上的整对象墓碑", version.name, record.2)))
            })
            .collect::<Vec<_>>()
    }))
}

fn i919(facts: &Facts, root: Root, published_only: bool) -> V {
    fold(judged_versions(facts).flat_map(|version| {
        let records = records_by_number(version);
        let segments = chain_of(facts, version, root).flatten();
        tombs(version)
            .into_iter()
            .filter(|tomb| !published_only || tomb.published)
            .filter_map(|tomb| {
                let segments = segments.as_ref()?;
                if !on_chain(segments, tomb.death) {
                    return None;
                }
                let alive: Vec<_> = records.get(&tomb.object)?.iter().filter(|record| record.2 == tomb.birth).copied().collect();
                if alive.is_empty() {
                    return None;
                }
                Some((
                    alive.iter().all(|record| record.0 == 0),
                    format!("{}：谱系上的墓碑 ({}, {}, 死于 {}) 指着 nlink {} 的记录", version.name, tomb.object, tomb.birth, tomb.death, alive[0].0),
                ))
            })
            .collect::<Vec<_>>()
    }))
}

fn i920(facts: &Facts, root: Root) -> V {
    fold(judged_versions(facts).flat_map(|version| {
        let segments = chain_of(facts, version, root).flatten();
        let mut seen: BTreeMap<(u64, u64), u64> = BTreeMap::new();
        let mut out = Vec::new();
        let Some(segments) = segments else {
            return out;
        };
        for tomb in tombs(version) {
            if !on_chain(&segments, tomb.death) {
                continue;
            }
            let first = seen.insert((tomb.object, tomb.birth), tomb.death);
            out.push((
                first.is_none(),
                format!("{}：谱系上的墓碑 ({}, {}) 两条（死于 {:?} 与 {}）", version.name, tomb.object, tomb.birth, first, tomb.death),
            ));
            out.push((tomb.start == 0 && tomb.end == u64::MAX, format!("{}：墓碑区间 [{}, {}]", version.name, tomb.start, tomb.end)));
        }
        out
    }))
}

fn t2_each(facts: &Facts, root: Root) -> V {
    fold(judged_versions(facts).flat_map(|version| {
        let records = records_by_number(version);
        let segments = chain_of(facts, version, root).flatten();
        tombs(version)
            .into_iter()
            .filter_map(|tomb| {
                let segments = segments.as_ref()?;
                if !on_chain(segments, tomb.death) {
                    return None;
                }
                let births: Vec<u64> = records.get(&tomb.object)?.iter().map(|record| record.2).collect();
                Some((
                    births.iter().all(|birth| *birth == tomb.birth),
                    format!("{}：号 {} 的墓碑出生代 {} 而记录 {:?}", version.name, tomb.object, tomb.birth, births),
                ))
            })
            .collect::<Vec<_>>()
    }))
}

fn t2_selected_against_the_map(facts: &Facts, root: Root) -> V {
    let Some(newest) = facts.versions.iter().find(|version| version.is_newest) else {
        return V::Na;
    };
    if newest.gap.is_some() {
        return V::Na;
    }
    let segments = match root {
        Root::NoFilter => vec![(0, u64::MAX)],
        _ => match chain(newest.txg, &lineage(newest)) {
            Some(segments) => segments,
            None => return V::Na,
        },
    };
    fold(tombs(newest).into_iter().filter(|tomb| tomb.published && on_chain(&segments, tomb.death)).filter_map(|tomb| {
        let birth = facts.inode_object_birth.get(&tomb.object)?;
        Some((*birth == tomb.birth, format!("所选根墓碑 ({}, {}) 对 checker 跨版本表里 {}", tomb.object, tomb.birth, birth)))
    }))
}

/// I-9.20 区间那一句管全部墓碑（T14「全部」），不重号那一句仍只管谱系上的（T3-a）。
fn i920_interval_all(facts: &Facts) -> V {
    fold(judged_versions(facts).flat_map(|version| {
        tombs(version)
            .into_iter()
            .map(|tomb| (tomb.start == 0 && tomb.end == u64::MAX, format!("{}：墓碑 ({}, {}) 区间 [{}, {}]", version.name, tomb.object, tomb.birth, tomb.start, tomb.end)))
            .collect::<Vec<_>>()
    }))
}

/// 攻方补的零轮候选：墓碑记录的死亡 txg 落在 [对象出生代, 这一版的 txg] 里（每版、全部墓碑）。
fn tomb_death_bounds(facts: &Facts) -> V {
    fold(judged_versions(facts).flat_map(|version| {
        tombs(version)
            .into_iter()
            .map(|tomb| {
                (
                    tomb.birth <= tomb.death && tomb.death <= version.txg,
                    format!("{}：墓碑 ({}, {}) 死于 {}", version.name, tomb.object, tomb.birth, tomb.death),
                )
            })
            .collect::<Vec<_>>()
    }))
}

fn i924(facts: &Facts) -> V {
    fold(facts.versions.iter().filter(|version| version.has_trees).filter_map(|version| {
        let queue = version.queue.as_ref()?;
        Some((queue.len() <= 1, format!("{}：队列 {:?}", version.name, queue)))
    }))
}

fn i921(facts: &Facts, only_selected: bool) -> V {
    fold(judged_versions(facts).filter(|version| !only_selected || version.is_newest).flat_map(|version| {
        let containers = &version.tombstone_containers;
        let mut out = Vec::new();
        for pair in containers.windows(2) {
            out.push((pair[0].container < pair[1].container, format!("{}：容器号 {} 不小于 {}", version.name, pair[0].container, pair[1].container)));
        }
        if let Some((_, rest)) = containers.split_last() {
            for container in rest {
                out.push((container.records.len() == 815, format!("{}：非最右容器 {} 只有 {} 条", version.name, container.container, container.records.len())));
            }
        }
        out
    }))
}

/// C479-乙：候选集里每一版的记录（加谱系上的墓碑，T3-a）号 → 出生代不许两样。
fn c479_yi(facts: &Facts, with_tombs: bool) -> V {
    let mut births: BTreeMap<u64, BTreeSet<(u64, String)>> = BTreeMap::new();
    let mut seen = false;
    for version in judged_versions(facts) {
        seen = true;
        for record in &version.inode_records {
            births.entry(record.inode).or_default().insert((record.birth, version.name.clone()));
        }
        if with_tombs {
            if let Some(segments) = chain(version.txg, &lineage(version)) {
                for tomb in tombs(version) {
                    if on_chain(&segments, tomb.death) {
                        births.entry(tomb.object).or_default().insert((tomb.birth, format!("{}的墓碑", version.name)));
                    }
                }
            }
        }
    }
    for (number, set) in &births {
        let distinct: BTreeSet<u64> = set.iter().map(|(birth, _)| *birth).collect();
        if distinct.len() > 1 {
            return V::Red(format!("号 {number} 在候选集里对应出生代 {set:?}"));
        }
    }
    if seen {
        V::Holds
    } else {
        V::Na
    }
}

/// C479-丙：每版 v 与它的父版本 p 都在候选集里时，v 里有、p 里没有的号 ≥ p 的水位；两边都有的号出生代相同。
/// `by_lineage`：父按谱系（T_rb 的父是 T_old）；否则一律 txg − 1。父按 txg 找（同一个 txg 有几版的，几版都比）。
fn c479_bing(facts: &Facts, by_lineage: bool) -> V {
    let versions: Vec<&Version> = judged_versions(facts).collect();
    fold(versions.iter().flat_map(|version| {
        let records = lineage(version);
        let parent_txg = if by_lineage {
            records.iter().find(|(rb, _)| *rb == version.txg).map_or(version.txg.checked_sub(1), |(_, old)| Some(*old))
        } else {
            version.txg.checked_sub(1)
        };
        let mut out = Vec::new();
        let Some(parent_txg) = parent_txg else {
            return out;
        };
        for parent in versions.iter().filter(|candidate| candidate.txg == parent_txg) {
            let Some(watermark) = parent.watermark else {
                continue;
            };
            let parent_numbers = records_by_number(parent);
            for record in &version.inode_records {
                match parent_numbers.get(&record.inode) {
                    None => out.push((
                        record.inode >= watermark,
                        format!("{}：新号 {} 小于父版本 {} 的水位 {watermark}", version.name, record.inode, parent.name),
                    )),
                    Some(parent_records) => out.push((
                        parent_records.iter().all(|parent_record| parent_record.2 == record.birth),
                        format!("{}：号 {} 出生代 {} 与父版本 {} 的 {:?} 不同", version.name, record.inode, record.birth, parent.name, parent_records),
                    )),
                }
            }
        }
        out
    }))
}

/// I-9.6 树内最大 key 的三种取法（T12）：code = checker 今天走读到的每一版里最大的（跨版本）对最新根水位；newest = 只取最新根自己的；
/// each = 每版自己的水位对自己的最大 key。
fn i96(facts: &Facts, how: &str) -> V {
    match how {
        "code" => match (facts.newest_watermark, facts.largest_key_walked) {
            (Some(watermark), Some(largest)) => {
                if watermark > largest {
                    V::Holds
                } else {
                    V::Red(format!("水位 {watermark} ≤ 跨版本最大 key {largest}"))
                }
            }
            _ => V::Na,
        },
        "newest" => {
            let Some(newest) = facts.versions.iter().find(|version| version.is_newest) else {
                return V::Na;
            };
            match (facts.newest_watermark, newest.inode_records.iter().map(|record| record.inode).max()) {
                (Some(watermark), Some(largest)) => {
                    if watermark > largest {
                        V::Holds
                    } else {
                        V::Red(format!("水位 {watermark} ≤ 最新根最大 key {largest}"))
                    }
                }
                _ => V::Na,
            }
        }
        _ => fold(judged_versions(facts).filter_map(|version| {
            let watermark = version.watermark?;
            let largest = version.inode_records.iter().map(|record| record.inode).max()?;
            Some((watermark > largest, format!("{}：水位 {watermark} ≤ 自己最大 key {largest}", version.name)))
        })),
    }
}

/// S4a 甲 / 乙（同一判法）：类型 2 容器里记录号沿记录序严格递增；丙不限类型：墓碑容器、谱系容器的记录也按「首 8 字节」严格递增。
fn sorted_within_containers(facts: &Facts, all_types: bool) -> V {
    fold(judged_versions(facts).flat_map(|version| {
        let mut out = Vec::new();
        let mut by_container: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
        for record in &version.inode_records {
            by_container.entry(record.container).or_default().push(record.inode);
        }
        for (container, numbers) in &by_container {
            out.push((numbers.windows(2).all(|pair| pair[0] < pair[1]), format!("{}：inode 容器 {container} 记录序 {numbers:?}", version.name)));
        }
        if all_types {
            for container in version.tombstone_containers.iter().chain(&version.lineage_containers) {
                let keys: Vec<u64> = container.records.iter().map(|record| u64_at(record, 0)).collect();
                out.push((keys.windows(2).all(|pair| pair[0] < pair[1]), format!("{}：容器 {} 首段序 {keys:?}", version.name, container.container)));
            }
        }
        out
    }))
}

pub const NAMES_UNUSED: [&str; 0] = [];
pub const NAMES: [&str; 30] = [
    "T2a/each", "T2c-A/each", "T2c-B/each", "T2a/sel-x-map", "T2c-B/sel-x-map",
    "I918/A", "I918/A+pub", "I918/B", "I918/B+pub", "I918/nofilter",
    "I919/A", "I919/A+pub", "I919/B", "I919/nofilter",
    "I920/A", "I920/B", "I920/all-T14",
    "I924/each",
    "I921/each", "I921/selected",
    "C479yi/rec", "C479yi/rec+tombA", "C479bing/lineage", "C479bing/txg-1",
    "I96/code", "I96/newest", "I96/each",
    "S4a-jia/type2-sorted", "S4a-bing/all-types-sorted",
    "gap/any-version",
];

pub fn evaluate(facts: &Facts) -> Vec<(&'static str, V)> {
    let gap = fold(facts.versions.iter().map(|version| (version.gap.is_none(), format!("{}：{:?}", version.name, version.gap))));
    vec![
        ("T2a/each", t2_each(facts, Root::NoFilter)),
        ("T2c-A/each", t2_each(facts, Root::Own)),
        ("T2c-B/each", t2_each(facts, Root::Selected)),
        ("T2c-A2/each", t2_each(facts, Root::OwnTxgSelectedLineage)),
        ("T2a/sel-x-map", t2_selected_against_the_map(facts, Root::NoFilter)),
        ("T2c-B/sel-x-map", t2_selected_against_the_map(facts, Root::Selected)),
        ("I918/A", i918(facts, Root::Own, false)),
        ("I918/A+pub", i918(facts, Root::Own, true)),
        ("I918/B", i918(facts, Root::Selected, false)),
        ("I918/B+pub", i918(facts, Root::Selected, true)),
        ("I918/nofilter", i918(facts, Root::NoFilter, false)),
        ("I918/A2", i918(facts, Root::OwnTxgSelectedLineage, false)),
        ("I919/A", i919(facts, Root::Own, false)),
        ("I919/A+pub", i919(facts, Root::Own, true)),
        ("I919/B", i919(facts, Root::Selected, false)),
        ("I919/nofilter", i919(facts, Root::NoFilter, false)),
        ("I919/A2", i919(facts, Root::OwnTxgSelectedLineage, false)),
        ("I920/A", i920(facts, Root::Own)),
        ("I920/B", i920(facts, Root::Selected)),
        ("I920/all-dedup", i920(facts, Root::NoFilter)),
        ("I920/A2", i920(facts, Root::OwnTxgSelectedLineage)),
        ("I920/interval-all-T14", i920_interval_all(facts)),
        ("I924/each", i924(facts)),
        ("TombDeath/birth-le-death-le-txg", tomb_death_bounds(facts)),
        ("I921/each", i921(facts, false)),
        ("I921/selected", i921(facts, true)),
        ("C479yi/rec", c479_yi(facts, false)),
        ("C479yi/rec+tombA", c479_yi(facts, true)),
        ("C479bing/lineage", c479_bing(facts, true)),
        ("C479bing/txg-1", c479_bing(facts, false)),
        ("I96/code", i96(facts, "code")),
        ("I96/newest", i96(facts, "newest")),
        ("I96/each", i96(facts, "each")),
        ("S4a-jia/type2-sorted", sorted_within_containers(facts, false)),
        ("S4a-bing/all-types-sorted", sorted_within_containers(facts, true)),
        ("gap/any-version", gap),
    ]
}
