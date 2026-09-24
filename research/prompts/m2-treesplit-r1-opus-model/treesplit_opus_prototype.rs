//! m2-treesplit-r1 云端攻方（Opus）的最小原型，只在副本上：一棵码 2 树（树 ID 11、key 宽 8）的分裂 / 收缩候选、
//! 发布写序候选、「末条装不下」候选，各自录成写流，按 D13（验证路线） 已定项 4 的段模型枚举崩溃状态，
//! 每个状态跑一个按 D23（journal 的角色与格式） 已定项 14 六条口径缩写的恢复、一个按码 2 头自描述字段判的 checker、
//! 一个多版本 oracle 与发布边界判据。
//!
//! 节点字节是真的：`singlefs_core::unit::build_index_node` 写、`parse_index_node` 解（头里树 ID、层级、key 宽、
//! key 区间、诞生代号、实例代号、出生序号都是实现的偏移）；条目格式、journal 记录、根记录是这个原型自己的简化格式，
//! 不是实现的字节（报告「限度」一节列了差在哪）。节点容量用测试开关压到 `leaf_cap` / `int_cap` 条，
//! 点名容量压到 `named_cap` 项，这样分裂与「末条装不下」在两位数写的负载上就出得来。
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use singlefs_core::address::{CheckpointTxg, InstanceGeneration, TreeIdentifier};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::pointer::BirthSequence;
use singlefs_core::unit::{build_index_node, parse_index_node, IndexNodeHeader};
use singlefs_harness::crash::closed_form_state_count;

const FSID: [u8; 16] = [7; 16];
const TREE: u64 = 11;
const KW: usize = 8;
const LEAF_W: u16 = 16;
const INT_W: u16 = 32;
const ROOT_RING: u64 = 16;
const DEVICES: u8 = 2;

// ---------------------------------------------------------------- 候选

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitPoint {
    /// 从中间切：左半 ⌈(n+1)/2⌉ 条、右半其余；左右两半都是新写的单元。
    Middle,
    /// 插入的 key 大于叶里每一条时在末尾切：左半一个字节不动（不重写），右半只装新条目（D8 已定项 6 inode 容器的写法外推）；
    /// 其余插入照 Middle。
    TailWhenAppend,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SepRule {
    /// 分隔 key = 该孩子创建时的最小 key，之后不许跟着变（D8 已定项 6 inode 树那一句照搬）。
    CreationMin,
    /// 父节点被重写时分隔 key 跟着维护：插到第一个分隔 key 之下就把它压低，借条目之后把右边那个分隔 key 改成右边新的最小 key。
    Maintained,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shrink {
    /// 不合并，只摘空节点；collapse_root：根只剩一个孩子时让孩子当根（树高降一层，孩子不重写）。
    DropEmpty { collapse_root: bool },
    /// 条目数低于 min_fill 时与兄弟合并（左吸收右），合不下就借一条（borrow = true 时）。空节点照样摘，根只剩一个孩子时降高。
    Merge { min_fill: usize, borrow: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteStep {
    /// 整条路径在一次发布里 COW 重写，分裂出来的节点与别的单元同一段（不加新写序步骤）。
    OnePublish,
    /// 同一次发布里，分裂新建的节点（右半、新根）先写、屏障、再写其余单元。
    SplitOwnBarrier,
    /// 分裂单开一次发布：先发一次只改结构、不改内容的发布（自顶向下把路径上满的节点预先切开），再发这次的内容。
    SplitOwnPublish,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LastRule {
    /// 维持拒绝：末条要点名的共享单元多于 named_cap ⇒ 整次发布拒绝，盘上不变。
    Refuse,
    /// 末条再跨记录：共享单元分摊到最后 m 条记录；恢复照今天的字面认末条——「点名了共享内生块的那一条」。
    SpreadLiteral,
    /// 末条再跨记录，末条带一个「末条」标志位（新增字段）；恢复认标志位。
    SpreadFlag,
    /// 发布拆成几次：按操作切，每一次的共享单元不多于 named_cap；单个操作就装不下的照样拒绝。
    CutPublish,
}

#[derive(Clone, Copy, Debug)]
pub struct Cfg {
    pub name: &'static str,
    pub leaf_cap: usize,
    pub int_cap: usize,
    pub split: SplitPoint,
    pub sep: SepRule,
    pub shrink: Shrink,
    pub step: WriteStep,
    pub last: LastRule,
    pub named_cap: usize,
}

// ---------------------------------------------------------------- 内存里的树

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
}

#[derive(Clone, Debug)]
pub enum Node {
    Leaf { kv: Vec<(u64, u64)>, disk: Option<Ptr>, by_split: bool },
    Int { level: u8, kids: Vec<(u64, Node)>, disk: Option<Ptr>, by_split: bool },
}

impl Node {
    fn empty_leaf() -> Node {
        Node::Leaf { kv: vec![], disk: None, by_split: false }
    }
    fn level(&self) -> u8 {
        match self {
            Node::Leaf { .. } => 0,
            Node::Int { level, .. } => *level,
        }
    }
    fn len(&self) -> usize {
        match self {
            Node::Leaf { kv, .. } => kv.len(),
            Node::Int { kids, .. } => kids.len(),
        }
    }
    fn dirty(&mut self) {
        match self {
            Node::Leaf { disk, .. } | Node::Int { disk, .. } => *disk = None,
        }
    }
    fn is_dirty(&self) -> bool {
        match self {
            Node::Leaf { disk, .. } | Node::Int { disk, .. } => disk.is_none(),
        }
    }
    fn min_key(&self) -> Option<u64> {
        match self {
            Node::Leaf { kv, .. } => kv.first().map(|e| e.0),
            Node::Int { kids, .. } => kids.first().and_then(|k| k.1.min_key()),
        }
    }
    fn max_key(&self) -> Option<u64> {
        match self {
            Node::Leaf { kv, .. } => kv.last().map(|e| e.0),
            Node::Int { kids, .. } => kids.last().and_then(|k| k.1.max_key()),
        }
    }
    fn collect(&self, out: &mut BTreeMap<u64, u64>) {
        match self {
            Node::Leaf { kv, .. } => out.extend(kv.iter().copied()),
            Node::Int { kids, .. } => kids.iter().for_each(|k| k.1.collect(out)),
        }
    }
    fn clear_split_marks(&mut self) {
        match self {
            Node::Leaf { by_split, .. } => *by_split = false,
            Node::Int { by_split, kids, .. } => {
                *by_split = false;
                kids.iter_mut().for_each(|k| k.1.clear_split_marks());
            }
        }
    }
}

// ---------------------------------------------------------------- 写路径上的结构规则

/// 按分隔 key 找孩子：最后一个分隔 key ≤ key 的那一条；key 比第一个分隔 key 还小时落到第 0 个孩子
/// （SepRule::Maintained 同时把第 0 个分隔 key 压低到 key）。
fn route(kids: &mut [(u64, Node)], key: u64, sep: SepRule, lowering: bool) -> usize {
    match kids.iter().rposition(|k| k.0 <= key) {
        Some(i) => i,
        None => {
            if lowering && sep == SepRule::Maintained {
                kids[0].0 = key;
            }
            0
        }
    }
}

fn route_ro(kids: &[(u64, Node)], key: u64) -> usize {
    kids.iter().rposition(|k| k.0 <= key).unwrap_or(0)
}

/// 插一条；节点装不下时交回 (新右半的分隔 key, 新右半)。
fn insert_rec(node: &mut Node, key: u64, value: u64, cfg: &Cfg) -> Option<(u64, Node)> {
    match node {
        Node::Leaf { kv, disk, .. } => {
            match kv.binary_search_by_key(&key, |e| e.0) {
                Ok(i) => {
                    kv[i].1 = value;
                    *disk = None;
                    None
                }
                Err(i) => {
                    if kv.len() < cfg.leaf_cap {
                        kv.insert(i, (key, value));
                        *disk = None;
                        return None;
                    }
                    if cfg.split == SplitPoint::TailWhenAppend && i == kv.len() {
                        // 末尾分裂：左半不动（disk 留着 ⇒ 不重写），右半只装新条目。
                        let right = Node::Leaf { kv: vec![(key, value)], disk: None, by_split: true };
                        return Some((key, right));
                    }
                    kv.insert(i, (key, value));
                    *disk = None;
                    let keep = kv.len().div_ceil(2);
                    let right_kv = kv.split_off(keep);
                    let sep = right_kv[0].0;
                    Some((sep, Node::Leaf { kv: right_kv, disk: None, by_split: true }))
                }
            }
        }
        Node::Int { level, kids, disk, .. } => {
            *disk = None;
            let i = route(kids, key, cfg.sep, true);
            if let Some((sep, right)) = insert_rec(&mut kids[i].1, key, value, cfg) {
                kids.insert(i + 1, (sep, right));
            }
            if kids.len() <= cfg.int_cap {
                return None;
            }
            let keep = kids.len().div_ceil(2);
            let right_kids = kids.split_off(keep);
            let sep = right_kids[0].0;
            Some((sep, Node::Int { level: *level, kids: right_kids, disk: None, by_split: true }))
        }
    }
}

pub fn insert(root: &mut Node, key: u64, value: u64, cfg: &Cfg) {
    if let Some((sep, right)) = insert_rec(root, key, value, cfg) {
        let left = std::mem::replace(root, Node::empty_leaf());
        let left_sep = left.min_key().expect("刚装满过的节点不空");
        let level = left.level() + 1;
        *root = Node::Int { level, kids: vec![(left_sep, left), (sep, right)], disk: None, by_split: true };
    }
}

/// 父节点里第 i 个孩子变少之后的收缩：空的摘掉；Merge 时低于 min_fill 就与兄弟合并或借。
fn shrink_child(kids: &mut Vec<(u64, Node)>, i: usize, cfg: &Cfg) {
    if kids[i].1.len() == 0 {
        kids.remove(i);
        return;
    }
    let Shrink::Merge { min_fill, borrow } = cfg.shrink else { return };
    if kids[i].1.len() >= min_fill || kids.len() < 2 {
        return;
    }
    let cap = if kids[i].1.level() == 0 { cfg.leaf_cap } else { cfg.int_cap };
    // 配对：有右兄弟就 (i, i+1)，否则 (i-1, i)。合并一律左吸收右。
    let (l, r) = if i + 1 < kids.len() { (i, i + 1) } else { (i - 1, i) };
    if kids[l].1.len() + kids[r].1.len() <= cap {
        let (_, right) = kids.remove(r);
        let left = &mut kids[l].1;
        left.dirty();
        match (left, right) {
            (Node::Leaf { kv, .. }, Node::Leaf { kv: rkv, .. }) => kv.extend(rkv),
            (Node::Int { kids: lk, .. }, Node::Int { kids: rk, .. }) => lk.extend(rk),
            _ => unreachable!("同层"),
        }
        return;
    }
    if !borrow {
        return;
    }
    // 借一条：欠的是左边就从右边第一条借给左边（左从右借），欠的是右边就从左边最后一条借给右边。
    let (left_part, right_part) = kids.split_at_mut(r);
    let left = &mut left_part[l].1;
    let (right_sep, right) = (&mut right_part[0].0, &mut right_part[0].1);
    left.dirty();
    right.dirty();
    let underfull_is_left = l == i;
    match (left, right) {
        (Node::Leaf { kv, .. }, Node::Leaf { kv: rkv, .. }) => {
            if underfull_is_left {
                kv.push(rkv.remove(0));
            } else {
                rkv.insert(0, kv.pop().unwrap());
            }
            if cfg.sep == SepRule::Maintained {
                *right_sep = rkv[0].0;
            }
        }
        (Node::Int { kids: lk, .. }, Node::Int { kids: rk, .. }) => {
            if underfull_is_left {
                lk.push(rk.remove(0));
            } else {
                rk.insert(0, lk.pop().unwrap());
            }
            if cfg.sep == SepRule::Maintained {
                *right_sep = rk[0].0;
            }
        }
        _ => unreachable!("同层"),
    }
}

fn delete_rec(node: &mut Node, key: u64, cfg: &Cfg) -> bool {
    match node {
        Node::Leaf { kv, disk, .. } => match kv.binary_search_by_key(&key, |e| e.0) {
            Ok(i) => {
                kv.remove(i);
                *disk = None;
                true
            }
            Err(_) => false,
        },
        Node::Int { kids, disk, .. } => {
            let i = route_ro(kids, key);
            if !delete_rec(&mut kids[i].1, key, cfg) {
                return false;
            }
            *disk = None;
            shrink_child(kids, i, cfg);
            true
        }
    }
}

pub fn delete(root: &mut Node, key: u64, cfg: &Cfg) {
    if !delete_rec(root, key, cfg) {
        return;
    }
    loop {
        let collapse = match cfg.shrink {
            Shrink::DropEmpty { collapse_root } => collapse_root,
            Shrink::Merge { .. } => true,
        };
        match root {
            Node::Int { kids, .. } if kids.is_empty() => {
                *root = Node::empty_leaf();
            }
            Node::Int { kids, .. } if kids.len() == 1 && collapse => {
                let (_, only) = kids.pop().unwrap();
                *root = only;
                continue;
            }
            _ => {}
        }
        break;
    }
}

/// 把 key 这条路径上装满的节点预先切开（SplitOwnPublish 那次结构发布用）：只动结构、不动内容。
/// 做完之后这条路径上每个节点都还有空位，随后那次内容发布插 key 不再分裂。
pub fn presplit_path(root: &mut Node, key: u64, cfg: &Cfg) -> bool {
    let (changed, split) = presplit_rec(root, key, cfg);
    if let Some((sep, right)) = split {
        let left = std::mem::replace(root, Node::empty_leaf());
        let left_sep = left.min_key().expect("切过的节点不空");
        let level = left.level() + 1;
        *root = Node::Int { level, kids: vec![(left_sep, left), (sep, right)], disk: None, by_split: true };
    }
    changed
}

fn presplit_rec(node: &mut Node, key: u64, cfg: &Cfg) -> (bool, Option<(u64, Node)>) {
    let mut changed = false;
    if let Node::Int { kids, disk, .. } = node {
        let i = route_ro(kids, key);
        let (ch, sp) = presplit_rec(&mut kids[i].1, key, cfg);
        if let Some((sep, right)) = sp {
            kids.insert(i + 1, (sep, right));
        }
        if ch {
            *disk = None;
            changed = true;
        }
    }
    let full = match node {
        Node::Leaf { kv, .. } => kv.len() >= cfg.leaf_cap && kv.binary_search_by_key(&key, |e| e.0).is_err(),
        Node::Int { kids, .. } => kids.len() >= cfg.int_cap,
    };
    if full && node.len() >= 2 {
        node.dirty();
        let (sep, right) = split_node_in_half(node);
        return (true, Some((sep, right)));
    }
    (changed, None)
}

fn split_node_in_half(n: &mut Node) -> (u64, Node) {
    match n {
        Node::Leaf { kv, .. } => {
            let keep = kv.len().div_ceil(2);
            let r = kv.split_off(keep);
            (r[0].0, Node::Leaf { kv: r, disk: None, by_split: true })
        }
        Node::Int { level, kids, .. } => {
            let keep = kids.len().div_ceil(2);
            let r = kids.split_off(keep);
            (r[0].0, Node::Int { level: *level, kids: r, disk: None, by_split: true })
        }
    }
}

// ---------------------------------------------------------------- 写流

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Loc {
    Unit(u64),
    Rec(u64),
    Root(u64),
}

#[derive(Clone, Debug)]
pub struct Rec {
    pub jsn: u64,
    pub txg: u64,
    /// 本次发布内序号，从 1 起。
    pub seq: u32,
    /// 「末条」标志位：只有 LastRule::SpreadFlag 写它。
    pub last_flag: bool,
    pub root: Ptr,
    /// 点名项：(槽号, 整单元 CRC)。
    pub named: Vec<(u64, u32)>,
}

#[derive(Clone, Debug)]
pub struct RootRec {
    pub txg: u64,
    pub root: Ptr,
    pub last_jsn: u64,
}

#[derive(Clone, Debug)]
pub enum Blob {
    Unit { crc: u32, header: IndexNodeHeader, by_split: bool },
    Rec(Rec),
    Root(RootRec),
}

#[derive(Clone, Copy, Debug)]
pub struct Write {
    pub dev: u8,
    pub loc: Loc,
    pub blob: usize,
    pub fua: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum StreamOp {
    W(Write),
    Barrier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Ins(u64),
    Del(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishKind {
    Content,
    Restructure,
}

#[derive(Clone, Debug)]
pub struct Version {
    pub txg: u64,
    pub content: BTreeMap<u64, u64>,
    pub root: Ptr,
    pub kind: PublishKind,
    /// 这次发布每条记录（按 jsn）两份写在写流里的下标。
    pub record_writes: Vec<Vec<usize>>,
    /// 点了名的记录（按 jsn 在这次发布里的位置）。
    pub named_record_positions: Vec<usize>,
    pub root_write: usize,
    pub units_written: usize,
    pub split_units: usize,
    pub segments_added: usize,
}

#[derive(Clone)]
pub struct Pool {
    pub cfg: Cfg,
    pub tree: Node,
    pub txg: u64,
    pub jsn: u64,
    pub next_slot: u64,
    pub stream: Vec<StreamOp>,
    pub writes: Vec<Write>,
    pub blobs: Vec<Blob>,
    pub versions: Vec<Version>,
    pub refused: Vec<(u64, Vec<Op>, usize)>,
    next_value: u64,
}

impl Pool {
    pub fn new(cfg: Cfg) -> Pool {
        let mut p = Pool {
            cfg,
            tree: Node::empty_leaf(),
            txg: 0,
            jsn: 0,
            next_slot: 100,
            stream: vec![],
            writes: vec![],
            blobs: vec![],
            versions: vec![],
            refused: vec![],
            next_value: 1,
        };
        // mkfs：txg 0 的空根兼叶与第 0 代根（不经记录）。
        let mut units = vec![];
        let root = build_tree(&mut p.tree, 0, &mut p.next_slot, &mut units);
        for (slot, bytes, by_split) in units {
            let crc = crc32_castagnoli(&bytes);
            let header = parse_index_node(&bytes).expect("刚写的码 2 节点解得开");
            let blob = p.push_blob(Blob::Unit { crc, header, by_split });
            for dev in 0..DEVICES {
                p.push_write(Write { dev, loc: Loc::Unit(slot), blob, fua: false });
            }
        }
        p.stream.push(StreamOp::Barrier);
        let blob = p.push_blob(Blob::Root(RootRec { txg: 0, root, last_jsn: 0 }));
        let w = p.push_write(Write { dev: 0, loc: Loc::Root(0), blob, fua: true });
        p.versions.push(Version {
            txg: 0,
            content: BTreeMap::new(),
            root,
            kind: PublishKind::Content,
            record_writes: vec![],
            named_record_positions: vec![],
            root_write: w,
            units_written: 1,
            split_units: 0,
            segments_added: 0,
        });
        p
    }

    fn push_blob(&mut self, b: Blob) -> usize {
        self.blobs.push(b);
        self.blobs.len() - 1
    }
    fn push_write(&mut self, w: Write) -> usize {
        self.writes.push(w);
        self.stream.push(StreamOp::W(w));
        self.writes.len() - 1
    }

    fn fresh_value(&mut self) -> u64 {
        self.next_value += 1;
        self.next_value
    }

    fn apply_ops(&mut self, tree: &mut Node, ops: &[Op]) {
        for op in ops {
            match *op {
                Op::Ins(k) => {
                    let v = self.fresh_value();
                    insert(tree, k, v, &self.cfg)
                }
                Op::Del(k) => delete(tree, k, &self.cfg),
            }
        }
    }

    /// 用户的一次写请求（一批操作）：按候选决定发几次、拒不拒。
    pub fn user_publish(&mut self, ops: &[Op]) {
        let cfg = self.cfg;
        if cfg.step == WriteStep::SplitOwnPublish {
            for op in ops {
                if let Op::Ins(k) = *op {
                    let mut t = self.tree.clone();
                    if presplit_path(&mut t, k, &cfg) {
                        let shared = count_dirty(&t);
                        if shared > cfg.named_cap && cfg.last == LastRule::Refuse {
                            self.refused.push((self.txg + 1, vec![], shared));
                            return;
                        }
                        self.tree = t;
                        self.emit(0, PublishKind::Restructure);
                    }
                }
            }
        }
        match cfg.last {
            LastRule::CutPublish => {
                let mut batch: Vec<Op> = vec![];
                for op in ops {
                    let mut t = self.tree.clone();
                    let mut with = batch.clone();
                    with.push(*op);
                    self.apply_ops(&mut t, &with);
                    if count_dirty(&t) <= cfg.named_cap {
                        batch = with;
                        continue;
                    }
                    if !batch.is_empty() {
                        let mut t = self.tree.clone();
                        self.apply_ops(&mut t, &batch);
                        self.tree = t;
                        self.emit(batch.len(), PublishKind::Content);
                    }
                    let mut t = self.tree.clone();
                    self.apply_ops(&mut t, &[*op]);
                    let shared = count_dirty(&t);
                    if shared > cfg.named_cap {
                        self.refused.push((self.txg + 1, vec![*op], shared));
                        batch = vec![];
                    } else {
                        batch = vec![*op];
                    }
                }
                if !batch.is_empty() {
                    let mut t = self.tree.clone();
                    self.apply_ops(&mut t, &batch);
                    self.tree = t;
                    self.emit(batch.len(), PublishKind::Content);
                }
            }
            _ => {
                let mut t = self.tree.clone();
                self.apply_ops(&mut t, ops);
                let shared = count_dirty(&t);
                if cfg.last == LastRule::Refuse && shared > cfg.named_cap {
                    self.refused.push((self.txg + 1, ops.to_vec(), shared));
                    return;
                }
                self.tree = t;
                self.emit(ops.len().max(1), PublishKind::Content);
            }
        }
    }

    /// 发一次：这次重写的单元（bump 次序：先叶后根、同层按 key 升序）→ 屏障 → 记录 → 屏障 → 根槽 FUA。
    fn emit(&mut self, op_records: usize, kind: PublishKind) {
        let cfg = self.cfg;
        self.txg += 1;
        let txg = self.txg;
        let segments_before = self.stream.iter().filter(|s| matches!(s, StreamOp::Barrier)).count();
        let mut units = vec![];
        let mut tree = std::mem::replace(&mut self.tree, Node::empty_leaf());
        let root = build_tree(&mut tree, txg, &mut self.next_slot, &mut units);
        tree.clear_split_marks();
        self.tree = tree;
        let split_units = units.iter().filter(|u| u.2).count();
        let mut unit_blobs = vec![];
        for (slot, bytes, by_split) in &units {
            let crc = crc32_castagnoli(bytes);
            let header = parse_index_node(bytes).expect("刚写的码 2 节点解得开");
            let blob = self.push_blob(Blob::Unit { crc, header, by_split: *by_split });
            unit_blobs.push((*slot, crc, blob, *by_split));
        }
        let mut write_units = |p: &mut Pool, only_split: Option<bool>| {
            for (slot, _, blob, by_split) in &unit_blobs {
                if only_split.is_some_and(|want| want != *by_split) {
                    continue;
                }
                for dev in 0..DEVICES {
                    p.push_write(Write { dev, loc: Loc::Unit(*slot), blob: *blob, fua: false });
                }
            }
        };
        if cfg.step == WriteStep::SplitOwnBarrier && split_units > 0 {
            write_units(self, Some(true));
            self.stream.push(StreamOp::Barrier);
            write_units(self, Some(false));
        } else {
            write_units(self, None);
        }
        self.stream.push(StreamOp::Barrier);
        // 点名：共享单元（这里就是这次重写的全部节点）按 bump 次序点名。
        let shared: Vec<(u64, u32)> = unit_blobs.iter().map(|u| (u.0, u.1)).collect();
        let chunks: Vec<Vec<(u64, u32)>> = match cfg.last {
            LastRule::SpreadLiteral | LastRule::SpreadFlag => {
                shared.chunks(cfg.named_cap.max(1)).map(<[_]>::to_vec).collect()
            }
            _ => {
                assert!(shared.len() <= cfg.named_cap, "拒绝与拆发布两条臂在 emit 之前判过");
                vec![shared.clone()]
            }
        };
        let chunks = if chunks.is_empty() { vec![vec![]] } else { chunks };
        let total = op_records.max(chunks.len()).max(1);
        let mut record_writes = vec![];
        let mut named_positions = vec![];
        let first_named = total - chunks.len();
        for position in 0..total {
            self.jsn += 1;
            let named = if position >= first_named { chunks[position - first_named].clone() } else { vec![] };
            if !named.is_empty() {
                named_positions.push(position);
            }
            let rec = Rec {
                jsn: self.jsn,
                txg,
                seq: u32::try_from(position + 1).unwrap(),
                last_flag: cfg.last == LastRule::SpreadFlag && position + 1 == total,
                root,
                named,
            };
            let blob = self.push_blob(Blob::Rec(rec));
            let mut copies = vec![];
            for dev in 0..DEVICES {
                copies.push(self.push_write(Write { dev, loc: Loc::Rec(self.jsn), blob, fua: false }));
            }
            record_writes.push(copies);
        }
        self.stream.push(StreamOp::Barrier);
        let blob = self.push_blob(Blob::Root(RootRec { txg, root, last_jsn: self.jsn }));
        let dev = u8::try_from(txg % u64::from(DEVICES)).unwrap();
        let root_write = self.push_write(Write { dev, loc: Loc::Root(txg % ROOT_RING), blob, fua: true });
        let mut content = BTreeMap::new();
        self.tree.collect(&mut content);
        let segments_after = self.stream.iter().filter(|s| matches!(s, StreamOp::Barrier)).count();
        self.versions.push(Version {
            txg,
            content,
            root,
            kind,
            record_writes,
            named_record_positions: named_positions,
            root_write,
            units_written: units.len(),
            split_units,
            segments_added: segments_after - segments_before,
        });
    }
}

pub fn count_dirty(n: &Node) -> usize {
    let own = usize::from(n.is_dirty());
    match n {
        Node::Leaf { .. } => own,
        Node::Int { kids, .. } => own + kids.iter().map(|k| count_dirty(&k.1)).sum::<usize>(),
    }
}

fn collect_dirty(n: &Node, path: &mut Vec<usize>, out: &mut Vec<(u8, u64, Vec<usize>)>) {
    if let Node::Int { kids, .. } = n {
        for (i, k) in kids.iter().enumerate() {
            path.push(i);
            collect_dirty(&k.1, path, out);
            path.pop();
        }
    }
    if n.is_dirty() {
        out.push((n.level(), n.min_key().unwrap_or(0), path.clone()));
    }
}

/// 把脏节点按 bump 次序（层级升序、同层 key 升序）发槽号与出生序号，再自底向上装字节；交回根指针。
pub fn build_tree(root: &mut Node, txg: u64, next_slot: &mut u64, units: &mut Vec<(u64, Vec<u8>, bool)>) -> Ptr {
    let mut dirty = vec![];
    collect_dirty(root, &mut vec![], &mut dirty);
    dirty.sort();
    let mut plan: HashMap<Vec<usize>, (u64, u32)> = HashMap::new();
    for (sequence, (_, _, path)) in dirty.iter().enumerate() {
        plan.insert(path.clone(), (*next_slot, u32::try_from(sequence).unwrap()));
        *next_slot += 1;
    }
    let mut built: BTreeMap<u64, (Vec<u8>, bool)> = BTreeMap::new();
    let ptr = build_rec(root, &mut vec![], &plan, txg, &mut built);
    // 写出次序就是槽号次序（= bump 次序）。
    for (slot, (bytes, by_split)) in built {
        units.push((slot, bytes, by_split));
    }
    ptr
}

fn build_rec(
    n: &mut Node,
    path: &mut Vec<usize>,
    plan: &HashMap<Vec<usize>, (u64, u32)>,
    txg: u64,
    built: &mut BTreeMap<u64, (Vec<u8>, bool)>,
) -> Ptr {
    let child_ptrs: Vec<(u64, Ptr)> = match n {
        Node::Leaf { .. } => vec![],
        Node::Int { kids, .. } => kids
            .iter_mut()
            .enumerate()
            .map(|(i, k)| {
                path.push(i);
                let p = build_rec(&mut k.1, path, plan, txg, built);
                path.pop();
                (k.0, p)
            })
            .collect(),
    };
    if let Node::Leaf { disk: Some(p), .. } | Node::Int { disk: Some(p), .. } = n {
        return *p;
    }
    let (slot, sequence) = plan[path];
    let level = n.level();
    let (lo, hi) = (n.min_key().unwrap_or(0), n.max_key().unwrap_or(0));
    let (width, entries): (u16, Vec<Vec<u8>>) = match n {
        Node::Leaf { kv, .. } => (
            LEAF_W,
            kv.iter()
                .map(|(k, v)| [k.to_le_bytes(), v.to_le_bytes()].concat())
                .collect(),
        ),
        Node::Int { .. } => (
            INT_W,
            child_ptrs
                .iter()
                .map(|(sep, p)| {
                    let mut e = sep.to_le_bytes().to_vec();
                    e.extend(p.bytes());
                    e.resize(usize::from(INT_W), 0);
                    e
                })
                .collect(),
        ),
    };
    let bytes = build_index_node(
        TreeIdentifier(TREE),
        level,
        KW,
        &lo.to_le_bytes(),
        &hi.to_le_bytes(),
        CheckpointTxg(txg),
        &FSID,
        InstanceGeneration(1),
        BirthSequence(sequence),
        width,
        &entries,
    );
    let ptr = Ptr { slot, crc: crc32_castagnoli(&bytes), level, birth: txg };
    let by_split = matches!(n, Node::Leaf { by_split: true, .. } | Node::Int { by_split: true, .. });
    built.insert(slot, (bytes, by_split));
    match n {
        Node::Leaf { disk, .. } | Node::Int { disk, .. } => *disk = Some(ptr),
    }
    ptr
}

// ---------------------------------------------------------------- 崩溃镜像、恢复、checker、oracle

pub struct Image<'a> {
    pub pool: &'a Pool,
    /// 前缀（不枚举、全部持久）里每个位置最后一次写的 blob。
    pub base: &'a HashMap<(u8, Loc), usize>,
    /// 枚举段里每个位置的写（下标进 pool.writes）；同一位置只写一次（断言过）。
    pub overlay: &'a HashMap<(u8, Loc), usize>,
    pub persisted: &'a [bool],
    pub first_enumerated_write: usize,
    /// 介质故障叠加：这些 blob 在两块盘上都读不出。
    pub unreadable: &'a BTreeSet<usize>,
}

impl Image<'_> {
    fn read(&self, dev: u8, loc: Loc) -> Option<usize> {
        let blob = match self.overlay.get(&(dev, loc)) {
            Some(w) if self.persisted[*w - self.first_enumerated_write] => Some(self.pool.writes[*w].blob),
            _ => self.base.get(&(dev, loc)).copied(),
        }?;
        if self.unreadable.contains(&blob) {
            return None;
        }
        Some(blob)
    }
    fn read_unit(&self, slot: u64, crc: u32) -> Option<&IndexNodeHeader> {
        for dev in 0..DEVICES {
            if let Some(b) = self.read(dev, Loc::Unit(slot)) {
                if let Blob::Unit { crc: c, header, .. } = &self.pool.blobs[b] {
                    if *c == crc {
                        return Some(header);
                    }
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct Recovered {
    pub txg: u64,
    pub root: Ptr,
    pub chosen_root_txg: u64,
}

pub fn recover(img: &Image<'_>) -> Result<Recovered, String> {
    let mut best: Option<&RootRec> = None;
    for dev in 0..DEVICES {
        for s in 0..ROOT_RING {
            if let Some(b) = img.read(dev, Loc::Root(s)) {
                if let Blob::Root(r) = &img.pool.blobs[b] {
                    if best.is_none_or(|x| r.txg > x.txg) {
                        best = Some(r);
                    }
                }
            }
        }
    }
    let root = best.ok_or("根环里没有一条自证合法的根")?;
    let mut cur = Recovered { txg: root.txg, root: root.root, chosen_root_txg: root.txg };
    // 链：从所选根覆盖的最后一条之后接，jsn 严格连续、txg 大于根。
    let mut chain: Vec<&Rec> = vec![];
    let mut j = root.last_jsn + 1;
    loop {
        let rec = (0..DEVICES).find_map(|dev| match img.read(dev, Loc::Rec(j)).map(|b| &img.pool.blobs[b]) {
            Some(Blob::Rec(r)) => Some(r),
            _ => None,
        });
        match rec {
            Some(r) if r.txg > root.txg => {
                chain.push(r);
                j += 1;
            }
            _ => break,
        }
    }
    let mut i = 0;
    while i < chain.len() {
        let txg = chain[i].txg;
        let group: Vec<&Rec> = chain[i..].iter().take_while(|r| r.txg == txg).copied().collect();
        i += group.len();
        if group[0].seq != 1 {
            break;
        }
        let last = match img.pool.cfg.last {
            LastRule::SpreadFlag => group.iter().position(|r| r.last_flag),
            // 今天的字面：末条 = 点名了这次发布共享的提交内生块的那一条（D23 已定项 14 第六条「发布边界怎么认」）。
            _ => group.iter().position(|r| !r.named.is_empty()),
        };
        let Some(last) = last else { break };
        let verified = group[..=last]
            .iter()
            .flat_map(|r| r.named.iter())
            .all(|(slot, crc)| img.read_unit(*slot, *crc).is_some());
        if !verified {
            break;
        }
        cur = Recovered { txg, root: group[last].root, chosen_root_txg: root.txg };
    }
    Ok(cur)
}

/// checker：从恢复出来的根往下走，每一步只拿指针与子节点码 2 头里自带的身份字段判。
pub fn check(img: &Image<'_>, rec: &Recovered) -> Result<BTreeMap<u64, u64>, String> {
    let mut content = BTreeMap::new();
    walk(img, rec.root, rec.txg, true, &mut content)?;
    Ok(content)
}

fn walk(img: &Image<'_>, p: Ptr, root_txg: u64, is_root: bool, out: &mut BTreeMap<u64, u64>) -> Result<(u64, u64, bool), String> {
    let cfg = img.pool.cfg;
    let h = img.read_unit(p.slot, p.crc).ok_or_else(|| format!("指针指向的槽 {} 读不出或 CRC 不符", p.slot))?;
    if h.tree.0 != TREE {
        return Err(format!("槽 {} 树 ID {} ≠ {TREE}", p.slot, h.tree.0));
    }
    if h.level != p.level {
        return Err(format!("槽 {} 层级 {} ≠ 父指针说的 {}", p.slot, h.level, p.level));
    }
    if h.birth_txg.0 != p.birth || h.birth_txg.0 > root_txg {
        return Err(format!("槽 {} 诞生代号 {} 与指针 {} / 根 {root_txg} 不符", p.slot, h.birth_txg.0, p.birth));
    }
    let lo_h = u64::from_le_bytes(h.smallest_key[..8].try_into().unwrap());
    let hi_h = u64::from_le_bytes(h.largest_key[..8].try_into().unwrap());
    let key = |e: &[u8]| u64::from_le_bytes(e[..8].try_into().unwrap());
    let keys: Vec<u64> = h.entries.iter().map(|e| key(e)).collect();
    if keys.windows(2).any(|w| w[0] >= w[1]) {
        return Err(format!("槽 {} 条目 key 不严格递增", p.slot));
    }
    let (lo, hi, empty) = if h.level == 0 {
        if h.entries.len() > cfg.leaf_cap {
            return Err(format!("槽 {} 叶条目数 {} 超容量", p.slot, h.entries.len()));
        }
        if h.entries.is_empty() && !is_root {
            return Err(format!("槽 {} 非根的空叶", p.slot));
        }
        for e in &h.entries {
            out.insert(key(e), u64::from_le_bytes(e[8..16].try_into().unwrap()));
        }
        (keys.first().copied().unwrap_or(0), keys.last().copied().unwrap_or(0), h.entries.is_empty())
    } else {
        if h.entries.len() > cfg.int_cap || h.entries.is_empty() {
            return Err(format!("槽 {} 内部条目数 {} 不在 [1, {}]", p.slot, h.entries.len(), cfg.int_cap));
        }
        let mut lo = u64::MAX;
        let mut hi = 0;
        let mut previous_max: Option<u64> = None;
        for e in &h.entries {
            let sep = key(e);
            let child = Ptr::parse(&e[8..29]);
            if child.level + 1 != h.level {
                return Err(format!("槽 {} 的孩子层级 {} 不是 {}", p.slot, child.level, h.level - 1));
            }
            let (clo, chi, cempty) = walk(img, child, root_txg, false, out)?;
            if cempty {
                return Err(format!("槽 {} 挂着空孩子", p.slot));
            }
            // I-9.12 的泛化：分隔 key ≤ 孩子区间下界，且 > 左邻孩子区间上界。
            if sep > clo {
                return Err(format!("槽 {} 分隔 key {sep} > 孩子区间下界 {clo}", p.slot));
            }
            if let Some(pm) = previous_max {
                if sep <= pm {
                    return Err(format!("槽 {} 分隔 key {sep} ≤ 左邻孩子区间上界 {pm}", p.slot));
                }
            }
            previous_max = Some(chi);
            lo = lo.min(clo);
            hi = hi.max(chi);
        }
        (lo, hi, false)
    };
    // D18 已定项 2：头里的 key 区间 = 子树覆盖区间。
    if (lo_h, hi_h) != (lo, hi) {
        return Err(format!("槽 {} 头区间 [{lo_h}, {hi_h}] ≠ 子树覆盖 [{lo}, {hi}]", p.slot));
    }
    Ok((lo, hi, empty))
}

/// 查找路径：按分隔 key 从根走到叶，看每个 key 是否找得到（checker 走的是全树，这里走的是读者的路由）。
pub fn lookup_misses(img: &Image<'_>, rec: &Recovered, content: &BTreeMap<u64, u64>) -> usize {
    let mut misses = 0;
    for (k, v) in content {
        let mut p = rec.root;
        let found = loop {
            let Some(h) = img.read_unit(p.slot, p.crc) else { break None };
            let key = |e: &[u8]| u64::from_le_bytes(e[..8].try_into().unwrap());
            if h.level == 0 {
                break h.entries.iter().find(|e| key(e) == *k).map(|e| u64::from_le_bytes(e[8..16].try_into().unwrap()));
            }
            // D8 已定项 6 的查找口径：取最后一个分隔 key ≤ 目标 key 的条目；一个都没有就找不到。
            let Some(i) = h.entries.iter().rposition(|e| key(e) <= *k) else { break None };
            p = Ptr::parse(&h.entries[i][8..29]);
        };
        if found != Some(*v) {
            misses += 1;
        }
    }
    misses
}

// ---------------------------------------------------------------- 枚举

#[derive(Default, Debug, Clone)]
pub struct Tally {
    pub histories: u64,
    pub states: u64,
    pub closed_form: u64,
    pub recover_fail: u64,
    pub check_red: u64,
    pub content_mismatch: u64,
    pub root_lost: u64,
    pub cut_publish_applied: u64,
    pub unverified_applied: u64,
    pub lookup_miss: u64,
    pub refused_publishes: u64,
    pub refused_histories: u64,
    pub publishes: u64,
    pub restructure_publishes: u64,
    pub units: u64,
    pub split_units: u64,
    pub ev_split: u64,
    pub ev_height_up: u64,
    pub ev_height_down: u64,
    pub ev_node_removed: u64,
    pub max_segment_writes: u64,
    pub overlay_states: u64,
    pub overlay_red: u64,
    pub examples: Vec<String>,
}

impl Tally {
    pub fn add(&mut self, o: &Tally) {
        macro_rules! s { ($($f:ident),*) => { $( self.$f += o.$f; )* } }
        s!(histories, states, closed_form, recover_fail, check_red, content_mismatch, root_lost, cut_publish_applied,
           unverified_applied, lookup_miss, refused_publishes, refused_histories, publishes, restructure_publishes, units,
           split_units, ev_split, ev_height_up, ev_height_down, ev_node_removed, overlay_states, overlay_red);
        self.max_segment_writes = self.max_segment_writes.max(o.max_segment_writes);
        for e in &o.examples {
            if self.examples.len() < 6 {
                self.examples.push(e.clone());
            }
        }
    }
    pub fn line(&self, name: &str) -> String {
        format!(
            "CFG {name} histories={} states={} closed_form={} recover_fail={} check_red={} content_mismatch={} root_lost={} \
             cut_publish_applied={} unverified_applied={} lookup_miss={} refused_publishes={} refused_histories={} publishes={} \
             restructure_publishes={} units={} split_units={} ev_split={} ev_height_up={} ev_height_down={} ev_node_removed={} \
             max_segment_writes={} overlay_states={} overlay_red={}",
            self.histories, self.states, self.closed_form, self.recover_fail, self.check_red, self.content_mismatch,
            self.root_lost, self.cut_publish_applied, self.unverified_applied, self.lookup_miss, self.refused_publishes,
            self.refused_histories, self.publishes, self.restructure_publishes, self.units, self.split_units, self.ev_split,
            self.ev_height_up, self.ev_height_down, self.ev_node_removed, self.max_segment_writes, self.overlay_states,
            self.overlay_red
        )
    }
}

fn height(n: &Node) -> u8 {
    n.level()
}
fn node_count(n: &Node) -> usize {
    match n {
        Node::Leaf { .. } => 1,
        Node::Int { kids, .. } => 1 + kids.iter().map(|k| node_count(&k.1)).sum::<usize>(),
    }
}

/// 跑一段历史：前缀（全部持久、不枚举）+ 要枚举的几次用户发布；枚举后缀录下的写流的全部崩溃状态。
pub fn run_history(cfg: Cfg, prefix: &[Vec<Op>], suffix: &[Vec<Op>], overlay: bool) -> Tally {
    run_history_from(&pool_after(cfg, prefix), prefix, suffix, overlay)
}

pub fn pool_after(cfg: Cfg, prefix: &[Vec<Op>]) -> Pool {
    let mut pool = Pool::new(cfg);
    for ops in prefix {
        pool.user_publish(ops);
    }
    pool
}

/// 同上，前缀之后的池由调用方给（扫描时每个前缀只建一次、每段历史拷一份）。
pub fn run_history_from(pool_after_prefix: &Pool, prefix: &[Vec<Op>], suffix: &[Vec<Op>], overlay: bool) -> Tally {
    let mut pool = pool_after_prefix.clone();
    let mut t = Tally { histories: 1, ..Tally::default() };
    // 前缀里被拒的不算：前缀是装置造的起点，不是被判的历史。
    pool.refused.clear();
    let stream_start = pool.stream.len();
    let write_start = pool.writes.len();
    let versions_start = pool.versions.len();
    for ops in suffix {
        let (h0, n0) = (height(&pool.tree), node_count(&pool.tree));
        let v0 = pool.versions.len();
        pool.user_publish(ops);
        let (h1, n1) = (height(&pool.tree), node_count(&pool.tree));
        if pool.versions[v0..].iter().any(|v| v.split_units > 0) {
            t.ev_split += 1;
        }
        t.ev_height_up += u64::from(h1 > h0);
        t.ev_height_down += u64::from(h1 < h0);
        t.ev_node_removed += u64::from(n1 < n0 && pool.versions[v0..].iter().all(|v| v.split_units == 0));
    }
    t.refused_publishes = pool.refused.len() as u64;
    t.refused_histories = u64::from(!pool.refused.is_empty());
    for v in &pool.versions[versions_start..] {
        t.publishes += 1;
        t.restructure_publishes += u64::from(v.kind == PublishKind::Restructure);
        t.units += v.units_written as u64;
        t.split_units += v.split_units as u64;
    }
    let mut base: HashMap<(u8, Loc), usize> = HashMap::new();
    for w in &pool.writes[..write_start] {
        base.insert((w.dev, w.loc), w.blob);
    }
    let mut over: HashMap<(u8, Loc), usize> = HashMap::new();
    for (i, w) in pool.writes.iter().enumerate().skip(write_start) {
        assert!(over.insert((w.dev, w.loc), i).is_none(), "枚举段里同一位置只写一次（槽不复用、环不回绕）");
    }
    // 切段：屏障切，FUA 写自成一段的末尾。
    let mut segments: Vec<Vec<usize>> = vec![];
    let mut cur = vec![];
    let mut wi = write_start;
    for op in &pool.stream[stream_start..] {
        match op {
            StreamOp::Barrier => {
                if !cur.is_empty() {
                    segments.push(std::mem::take(&mut cur));
                }
            }
            StreamOp::W(w) => {
                cur.push(wi - write_start);
                wi += 1;
                if w.fua {
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
    let n = pool.writes.len() - write_start;
    let none = BTreeSet::new();
    let suffix_versions: Vec<&Version> = pool.versions[versions_start..].iter().collect();
    let prefix_newest = pool.versions[..versions_start].last().map_or(0, |v| v.txg);
    let mut persisted = vec![false; n];
    let mut eval = |persisted: &[bool], t: &mut Tally, unreadable: &BTreeSet<usize>, overlay_for: Option<&Version>| {
        let img = Image { pool: &pool, base: &base, overlay: &over, persisted, first_enumerated_write: write_start, unreadable };
        let p = |w: usize| persisted[w - write_start];
        if let Some(v) = overlay_for {
            if p(v.root_write) {
                return;
            }
            t.overlay_states += 1;
        } else {
            t.states += 1;
        }
        let describe = |what: &str, detail: &str| {
            let persisted_list: Vec<String> = (0..n)
                .filter(|i| persisted[*i])
                .map(|i| {
                    let w = pool.writes[write_start + i];
                    format!("{:?}@d{}", w.loc, w.dev)
                })
                .collect();
            format!("{what}: {detail}; prefix={prefix:?} suffix={suffix:?} persisted=[{}] unreadable={unreadable:?}", persisted_list.join(","))
        };
        let rec = match recover(&img) {
            Ok(r) => r,
            Err(e) => {
                if overlay_for.is_some() { t.overlay_red += 1 } else { t.recover_fail += 1 }
                if t.examples.len() < 3 { t.examples.push(describe("RECOVER_FAIL", &e)); }
                return;
            }
        };
        let content = match check(&img, &rec) {
            Ok(c) => c,
            Err(e) => {
                if overlay_for.is_some() { t.overlay_red += 1 } else { t.check_red += 1 }
                if t.examples.len() < 3 { t.examples.push(describe("CHECK_RED", &format!("{e} (recovered txg {})", rec.txg))); }
                return;
            }
        };
        if overlay_for.is_some() {
            return;
        }
        let version = pool.versions.iter().find(|v| v.txg == rec.txg).expect("恢复只会落在发过的版本上");
        if version.content != content {
            t.content_mismatch += 1;
        }
        let newest_persisted_root = suffix_versions
            .iter()
            .filter(|v| p(v.root_write))
            .map(|v| v.txg)
            .max()
            .unwrap_or(prefix_newest);
        if rec.txg < newest_persisted_root {
            t.root_lost += 1;
        }
        let mut cut = false;
        let mut unverified = false;
        for v in &suffix_versions {
            if rec.txg < v.txg || p(v.root_write) {
                continue;
            }
            let last_persisted = v.record_writes.last().is_some_and(|c| c.iter().any(|w| p(*w)));
            if !last_persisted {
                cut = true;
            }
            if v.named_record_positions.iter().any(|pos| !v.record_writes[*pos].iter().any(|w| p(*w))) {
                unverified = true;
            }
        }
        if cut {
            t.cut_publish_applied += 1;
            if t.examples.len() < 3 { t.examples.push(describe("CUT_PUBLISH_APPLIED", &format!("recovered txg {}", rec.txg))); }
        }
        t.unverified_applied += u64::from(unverified);
        if lookup_misses(&img, &rec, &content) > 0 {
            t.lookup_miss += 1;
            if t.examples.len() < 3 { t.examples.push(describe("LOOKUP_MISS", &format!("recovered txg {}", rec.txg))); }
        }
    };
    let mut run_all = |t: &mut Tally, unreadable: &BTreeSet<usize>, overlay_for: Option<&Version>| {
        persisted.iter_mut().for_each(|x| *x = false);
        eval(&persisted, t, unreadable, overlay_for);
        for seg in &segments {
            for mask in 1u64..(1u64 << seg.len()) {
                for (bit, w) in seg.iter().enumerate() {
                    persisted[*w] = mask & (1 << bit) != 0;
                }
                eval(&persisted, t, unreadable, overlay_for);
            }
            for w in seg {
                persisted[*w] = true;
            }
        }
    };
    run_all(&mut t, &none, None);
    if overlay {
        // 介质故障叠加：这次发布的某个单元两份都读不出，只数这次发布的根没落的状态。
        for v in &suffix_versions {
            let unit_blobs: Vec<usize> = pool.writes[write_start..]
                .iter()
                .filter(|w| matches!(w.loc, Loc::Unit(_)) && w.dev == 0)
                .filter(|w| matches!(&pool.blobs[w.blob], Blob::Unit { header, .. } if header.birth_txg.0 == v.txg))
                .map(|w| w.blob)
                .collect();
            for u in unit_blobs {
                let bad = BTreeSet::from([u]);
                run_all(&mut t, &bad, Some(v));
            }
        }
    }
    assert_eq!(t.states, t.closed_form, "逐个枚举的状态数与闭式相等");
    t
}

// ---------------------------------------------------------------- 历史的扫描

pub fn base_cfg(name: &'static str) -> Cfg {
    Cfg {
        name,
        leaf_cap: 4,
        int_cap: 4,
        split: SplitPoint::Middle,
        sep: SepRule::Maintained,
        shrink: Shrink::Merge { min_fill: 2, borrow: true },
        step: WriteStep::OnePublish,
        last: LastRule::Refuse,
        named_cap: 1000,
    }
}

fn ins_all(keys: impl IntoIterator<Item = u64>) -> Vec<Vec<Op>> {
    keys.into_iter().map(|k| vec![Op::Ins(k)]).collect()
}

/// 前缀：装置造的起点（全部持久、不枚举）。
pub fn prefixes() -> Vec<(&'static str, Vec<Vec<Op>>)> {
    let pa = ins_all((1..=8).map(|i| i * 10));
    let pb = ins_all((1..=20).map(|i| i * 10));
    let mut pc = pb.clone();
    pc.extend([30u64, 40, 50, 60, 70, 120, 130].iter().map(|k| vec![Op::Del(*k)]));
    let pd = ins_all((1..=64).map(|i| i * 10));
    let mut pe = pa.clone();
    pe.extend(ins_all([90u64, 100, 110, 120, 35, 65, 95]));
    vec![("PA", pa), ("PB", pb), ("PC", pc), ("PD", pd), ("PE", pe)]
}

/// 用户动作的菜单：前缀之后树里有哪些 key，就在它们之间、之下、之上各插一个，或删掉其中一个；另加相邻两插的一批。
pub fn menus(prefix: &[Vec<Op>]) -> (Vec<Vec<Op>>, Vec<Vec<Op>>) {
    let mut keys = BTreeSet::new();
    for ops in prefix {
        for op in ops {
            match *op {
                Op::Ins(k) => {
                    keys.insert(k);
                }
                Op::Del(k) => {
                    keys.remove(&k);
                }
            }
        }
    }
    let mut singles: Vec<Vec<Op>> = vec![];
    let mut inserts = vec![1u64];
    let top = keys.iter().max().copied().unwrap_or(0);
    inserts.extend((0..=top / 10 + 1).map(|i| i * 10 + 5));
    for k in &inserts {
        singles.push(vec![Op::Ins(*k)]);
    }
    for k in &keys {
        singles.push(vec![Op::Del(*k)]);
    }
    let mut firsts = singles.clone();
    for k in &inserts {
        firsts.push(vec![Op::Ins(*k), Op::Ins(*k + 2)]);
    }
    let dels: Vec<u64> = keys.iter().copied().collect();
    for w in dels.windows(2) {
        firsts.push(vec![Op::Del(w[0]), Op::Del(w[1])]);
    }
    (firsts, singles)
}

pub fn scan(cfg: Cfg, overlay: bool, threads: usize) -> Tally {
    let mut jobs: Vec<(usize, Vec<Vec<Op>>)> = vec![];
    let wanted = std::env::var("TREESPLIT_PREFIXES").unwrap_or_else(|_| "PA,PB,PC,PD,PE".to_string());
    let built: Vec<(Vec<Vec<Op>>, Pool)> = prefixes()
        .into_iter()
        .filter(|(name, _)| wanted.split(',').any(|w| w == *name))
        .map(|(_, prefix)| {
            let pool = pool_after(cfg, &prefix);
            (prefix, pool)
        })
        .collect();
    for (index, (prefix, _)) in built.iter().enumerate() {
        let (firsts, seconds) = menus(prefix);
        for a in &firsts {
            for b in &seconds {
                jobs.push((index, vec![a.clone(), b.clone()]));
            }
        }
    }
    let next = AtomicU64::new(0);
    let total = Mutex::new(Tally::default());
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                let mut local = Tally::default();
                loop {
                    let i = usize::try_from(next.fetch_add(1, Ordering::Relaxed)).unwrap();
                    if i >= jobs.len() {
                        break;
                    }
                    let (index, suffix) = &jobs[i];
                    let (prefix, pool) = &built[*index];
                    local.add(&run_history_from(pool, prefix, suffix, overlay));
                }
                total.lock().unwrap().add(&local);
            });
        }
    });
    total.into_inner().unwrap()
}

fn threads() -> usize {
    std::env::var("TREESPLIT_THREADS").ok().and_then(|s| s.parse().ok()).unwrap_or(8)
}

fn print(cfg: &Cfg, t: &Tally) {
    println!("{}", t.line(cfg.name));
    println!(
        "  SHAPE {} split={:?} sep={:?} shrink={:?} step={:?} last={:?} leaf_cap={} int_cap={} named_cap={}",
        cfg.name, cfg.split, cfg.sep, cfg.shrink, cfg.step, cfg.last, cfg.leaf_cap, cfg.int_cap, cfg.named_cap
    );
    for e in &t.examples {
        println!("  EXAMPLE {} {e}", cfg.name);
    }
}

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn t1_configs() -> Vec<Cfg> {
    let mut v = vec![];
    for split in [SplitPoint::Middle, SplitPoint::TailWhenAppend] {
        for sep in [SepRule::CreationMin, SepRule::Maintained] {
            for shrink in [
                Shrink::DropEmpty { collapse_root: false },
                Shrink::DropEmpty { collapse_root: true },
                Shrink::Merge { min_fill: 2, borrow: false },
                Shrink::Merge { min_fill: 2, borrow: true },
            ] {
                for step in [WriteStep::OnePublish, WriteStep::SplitOwnBarrier, WriteStep::SplitOwnPublish] {
                    let name = leak(format!("T1/{split:?}/{sep:?}/{shrink:?}/{step:?}").replace(' ', ""));
                    v.push(Cfg { name, split, sep, shrink, step, ..base_cfg("") });
                }
            }
        }
    }
    v
}

pub fn t6_configs() -> Vec<Cfg> {
    [LastRule::Refuse, LastRule::SpreadLiteral, LastRule::SpreadFlag, LastRule::CutPublish]
        .into_iter()
        .map(|last| Cfg { name: leak(format!("T6/{last:?}/named_cap=3")), last, named_cap: 3, ..base_cfg("") })
        .collect()
}

// ---------------------------------------------------------------- 自检：判据自己会不会红

#[test]
fn self_test_the_checker_and_oracle_can_go_red() {
    let cfg = base_cfg("self");
    // 1. 正常历史：checker 绿、oracle 零违例。
    let t = run_history(cfg, &ins_all([10, 20, 30, 40]), &[vec![Op::Ins(50)], vec![Op::Del(10)]], false);
    assert_eq!((t.check_red, t.content_mismatch, t.root_lost, t.cut_publish_applied, t.lookup_miss), (0, 0, 0, 0, 0), "{t:?}");
    assert!(t.ev_split >= 1, "第五条插入要分裂：{t:?}");
    // 2. 指针坏：把一个单元两份都读不出、而它的根已落 ⇒ checker 必须红（overlay 只数根没落的状态，这里直接构造）。
    let mut pool = Pool::new(cfg);
    for ops in ins_all([10, 20, 30, 40, 50]) {
        pool.user_publish(&ops);
    }
    let mut base = HashMap::new();
    for w in &pool.writes {
        base.insert((w.dev, w.loc), w.blob);
    }
    let over = HashMap::new();
    let leaf_blob = pool
        .writes
        .iter()
        .rev()
        .find(|w| matches!(&pool.blobs[w.blob], Blob::Unit { header, .. } if header.level == 0))
        .unwrap()
        .blob;
    let bad = BTreeSet::from([leaf_blob]);
    let img = Image { pool: &pool, base: &base, overlay: &over, persisted: &[], first_enumerated_write: pool.writes.len(), unreadable: &bad };
    let rec = recover(&img).unwrap();
    assert!(check(&img, &rec).is_err(), "叶两份都读不出，checker 必须红");
    // 3. 分隔 key 照 inode 树的「创建时的最小 key、不许跟着变」：插到最左分隔 key 之下，checker 与查找都必须红。
    let creation_min = Cfg { sep: SepRule::CreationMin, ..cfg };
    let t = run_history(creation_min, &ins_all([10, 20, 30, 40, 50]), &[vec![Op::Ins(1)]], false);
    assert!(t.check_red > 0, "CreationMin 插到最左分隔 key 之下必须红：{t:?}");
    let t = run_history(cfg, &ins_all([10, 20, 30, 40, 50]), &[vec![Op::Ins(1)]], false);
    assert_eq!(t.check_red, 0, "Maintained 同一段历史不红：{t:?}");
    // 4. 发布边界判据自己会红：SpreadLiteral 在记录段切在末条之前的状态上施加了半次发布。
    let literal = Cfg { last: LastRule::SpreadLiteral, named_cap: 1, ..cfg };
    let t = run_history(literal, &ins_all([10, 20, 30, 40]), &[vec![Op::Ins(50)]], false);
    assert!(t.cut_publish_applied > 0, "字面认末条必须在切开的状态上施加：{t:?}");
    let flag = Cfg { last: LastRule::SpreadFlag, named_cap: 1, ..cfg };
    let t = run_history(flag, &ins_all([10, 20, 30, 40]), &[vec![Op::Ins(50)]], false);
    assert_eq!(t.cut_publish_applied, 0, "标志位认末条不施加半次：{t:?}");
    println!("SELFTEST ok");
}

#[test]
fn scan_t1_candidates() {
    if std::env::var("TREESPLIT_SCAN").ok().as_deref() != Some("t1") {
        return;
    }
    let only = std::env::var("TREESPLIT_ONLY").ok();
    for cfg in t1_configs() {
        if only.as_ref().is_some_and(|o| !cfg.name.contains(o.as_str())) {
            continue;
        }
        let t = scan(cfg, false, threads());
        print(&cfg, &t);
    }
}

#[test]
fn scan_t6_candidates() {
    if std::env::var("TREESPLIT_SCAN").ok().as_deref() != Some("t6") {
        return;
    }
    for cfg in t6_configs() {
        let t = scan(cfg, true, threads());
        print(&cfg, &t);
    }
}

fn shape(n: &Node) -> String {
    match n {
        Node::Leaf { kv, .. } => format!("{:?}", kv.iter().map(|e| e.0).collect::<Vec<_>>()),
        Node::Int { kids, level, .. } => format!(
            "L{level}<{}>",
            kids.iter().map(|k| format!("{}:{}", k.0, shape(&k.1))).collect::<Vec<_>>().join(" ")
        ),
    }
}

#[test]
fn print_prefix_shapes() {
    for cfg in [base_cfg("Middle"), Cfg { split: SplitPoint::TailWhenAppend, ..base_cfg("Tail") }] {
        for (name, prefix) in prefixes() {
            let mut pool = Pool::new(cfg);
            for ops in &prefix {
                pool.user_publish(ops);
            }
            println!("PREFIX {} {name} height={} nodes={} {}", cfg.name, height(&pool.tree), node_count(&pool.tree), shape(&pool.tree));
        }
    }
}

/// 定向历史：D8 已定项 6 的「合并只许左吸收右 / 左从右借」+「分隔 key = 创建时的最小 key、不许跟着变」搬到一般的树上，
/// 左从右借之后借过来的那条 key 落在左孩子里、而它 ≥ 右孩子的分隔 key ⇒ 按「最后一个分隔 key ≤ 目标 key」查找走到右孩子，找不到。
/// 不插比最左分隔 key 小的 key（那一种另算），只删。
#[test]
fn borrow_from_the_right_under_the_creation_min_separator() {
    for sep in [SepRule::CreationMin, SepRule::Maintained] {
        let cfg = Cfg {
            name: "borrow",
            split: SplitPoint::TailWhenAppend,
            sep,
            shrink: Shrink::Merge { min_fill: 2, borrow: true },
            ..base_cfg("borrow")
        };
        let prefix = ins_all((1..=8).map(|i| i * 10));
        let suffix = vec![vec![Op::Del(10), Op::Del(20)], vec![Op::Del(30)]];
        let pool = {
            let mut p = pool_after(cfg, &prefix);
            for ops in &suffix {
                p.user_publish(ops);
            }
            p
        };
        let base: HashMap<(u8, Loc), usize> = pool.writes.iter().map(|w| ((w.dev, w.loc), w.blob)).collect();
        let over = HashMap::new();
        let none = BTreeSet::new();
        let img = Image { pool: &pool, base: &base, overlay: &over, persisted: &[], first_enumerated_write: pool.writes.len(), unreadable: &none };
        let rec = recover(&img).unwrap();
        let expected = &pool.versions.last().unwrap().content;
        let checked = check(&img, &rec);
        let misses = lookup_misses(&img, &rec, expected);
        println!(
            "BORROW sep={sep:?} tree={} checker={} lookup_misses={misses} keys={:?}",
            shape(&pool.tree),
            match &checked { Ok(_) => "green".to_string(), Err(e) => format!("red: {e}") },
            expected.keys().collect::<Vec<_>>()
        );
        let t = run_history(cfg, &prefix, &suffix, false);
        println!("BORROW_ENUM sep={sep:?} {}", t.line("borrow"));
    }
}

/// 定向历史：TailWhenAppend 的根分裂（树高涨一层）在两步菜单里没出现（t1.out 的 ev_height_up=0），
/// 这里按升序追加，找到让树高从 2 涨到 3 的那一次插入，把它（与它之后的一次删除）放进枚举段。
#[test]
fn tail_split_root_growth_is_enumerated() {
    for step in [WriteStep::OnePublish, WriteStep::SplitOwnBarrier, WriteStep::SplitOwnPublish] {
        let cfg = Cfg { name: "tail-root", split: SplitPoint::TailWhenAppend, step, ..base_cfg("tail-root") };
        let mut n = 1u64;
        let found = loop {
            let prefix = ins_all((1..=n).map(|i| i * 10));
            let before = pool_after(cfg, &prefix);
            let mut after = before.clone();
            after.user_publish(&[Op::Ins((n + 1) * 10)]);
            if height(&before.tree) == 2 && height(&after.tree) == 3 {
                break prefix;
            }
            n += 1;
            assert!(n < 400, "找不到让树高涨到 3 的插入");
        };
        let suffix = vec![vec![Op::Ins((n + 1) * 10)], vec![Op::Del(10)]];
        let t = run_history(cfg, &found, &suffix, false);
        println!("TAILROOT step={step:?} prefix_len={n} {}", t.line("tail-root"));
    }
}
