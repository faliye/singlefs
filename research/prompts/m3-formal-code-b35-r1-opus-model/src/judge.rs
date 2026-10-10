//! 几份池级 checker 同一张镜像各判一遍：新 = 冻结副本今天的 singlefs-checker；pre35 = walk.rs 反打 35-1；probe = 今天的 walk.rs 加探针；
//! mutr / mutn / mutz = 三处 CPU 变异（GPU 三行变异的 CPU 对应）。几份 crate 各有一个同名 trait，各写一个转接。
use std::collections::{BTreeMap, BTreeSet};

use singlefs_checker::image::{ImageReader, InvariantVerdict};

macro_rules! adapter {
    ($name:ident, $krate:ident) => {
        pub struct $name<'a>(pub &'a dyn ImageReader);
        impl $krate::image::ImageReader for $name<'_> {
            fn devices(&self) -> Vec<u32> {
                self.0.devices()
            }
            fn device_bytes(&self, device: u32) -> Option<u64> {
                self.0.device_bytes(device)
            }
            fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
                self.0.read(device, offset, length)
            }
            fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>> {
                self.0.candidate_unit_slots(device)
            }
            fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
                self.0.candidate_journal_slots(device)
            }
        }
    };
}
adapter!(Pre35, singlefs_checker_pre35);
adapter!(ProbeReader, singlefs_checker_probe);
adapter!(MutR, singlefs_checker_mutr);
adapter!(MutN, singlefs_checker_mutn);
adapter!(MutZ, singlefs_checker_mutz);

/// 一条判定压成「种类 + 原文」：H 成立、V 违例、N 不适用、P 部分没判。
pub type Verdicts = BTreeMap<&'static str, (char, String)>;

macro_rules! kinds {
    ($list:expr, $krate:ident) => {
        $list
            .iter()
            .map(|(name, verdict)| {
                use $krate::image::InvariantVerdict as V;
                let kind = match verdict {
                    V::Holds => ('H', String::new()),
                    V::Violated(text) => ('V', text.clone()),
                    V::NotApplicable(text) => ('N', (*text).to_string()),
                    V::PartiallyJudged(text) => ('P', text.clone()),
                };
                (*name, kind)
            })
            .collect::<Verdicts>()
    };
}

pub fn kinds_of_new(list: &[(&'static str, InvariantVerdict)]) -> Verdicts {
    kinds!(list, singlefs_checker)
}

/// 探针：认出的由记录施加出来的版本（实例、txg、新根段）、每一版取没取末条、下一次挂载先施加的那一版、I-4.1 重放施加出来的那一版。
#[derive(Clone, Debug, Default)]
pub struct ProbeInfo {
    pub versions: Vec<(u32, u64, Vec<u8>)>,
    pub from_last: Vec<((u32, u64), bool)>,
    pub next_mount: Option<(u32, u64, Vec<u8>)>,
    pub replayed: Option<(u32, u64, Vec<u8>, u64)>,
}

macro_rules! probed {
    ($krate:ident, $reader:expr) => {{
        $krate::walk::PROBE.with(|p| *p.borrow_mut() = Default::default());
        let list = $krate::walk::check_pool_image($reader);
        let probe = $krate::walk::PROBE.with(|p| p.borrow().clone());
        (
            kinds!(list, $krate),
            ProbeInfo {
                versions: probe.versions,
                from_last: probe.from_last,
                next_mount: probe.next_mount,
                replayed: probe.replayed,
            },
        )
    }};
}

pub fn new(reader: &dyn ImageReader) -> Verdicts {
    kinds_of_new(&singlefs_checker::walk::check_pool_image(reader))
}
pub fn probe(reader: &dyn ImageReader) -> (Verdicts, ProbeInfo) {
    probed!(singlefs_checker_probe, &ProbeReader(reader))
}
pub fn pre35(reader: &dyn ImageReader) -> (Verdicts, ProbeInfo) {
    probed!(singlefs_checker_pre35, &Pre35(reader))
}
pub fn mutant(which: &str, reader: &dyn ImageReader) -> Verdicts {
    match which {
        "mutr" => kinds!(singlefs_checker_mutr::walk::check_pool_image(&MutR(reader)), singlefs_checker_mutr),
        "mutn" => kinds!(singlefs_checker_mutn::walk::check_pool_image(&MutN(reader)), singlefs_checker_mutn),
        "mutz" => kinds!(singlefs_checker_mutz::walk::check_pool_image(&MutZ(reader)), singlefs_checker_mutz),
        _ => panic!("不认得的变异 {which}"),
    }
}

pub fn reds(verdicts: &Verdicts) -> BTreeSet<&'static str> {
    verdicts.iter().filter(|(_, k)| k.0 == 'V').map(|(n, _)| *n).collect()
}

/// 种类不同、或同为不适用 / 部分没判而原文不同的不变量：(名字, 左, 右)。
pub fn differences(left: &Verdicts, right: &Verdicts) -> Vec<(&'static str, (char, String), (char, String))> {
    let mut out = Vec::new();
    for (name, l) in left {
        let r = right.get(name).cloned().unwrap_or(('-', String::new()));
        if l.0 != r.0 || (matches!(l.0, 'N' | 'P') && l.1 != r.1) {
            out.push((*name, l.clone(), r));
        }
    }
    out
}

/// 这张镜像上树表指针整条全零的：读得出、自证过的根（根记录 36..122），与 journal 环里解得开的记录（新根段打头 86 字节）。
pub fn all_zero_tree_table_pointers(reader: &dyn ImageReader) -> (u64, u64) {
    use singlefs_checker::image::{chosen_system_configurations, valid_roots};
    let Some(geometry) = chosen_system_configurations(reader)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
    else {
        return (0, 0);
    };
    let roots = valid_roots(reader, &geometry)
        .into_iter()
        .filter(|(_, _, view)| view.record_bytes[36..122].iter().all(|b| *b == 0))
        .count() as u64;
    let mut records = 0;
    for device in reader.devices() {
        for slot in reader.candidate_journal_slots(device).unwrap_or_default() {
            let offset = singlefs_format::JOURNAL_RING_START_SLOT * singlefs_format::SLOT_BYTES + slot * 4096;
            let Some(bytes) = reader.read(device, offset, 4096) else { continue };
            let fsid = u64::from_le_bytes(bytes[463..471].try_into().expect("8"));
            if singlefs_core::journal::JournalRecord::parse(&bytes, fsid).is_some() && bytes[99..185].iter().all(|b| *b == 0) {
                records += 1;
            }
        }
    }
    (roots, records)
}
