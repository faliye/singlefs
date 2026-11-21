//! E180（准入读数与真可写量）入库装置：跑前登记 `research/prompts/e180-preregistration.md`（岔路单
//! `research/prompts/m3-close-admission-and-shadow-ledger-forks.md` 的 F2、F5、F6、F8 四行）。只读、只驱动与观测 `crates/` 今天的
//! 会话发布路径、准入读数与分配器，不改 `crates/singlefs-core`、`crates/singlefs-checker`。本文件是第一段（读数段）：
//! F2 的全部臂、F5 的残差、F8 的轨迹；F6 的造格、重试与崩溃点枚举（第二段）不在这里。
//!
//! 用法：`e180_admission_reading_versus_writable_bytes reading [--geometry <几何名>] [--family <族名>]`（产物打到标准输出，末行 `E name=done emitted=<行数>`）；
//! `… reading --geometry <几何名> --family <族名> --shard-output <文件>`（只跑一片，写片文件，见 [`shard_text`]）；
//! `… list-shards`（全部片与各占的线程数）；`… merge <片目录>`（把齐了的片合成与一次跑全部逐字节相同的产物）；
//! `e180_admission_reading_versus_writable_bytes calibrate`（在默认几何上量探测与恢复的挂钟，外推整段，不进产物）。
//! 分片、续跑与多机的跑法在 `scripts/experiment-apparatus/e180-reading-shards.sh`。
//! 每一行产物以 `E name=` 开头；停机条款不过打 `E name=stop reason=…` 并退 4。
//!
//! 「这一刻写得进的最大字节数」T 的探测（登记 5.1）：在状态的拷贝上经会话做一次 `SequentialWrite`，设备是写时复制的叠层，
//! 第一个数据单元的写到达时报块设备错，会话交回这个错、设备记「数据写到达」即记放行；空间不够由会话交回
//! `NoSpaceAfterRaisingTheFloor` 记被拒。一次 journal 记录数超过上限（`JournalRecordsOfThePublishExceedTheLimit`）的拒绝不是空间不够，
//! 按登记的字面那个状态 T 记「取不出」；登记没料到默认环的在飞记录上限（65536 条）把「空间很空」的状态的 T 截在一个下界上，
//! 装置对这类状态另记「截尾」并给出下界，登记第十二节修订写了补的判据。

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_core::admission::{
    admission_reading_before_a_publish, checkpoint_cost_of_the_version_to_build_on,
    AdmissionReading,
};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, unmount, MountOutput, Mounted, Unmounted};
use singlefs_core::mounted_session::{
    refusal_is_short_of_space, MountedSession, UserChange, UserChangePublished, UserChangeRefused,
};
use singlefs_core::recovery::allocation_records_under_root;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    publish_first_file, FirstFile, PoolVersion, PoolWriter, PublishError, TransactionOutput,
    TransactionUnit,
};

mod arm_residual_and_family_tallies;
mod arms_and_judgement;
mod changes_and_remounts;
mod geometry_cell_product_lines;
mod history_run;
mod local_constants_checked_against_the_implementation;
mod overlay_disk;
mod pool_and_publish_log;
mod probe_classification;
mod probed_families;
mod reading_of_a_state;
mod registered_geometries;
mod shard_files;
mod writable_units_search;

use arm_residual_and_family_tallies::*;
use arms_and_judgement::*;
use changes_and_remounts::*;
use geometry_cell_product_lines::*;
use history_run::*;
use local_constants_checked_against_the_implementation::*;
use overlay_disk::*;
use pool_and_publish_log::*;
use probe_classification::*;
use probed_families::*;
use reading_of_a_state::*;
use registered_geometries::*;
use shard_files::*;
use writable_units_search::*;

/// 产物行的出口：整行经一把锁打到标准输出，不同线程的行不交错；线程里攒行的另走 [`LineBuffer`]。
static OUTPUT_LOCK: Mutex<()> = Mutex::new(());

fn print_product_line(line: &str) {
    let _guard = OUTPUT_LOCK.lock().expect("输出锁没被毒化");
    println!("{line}");
}

/// 一段历史攒下的产物行：线程里先攒着，整段跑完由主线程按固定次序打出，行序与线程调度无关。
#[derive(Default)]
struct LineBuffer {
    lines: Vec<String>,
}

impl LineBuffer {
    fn push(&mut self, line: String) {
        self.lines.push(line);
    }
}

/// 停机：打一行 `E name=stop` 并退 [`STOP_EXIT_CODE`]（登记第十一节停机条款）。
fn stop_with(reason: &str, detail: &str) -> ! {
    print_product_line(&format!("E name=stop reason={reason} {detail}"));
    std::process::exit(STOP_EXIT_CODE)
}

struct ReadingOptions {
    geometry: Option<String>,
    family: Option<String>,
    /// 给了就只跑一片（`--geometry` 与 `--family` 都要给），结果写成片文件（[`shard_text`]），原子换名到这个路径，标准输出不打产物。
    shard_output: Option<String>,
}

fn parse_reading_options(options: &[String]) -> ReadingOptions {
    let mut parsed = ReadingOptions {
        geometry: None,
        family: None,
        shard_output: None,
    };
    let mut index = 0;
    while index < options.len() {
        let value = options.get(index + 1);
        match (options[index].as_str(), value) {
            ("--geometry", Some(name)) => parsed.geometry = Some(name.clone()),
            ("--family", Some(name)) => parsed.family = Some(name.clone()),
            ("--shard-output", Some(path)) => parsed.shard_output = Some(path.clone()),
            (other, _) => stop_with("bad_option", &format!("option={other}")),
        }
        index += 2;
    }
    if parsed.shard_output.is_some() && (parsed.geometry.is_none() || parsed.family.is_none()) {
        stop_with("bad_option", "shard_output_needs_both_geometry_and_family");
    }
    parsed
}

fn families_of_the_geometry(geometry: Geometry, only: Option<&str>) -> Vec<&'static str> {
    let mut families: Vec<&'static str> = PROBED_FAMILIES.to_vec();
    if geometry.name != "unequal_devices_geometry" {
        families.push(LONG_MOUNT_FAMILY);
    }
    match only {
        Some(name) => vec![static_family_name(name)],
        None => families,
    }
}

/// 一族的探测线程数 K（一轮探几个点）：总线程数按这个几何格的全部族数平分，与这一次只跑其中几族无关。
/// 一片只跑一族，K 照样按全部族数分，片跑出来的与一次跑全部的那一族逐字节相同（K 会改探测点，见 [`find_writable_units`]）。
fn probe_threads_of_the_geometry(geometry: Geometry, total_threads: usize) -> usize {
    (total_threads / families_of_the_geometry(geometry, None).len()).max(1)
}

/// 片文件名：`<几何名>-<族名>.shard`；合并按 [`ALL_GEOMETRIES`] 与 [`families_of_the_geometry`] 的次序找。
fn shard_file_name(geometry: Geometry, family: &str) -> String {
    format!("{}-{family}.shard", geometry.name)
}

/// 一个几何格的结果，族按 [`families_of_the_geometry`] 的次序。
struct GeometryCellOutcome {
    geometry: Geometry,
    outcomes: Vec<FamilyOutcome>,
}

/// 产物的全部行（一次跑全部与合并片文件共用这一个函数，两条路打出的产物因此逐字节相同）：本地常量、独立算术，
/// 每个几何格的几何行、锚点、各族的状态行与族汇总、格汇总、判决行，最后几何敏感性与完成标记 `E name=done emitted=<行数>`。
fn reading_product_lines(
    local_constants_checked: usize,
    cells: &[GeometryCellOutcome],
) -> Vec<String> {
    let mut output = Vec::new();
    output.push(
        ProductLine::new("local_constants_checked")
            .field("count", local_constants_checked)
            .finish(),
    );
    append_independent_arithmetic(&mut output);
    let mut merged_by_geometry: BTreeMap<&'static str, FamilyTally> = BTreeMap::new();
    let mut trajectories: BTreeMap<(&'static str, &'static str), TrajectoryTally> = BTreeMap::new();
    for cell in cells {
        let geometry = cell.geometry;
        let outcomes = &cell.outcomes;
        append_geometry_header(
            &mut output,
            geometry,
            &outcomes[0].unit_area_slots_per_device,
        );
        append_anchors(&mut output, geometry, &outcomes[0].anchors);
        for outcome in outcomes {
            output.extend(outcome.lines.iter().cloned());
            append_family_summary(&mut output, geometry.name, outcome);
            if let Some(trajectory) = outcome.tally.closed_minus_ring_window.first() {
                trajectories.insert((geometry.name, outcome.family), trajectory.clone());
            }
        }
        let merged = append_cell_summary(&mut output, geometry.name, outcomes);
        append_verdict(
            &mut output,
            geometry.name,
            outcomes,
            &merged,
            local_constants_checked,
        );
        merged_by_geometry.insert(geometry.name, merged);
    }
    append_geometry_sensitivity(&mut output, &merged_by_geometry, &trajectories);
    let emitted = output.len() + 1;
    output.push(ProductLine::new("done").field("emitted", emitted).finish());
    output
}

fn reading(options: &[String]) {
    let options = parse_reading_options(options);
    let local_constants_checked = local_constants_against_the_implementation();
    let geometries: Vec<Geometry> = match &options.geometry {
        Some(name) => vec![geometry_named(name)],
        None => ALL_GEOMETRIES.to_vec(),
    };
    let total_threads = total_threads();
    let mut cells = Vec::new();
    for geometry in geometries {
        let families = families_of_the_geometry(geometry, options.family.as_deref());
        let outcomes = run_geometry_cell(geometry, &families, total_threads);
        cells.push(GeometryCellOutcome { geometry, outcomes });
    }
    if let Some(path) = &options.shard_output {
        let cell = cells.pop().expect("一片只有一个几何格");
        let outcome = cell.outcomes.into_iter().next().expect("一片只有一族");
        let header = ShardHeader {
            geometry_name: cell.geometry.name.to_string(),
            family: outcome.family.to_string(),
            total_threads,
            probe_threads: probe_threads_of_the_geometry(cell.geometry, total_threads),
            local_constants_checked,
        };
        write_shard_atomically(path, &shard_text(&header, &outcome));
        print_product_line(
            &ProductLine::new("shard_written")
                .field("shard", shard_file_name(cell.geometry, outcome.family))
                .field("states", outcome.tally.states)
                .field("probes_run", outcome.tally.probes_run)
                .field("seconds", outcome.seconds)
                .finish(),
        );
        return;
    }
    for line in reading_product_lines(local_constants_checked, &cells) {
        print_product_line(&line);
    }
}

/// 列出全部片：一行一片「片文件名 TAB 几何名 TAB 族名 TAB 这一片占的线程数」，次序就是合并的次序。
/// 占的线程数：带探测的族是 [`probe_threads_of_the_geometry`]（按 `E180_THREADS` 或登记的每台默认重活线程数算），长挂载族不探测记 1；长跑脚本按它往一台上并排放片。
fn list_shards() {
    for line in shard_list_lines(total_threads()) {
        println!("{line}");
    }
}

/// [`list_shards`] 打的行（单测数片数、钉每片的线程数）。
fn shard_list_lines(total_threads: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for geometry in ALL_GEOMETRIES {
        for family in families_of_the_geometry(geometry, None) {
            let threads_of_the_shard = if family == LONG_MOUNT_FAMILY {
                1
            } else {
                probe_threads_of_the_geometry(geometry, total_threads)
            };
            lines.push(format!(
                "{}\t{}\t{family}\t{threads_of_the_shard}",
                shard_file_name(geometry, family),
                geometry.name
            ));
        }
    }
    lines
}

/// `merge <片目录>`：读目录里全部 `*.shard`（别的文件不看），经 [`merge_shard_texts`] 合成产物打到标准输出；片不齐、对不上就停机。
fn merge(arguments: &[String]) {
    let [directory] = arguments else {
        stop_with("bad_option", "usage=merge_<shard_directory>")
    };
    let entries = std::fs::read_dir(directory).unwrap_or_else(|error| {
        stop_with(
            "shard_directory_unreadable",
            &format!("directory={directory} error={error}"),
        )
    });
    let mut texts = BTreeMap::new();
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            stop_with(
                "shard_directory_unreadable",
                &format!("directory={directory} error={error}"),
            )
        });
        let name = entry
            .file_name()
            .into_string()
            .unwrap_or_else(|raw| stop_with("shard_name_is_not_utf8", &format!("name={raw:?}")));
        if !name.ends_with(".shard") {
            continue;
        }
        let text = std::fs::read_to_string(entry.path()).unwrap_or_else(|error| {
            stop_with("shard_unreadable", &format!("shard={name} error={error}"))
        });
        texts.insert(name, text);
    }
    match merge_shard_texts(&texts) {
        Ok(lines) => {
            for line in lines {
                print_product_line(&line);
            }
        }
        Err(problem) => stop_with("shard_merge_refused", &problem),
    }
}

/// 开跑前量一小段外推挂钟（登记 5.6）：默认几何上量 20 次探测与 20 次恢复，再在一个有分量的状态上量一次完整的 T 搜索。不进产物（打标准错误）。
fn calibrate() {
    let geometry = geometry_named("default_geometry");
    let build_pool_started = Instant::now();
    let (mut pool, _) = build_pool(geometry);
    eprintln!(
        "E180-calibration build_pool_seconds={:.2}",
        build_pool_started.elapsed().as_secs_f64()
    );
    let probe_threads = total_threads();
    let change_started = Instant::now();
    let outcome = apply_change(&mut pool, ChangeKind::SequentialWriteOfUnits(5000));
    eprintln!(
        "E180-calibration change_sequential_write_of_5000_units_seconds={:.2} outcome={}",
        change_started.elapsed().as_secs_f64(),
        outcome.label()
    );
    let read_state_started = Instant::now();
    let state = read_state(&pool);
    eprintln!(
        "E180-calibration read_state_seconds={:.3}",
        read_state_started.elapsed().as_secs_f64()
    );
    let stripped = strip_bytes_the_probe_does_not_need(&mut pool.session);
    let probes_started = Instant::now();
    let verdicts: Vec<ProbeVerdict> = {
        let prober = StateProber {
            pool: &pool,
            probe_threads,
            probes_run: AtomicU64::new(0),
        };
        (0..20).map(|_| prober.probe_one(2000)).collect()
    };
    eprintln!(
        "E180-calibration twenty_serial_probes_of_2000_units_seconds={:.2} first_verdict={:?}",
        probes_started.elapsed().as_secs_f64(),
        verdicts[0]
    );
    let search_started = Instant::now();
    let mut model = JournalModel::NOT_LEARNED_YET;
    let writable = {
        let prober = StateProber {
            pool: &pool,
            probe_threads,
            probes_run: AtomicU64::new(0),
        };
        let writable = find_writable_units(
            &prober,
            upper_bound_units_by_the_space_conditions(&pool, &state),
            &mut model,
            &LowerBoundTightening::OnlyWhereAnArmJudgementCanChange {
                units_asked_for_by_the_arms: state.units_asked_for_by_every_arm(),
            },
        );
        eprintln!(
            "E180-calibration probes_in_one_search={}",
            prober.probes_run.load(Ordering::Relaxed)
        );
        writable
    };
    eprintln!(
        "E180-calibration one_full_search_seconds={:.2} result={writable:?} journal_model={model:?}",
        search_started.elapsed().as_secs_f64()
    );
    restore_stripped_bytes(&mut pool.session, stripped);
    let mounts_started = Instant::now();
    for _ in 0..20 {
        mount_again(&mut pool);
    }
    eprintln!(
        "E180-calibration twenty_writable_mounts_seconds={:.2}",
        mounts_started.elapsed().as_secs_f64()
    );
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("reading") => reading(&arguments[1..]),
        Some("list-shards") => list_shards(),
        Some("merge") => merge(&arguments[1..]),
        Some("calibrate") => calibrate(),
        Some(other) => stop_with("unknown_mode", &format!("mode={other}")),
        None => stop_with("missing_mode", "usage=reading|list-shards|merge|calibrate"),
    }
}
