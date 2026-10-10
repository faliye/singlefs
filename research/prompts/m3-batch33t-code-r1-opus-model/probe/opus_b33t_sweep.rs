//! m3-batch33t-code-r1 攻方（Opus）探针之二：C692 新下限（T7）在随机用户历史上的余量，不断言、只打印 `OPUS ` 行。只在攻方的仓副本里编，不入库。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports, reason = "攻方原型")]

#[path = "common_admission/mod.rs"]
mod common_admission;

use std::collections::BTreeMap;

use common_admission::*;
use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::mounted_session::UserChange;
use singlefs_core::recovery::{placements_on_each_device_after_the_new_pool_file_creation, recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;

fn kind_of(text: &str) -> String {
    text.split([' ', '(', '{']).next().unwrap_or("").to_string()
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

/// 一步用户动作：o 覆盖写、s 顺序写 1..4 个数据单元、n 建 inode、u 删第一个文件、c 崩了再可写挂载。
fn step(pool: &mut PoolUnderTest<SparseBlockDevice>, op: char, rng: &mut Lcg) -> Result<(), String> {
    let outcome = match op {
        'o' => pool.overwrite(OVERWRITE_BYTES).map(|_| ()),
        's' => {
            let units = 1 + (rng.next() % 4) as usize;
            pool.sequential_write(units * singlefs_core::unit::data_unit_payload_capacity()).map(|_| ())
        }
        'n' => pool.create_one_inode().map(|_| ()),
        'u' => {
            pool.write_time_seconds += 1;
            let write_time_seconds = pool.write_time_seconds;
            pool.publish(UserChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER), write_time_seconds }, None).map(|_| ())
        }
        'c' => return pool.crash_and_mount_writable().map(|_| ()).map_err(|e| format!("MOUNT:{}", kind_of(&format!("{e:?}")))),
        other => panic!("{other}"),
    };
    outcome.map_err(|e| kind_of(&format!("{e:?}")))
}

fn sweep(name: &str, mut pool: PoolUnderTest<SparseBlockDevice>, seed: u64, steps: usize, weights: &str) -> (usize, usize, i64) {
    let unit_area_start = UnitAreaStart::following_the_journal_ring(pool.parameters.geometry.journal_ring_bytes).expect("单元区起点");
    let sizes: Vec<(DeviceIdentity, u64)> = pool.devices.iter().map(|(d, _)| (*d, pool.device_bytes)).collect();
    let bound = placements_on_each_device_after_the_new_pool_file_creation(&sizes, unit_area_start).0;
    let alphabet: Vec<char> = weights.chars().collect();
    let mut rng = Lcg(seed);
    let mut min_margin = i64::MAX;
    let mut walk_red = 0usize;
    let mut refused = 0usize;
    for index in 0..steps {
        let op = alphabet[(rng.next() % alphabet.len() as u64) as usize];
        if let Err(kind) = step(&mut pool, op, &mut rng) {
            refused += 1;
            if kind.starts_with("MOUNT:") {
                println!("OPUS T7 {name} seed={seed} step={index} op={op} {kind}");
                break;
            }
            continue;
        }
        let Some(session) = pool.session.as_ref() else { break };
        let mut per_device: BTreeMap<DeviceIdentity, usize> = BTreeMap::new();
        for record in session.allocator.records() {
            *per_device.entry(record.device).or_default() += 1;
        }
        let fewest = per_device.values().copied().min().unwrap_or(0);
        let margin = fewest as i64 - bound as i64;
        min_margin = min_margin.min(margin);
        let report = recover(&pool.image(), JournalPolicy::Consult);
        if let RecoveryOutcome::Failed { failure, .. } = &report.outcome {
            walk_red += 1;
            println!("OPUS T7 {name} seed={seed} step={index} op={op} records_per_device={fewest} bound={bound} RECOVER_FAILED {}", format!("{failure:?}").chars().take(160).collect::<String>());
        }
    }
    println!("OPUS T7TALLY {name} seed={seed} steps={steps} bound={bound} min_margin={min_margin} recover_failed={walk_red} refused_ops={refused}");
    (walk_red, refused, min_margin)
}

#[test]
fn opus_t7_records_per_device_against_the_new_lower_bound() {
    let seeds: u64 = std::env::var("OPUS_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
    let steps: usize = std::env::var("OPUS_STEPS").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    let widths = std::env::var("OPUS_WIDTHS").unwrap_or_else(|_| "w384,w256,w240,narrow".to_string());
    let selected = |name: &str| widths.split(',').any(|w| w == name);
    for (name, width) in [
        ("w384", HistoryDeviceWidth::UnitAreaOf384Slots),
        ("w256", HistoryDeviceWidth::UnitAreaOf256Slots),
        ("w240", HistoryDeviceWidth::UnitAreaOf240Slots),
    ] {
        if !selected(name) {
            continue;
        }
        for seed in 1..=seeds {
            sweep(name, start_plain(width), seed, steps, "ooooosssnc");
            sweep(&format!("{name}-del"), start_plain(width), seed + 100, steps, "oosnnuc");
        }
    }
    for unit_area_slots in if selected("narrow") { vec![176u64, 264] } else { vec![] } {
        for seed in 1..=seeds {
            let pool = start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
                HistoryDeviceWidth::UnitAreaOf384Slots,
                unit_area_slots,
            );
            sweep(&format!("narrow{unit_area_slots}"), pool, seed + 200, steps, "ooooosssn");
        }
    }
}

/// 4 GiB 盘、1024 MiB 环那套几何（新下限 16）与默认几何（14）各走一段。
#[test]
fn opus_t7_four_gibibyte_geometries() {
    let seeds: u64 = std::env::var("OPUS_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
    let steps: usize = std::env::var("OPUS_STEPS").ok().and_then(|v| v.parse().ok()).unwrap_or(40);
    for seed in 1..=seeds {
        sweep("4g-default", start_plain(HistoryDeviceWidth::FourGibibytes), seed, steps, "ooooosssnc");
        let mut parameters = HistoryDeviceWidth::FourGibibytes.parameters();
        parameters.geometry.journal_ring_bytes = 1 << 30;
        let pool = PoolUnderTest::start_after_the_first_file_on_devices_of(parameters, 4 << 30, plain_device);
        sweep("4g-ring1g", pool, seed, steps, "ooooosssnc");
    }
}
