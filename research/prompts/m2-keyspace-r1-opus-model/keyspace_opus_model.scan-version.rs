//! m2-keyspace-r1 云端攻方（Opus）的最小原型，只在副本上跑，不是入库装置。
//!
//! 被判的是「派生树按 key 空间定形状、不分裂」的几个候选：
//! - S1 分配记录树：叶管定宽 `aw` 个槽（按起点槽落叶），内部节点按位置隐含子区间、扇出 `af`，顶层按设备分两路；
//!   缺席的一段是「全空闲」（Absent）还是 mkfs 起就写满（Full）；跨叶边界的记录，点查读者回看前一片叶（LookBack）或不回看（NoLookBack）。
//! - S2 extent 树：每文件一棵、根指针住 inode 记录（PerFile）；全局两段，上段按 (locality, inode) 的 key 排、会分裂（GlobalKeyed）；
//!   全局两段，上段按 inode 号按位置寻址（GlobalRadix）。下段都是按单元号按位置寻址、叶宽 `ew`、扇出 `ef`，洞 = 缺席。
//! - S4 写序：一次发布 = 数据单元 + 这次脏的全部节点（extent → inode 容器 → 分配记录树，树内先叶后根）→ 屏障 → 一条 journal 记录
//!   （点名这次写的全部单元）→ 屏障 → 根槽 FUA。固定点：每给一个脏节点发新槽就在分配记录树里记一条，旧副本改成已释放，
//!   改到的叶又变脏，直到不再有新脏节点（「释放链」）。
//!
//! 节点字节由 `singlefs_core::unit::build_index_node` 写、`parse_index_node` 解（树 ID、层级、key 区间、诞生代号在实现的偏移上）；
//! key 宽一律 8、条目宽 32，是原型自己的简化条目格式，不是实现的 20 / 112 / 110 字节条目。崩溃状态照 D13（验证路线） 已定项 4：
//! 屏障切段、FUA 自成段尾、当前段任意子集、没持久的位置放旧字节；两盘镜像的同一单元按「都没落 / 只落一份 / 两份都落」三类枚举，
//! 只落一份那类权重 2（两盘旧字节相同时两种单落逐状态同结局，每段开头断言过；不同就拆开各算一次），权重和逐段断言等于闭式。
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Mutex;

use singlefs_core::address::{CheckpointTxg, InstanceGeneration, TreeIdentifier};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::pointer::BirthSequence;
use singlefs_core::unit::{build_index_node, parse_index_node, IndexNodeHeader};
use singlefs_harness::crash::closed_form_state_count;

const FSID: [u8; 16] = [7; 16];
const T_EXT: u64 = 11;
const T_INODE: u64 = 12;
const T_ALLOC: u64 = 13;
const DEVS: u64 = 2;
const KW: usize = 8;
const ENTRY_W: u16 = 32;
const MAX_SPAN: u64 = 2;

// ---------------------------------------------------------------- 候选

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Absence {
    /// 没有记录的一段不写节点，父节点那一格是空指针，读者读成「全空闲」。
    Absent,
    /// mkfs 起每一段都有节点（空叶照写），父节点每一格都有指针；读者遇到空指针判损坏。
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cross {
    /// 点查槽 x 时看 x 所在的叶，再看前一片叶里起点在 [叶起点 − (最大跨度 − 1), 叶起点) 的记录。
    LookBack,
    /// 只看 x 所在的叶。
    NoLookBack,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ext {
    PerFile,
    GlobalKeyed,
    GlobalRadix,
    /// 同 GlobalRadix，但只有单元 0 的文件不建下段：上段叶的条目里直接放数据指针（高度 0 的下段）。
    GlobalRadixInline,
}

#[derive(Clone, Copy, Debug)]
pub struct Cfg {
    pub name: &'static str,
    /// 每块盘单元区的槽数（槽号从 0 起，0 是偶数槽）。
    pub slots: u64,
    /// 聚簇段槽数；0 = 不开聚簇段，提交内生块一律走回落（该盘槽号最小的空槽，D3（空间分配） 已定项 8 ②）。
    pub seg: u64,
    /// 根环里一共留几个根（两盘轮流写，每盘 ring / 2 个槽）；回收下界 = txg − ring。
    pub ring: u64,
    pub aw: u64,
    pub af: u64,
    /// 叶边界相对槽号 0 的偏移：key = 设备 × 设备跨度 + 槽号 + aoff。
    pub aoff: u64,
    /// true：两层——顶层直接按（设备, 叶号）指到叶，扇出 = 设备数 × 每盘叶数（崩溃模型用，压每次发布的节点数）。
    pub aflat: bool,
    pub absence: Absence,
    pub cross: Cross,
    pub ext: Ext,
    pub ew: u64,
    pub ef: u64,
    /// GlobalRadix 上段叶宽（inode 数）/ GlobalKeyed 上段叶与内部节点的容量。
    pub tw: u64,
    /// true：写节点字节、录写流（崩溃模型）；false：只数（代价模型）。
    pub bytes: bool,
    /// 自己提的改法（零轮）：分配记录树自己的节点只落在每盘单元区开头 [0, aregion) 这一小段里，数据与别的提交内生块不进这一段。0 = 不开。
    pub aregion: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ptr {
    pub slot: u64,
    pub crc: u32,
    pub level: u8,
    pub birth: u64,
}

impl Ptr {
    fn bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(21);
        v.extend_from_slice(&self.slot.to_le_bytes());
        v.extend_from_slice(&self.crc.to_le_bytes());
        v.push(self.level);
        v.extend_from_slice(&self.birth.to_le_bytes());
        v
    }
    fn parse(b: &[u8]) -> Ptr {
        Ptr {
            slot: u64::from_le_bytes(b[0..8].try_into().unwrap()),
            crc: u32::from_le_bytes(b[8..12].try_into().unwrap()),
            level: b[12],
            birth: u64::from_le_bytes(b[13..21].try_into().unwrap()),
        }
    }
    fn is_null(b: &[u8]) -> bool {
        b[..21].iter().all(|x| *x == 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rec {
    pub span: u64,
    pub gen: u64,
    pub released: bool,
}

/// 节点身份：同一个身份跨发布不变，内容变了就 COW 到新槽。枚举序即 bump 的树序（extent 11 → inode 12 → 分配记录 13）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Nid {
    Ext(u64, u8, u64),
    Top(u8, u64),
    KTop(usize),
    Cont,
    Alloc(u8, u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cause {
    Data,
    CommitOther,
    CommitAlloc,
}

impl Cfg {
    /// 每块盘那棵子树的层数（含叶层）与一块盘的 key 跨度。
    pub fn ageo(&self) -> (u8, u64) {
        if self.aflat {
            return (1, self.aw * (self.slots + self.aoff).div_ceil(self.aw));
        }
        let mut h = 1u8;
        let mut cover = self.aw;
        while cover < self.slots + self.aoff {
            cover *= self.af;
            h += 1;
        }
        (h, cover)
    }
    /// 分配记录树的顶层（按设备分路那一层）的层号。
    pub fn atop(&self) -> u8 {
        self.ageo().0
    }
    pub fn aspan(&self, l: u8) -> u64 {
        let (h, dev) = self.ageo();
        if l == h {
            dev * DEVS
        } else {
            self.aw * self.af.pow(u32::from(l))
        }
    }
    pub fn afan(&self, l: u8) -> u64 {
        if l == self.atop() {
            if self.aflat {
                DEVS * self.ageo().1 / self.aw
            } else {
                DEVS
            }
        } else {
            self.af
        }
    }
    pub fn akey(&self, dev: u64, slot: u64) -> u64 {
        dev * self.ageo().1 + slot + self.aoff
    }
    /// key → (设备, 槽)；落在一块盘的有效槽之外时 None。
    pub fn aunkey(&self, key: u64) -> Option<(u64, u64)> {
        let dev_span = self.ageo().1;
        let dev = key / dev_span;
        let rel = key % dev_span;
        if dev >= DEVS || rel < self.aoff || rel - self.aoff >= self.slots {
            return None;
        }
        Some((dev, rel - self.aoff))
    }
    pub fn espan(&self, l: u8) -> u64 {
        self.ew * self.ef.pow(u32::from(l))
    }
    /// 罩住单元号 0..=max 要几层（至少 1）。
    pub fn eheight(&self, max_unit: u64) -> u8 {
        let mut h = 1u8;
        while self.espan(h - 1) <= max_unit {
            h += 1;
        }
        h
    }
    pub fn tspan(&self, l: u8) -> u64 {
        self.tw * self.ef.pow(u32::from(l))
    }
    pub fn theight(&self, max_inode: u64) -> u8 {
        let mut h = 1u8;
        while self.tspan(h - 1) <= max_inode {
            h += 1;
        }
        h
    }
}

// ---------------------------------------------------------------- 上段按 key 排、会分裂的那棵（GlobalKeyed 的上段）

#[derive(Clone, Debug)]
pub struct KNode {
    pub level: u8,
    /// 叶：inode 号；内部：每个孩子的分隔 key（= 孩子子树的最小 key，跟着维护）。
    pub keys: Vec<u64>,
    pub kids: Vec<usize>,
    pub alive: bool,
}

#[derive(Clone, Debug)]
pub struct KTree {
    pub nodes: Vec<KNode>,
    pub root: usize,
}

impl KTree {
    pub fn new() -> KTree {
        KTree { nodes: vec![KNode { level: 0, keys: vec![], kids: vec![], alive: true }], root: 0 }
    }
    pub fn path(&self, k: u64) -> Vec<usize> {
        let mut out = vec![self.root];
        let mut cur = self.root;
        while self.nodes[cur].level > 0 {
            let n = &self.nodes[cur];
            let i = n.keys.iter().rposition(|s| *s <= k).unwrap_or(0);
            cur = n.kids[i];
            out.push(cur);
        }
        out
    }
    /// 插一个 inode；交回这次要重写的节点（路径 + 分裂出来的新节点）。
    pub fn insert(&mut self, k: u64, cap: usize) -> Vec<usize> {
        let path = self.path(k);
        let mut dirty: Vec<usize> = path.clone();
        for w in path.windows(2) {
            let (parent, child) = (w[0], w[1]);
            let i = self.nodes[parent].kids.iter().position(|c| *c == child).unwrap();
            if k < self.nodes[parent].keys[i] {
                self.nodes[parent].keys[i] = k;
            }
        }
        let leaf = *path.last().unwrap();
        let pos = self.nodes[leaf].keys.partition_point(|x| *x < k);
        assert!(self.nodes[leaf].keys.get(pos) != Some(&k), "同一个 inode 插两次");
        self.nodes[leaf].keys.insert(pos, k);
        let mut depth = path.len() - 1;
        let mut node = leaf;
        while self.nodes[node].keys.len() > cap {
            let mid = self.nodes[node].keys.len() / 2;
            let level = self.nodes[node].level;
            let right_keys = self.nodes[node].keys.split_off(mid);
            let right_kids = if level > 0 { self.nodes[node].kids.split_off(mid) } else { vec![] };
            let sep = right_keys[0];
            self.nodes.push(KNode { level, keys: right_keys, kids: right_kids, alive: true });
            let right = self.nodes.len() - 1;
            dirty.push(right);
            if depth == 0 {
                let left_min = self.nodes[node].keys[0];
                self.nodes.push(KNode { level: level + 1, keys: vec![left_min, sep], kids: vec![node, right], alive: true });
                self.root = self.nodes.len() - 1;
                dirty.push(self.root);
                break;
            }
            let parent = path[depth - 1];
            let i = self.nodes[parent].kids.iter().position(|c| *c == node).unwrap();
            self.nodes[parent].keys.insert(i + 1, sep);
            self.nodes[parent].kids.insert(i + 1, right);
            node = parent;
            depth -= 1;
        }
        dirty
    }
    pub fn min_max(&self, id: usize) -> (u64, u64) {
        let mut lo = id;
        while self.nodes[lo].level > 0 {
            lo = self.nodes[lo].kids[0];
        }
        let mut hi = id;
        while self.nodes[hi].level > 0 {
            hi = *self.nodes[hi].kids.last().unwrap();
        }
        (self.nodes[lo].keys.first().copied().unwrap_or(0), self.nodes[hi].keys.last().copied().unwrap_or(0))
    }
    pub fn height(&self) -> u8 {
        self.nodes[self.root].level + 1
    }
}

// ---------------------------------------------------------------- 写流

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootFields {
    pub alloc: Ptr,
    pub top: Option<Ptr>,
    pub cont: Ptr,
}

#[derive(Clone, Debug)]
pub struct JRec {
    pub jsn: u64,
    pub txg: u64,
    pub fields: RootFields,
    pub named: Vec<(u64, u64, u32)>,
}

#[derive(Clone, Debug)]
pub struct RootRec {
    pub txg: u64,
    pub fields: RootFields,
    pub last_jsn: u64,
}

#[derive(Clone, Debug)]
pub enum Blob {
    Node { crc: u32, header: IndexNodeHeader },
    Data { crc: u32, inode: u64, unit: u64, ver: u64, birth: u64 },
    Cont { crc: u32, files: BTreeMap<u64, Option<Ptr>> },
    Rec(JRec),
    Root(RootRec),
}

impl Blob {
    fn crc(&self) -> u32 {
        match self {
            Blob::Node { crc, .. } | Blob::Data { crc, .. } | Blob::Cont { crc, .. } => *crc,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Loc {
    Slot(u64),
    Rec(u64),
    Root(u64),
}

#[derive(Clone, Copy, Debug)]
pub struct Write {
    pub dev: u8,
    pub loc: Loc,
    pub span: u64,
    pub blob: usize,
    pub fua: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum StreamOp {
    W(usize),
    Barrier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Write(u64, u64),
    Trunc(u64, u64),
    Create(u64),
    /// 一次发布写一个文件的 k 个相邻单元（代价模型用）。
    Range(u64, u64, u64),
}

#[derive(Clone, Copy, Debug)]
pub struct DataRef {
    pub slot: u64,
    pub crc: u32,
    pub ver: u64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Metrics {
    pub data: u64,
    pub alloc_nodes: u64,
    pub alloc_leaves: u64,
    pub leaves_by_data: u64,
    pub leaves_by_other: u64,
    pub leaves_by_alloc: u64,
    pub ext_nodes: u64,
    pub cont: u64,
    pub rounds: u64,
    pub named: u64,
    pub formula: u64,
    pub double_alloc: u64,
    pub release_missing: u64,
    pub fallback: u64,
    pub ext_grew: bool,
    pub ext_shrank: bool,
    pub top_split: bool,
    pub refused: bool,
}

#[derive(Clone, Debug)]
pub struct Version {
    pub txg: u64,
    pub content: BTreeMap<(u64, u64), u64>,
    pub inodes: BTreeSet<u64>,
    pub fields: RootFields,
    pub root_write: usize,
    pub rec_writes: Vec<usize>,
    pub m: Metrics,
}

// ---------------------------------------------------------------- 池

#[derive(Clone)]
pub struct Pool {
    pub cfg: Cfg,
    pub txg: u64,
    pub jsn: u64,
    pub recs: Vec<BTreeMap<u64, Rec>>,
    pub files: BTreeMap<u64, BTreeMap<u64, DataRef>>,
    pub disk: BTreeMap<Nid, Ptr>,
    pub ktree: KTree,
    pub fields: Option<RootFields>,
    pub opened: BTreeSet<u64>,
    pub open_seg: Option<u64>,
    pub cursor: u64,
    naive: bool,
    used: Vec<u64>,
    seg_used: Vec<u32>,
    seg_hint: u64,
    data_hint: u64,
    fallback_hint: u64,
    reclaim_q: BTreeMap<u64, Vec<(u64, u64)>>,
    pub stream: Vec<StreamOp>,
    pub writes: Vec<Write>,
    pub blobs: Vec<Blob>,
    pub versions: Vec<Version>,
    next_ver: u64,
    // 这一次发布的草稿
    floor: u64,
    dirty: BTreeMap<Nid, Cause>,
    m: Metrics,
    pending_data: Vec<(u64, usize)>,
    /// 写者缺陷开关（坏镜像自证用）：装第 1 层分配记录节点时漏掉这个叶的指针。
    pub bug_drop_leaf: Option<u64>,
    /// 写者缺陷开关（坏镜像自证用）：文件一次长高不止一层时，漏写旧根之上位置 0 那一串新节点（父节点里那一格留空 = 洞）。
    pub bug_skip_spine: bool,
    /// 写者缺陷开关：矮下去不止一层时不标旧的那一串（原型第一版的写法），用来复现 checker 判 I-3.1 泄漏。
    pub bug_skip_shrink_spine: bool,
    pub refusal: String,
}

fn live(r: &Rec, floor: u64) -> bool {
    !r.released || r.gen > floor
}

fn crc_of(parts: &[u64]) -> u32 {
    let mut b = Vec::with_capacity(parts.len() * 8);
    for p in parts {
        b.extend_from_slice(&p.to_le_bytes());
    }
    crc32_castagnoli(&b)
}

impl Pool {
    pub fn new(cfg: Cfg) -> Pool {
        assert!(cfg.ring % DEVS == 0 && cfg.ring >= DEVS);
        let naive = cfg.slots <= 4096;
        let segs = if cfg.seg > 0 { cfg.slots / cfg.seg } else { 0 };
        let mut p = Pool {
            cfg,
            txg: 0,
            jsn: 0,
            recs: vec![BTreeMap::new(); DEVS as usize],
            files: BTreeMap::new(),
            disk: BTreeMap::new(),
            ktree: KTree::new(),
            fields: None,
            opened: BTreeSet::new(),
            open_seg: None,
            cursor: 0,
            naive,
            used: if naive { vec![] } else { vec![0; (cfg.slots as usize).div_ceil(64)] },
            seg_used: vec![0; segs as usize],
            seg_hint: 0,
            data_hint: 0,
            fallback_hint: 0,
            reclaim_q: BTreeMap::new(),
            stream: vec![],
            writes: vec![],
            blobs: vec![],
            versions: vec![],
            next_ver: 0,
            floor: 0,
            dirty: BTreeMap::new(),
            m: Metrics::default(),
            pending_data: vec![],
            bug_drop_leaf: None,
            bug_skip_spine: false,
            bug_skip_shrink_spine: false,
            refusal: String::new(),
        };
        // mkfs = txg 0：顶层根、inode 容器、（Full 时）全部分配记录节点、（GlobalKeyed 时）上段的空根叶。
        let top = cfg.atop();
        p.dirty.insert(Nid::Alloc(top, 0), Cause::CommitOther);
        p.dirty.insert(Nid::Cont, Cause::CommitOther);
        if cfg.ext == Ext::GlobalKeyed {
            p.dirty.insert(Nid::KTop(0), Cause::CommitOther);
        }
        if cfg.absence == Absence::Full {
            for l in 0..top {
                let span = cfg.aspan(l);
                let n = cfg.aspan(top) / span;
                for idx in 0..n {
                    if p.present(Nid::Alloc(l, idx)) {
                        p.dirty.insert(Nid::Alloc(l, idx), Cause::CommitOther);
                    }
                }
            }
        }
        p.finish_publish(true).expect("mkfs 装得下");
        p
    }

    // ---------------- 空闲判定

    pub fn free_slots(&self) -> u64 {
        (0..self.cfg.slots).filter(|x| (0..DEVS).all(|d| self.truth_free(d, *x))).count() as u64
    }

    fn used_get(&self, x: u64) -> bool {
        self.used[(x / 64) as usize] >> (x % 64) & 1 == 1
    }
    fn used_set(&mut self, x: u64, span: u64, on: bool) {
        for y in x..x + span {
            let was = self.used_get(y);
            if was == on {
                continue;
            }
            self.used[(y / 64) as usize] ^= 1 << (y % 64);
            if self.cfg.seg > 0 {
                let s = (y / self.cfg.seg) as usize;
                if s < self.seg_used.len() {
                    if on {
                        self.seg_used[s] += 1;
                    } else {
                        self.seg_used[s] -= 1;
                        if self.seg_used[s] == 0 {
                            self.seg_hint = self.seg_hint.min(s as u64);
                        }
                    }
                }
            }
            if !on {
                self.data_hint = self.data_hint.min(y & !1);
                self.fallback_hint = self.fallback_hint.min(y);
            }
        }
    }

    /// 槽 x 所在叶的起点槽（叶从槽 0 之前起时取 0）。
    fn leaf_start_slot(&self, dev: u64, x: u64) -> u64 {
        let k = self.cfg.akey(dev, x);
        let lo_key = k / self.cfg.aw * self.cfg.aw;
        let dev_base = dev * self.cfg.ageo().1 + self.cfg.aoff;
        lo_key.saturating_sub(dev_base)
    }

    /// 被判候选的点查读者：槽 x 在这块盘上空不空。
    pub fn reader_free(&self, dev: u64, x: u64) -> bool {
        if !self.naive {
            return !self.used_get(x);
        }
        let recs = &self.recs[dev as usize];
        let lo = self.leaf_start_slot(dev, x);
        for (s, r) in recs.range(lo..=x) {
            if s + r.span > x && live(r, self.floor) {
                return false;
            }
        }
        if self.cfg.cross == Cross::LookBack {
            for (s, r) in recs.range(lo.saturating_sub(MAX_SPAN - 1)..lo) {
                if s + r.span > x && live(r, self.floor) {
                    return false;
                }
            }
        }
        true
    }

    pub fn truth_free(&self, dev: u64, x: u64) -> bool {
        self.recs[dev as usize]
            .range(x.saturating_sub(MAX_SPAN - 1)..=x)
            .all(|(s, r)| !(s + r.span > x && live(r, self.floor)))
    }

    fn free_all(&self, x: u64, span: u64) -> bool {
        x + span <= self.cfg.slots && (0..DEVS).all(|d| (x..x + span).all(|y| self.reader_free(d, y)))
    }

    fn seg_empty(&self, t: u64) -> bool {
        if self.naive {
            (t * self.cfg.seg..(t + 1) * self.cfg.seg).all(|y| (0..DEVS).all(|d| self.reader_free(d, y)))
        } else {
            self.seg_used[t as usize] == 0
        }
    }

    fn place_data(&mut self) -> Option<u64> {
        let mut x = if self.naive { 0 } else { self.data_hint };
        x = x.max(self.cfg.aregion.next_multiple_of(2));
        while x + 1 < self.cfg.slots {
            let in_opened = self.cfg.seg > 0 && self.opened.contains(&(x / self.cfg.seg));
            if !in_opened && self.free_all(x, 2) {
                if !self.naive {
                    self.data_hint = x;
                }
                return Some(x);
            }
            x += 2;
        }
        None
    }

    fn place_commit(&mut self, span: u64, alloc_node: bool) -> Option<u64> {
        if self.cfg.aregion > 0 && alloc_node {
            if let Some(x) = (0..self.cfg.aregion).find(|x| self.free_all(*x, 1)) {
                return Some(x);
            }
        }
        let first_seg = if self.cfg.seg > 0 { self.cfg.aregion.div_ceil(self.cfg.seg) } else { 0 };
        if self.cfg.seg > 0 {
            loop {
                if let Some(s) = self.open_seg {
                    let end = (s + 1) * self.cfg.seg;
                    let mut c = self.cursor;
                    if span == 2 && c % 2 == 1 {
                        c += 1;
                    }
                    while c + span <= end {
                        if self.free_all(c, span) {
                            self.cursor = c + span;
                            return Some(c);
                        }
                        c += span;
                    }
                }
                let n = self.cfg.slots / self.cfg.seg;
                let start = (if self.naive { 0 } else { self.seg_hint }).max(first_seg);
                let next = (start..n).find(|t| Some(*t) != self.open_seg && self.seg_empty(*t));
                match next {
                    Some(t) => {
                        if !self.naive {
                            self.seg_hint = t;
                        }
                        self.open_seg = Some(t);
                        self.opened.insert(t);
                        self.cursor = t * self.cfg.seg;
                    }
                    None => break,
                }
            }
        }
        self.m.fallback += 1;
        let mut x = if self.naive { 0 } else { self.fallback_hint & if span == 2 { !1 } else { !0 } };
        x = x.max(self.cfg.aregion.next_multiple_of(2));
        while x + span <= self.cfg.slots {
            if self.free_all(x, span) {
                return Some(x);
            }
            x += span;
        }
        None
    }

    // ---------------- 脏标记

    fn mark(&mut self, n: Nid, cause: Cause) {
        self.dirty.entry(n).or_insert(cause);
    }
    fn dirty_alloc(&mut self, dev: u64, slot: u64, cause: Cause) {
        let k = self.cfg.akey(dev, slot);
        for l in 0..=self.cfg.atop() {
            self.mark(Nid::Alloc(l, k / self.cfg.aspan(l)), cause);
        }
    }
    fn file_height(&self, inode: u64) -> Option<u8> {
        let f = self.files.get(&inode)?;
        let max = *f.keys().next_back()?;
        if self.cfg.ext == Ext::GlobalRadixInline && max == 0 {
            return Some(0);
        }
        Some(self.cfg.eheight(max))
    }
    fn dirty_ext(&mut self, inode: u64, unit: u64, h: u8) {
        for l in 0..h {
            self.mark(Nid::Ext(inode, l, unit / self.cfg.espan(l)), Cause::CommitOther);
            // 长高不止一层时，旧根之上那一串位置 0 的节点是新的（它们罩着旧根），也要写。
            let spine = Nid::Ext(inode, l, 0);
            if !self.bug_skip_spine && !self.disk.contains_key(&spine) && self.present(spine) {
                self.mark(spine, Cause::CommitOther);
            }
        }
    }
    fn max_inode(&self) -> u64 {
        self.files.keys().next_back().copied().unwrap_or(0)
    }
    fn dirty_top_for(&mut self, inode: u64) {
        match self.cfg.ext {
            Ext::PerFile => {}
            Ext::GlobalRadix | Ext::GlobalRadixInline => {
                let h = self.cfg.theight(self.max_inode());
                for l in 0..h {
                    self.mark(Nid::Top(l, inode / self.cfg.tspan(l)), Cause::CommitOther);
                    let spine = Nid::Top(l, 0);
                    if !self.disk.contains_key(&spine) && self.present(spine) {
                        self.mark(spine, Cause::CommitOther);
                    }
                }
            }
            Ext::GlobalKeyed => {
                for id in self.ktree.path(inode) {
                    self.mark(Nid::KTop(id), Cause::CommitOther);
                }
            }
        }
        self.mark(Nid::Cont, Cause::CommitOther);
    }

    // ---------------- 在不在

    pub fn present(&self, n: Nid) -> bool {
        let cfg = &self.cfg;
        match n {
            Nid::Alloc(l, idx) => {
                let top = cfg.atop();
                if l == top {
                    return idx == 0;
                }
                let span = cfg.aspan(l);
                let lo = idx * span;
                let (_, ds) = cfg.ageo();
                let dev = lo / ds;
                if dev >= DEVS {
                    return false;
                }
                let rel_lo = lo % ds;
                let rel_hi = rel_lo + span;
                let a = rel_lo.max(cfg.aoff);
                let b = rel_hi.min(cfg.aoff + cfg.slots);
                if a >= b {
                    return false;
                }
                match cfg.absence {
                    Absence::Full => true,
                    Absence::Absent => self.recs[dev as usize].range(a - cfg.aoff..b - cfg.aoff).next().is_some(),
                }
            }
            Nid::Ext(inode, l, idx) => {
                let Some(h) = self.file_height(inode) else { return false };
                if l >= h || (l == h - 1 && idx != 0) {
                    return false;
                }
                let span = cfg.espan(l);
                self.files[&inode].range(idx * span..(idx + 1) * span).next().is_some()
            }
            Nid::Top(l, idx) => {
                if self.files.is_empty() {
                    return false;
                }
                let h = cfg.theight(self.max_inode());
                if l >= h || (l == h - 1 && idx != 0) {
                    return false;
                }
                let span = cfg.tspan(l);
                self.files.range(idx * span..(idx + 1) * span).next().is_some()
            }
            Nid::KTop(id) => self.ktree.nodes[id].alive,
            Nid::Cont => true,
        }
    }

    // ---------------- 记录的改写

    fn record(&mut self, x: u64, span: u64, cause: Cause) {
        let txg = self.txg;
        let floor = self.floor;
        for dev in 0..DEVS {
            let overl: Vec<(u64, Rec)> = self.recs[dev as usize]
                .range(x.saturating_sub(MAX_SPAN - 1)..x + span)
                .filter(|(s, r)| **s + r.span > x)
                .map(|(s, r)| (*s, *r))
                .collect();
            for (s, r) in overl {
                if live(&r, floor) && dev == 0 {
                    self.m.double_alloc += 1;
                }
                if s != x {
                    self.recs[dev as usize].remove(&s);
                    self.dirty_alloc(dev, s, cause);
                }
            }
            self.recs[dev as usize].insert(x, Rec { span, gen: txg, released: false });
            self.dirty_alloc(dev, x, cause);
        }
        if !self.naive {
            self.used_set(x, span, true);
        }
    }

    fn release(&mut self, x: u64, span: u64, cause: Cause) {
        let txg = self.txg;
        for dev in 0..DEVS {
            match self.recs[dev as usize].get_mut(&x) {
                Some(r) if !r.released && r.span == span => {
                    r.released = true;
                    r.gen = txg;
                    self.dirty_alloc(dev, x, cause);
                }
                _ => {
                    if dev == 0 {
                        self.m.release_missing += 1;
                    }
                }
            }
        }
        if !self.naive {
            self.reclaim_q.entry(txg).or_default().push((x, span));
        }
    }

    fn reclaim_to(&mut self, floor: u64) {
        self.floor = floor;
        if self.naive {
            return;
        }
        let gens: Vec<u64> = self.reclaim_q.range(..=floor).map(|(g, _)| *g).collect();
        for g in gens {
            for (x, span) in self.reclaim_q.remove(&g).unwrap() {
                // 这期间被重新分配过的不清（record 已经把位置回 1，而它的记录是新的、未释放）。
                let still_released = self.recs[0].get(&x).is_some_and(|r| r.released && r.gen == g);
                if still_released {
                    self.used_set(x, span, false);
                }
            }
        }
    }
}

fn entry(key: u64, payload: &[u8]) -> Vec<u8> {
    let mut e = key.to_le_bytes().to_vec();
    e.extend_from_slice(payload);
    e.resize(usize::from(ENTRY_W), 0);
    e
}

fn group(n: &Nid) -> u8 {
    match n {
        Nid::Ext(..) => 0,
        Nid::Top(..) | Nid::KTop(..) => 1,
        Nid::Cont => 2,
        Nid::Alloc(..) => 3,
    }
}

impl Pool {
    fn nid_level(&self, n: &Nid) -> u8 {
        match *n {
            Nid::Ext(_, l, _) | Nid::Top(l, _) | Nid::Alloc(l, _) => l,
            Nid::KTop(id) => self.ktree.nodes[id].level,
            Nid::Cont => 0,
        }
    }
    fn sort_key(&self, n: &Nid) -> (u8, u8, Nid) {
        (group(n), self.nid_level(n), *n)
    }
    fn subtree_root(&self, inode: u64) -> Option<Ptr> {
        let h = self.file_height(inode)?;
        if h == 0 {
            return None;
        }
        self.disk.get(&Nid::Ext(inode, h - 1, 0)).copied()
    }
    /// 上段叶条目 key 之后的载荷：标签 0 = 没有单元；1 = 下段根指针；2 = 内联的数据指针（槽 8 + CRC 4 + 版本 8）。
    fn top_payload(&self, i: u64) -> Vec<u8> {
        match self.file_height(i) {
            None => vec![0; 22],
            Some(0) => {
                let d = self.files[&i][&0];
                let mut v = vec![2];
                v.extend_from_slice(&d.slot.to_le_bytes());
                v.extend_from_slice(&d.crc.to_le_bytes());
                v.extend_from_slice(&d.ver.to_le_bytes());
                v
            }
            Some(_) => {
                let mut v = vec![1];
                v.extend(self.subtree_root(i).expect("下段根在上段之前装好").bytes());
                v
            }
        }
    }

    fn top_root_nid(&self) -> Option<Nid> {
        match self.cfg.ext {
            Ext::PerFile => None,
            Ext::GlobalRadix | Ext::GlobalRadixInline => {
                if self.files.is_empty() {
                    None
                } else {
                    Some(Nid::Top(self.cfg.theight(self.max_inode()) - 1, 0))
                }
            }
            Ext::GlobalKeyed => Some(Nid::KTop(self.ktree.root)),
        }
    }

    /// 一次用户操作 = 一次发布。装不下（落点取不到）时整次拒绝、池不变，交回 false。
    pub fn publish(&mut self, op: Op) -> bool {
        let backup = if self.naive { Some(self.clone()) } else { None };
        self.txg += 1;
        let floor = self.txg.saturating_sub(self.cfg.ring);
        self.reclaim_to(floor);
        self.dirty.clear();
        self.m = Metrics::default();
        self.pending_data.clear();
        let applied = self.apply(op).is_some();
        let ok = applied && self.finish_publish(false).is_some();
        if !ok {
            self.refusal = format!(
                "txg={} op={op:?} stage={} open_seg={:?} opened_segments={} free_slots={}",
                self.txg,
                if applied { "commit_generated" } else { "user_data" },
                self.open_seg,
                self.opened.len(),
                self.free_slots()
            );
            // 代价模型不留备份：拒绝之后池作废，调用方就此停下。
            if let Some(b) = backup {
                let refusal = std::mem::take(&mut self.refusal);
                *self = b;
                self.refusal = refusal;
            }
            self.m.refused = true;
        }
        ok
    }

    fn apply(&mut self, op: Op) -> Option<()> {
        let cap = self.cfg.tw as usize;
        let ensure = |p: &mut Pool, i: u64| {
            if !p.files.contains_key(&i) {
                p.files.insert(i, BTreeMap::new());
                if p.cfg.ext == Ext::GlobalKeyed {
                    let before = p.ktree.nodes.len();
                    for id in p.ktree.insert(i, cap) {
                        p.mark(Nid::KTop(id), Cause::CommitOther);
                    }
                    p.m.top_split |= p.ktree.nodes.len() > before;
                }
            }
        };
        match op {
            Op::Create(i) => {
                ensure(self, i);
                self.dirty_top_for(i);
            }
            Op::Range(i, start, k) => {
                for u in start..start + k {
                    self.apply(Op::Write(i, u))?;
                }
            }
            Op::Write(i, u) => {
                ensure(self, i);
                let old_h = self.file_height(i);
                let x = self.place_data()?;
                self.next_ver += 1;
                let ver = self.next_ver;
                let crc = crc_of(&[i, u, ver, self.txg]);
                let blob = if self.cfg.bytes {
                    self.blobs.push(Blob::Data { crc, inode: i, unit: u, ver, birth: self.txg });
                    self.blobs.len() - 1
                } else {
                    usize::MAX
                };
                self.pending_data.push((x, blob));
                self.record(x, 2, Cause::Data);
                let old = self.files.get_mut(&i).unwrap().insert(u, DataRef { slot: x, crc, ver });
                if let Some(o) = old {
                    self.release(o.slot, 2, Cause::Data);
                }
                let h = self.file_height(i).unwrap();
                self.m.ext_grew |= old_h.is_some_and(|o| h > o);
                self.dirty_ext(i, u, h);
                self.dirty_top_for(i);
            }
            Op::Trunc(i, keep) => {
                let Some(h) = self.file_height(i) else {
                    self.mark(Nid::Cont, Cause::CommitOther);
                    return Some(());
                };
                let gone: Vec<(u64, DataRef)> =
                    self.files[&i].range(keep..).map(|(u, d)| (*u, *d)).collect();
                for (u, d) in gone {
                    self.files.get_mut(&i).unwrap().remove(&u);
                    self.release(d.slot, 2, Cause::Data);
                    self.dirty_ext(i, u, h);
                }
                let new_h = self.file_height(i).unwrap_or(0);
                self.m.ext_shrank |= new_h < h;
                // 一次矮下去不止一层：旧根之上（含旧根）那一串位置 0 的节点也不在了，要标脏才会被当成「不在了」释放。
                // （原型第一版漏了这一步，checker 在截断之后判 I-3.1 泄漏，见报告。）
                if !self.bug_skip_shrink_spine {
                    for l in new_h..h {
                        self.mark(Nid::Ext(i, l, 0), Cause::CommitOther);
                    }
                }
                self.dirty_top_for(i);
            }
        }
        Some(())
    }

    fn finish_publish(&mut self, mkfs: bool) -> Option<()> {
        let txg = self.txg;
        let mut assigned: BTreeMap<Nid, u64> = BTreeMap::new();
        let mut bump_order: Vec<Nid> = vec![];
        let mut gone_done: BTreeSet<Nid> = BTreeSet::new();
        loop {
            let gone: Vec<Nid> = self
                .dirty
                .keys()
                .filter(|n| !gone_done.contains(*n) && self.disk.contains_key(*n) && !self.present(**n))
                .copied()
                .collect();
            let mut pending: Vec<Nid> = self
                .dirty
                .keys()
                .filter(|n| !assigned.contains_key(*n) && self.present(**n))
                .copied()
                .collect();
            if gone.is_empty() && pending.is_empty() {
                break;
            }
            self.m.rounds += 1;
            pending.sort_by_key(|n| self.sort_key(n));
            for n in gone {
                let p = self.disk[&n];
                let span = if n == Nid::Cont { 2 } else { 1 };
                let cause = if group(&n) == 3 { Cause::CommitAlloc } else { Cause::CommitOther };
                self.release(p.slot, span, cause);
                gone_done.insert(n);
            }
            for n in pending {
                let span = if n == Nid::Cont { 2 } else { 1 };
                let cause = if group(&n) == 3 { Cause::CommitAlloc } else { Cause::CommitOther };
                let x = self.place_commit(span, group(&n) == 3)?;
                self.record(x, span, cause);
                if let Some(p) = self.disk.get(&n).copied() {
                    self.release(p.slot, span, cause);
                }
                assigned.insert(n, x);
                bump_order.push(n);
            }
        }
        for n in &gone_done {
            self.disk.remove(n);
        }
        // 数
        for n in assigned.keys() {
            match n {
                Nid::Alloc(l, _) => {
                    self.m.alloc_nodes += 1;
                    if *l == 0 {
                        self.m.alloc_leaves += 1;
                        match self.dirty[n] {
                            Cause::Data => self.m.leaves_by_data += 1,
                            Cause::CommitOther => self.m.leaves_by_other += 1,
                            Cause::CommitAlloc => self.m.leaves_by_alloc += 1,
                        }
                    }
                }
                Nid::Cont => self.m.cont += 1,
                _ => self.m.ext_nodes += 1,
            }
        }
        self.m.data = self.pending_data.len() as u64;
        self.m.named = self.m.data + assigned.len() as u64;
        let ext_h = match self.cfg.ext {
            Ext::PerFile => 0,
            Ext::GlobalRadix | Ext::GlobalRadixInline => self.cfg.theight(self.max_inode()),
            Ext::GlobalKeyed => self.ktree.height(),
        };
        let file_h = self.files.keys().filter_map(|i| self.file_height(*i)).max().unwrap_or(1);
        self.m.formula = u64::from(self.cfg.atop() + 1) + u64::from(ext_h) + u64::from(file_h) + 2 + 1;
        // 装字节、录写流
        let mut build: Vec<Nid> = assigned.keys().copied().collect();
        build.sort_by_key(|n| self.sort_key(n));
        let mut units: Vec<(u64, u64, usize)> = self.pending_data.iter().map(|(x, b)| (*x, 2, *b)).collect();
        for n in build {
            let x = assigned[&n];
            let seq = bump_order.iter().position(|m| *m == n).unwrap() as u32;
            let span = if n == Nid::Cont { 2 } else { 1 };
            let (ptr, blob) = if self.cfg.bytes {
                let blob = self.build(n, txg, seq);
                let crc = blob.crc();
                self.blobs.push(blob);
                (Ptr { slot: x, crc, level: self.nid_level(&n), birth: txg }, self.blobs.len() - 1)
            } else {
                (Ptr { slot: x, crc: 0, level: self.nid_level(&n), birth: txg }, usize::MAX)
            };
            self.disk.insert(n, ptr);
            units.push((x, span, blob));
        }
        let top_root = self.top_root_nid().map(|n| self.disk[&n]);
        let fields = RootFields { alloc: self.disk[&Nid::Alloc(self.cfg.atop(), 0)], top: top_root, cont: self.disk[&Nid::Cont] };
        self.fields = Some(fields.clone());
        if !self.cfg.bytes {
            return Some(());
        }
        for (x, span, blob) in &units {
            for dev in 0..DEVS as u8 {
                self.writes.push(Write { dev, loc: Loc::Slot(*x), span: *span, blob: *blob, fua: false });
                self.stream.push(StreamOp::W(self.writes.len() - 1));
            }
        }
        self.stream.push(StreamOp::Barrier);
        let mut rec_writes = vec![];
        if !mkfs {
            self.jsn += 1;
            let named = units.iter().map(|(x, s, b)| (*x, *s, self.blobs[*b].crc())).collect();
            self.blobs.push(Blob::Rec(JRec { jsn: self.jsn, txg, fields: fields.clone(), named }));
            let b = self.blobs.len() - 1;
            for dev in 0..DEVS as u8 {
                self.writes.push(Write { dev, loc: Loc::Rec(self.jsn), span: 0, blob: b, fua: false });
                self.stream.push(StreamOp::W(self.writes.len() - 1));
                rec_writes.push(self.writes.len() - 1);
            }
            self.stream.push(StreamOp::Barrier);
        }
        self.blobs.push(Blob::Root(RootRec { txg, fields: fields.clone(), last_jsn: self.jsn }));
        let b = self.blobs.len() - 1;
        let dev = (txg % DEVS) as u8;
        let slot = (txg / DEVS) % (self.cfg.ring / DEVS);
        self.writes.push(Write { dev, loc: Loc::Root(slot), span: 0, blob: b, fua: true });
        self.stream.push(StreamOp::W(self.writes.len() - 1));
        let root_write = self.writes.len() - 1;
        let content = self
            .files
            .iter()
            .flat_map(|(i, f)| f.iter().map(move |(u, d)| ((*i, *u), d.ver)))
            .collect();
        self.versions.push(Version {
            txg,
            content,
            inodes: self.files.keys().copied().collect(),
            fields,
            root_write,
            rec_writes,
            m: self.m,
        });
        Some(())
    }

    fn build(&self, n: Nid, txg: u64, seq: u32) -> Blob {
        let cfg = &self.cfg;
        let (tree, level, lo, hi, entries): (u64, u8, u64, u64, Vec<Vec<u8>>) = match n {
            Nid::Alloc(l, idx) => {
                let span = cfg.aspan(l);
                let lo = idx * span;
                let entries = if l == 0 {
                    let (_, ds) = cfg.ageo();
                    let dev = lo / ds;
                    let a = (lo % ds).saturating_sub(cfg.aoff);
                    let b = (lo % ds + span).saturating_sub(cfg.aoff);
                    self.recs[dev as usize]
                        .range(a..b)
                        .map(|(s, r)| {
                            let mut p = (r.span as u16).to_le_bytes().to_vec();
                            p.push(u8::from(r.released));
                            p.extend_from_slice(&r.gen.to_le_bytes());
                            entry(cfg.akey(dev, *s), &p)
                        })
                        .collect()
                } else {
                    let fan = cfg.afan(l);
                    (idx * fan..(idx + 1) * fan)
                        .filter(|c| self.present(Nid::Alloc(l - 1, *c)))
                        .filter(|c| !(l == 1 && self.bug_drop_leaf == Some(*c)))
                        .map(|c| entry(c * cfg.aspan(l - 1), &self.disk[&Nid::Alloc(l - 1, c)].bytes()))
                        .collect()
                };
                (T_ALLOC, l, lo, lo + span - 1, entries)
            }
            Nid::Ext(i, l, idx) => {
                let span = cfg.espan(l);
                let lo = idx * span;
                let entries = if l == 0 {
                    self.files[&i]
                        .range(lo..lo + span)
                        .map(|(u, d)| {
                            let mut p = d.slot.to_le_bytes().to_vec();
                            p.extend_from_slice(&d.crc.to_le_bytes());
                            p.extend_from_slice(&d.ver.to_le_bytes());
                            entry(i << 32 | u, &p)
                        })
                        .collect()
                } else {
                    (idx * cfg.ef..(idx + 1) * cfg.ef)
                        .filter(|c| self.present(Nid::Ext(i, l - 1, *c)))
                        .filter(|c| !self.bug_skip_spine || self.disk.contains_key(&Nid::Ext(i, l - 1, *c)))
                        .map(|c| entry(i << 32 | c * cfg.espan(l - 1), &self.disk[&Nid::Ext(i, l - 1, c)].bytes()))
                        .collect()
                };
                (T_EXT, l, i << 32 | lo, i << 32 | (lo + span - 1), entries)
            }
            Nid::Top(l, idx) => {
                let span = cfg.tspan(l);
                let lo = idx * span;
                let entries = if l == 0 {
                    self.files
                        .range(lo..lo + span)
                        .map(|(i, _)| entry(*i, &self.top_payload(*i)))
                        .collect()
                } else {
                    (idx * cfg.ef..(idx + 1) * cfg.ef)
                        .filter(|c| self.present(Nid::Top(l - 1, *c)))
                        .map(|c| entry(c * cfg.tspan(l - 1), &self.disk[&Nid::Top(l - 1, c)].bytes()))
                        .collect()
                };
                (T_EXT, l, lo, lo + span - 1, entries)
            }
            Nid::KTop(id) => {
                let node = &self.ktree.nodes[id];
                let (lo, hi) = self.ktree.min_max(id);
                let entries = if node.level == 0 {
                    node.keys
                        .iter()
                        .map(|i| entry(*i, &self.top_payload(*i)))
                        .collect()
                } else {
                    node.keys
                        .iter()
                        .zip(&node.kids)
                        .map(|(s, k)| entry(*s, &self.disk[&Nid::KTop(*k)].bytes()))
                        .collect()
                };
                (T_EXT, node.level, lo, hi, entries)
            }
            Nid::Cont => {
                let files: BTreeMap<u64, Option<Ptr>> = self
                    .files
                    .keys()
                    .map(|i| (*i, if cfg.ext == Ext::PerFile { self.subtree_root(*i) } else { None }))
                    .collect();
                let mut b = txg.to_le_bytes().to_vec();
                for (i, p) in &files {
                    b.extend_from_slice(&i.to_le_bytes());
                    b.extend(p.map_or(vec![0; 21], |p| p.bytes()));
                }
                return Blob::Cont { crc: crc32_castagnoli(&b), files };
            }
        };
        let bytes = build_index_node(
            TreeIdentifier(tree),
            level,
            KW,
            &lo.to_le_bytes(),
            &hi.to_le_bytes(),
            CheckpointTxg(txg),
            &FSID,
            InstanceGeneration(1),
            BirthSequence(seq),
            ENTRY_W,
            &entries,
        );
        let crc = crc32_castagnoli(&bytes);
        Blob::Node { crc, header: parse_index_node(&bytes).expect("刚写的码 2 节点解得开") }
    }
}

// ---------------------------------------------------------------- 崩溃镜像、恢复、checker、oracle

pub struct Image<'a> {
    pub pool: &'a Pool,
    pub base: &'a HashMap<(u8, Loc), (usize, u8)>,
    pub overlay: &'a HashMap<(u8, Loc), Vec<(usize, usize, u8)>>,
    pub persisted: &'a [bool],
}

impl<'p> Image<'p> {
    pub fn read(&self, dev: u8, loc: Loc) -> Option<(usize, u8)> {
        if let Some(list) = self.overlay.get(&(dev, loc)) {
            if let Some((_, b, part)) = list.iter().rev().find(|(w, _, _)| self.persisted[*w]) {
                return Some((*b, *part));
            }
        }
        self.base.get(&(dev, loc)).copied()
    }
    pub fn read_unit(&self, slot: u64, span: u64, crc: u32) -> Option<&'p Blob> {
        let pool: &'p Pool = self.pool;
        for dev in 0..DEVS as u8 {
            let first = self.read(dev, Loc::Slot(slot));
            let Some((b, 0)) = first else { continue };
            if pool.blobs[b].crc() != crc {
                continue;
            }
            if (1..span).all(|i| self.read(dev, Loc::Slot(slot + i)) == Some((b, i as u8))) {
                return Some(&pool.blobs[b]);
            }
        }
        None
    }
    fn node(&self, p: Ptr) -> Option<&'p IndexNodeHeader> {
        match self.read_unit(p.slot, 1, p.crc)? {
            Blob::Node { header, .. } => Some(header),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Recovered {
    pub txg: u64,
    pub fields: RootFields,
    pub applied: u64,
}

pub fn recover(img: &Image<'_>) -> Result<Recovered, String> {
    let mut best: Option<&RootRec> = None;
    for dev in 0..DEVS as u8 {
        for k in 0..img.pool.cfg.ring / DEVS {
            if let Some((b, _)) = img.read(dev, Loc::Root(k)) {
                if let Blob::Root(r) = &img.pool.blobs[b] {
                    if best.is_none_or(|x| r.txg > x.txg) {
                        best = Some(r);
                    }
                }
            }
        }
    }
    let root = best.ok_or("根环里没有一条根")?;
    let mut cur = Recovered { txg: root.txg, fields: root.fields.clone(), applied: 0 };
    let mut j = root.last_jsn + 1;
    loop {
        let rec = (0..DEVS as u8).find_map(|dev| match img.read(dev, Loc::Rec(j)).map(|(b, _)| &img.pool.blobs[b]) {
            Some(Blob::Rec(r)) => Some(r),
            _ => None,
        });
        let Some(r) = rec else { break };
        if r.txg != cur.txg + 1 {
            break;
        }
        if !r.named.iter().all(|(s, span, crc)| img.read_unit(*s, *span, *crc).is_some()) {
            break;
        }
        cur = Recovered { txg: r.txg, fields: r.fields.clone(), applied: cur.applied + 1 };
        j += 1;
    }
    Ok(cur)
}

#[derive(Default, Debug)]
pub struct Checked {
    pub content: BTreeMap<(u64, u64), u64>,
    pub inodes: BTreeSet<u64>,
    pub crossing: u64,
    pub recs: Vec<BTreeMap<u64, Rec>>,
}

fn key_of(e: &[u8]) -> u64 {
    u64::from_le_bytes(e[..8].try_into().unwrap())
}

struct Walk<'a, 'p> {
    img: &'a Image<'p>,
    txg: u64,
    reach: Vec<(u64, u64, u64)>,
    out: Checked,
}

impl<'p> Walk<'_, 'p> {
    fn node_checked(&mut self, p: Ptr, tree: u64, level: u8, lo: u64, hi: u64, what: &str) -> Result<&'p IndexNodeHeader, String> {
        let h = self.img.node(p).ok_or_else(|| format!("{what} 槽 {} 读不出或 CRC 不符", p.slot))?;
        if h.tree.0 != tree || h.level != level || p.level != level {
            return Err(format!("{what} 槽 {} 树 ID / 层级不符", p.slot));
        }
        if h.birth_txg.0 != p.birth || p.birth > self.txg {
            return Err(format!("{what} 槽 {} 诞生代号不符", p.slot));
        }
        let (a, b) = (key_of(&h.smallest_key), key_of(&h.largest_key));
        if (a, b) != (lo, hi) {
            return Err(format!("{what} 槽 {} 头里的区间 [{a},{b}] ≠ 按位置规定的 [{lo},{hi}]", p.slot));
        }
        let keys: Vec<u64> = h.entries.iter().map(|e| key_of(e)).collect();
        if keys.windows(2).any(|w| w[0] >= w[1]) || keys.iter().any(|k| *k < lo || *k > hi) {
            return Err(format!("{what} 槽 {} 条目 key 乱序或出了区间", p.slot));
        }
        self.reach.push((p.slot, 1, p.birth));
        Ok(h)
    }

    fn alloc(&mut self, p: Ptr, l: u8, idx: u64) -> Result<(), String> {
        let cfg = self.img.pool.cfg;
        let span = cfg.aspan(l);
        let lo = idx * span;
        let h = self.node_checked(p, T_ALLOC, l, lo, lo + span - 1, "分配记录节点")?;
        if l == 0 {
            for e in &h.entries {
                let k = key_of(e);
                let (dev, slot) = cfg.aunkey(k).ok_or("分配记录的 key 不在池的几何里")?;
                let rspan = u64::from(u16::from_le_bytes(e[8..10].try_into().unwrap()));
                let rec = Rec { span: rspan, released: e[10] == 1, gen: u64::from_le_bytes(e[11..19].try_into().unwrap()) };
                if rspan == 0 || slot + rspan > cfg.slots {
                    return Err("记录跨度越界".into());
                }
                if k + rspan - 1 > lo + span - 1 {
                    self.out.crossing += 1;
                }
                self.out.recs[dev as usize].insert(slot, rec);
            }
            return Ok(());
        }
        let fan = cfg.afan(l);
        let child_span = cfg.aspan(l - 1);
        let mut seen = BTreeSet::new();
        for e in &h.entries {
            let k = key_of(e);
            if k % child_span != 0 || k / child_span < idx * fan || k / child_span >= (idx + 1) * fan {
                return Err("内部条目的 key 不是某个孩子的规定起点".into());
            }
            if Ptr::is_null(&e[8..]) {
                return Err("内部条目指针为空".into());
            }
            seen.insert(k / child_span);
            self.alloc(Ptr::parse(&e[8..]), l - 1, k / child_span)?;
        }
        if cfg.absence == Absence::Full {
            for c in idx * fan..(idx + 1) * fan {
                let clo = c * child_span;
                let (_, ds) = cfg.ageo();
                let valid = clo / ds < DEVS && {
                    let rel = clo % ds;
                    rel + child_span > cfg.aoff && rel < cfg.aoff + cfg.slots
                };
                if valid && !seen.contains(&c) {
                    return Err(format!("Full：第 {l} 层节点 {idx} 缺第 {c} 个孩子（覆盖有洞）"));
                }
            }
        }
        Ok(())
    }

    fn ext(&mut self, i: u64, p: Ptr, l: u8, idx: u64) -> Result<(), String> {
        let cfg = self.img.pool.cfg;
        let span = cfg.espan(l);
        let lo = i << 32 | idx * span;
        let h = self.node_checked(p, T_EXT, l, lo, lo + span - 1, "extent 节点")?;
        if h.entries.is_empty() {
            return Err("extent 节点是空的（缺席应当不写）".into());
        }
        for e in &h.entries {
            let k = key_of(e);
            if l == 0 {
                let slot = u64::from_le_bytes(e[8..16].try_into().unwrap());
                let crc = u32::from_le_bytes(e[16..20].try_into().unwrap());
                let ver = u64::from_le_bytes(e[20..28].try_into().unwrap());
                let b = self.img.read_unit(slot, 2, crc).ok_or_else(|| format!("数据单元槽 {slot} 读不出"))?;
                let Blob::Data { inode, unit, ver: v, birth, .. } = b else { return Err("数据指针指到的不是数据单元".into()) };
                if *inode != i || *unit != k & 0xffff_ffff || *v != ver {
                    return Err("数据单元的身份与 extent 条目不符".into());
                }
                self.reach.push((slot, 2, *birth));
                self.out.content.insert((i, k & 0xffff_ffff), ver);
            } else {
                let cs = cfg.espan(l - 1);
                let rel = k & 0xffff_ffff;
                if k >> 32 != i || rel % cs != 0 || rel / cs < idx * cfg.ef || rel / cs >= (idx + 1) * cfg.ef || Ptr::is_null(&e[8..]) {
                    return Err("extent 内部条目不是某个孩子的规定起点".into());
                }
                self.ext(i, Ptr::parse(&e[8..]), l - 1, rel / cs)?;
            }
        }
        Ok(())
    }

    fn file_root(&mut self, i: u64, e: &[u8]) -> Result<(), String> {
        if !self.out.inodes.contains(&i) {
            return Err(format!("上段里的 inode {i} 在 inode 容器里没有"));
        }
        match e[0] {
            0 => {}
            1 => {
                let p = Ptr::parse(&e[1..]);
                self.ext(i, p, p.level, 0)?;
            }
            2 if self.img.pool.cfg.ext == Ext::GlobalRadixInline => {
                let slot = u64::from_le_bytes(e[1..9].try_into().unwrap());
                let crc = u32::from_le_bytes(e[9..13].try_into().unwrap());
                let ver = u64::from_le_bytes(e[13..21].try_into().unwrap());
                let b = self.img.read_unit(slot, 2, crc).ok_or_else(|| format!("内联数据单元槽 {slot} 读不出"))?;
                let Blob::Data { inode, unit, ver: v, birth, .. } = b else { return Err("内联指针指到的不是数据单元".into()) };
                if *inode != i || *unit != 0 || *v != ver {
                    return Err("内联数据单元的身份与条目不符".into());
                }
                self.reach.push((slot, 2, *birth));
                self.out.content.insert((i, 0), ver);
            }
            _ => return Err("上段叶条目标签非法".into()),
        }
        Ok(())
    }

    fn top(&mut self, p: Ptr, l: u8, idx: u64, seen: &mut BTreeSet<u64>) -> Result<(), String> {
        let cfg = self.img.pool.cfg;
        let span = cfg.tspan(l);
        let lo = idx * span;
        let h = self.node_checked(p, T_EXT, l, lo, lo + span - 1, "上段节点")?;
        for e in &h.entries {
            let k = key_of(e);
            if l == 0 {
                seen.insert(k);
                self.file_root(k, &e[8..])?;
            } else {
                let cs = cfg.tspan(l - 1);
                if k % cs != 0 || Ptr::is_null(&e[8..]) {
                    return Err("上段内部条目不是孩子的规定起点".into());
                }
                self.top(Ptr::parse(&e[8..]), l - 1, k / cs, seen)?;
            }
        }
        Ok(())
    }

    fn ktop(&mut self, p: Ptr, seen: &mut BTreeSet<u64>) -> Result<(u64, u64), String> {
        let h = self.img.node(p).ok_or("上段（按 key）节点读不出")?;
        if h.tree.0 != T_EXT || h.level != p.level || h.birth_txg.0 != p.birth || p.birth > self.txg {
            return Err("上段（按 key）节点身份不符".into());
        }
        self.reach.push((p.slot, 1, p.birth));
        let (a, b) = (key_of(&h.smallest_key), key_of(&h.largest_key));
        let mut sub = (u64::MAX, 0);
        let mut prev_hi: Option<u64> = None;
        for e in &h.entries {
            let k = key_of(e);
            let (lo, hi) = if h.level == 0 {
                seen.insert(k);
                self.file_root(k, &e[8..])?;
                (k, k)
            } else {
                let (lo, hi) = self.ktop(Ptr::parse(&e[8..]), seen)?;
                if k > lo || prev_hi.is_some_and(|x| k <= x) {
                    return Err("分隔 key 与孩子区间不符".into());
                }
                (lo, hi)
            };
            prev_hi = Some(hi);
            sub = (sub.0.min(lo), sub.1.max(hi));
        }
        if !h.entries.is_empty() && (a, b) != sub {
            return Err("上段（按 key）节点头里的区间 ≠ 子树覆盖区间".into());
        }
        Ok(sub)
    }
}

pub fn check(img: &Image<'_>, rec: &Recovered) -> Result<Checked, String> {
    let pool = img.pool;
    let cfg = pool.cfg;
    let mut w = Walk { img, txg: rec.txg, reach: vec![], out: Checked { recs: vec![BTreeMap::new(); DEVS as usize], ..Checked::default() } };
    w.alloc(rec.fields.alloc, cfg.atop(), 0)?;
    for dev in 0..DEVS as usize {
        let mut end = 0;
        for (s, r) in &w.out.recs[dev] {
            if *s < end {
                return Err("I-5.4：同一块盘上两条记录罩住同一个槽".into());
            }
            end = s + r.span;
        }
    }
    let cont = img.read_unit(rec.fields.cont.slot, 2, rec.fields.cont.crc).ok_or("inode 容器读不出")?;
    let Blob::Cont { files, .. } = cont else { return Err("inode 容器指针指到别的东西".into()) };
    if rec.fields.cont.birth > rec.txg {
        return Err("容器诞生代号晚于根".into());
    }
    w.reach.push((rec.fields.cont.slot, 2, rec.fields.cont.birth));
    w.out.inodes = files.keys().copied().collect();
    match cfg.ext {
        Ext::PerFile => {
            for (i, p) in files.clone() {
                if let Some(p) = p {
                    w.ext(i, p, p.level, 0)?;
                }
            }
        }
        Ext::GlobalRadix | Ext::GlobalRadixInline => {
            let mut seen = BTreeSet::new();
            if let Some(p) = rec.fields.top {
                w.top(p, p.level, 0, &mut seen)?;
            }
            if seen != w.out.inodes {
                return Err("上段里的 inode 集合 ≠ 容器里的".into());
            }
        }
        Ext::GlobalKeyed => {
            let mut seen = BTreeSet::new();
            let p = rec.fields.top.ok_or("按 key 的上段没有根")?;
            w.ktop(p, &mut seen)?;
            if seen != w.out.inodes {
                return Err("上段里的 inode 集合 ≠ 容器里的".into());
            }
        }
    }
    // I-3.1 / I-3.10 / I-5.1 的缩写：走得到的每个单元在每块盘上都有一条未释放、跨度对、分配代 = 诞生代号的记录；未释放的记录都走得到；走得到的单元不相交。
    let mut reach = w.reach.clone();
    reach.sort();
    for win in reach.windows(2) {
        if win[0].0 + win[0].1 > win[1].0 {
            return Err(format!("I-5.1：走得到的两个单元罩住同一个槽 {}", win[1].0));
        }
    }
    let reach_slots: BTreeSet<u64> = reach.iter().map(|r| r.0).collect();
    for dev in 0..DEVS as usize {
        for (s, span, birth) in &reach {
            match w.out.recs[dev].get(s) {
                Some(r) if !r.released && r.span == *span && r.gen == *birth => {}
                _ => return Err(format!("I-3.1/I-3.10：走得到的单元（盘 {dev} 槽 {s}）没有对得上的未释放记录")),
            }
        }
        for (s, r) in &w.out.recs[dev] {
            if !r.released && !reach_slots.contains(s) {
                return Err(format!("I-3.1：未释放的记录（盘 {dev} 槽 {s}）没有单元引用"));
            }
        }
    }
    Ok(w.out)
}

// ---------------------------------------------------------------- 一段历史的崩溃枚举

#[derive(Clone, Debug, Default)]
pub struct Tally {
    pub histories: u64,
    pub publishes: u64,
    pub refused: u64,
    pub too_big: u64,
    pub states: u64,
    pub closed_form: u64,
    pub classes: u64,
    pub recover_fail: u64,
    pub check_red: u64,
    pub content_mismatch: u64,
    pub root_lost: u64,
    pub applied_by_record: u64,
    pub crossing_states: u64,
    pub double_alloc: u64,
    pub release_missing: u64,
    pub fallback: u64,
    pub grew: u64,
    pub shrank: u64,
    pub top_split: u64,
    pub units: u64,
    pub max_units: u64,
    pub alloc_nodes: u64,
    pub ext_nodes: u64,
    pub leaves_by_alloc: u64,
    pub max_segment_writes: u64,
    pub red_reasons: BTreeMap<String, u64>,
    pub examples: Vec<String>,
}

impl Tally {
    pub fn add(&mut self, o: &Tally) {
        macro_rules! s { ($($f:ident),*) => { $( self.$f += o.$f; )* } }
        s!(histories, publishes, refused, too_big, states, closed_form, classes, recover_fail, check_red, content_mismatch, root_lost,
           applied_by_record, crossing_states, double_alloc, release_missing, fallback, grew, shrank, top_split, units, alloc_nodes, ext_nodes, leaves_by_alloc);
        self.max_units = self.max_units.max(o.max_units);
        self.max_segment_writes = self.max_segment_writes.max(o.max_segment_writes);
        for (k, v) in &o.red_reasons {
            *self.red_reasons.entry(k.clone()).or_default() += v;
        }
        for e in &o.examples {
            if self.examples.len() < 4 {
                self.examples.push(e.clone());
            }
        }
    }
    pub fn line(&self, name: &str) -> String {
        format!(
            "{name} histories={} publishes={} refused={} too_big={} states={} closed_form={} classes={} recover_fail={} check_red={} content_mismatch={} root_lost={} \
             applied_by_record={} crossing_states={} double_alloc={} release_missing={} fallback={} grew={} shrank={} top_split={} units={} max_units={} \
             alloc_nodes={} ext_nodes={} leaves_by_alloc_chain={} max_segment_writes={} red_reasons={:?}",
            self.histories, self.publishes, self.refused, self.too_big, self.states, self.closed_form, self.classes, self.recover_fail, self.check_red,
            self.content_mismatch, self.root_lost, self.applied_by_record, self.crossing_states, self.double_alloc, self.release_missing, self.fallback,
            self.grew, self.shrank, self.top_split, self.units, self.max_units, self.alloc_nodes, self.ext_nodes, self.leaves_by_alloc,
            self.max_segment_writes, self.red_reasons
        )
    }
}

fn reason_class(e: &str) -> String {
    e.split(|c: char| c == '：' || c.is_ascii_digit()).next().unwrap_or(e).trim().to_string()
}

/// 前缀之后的池（全部持久、不枚举）接着发 `suffix`，枚举这几次发布写流上的全部崩溃状态。`full` = 不用三类合并、逐个子集枚举（自证用）。
pub fn run_history(after_prefix: &Pool, suffix: &[Op], full: bool, max_groups: usize) -> Tally {
    run_history_from(after_prefix, suffix, full, max_groups, 0)
}

/// `enumerate_from`：后缀里前几次发布并进起点镜像、不枚举（「起点镜像不枚举 + 只录那几次」）。
pub fn run_history_from(after_prefix: &Pool, suffix: &[Op], full: bool, max_groups: usize, enumerate_from: usize) -> Tally {
    let mut pool = after_prefix.clone();
    let mut t = Tally { histories: 1, ..Tally::default() };
    let metrics_start = pool.versions.len();
    for op in &suffix[..enumerate_from] {
        if !pool.publish(*op) {
            t.refused += 1;
        }
    }
    let stream_start = pool.stream.len();
    let write_start = pool.writes.len();
    let versions_start = pool.versions.len();
    for op in &suffix[enumerate_from..] {
        if !pool.publish(*op) {
            t.refused += 1;
        }
    }
    for v in &pool.versions[metrics_start..] {
        t.publishes += 1;
        t.double_alloc += v.m.double_alloc;
        t.release_missing += v.m.release_missing;
        t.fallback += v.m.fallback;
        t.grew += u64::from(v.m.ext_grew);
        t.shrank += u64::from(v.m.ext_shrank);
        t.top_split += u64::from(v.m.top_split);
        t.units += v.m.named;
        t.max_units = t.max_units.max(v.m.named);
        t.alloc_nodes += v.m.alloc_nodes;
        t.ext_nodes += v.m.ext_nodes;
        t.leaves_by_alloc += v.m.leaves_by_alloc;
    }
    let mut base: HashMap<(u8, Loc), (usize, u8)> = HashMap::new();
    let put = |m: &mut HashMap<(u8, Loc), (usize, u8)>, w: &Write| match w.loc {
        Loc::Slot(x) => {
            for i in 0..w.span {
                m.insert((w.dev, Loc::Slot(x + i)), (w.blob, i as u8));
            }
        }
        loc => {
            m.insert((w.dev, loc), (w.blob, 0));
        }
    };
    for w in &pool.writes[..write_start] {
        put(&mut base, w);
    }
    let n = pool.writes.len() - write_start;
    let mut overlay: HashMap<(u8, Loc), Vec<(usize, usize, u8)>> = HashMap::new();
    for (rel, w) in pool.writes[write_start..].iter().enumerate() {
        match w.loc {
            Loc::Slot(x) => {
                for i in 0..w.span {
                    overlay.entry((w.dev, Loc::Slot(x + i))).or_default().push((rel, w.blob, i as u8));
                }
            }
            loc => overlay.entry((w.dev, loc)).or_default().push((rel, w.blob, 0)),
        }
    }
    // 切段（物理写）并按「同一位置同一 blob、不同盘」合成逻辑写
    let mut segments: Vec<Vec<usize>> = vec![];
    let mut cur = vec![];
    for op in &pool.stream[stream_start..] {
        match op {
            StreamOp::Barrier => {
                if !cur.is_empty() {
                    segments.push(std::mem::take(&mut cur));
                }
            }
            StreamOp::W(wi) => {
                cur.push(wi - write_start);
                if pool.writes[*wi].fua {
                    segments.push(std::mem::take(&mut cur));
                }
            }
        }
    }
    if !cur.is_empty() {
        segments.push(cur);
    }
    t.closed_form = closed_form_state_count(&segments);
    t.max_segment_writes = segments.iter().map(|s| s.len() as u64).max().unwrap_or(0);
    let prefix_newest = pool.versions[..versions_start].last().map_or(0, |v| v.txg);
    let suffix_versions: Vec<Version> = pool.versions[versions_start..].to_vec();
    let mut persisted = vec![false; n];
    let pool_ref = &pool;
    let mut eval = |persisted: &[bool], weight: u64, t: &mut Tally| {
        t.states += weight;
        t.classes += 1;
        let img = Image { pool: pool_ref, base: &base, overlay: &overlay, persisted };
        let rec = match recover(&img) {
            Ok(r) => r,
            Err(e) => {
                t.recover_fail += weight;
                *t.red_reasons.entry(format!("recover:{}", reason_class(&e))).or_default() += weight;
                return;
            }
        };
        t.applied_by_record += weight * u64::from(rec.applied > 0);
        let checked = match check(&img, &rec) {
            Ok(c) => c,
            Err(e) => {
                t.check_red += weight;
                *t.red_reasons.entry(reason_class(&e)).or_default() += weight;
                if t.examples.len() < 2 {
                    t.examples.push(format!("CHECK_RED {e}; recovered txg {}; suffix={suffix:?}", rec.txg));
                }
                return;
            }
        };
        if checked.crossing > 0 {
            t.crossing_states += weight;
        }
        let version = pool_ref.versions.iter().find(|v| v.txg == rec.txg);
        match version {
            Some(v) if v.content == checked.content && v.inodes == checked.inodes => {}
            _ => {
                t.content_mismatch += weight;
                if t.examples.len() < 2 {
                    t.examples.push(format!("CONTENT_MISMATCH recovered txg {}; suffix={suffix:?}", rec.txg));
                }
            }
        }
        let newest = suffix_versions
            .iter()
            .filter(|v| persisted[v.root_write - write_start])
            .map(|v| v.txg)
            .max()
            .unwrap_or(prefix_newest);
        if rec.txg < newest {
            t.root_lost += weight;
        }
    };
    eval(&persisted, 1, &mut t);
    for seg in &segments {
        // 逻辑写：同一位置、同一 blob 的几份
        let mut groups: Vec<Vec<usize>> = vec![];
        for w in seg {
            let wr = pool.writes[write_start + w];
            match groups.last_mut() {
                Some(g) if { let f = pool.writes[write_start + g[0]]; f.loc == wr.loc && f.blob == wr.blob } => g.push(*w),
                _ => groups.push(vec![*w]),
            }
        }
        if groups.len() > max_groups {
            t.too_big += 1;
            for w in seg {
                persisted[*w] = true;
            }
            continue;
        }
        // 每个逻辑写的类：(每份落不落, 权重)
        let img_now = Image { pool: &pool, base: &base, overlay: &overlay, persisted: &persisted };
        let classes: Vec<Vec<(Vec<bool>, u64)>> = groups
            .iter()
            .map(|g| {
                if g.len() == 1 {
                    return vec![(vec![false], 1), (vec![true], 1)];
                }
                assert_eq!(g.len(), 2);
                let w0 = pool.writes[write_start + g[0]];
                let locs: Vec<Loc> = match w0.loc {
                    Loc::Slot(x) => (0..w0.span).map(|i| Loc::Slot(x + i)).collect(),
                    l => vec![l],
                };
                let symmetric = locs.iter().all(|l| img_now.read(0, *l) == img_now.read(1, *l));
                if full || !symmetric {
                    vec![(vec![false, false], 1), (vec![true, false], 1), (vec![false, true], 1), (vec![true, true], 1)]
                } else {
                    vec![(vec![false, false], 1), (vec![true, false], 2), (vec![true, true], 1)]
                }
            })
            .collect();
        let mut digits = vec![0usize; groups.len()];
        loop {
            // 下一个组合（跳过全不落：它就是上一段全落的那个状态）
            let mut i = 0;
            loop {
                if i == digits.len() {
                    break;
                }
                digits[i] += 1;
                if digits[i] < classes[i].len() {
                    break;
                }
                digits[i] = 0;
                i += 1;
            }
            if i == digits.len() {
                break;
            }
            let mut weight = 1;
            for (gi, g) in groups.iter().enumerate() {
                let (bits, wgt) = &classes[gi][digits[gi]];
                weight *= wgt;
                for (k, w) in g.iter().enumerate() {
                    persisted[*w] = bits[k];
                }
            }
            eval(&persisted, weight, &mut t);
        }
        for w in seg {
            persisted[*w] = true;
        }
    }
    if t.too_big == 0 {
        assert_eq!(t.states, t.closed_form, "加权状态数等于闭式");
    }
    t
}

// ---------------------------------------------------------------- 从盘上点查（挂载之后被问到「这个槽空不空」那一刻）

/// 从镜像上按位置走到槽 x 所在的叶，答它空不空。Absent 下空指针 = 空闲；Full 下空指针 = 损坏（Err）。
pub fn disk_point_free(img: &Image<'_>, fields: &RootFields, dev: u64, x: u64, floor: u64) -> Result<bool, String> {
    let cfg = img.pool.cfg;
    let key = cfg.akey(dev, x);
    let mut p = fields.alloc;
    let mut l = cfg.atop();
    loop {
        let h = img.node(p).ok_or("节点读不出")?;
        if l == 0 {
            let lo = key / cfg.aw * cfg.aw;
            let mut free = true;
            for e in &h.entries {
                let k = key_of(e);
                let span = u64::from(u16::from_le_bytes(e[8..10].try_into().unwrap()));
                let rec = Rec { span, released: e[10] == 1, gen: u64::from_le_bytes(e[11..19].try_into().unwrap()) };
                if k <= key && k + span > key && live(&rec, floor) {
                    free = false;
                }
            }
            let _ = lo;
            return Ok(free);
        }
        let child = key / cfg.aspan(l - 1);
        let e = h.entries.iter().find(|e| key_of(e) == child * cfg.aspan(l - 1));
        match e {
            Some(e) => {
                p = Ptr::parse(&e[8..]);
                l -= 1;
            }
            None => {
                return match cfg.absence {
                    Absence::Absent => Ok(true),
                    Absence::Full => Err(format!("第 {l} 层缺孩子 {child}")),
                }
            }
        }
    }
}

// ---------------------------------------------------------------- 候选、前缀、菜单

pub fn crash_cfg(name: &'static str) -> Cfg {
    Cfg {
        name,
        slots: 240,
        seg: 8,
        ring: 4,
        aw: 16,
        af: 8,
        aoff: 0,
        aflat: true,
        absence: Absence::Absent,
        cross: Cross::LookBack,
        ext: Ext::PerFile,
        ew: 2,
        ef: 2,
        tw: 2,
        bytes: true,
        aregion: 0,
    }
}

pub fn s1_cfgs() -> Vec<Cfg> {
    let b = crash_cfg("S1-Absent-LookBack-even");
    vec![
        b,
        Cfg { name: "S1-Full-LookBack-even", absence: Absence::Full, ..b },
        Cfg { name: "S1-Absent-NoLookBack-even", cross: Cross::NoLookBack, ..b },
        Cfg { name: "S1-Absent-LookBack-odd", aoff: 1, ..b },
        Cfg { name: "S1-Absent-NoLookBack-odd", aoff: 1, cross: Cross::NoLookBack, ..b },
        Cfg { name: "S1-Absent-LookBack-odd-fallback", aoff: 1, seg: 0, ..b },
        Cfg { name: "S1-Absent-NoLookBack-odd-fallback", aoff: 1, seg: 0, cross: Cross::NoLookBack, ..b },
        Cfg { name: "S1-Absent-NoLookBack-even-fallback", seg: 0, cross: Cross::NoLookBack, ..b },
    ]
}

pub fn s2_cfgs() -> Vec<Cfg> {
    let b = crash_cfg("S2-PerFile");
    vec![
        b,
        Cfg { name: "S2-GlobalKeyed", ext: Ext::GlobalKeyed, ..b },
        Cfg { name: "S2-GlobalRadix", ext: Ext::GlobalRadix, ..b },
        Cfg { name: "S2-GlobalRadixInline", ext: Ext::GlobalRadixInline, ..b },
    ]
}

pub fn prefixes() -> Vec<(&'static str, Vec<Op>)> {
    use Op::*;
    let pb: Vec<Op> = vec![Write(1, 0), Write(1, 1), Write(1, 2), Write(1, 3), Write(2, 0), Write(2, 1)];
    let mut pc = pb.clone();
    pc.extend([Write(1, 0), Write(2, 0), Write(1, 2), Write(1, 1), Write(2, 1), Write(1, 3), Write(1, 0), Write(2, 0)]);
    let mut pd = pb.clone();
    pd.extend([Create(3), Write(3, 5), Write(4, 0)]);
    vec![("P1", vec![Write(1, 0)]), ("PA", vec![Write(1, 0), Write(1, 1)]), ("PB", pb), ("PC", pc), ("PD", pd)]
}

/// 用户动作的菜单（前缀之后第一步、第二步都从这里任取，全组合）。
pub fn menu(p: &Pool) -> Vec<Op> {
    let mut m = vec![];
    for (i, f) in &p.files {
        let Some(max) = f.keys().next_back().copied() else {
            m.push(Op::Write(*i, 0));
            continue;
        };
        for u in f.keys().take(2) {
            m.push(Op::Write(*i, *u));
        }
        m.push(Op::Write(*i, max + 1));
        let h = p.cfg.eheight(max);
        m.push(Op::Write(*i, p.cfg.espan(h - 1) * 2));
        m.push(Op::Trunc(*i, 1));
        m.push(Op::Trunc(*i, 0));
    }
    let next = p.files.keys().next_back().copied().unwrap_or(0) + 1;
    m.push(Op::Write(next, 0));
    m.push(Op::Create(next + 1));
    m
}

pub fn pool_after(cfg: Cfg, ops: &[Op]) -> (Pool, u64) {
    let mut p = Pool::new(cfg);
    let mut refused = 0;
    for op in ops {
        if !p.publish(*op) {
            refused += 1;
        }
    }
    (p, refused)
}

fn enum_from() -> usize {
    std::env::var("KS_ENUM_FROM").ok().and_then(|s| s.parse().ok()).unwrap_or(0)
}

fn threads() -> usize {
    std::env::var("KS_THREADS").ok().and_then(|s| s.parse().ok()).unwrap_or(8)
}

pub fn scan(cfg: Cfg, max_groups: usize) -> (Tally, Vec<String>) {
    let mut notes = vec![];
    let total = Mutex::new(Tally::default());
    let only = std::env::var("KS_PREFIXES").ok();
    for (pname, prefix) in prefixes() {
        if only.as_ref().is_some_and(|o| !o.split(',').any(|x| x == pname)) {
            continue;
        }
        let (pool, refused) = pool_after(cfg, &prefix);
        let prefix_dbl: u64 = pool.versions.iter().map(|v| v.m.double_alloc).sum();
        notes.push(format!("  prefix {pname}: refused_in_prefix={refused} double_alloc_in_prefix={prefix_dbl} txg={} files={:?}", pool.txg,
            pool.files.iter().map(|(i, f)| (*i, f.len())).collect::<Vec<_>>()));
        let menu = menu(&pool);
        // 单步后缀整段枚举；两步后缀只枚举第二步（第一步并进起点镜像）——第一步的崩溃状态由单步后缀那一批罩着。
        let mut jobs: Vec<(Vec<Op>, usize)> = menu.iter().map(|a| (vec![*a], 0)).collect();
        jobs.extend(menu.iter().flat_map(|a| menu.iter().map(move |b| (vec![*a, *b], 1))));
        let started = std::time::Instant::now();
        let next = std::sync::atomic::AtomicUsize::new(0);
        std::thread::scope(|s| {
            for _ in 0..threads() {
                s.spawn(|| {
                    let mut local = Tally::default();
                    loop {
                        let j = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if j >= jobs.len() {
                            break;
                        }
                        let (ops, from) = &jobs[j];
                        local.add(&run_history_from(&pool, ops, false, max_groups, *from));
                    }
                    total.lock().unwrap().add(&local);
                });
            }
        });
        notes.push(format!("  prefix {pname}: histories={} secs={}", jobs.len(), started.elapsed().as_secs()));
    }
    (total.into_inner().unwrap(), notes)
}

// ---------------------------------------------------------------- 用例

fn print_pool_shape(tag: &str, p: &Pool) {
    let alloc_nodes = p.disk.keys().filter(|n| matches!(n, Nid::Alloc(..))).count();
    let ext_nodes = p.disk.keys().filter(|n| matches!(n, Nid::Ext(..) | Nid::Top(..) | Nid::KTop(..))).count();
    println!(
        "SHAPE {tag} cfg={} txg={} alloc_levels={} alloc_nodes_on_disk={} ext_nodes_on_disk={} files={:?} records_dev0={}",
        p.cfg.name,
        p.txg,
        p.cfg.atop() + 1,
        alloc_nodes,
        ext_nodes,
        p.files.iter().map(|(i, f)| (*i, f.len(), p.file_height(*i))).collect::<Vec<_>>(),
        p.recs[0].len()
    );
}

fn full_image(p: &Pool) -> (HashMap<(u8, Loc), (usize, u8)>, HashMap<(u8, Loc), Vec<(usize, usize, u8)>>) {
    let mut base = HashMap::new();
    for w in &p.writes {
        match w.loc {
            Loc::Slot(x) => {
                for i in 0..w.span {
                    base.insert((w.dev, Loc::Slot(x + i)), (w.blob, i as u8));
                }
            }
            loc => {
                base.insert((w.dev, loc), (w.blob, 0));
            }
        }
    }
    (base, HashMap::new())
}

#[test]
fn selftest_enumeration_checker_and_known_bad_images() {
    // 1. 三类合并与逐子集枚举逐项相等（两条流：一条干净、一条会红）。
    for (cfg, prefix, suffix) in [
        (crash_cfg("S1-Absent-LookBack-even"), 0, vec![Op::Write(1, 1)]),
        (Cfg { name: "S1-Absent-NoLookBack-odd-fallback", aoff: 1, seg: 0, cross: Cross::NoLookBack, ..crash_cfg("x") }, 0, vec![Op::Write(1, 1)]),
        (Cfg { name: "S2-GlobalKeyed", ext: Ext::GlobalKeyed, ..crash_cfg("x") }, 0, vec![Op::Write(2, 0)]),
        (Cfg { name: "S2-GlobalRadixInline", ext: Ext::GlobalRadixInline, ..crash_cfg("x") }, 0, vec![Op::Write(1, 1)]),
    ] {
        let (pname, pops) = &prefixes()[prefix];
        let (pool, _) = pool_after(cfg, pops);
        print_pool_shape(pname, &pool);
        let a = run_history_from(&pool, &suffix, true, 64, 0);
        let b = run_history_from(&pool, &suffix, false, 64, 0);
        println!("SELFTEST_FULL    {}", a.line(cfg.name));
        println!("SELFTEST_CLASSES {}", b.line(cfg.name));
        for e in &b.examples {
            println!("  example: {e}");
        }
        let strip = |t: &Tally| { let mut t = t.clone(); t.classes = 0; t.examples.clear(); t.line("") };
        assert_eq!(strip(&a), strip(&b), "三类合并与逐子集枚举在每个计数上相等");
        if cfg.cross == Cross::NoLookBack {
            assert!(a.check_red > 0 && a.double_alloc > 0, "奇数叶边界 + 不回看的读者：枚举里 checker 必须判红（checker 会红的自证）");
        }
    }
    // 2. 写者漏掉一片叶的指针（坏镜像）：Absent 与 Full 各自谁判得出、挂载之后点查答什么。
    for absence in [Absence::Absent, Absence::Full] {
        let cfg = Cfg { name: if absence == Absence::Absent { "S1-Absent" } else { "S1-Full" }, absence, ..crash_cfg("x") };
        let (mut pool, _) = pool_after(cfg, &prefixes()[2].1);
        let data_slot = pool.files[&1][&0].slot;
        let leaf = cfg.akey(0, data_slot) / cfg.aw;
        pool.bug_drop_leaf = Some(leaf);
        assert!(pool.publish(Op::Write(2, 0)));
        let (base, overlay) = full_image(&pool);
        let persisted = vec![];
        let img = Image { pool: &pool, base: &base, overlay: &overlay, persisted: &persisted };
        let rec = recover(&img).expect("恢复");
        let verdict = check(&img, &rec);
        let floor = pool.txg.saturating_sub(cfg.ring);
        let mut live_read_free = 0;
        let mut refused = 0;
        let mut live = 0;
        let lo = leaf * cfg.aw;
        for x in 0..cfg.slots {
            if cfg.akey(0, x) / cfg.aw != leaf || cfg.akey(0, x) < lo {
                continue;
            }
            if !pool.truth_free(0, x) {
                live += 1;
                match disk_point_free(&img, &rec.fields, 0, x, floor) {
                    Ok(true) => live_read_free += 1,
                    Ok(false) => {}
                    Err(_) => refused += 1,
                }
            }
        }
        println!(
            "KNOWN_BAD_DROPPED_LEAF_POINTER cfg={} dropped_leaf={leaf} checker={:?} live_slots_in_leaf={live} point_query_says_free={live_read_free} point_query_refuses={refused}",
            cfg.name,
            verdict.as_ref().map(|_| "green").map_err(|e| e.clone())
        );
        assert!(verdict.is_err(), "漏掉一片叶的指针，全量 checker 必须判红");
    }
    // 3. 写者在「一次长高两层」时漏写旧根之上那一串（缺席 = 洞）：checker 与 oracle 各判什么、读者读到什么。
    for ext in [Ext::PerFile, Ext::GlobalRadix] {
        let cfg = Cfg { name: if ext == Ext::PerFile { "S2-PerFile" } else { "S2-GlobalRadix" }, ext, ..crash_cfg("x") };
        let (mut pool, _) = pool_after(cfg, &prefixes()[2].1);
        pool.bug_skip_spine = true;
        let before = pool.files[&2].len();
        assert!(pool.publish(Op::Write(2, 5)));
        let (base, overlay) = full_image(&pool);
        let persisted = vec![];
        let img = Image { pool: &pool, base: &base, overlay: &overlay, persisted: &persisted };
        let rec = recover(&img).expect("恢复");
        let verdict = check(&img, &rec);
        let mut w = Walk { img: &img, txg: rec.txg, reach: vec![], out: Checked { recs: vec![BTreeMap::new(); DEVS as usize], ..Checked::default() } };
        let cont = img.read_unit(rec.fields.cont.slot, 2, rec.fields.cont.crc).unwrap();
        let Blob::Cont { files, .. } = cont else { unreachable!() };
        w.out.inodes = files.keys().copied().collect();
        let reader = match ext {
            Ext::PerFile => files[&2].map(|p| w.ext(2, p, p.level, 0)).transpose(),
            _ => { let p = rec.fields.top.unwrap(); w.top(p, p.level, 0, &mut BTreeSet::new()).map(Some) }
        };
        let units_read: Vec<u64> = w.out.content.keys().filter(|(i, _)| *i == 2).map(|(_, u)| *u).collect();
        println!(
            "KNOWN_BAD_SKIPPED_SPINE cfg={} file2_units_before={before} reader_walk={:?} file2_units_the_reader_finds={units_read:?} checker={:?}",
            cfg.name,
            reader.map(|_| "ok"),
            verdict.as_ref().map(|_| "green").map_err(|e| e.clone())
        );
        assert!(verdict.is_err(), "长高漏写那一串，全量 checker 必须判红");
    }
    // 4. 原型第一版的写法：矮下去两层时不释放旧根之上那一串（泄漏）。
    let cfg = crash_cfg("S2-PerFile");
    let (mut pool, _) = pool_after(cfg, &prefixes()[0].1);
    pool.bug_skip_shrink_spine = true;
    let t = run_history_from(&pool, &[Op::Write(1, 4), Op::Trunc(1, 1)], false, 64, 0);
    println!("KNOWN_BAD_SKIPPED_SHRINK_SPINE {}", t.line(cfg.name));
    for e in &t.examples {
        println!("  example: {e}");
    }
    assert!(t.check_red > 0, "矮两层不释放旧的那一串，checker 必须判 I-3.1 泄漏");
    pool.bug_skip_shrink_spine = false;
    let t = run_history_from(&pool, &[Op::Write(1, 4), Op::Trunc(1, 1)], false, 64, 0);
    println!("FIXED_SHRINK_SPINE {}", t.line(cfg.name));
    assert_eq!(t.check_red, 0);
}

fn scan_named(which: &str) {
    let cfgs = match which {
        "s1" => s1_cfgs(),
        "s2" => s2_cfgs(),
        _ => vec![],
    };
    let max_groups: usize = std::env::var("KS_MAX_GROUPS").ok().and_then(|s| s.parse().ok()).unwrap_or(13);
    let only = std::env::var("KS_CFGS").ok();
    for cfg in cfgs {
        if only.as_ref().is_some_and(|o| !o.split(',').any(|x| x == cfg.name)) {
            continue;
        }
        let started = std::time::Instant::now();
        let (t, notes) = scan(cfg, max_groups);
        println!("SCAN {} secs={}", t.line(cfg.name), started.elapsed().as_secs());
        for n in notes {
            println!("{n}");
        }
        for e in &t.examples {
            println!("  example: {e}");
        }
    }
}

#[test]
fn scan_candidates() {
    let which = std::env::var("KS_SCAN").unwrap_or_default();
    for w in which.split(',') {
        scan_named(w);
    }
}

// ---------------------------------------------------------------- 代价模型（真几何，不写字节）

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

#[derive(Default)]
struct Agg {
    n: u64,
    sum: BTreeMap<&'static str, u64>,
    max: BTreeMap<&'static str, u64>,
    over67: u64,
    over_formula: u64,
    vals_named: Vec<u64>,
}

impl Agg {
    fn add(&mut self, m: &Metrics) {
        self.n += 1;
        let commit = m.named - m.data;
        for (k, v) in [
            ("data", m.data),
            ("alloc_nodes", m.alloc_nodes),
            ("alloc_leaves", m.alloc_leaves),
            ("leaves_by_data", m.leaves_by_data),
            ("leaves_by_other", m.leaves_by_other),
            ("leaves_by_alloc_chain", m.leaves_by_alloc),
            ("ext_nodes", m.ext_nodes),
            ("commit", commit),
            ("named", m.named),
            ("formula", m.formula),
            ("rounds", m.rounds),
            ("fallback", m.fallback),
        ] {
            *self.sum.entry(k).or_default() += v;
            let e = self.max.entry(k).or_default();
            *e = (*e).max(v);
        }
        // 共享的提交内生块只在末条点名（D23 已定项 17），末条另带最后一个数据单元：末条点名项 = 提交内生块 + min(数据单元, 1)。
        let last_record = commit + m.data.min(1);
        *self.sum.entry("last_record_named").or_default() += last_record;
        let e = self.max.entry("last_record_named").or_default();
        *e = (*e).max(last_record);
        self.over67 += u64::from(last_record > 67);
        self.over_formula += u64::from(commit > m.formula);
        self.vals_named.push(last_record);
    }
    fn line(&mut self, tag: &str) -> String {
        self.vals_named.sort_unstable();
        let p99 = self.vals_named.get(self.vals_named.len() * 99 / 100).copied().unwrap_or(0);
        let mean: Vec<String> = self.sum.iter().map(|(k, v)| format!("{k}={:.2}/{}", *v as f64 / self.n.max(1) as f64, self.max[k])).collect();
        format!("COST {tag} publishes={} last_record_named_p99={p99} last_record_named_over_67={} commit_over_formula={} mean/max: {}", self.n, self.over67, self.over_formula, mean.join(" "))
    }
}

fn cost_cfg(name: &'static str, slots: u64, af: u64, ext: Ext, absence: Absence) -> Cfg {
    Cfg { name, slots, seg: 64, ring: 24, aw: 812, af, aoff: 0, aflat: false, absence, cross: Cross::LookBack, ext, ew: 144, ef: 147, tw: 147, bytes: false, aregion: 0 }
}

fn run_workload(cfg: Cfg, wl: &str) -> String {
    let mut p = Pool::new(cfg);
    let mkfs_nodes = p.disk.len();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut agg = Agg::default();
    let mut refused: Option<String> = None;
    let mut push = |p: &mut Pool, op: Op, agg: &mut Agg, count: bool| -> bool {
        if refused.is_some() {
            return false;
        }
        if !p.publish(op) {
            refused = Some(p.refusal.clone());
            return false;
        }
        if count {
            agg.add(&p.m);
        }
        true
    };
    match wl {
        "first" => {
            push(&mut p, Op::Write(1, 0), &mut agg, true);
        }
        w if w.starts_with("seq") => {
            let k: u64 = w[3..].parse().unwrap();
            let units = 32903u64; // 1 GiB ÷ 32634
            let mut u = 0;
            while u < units {
                let kk = k.min(units - u);
                if !push(&mut p, Op::Range(1, u, kk), &mut agg, true) {
                    break;
                }
                u += kk;
            }
        }
        w if w.starts_with("age") => {
            let k: u64 = w[3..].parse().unwrap();
            let (files, per) = (300u64, 100u64);
            for i in 1..=files {
                push(&mut p, Op::Range(i, 0, per), &mut agg, false);
            }
            for _ in 0..3000 {
                let i = 1 + rng.below(files);
                let start = rng.below(per - k + 1);
                push(&mut p, Op::Range(i, start, k), &mut agg, true);
            }
        }
        "manyfiles" => {
            for i in 1..=30000u64 {
                if !push(&mut p, Op::Write(i, 0), &mut agg, true) {
                    break;
                }
            }
        }
        _ => unreachable!(),
    }
    let alloc_on_disk = p.disk.keys().filter(|n| matches!(n, Nid::Alloc(..))).count();
    let ext_on_disk = p.disk.keys().filter(|n| matches!(n, Nid::Ext(..) | Nid::Top(..) | Nid::KTop(..))).count();
    let _ = push;
    format!(
        "{} mkfs_nodes={mkfs_nodes} alloc_levels={} alloc_nodes_on_disk_at_end={alloc_on_disk} ext_nodes_on_disk_at_end={ext_on_disk} opened_segments={} refused={}",
        agg.line(&format!("{} {wl}", cfg.name)),
        cfg.atop() + 1,
        p.opened.len(),
        refused.unwrap_or_else(|| "none".into())
    )
}

#[test]
fn cost_model() {
    if std::env::var("KS_COST").is_err() {
        return;
    }
    const G4: u64 = 211_968;
    const G64: u64 = 4_194_304 - 50_176;
    const T1: u64 = 67_108_864 - 50_176;
    let block = std::env::var("KS_COST").unwrap();
    let mut jobs: Vec<(Cfg, &str)> = vec![];
    if block.contains('a') {
        for (sz, slots) in [("4G", G4), ("64G", G64), ("1T", T1)] {
            for (af, tag) in [(169, "keyed169"), (188, "ptronly188")] {
                let name: &'static str = Box::leak(format!("S1-Absent-{tag}-{sz}").into_boxed_str());
                let cfg = cost_cfg(name, slots, af, Ext::PerFile, Absence::Absent);
                for wl in ["first", "seq1", "age1", "age10", "age100"] {
                    jobs.push((cfg, wl));
                }
            }
            let name: &'static str = Box::leak(format!("S1-Full-keyed169-{sz}").into_boxed_str());
            let cfg = cost_cfg(name, slots, 169, Ext::PerFile, Absence::Full);
            for wl in ["first", "age10"] {
                jobs.push((cfg, wl));
            }
        }
    }
    if block.contains('b') {
        for (ext, tag) in [(Ext::PerFile, "PerFile"), (Ext::GlobalRadix, "GlobalRadix"), (Ext::GlobalKeyed, "GlobalKeyed"), (Ext::GlobalRadixInline, "GlobalRadixInline")] {
            let name: &'static str = Box::leak(format!("S2-{tag}-4G").into_boxed_str());
            let cfg = cost_cfg(name, G4, 169, ext, Absence::Absent);
            for wl in ["first", "seq1", "seq100", "age1", "age10", "age100", "manyfiles"] {
                jobs.push((cfg, wl));
            }
        }
    }
    if block.contains('c') {
        // 叶宽取 768（= 12 个 64 槽聚簇段，段不跨叶）对 812（容量上限，段会跨叶）。
        for (aw, tag) in [(812u64, "aw812"), (768, "aw768")] {
            let name: &'static str = Box::leak(format!("S1-Absent-keyed169-{tag}-4G").into_boxed_str());
            let cfg = Cfg { aw, ..cost_cfg(name, G4, 169, Ext::PerFile, Absence::Absent) };
            for wl in ["seq1", "age1", "age10"] {
                jobs.push((cfg, wl));
            }
        }
    }
    if block.contains('d') {
        // 自己提的改法（零轮）：分配记录树的节点只落在单元区开头一小段（两片叶宽 = 1624 槽）。
        for (aregion, tag) in [(0u64, "noregion"), (1624, "region1624")] {
            for (sz, slots) in [("4G", G4), ("1T", T1)] {
                let name: &'static str = Box::leak(format!("S1-Absent-keyed169-{tag}-{sz}").into_boxed_str());
                let cfg = Cfg { aregion, ..cost_cfg(name, slots, 169, Ext::PerFile, Absence::Absent) };
                for wl in ["seq1", "age1", "age10"] {
                    jobs.push((cfg, wl));
                }
            }
        }
    }
    let out = Mutex::new(vec![]);
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..threads() {
            s.spawn(|| loop {
                let j = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if j >= jobs.len() {
                    break;
                }
                let (cfg, wl) = jobs[j];
                let line = run_workload(cfg, wl);
                out.lock().unwrap().push((j, line));
            });
        }
    });
    let mut out = out.into_inner().unwrap();
    out.sort();
    for (_, l) in out {
        println!("{l}");
    }
}
