//! 影子历史：C2a（记账按每次建 inode 的发布留 (g, H) 台阶）、C2a-all（每次水位变了的发布都留一行，含回退那次的跳升）、
//! C3r（根下只追加的 (号, 出生代)），各加谱系核对；外加真相（这条时间线上发过的 (号, 出生代)）。
//! 历史随现行那一版走：发布接在上一版后面就从上一版带；回退那次从 cur 带（不取 R_old），谱系追加一条
//! (T_rb, T_old, T_cur, H_cur, |log(cur)|)。这些都只是影子，盘上没有。

use singlefs_core::transaction::TransactionOutput;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Hist {
    /// C2a 字面：每次建 inode 的发布一行 (这次的 txg, 这次之后的水位)。
    pub rows_create: Vec<(u64, u64)>,
    /// C2a-all：每次水位变了的发布一行（回退那次跳升也记）。
    pub rows_all: Vec<(u64, u64)>,
    /// C3r：(号, 出生代)。
    pub log: Vec<(u64, u64)>,
    /// 谱系：(T_rb, T_old, T_cur, H_cur, |log(cur)|, 回退那次发布之后的水位 H_new)。
    pub lineage: Vec<(u64, u64, u64, u64, u64, u64)>,
    /// 真相：这条时间线上发过的 (号, 出生代)（含管理员回退抛弃那一段，不含崩溃恢复抛弃那一段）。
    pub issued: Vec<(u64, u64)>,
}

fn push_sorted(list: &mut Vec<(u64, u64)>, item: (u64, u64)) {
    if !list.contains(&item) {
        list.push(item);
        list.sort_unstable();
    }
}

impl Hist {
    pub fn publish(&mut self, txg: u64, wm_before: u64, wm_after: u64, records: &[(u64, u64, u32)]) {
        let new: Vec<(u64, u64)> = records.iter().filter(|r| r.1 == txg).map(|r| (r.0, r.1)).collect();
        if !new.is_empty() {
            self.rows_create.push((txg, wm_after));
        }
        if wm_after != wm_before {
            self.rows_all.push((txg, wm_after));
        }
        for item in new {
            push_sorted(&mut self.log, item);
            push_sorted(&mut self.issued, item);
        }
    }

    #[must_use]
    pub fn after_publish(mut self, previous: &TransactionOutput, next: &TransactionOutput, next_records: &[(u64, u64, u32)]) -> Hist {
        self.publish(next.root.checkpoint_txg.0, previous.inode_number_watermark(), next.inode_number_watermark(), next_records);
        self
    }

    /// 管理员回退：历史从 cur 带，水位变了就在 rows_all 记一行；谱系追加一条。真相并上 R_old 的树（本来就在里面）。
    #[must_use]
    pub fn after_rollback(mut self, current: &TransactionOutput, new: &TransactionOutput, target_txg: u64, target_records: &[(u64, u64, u32)]) -> Hist {
        let t_rb = new.root.checkpoint_txg.0;
        if new.inode_number_watermark() != current.inode_number_watermark() {
            self.rows_all.push((t_rb, new.inode_number_watermark()));
        }
        let count = u64::try_from(self.log.len()).expect("小");
        self.lineage.push((t_rb, target_txg, current.root.checkpoint_txg.0, current.inode_number_watermark(), count, new.inode_number_watermark()));
        for r in target_records {
            push_sorted(&mut self.issued, (r.0, r.1));
        }
        self
    }

    pub fn reissue_on_the_timeline(&self) -> bool {
        self.issued.windows(2).any(|w| w[0].0 == w[1].0 && w[0].1 != w[1].1)
    }
}

pub const NAMES: [&str; 13] = [
    "C2a-create:I930-1",
    "C2a-create:I930-2",
    "C2a-create:I930-3",
    "C2a-all:I930-123",
    "C2aL-create:pair",
    "C2aL-all:pair",
    "C3r",
    "C3rs",
    "C3r:I930-3",
    "C3rL:count",
    "C2a-create:I930-123",
    "C2aL-all:pair(Tcur,Hnew)",
    "C2aL-all:pair(Trb,Hnew)",
];

pub struct Judge;

fn floor_red(rows: &[(u64, u64)], wm: u64) -> bool {
    rows.iter().any(|(_, h)| *h > wm)
}
fn cut_red(rows: &[(u64, u64)], tree: &[(u64, u64)]) -> bool {
    tree.iter().any(|&(n, b)| rows.iter().any(|&(g, h)| h > n && b > g))
}
fn cover_red(rows: &[(u64, u64)], wm: u64, txg: u64) -> bool {
    wm > 2 && !rows.iter().any(|&(g, h)| h >= wm && g <= txg)
}
fn pair_red(rows: &[(u64, u64)], lineage: &[(u64, u64, u64, u64, u64, u64)]) -> bool {
    lineage.iter().any(|&(_, _, t_cur, h_cur, _, _)| !rows.iter().any(|&(g, h)| g <= t_cur && h >= h_cur))
}

impl Judge {
    /// 最新那一版：水位、txg、树里的 (号, 出生代)。
    #[must_use]
    pub fn verdicts(h: &Hist, wm: u64, txg: u64, tree: &[(u64, u64)]) -> [bool; 13] {
        let log_two = h.log.windows(2).any(|w| w[0].0 == w[1].0 && w[0].1 != w[1].1);
        let log_high = h.log.iter().any(|(n, _)| *n >= wm);
        let c3r = log_two || log_high;
        let c3rs = c3r || tree.iter().any(|p| !h.log.contains(p));
        let log_max = h.log.iter().map(|(n, _)| *n).max().unwrap_or(0);
        let c3r_cover = wm > 2 && log_max + 1 < wm;
        let c3rl = h.lineage.iter().any(|&(_, _, t_cur, _, count, _)| {
            u64::try_from(h.log.iter().filter(|(_, b)| *b <= t_cur).count()).expect("小") < count
        });
        let a1 = floor_red(&h.rows_create, wm);
        let a2 = cut_red(&h.rows_create, tree);
        let a3 = cover_red(&h.rows_create, wm, txg);
        let all = floor_red(&h.rows_all, wm) || cut_red(&h.rows_all, tree) || cover_red(&h.rows_all, wm, txg);
        [
            a1,
            a2,
            a3,
            all,
            pair_red(&h.rows_create, &h.lineage),
            pair_red(&h.rows_all, &h.lineage),
            c3r,
            c3rs,
            c3r_cover,
            c3rl,
            a1 || a2 || a3,
            h.lineage.iter().any(|&(_, _, t_cur, _, _, h_new)| !h.rows_all.iter().any(|&(g, hh)| g <= t_cur && hh >= h_new)),
            h.lineage.iter().any(|&(t_rb, _, _, _, _, h_new)| !h.rows_all.iter().any(|&(g, hh)| g <= t_rb && hh >= h_new)),
        ]
    }
}
