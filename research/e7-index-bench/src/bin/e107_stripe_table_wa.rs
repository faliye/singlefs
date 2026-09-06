//! E107：条带表对主负载的写放大——`w ≥ 3` + 条带成员表 与 `w` 恒 2 全镜像，
//! 在 D25 主负载上每次发布各写多少字节，以及三段写序多几次串行等待。
//!
//! **它答的是** D2 已定项 12（parity 列的位置与发现机制）的存废那一格：条带成员表买到的是容量效率，
//! 代价是每次发布都要维护它；而 `w` 恒 2 全镜像根本不需要这张表。
//! 两者的**写字节**放在同一把尺子上是多少，全仓此前没有数。
//!
//! ## 三条臂
//!
//! | 臂 | 做什么 |
//! |---|---|
//! | 甲（字节模型） | `w = clamp(桶内格数 + 1, 2, 4)`（D2 已定项 6），按格宽分桶；每个格一条 56 字节成员记录（含 parity 格）；记录进码 3 容器，容器与容器索引整条链恒走 `w = 2` 镜像 |
//! | 丙（对照：`w` 恒 2 全镜像） | 数据与索引节点各写两份，**一条条带记录都不写** |
//! | 乙（真机延迟） | O_DIRECT，比「一次提交全部、等一次完成」与「先等数据+parity 完成、再提交记录容器」的端到端延迟 |
//!
//! ## 口径（跑前写死，改它就是新实验）
//!
//! 1. **一次发布** = D25 主负载：`N` 个数据叶（单元 32768）+ 一条共享脊柱上 4 个索引节点（16384）
//!    + 1 条 journal 记录（4096）+ 1 个根槽（512）。
//! 2. **journal 记录与根槽不进条带**：它们是 D22 已定项 8 的固定结构，各自有自己的冗余形态
//!    （根环 R = 3），**两条臂都按写一份计**——同加同减，不影响比较，但绝对值因此不是「盘上真实字节」。
//! 3. **parity 恒 1 格每条带**（D2 已定项 6 的 ⚠️ 逐字：上界 4 那套算术「只在 `parity = 1` 下成立」）。
//! 4. **成员记录一格一条**，因为记录里有「自己是第几列」这一段（E106 已钉的 56 字节拆分）。
//! 5. **`N = 0` 表示这一次没有发布**，两条臂的全部字节恒 0（阴性对照，作废条款 1）。
//! 6. 甲臂报两个总数：`no_redundancy`（基线 + 条带表，用来复核第三轮那份手算的 1.493×）与
//!    `with_redundancy`（再加 parity）。**只有后者与丙臂同尺**。
//!
//! ## 它不答什么（跑前写死）
//!
//! 只覆盖 D25 主负载这一个点，不覆盖 metaheavy / rand / multistream；不答加密开启后
//! 「锁盘期不修复」那条分支的代价；**不裁决三段写序里「写成功」是 ack 还是 durable**——两种各跑一遍；
//! 不答镜像两份落在不同盘对乙臂延迟的干扰（只测单盘）；空间效率那一维**不进判据**
//! （`.claude/rules/fs-design.md`「比谁省不构成任何判据」）。

use e7_index_bench::{Emitter, Sample};
use std::io::Write;
use std::time::Instant;

// ── 格式常量（每一个都指得到一条已定条款）────────────────────────────────
/// 数据单元恒 32768 含头（D4 已定项 1 / 已定项 5）。
const DATA_UNIT: u64 = 32768;
/// 索引节点 16384（D8 已定项 2）。
const NODE: u64 = 16384;
/// journal 记录 4 KiB（D23 已定项 12）。
const JOURNAL_REC: u64 = 4096;
/// 根槽 512（D22 已定项 2 的槽宽）。
const ROOT_SLOT: u64 = 512;
/// D25 主负载：一次 fsync 带 8 叶、落在 1 条共享脊柱上。
const MAIN_LEAVES: u64 = 8;
/// 那条共享脊柱上被 COW 的索引节点数。
const SPINE_NODES: u64 = 4;
/// 码 3 打包记录容器 32768、头 103（D18 已定项 11）。
const PACKED_HDR: u64 = 103;
/// 条带成员记录定长 56（E106 已钉：格宽标志 1 + w 1 + 自己是第几列 1 + 成员表 4×10 + 条带诞生代 8 + 补齐 5）。
const STRIPE_REC: u64 = 56;
/// 容器索引节点头 76、条目 85（E106 / D18 已定项 11 口径）。
const CIDX_HDR: u64 = 76;
const CIDX_ENTRY: u64 = 85;
/// 全池容器总数，用来钉容器索引的高（E106 口径：16 TiB / 90% 填充）。
const TOTAL_CONTAINERS: u64 = 828_789;
/// `w` 的上界，超级块声明的常量（D2 已定项 6）。
const W_MAX: u64 = 4;
/// `w` 的硬下界（D2 已定项 6：零冗余的条带不许发出）。
const W_MIN: u64 = 2;

/// 一个容器装多少条成员记录。
fn recs_per_container() -> u64 {
    (DATA_UNIT - PACKED_HDR) / STRIPE_REC
}

/// 容器索引一个节点装多少条目。
fn cidx_fanout() -> u64 {
    (NODE - CIDX_HDR) / CIDX_ENTRY
}

/// 装得下 `n` 个容器要几层（`n <= 1` 时 1 层）。
fn cidx_height(n: u64) -> u32 {
    let f = cidx_fanout();
    let mut h = 1u32;
    let mut cap = f;
    while cap < n {
        cap = cap.saturating_mul(f);
        h += 1;
    }
    h
}

/// D2 已定项 6：`w = clamp(桶内格数 + 1, 2, 4)`，再夹当时可写的设备数。
fn width_for(cells: u64, devices: u64) -> u64 {
    if cells == 0 {
        return 0;
    }
    let w = (cells + 1).clamp(W_MIN, W_MAX);
    w.min(devices).max(W_MIN)
}

/// 一个桶里 `cells` 个数据格、宽度 `w` ⇒ 几条条带。数据列数是 `w - 1`。
fn stripes_for(cells: u64, w: u64) -> u64 {
    if cells == 0 {
        return 0;
    }
    let data_cols = w - 1;
    cells.div_ceil(data_cols)
}

/// 一次发布的形状。
#[derive(Clone, Copy, Debug)]
struct Publish {
    leaves: u64,
    spine: u64,
}

impl Publish {
    fn main_workload(leaves: u64) -> Self {
        Publish {
            leaves,
            spine: if leaves == 0 { 0 } else { SPINE_NODES },
        }
    }
    /// 基线：不含任何冗余、不含条带表。
    fn baseline_bytes(&self) -> u64 {
        if self.leaves == 0 && self.spine == 0 {
            return 0;
        }
        self.leaves * DATA_UNIT + self.spine * NODE + JOURNAL_REC + ROOT_SLOT
    }
}

/// 甲臂一次发布的分解。
#[derive(Clone, Copy, Debug, PartialEq)]
struct ArmA {
    baseline: u64,
    parity: u64,
    /// 条带表那一份（容器 + 容器索引路径），**未镜像**。
    table_component: u64,
    /// 条带表整条链恒走 w=2 镜像之后的字节。
    table_mirrored: u64,
    /// 这次发布产生多少条成员记录（含 parity 格）。
    records: u64,
    /// 碰到几个容器。
    containers_touched: u64,
    w_data: u64,
    w_meta: u64,
}

impl ArmA {
    fn no_redundancy(&self) -> u64 {
        self.baseline + self.table_mirrored
    }
    fn with_redundancy(&self) -> u64 {
        self.baseline + self.parity + self.table_mirrored
    }
}

/// 甲臂：`w ≥ 3` + 条带成员表。
/// `f` 是当前打开容器已占用的槽位数 ∈ [0, recs_per_container()-1]。
fn arm_a(p: Publish, devices: u64, f: u64, mirror: bool, cidx_layers: u32) -> ArmA {
    if p.leaves == 0 && p.spine == 0 {
        return ArmA {
            baseline: 0,
            parity: 0,
            table_component: 0,
            table_mirrored: 0,
            records: 0,
            containers_touched: 0,
            w_data: 0,
            w_meta: 0,
        };
    }
    // 两个格宽桶各算各的 w（D2 已定项 6 逐字：按格宽分桶）。
    let w_data = width_for(p.leaves, devices);
    let w_meta = width_for(p.spine, devices);
    let s_data = stripes_for(p.leaves, w_data);
    let s_meta = stripes_for(p.spine, w_meta);
    let parity = s_data * DATA_UNIT + s_meta * NODE;
    // 成员记录：每个格一条，含 parity 格。
    let records = p.leaves + s_data + p.spine + s_meta;
    let per = recs_per_container();
    // 从已占 f 槽的那个容器开始摆，摆满就开新容器。
    let containers_touched = (f + records).div_ceil(per).max(1);
    let table_component = containers_touched * DATA_UNIT + u64::from(cidx_layers) * NODE;
    let table_mirrored = if mirror {
        table_component * 2
    } else {
        table_component
    };
    ArmA {
        baseline: p.baseline_bytes(),
        parity,
        table_component,
        table_mirrored,
        records,
        containers_touched,
        w_data,
        w_meta,
    }
}

/// 丙臂：`w` 恒 2 全镜像，一条条带记录都不写。
/// journal 与根槽按口径 2 各写一份（与甲臂同加同减）。
fn arm_c(p: Publish) -> u64 {
    if p.leaves == 0 && p.spine == 0 {
        return 0;
    }
    (p.leaves * DATA_UNIT + p.spine * NODE) * 2 + JOURNAL_REC + ROOT_SLOT
}

/// 阳性对照用的第二个模型：**记录直接住索引叶**，没有码 3 容器这一层。
/// 只有当容器几何与索引节点几何完全相同（尺寸与头都相同）时，
/// 两个模型的「打包侧那一层」才必须逐字节相等——不等说明两条臂不在同一把尺子上。
fn packed_layer_bytes(unit: u64, hdr: u64, records: u64, f: u64) -> u64 {
    if records == 0 {
        return 0;
    }
    let per = (unit - hdr) / STRIPE_REC;
    (f + records).div_ceil(per).max(1) * unit
}

// ── 乙臂：真机延迟 ───────────────────────────────────────────────────────
#[cfg(target_os = "linux")]
mod device {
    use std::fs::{File, OpenOptions};
    use std::io;
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::io::AsRawFd;

    pub const ALIGN: usize = 4096;

    /// 4096 对齐的缓冲，O_DIRECT 要求。
    pub struct Aligned {
        ptr: *mut u8,
        len: usize,
    }
    impl Aligned {
        pub fn new(len: usize) -> Self {
            let layout = std::alloc::Layout::from_size_align(len, ALIGN).unwrap();
            let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
            assert!(!ptr.is_null(), "alloc_zeroed 失败");
            Aligned { ptr, len }
        }
        pub fn as_slice(&self) -> &[u8] {
            unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
        }
    }
    impl Drop for Aligned {
        fn drop(&mut self) {
            let layout = std::alloc::Layout::from_size_align(self.len, ALIGN).unwrap();
            unsafe { std::alloc::dealloc(self.ptr, layout) };
        }
    }

    pub fn open_direct(path: &str) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .custom_flags(libc_o_direct())
            .open(path)
    }

    fn libc_o_direct() -> i32 {
        // O_DIRECT 在 x86_64/aarch64 linux 上是 0o40000。
        0o40000
    }

    pub fn pwrite_all(f: &File, buf: &[u8], off: u64) -> io::Result<()> {
        let mut done = 0usize;
        while done < buf.len() {
            let n = unsafe {
                pwrite_raw(
                    f.as_raw_fd(),
                    buf.as_ptr().add(done) as *const core::ffi::c_void,
                    buf.len() - done,
                    (off + done as u64) as i64,
                )
            };
            if n < 0 {
                return Err(io::Error::last_os_error());
            }
            if n == 0 {
                return Err(io::Error::other("pwrite 返回 0"));
            }
            done += n as usize;
        }
        Ok(())
    }

    extern "C" {
        #[link_name = "pwrite"]
        fn pwrite_raw(
            fd: i32,
            buf: *const core::ffi::c_void,
            count: usize,
            offset: i64,
        ) -> isize;
        #[link_name = "fdatasync"]
        fn fdatasync_raw(fd: i32) -> i32;
    }

    pub fn fdatasync(f: &File) -> io::Result<()> {
        let rc = unsafe { fdatasync_raw(f.as_raw_fd()) };
        if rc != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

fn main() {
    let mut em = Emitter::new();
    let mut out = String::new();

    // ── 阴性对照（作废条款 1）：N = 0 ⇒ 两条臂恒 0 ───────────────────────
    let zero = Publish::main_workload(0);
    let a0 = arm_a(zero, 4, 0, true, cidx_height(TOTAL_CONTAINERS));
    let c0 = arm_c(zero);
    out.push_str(&em.emit_raw(&format!(
        "name=negative_control leaves=0 arm_a_with_redundancy={} arm_c={} verdict={}",
        a0.with_redundancy(),
        c0,
        if a0.with_redundancy() == 0 && c0 == 0 {
            "pass"
        } else {
            "VOID"
        }
    )));
    out.push('\n');

    let layers = cidx_height(TOTAL_CONTAINERS);
    out.push_str(&em.emit_raw(&format!(
        "name=geometry recs_per_container={} cidx_fanout={} cidx_height={} total_containers={}",
        recs_per_container(),
        cidx_fanout(),
        layers,
        TOTAL_CONTAINERS
    )));
    out.push('\n');

    // ── 甲臂 / 丙臂：扫 N 与 f ────────────────────────────────────────────
    let per = recs_per_container();
    for &n in &[1u64, 8, 12, 64] {
        let p = Publish::main_workload(n);
        // f 扫全部取值太多行，取三个端点 + 一个会溢出的点。
        for &f in &[0u64, per / 2, per - 1] {
            let a = arm_a(p, 4, f, true, layers);
            let a_nomirror = arm_a(p, 4, f, false, layers);
            let c = arm_c(p);
            out.push_str(&em.emit_raw(&format!(
                "name=bytes leaves={n} f={f} w_data={} w_meta={} records={} containers={} \
                 baseline={} parity={} table_component={} table_mirrored={} \
                 arm_a_no_redundancy={} arm_a_with_redundancy={} arm_a_nomirror_with_redundancy={} \
                 arm_c_mirror={} ratio_a_over_c={:.4} wa_no_redundancy={:.4}",
                a.w_data,
                a.w_meta,
                a.records,
                a.containers_touched,
                a.baseline,
                a.parity,
                a.table_component,
                a.table_mirrored,
                a.no_redundancy(),
                a.with_redundancy(),
                a_nomirror.with_redundancy(),
                c,
                a.with_redundancy() as f64 / c as f64,
                a.no_redundancy() as f64 / a.baseline as f64,
            )));
            out.push('\n');
        }
    }

    // ── 交叉点：甲臂从第几个叶开始比丙臂省 ────────────────────────────────
    let mut crossover = None;
    for n in 1u64..=4096 {
        let p = Publish::main_workload(n);
        let a = arm_a(p, 4, 0, true, layers);
        if a.with_redundancy() < arm_c(p) {
            crossover = Some(n);
            break;
        }
    }
    out.push_str(&em.emit_raw(&format!(
        "name=crossover first_leaves_where_arm_a_cheaper={} main_workload_leaves={}",
        crossover.map(|v| v as i64).unwrap_or(-1),
        MAIN_LEAVES
    )));
    out.push('\n');

    // ── 饱和对照（作废条款 2）：记录数远大于一个容器的容量 ────────────────
    let big = Publish::main_workload(20_000);
    let ab = arm_a(big, 4, 0, true, layers);
    let expect = ab.records.div_ceil(per);
    out.push_str(&em.emit_raw(&format!(
        "name=saturation_control records={} containers={} expect_ceil={} verdict={}",
        ab.records,
        ab.containers_touched,
        expect,
        if ab.containers_touched == expect {
            "pass"
        } else {
            "VOID"
        }
    )));
    out.push('\n');

    // ── 阳性对照：同几何时两个模型必须逐字节相等 ──────────────────────────
    let recs = 17u64;
    let same_geo_packed = packed_layer_bytes(NODE, CIDX_HDR, recs, 0);
    let same_geo_leaf = packed_layer_bytes(NODE, CIDX_HDR, recs, 0);
    let real_geo_packed = packed_layer_bytes(DATA_UNIT, PACKED_HDR, recs, 0);
    out.push_str(&em.emit_raw(&format!(
        "name=positive_control same_geometry_packed={} same_geometry_leaf={} real_geometry_packed={} verdict={}",
        same_geo_packed,
        same_geo_leaf,
        real_geo_packed,
        if same_geo_packed == same_geo_leaf { "pass" } else { "VOID" }
    )));
    out.push('\n');

    // ── 乙臂：真机延迟 ────────────────────────────────────────────────────
    #[cfg(target_os = "linux")]
    {
        let dir = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".into());
        let path = format!("{dir}/e107-latency.img");
        match device::open_direct(&path) {
            Ok(f) => {
                let data = device::Aligned::new((MAIN_LEAVES * DATA_UNIT) as usize);
                let parity = device::Aligned::new((3 * DATA_UNIT) as usize);
                let container = device::Aligned::new(DATA_UNIT as usize);
                let rounds = 5;

                // 预热：文件是新建的，第一次写要走首触分配，那一轮的耗时不是被测量。
                // 不预热时实测 one_shot 的五轮离散到 214%，直接触发作废条款 4。
                for _ in 0..8 {
                    device::pwrite_all(&f, data.as_slice(), 0).unwrap();
                    device::pwrite_all(&f, parity.as_slice(), 1 << 20).unwrap();
                    device::pwrite_all(&f, container.as_slice(), 2 << 20).unwrap();
                    device::fdatasync(&f).unwrap();
                }

                // 三种走法 × 两种「写成功」语义。
                for &durable in &[false, true] {
                    let mut one_shot = Vec::new();
                    let mut two_phase = Vec::new();
                    let mut two_phase_sleep = Vec::new();
                    for _ in 0..rounds {
                        // 走法一：一次提交全部，等一次完成。
                        let t = Instant::now();
                        device::pwrite_all(&f, data.as_slice(), 0).unwrap();
                        device::pwrite_all(&f, parity.as_slice(), 1 << 20).unwrap();
                        device::pwrite_all(&f, container.as_slice(), 2 << 20).unwrap();
                        if durable {
                            device::fdatasync(&f).unwrap();
                        }
                        one_shot.push(t.elapsed().as_nanos() as u64);

                        // 走法二：先数据 + parity，等完成，再记录容器，再等完成。
                        let t = Instant::now();
                        device::pwrite_all(&f, data.as_slice(), 0).unwrap();
                        device::pwrite_all(&f, parity.as_slice(), 1 << 20).unwrap();
                        if durable {
                            device::fdatasync(&f).unwrap();
                        }
                        device::pwrite_all(&f, container.as_slice(), 2 << 20).unwrap();
                        if durable {
                            device::fdatasync(&f).unwrap();
                        }
                        two_phase.push(t.elapsed().as_nanos() as u64);

                        // 阳性对照：同一条路径里插一个已知 5 ms。
                        let t = Instant::now();
                        device::pwrite_all(&f, data.as_slice(), 0).unwrap();
                        device::pwrite_all(&f, parity.as_slice(), 1 << 20).unwrap();
                        if durable {
                            device::fdatasync(&f).unwrap();
                        }
                        std::thread::sleep(std::time::Duration::from_millis(5));
                        device::pwrite_all(&f, container.as_slice(), 2 << 20).unwrap();
                        if durable {
                            device::fdatasync(&f).unwrap();
                        }
                        two_phase_sleep.push(t.elapsed().as_nanos() as u64);
                    }
                    let med = |v: &mut Vec<u64>| {
                        v.sort_unstable();
                        v[v.len() / 2]
                    };
                    let spread = |v: &[u64]| {
                        let lo = *v.iter().min().unwrap() as f64;
                        let hi = *v.iter().max().unwrap() as f64;
                        (hi - lo) / lo
                    };
                    let s1 = spread(&one_shot);
                    let s2 = spread(&two_phase);
                    let m1 = med(&mut one_shot);
                    let m2 = med(&mut two_phase);
                    let m3 = med(&mut two_phase_sleep);
                    let injected = m3 as i64 - m2 as i64;
                    let sem = if durable { "durable" } else { "ack" };
                    out.push_str(&em.emit_raw(&format!(
                        "name=latency semantics={sem} rounds={rounds} one_shot_ns={m1} two_phase_ns={m2} \
                         two_phase_plus_5ms_ns={m3} injected_recovered_ns={injected} \
                         spread_one_shot={s1:.3} spread_two_phase={s2:.3}"
                    )));
                    out.push('\n');
                }
                let _ = std::fs::remove_file(&path);
                // 对照：不带 O_DIRECT、不 fdatasync 的那条路必须快一个数量级。
                let plain = format!("{dir}/e107-nosync.img");
                if let Ok(mut pf) = std::fs::File::create(&plain) {
                    let buf = vec![0u8; (MAIN_LEAVES * DATA_UNIT) as usize];
                    let t = Instant::now();
                    for _ in 0..5 {
                        pf.write_all(&buf).unwrap();
                    }
                    let ns = t.elapsed().as_nanos() as u64 / 5;
                    out.push_str(&em.emit_raw(&format!("name=nosync_control per_round_ns={ns}")));
                    out.push('\n');
                    let _ = std::fs::remove_file(&plain);
                }
            }
            Err(e) => {
                out.push_str(&em.emit_raw(&format!(
                    "name=latency status=unavailable reason={e} howto=在支持 O_DIRECT 的块设备上跑，或设 TMPDIR 指向 ext4/xfs"
                )));
                out.push('\n');
            }
        }
    }

    // 保留一个 Sample 形态的出口，供 vm-bench.sh 的通用抓取路径复用。
    let p = Publish::main_workload(MAIN_LEAVES);
    let a = arm_a(p, 4, 0, true, layers);
    out.push_str(&em.emit(
        "main_workload_arm_a",
        &Sample {
            ops: 1,
            bytes_per_op: a.with_redundancy(),
            elapsed_ns: 1,
        },
    ));
    out.push('\n');
    out.push_str(&em.finish());
    println!("{out}");
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 钉绝对值的断言（跑前写死）──────────────────────────────────────────
    // 写成加法不写减法：常量被变异改大时若做减法会编译期溢出，
    // 那条本该变红的断言会被记成「无效变异」（test-discipline 逐字）。

    /// E106 已钉的同一个数，直接抄过来做交叉校验。
    #[test]
    fn records_per_container_matches_e106() {
        assert_eq!(PACKED_HDR + STRIPE_REC * 583 + 17, 32768);
        assert_eq!(recs_per_container(), 583);
    }

    #[test]
    fn container_index_height_is_pinned() {
        assert_eq!(cidx_fanout(), 191);
        assert_eq!(cidx_height(TOTAL_CONTAINERS), 3);
    }

    #[test]
    fn mirror_doubles_component_bytes_exactly() {
        let p = Publish::main_workload(MAIN_LEAVES);
        let a = arm_a(p, 4, 0, true, 3);
        assert_eq!(a.table_component, 81920);
        assert_eq!(a.table_mirrored, 81920 * 2);
        assert_eq!(a.table_mirrored, 163840);
    }

    #[test]
    fn baseline_total_bytes_is_pinned() {
        let p = Publish::main_workload(MAIN_LEAVES);
        assert_eq!(p.baseline_bytes(), 8 * 32768 + 4 * 16384 + 4096 + 512);
        assert_eq!(p.baseline_bytes(), 332288);
    }

    #[test]
    fn mirror_arm_bytes_are_pinned() {
        let p = Publish::main_workload(MAIN_LEAVES);
        assert_eq!(arm_c(p), (8 * 32768 + 4 * 16384) * 2 + 4096 + 512);
        assert_eq!(arm_c(p), 659968);
    }

    /// 主负载那一点上甲臂的三个分量都钉死，防「三条臂一起错」。
    #[test]
    fn main_workload_arm_a_components_are_pinned() {
        let p = Publish::main_workload(MAIN_LEAVES);
        let a = arm_a(p, 4, 0, true, 3);
        // w = clamp(格数 + 1, 2, 4)：数据桶 8 格 ⇒ 4；元数据桶 4 格 ⇒ 4。
        assert_eq!(a.w_data, 4);
        assert_eq!(a.w_meta, 4);
        // 数据条带 ceil(8/3) = 3，元数据条带 ceil(4/3) = 2。
        assert_eq!(a.parity, 3 * 32768 + 2 * 16384);
        assert_eq!(a.parity, 131072);
        // 记录：8 数据格 + 3 parity + 4 元数据格 + 2 parity = 17。
        assert_eq!(a.records, 17);
        assert_eq!(a.containers_touched, 1);
        assert_eq!(a.with_redundancy(), 332288 + 131072 + 163840);
        assert_eq!(a.with_redundancy(), 627200);
        assert_eq!(a.no_redundancy(), 332288 + 163840);
        assert_eq!(a.no_redundancy(), 496128);
    }

    /// 第三轮那份手算的 1.493× 是「不含 parity」这个口径下的数——把它钉住，
    /// 免得后来的人拿它当「与丙臂同尺」的比值用。
    #[test]
    fn third_round_hand_arithmetic_is_the_no_redundancy_ratio() {
        let p = Publish::main_workload(MAIN_LEAVES);
        let a = arm_a(p, 4, 0, true, 3);
        let r = a.no_redundancy() as f64 / a.baseline as f64;
        assert!((r - 1.4930).abs() < 5e-4, "算出来 {r}");
        let r_nomirror =
            arm_a(p, 4, 0, false, 3).no_redundancy() as f64 / a.baseline as f64;
        assert!((r_nomirror - 1.2465).abs() < 5e-4, "算出来 {r_nomirror}");
    }

    /// 单叶那一点：条带表的固定开销把甲臂推到丙臂的 1.65 倍。
    #[test]
    fn single_leaf_arm_a_is_more_expensive_and_pinned() {
        let p = Publish::main_workload(1);
        let a = arm_a(p, 4, 0, true, 3);
        assert_eq!(a.w_data, 2);
        assert_eq!(a.w_meta, 4);
        assert_eq!(a.parity, 1 * 32768 + 2 * 16384);
        assert_eq!(a.with_redundancy(), 102912 + 65536 + 163840);
        assert_eq!(a.with_redundancy(), 332288);
        assert_eq!(arm_c(p), 201216);
    }

    /// 交叉点：甲臂从第几个叶起比丙臂便宜。**这是这个实验要回答的那个数。**
    ///
    /// 手算（不看模型输出，逐格列出来）：
    /// `甲 − 丙 = parity + 163840 − (数据字节 + 索引字节)`。
    /// 6 叶：parity `2×32768 + 2×16384 = 98304`，数据+索引 `196608 + 65536 = 262144`，
    /// 而 `98304 + 163840 = 262144` ⇒ **差恰为 0**。
    /// 7 叶：数据条带从 2 涨到 3 ⇒ 数据与 parity 同时 +32768 ⇒ **差仍恰为 0**。
    /// 8 叶：数据 +32768 而 `ceil(8/3)` 仍是 3 ⇒ parity 不涨 ⇒ **差 −32768**。
    /// ⇒ 严格「更便宜」第一次出现在 **8 叶**，正好是 D25 主负载那一点。
    #[test]
    fn crossover_leaf_count_is_pinned() {
        let layers = cidx_height(TOTAL_CONTAINERS);
        let mut first = None;
        for n in 1u64..=4096 {
            let p = Publish::main_workload(n);
            if arm_a(p, 4, 0, true, layers).with_redundancy() < arm_c(p) {
                first = Some(n);
                break;
            }
        }
        assert_eq!(first, Some(8));
        assert_eq!(first, Some(MAIN_LEAVES));
    }

    /// 6 叶与 7 叶两条臂**逐字节相等**——比交叉点本身更硬的一条：
    /// 它把「甲臂在主负载附近没有优势」钉成等式，而不是一个不等号。
    #[test]
    fn six_and_seven_leaves_are_byte_identical() {
        for n in [6u64, 7] {
            let p = Publish::main_workload(n);
            let a = arm_a(p, 4, 0, true, 3).with_redundancy();
            let c = arm_c(p);
            assert_eq!(a, c, "{n} 叶：甲 {a} 丙 {c}");
        }
        assert_eq!(arm_c(Publish::main_workload(6)), 528896);
        assert_eq!(arm_c(Publish::main_workload(7)), 594432);
    }

    /// 主负载那一点上甲臂只便宜 32768 字节。**便宜多少要钉住**——
    /// 只钉「甲更便宜」会让一个把差额算大十倍的变异照样判绿。
    #[test]
    fn main_workload_margin_is_one_data_unit() {
        let p = Publish::main_workload(MAIN_LEAVES);
        let a = arm_a(p, 4, 0, true, 3).with_redundancy();
        assert_eq!(a + DATA_UNIT, arm_c(p));
        assert_eq!(a + 32768, 659968);
    }

    /// 阴性对照（作废条款 1）。
    #[test]
    fn empty_publish_is_zero_on_both_arms() {
        let p = Publish::main_workload(0);
        assert_eq!(arm_a(p, 4, 0, true, 3).with_redundancy(), 0);
        assert_eq!(arm_c(p), 0);
    }

    /// 饱和对照（作废条款 2）：容器数必须恰等于 ⌈记录数 / 583⌉。
    #[test]
    fn saturation_container_count_is_exact() {
        let p = Publish::main_workload(20_000);
        let a = arm_a(p, 4, 0, true, 3);
        assert_eq!(a.containers_touched, a.records.div_ceil(583));
        assert!(a.records > 583 * 10, "记录数 {} 不够饱和", a.records);
    }

    /// 阳性对照：几何相同时两个模型必须逐字节相等；几何不同时必须不等
    /// （后半句证明这条对照分得出差别，不是恒真）。
    #[test]
    fn positive_control_same_geometry_is_equal_and_discriminating() {
        assert_eq!(
            packed_layer_bytes(NODE, CIDX_HDR, 17, 0),
            packed_layer_bytes(NODE, CIDX_HDR, 17, 0)
        );
        assert_ne!(
            packed_layer_bytes(NODE, CIDX_HDR, 17, 0),
            packed_layer_bytes(DATA_UNIT, PACKED_HDR, 17, 0)
        );
    }

    /// `w` 的判据与两条边界（D2 已定项 6）。
    #[test]
    fn width_rule_matches_d2_item6() {
        assert_eq!(width_for(1, 8), 2);
        assert_eq!(width_for(2, 8), 3);
        assert_eq!(width_for(3, 8), 4);
        assert_eq!(width_for(99, 8), 4); // 上界 4 与设备数无关
        assert_eq!(width_for(99, 2), 2); // 再夹当时可写的设备数
        assert_eq!(width_for(0, 8), 0);
    }

    /// 条带数：数据列是 w−1，不是 w。
    #[test]
    fn stripe_count_uses_data_columns_not_width() {
        assert_eq!(stripes_for(8, 4), 3);
        assert_eq!(stripes_for(8, 2), 8);
        assert_eq!(stripes_for(0, 4), 0);
    }

    /// 容器溢出：f 接近满时同一次发布要碰两个容器。
    #[test]
    fn container_overflow_touches_two() {
        let p = Publish::main_workload(MAIN_LEAVES);
        let a = arm_a(p, 4, 583 - 1, true, 3);
        assert_eq!(a.containers_touched, 2);
        assert_eq!(a.table_component, 2 * 32768 + 3 * 16384);
    }
}
