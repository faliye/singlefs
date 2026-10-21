//! m3-close-c656-r3 攻方原型（只在仓副本里编、经内存包装直接执行编出来的二进制；不调 checker 档测试）。
//! 历史与崩溃态的取法照 m3-close-c656-r2 攻方原型（`research/prompts/m3-close-c656-r2-opus-model/probe/src/main.rs`）：
//! 崩溃态走崩溃点重放同一个计划（`visit_crash_state_range_of_the_read_back_flow_with_slices`，副本里改 pub），展开取 `full_expansion`
//! （每段任意真子集、原地覆写带撕裂镜像）。这一份多量「强制新 fsid」三种落法：
//!   生成：每个崩溃态上再跑一次乙写序的 mkfs，fsid 取又一个新的（`FSID_REGENERATED`），看恢复与池级 checker；
//!   拒：读每盘 [0, 根环基址) 里自证得过的系统配置，fsid 前 8 字节与要用的相同就拒（`sysconf_guard`）；
//!   单元头：逐槽扫单元区，按 checker 认单元写序的同一种写法认头（头校验和 + fsid 前 8 字节），数实例代号 > 0 与 = 0 的个数（`unit_scan`）。
//! 另有：aba / aba-used 两份历史（池 A 用 fsid X → 池 B 用 fsid Y 盖上 → 新 mkfs 又用 X）、drift（两次崩溃让两盘系统配置世代号差 2）、
//! bytes（同参数两次 mkfs、换 fsid 两次 mkfs 各差多少扇区）。流与历史都在这里自己造，不从崩溃枚举用例拷。
//! 用法：
//!   c656-r3-attack-probe count <历史> <写序>
//!   c656-r3-attack-probe run <历史> <写序> [起] [止]
//!   c656-r3-attack-probe final <历史> <写序>
//!   c656-r3-attack-probe handbuilt <历史> <写序> <落了的写下标,逗号分隔>
//!   c656-r3-attack-probe drift <轮数>
//!   c656-r3-attack-probe bytes
//!   历史：same、prefix、distinct、mixed-origin（同 r2）；aba（A 用 X、u=5 → B 用 Y 只 mkfs → 新池用 X）；aba-used（B 另用 u=3）
//!   写序：today、yi、yi-barrier-d0（同 r2 的补丁）
#![allow(dead_code, clippy::all)]

#[path = "../../singlefs-harness/tests/common_admission_pool/mod.rs"]
mod common_admission_pool;

use common_admission_pool::plain_devices_on;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion, no_segment_slices,
    visit_crash_state_range_of_the_read_back_flow_with_slices,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, make_filesystem_with_order, MakeFilesystemOutput, MakeFilesystemParameters,
    MkfsOrderForProbe,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome};
use singlefs_core::root_ring::{region_start, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableSizes};
use singlefs_core::transaction::{acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter};
use singlefs_format::{JOURNAL_RING_START_SLOT, SLOT_BYTES};
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::collections::BTreeMap;
use std::sync::Mutex;

/// 一次调用不超过 10⁶ 态（派发定义第 3b 步）。
const STATE_LIMIT_PER_RUN: u64 = 1_000_000;
const JOURNAL_RING_BYTES: u64 = 6 << 20;
const DEVICE_SLOTS: u64 = 1408 + 1007 + 384;
const DEVICE_BYTES: u64 = DEVICE_SLOTS * 16384;
const FSID_NEW: [u8; 16] = *b"c656-r2-new-pool";
/// 与新池 fsid 前 8 字节相同（单元头、journal 记录只带这 8 字节），后 8 字节不同。
const FSID_PREFIX: [u8; 16] = *b"c656-r2-old-pool";
/// 前 8 字节就与新池不同；aba 历史里是池 A 与新池共用的 X。
const FSID_DISTINCT: [u8; 16] = *b"OLDPOOL!distinct";
const FSID_Q: [u8; 16] = *b"QQPOOL!!other-q!";
/// aba 历史里夹在中间的池 B（Y）。
const FSID_B: [u8; 16] = *b"BBPOOL!!between!";
/// 「生成」那一种落法下重跑 mkfs 拿到的又一个新 fsid：前 8 字节与上面每一个都不同。
const FSID_REGENERATED: [u8; 16] = *b"R3REGEN!fresh-n2";
const WRITE_TIME: u64 = 1_788_000_000;
/// 单元区起点（槽）：journal 环起点 + 环长（这台小盘的几何；history() 里与 mkfs 交回的起点核一次）。
const UNIT_AREA_START_SLOT: u64 = JOURNAL_RING_START_SLOT + JOURNAL_RING_BYTES / SLOT_BYTES;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn threads() -> u64 {
    std::env::var("SINGLEFS_CRASH_POINT_REPLAY_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(1).max(1)
}

fn variant(debug: &str) -> String {
    debug.split(|c: char| c == '(' || c == '{' || c == ' ').next().unwrap_or("").to_string()
}

/// 错误的 Debug 串取到第一个空格或花括号为止（`Open(TreeTableHasNoEntries)` 整个留下）。
fn short_debug(debug: &str) -> String {
    debug.split(|c: char| c == '{' || c == ' ').next().unwrap_or("").trim_end_matches(',').to_string()
}

fn parameters(fsid: [u8; 16], io_min: u32) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: fsid,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: io_min,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(io_min),
            journal_ring_bytes: JOURNAL_RING_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn fresh_device() -> SparseBlockDevice {
    SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512))
}

fn clone_device(device: &SparseBlockDevice) -> SparseBlockDevice {
    let mut copy = fresh_device();
    copy.image = device.image.clone();
    copy
}

fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool { devices: devices.iter().map(|(i, d)| (*i, d.image.clone())).collect(), device_size_in_bytes: DEVICE_BYTES }
}

fn materialize(base: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let mut pool = base.clone();
    for (write, is_persisted) in writes.iter().zip(persisted) {
        if *is_persisted {
            pool.apply_writes(std::slice::from_ref(write));
        }
    }
    pool
}

fn content(seed: u64) -> Vec<u8> {
    (0..3000u64).map(|i| u8::try_from((i * 137 + seed * 11 + 3) % 251).expect("<256")).collect()
}

/// mkfs 之后的用户动作：取号 + 暖机 + 第一个文件 + (u − 2) 次覆盖写。交回每一版的内容与最后一版的根 (实例, txg)。
fn use_pool(devices: &mut Devices, parameters: &MakeFilesystemParameters, genesis: &MakeFilesystemOutput, usage: u32, seed: u64) -> (Vec<Vec<u8>>, (u32, u64)) {
    let mut allocator = allocator_after_make_filesystem(parameters, devices, genesis);
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let first = content(seed);
    let mut previous = publish_first_file(&mut writer, &mut allocator, warmed.roots.last().expect("两代根"), FirstFile { content: &first, write_time_seconds: WRITE_TIME }, instance, &warmed.last_record_bytes).expect("第一个文件");
    let mut versions = vec![first];
    for k in 0..usage - 2 {
        let c = content(seed + 1 + u64::from(k));
        previous = publish_overwrite(&mut writer, &mut allocator, &previous, FirstFile { content: &c, write_time_seconds: WRITE_TIME + 1 + u64::from(k) }, instance).expect("覆盖写");
        versions.push(c);
    }
    (versions, (previous.root.instance.0, previous.root.checkpoint_txg.0))
}

/// 旧池：mkfs（今天的写序）+ use_pool。
fn make_and_use(devices: &mut Devices, parameters: &MakeFilesystemParameters, usage: u32, seed: u64) -> (Vec<Vec<u8>>, (u32, u64)) {
    let genesis = make_filesystem(parameters, devices).expect("mkfs");
    assert_eq!(genesis.unit_area_start.slot().0, UNIT_AREA_START_SLOT, "单元区起点与原型算的不同");
    use_pool(devices, parameters, &genesis, usage, seed)
}

/// (参数, 各版内容, 最后一版的根)。只做过 mkfs 的池各版内容是空表、根是 (0, 0)。
type PoolSpec = (MakeFilesystemParameters, Vec<Vec<u8>>, (u32, u64));

struct History {
    name: String,
    order_name: String,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    recorded: usize,
    /// 盘上那个旧池（mkfs 之前盘上可挂的那个池）。
    old: PoolSpec,
    /// 盘 1 来自另一个池时（只 mixed-origin 有）。
    other: Option<PoolSpec>,
    /// 旧池之前的池（只 aba、aba-used 有）：它的系统配置与根已被旧池的 mkfs 清掉，单元区里的单元还在。
    ancestor: Option<PoolSpec>,
    new_parameters: MakeFilesystemParameters,
}

fn order_of(name: &str) -> MkfsOrderForProbe {
    match name {
        "today" => MkfsOrderForProbe::Today,
        "yi" => MkfsOrderForProbe::InvalidateFirst,
        "yi-unitclear" => MkfsOrderForProbe::InvalidateFirstThenClearUnitArea,
        "yi-unitclear-early" => MkfsOrderForProbe::ClearUnitAreaTogetherWithInvalidation,
        "yi-barrier-d0" => MkfsOrderForProbe::InvalidateFirstBarrierOnlyOnTheFirstDevice,
        other => panic!("未知写序 {other}"),
    }
}

fn history(name: &str, order_name: &str) -> History {
    let order = order_of(order_name);
    let mut new_parameters = parameters(FSID_NEW, 512);
    let (devices, old, other, ancestor) = match name {
        "same" | "prefix" | "distinct" => {
            let fsid = match name { "same" => FSID_NEW, "prefix" => FSID_PREFIX, _ => FSID_DISTINCT };
            let old_parameters = parameters(fsid, 512);
            let mut old: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
            let (versions, last) = make_and_use(&mut old, &old_parameters, 5, 7);
            (old, (old_parameters, versions, last), None, None)
        }
        "mixed-origin" => {
            let p = parameters(FSID_DISTINCT, 512);
            let mut pair_p: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
            let (p_versions, p_last) = make_and_use(&mut pair_p, &p, 5, 7);
            let q = parameters(FSID_Q, 65536);
            let mut pair_q: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
            let (q_versions, q_last) = make_and_use(&mut pair_q, &q, 5, 70);
            let devices: Devices = vec![(DeviceIdentity(0), clone_device(&pair_p[0].1)), (DeviceIdentity(1), clone_device(&pair_q[1].1))];
            (devices, (p, p_versions, p_last), Some((q, q_versions, q_last)), None)
        }
        // 池 A 用 X 用过（u = 5）→ 池 B 用 Y 在同两块盘上 mkfs（今天的写序，全落）、aba-used 另用 u = 3 → 新 mkfs 又用 X。
        "aba" | "aba-used" => {
            let a = parameters(FSID_DISTINCT, 512);
            let mut devices: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
            let (a_versions, a_last) = make_and_use(&mut devices, &a, 5, 7);
            let b = parameters(FSID_B, 512);
            let (b_versions, b_last) = if name == "aba-used" {
                make_and_use(&mut devices, &b, 3, 40)
            } else {
                make_filesystem(&b, &mut devices).expect("池 B 的 mkfs");
                (Vec::new(), (0, 0))
            };
            new_parameters = parameters(FSID_DISTINCT, 512);
            (devices, (b, b_versions, b_last), None, Some((a, a_versions, a_last)))
        }
        other => panic!("未知历史 {other}"),
    };
    let base = image_of(&devices);
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = devices
        .iter()
        .map(|(i, d)| (*i, RecordingBlockDevice::with_shared_stream(*i, clone_device(d), stream.clone())))
        .collect();
    make_filesystem_with_order(&new_parameters, &mut recorded, order).expect("新 mkfs");
    let fixed = FixedGeometry {
        fixed_structure_slot_spacing: new_parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: new_parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: new_parameters.geometry.root_ring_slots_per_region,
    };
    let (writes, segments) = writes_and_segments(&stream.retained_operations(), &fixed);
    let recorded_count = writes.len();
    History { name: name.into(), order_name: order_name.into(), base, writes, segments, recorded: recorded_count, old, other, ancestor, new_parameters }
}

fn content_class(h: &History, root: (u32, u64), bytes: &[u8]) -> String {
    let classify = |spec: &PoolSpec, tag: &str| -> Option<String> {
        let (_, versions, last_root) = spec;
        let last = versions.last()?;
        if bytes == last.as_slice() && root == *last_root {
            Some(format!("{tag}_last_same_root"))
        } else if bytes == last.as_slice() {
            Some(format!("{tag}_last_other_root"))
        } else if versions.iter().any(|c| c.as_slice() == bytes) {
            Some(format!("{tag}_older_version"))
        } else {
            None
        }
    };
    for (spec, tag) in [(Some(&h.old), "old"), (h.other.as_ref(), "other_old"), (h.ancestor.as_ref(), "ancestor")] {
        if let Some(spec) = spec {
            if let Some(c) = classify(spec, tag) {
                return c;
            }
        }
    }
    "other_content".into()
}

fn recover_label(h: &History, pool: &MemoryPool) -> String {
    match recover(pool, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, content } => format!("FileRead(i{}t{}):{}", root.0 .0, root.1 .0, content_class(h, (root.0 .0, root.1 .0), &content)),
        RecoveryOutcome::NoFile { root } => format!("NoFile(i{}t{})", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { failure, .. } => format!("Failed:{}", variant(&format!("{failure:?}"))),
    }
}

fn reds(image: &MemoryPool) -> Vec<String> {
    let mut names: Vec<String> = check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| matches!(verdict, InvariantVerdict::Violated(_)).then(|| invariant.to_string()))
        .collect();
    names.sort();
    names.dedup();
    names
}

/// 「拒」那一种落法的守卫（原型里的最强形，同 r2）：每块盘 [0, 根环基址) 里逐 4096 字节试解一份系统配置，
/// 自证得过的收下它的 fsid 前 8 字节；有一份与要用的 fsid 前 8 字节相同就拒。交回 (拒不拒, 找到几种前 8 字节)。
fn sysconf_guard(pool: &MemoryPool, fsid: &[u8; 16]) -> (bool, usize) {
    let base = region_start(0).0;
    let mut found: std::collections::BTreeSet<[u8; 8]> = std::collections::BTreeSet::new();
    for device in pool.devices.keys() {
        let mut offset = 0;
        while offset + 4096 <= base {
            if let Some(bytes) = PoolReader::read(pool, *device, DeviceOffsetInBytes(offset), 4096) {
                if bytes.iter().any(|b| *b != 0) {
                    if let Ok(c) = SystemConfiguration::parse_slot(&bytes) {
                        let mut low = [0u8; 8];
                        low.copy_from_slice(&c.immutable.filesystem_identifier[..8]);
                        found.insert(low);
                    }
                }
            }
            offset += 4096;
        }
    }
    (found.iter().any(|low| low[..] == fsid[..8]), found.len())
}

/// 头校验和（与 checker `checksum_field_holds` 同一种写法：罩 [0, cover_end)、校验和字段 32 字节、CRC-32C 放前 4 字节、其余 0）。
fn header_checksum_holds(bytes: &[u8], cover_end: usize, field_offset: usize) -> bool {
    let width = 32;
    if bytes.len() < cover_end || field_offset + width > cover_end {
        return false;
    }
    let mut covered = bytes[..cover_end].to_vec();
    covered[field_offset..field_offset + width].fill(0);
    let expected = crc32_castagnoli(&covered).to_le_bytes();
    let stored = &bytes[field_offset..field_offset + width];
    stored[..4] == expected && stored[4..].iter().all(|b| *b == 0)
}

/// 照 checker 认单元写序的写法（`crates/singlefs-checker/src/walk.rs` 的 `unit_write_order_instance`）认一个单元头：
/// 魔数、类标签定头长与 fsid / 实例代号的偏移、头校验和过、fsid 前 8 字节相同 ⇒ 交回写序里的实例代号。
fn unit_header_instance(bytes: &[u8], fsid_low: u64) -> Option<u32> {
    if bytes.len() < 52 || &bytes[..4] != b"SFSU" {
        return None;
    }
    let (header_end, fsid_offset, instance_offset) = match bytes[6] {
        1 => (105, 83, 91),
        2 => {
            let key_span = 2 * usize::from(bytes[51]);
            (86 + key_span, 60 + key_span, 68 + key_span)
        }
        3 => (107, 81, 93),
        _ => return None,
    };
    if header_end > bytes.len() || !header_checksum_holds(bytes, header_end, 10) {
        return None;
    }
    let fsid_in_header = u64::from_le_bytes(bytes[fsid_offset..fsid_offset + 8].try_into().expect("8"));
    (fsid_in_header == fsid_low).then(|| u32::from_le_bytes(bytes[instance_offset..instance_offset + 4].try_into().expect("4")))
}

/// 「单元头」那一种落法的守卫要读的东西：逐槽扫两块盘的单元区，fsid 前 8 字节与 `fsid` 相同的单元头里实例代号 > 0 的个数与 = 0 的个数。
fn unit_scan(pool: &MemoryPool, fsid: &[u8; 16]) -> (usize, usize) {
    let low = u64::from_le_bytes(fsid[..8].try_into().expect("8"));
    let (mut high, mut zero) = (0, 0);
    for device in pool.devices.keys() {
        for slot in UNIT_AREA_START_SLOT..DEVICE_BYTES / SLOT_BYTES {
            if let Some(bytes) = PoolReader::read(pool, *device, DeviceOffsetInBytes(slot * SLOT_BYTES), 4096) {
                match unit_header_instance(&bytes, low) {
                    Some(0) => zero += 1,
                    Some(_) => high += 1,
                    None => {}
                }
            }
        }
    }
    (high, zero)
}

/// 「生成」那一种落法下用户在这个态上再跑一次 mkfs（乙写序、全落），fsid 是又一个新的：交回 恢复|池级 checker 红。
fn regenerated_rerun(h: &History, pool: &MemoryPool) -> String {
    let p = MakeFilesystemParameters { filesystem_identifier: FSID_REGENERATED, ..h.new_parameters };
    let mut devices: Devices = plain_devices_on(pool);
    match make_filesystem_with_order(&p, &mut devices, MkfsOrderForProbe::InvalidateFirst) {
        Ok(_) => {
            let after = image_of(&devices);
            format!("{}|{:?}", recover_label(h, &after), reds(&after))
        }
        Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
    }
}

/// 三种落法在这个态上各给什么：生成（再跑 mkfs 的结局）、拒（守卫对新池 fsid 与旧池 fsid 拒不拒）、单元头（两个 fsid 的实例代号 >0 / =0 个数）。
fn landings(h: &History, pool: &MemoryPool) -> String {
    if std::env::var("PROBE_LANDINGS").map_or(false, |v| v == "0") {
        return "landings_not_tried".into();
    }
    let new = h.new_parameters.filesystem_identifier;
    let old = h.old.0.filesystem_identifier;
    let (refuse_new, found_new) = sysconf_guard(pool, &new);
    let (refuse_old, _) = sysconf_guard(pool, &old);
    let (new_high, new_zero) = unit_scan(pool, &new);
    let (old_high, old_zero) = unit_scan(pool, &old);
    format!(
        "regen_rerun={} sysconf_guard[new]={}/{found_new} sysconf_guard[old]={} unit_scan[new]={new_high}/{new_zero} unit_scan[old]={old_high}/{old_zero}",
        regenerated_rerun(h, pool),
        if refuse_new { "refuse" } else { "allow" },
        if refuse_old { "refuse" } else { "allow" },
    )
}

/// 一个状态的写落法：第一个没落的写在第几段（全落是 `all`）、没落的写、落了的撕裂镜像（撕裂镜像排在录制的写之后）。
fn describe_cut(h: &History, all_writes: &[RetainedWrite], persisted: &[bool]) -> (String, String, bool) {
    let first_withheld = (0..h.recorded).find(|i| !persisted[*i]);
    let phase = match first_withheld {
        Some(w) => format!("seg{}", h.segments.iter().position(|s| s.contains(&w)).unwrap_or(h.segments.len())),
        None => "all".to_string(),
    };
    let withheld: Vec<String> = (0..h.recorded).filter(|i| !persisted[*i]).map(|i| format!("{i}:{}@d{}", h.writes[i].kind.name(), h.writes[i].device.0)).collect();
    let torn: Vec<String> = (h.recorded..all_writes.len()).filter(|i| persisted[*i]).map(|i| format!("{i}:{}@d{}o{}", all_writes[i].kind.name(), all_writes[i].device.0, all_writes[i].offset.0)).collect();
    let torn_any = !torn.is_empty();
    (phase, format!("withheld=[{}] torn_images_landed=[{}]", withheld.join(","), torn.join(",")), torn_any)
}

fn spec_of_fsid<'a>(h: &'a History, fsid: &[u8; 16], generation: u64) -> Option<(&'static str, &'a MakeFilesystemParameters)> {
    if *fsid == h.new_parameters.filesystem_identifier && !(h.old.0.filesystem_identifier == *fsid && generation != 1) {
        Some(("new", &h.new_parameters))
    } else if *fsid == h.old.0.filesystem_identifier {
        Some(("old", &h.old.0))
    } else if let Some((q, _, _)) = &h.other {
        (*fsid == q.filesystem_identifier).then_some(("other_old", q))
    } else {
        None
    }
}

fn read_only_label(pool: &MemoryPool) -> String {
    match mount_read_only(pool) {
        Ok(m) => format!("Ok(i{}t{})", m.effective_root.instance.0, m.effective_root.checkpoint_txg.0),
        Err(e) => format!("Err:{}", short_debug(&format!("{e:?}"))),
    }
}

fn writable_label(pool: &MemoryPool, p: &MakeFilesystemParameters) -> String {
    let mut devices: Devices = plain_devices_on(pool);
    match mount_writable(p, &mut devices) {
        Ok(m) => format!("Ok(i{}t{})", m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0),
        Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
    }
}

fn judge(h: &History, pool: &MemoryPool, all_writes: &[RetainedWrite], persisted: &[bool]) -> (String, String) {
    let (phase, cut, torn) = describe_cut(h, all_writes, persisted);
    let (which, chosen_parameters, root) = match choose_system_configuration(pool) {
        Ok(c) => {
            let fsid = c.immutable.filesystem_identifier;
            let spacing = c.immutable.sizes.fixed_structure_slot_spacing;
            let generation = c.quantities.slot_generation;
            let (tag, p) = match spec_of_fsid(h, &fsid, generation) {
                Some((tag, p)) => (tag, Some(p)),
                None => ("unknown", None),
            };
            let root = choose_root(pool, &c).map_or("none".to_string(), |r| format!("i{}t{}", r.instance.0, r.checkpoint_txg.0));
            (format!("{tag}@{spacing}g{generation}"), p, root)
        }
        Err(e) => (format!("Err:{}", variant(&format!("{e:?}"))), None, "-".to_string()),
    };
    let recovered = recover_label(h, pool);
    let raw_red = reds(pool);
    let read_only = read_only_label(pool);
    let writable = chosen_parameters.map_or("not_tried".to_string(), |p| writable_label(pool, p));
    let old_intact = |tag: &str, spec: &PoolSpec| -> bool {
        if spec.1.is_empty() { recovered == "NoFile(i0t0)" } else { recovered.ends_with(&format!(":{tag}_last_same_root")) }
    };
    let verdict = if which.starts_with("new@") {
        if root == "i0t0" && recovered == "NoFile(i0t0)" {
            "new_complete".to_string()
        } else if root == "none" {
            "refused_new_system_configuration_without_a_root".to_string()
        } else {
            format!("MIXTURE_OR_BROKEN_NEW(root={root})")
        }
    } else if which.starts_with("old@") {
        if old_intact("old", &h.old) { "old_readback_intact".to_string() } else { "OLD_READBACK_NOT_INTACT".to_string() }
    } else if which.starts_with("other_old@") {
        if old_intact("other_old", h.other.as_ref().expect("other")) { "other_old_readback_intact".to_string() } else { "OTHER_OLD_READBACK_NOT_INTACT".to_string() }
    } else if which.starts_with("Err:") {
        "refused".to_string()
    } else {
        format!("UNKNOWN_POOL({which})")
    };
    let key = format!("phase={phase} torn={torn} choice={which} root={root} recover={recovered} raw_red={raw_red:?} read_only={read_only} writable={writable} verdict={verdict} {}", landings(h, pool));
    (key, cut)
}

#[derive(Default)]
struct Tally {
    states: u64,
    signatures: BTreeMap<String, u64>,
    examples: BTreeMap<String, (u64, String)>,
}

fn generations_by_device(pool: &MemoryPool, spacing: u64) -> Vec<String> {
    pool.devices
        .keys()
        .map(|device| {
            let gens: Vec<String> = (0..2u64)
                .map(|slot| {
                    PoolReader::read(pool, *device, DeviceOffsetInBytes(slot * spacing), 4096)
                        .and_then(|b| SystemConfiguration::parse_slot(&b).ok())
                        .map_or("-".to_string(), |c| c.quantities.slot_generation.to_string())
                })
                .collect();
            format!("d{}[{}]", device.0, gens.join(","))
        })
        .collect()
}

/// drift：一个用过的池（fsid X、u = 5），每一轮在它上面录一次可写挂载，崩在录制流里盘 1 第一次写系统配置槽之前（流的一个前缀：
/// 盘 0 那一写已落、盘 1 那一写与之后的都没落）。`rounds` 轮之后两盘系统配置世代号差 `rounds`；再看今天的实现择不择、读不读得回、挂不挂得上，checker 报什么。
fn drift(rounds: u32) {
    let p = parameters(FSID_DISTINCT, 512);
    let mut devices: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
    let (versions, last) = make_and_use(&mut devices, &p, 5, 7);
    let mut image = image_of(&devices);
    let spacing = u64::from(p.geometry.fixed_structure_slot_spacing);
    let fixed = FixedGeometry {
        fixed_structure_slot_spacing: p.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: p.geometry.journal_ring_bytes,
        root_ring_slots_per_region: p.geometry.root_ring_slots_per_region,
    };
    let h = History {
        name: "drift".into(), order_name: "-".into(), base: image.clone(), writes: Vec::new(), segments: Vec::new(), recorded: 0,
        old: (p.clone(), versions.clone(), last), other: None, ancestor: None, new_parameters: p.clone(),
    };
    println!("name=drift_start last_root={last:?} generations={:?} recover={} raw_red={:?}", generations_by_device(&image, spacing), recover_label(&h, &image), reds(&image));
    for round in 1..=rounds {
        let stream = SharedStream::retaining_contents();
        let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&image)
            .into_iter()
            .map(|(i, d)| (i, RecordingBlockDevice::with_shared_stream(i, d, stream.clone())))
            .collect();
        let mounted = match mount_writable(&p, &mut recorded) {
            Ok(m) => format!("Ok(i{}t{})", m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0),
            Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
        };
        let (writes, segments) = writes_and_segments(&stream.retained_operations(), &fixed);
        let kinds: Vec<String> = writes.iter().map(|w| format!("{}@d{}", w.kind.name(), w.device.0)).collect();
        let first_d0 = writes.iter().position(|w| w.kind.name() == "system_configuration_slot" && w.device.0 == 0);
        let cut = writes.iter().position(|w| w.kind.name() == "system_configuration_slot" && w.device.0 == 1).expect("录到盘 1 写系统配置");
        assert!(first_d0.map_or(false, |d0| d0 < cut), "盘 0 的系统配置写要在盘 1 的之前");
        let persisted: Vec<bool> = (0..writes.len()).map(|i| i < cut).collect();
        image = materialize(&image, &writes, &persisted);
        let shape: Vec<usize> = segments.iter().map(Vec::len).collect();
        println!("name=drift_round round={round} mount_while_recording={mounted} writes={} segment_sizes={shape:?} cut_before_write={cut} kinds_up_to_cut={:?} generations={:?}", writes.len(), &kinds[..=cut.min(kinds.len() - 1)], generations_by_device(&image, spacing));
    }
    let choice = match choose_system_configuration(&image) {
        Ok(c) => format!("d{}g{}", c.immutable.this_device.0, c.quantities.slot_generation),
        Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
    };
    let writable = writable_label(&image, &p);
    let mut after = plain_devices_on(&image);
    let _ = mount_writable(&p, &mut after);
    let after_image = image_of(&after);
    println!(
        "name=drift_end rounds={rounds} generations={:?} choice={choice} recover={} raw_red={:?} read_only={} writable={writable} generations_after_one_more_writable_mount={:?}",
        generations_by_device(&image, spacing), recover_label(&h, &image), reds(&image), read_only_label(&image), generations_by_device(&after_image, spacing)
    );
}

/// bytes：同参数两次 mkfs（两对全新的盘）逐字节比；fsid 换一个再比，数差几个 512 字节扇区、各在哪一段。
fn bytes_mode() {
    let run = |fsid: [u8; 16]| -> MemoryPool {
        let mut devices: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
        make_filesystem_with_order(&parameters(fsid, 512), &mut devices, MkfsOrderForProbe::InvalidateFirst).expect("mkfs");
        image_of(&devices)
    };
    let a = run(FSID_NEW);
    for (tag, other) in [("same_parameters", run(FSID_NEW)), ("other_fsid", run(FSID_REGENERATED))] {
        let mut by_area: BTreeMap<&'static str, u64> = BTreeMap::new();
        for device in a.devices.keys() {
            for sector in 0..DEVICE_BYTES / 512 {
                let x = PoolReader::read(&a, *device, DeviceOffsetInBytes(sector * 512), 512);
                let y = PoolReader::read(&other, *device, DeviceOffsetInBytes(sector * 512), 512);
                if x != y {
                    let offset = sector * 512;
                    let area = if offset < region_start(0).0 { "system_configuration_area" } else if offset < JOURNAL_RING_START_SLOT * SLOT_BYTES { "root_ring_area" } else if offset < UNIT_AREA_START_SLOT * SLOT_BYTES { "journal_ring" } else { "unit_area" };
                    *by_area.entry(area).or_insert(0) += 1;
                }
            }
        }
        println!("name=bytes compare={tag} differing_sectors_by_area={by_area:?} total={}", by_area.values().sum::<u64>());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("");
    match mode {
        "drift" => return drift(args.get(2).and_then(|s| s.parse().ok()).expect("轮数")),
        "bytes" => return bytes_mode(),
        _ => {}
    }
    let h = history(args.get(2).map(String::as_str).expect("历史"), args.get(3).map(String::as_str).expect("写序"));
    let total = crash_state_count_with_torn_in_place_overwrites(&h.base, &h.writes, &h.segments, &full_expansion);
    let shape: Vec<usize> = h.segments.iter().map(Vec::len).collect();
    let kinds: Vec<String> = h.writes[..h.recorded].iter().map(|w| format!("{}@d{}o{}", w.kind.name(), w.device.0, w.offset.0)).collect();
    let base_choice = match choose_system_configuration(&h.base) {
        Ok(c) => format!("{}g{}", String::from_utf8_lossy(&c.immutable.filesystem_identifier), c.quantities.slot_generation),
        Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
    };
    let closed = if shape.iter().all(|n| *n < 64) { closed_form_state_count(&h.segments).to_string() } else { "too_long".into() };
    println!("name=history h={} order={} writes={} segment_sizes={shape:?} closed_form_two_state={closed} states_with_torn={total} old_last_root={:?} base_choice={base_choice} base_recover={} kinds={kinds:?}", h.name, h.order_name, h.recorded, h.old.2, recover_label(&h, &h.base));
    match mode {
        "count" => {}
        "final" => {
            let all = vec![true; h.recorded];
            let pool = materialize(&h.base, &h.writes, &all);
            let (key, _) = judge(&h, &pool, &h.writes, &all);
            let (refuse_new, found_new) = sysconf_guard(&h.base, &h.new_parameters.filesystem_identifier);
            let (new_high, new_zero) = unit_scan(&h.base, &h.new_parameters.filesystem_identifier);
            println!("name=final h={} order={} before_mkfs:sysconf_guard[new]={}/{found_new} before_mkfs:unit_scan[new]={new_high}/{new_zero} {key}", h.name, h.order_name, if refuse_new { "refuse" } else { "allow" });
        }
        "handbuilt" => {
            let landed: Vec<usize> = args.get(4).expect("落了的写下标").split(',').map(|s| s.parse().expect("下标")).collect();
            let persisted: Vec<bool> = (0..h.recorded).map(|i| landed.contains(&i)).collect();
            let pool = materialize(&h.base, &h.writes, &persisted);
            let (key, cut) = judge(&h, &pool, &h.writes, &persisted);
            println!("name=handbuilt h={} order={} landed={landed:?} {key} :: {cut}", h.name, h.order_name);
        }
        "run" => {
            let start: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
            let end: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(total).min(total);
            assert!(end > start && end - start <= STATE_LIMIT_PER_RUN, "序号段空或超过 10⁶ 态");
            let started = std::time::Instant::now();
            let threads = threads();
            let chunk = (end - start).div_ceil(threads).max(1);
            let results = Mutex::new(Vec::new());
            std::thread::scope(|scope| {
                for thread in 0..threads {
                    let from = start + thread * chunk;
                    let to = (start + (thread + 1) * chunk).min(end);
                    if from >= to {
                        continue;
                    }
                    let (h, results) = (&h, &results);
                    scope.spawn(move || {
                        let mut tally = Tally::default();
                        let slices = no_segment_slices(h.segments.len());
                        let _ = visit_crash_state_range_of_the_read_back_flow_with_slices(&h.base, &h.writes, &h.segments, &full_expansion, &slices, from..to, &mut |ordinal, _publish, image| {
                            let pool = materialize(image.base, image.writes, &image.persisted);
                            let (key, cut) = judge(h, &pool, image.writes, &image.persisted);
                            tally.states += 1;
                            if tally.examples.get(&key).map_or(true, |(o, _)| ordinal < *o) {
                                tally.examples.insert(key.clone(), (ordinal, cut));
                            }
                            *tally.signatures.entry(key).or_insert(0) += 1;
                        });
                        results.lock().expect("锁").push(tally);
                    });
                }
            });
            let mut all = Tally::default();
            for t in results.into_inner().expect("锁") {
                all.states += t.states;
                for (k, v) in t.signatures { *all.signatures.entry(k).or_insert(0) += v; }
                for (k, (o, c)) in t.examples {
                    if all.examples.get(&k).map_or(true, |(kept, _)| o < *kept) { all.examples.insert(k, (o, c)); }
                }
            }
            println!("name=done h={} order={} states_judged={} range={start}..{end} seconds={:.1} threads={threads}", h.name, h.order_name, all.states, started.elapsed().as_secs_f64());
            for (k, v) in &all.signatures { println!("name=sig h={} order={} count={v} {k}", h.name, h.order_name); }
            for (k, (o, c)) in &all.examples { println!("name=example h={} order={} ordinal={o} {k} :: {c}", h.name, h.order_name); }
        }
        other => panic!("用法：count|run|final|handbuilt|drift|bytes，收到 {other}"),
    }
}
