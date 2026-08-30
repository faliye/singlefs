//! E9：全路径索引 vs inode vs inode + 继承式 locality 前缀。
//!
//! 测的是 **key 编码决定的遍历局部性**：一次子树递归遍历要碰多少个不同的叶子。
//! 那个数直接决定设备 I/O 次数，而 key 编码是 [decisions.md] D8 已定的东西。
//!
//! **四档，第四档是阳性对照**：哈希打散的 key 在遍历局部性上必须最差。
//! 若度量分不出「哈希打散」和「全路径」，那它就没有判别力，前三档的结论一并作废。

use e7_index_bench::{Emitter, Lru};
use std::alloc::{alloc, dealloc, Layout};
use std::os::unix::fs::OpenOptionsExt;
use std::io::{Read, Seek, SeekFrom, Write};

const O_DIRECT: i32 = 0o40000;
const PAGE: usize = 4096;
const SLOTS: usize = (PAGE - 16) / 16; // 每叶能放多少 (key,val)

// ── 目录树：DEPTH 层，每层 DIR_FANOUT 个子目录，叶目录里 FILES_PER_DIR 个文件 ──
const DEPTH: usize = 4;
const DIR_FANOUT: usize = 6;
const FILES_PER_DIR: usize = 24;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Layout4 { FullPath, Inode, Locality, Hashed }

#[derive(Clone)]
struct Obj {
    path: [u8; DEPTH],   // **当前**路径（改名会改它）
    inode: u64,          // 创建顺序，永不变
    locality: u64,       // 创建时继承，**改名不更新**
    dir_id: usize,       // 当前所属叶目录的序号（改名后重算）
}

fn build_tree(interleave: bool, seed: u64) -> (Vec<Obj>, usize) {
    let n_dirs = DIR_FANOUT.pow(DEPTH as u32);
    let mut objs: Vec<Obj> = Vec::with_capacity(n_dirs * FILES_PER_DIR);
    // 先按深度优先列出所有叶目录的路径
    let mut dirs: Vec<[u8; DEPTH]> = Vec::with_capacity(n_dirs);
    let mut p = [0u8; DEPTH];
    fn rec(lvl: usize, p: &mut [u8; DEPTH], out: &mut Vec<[u8; DEPTH]>) {
        if lvl == DEPTH { out.push(*p); return; }
        for b in 0..DIR_FANOUT { p[lvl] = b as u8; rec(lvl + 1, p, out); }
    }
    rec(0, &mut p, &mut dirs);

    // locality_id：创建时从父目录继承（[decisions.md] D8 原文），根下每个顶层目录铸一个新的，
    // 其后代全部继承。**改名时故意不更新**——那正是 D8 说的「局部性缓慢退化」的来源。
    let loc_of = |path: &[u8; DEPTH]| -> u64 { path[0] as u64 };

    // 创建顺序：depth-first（解压 tar 的形态）或 interleave（老化的代理）
    let mut order: Vec<usize> = (0..n_dirs).collect();
    if interleave {
        let mut s = seed | 1;
        for i in (1..order.len()).rev() {
            s ^= s >> 12; s ^= s << 25; s ^= s >> 27;
            let j = (s.wrapping_mul(0x2545F4914F6CDD1D) as usize) % (i + 1);
            order.swap(i, j);
        }
    }
    let mut ino = 0u64;
    for &di in &order {
        for _f in 0..FILES_PER_DIR {
            objs.push(Obj { path: dirs[di], inode: ino, locality: loc_of(&dirs[di]), dir_id: di });
            ino += 1;
        }
    }
    (objs, n_dirs)
}

/// 改名 R 次：每次随机挑一层 `lvl`（1..DEPTH），把「该层某个分支」整体挪到同层另一个分支号下。
/// 只改 path，**不改 locality 也不改 inode**——这是 D8 明写的语义。
/// 返回「全路径编码下必须重写的 key 数」，即全路径索引为改名付的代价。
fn apply_renames(objs: &mut [Obj], n_renames: usize, seed: u64) -> u64 {
    let mut s = seed | 1;
    let mut r = || { s ^= s >> 12; s ^= s << 25; s ^= s >> 27; s.wrapping_mul(0x2545F4914F6CDD1D) };
    let mut fullpath_key_writes = 0u64;
    for _ in 0..n_renames {
        // 交换两棵同层子树的位置。**必须保树形**——直接把 from 改写成 to 是把两棵子树合并，
        // 目录数会被销毁，遍历要访问的目录变少，结果会朝「改名越多越快」这个不可能的方向走。
        // 交换两棵**前缀可以不同**的同层子树 —— 即把一棵深层子树移到另一个顶层分支下。
        // 这一条是退化的唯一来源：locality_id 是创建时铸的、改名不更新，
        // 只有当子树跨过 locality 组边界时，它才与当前路径失配。
        // ⚠️ 前缀相同的交换（同一父目录内换位）测不出退化——组内换位不拆散任何组；
        // 顶层整体交换也测不出——整组换个位置被访问，组内仍然连续。
        let lvl = 1 + (r() as usize) % DEPTH;           // 子树根所在的层，1..=DEPTH
        let mut p1 = [0u8; DEPTH]; let mut p2 = [0u8; DEPTH];
        for i in 0..lvl { p1[i] = ((r() as usize) % DIR_FANOUT) as u8;
                          p2[i] = ((r() as usize) % DIR_FANOUT) as u8; }
        if p1[..lvl] == p2[..lvl] { continue; }
        for o in objs.iter_mut() {
            if o.path[..lvl] == p1[..lvl] {
                o.path[..lvl].copy_from_slice(&p2[..lvl]); fullpath_key_writes += 1;
            } else if o.path[..lvl] == p2[..lvl] {
                o.path[..lvl].copy_from_slice(&p1[..lvl]); fullpath_key_writes += 1;
            }
        }
    }
    // 路径变了，重算 dir_id（遍历按当前路径顺序走）
    let mut ids: Vec<[u8; DEPTH]> = objs.iter().map(|o| o.path).collect();
    ids.sort_unstable(); ids.dedup();
    let idx: std::collections::HashMap<[u8; DEPTH], usize> =
        ids.iter().enumerate().map(|(i, p)| (*p, i)).collect();
    for o in objs.iter_mut() { o.dir_id = idx[&o.path]; }
    fullpath_key_writes
}

fn key_of(o: &Obj, l: Layout4) -> u64 {
    match l {
        // 全路径：按路径分量逐层打包 —— 兄弟相邻，子树连续
        Layout4::FullPath => {
            let mut k = 0u64;
            for lv in 0..DEPTH { k = k * DIR_FANOUT as u64 + o.path[lv] as u64; }
            k * FILES_PER_DIR as u64 + (o.inode % FILES_PER_DIR as u64)
        }
        // 纯 inode：创建顺序
        Layout4::Inode => o.inode,
        // locality 前缀 + inode：一起创建的子树聚在一起，块内按 inode 序
        Layout4::Locality => (o.locality << 40) | o.inode,
        // 阳性对照：哈希打散，遍历局部性必须最差
        Layout4::Hashed => {
            let mut x = o.inode.wrapping_mul(0x9E3779B97F4A7C15);
            x ^= x >> 31; x = x.wrapping_mul(0xBF58476D1CE4E5B9); x ^= x >> 27; x
        }
    }
}

// ── 盘上一棵按 key 排序的 B+ 树，只关心「一个 key 落在哪个叶」 ──
struct Aligned { ptr: *mut u8, len: usize, lay: Layout }
impl Aligned {
    fn new(len: usize) -> Self {
        let lay = Layout::from_size_align(len, PAGE).unwrap();
        Aligned { ptr: unsafe { alloc(lay) }, len, lay }
    }
    fn as_mut(&mut self) -> &mut [u8] { unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) } }
}
impl Drop for Aligned { fn drop(&mut self) { unsafe { dealloc(self.ptr, self.lay) } } }

struct Dev { f: std::fs::File, reads: u64, writes: u64 }
impl Dev {
    fn open(p: &str) -> std::io::Result<Self> {
        let f = std::fs::OpenOptions::new().read(true).write(true).custom_flags(O_DIRECT).open(p)?;
        Ok(Dev { f, reads: 0, writes: 0 })
    }
    fn read(&mut self, pg: u64, b: &mut Aligned) -> std::io::Result<()> {
        self.f.seek(SeekFrom::Start(pg * PAGE as u64))?;
        self.f.read_exact(b.as_mut())?; self.reads += 1; Ok(())
    }
    fn write(&mut self, pg: u64, b: &mut Aligned) -> std::io::Result<()> {
        self.f.seek(SeekFrom::Start(pg * PAGE as u64))?;
        self.f.write_all(b.as_mut())?; self.writes += 1; Ok(())
    }
}

/// 跑一档：把全部对象按该编码排序，切成叶；然后逐个子树遍历，数「碰了多少不同的叶」
/// 和「实际发生多少次设备读」（带 LRU 缓存，缓存大小是自变量）。
/// 按某种 key 布局排序，算出每个对象落在第几个叶，以及叶总数。
///
/// ⚠️ **抽成函数是为了让测试能考它本身，不是为了好看**（2026-08-29 对抗验证）：
/// 此前这段逻辑内联在 `run` 里，而新加的绝对值测试**自己重实现了一遍**——
/// 变异测试把生产侧的 `pos / SLOTS` 改成 `pos / (SLOTS + 1)` 时**一个测试都没红**。
/// 那正是 kb 在 E28 记过的同一个错：测试绕开了被测代码。
fn assign_leaves(objs: &[Obj], l: Layout4) -> (Vec<usize>, usize) {
    let mut ks: Vec<(u64, usize)> = objs.iter().enumerate().map(|(i, o)| (key_of(o, l), i)).collect();
    ks.sort_unstable();
    let mut leaf_of_obj = vec![0usize; objs.len()];
    for (pos, &(_, oi)) in ks.iter().enumerate() { leaf_of_obj[oi] = pos / SLOTS; }
    (leaf_of_obj, (ks.len() + SLOTS - 1) / SLOTS)
}

fn run(dev: &mut Dev, objs: &[Obj], n_dirs: usize, l: Layout4, cache_leaves: usize)
    -> (usize, u64, u64, usize) {
    // 1) 排序 → 叶编号
    let (leaf_of_obj, n_leaves) = assign_leaves(objs, l);

    // 2) 把叶写到盘上（内容不重要，重要的是它们真的落盘、真的被读回）
    let mut buf = Aligned::new(PAGE);
    for li in 0..n_leaves {
        buf.as_mut()[0..8].copy_from_slice(&(li as u64).to_le_bytes());
        dev.write(li as u64, &mut buf).expect("write leaf");
    }
    let (r0, w0) = (dev.reads, dev.writes);

    // 3) 每个叶目录做一次子树遍历，数不同叶数与实际设备读数
    let mut by_dir: Vec<Vec<usize>> = vec![Vec::new(); n_dirs];
    for (i, o) in objs.iter().enumerate() { by_dir[o.dir_id].push(i); }

    let mut lru = Lru::new(cache_leaves);
    let mut distinct_total = 0usize;
    for d in 0..n_dirs {
        let mut seen: Vec<usize> = by_dir[d].iter().map(|&i| leaf_of_obj[i]).collect();
        seen.sort_unstable(); seen.dedup();
        distinct_total += seen.len();
        for lf in seen {
            let pg = lf as u64;
            let hit = lru.contains(pg);
            lru.touch(pg);                              // 命中就提到最近端，不命中就插入并逐出
            if !hit { dev.read(pg, &mut buf).expect("read leaf"); }
        }
    }
    (n_leaves, dev.reads - r0, dev.writes - w0, distinct_total)
}

#[derive(Clone, Copy)]
struct Blk { r: u64, w: u64 }
fn blkstat(dev_path: &str) -> Option<Blk> {
    let name = dev_path.rsplit('/').next()?;
    let t = std::fs::read_to_string(format!("/sys/block/{name}/stat")).ok()?;
    let f: Vec<u64> = t.split_whitespace().filter_map(|x| x.parse().ok()).collect();
    if f.len() < 8 { return None; }
    Some(Blk { r: f[0], w: f[4] })
}

fn main() {
    let dev_path = std::env::args().nth(1).unwrap_or_else(|| { eprintln!("用法: e9-keylayout <dev> [seed] [interleave] [cache_leaves]"); std::process::exit(2) });
    let seed: u64 = std::env::args().nth(2).and_then(|x| x.parse().ok()).unwrap_or(11);
    let interleave = std::env::args().nth(3).map(|x| x == "interleave").unwrap_or(false);
    let cache_leaves: usize = std::env::args().nth(4).and_then(|x| x.parse().ok()).unwrap_or(64);
    let n_renames: usize = std::env::args().nth(5).and_then(|x| x.parse().ok()).unwrap_or(0);

    let (mut objs, _) = build_tree(interleave, seed);
    let fullpath_key_writes = apply_renames(&mut objs, n_renames, seed ^ 0xABCD);
    // 改名会把不同路径合并，目录数因此会变——按当前路径重数
    let n_dirs = { let mut v: Vec<[u8; DEPTH]> = objs.iter().map(|o| o.path).collect();
                   v.sort_unstable(); v.dedup(); v.len() };
    let objs = objs;
    let mut em = Emitter::new();
    let mut dev = Dev::open(&dev_path).expect("open dev");
    let has_blk = blkstat(&dev_path).is_some();

    println!("{}", em.emit_raw(&format!(
        "name=config depth={DEPTH} dir_fanout={DIR_FANOUT} files_per_dir={FILES_PER_DIR} \
         dirs={n_dirs} objs={} slots_per_leaf={SLOTS} cache_leaves={cache_leaves} \
         interleave={interleave} seed={seed} renames={n_renames} \
         fullpath_key_writes={fullpath_key_writes} o_direct=true blkstat={has_blk}",
        objs.len())));

    for l in [Layout4::FullPath, Layout4::Inode, Layout4::Locality, Layout4::Hashed] {
        let b0 = blkstat(&dev_path);
        let (n_leaves, reads, writes, distinct) = run(&mut dev, &objs, n_dirs, l, cache_leaves);
        let b1 = blkstat(&dev_path);
        let (br, bw) = match (b0, b1) { (Some(a), Some(b)) => (b.r - a.r, b.w - a.w), _ => (0, 0) };
        println!("{}", em.emit_raw(&format!(
            "name=scan layout={l:?} leaves={n_leaves} distinct_leaves={distinct} \
             distinct_per_dir={:.4} dev_reads={reads} dev_writes={writes} blk_r={br} blk_w={bw}",
            distinct as f64 / n_dirs as f64)));
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 树的形状必须是声明的那个：`DIR_FANOUT^DEPTH` 个叶目录，每个 `FILES_PER_DIR` 个文件。
    /// 形状错了，所有「每目录多少个不同叶」的数都错。
    #[test]
    fn tree_shape_is_as_declared() {
        let (objs, n_dirs) = build_tree(false, 1);
        assert_eq!(n_dirs, DIR_FANOUT.pow(DEPTH as u32));
        assert_eq!(objs.len(), n_dirs * FILES_PER_DIR);
        let mut per_dir = std::collections::HashMap::new();
        for o in &objs { *per_dir.entry(o.dir_id).or_insert(0usize) += 1; }
        assert_eq!(per_dir.len(), n_dirs, "有目录一个文件都没有");
        assert!(per_dir.values().all(|&c| c == FILES_PER_DIR), "每目录的文件数不齐");
    }

    /// **把叶总数钉成一个独立算出来的绝对值：122。**
    ///
    /// ⚠️ **这条是补 `test-discipline.md`「只让多条臂互相比，测不出所有臂一起错」的洞**
    /// （2026-08-29 对抗验证补入）。本实验此前**一条绝对值断言都没有**：
    /// 七个单测钉的全是结构性质（树形、key 编码、改名语义、interleave 真的换了顺序），
    /// **没有一条钉住被量的那个数**。⇒ 叶归属或几何一旦整体错掉，
    /// 四条臂会一起错，而 `Inode/Locality = 1.46×` 这个比值仍然「成立」——
    /// **而那个比值正是 D8 采纳 locality 前缀时引用的数。**
    ///
    /// 122 由独立算术给出，不从代码里读：
    /// 槽数 `(4096 − 16) / 16 = 255`；对象数 `6^4 × 24 = 31104`；
    /// 叶数 `⌈31104 / 255⌉ = 122`。
    #[test]
    fn leaf_count_is_the_independently_computed_absolute_value() {
        assert_eq!(SLOTS, 255, "每叶槽数变了，122 这个绝对值跟着失效");
        let (objs, n_dirs) = build_tree(false, 1);
        assert_eq!(n_dirs * FILES_PER_DIR, 31104, "对象总数变了");
        let (_, n_leaves) = assign_leaves(&objs, Layout4::Inode);
        assert_eq!(n_leaves, 122, "叶总数不是独立算出来的那个 122");
    }

    /// **缓存装得下整棵树时，四条臂的读次数必须全部等于叶总数（122）。**
    ///
    /// 这是上一条的另一半：它把「四条臂相等」从一句互比，
    /// 变成「四条臂都等于一个独立算出来的绝对值」。
    /// kb 里那句「`cache_leaves ≥ 128` 时四条臂全部相等（各 122 次设备读）」
    /// 此前**只是观测记录，没有任何断言钉它**。
    ///
    /// ⚠️ 判据用的是「每条臂触到的不同叶的并集」——无限缓存下读次数恰等于它。
    /// 并集小于 122 说明有叶从没被任何目录读到，那时 122 这个上界就不是被测出来的。
    #[test]
    fn every_arm_touches_every_leaf_so_an_infinite_cache_reads_exactly_122() {
        let (objs, n_dirs) = build_tree(true, 7);
        for l in [Layout4::FullPath, Layout4::Inode, Layout4::Locality, Layout4::Hashed] {
            // ⚠️ 走**生产侧**那个 `assign_leaves`，不许在测试里重实现一遍
            let (leaf_of_obj, n_leaves) = assign_leaves(&objs, l);
            let mut by_dir: Vec<Vec<usize>> = vec![Vec::new(); n_dirs];
            for (i, o) in objs.iter().enumerate() { by_dir[o.dir_id].push(i); }
            let mut union = std::collections::HashSet::new();
            for d in 0..n_dirs { for &i in &by_dir[d] { union.insert(leaf_of_obj[i]); } }
            assert_eq!(union.len(), n_leaves, "{l:?} 臂没有触到全部叶");
            assert_eq!(union.len(), 122, "{l:?} 臂的无限缓存读次数不是 122");
        }
    }

    /// **一个叶装不下超过 `SLOTS` 个对象。** 这是叶归属那段算术的绝对判据。
    ///
    /// ⚠️ **它是变异测试逼出来的**（2026-08-29 对抗验证）：把生产侧的
    /// `pos / SLOTS` 改成 `pos / (SLOTS + 1)` 时，**叶总数、并集大小、每目录不同叶数全都不变**
    /// ——那三个量在这个变异下恰好巧合地相等，于是先加的两条绝对值断言一条都没红。
    /// 真正被破坏的是「一个叶装了 256 个对象而它只有 255 个槽」，**只有这条看得见**。
    #[test]
    fn no_leaf_holds_more_objects_than_it_has_slots() {
        let (objs, _) = build_tree(true, 7);
        for l in [Layout4::FullPath, Layout4::Inode, Layout4::Locality, Layout4::Hashed] {
            let (leaf_of_obj, n_leaves) = assign_leaves(&objs, l);
            let mut per_leaf = vec![0usize; n_leaves];
            for &lf in &leaf_of_obj { per_leaf[lf] += 1; }
            let max = *per_leaf.iter().max().unwrap();
            assert!(max <= SLOTS, "{l:?} 臂有叶装了 {max} 个对象，而每叶只有 {SLOTS} 个槽");
            assert_eq!(per_leaf.iter().sum::<usize>(), objs.len(), "{l:?} 臂有对象没被分配到叶");
        }
    }

    /// **全路径布局下，每个目录的 24 个文件最多跨 2 个叶。** 这是「排序真的发生了」的绝对判据。
    ///
    /// ⚠️ **同样是变异测试逼出来的**：把 `ks.sort_unstable()` 整行删掉时，
    /// 叶总数与并集大小仍然不变（对象照样铺满 122 个叶），**先加的两条断言都没红**。
    /// 而没有排序，全路径 key 的全部意义就没了——**这条才看得见它。**
    ///
    /// 上界 2 由算术给出：一个目录 24 个文件在 key 序上连续，
    /// 而一个叶有 255 个槽，24 个连续对象最多被一条叶边界切一次。
    #[test]
    fn sorting_makes_each_directory_span_at_most_two_leaves_under_fullpath() {
        let (objs, n_dirs) = build_tree(true, 7);
        let (leaf_of_obj, _) = assign_leaves(&objs, Layout4::FullPath);
        let mut by_dir: Vec<std::collections::HashSet<usize>> =
            vec![std::collections::HashSet::new(); n_dirs];
        for (i, o) in objs.iter().enumerate() { by_dir[o.dir_id].insert(leaf_of_obj[i]); }
        let worst = by_dir.iter().map(|s| s.len()).max().unwrap();
        assert!(worst <= 2, "全路径下有目录跨了 {worst} 个叶，排序没生效或 key 编码坏了");
        // 阴性对照：哈希打散必须显著更差，否则这条判据没有判别力
        let (hl, _) = assign_leaves(&objs, Layout4::Hashed);
        let mut hd: Vec<std::collections::HashSet<usize>> =
            vec![std::collections::HashSet::new(); n_dirs];
        for (i, o) in objs.iter().enumerate() { hd[o.dir_id].insert(hl[i]); }
        let hworst = hd.iter().map(|s| s.len()).max().unwrap();
        assert!(hworst > 10, "哈希臂只跨了 {hworst} 个叶，这条判据没有判别力");
    }

    /// **全路径 key 必须让兄弟相邻、子树连续。** 这是它作为遍历局部性上界的全部理由。
    /// 判据：把所有对象按 FullPath key 排序后，同一个目录的文件必须是连续的一段。
    #[test]
    fn fullpath_key_keeps_each_directory_contiguous() {
        let (mut objs, _) = build_tree(true, 3);
        objs.sort_by_key(|o| key_of(o, Layout4::FullPath));
        let mut seen = std::collections::HashSet::new();
        let mut prev = usize::MAX;
        for o in &objs {
            if o.dir_id != prev {
                assert!(seen.insert(o.dir_id), "目录 {} 的文件在排序后不连续", o.dir_id);
                prev = o.dir_id;
            }
        }
    }

    /// **locality key 的高位必须就是 locality**，否则「按 locality 分组」这件事不成立。
    #[test]
    fn locality_key_groups_by_locality() {
        let (objs, _) = build_tree(true, 5);
        for o in objs.iter().take(500) {
            assert_eq!(key_of(o, Layout4::Locality) >> 40, o.locality,
                       "locality key 的高位不是 locality");
        }
    }

    /// **阳性对照的机制，与全路径那条对称**：按哈希 key 排序后，
    /// 同一个目录的文件**必须被打散**（绝大多数目录不再连续）。
    /// 打不散的话 `Hashed` 就不再是「最差」的基准，整个度量失去判别力。
    ///
    /// ⚠️ 此前这条写成「相邻 inode 的哈希 key 之差不该很小」，
    /// 变异测试证明它抓不住——把第一次乘法去掉，后面的异或移位照样打散，
    /// 而**目录连续性**这个真正承重的性质当时没被测。
    #[test]
    fn hashed_key_destroys_directory_contiguity() {
        let (mut objs, n_dirs) = build_tree(false, 7);
        objs.sort_by_key(|o| key_of(o, Layout4::Hashed));
        // 数「目录在排序后被切成几段」：完全连续 = n_dirs 段，打散 = 远多于 n_dirs
        let mut runs = 0usize;
        let mut prev = usize::MAX;
        for o in &objs { if o.dir_id != prev { runs += 1; prev = o.dir_id; } }
        assert!(runs > n_dirs * 10,
                "哈希后目录只被切成 {runs} 段（目录数 {n_dirs}），没打散");
        // 对照：同一份数据按全路径排序，段数必须恰好等于目录数
        objs.sort_by_key(|o| key_of(o, Layout4::FullPath));
        let mut fp_runs = 0usize; let mut p = usize::MAX;
        for o in &objs { if o.dir_id != p { fp_runs += 1; p = o.dir_id; } }
        assert_eq!(fp_runs, n_dirs, "全路径排序后目录不连续，对照本身就不成立");
    }

    /// **改名只许改 path，不许动 inode 与 locality。**
    /// 这是 [decisions.md] D8 明写的语义：`locality_id` 改名时故意不更新。
    /// 动了它，「局部性缓慢退化」这个被测现象就不存在了。
    #[test]
    fn renames_change_only_the_path() {
        let (mut objs, _) = build_tree(false, 11);
        let before: Vec<(u64, u64)> = objs.iter().map(|o| (o.inode, o.locality)).collect();
        apply_renames(&mut objs, 500, 13);
        let after: Vec<(u64, u64)> = objs.iter().map(|o| (o.inode, o.locality)).collect();
        assert_eq!(before, after, "改名动了 inode 或 locality");
    }

    /// **改名必须保树形**：目录数不许变。
    /// 源码注释自己写着「直接把 from 改写成 to 是把两棵子树合并，目录数会被销毁，
    /// 结果会朝『改名越多越快』这个不可能的方向走」。这条把那个陷阱钉住。
    #[test]
    fn renames_preserve_the_number_of_directories() {
        let (mut objs, n_dirs) = build_tree(false, 17);
        for r in [100usize, 1000, 5000] {
            let mut o = objs.clone();
            apply_renames(&mut o, r, 19);
            let dirs: std::collections::HashSet<_> = o.iter().map(|x| x.path).collect();
            assert_eq!(dirs.len(), n_dirs, "改名 {r} 次后目录数从 {n_dirs} 变成 {}", dirs.len());
        }
        let _ = &mut objs;
    }

    /// `interleave` 必须真的改变创建顺序，否则那个自变量是假的——
    /// 而 E9 的结论恰恰只在 `interleave` 打开时才分得开三条臂。
    #[test]
    fn interleave_actually_changes_creation_order() {
        let (a, _) = build_tree(false, 23);
        let (b, _) = build_tree(true, 23);
        let pa: Vec<_> = a.iter().map(|o| o.path).collect();
        let pb: Vec<_> = b.iter().map(|o| o.path).collect();
        assert_ne!(pa, pb, "interleave 没有改变创建顺序，那个自变量是假的");
    }
}
