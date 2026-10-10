//! 三份池级 checker 同一张镜像各判一遍：新 = 冻结副本今天的 singlefs-checker（R1）；probe = 今天的 walk.rs 加探针；
//! pre36 = walk.rs 反打 36-1b（第 35 批的 FL）加探针。几份 crate 各有一个同名 trait，各写一个转接（照第 35 批攻方原型）。
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
adapter!(ProbeReader, singlefs_checker_probe);
adapter!(Pre36, singlefs_checker_pre36);

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

/// 探针：最新根、下一次挂载先施加的那一版、I-4.1 重放施加出来的那一版、重放结局种类（只 R1 那份有）。
#[derive(Clone, Debug, Default)]
pub struct ProbeInfo {
    pub newest: (u32, u64),
    pub next_mount: Option<(u32, u64, Vec<u8>)>,
    pub replayed: Option<(u32, u64, Vec<u8>, u64)>,
    pub replay_kind: String,
}

macro_rules! probed {
    ($krate:ident, $reader:expr) => {{
        $krate::walk::PROBE.with(|p| *p.borrow_mut() = Default::default());
        let list = $krate::walk::check_pool_image($reader);
        let probe = $krate::walk::PROBE.with(|p| p.borrow().clone());
        (
            kinds!(list, $krate),
            ProbeInfo { newest: probe.newest, next_mount: probe.next_mount, replayed: probe.replayed, replay_kind: probe.replay_kind },
        )
    }};
}

pub fn new(reader: &dyn ImageReader) -> Verdicts {
    kinds_of_new(&singlefs_checker::walk::check_pool_image(reader))
}
pub fn probe(reader: &dyn ImageReader) -> (Verdicts, ProbeInfo) {
    probed!(singlefs_checker_probe, &ProbeReader(reader))
}
pub fn pre36(reader: &dyn ImageReader) -> (Verdicts, ProbeInfo) {
    probed!(singlefs_checker_pre36, &Pre36(reader))
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
