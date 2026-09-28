//! K3-9 攻方腿原型（副本，第一轮 k3-9-barrier-r1）。用法：
//!   k39-barrier-attack survey <width>
//!   k39-barrier-attack stage1 <scenario> <width> <full_limit> <random>
//!   k39-barrier-attack stage2 <scenario> <width> <full_limit> <random> <max_crash1_states> <suffix>
//! 臂由环境变量 K39_DROP_UNIT_RECORD_BARRIER 定（设了 = 删掉单元与记录之间那道屏障）。
mod common;
mod faults;
mod scenario;
mod third_cut;

use std::collections::BTreeMap;

use singlefs_checker_tier::crash::Layer0Tally;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::closed_form_state_count;
use singlefs_harness::segments::StepKind;

use common::*;

pub fn width_of(name: &str) -> HistoryDeviceWidth {
    match name {
        "4g" => HistoryDeviceWidth::FourGibibytes,
        "384" => HistoryDeviceWidth::UnitAreaOf384Slots,
        other => panic!("宽度 {other}"),
    }
}

pub const SCENARIOS: [&str; 6] = ["first", "over", "seq2", "seq3", "over_i2", "rollback"];

/// 被判那一次发布的每个崩溃状态（段 + 段内掩码）。
pub fn crash1_states(built: &scenario::Built, full_limit: usize, random: usize) -> (Vec<(usize, Vec<bool>)>, Vec<String>) {
    let (writes, segments) = split(&built.publish_ops, built.width);
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let mut states = Vec::new();
    let mut notes = Vec::new();
    for (index, segment) in segments.iter().enumerate() {
        let (masks, full) = masks_for_segment(&writes, segment, full_limit, random, &built.data_unit_offsets, &mut rng);
        notes.push(format!("segment {index}: {} writes, {} states, {}", segment.len(), masks.len(), if full { "full" } else { "hand-placed" }));
        for mask in masks {
            states.push((index, mask));
        }
    }
    (states, notes)
}

fn judged_root_index(writes: &[singlefs_harness::memory_pool::RetainedWrite]) -> usize {
    writes.iter().position(|w| w.kind == StepKind::RootRecordFua).unwrap_or(0)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("survey");
    println!("ARM {}", arm_name());
    match mode {
        "survey" => {
            let width = width_of(&args[2]);
            for name in SCENARIOS {
                let built = scenario::build(name, width);
                let (writes, segments) = split(&built.publish_ops, width);
                println!(
                    "SURVEY {name} width={} target={:?} floor={:?} segments {} closed_form={}",
                    args[2],
                    (built.target.0 .0, built.target.1 .0),
                    (built.floor.0 .0, built.floor.1 .0),
                    describe_segments(&writes, &segments),
                    closed_form_state_count(&segments)
                );
            }
        }
        "stage1" => {
            let name = &args[2];
            let width = width_of(&args[3]);
            let full_limit: usize = args[4].parse().unwrap();
            let random: usize = args[5].parse().unwrap();
            let built = scenario::build(name, width);
            let (writes, segments) = split(&built.publish_ops, width);
            println!("SEGMENTS {name} {}", describe_segments(&writes, &segments));
            // 被判那一次的单元写有几份落在「之前已写过、带可用单元头」的槽上（复用回收的槽）。
            let reused = writes
                .iter()
                .filter(|w| w.kind == StepKind::UnitWrite)
                .filter(|w| built.base.devices[&w.device].written_sectors_in(w.offset, w.length_in_bytes()).len() > 0)
                .count();
            println!("REUSED unit writes landing on previously written slots: {reused}");
            let (states, notes) = crash1_states(&built, full_limit, random);
            for note in notes {
                println!("PLAN {note}");
            }
            let judged = judged_root_index(&writes);
            let mut tally = Layer0Tally::default();
            let mut signatures: BTreeMap<String, u64> = BTreeMap::new();
            let started = std::time::Instant::now();
            for (segment_index, mask) in &states {
                let persisted = persisted_of(&segments, writes.len(), *segment_index, mask);
                let (sig, _) = evaluate(&built.base, &writes, persisted, judged, &built.versions, Some(built.floor), &mut tally);
                if sig.contains("checker=[\"") && mask.iter().filter(|p| !**p).count() <= 2 {
                    let bits: String = mask.iter().map(|b| if *b { '1' } else { '0' }).collect();
                    let kinds: String = segments[*segment_index].iter().map(|w| match writes[*w].kind { StepKind::UnitWrite => 'u', StepKind::JournalRecord => 'r', _ => 's' }).collect();
                    let withheld: Vec<String> = segments[*segment_index].iter().zip(mask).filter(|(_, p)| !**p).map(|(w, _)| format!("{}@dev{}+{}", writes[*w].kind.name(), writes[*w].device.0, writes[*w].offset.0 / 32768)).collect();
                    println!("CHECKER-RED-FEW-WITHHELD seg{segment_index} kinds={kinds} mask={bits} withheld={withheld:?}");
                }
                *signatures.entry(format!("seg{segment_index} {sig}")).or_insert(0) += 1;
            }
            for (sig, count) in &signatures {
                println!("SIG {count} {sig}");
            }
            println!("{}", tally_line(name, &tally));
            println!("ELAPSED {:.1}s states={}", started.elapsed().as_secs_f64(), states.len());
        }
        "full1" => {
            // 被判那一次发布的录制流在它之前的池上按层 0 的域全量展开（每一段都展开，含系统配置槽原地覆写的撕裂态）。
            let name = &args[2];
            let width = width_of(&args[3]);
            let built = scenario::build(name, width);
            let (writes, segments) = split(&built.publish_ops, width);
            println!("SEGMENTS {name} {}", describe_segments(&writes, &segments));
            let mut budget: u64 = 1_000_000;
            let mut signatures: BTreeMap<String, u64> = BTreeMap::new();
            let started = std::time::Instant::now();
            match third_cut::enumerate_stream(&built.base, &writes, &segments, &built.versions, built.floor, &mut budget, &mut signatures, name) {
                Some(tally) => {
                    for (sig, count) in &signatures {
                        println!("SIG {count} {sig}");
                    }
                    println!("{}", tally_line(name, &tally));
                }
                None => println!("NOT-RUN {name}: 全量超过 10^6"),
            }
            println!("ELAPSED {:.1}s", started.elapsed().as_secs_f64());
        }
        "stage2" => third_cut::run(&args),
        "faults" => faults::run(&args),
        other => panic!("模式 {other}"),
    }
}
