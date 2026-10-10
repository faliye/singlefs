//! 新旧两份池级 checker 同一张镜像各判一遍、逐条比（新 = 冻结副本今天的 singlefs-checker，旧 = walk.rs 反打 34-1 的拷贝）。
use std::collections::BTreeMap;

use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker_pre34::image::{
    ImageReader as OldImageReader, InvariantVerdict as OldVerdict,
};

/// 把新 checker 的读口子转给旧那一份（两份 crate 各有一个同名 trait）。
pub struct OldReader<'a>(pub &'a dyn ImageReader);

impl OldImageReader for OldReader<'_> {
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

/// 一条判定压成「种类 + 原文」：H 成立、V 违例、N 不适用、P 部分没判。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kind(pub char, pub String);

pub fn kind_new(verdict: &InvariantVerdict) -> Kind {
    match verdict {
        InvariantVerdict::Holds => Kind('H', String::new()),
        InvariantVerdict::Violated(text) => Kind('V', text.clone()),
        InvariantVerdict::NotApplicable(text) => Kind('N', (*text).to_string()),
        InvariantVerdict::PartiallyJudged(text) => Kind('P', text.clone()),
    }
}

pub fn kind_old(verdict: &OldVerdict) -> Kind {
    match verdict {
        OldVerdict::Holds => Kind('H', String::new()),
        OldVerdict::Violated(text) => Kind('V', text.clone()),
        OldVerdict::NotApplicable(text) => Kind('N', (*text).to_string()),
        OldVerdict::PartiallyJudged(text) => Kind('P', text.clone()),
    }
}

/// 一张镜像的对拍结果。
pub struct Compared {
    pub new: BTreeMap<&'static str, Kind>,
    pub old: BTreeMap<&'static str, Kind>,
    /// 旧 checker 探针：认出的由记录施加出来的版本 (实例, txg)，有没有下一次挂载先施加的那一版。
    pub versions_applied_only_by_records: Vec<(u32, u64)>,
    pub next_mount_version: bool,
    /// 环境变量 B34_WITH_FL=1 时另跑改法 FL 那一份。
    pub fl: Option<BTreeMap<&'static str, Kind>>,
    /// 这张镜像上树表指针整条全零的：读得出、自证过的根（根记录 36..122），与 journal 环里解得开的记录（新根段打头 86 字节）。
    pub all_zero_tree_table_pointers: (u64, u64),
}

/// 数这张镜像上树表指针整条全零的根与记录（Q2：I-2.5 收窄豁免在合法镜像上会不会多红）。
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

impl Compared {
    pub fn new_violations(&self) -> Vec<&'static str> {
        self.new.iter().filter(|(_, k)| k.0 == 'V').map(|(n, _)| *n).collect()
    }
    /// 新旧判定种类不同、或同为不适用 / 部分没判而原文不同的不变量。
    pub fn differences(&self) -> Vec<(&'static str, Kind, Kind)> {
        let mut out = Vec::new();
        for (name, new_kind) in &self.new {
            let old_kind = self.old.get(name).cloned().unwrap_or(Kind('-', String::new()));
            let differs = new_kind.0 != old_kind.0
                || (matches!(new_kind.0, 'N' | 'P') && new_kind.1 != old_kind.1);
            if differs {
                out.push((*name, old_kind, new_kind.clone()));
            }
        }
        out
    }
}

pub fn compare(reader: &dyn ImageReader) -> Compared {
    use singlefs_checker_pre34::walk::{
        PROBE_NEXT_MOUNT_VERSION, PROBE_VERSIONS_APPLIED_ONLY_BY_RECORDS,
    };
    PROBE_VERSIONS_APPLIED_ONLY_BY_RECORDS.with(|cell| cell.set(Vec::new()));
    PROBE_NEXT_MOUNT_VERSION.with(|cell| cell.set(false));
    let new: BTreeMap<&'static str, Kind> = singlefs_checker::walk::check_pool_image(reader)
        .iter()
        .map(|(name, verdict)| (*name, kind_new(verdict)))
        .collect();
    let old: BTreeMap<&'static str, Kind> =
        singlefs_checker_pre34::walk::check_pool_image(&OldReader(reader))
            .iter()
            .map(|(name, verdict)| (*name, kind_old(verdict)))
            .collect();
    Compared {
        new,
        old,
        versions_applied_only_by_records: PROBE_VERSIONS_APPLIED_ONLY_BY_RECORDS
            .with(|cell| cell.take()),
        next_mount_version: PROBE_NEXT_MOUNT_VERSION.with(std::cell::Cell::get),
        fl: (std::env::var("B34_WITH_FL").as_deref() == Ok("1")).then(|| fl_kinds(reader)),
        all_zero_tree_table_pointers: all_zero_tree_table_pointers(reader),
    }
}

/// 改法 FL 那一份（`crates/b34-checker-fl`，零轮）：两类由记录造出来的版本取带末条标志的那条记录的新根段。
pub struct FlReader<'a>(pub &'a dyn ImageReader);

impl singlefs_checker_fl::image::ImageReader for FlReader<'_> {
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

pub fn fl_kinds(reader: &dyn ImageReader) -> BTreeMap<&'static str, Kind> {
    use singlefs_checker_fl::image::InvariantVerdict as FlVerdict;
    singlefs_checker_fl::walk::check_pool_image(&FlReader(reader))
        .iter()
        .map(|(name, verdict)| {
            let kind = match verdict {
                FlVerdict::Holds => Kind('H', String::new()),
                FlVerdict::Violated(text) => Kind('V', text.clone()),
                FlVerdict::NotApplicable(text) => Kind('N', (*text).to_string()),
                FlVerdict::PartiallyJudged(text) => Kind('P', text.clone()),
            };
            (*name, kind)
        })
        .collect()
}
