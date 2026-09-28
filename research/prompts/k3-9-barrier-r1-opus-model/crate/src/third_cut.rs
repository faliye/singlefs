//! 第二、三截：在被判那一次发布的崩溃镜像上起可写挂载（取号、写行、暖机），挂载那一段录制流按层 0 的枚举域全量展开
//! （每一段都展开，`enumerate_layer0_selecting_versions_observing_each_state` 的 expand 恒真），每个状态跑恢复、oracle、池级 checker、
//! 记录核对器；挂载做完之后按后缀再做一步（不做 / 再关掉重挂一次 / 发一次布，发布那一段手摆）。
use std::collections::BTreeMap;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    classified_oracle_violation_for_versions, enumerate_layer0_selecting_versions_observing_each_state,
    full_expansion, layer0_state_count_with_torn_in_place_overwrites, Layer0Tally,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::mount_writable_with_space_admission;
use singlefs_core::recovery::RecoveryOutcome;
use singlefs_core::transaction::{publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::memory_pool::{newest_persisted_root, root_identity_of_write, MemoryPool, PublishedVersion, RetainedWrite};
use singlefs_harness::segments::StepKind;

use crate::common::*;
use crate::scenario;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};

/// 挂载期间一段落点读不出（整次挂载都读不出，读回 I/O 错；写照常落到里层）：方向 ③ 在带屏障那一臂上造「记录与单元都在、挂载读不出一份单元」。
pub struct ReadFault<Inner: BlockDevice> {
    pub inner: Inner,
    pub hidden: Option<(u64, u64)>,
}

impl<Inner: BlockDevice> BlockDevice for ReadFault<Inner> {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if let Some((start, length)) = self.hidden {
            let end = offset.0 + buffer.len() as u64;
            if offset.0 < start + length && start < end {
                return Err(BlockDeviceError::InputOutput(std::io::Error::other("k39 读故障")));
            }
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

fn absorb(total: &mut Layer0Tally, part: &Layer0Tally) {
    total.states += part.states;
    total.violations += part.violations;
    total.ignored_violations += part.ignored_violations;
    total.record_root_without_record += part.record_root_without_record;
    total.record_claimed_state_missing_unit += part.record_claimed_state_missing_unit;
    total.verification_failed_states += part.verification_failed_states;
    total.root_persisted_states += part.root_persisted_states;
    total.failed_states += part.failed_states;
    for (k, v) in &part.checker_violated_states {
        *total.checker_violated_states.entry(k).or_insert(0) += v;
    }
    if total.first_violation.is_none() {
        total.first_violation = part.first_violation.clone();
    }
}

fn versions_with_mount_roots(known: &[PublishedVersion], writes: &[RetainedWrite], landed_content: Option<&Vec<u8>>) -> Vec<PublishedVersion> {
    let mut versions = known.to_vec();
    if let Some(content) = landed_content {
        for write in writes.iter().filter(|w| w.kind == StepKind::RootRecordFua) {
            let (instance, checkpoint_txg) = root_identity_of_write(write);
            if !versions.iter().any(|v| v.instance == instance && v.checkpoint_txg == checkpoint_txg) {
                versions.push(PublishedVersion { instance, checkpoint_txg, content: content.clone() });
            }
        }
    }
    versions
}

/// 一段录制流在 `base` 上按层 0 的域全量展开。超过剩余预算就不跑、交回 None。
pub fn enumerate_stream(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    versions: &[PublishedVersion],
    floor: Key,
    budget_left: &mut u64,
    signatures: &mut BTreeMap<String, u64>,
    label: &str,
) -> Option<Layer0Tally> {
    let count = layer0_state_count_with_torn_in_place_overwrites(base, writes, segments, &full_expansion);
    if count > *budget_left || count > 1_000_000 {
        println!("SKIP {label} state_count={count} budget_left={budget_left}");
        return None;
    }
    *budget_left -= count;
    let judged = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).unwrap_or(0);
    let mut local: BTreeMap<String, u64> = BTreeMap::new();
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        base,
        writes,
        segments,
        judged,
        versions,
        &|_, _| true,
        &mut |image, report| {
            let newest = newest_persisted_root(image.writes, &image.persisted);
            let oracle = classified_oracle_violation_for_versions(&report.outcome, report.effective_root, newest, versions)
                .map(|v| format!("{:?}", v.kind));
            let sig = signature(report, versions, Some(floor), oracle, &[], (false, false));
            *local.entry(sig).or_insert(0) += 1;
        },
    );
    assert_eq!(tally.states, count, "枚举出的状态数等于算出的全量");
    for (sig, n) in local {
        *signatures.entry(format!("{label} {sig}")).or_insert(0) += n;
    }
    Some(tally)
}

pub fn run(args: &[String]) {
    let name = &args[2];
    let width = crate::width_of(&args[3]);
    let full_limit: usize = args[4].parse().unwrap();
    let random: usize = args[5].parse().unwrap();
    let max_crash1: usize = args[6].parse().unwrap();
    let suffix = args[7].as_str();
    let mut budget_left: u64 = std::env::var("K39_STATE_BUDGET").ok().and_then(|v| v.parse().ok()).unwrap_or(1_000_000);
    let built = scenario::build(name, width);
    let (writes, segments) = split(&built.publish_ops, width);
    let (states, _) = crate::crash1_states(&built, full_limit, random);
    // 挑第一截的状态：按类轮流挑，每类内等距取。类 0 是删掉屏障才造得出的「有记录落了而有单元缺席」，再细分：
    // A = 记录全落、恰一个单元两份都缺；B = 记录全落、恰一份缺（一份落一份没落）；S = 记录全落、数据以外的单元全缺（共享内生块）；
    // R = 只落了一部分记录而有单元缺；O = 其余。类 1 = 有记录落、单元全在；类 2 = 一条记录都没落。
    let mut classes: BTreeMap<String, Vec<(u8, usize, Vec<bool>)>> = BTreeMap::new();
    for (segment_index, mask) in &states {
        let persisted = persisted_of(&segments, writes.len(), *segment_index, mask);
        let records: Vec<usize> = (0..writes.len()).filter(|w| writes[*w].kind == StepKind::JournalRecord).collect();
        let units: Vec<usize> = (0..writes.len()).filter(|w| writes[*w].kind == StepKind::UnitWrite).collect();
        let record_persisted = records.iter().any(|w| persisted[*w]);
        let all_records = records.iter().all(|w| persisted[*w]);
        let withheld: Vec<usize> = units.iter().copied().filter(|w| !persisted[*w]).collect();
        let class = if record_persisted && !withheld.is_empty() {
            let mut groups: BTreeMap<u64, (usize, usize)> = BTreeMap::new();
            for w in &units {
                let entry = groups.entry(writes[*w].offset.0).or_insert((0, 0));
                entry.0 += 1;
                if !persisted[*w] { entry.1 += 1; }
            }
            let fully = groups.values().filter(|(n, k)| *k == *n).count();
            let shared_only = withheld.iter().all(|w| !built.data_unit_offsets.contains(&writes[*w].offset.0))
                && units.iter().filter(|w| !built.data_unit_offsets.contains(&writes[**w].offset.0)).all(|w| !persisted[*w]);
            if !all_records { "0R" } else if withheld.len() == 1 { "0B" } else if shared_only { "0S" } else if fully == 1 && withheld.len() == 2 { "0A" } else { "0O" }
        } else if record_persisted { "1" } else { "2" };
        let rank = class.as_bytes()[0] - b'0';
        classes.entry(class.to_string()).or_default().push((rank, *segment_index, mask.clone()));
    }
    for (class, members) in &classes {
        println!("CLASS {class} candidates={}", members.len());
    }
    let mut ranked: Vec<(u8, usize, Vec<bool>)> = Vec::new();
    let class_names: Vec<String> = classes.keys().cloned().collect();
    let mut round = 0usize;
    while ranked.len() < max_crash1 && round < 1000 {
        for class in &class_names {
            let members = &classes[class];
            let stride = (members.len() / 7).max(1);
            let pick = round * stride + round % stride.max(1);
            if pick < members.len() && ranked.len() < max_crash1 {
                println!("PICK {class} #{pick}");
                ranked.push(members[pick].clone());
            }
        }
        round += 1;
    }
    let mut per_rank: BTreeMap<u8, usize> = BTreeMap::new();
    let mut signatures: BTreeMap<String, u64> = BTreeMap::new();
    let mut mount_outcomes: BTreeMap<String, u64> = BTreeMap::new();
    let mut total = Layer0Tally::default();
    let mut suffix_tally = Layer0Tally::default();
    let mut crash1_done = 0usize;
    let started = std::time::Instant::now();
    for (rank, segment_index, mask) in ranked {
        if crash1_done >= max_crash1 {
            break;
        }
        *per_rank.entry(rank).or_insert(0) += 1;
        crash1_done += 1;
        let persisted = persisted_of(&segments, writes.len(), segment_index, &mask);
        let image = materialize(&built.base, &writes, &persisted);
        let landed = recover_image(&image);
        let landed_content = match &landed.outcome {
            RecoveryOutcome::FileRead { content, .. } => Some(content.clone()),
            _ => None,
        };
        let landed_key = landed.effective_root.map(|(i, t)| (i.0, t.0));
        // 版本表只留这条时间线上的：被判那一次没被恢复落到时它不算发布过（第一版原型把它留着，oracle 在「第一个文件」那一格报假红）。
        let lineage: Vec<PublishedVersion> = built
            .versions
            .iter()
            .filter(|v| (v.instance, v.checkpoint_txg) != built.target || landed.effective_root == Some(built.target))
            .cloned()
            .collect();
        let mut session = Session::from_pool(&image, width);
        // K39_MOUNT_READ_FAULT=1：这次挂载读不出被判那一次发布盘 0 上的第一份单元（那一格撕开读故障与缺席）。
        let hidden = std::env::var_os("K39_MOUNT_READ_FAULT").and_then(|_| {
            writes.iter().find(|w| w.kind == StepKind::UnitWrite && w.device.0 == 0).map(|w| (w.offset.0, w.length_in_bytes()))
        });
        let mut faulty: Vec<(DeviceIdentity, ReadFault<Dev>)> = session
            .devices
            .drain(..)
            .map(|(identity, device)| (identity, ReadFault { inner: device, hidden: if identity.0 == 0 { hidden } else { None } }))
            .collect();
        let mounted = mount_writable_with_space_admission(&session.parameters, &mut faulty, SpaceAdmission::JudgedByTheFormula);
        session.devices = faulty.into_iter().map(|(identity, device)| (identity, device.inner)).collect();
        let mount_label = match &mounted {
            Ok(m) => format!("mounted eff={:?} instance={}", (m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0), m.output.instance.0),
            Err(e) => format!("refused {e:?}"),
        };
        *mount_outcomes.entry(format!("rank{rank} landed={landed_key:?} vfail={} {mount_label}", landed.journal.verification_failed.min(1))).or_insert(0) += 1;
        let mut mounted = match mounted {
            Ok(m) => Some(m),
            Err(_) => None,
        };
        if suffix == "remount" {
            if let Some(m) = mounted.take() {
                drop(m);
                let again = mount_writable_with_space_admission(&session.parameters, &mut session.devices, SpaceAdmission::JudgedByTheFormula);
                *mount_outcomes.entry(format!("rank{rank} second-mount {}", match &again { Ok(m) => format!("mounted instance={}", m.output.instance.0), Err(e) => format!("refused {e:?}") })).or_insert(0) += 1;
                mounted = again.ok();
            }
        }
        let ops = session.operations();
        let (mount_writes, mount_segments) = split(&ops, width);
        // 挂载真正接着走的那一版（带读故障时它可以比只读恢复落到的旧）：版本表按它排，挂载写的根照它的内容。
        let mount_effective = mounted.as_ref().map(|m| (m.output.effective_root.instance, m.output.effective_root.checkpoint_txg)).or(landed.effective_root);
        let lineage: Vec<PublishedVersion> = lineage
            .into_iter()
            .filter(|v| (v.instance, v.checkpoint_txg) != built.target || mount_effective == Some(built.target))
            .collect();
        let landed_content = match mount_effective {
            Some(key) if key == landed.effective_root.unwrap_or(key) => landed_content,
            Some((instance, txg)) => lineage.iter().find(|v| v.instance == instance && v.checkpoint_txg == txg).map(|v| v.content.clone()),
            None => None,
        };
        let versions = versions_with_mount_roots(&lineage, &mount_writes, landed_content.as_ref());
        let bits: String = mask.iter().map(|b| if *b { '1' } else { '0' }).collect();
        let kinds: String = segments[segment_index].iter().map(|w| match writes[*w].kind { StepKind::UnitWrite => 'u', StepKind::JournalRecord => 'r', StepKind::RootRecordFua => 'R', _ => 's' }).collect();
        println!("CRASH1 #{crash1_done} rank{rank} seg{segment_index} kinds={kinds} mask={bits} landed={landed_key:?}");
        let label = format!("crash1-#{crash1_done}-rank{rank}");
        if !mount_writes.is_empty() {
            if let Some(tally) = enumerate_stream(&image, &mount_writes, &mount_segments, &versions, built.floor, &mut budget_left, &mut signatures, &label) {
                absorb(&mut total, &tally);
            }
        }
        // 挂载整段落盘之后的池：恢复与 checker。
        let after = session.memory_pool();
        let after_report = recover_image(&after);
        let after_checker: Vec<&str> = check_pool_image(&after)
            .into_iter()
            .filter_map(|(invariant, verdict)| matches!(verdict, InvariantVerdict::Violated(_)).then_some(invariant))
            .collect();
        let after_oracle = classified_oracle_violation_for_versions(&after_report.outcome, after_report.effective_root, None, &versions).map(|v| format!("{:?}", v.kind));
        *signatures.entry(format!("after-mount-#{crash1_done}-rank{rank} {}", signature(&after_report, &versions, Some(built.floor), after_oracle, &after_checker, (false, false)))).or_insert(0) += 1;
        if suffix == "publish" {
            if let Some(mut m) = mounted.take() {
                let begin = session.operation_count();
                let base2 = session.memory_pool();
                let c3 = scenario::content(1, 3, 5000);
                let instance = m.output.instance;
                let published = {
                    let mut writer = PoolWriter::new(&session.parameters, session.devices.as_mut_slice());
                    match &m.current {
                        PoolVersion::WithFile(current) => publish_overwrite(&mut writer, &mut m.allocator, current, FirstFile { content: &c3, write_time_seconds: scenario::WRITE_TIME + 5 }, instance),
                        PoolVersion::WithoutFile(current) => publish_first_file(&mut writer, &mut m.allocator, &current.root, FirstFile { content: &c3, write_time_seconds: scenario::WRITE_TIME + 5 }, instance, &current.record_bytes),
                    }
                };
                match published {
                    Err(e) => {
                        *mount_outcomes.entry(format!("rank{rank} suffix-publish refused {e:?}")).or_insert(0) += 1;
                    }
                    Ok(output) => {
                        let mut versions3 = versions.clone();
                        versions3.push(PublishedVersion { instance: output.root.instance, checkpoint_txg: output.root.checkpoint_txg, content: c3.clone() });
                        let ops3 = session.operations()[begin..].to_vec();
                        let (w3, s3) = split(&ops3, width);
                        let data: std::collections::BTreeSet<u64> = output
                            .rewritten
                            .iter()
                            .filter(|role| matches!(role, singlefs_core::transaction::TransactionUnit::Data(_)))
                            .map(|role| output.unit(*role).slot.to_device_offset().0)
                            .collect();
                        let mut rng = Rng(0x1234_5678 ^ crash1_done as u64);
                        let judged = w3.iter().position(|w| w.kind == StepKind::RootRecordFua).unwrap_or(0);
                        let floor3 = after_report.effective_root.map(|(i, t)| (i, t)).unwrap_or(built.floor);
                        for (index, segment) in s3.iter().enumerate() {
                            let (masks, _) = masks_for_segment(&w3, segment, full_limit, random.min(8), &data, &mut rng);
                            for mask3 in masks {
                                let persisted3 = persisted_of(&s3, w3.len(), index, &mask3);
                                let (sig, _) = evaluate(&base2, &w3, persisted3, judged, &versions3, Some(floor3), &mut suffix_tally);
                                *signatures.entry(format!("suffix-publish-#{crash1_done}-rank{rank} seg{index} {sig}")).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    for (outcome, n) in &mount_outcomes {
        println!("MOUNT {n} {outcome}");
    }
    for (sig, n) in &signatures {
        println!("SIG {n} {sig}");
    }
    println!("{}", tally_line(&format!("{name}-third-cut"), &total));
    println!("{}", tally_line(&format!("{name}-suffix-publish"), &suffix_tally));
    println!("CRASH1_STATES {crash1_done} per_rank={per_rank:?} budget_left={budget_left} ELAPSED {:.1}s", started.elapsed().as_secs_f64());
}
