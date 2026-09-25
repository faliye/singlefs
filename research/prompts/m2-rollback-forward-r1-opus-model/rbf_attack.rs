//! m2-rollback-forward-r1 云端攻方腿的原型用例（冻结副本的拷贝上，不入库）。
//! 原型：`singlefs_core::mount::prototype_mount_rollback_forward`（挂载时做的向前发布回退）与
//! `prototype_unmount_raising_the_floor_to_the_newest_root`（B：卸载时抬 F 到最新根）。
//! 每条用例打一行 `RBF ...` 给报告抄。

mod common;

use common::{build_pool, file_content, parameters, IMAGE_BYTES, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::mount::{
    mount_writable, prototype_mount_rollback_forward, prototype_raise_rollback_floor,
    prototype_unmount_raising_the_floor_to_the_newest_root, rollback_floor_ceiling, Mounted,
    PrototypeForwardArm, PrototypeForwardCounts, PrototypeForwardError, RollbackTarget,
    ShadowLedger,
};
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_chain_of_root, readable_roots, recover,
    walk_to_file, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::collections::BTreeMap;

type Dev = RecordingBlockDevice<SparseBlockDevice>;

/// 一个在内存里的池：两块稀疏盘（写录进一条流），与每一版（实例, txg）的文件内容。
pub struct Session {
    pub devices: Vec<(DeviceIdentity, Dev)>,
    pub stream: SharedStream,
    pub contents: BTreeMap<(u32, u64), Vec<u8>>,
}

/// 一次可写挂载之后这个进程手里的东西。
pub struct Open {
    pub allocator: PoolAllocator,
    pub current: TransactionOutput,
    pub instance: InstanceGeneration,
}

pub fn content_of(seed: usize) -> Vec<u8> {
    (0..2600 + seed)
        .map(|index| u8::try_from((index * 7 + seed * 11) % 239).expect("小于 256"))
        .collect()
}

/// 起点：mkfs、暖机、A（实例 1，txg 3，内容 `file_content()`），转成内存池，A 那一版还开着。
pub fn start(tag: &str) -> (Session, Open) {
    let pool = build_pool(tag);
    let image = pool.memory_pool();
    let stream = SharedStream::retaining_contents();
    let devices = common::crash_state_devices(&image, &[], &[], &stream);
    let mut contents = BTreeMap::new();
    contents.insert((1, 3), file_content());
    let open = Open {
        allocator: pool.allocator.clone(),
        current: pool.output.clone(),
        instance: InstanceGeneration(1),
    };
    (
        Session {
            devices,
            stream,
            contents,
        },
        open,
    )
}

impl Session {
    pub fn image(&self) -> MemoryPool {
        common::memory_pool_of_sparse_devices(&self.devices)
    }

    fn note_mount(&mut self, mounted: &Mounted, content: &[u8]) {
        for published in std::iter::once(&mounted.output.row_publish).chain(&mounted.output.warm_up_publishes) {
            let root = published.root();
            self.contents
                .insert((root.instance.0, root.checkpoint_txg.0), content.to_vec());
        }
    }

    pub fn mount(&mut self) -> Open {
        let mounted = mount_writable(&parameters(), &mut self.devices).expect("可写挂载");
        let effective = mounted.output.effective_root;
        let content = self
            .contents
            .get(&(effective.instance.0, effective.checkpoint_txg.0))
            .cloned()
            .unwrap_or_else(|| walk_to_file(&self.image(), &effective, &mut 0).ok().flatten().expect("所选那一版读得出"));
        self.note_mount(&mounted, &content);
        let instance = mounted.output.instance;
        Open {
            allocator: mounted.allocator,
            current: mounted.current.into_file_version().expect("带文件"),
            instance,
        }
    }

    pub fn rollback_forward(
        &mut self,
        target: (u32, u64),
        arm: PrototypeForwardArm,
    ) -> Result<(Open, PrototypeForwardCounts, Mounted), PrototypeForwardError> {
        let (mounted, counts) = prototype_mount_rollback_forward(
            &parameters(),
            &mut self.devices,
            RollbackTarget {
                instance: InstanceGeneration(target.0),
                checkpoint_txg: CheckpointTxg(target.1),
            },
            arm,
        )?;
        let content = self.contents.get(&target).cloned().expect("目标的内容记着");
        self.note_mount(&mounted, &content);
        let open = Open {
            allocator: mounted.allocator.clone(),
            current: mounted.current.clone().into_file_version().expect("带文件"),
            instance: mounted.output.instance,
        };
        Ok((open, counts, mounted))
    }

    pub fn overwrite(&mut self, open: &mut Open, content: &[u8]) -> TransactionOutput {
        self.try_overwrite(open, content).expect("覆盖写")
    }

    pub fn try_overwrite(&mut self, open: &mut Open, content: &[u8]) -> Result<TransactionOutput, String> {
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        let output = publish_overwrite(
            &mut writer,
            &mut open.allocator,
            &open.current,
            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            open.instance,
        )
        .map_err(|error| format!("{error:?}"))?;
        self.contents.insert(
            (output.root.instance.0, output.root.checkpoint_txg.0),
            content.to_vec(),
        );
        open.current = output.clone();
        Ok(output)
    }

    pub fn new_inodes(&mut self, open: &mut Open, count: u64) -> TransactionOutput {
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        let content = self
            .contents
            .get(&(open.current.root.instance.0, open.current.root.checkpoint_txg.0))
            .cloned()
            .expect("上一版的内容记着");
        let output = publish_new_inodes(
            &mut writer,
            &mut open.allocator,
            &open.current,
            count,
            FIXED_WRITE_TIME_SECONDS + 90,
            open.instance,
        )
        .expect("建 inode");
        self.contents.insert(
            (output.root.instance.0, output.root.checkpoint_txg.0),
            content,
        );
        open.current = output.clone();
        output
    }

    /// B：卸载时抬 F 到最新根。
    pub fn unmount_b(&mut self, open: &mut Open) -> Vec<TransactionOutput> {
        let content = self
            .contents
            .get(&(open.current.root.instance.0, open.current.root.checkpoint_txg.0))
            .cloned()
            .expect("内容记着");
        let raised = prototype_unmount_raising_the_floor_to_the_newest_root(
            &parameters(),
            &mut self.devices,
            &mut open.allocator,
            &mut open.current,
        )
        .expect("卸载抬 F");
        for published in &raised.publishes {
            self.contents.insert(
                (published.root.instance.0, published.root.checkpoint_txg.0),
                content.clone(),
            );
        }
        raised.publishes
    }

    /// 今天的抬 F（判上限），到 `floor`。
    pub fn raise_floor_today(&mut self, open: &mut Open, floor: u64) -> Result<usize, String> {
        let content = self
            .contents
            .get(&(open.current.root.instance.0, open.current.root.checkpoint_txg.0))
            .cloned()
            .expect("内容记着");
        let raised = prototype_raise_rollback_floor(
            &parameters(),
            &mut self.devices,
            &mut open.allocator,
            &mut open.current,
            CheckpointTxg(floor),
            ShadowLedger::On,
            false,
        )
        .map_err(|error| format!("{error:?}"))?;
        for published in &raised.publishes {
            self.contents.insert(
                (published.root.instance.0, published.root.checkpoint_txg.0),
                content.clone(),
            );
        }
        Ok(raised.reclaimed.len())
    }

    /// 今天的上限（第 4 新的非空有效根、每块盘最新有效根取小）。
    pub fn ceiling_today(&self, open: &Open) -> u64 {
        let system_configuration = choose_system_configuration(&self.devices).expect("系统配置");
        let table = instance_table_chain_of_root(&self.devices, &open.current.root)
            .expect("实例表")
            .records;
        rollback_floor_ceiling(
            &self.devices,
            &system_configuration,
            open.current.root.rollback_floor,
            &table,
        )
        .expect("上限")
        .0
    }
}

pub fn roots_of(image: &MemoryPool) -> Vec<RootRecord> {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let mut roots = readable_roots(
        image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    roots.sort_by_key(|root| (root.checkpoint_txg, root.instance));
    roots
}

pub fn read_under(
    image: &MemoryPool,
    root: &(InstanceGeneration, CheckpointTxg),
) -> Option<Vec<u8>> {
    let found = roots_of(image)
        .into_iter()
        .find(|candidate| (candidate.instance, candidate.checkpoint_txg) == *root)?;
    walk_to_file(image, &found, &mut 0).ok().flatten()
}

/// 沿一条根走到文件：读得出就交回内容，读不出交回错。
pub fn walk_under(image: &MemoryPool, instance: u32, txg: u64) -> Result<Option<Vec<u8>>, String> {
    let found = roots_of(image)
        .into_iter()
        .find(|candidate| {
            (candidate.instance.0, candidate.checkpoint_txg.0) == (instance, txg)
        })
        .ok_or_else(|| "不在环里".to_string())?;
    walk_to_file(image, &found, &mut 0).map_err(|failure| format!("{failure:?}"))
}

/// 把这几条根（按 txg 认）的根槽各翻一个字节（读得出、自证不过）。
pub fn corrupt_roots(image: &mut MemoryPool, txgs: &[u64]) {
    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
    for txg in txgs {
        let target = target_for_publish(CheckpointTxg(*txg), slots_per_region);
        let device = parameters().region_devices[usize::try_from(target.region).expect("区域")];
        image.flip_byte(device, slot_offset(target, 4096), 100);
    }
}

/// checker 判红的不变量（名字 + 首条细节截短）。
pub fn checker_reds(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => {
                Some(format!("{invariant}:{}", detail.chars().take(160).collect::<String>()))
            }
            _ => None,
        })
        .collect()
}

/// 恢复一遍，判落到的那一版读回的内容对不对：`Ok((实例, txg))` 或 `Err(说明)`。
pub fn recover_and_judge(
    image: &MemoryPool,
    contents: &BTreeMap<(u32, u64), Vec<u8>>,
) -> Result<(u32, u64), String> {
    let report = recover(image, JournalPolicy::Consult);
    let effective = report.effective_root;
    match report.outcome {
        RecoveryOutcome::FileRead { root, content } => {
            let root = effective.unwrap_or(root);
            match contents.get(&(root.0 .0, root.1 .0)) {
                Some(expected) if *expected == content => Ok((root.0 .0, root.1 .0)),
                Some(_) => Err(format!("落在 ({}, {})，读回的内容不是那一版的", root.0 .0, root.1 .0)),
                None => Err(format!("落在 ({}, {})，没有这一版的记录", root.0 .0, root.1 .0)),
            }
        }
        RecoveryOutcome::NoFile { root } => {
            // 第一个文件之前的根（mkfs 与第一个事务的暖机，txg ≤ 2）下面本来就没有文件。
            if root.1 .0 <= 2 && !contents.contains_key(&(root.0 .0, root.1 .0)) {
                Ok((root.0 .0, root.1 .0))
            } else {
                Err(format!("落在 ({}, {})，报没有文件", root.0 .0, root.1 .0))
            }
        }
        RecoveryOutcome::Failed { root, failure } => Err(format!(
            "落在 {:?}，走读失败：{}",
            root.map(|(i, t)| (i.0, t.0)),
            format!("{failure:?}").chars().take(200).collect::<String>()
        )),
    }
}

/// 对每一条环里的根 X：把比 X 新的根全改坏（只改根槽，记录不动），恢复落在哪、对不对。交回 (改坏几条, 落点或错) 的表，按改坏条数升序。
pub fn landing_table(
    image: &MemoryPool,
    contents: &BTreeMap<(u32, u64), Vec<u8>>,
) -> Vec<(usize, u64, Result<(u32, u64), String>)> {
    let roots = roots_of(image);
    let mut table = Vec::new();
    for (index, root) in roots.iter().enumerate().rev() {
        let newer: Vec<u64> = roots[index + 1..]
            .iter()
            .map(|newer_root| newer_root.checkpoint_txg.0)
            .collect();
        let mut damaged = image.clone();
        corrupt_roots(&mut damaged, &newer);
        table.push((newer.len(), root.checkpoint_txg.0, recover_and_judge(&damaged, contents)));
    }
    table
}

pub fn first_bad(table: &[(usize, u64, Result<(u32, u64), String>)]) -> String {
    table
        .iter()
        .find(|(_, _, verdict)| verdict.is_err())
        .map_or_else(
            || "none".to_string(),
            |(killed, txg, verdict)| format!("k={killed} (land≈txg {txg}) {verdict:?}"),
        )
}

pub fn txgs(outputs: &[TransactionOutput]) -> Vec<u64> {
    outputs.iter().map(|output| output.root.checkpoint_txg.0).collect()
}

pub fn mounted_txgs(mounted: &Mounted) -> Vec<u64> {
    std::iter::once(&mounted.output.row_publish)
        .chain(&mounted.output.warm_up_publishes)
        .map(|published: &PoolVersion| published.root().checkpoint_txg.0)
        .collect()
}

pub fn units_written(mounted: &Mounted) -> Vec<usize> {
    std::iter::once(&mounted.output.row_publish)
        .chain(&mounted.output.warm_up_publishes)
        .map(|published: &PoolVersion| match published {
            PoolVersion::WithFile(version) => version.rewritten.len(),
            PoolVersion::WithoutFile(_) => 0,
        })
        .collect()
}

#[allow(dead_code)]
pub fn device_count() -> u64 {
    IMAGE_BYTES
}

/// A(1,3) B(1,4) C(1,5)，关掉，重开走向前发布回退到 (1,3)。
fn through_c_then_rollback(tag: &str, arm: PrototypeForwardArm) -> (Session, Open, PrototypeForwardCounts, Mounted) {
    let (mut session, mut open) = start(tag);
    session.overwrite(&mut open, &content_of(1));
    session.overwrite(&mut open, &content_of(2));
    drop(open);
    let (open, counts, mounted) = session.rollback_forward((1, 3), arm).expect("向前回退");
    (session, open, counts, mounted)
}

#[test]
fn t1_forward_rollback_basic() {
    for arm in [
        PrototypeForwardArm::Full,
        PrototypeForwardArm::ReleaseOnlyNoRevive,
        PrototypeForwardArm::BirthPrunedRelease,
    ] {
        let (mut session, mut open, counts, mounted) = through_c_then_rollback("t1", arm);
        let image = session.image();
        let reds_after = checker_reds(&image);
        let landing = recover_and_judge(&image, &session.contents);
        let under_b = walk_under(&image, 1, 4).map(|c| c == Some(content_of(1)));
        let under_c = walk_under(&image, 1, 5).map(|c| c == Some(content_of(2)));
        println!(
            "RBF t1 arm={arm:?} instance={} txgs={:?} units_written={:?} counts={counts:?} recover={landing:?} under_B_ok={under_b:?} under_C_ok={under_c:?} checker_reds={reds_after:?}",
            mounted.output.instance.0,
            mounted_txgs(&mounted),
            units_written(&mounted)
        );
        let third = session.try_overwrite(&mut open, &content_of(3)).map(|o| o.root.checkpoint_txg.0);
        let fourth = session.try_overwrite(&mut open, &content_of(4)).map(|o| o.root.checkpoint_txg.0);
        let image = session.image();
        println!(
            "RBF t1b arm={arm:?} overwrites={third:?},{fourth:?} recover={:?} under_B_ok={:?} under_C_ok={:?} checker_reds={:?}",
            recover_and_judge(&image, &session.contents),
            walk_under(&image, 1, 4).map(|c| c == Some(content_of(1))),
            walk_under(&image, 1, 5).map(|c| c == Some(content_of(2))),
            checker_reds(&image)
        );
    }
}

/// 刚回退又回退：A B C → 回退到 A（6、7）→ D（8）→ 关掉 → 第二次回退到 B / C / (2,7)；两臂：精确差集、按诞生代号剪枝。
#[test]
fn t3_rollback_again_after_a_forward_rollback() {
    for (between, second_target) in [
        ("overwrite", (1u32, 4u64)),
        ("overwrite", (1, 5)),
        ("overwrite", (2, 7)),
        ("new_inodes", (1, 4)),
        ("new_inodes", (1, 5)),
        ("nothing", (1, 4)),
        ("nothing", (1, 5)),
    ] {
        for arm in [PrototypeForwardArm::Full, PrototypeForwardArm::BirthPrunedRelease] {
            let (mut session, mut open, _first_counts, _first) =
                through_c_then_rollback("t3", PrototypeForwardArm::Full);
            match between {
                "overwrite" => {
                    session.overwrite(&mut open, &content_of(3));
                }
                "new_inodes" => {
                    session.new_inodes(&mut open, 5);
                }
                _ => {}
            }
            drop(open);
            let result = session.rollback_forward(second_target, arm);
            match result {
                Ok((mut open, counts, mounted)) => {
                    let image = session.image();
                    let reds = checker_reds(&image);
                    let landing = recover_and_judge(&image, &session.contents);
                    let more = session.try_overwrite(&mut open, &content_of(5)).map(|o| o.root.checkpoint_txg.0);
                    let image2 = session.image();
                    println!(
                        "RBF t3 between={between} target={second_target:?} arm={arm:?} txgs={:?} counts={counts:?} recover={landing:?} checker_reds={reds:?} then_overwrite={more:?} reds_after={:?} under_B={:?} under_C={:?} under_D={:?}",
                        mounted_txgs(&mounted),
                        checker_reds(&image2),
                        walk_under(&image2, 1, 4).map(|c| c == Some(content_of(1))),
                        walk_under(&image2, 1, 5).map(|c| c == Some(content_of(2))),
                        walk_under(&image2, 2, 8).map(|c| c == Some(content_of(3))),
                    );
                }
                Err(error) => println!("RBF t3 between={between} target={second_target:?} arm={arm:?} refused={error:?}"),
            }
        }
    }
}

#[test]
fn t3_debug_inode_watermark() {
    let (mut session, mut open, _c, _m) = through_c_then_rollback("t3d", PrototypeForwardArm::Full);
    let n = session.new_inodes(&mut open, 5);
    println!("RBF t3d N txg={} containers={:?} wm={}", n.root.checkpoint_txg.0,
        n.inode_leaf_containers.iter().map(|c| format!("{:?}", c.contents)).collect::<Vec<_>>().iter().map(|s| s.chars().take(300).collect::<String>()).collect::<Vec<_>>(), n.inode_number_watermark());
    drop(open);
    let (open, _counts, mounted) = session.rollback_forward((1, 4), PrototypeForwardArm::Full).expect("回退");
    let row = mounted.output.row_publish.clone().into_file_version().unwrap();
    println!("RBF t3d row txg={} containers={:?} wm={} rewritten={:?}", row.root.checkpoint_txg.0,
        row.inode_leaf_containers.iter().map(|c| format!("{:?}", c.contents)).collect::<Vec<_>>().iter().map(|s| s.chars().take(300).collect::<String>()).collect::<Vec<_>>(), row.inode_number_watermark(), row.rewritten);
    println!("RBF t3d cur wm={}", open.current.inode_number_watermark());
}

use singlefs_harness::crash::{
    enumerate_layer0_selecting_versions_observing_each_state, writes_and_segments, CrashImage,
    PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

fn versions_of(contents: &BTreeMap<(u32, u64), Vec<u8>>) -> Vec<PublishedVersion> {
    contents
        .iter()
        .map(|((instance, txg), content)| PublishedVersion {
            instance: InstanceGeneration(*instance),
            checkpoint_txg: CheckpointTxg(*txg),
            content: content.clone(),
        })
        .collect()
}

/// 对一条流（`base` 之后录下的 `session.stream`）做快的崩溃枚举（10 写以上的段不展开），每个状态另加两道探针：
/// 最新的一条可读根改坏、最新的两条改坏，恢复落在哪、读回的内容对不对。
fn crash_enumerate(name: &str, base: &MemoryPool, session: &Session) {
    let operations = session.stream.retained_operations();
    let (writes, segments) = writes_and_segments(&operations, &common::geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write: &RetainedWrite| write.kind == StepKind::RootRecordFua)
        .expect("流里有根槽写");
    let versions = versions_of(&session.contents);
    let full = std::env::var("RBF_FULL").is_ok();
    let expand = |_index: usize, segment: &[usize]| full || segment.len() < 10;
    println!(
        "RBF crash name={name} closed_form_full={} segment_lengths={:?} full={full}",
        singlefs_harness::crash::closed_form_state_count(&segments),
        segments.iter().map(Vec::len).collect::<Vec<_>>()
    );
    let mut probe_states = 0u64;
    let mut state_index = 0u64;
    let mut probe_bad = [0u64; 3];
    let mut probe_first_bad: [Option<String>; 3] = [None, None, None];
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        base,
        &writes,
        &segments,
        judged_root_index,
        &versions,
        &expand,
        &mut |image: &CrashImage<'_>, _report| {
            if std::env::var("RBF_NOPROBE").is_ok() {
                return;
            }
            let every: u64 = std::env::var("RBF_PROBE_EVERY").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
            state_index += 1;
            if (state_index - 1) % every != 0 {
                return;
            }
            let mut materialized = image.base.clone();
            let persisted: Vec<RetainedWrite> = image
                .writes
                .iter()
                .zip(&image.persisted)
                .filter(|(_, persisted)| **persisted)
                .map(|(write, _)| write.clone())
                .collect();
            materialized.apply_writes(&persisted);
            probe_states += 1;
            let roots = roots_of(&materialized);
            for killed in 1..=3usize {
                if roots.len() <= killed {
                    continue;
                }
                let newest: Vec<u64> = roots[roots.len() - killed..]
                    .iter()
                    .map(|root| root.checkpoint_txg.0)
                    .collect();
                let mut damaged = materialized.clone();
                corrupt_roots(&mut damaged, &newest);
                if let Err(reason) = recover_and_judge(&damaged, &session.contents) {
                    probe_bad[killed - 1] += 1;
                    if probe_first_bad[killed - 1].is_none() {
                        probe_first_bad[killed - 1] = Some(format!("killed={newest:?} {reason}"));
                    }
                }
            }
        },
    );
    println!(
        "RBF crash name={name} writes={} states={} violations={} failed={} first_violation={} probe_states={probe_states} probe_bad(kill1,kill2,kill3)={probe_bad:?} probe_first_bad={probe_first_bad:?}",
        writes.len(),
        tally.states,
        tally.violations,
        tally.failed_states,
        tally.first_violation.as_deref().unwrap_or("none"),
    );
    let reds: Vec<String> = tally
        .checker_violated_states
        .iter()
        .map(|(invariant, count)| {
            format!(
                "{invariant}={count}:{}",
                tally
                    .checker_first_violation
                    .get(invariant)
                    .map(|s| s.chars().take(140).collect::<String>())
                    .unwrap_or_default()
            )
        })
        .collect();
    println!("RBF crash name={name} checker_violated={reds:?}");
}

#[test]
fn t4_crash_points_of_the_forward_rollback_stream() {
    let (mut session, mut open) = start("t4");
    session.overwrite(&mut open, &content_of(1));
    session.overwrite(&mut open, &content_of(2));
    drop(open);
    // 基线取到 C 为止：流只含回退那次挂载。
    let base = session.image();
    session.stream = SharedStream::retaining_contents();
    let devices = common::crash_state_devices(&base, &[], &[], &session.stream);
    session.devices = devices;
    let (_open, counts, mounted) = session
        .rollback_forward((1, 3), PrototypeForwardArm::Full)
        .expect("向前回退");
    println!("RBF t4 txgs={:?} counts={counts:?}", mounted_txgs(&mounted));
    crash_enumerate("forward-rollback", &base, &session);
}

#[test]
fn t5a_crash_points_of_the_unmount_chain() {
    let (mut session, mut open) = start("t5a");
    session.overwrite(&mut open, &content_of(1));
    session.overwrite(&mut open, &content_of(2));
    let base = session.image();
    session.stream = SharedStream::retaining_contents();
    session.devices = common::crash_state_devices(&base, &[], &[], &session.stream);
    let chain = session.unmount_b(&mut open);
    println!("RBF t5a chain_txgs={:?} units_written={:?} writes={:?}", txgs(&chain),
        chain.iter().map(|o| o.rewritten.len()).collect::<Vec<_>>(),
        chain.iter().map(|o| { let t = o.writes.total(); (t.write_calls, t.written_bytes) }).collect::<Vec<_>>());
    crash_enumerate("unmount-b", &base, &session);
}

/// B 之后再挂载、复用：A B C →（臂：卸载时抬 F / 普通关掉）→ 重开可写挂载 → D E。对环里每条根 X 把比它新的根全改坏，恢复落在哪、对不对。
#[test]
fn t5b_landing_after_unmount_b_and_reuse() {
    for (with_rollback, with_b) in [(false, false), (false, true), (true, false), (true, true)] {
        let (mut session, mut open) = start("t5b");
        session.overwrite(&mut open, &content_of(1));
        session.overwrite(&mut open, &content_of(2));
        let mut chain = Vec::new();
        if with_rollback {
            drop(open);
            let (rolled, _counts, _m) = session
                .rollback_forward((1, 3), PrototypeForwardArm::Full)
                .expect("向前回退");
            open = rolled;
        }
        if with_b {
            chain = session.unmount_b(&mut open);
        }
        drop(open);
        let mut reopened = session.mount();
        let d = session.overwrite(&mut reopened, &content_of(3));
        let e = session.overwrite(&mut reopened, &content_of(4));
        let image = session.image();
        let reds = checker_reds(&image);
        let table = landing_table(&image, &session.contents);
        println!(
            "RBF t5b rollback={with_rollback} b={with_b} chain={:?} D_data_slot={} E_data_slot={} newest={} checker_reds={reds:?} first_bad={}",
            txgs(&chain),
            d.data_pointers[0].locations[0].slot.0,
            e.data_pointers[0].locations[0].slot.0,
            e.root.checkpoint_txg.0,
            first_bad(&table)
        );
        for (killed, txg, verdict) in &table {
            println!("RBF t5b-row rollback={with_rollback} b={with_b} killed={killed} land_txg={txg} verdict={verdict:?}");
        }
    }
}

/// B 之后 F 回落（C419 那一形）：A B C → B 抬 F → 重开 → D E；改坏盘 1 上带新 F 的根，F_生效 回落，再回退到 F 之下的根。
#[test]
fn t6_floor_falls_back_after_b_and_a_rollback_below_it() {
    for arm in [PrototypeForwardArm::Full, PrototypeForwardArm::FullWithoutProtectionCheck] {
        for target in [(1u32, 3u64), (1, 4)] {
            let (mut session, mut open) = start("t6");
            session.overwrite(&mut open, &content_of(1));
            session.overwrite(&mut open, &content_of(2));
            let chain = session.unmount_b(&mut open);
            drop(open);
            let mut reopened = session.mount();
            session.overwrite(&mut reopened, &content_of(3));
            session.overwrite(&mut reopened, &content_of(4));
            drop(reopened);
            let image = session.image();
            // 带新 F 的根：F 字段非 0 的根；按根环落点公式挑出落在盘 1 上的那几条。
            let floor_roots: Vec<(u64, u64)> = roots_of(&image)
                .iter()
                .filter(|root| root.rollback_floor.0 > 0)
                .map(|root| (root.checkpoint_txg.0, root.rollback_floor.0))
                .collect();
            let slots_per_region = parameters().geometry.root_ring_slots_per_region;
            let on_device_one: Vec<u64> = floor_roots
                .iter()
                .map(|(txg, _)| *txg)
                .filter(|txg| {
                    parameters().region_devices[usize::try_from(
                        target_for_publish(CheckpointTxg(*txg), slots_per_region).region,
                    )
                    .expect("区域")]
                        == DeviceIdentity(1)
                })
                .collect();
            let mut damaged = image.clone();
            corrupt_roots(&mut damaged, &on_device_one);
            let effective_floor = singlefs_core::recovery::effective_rollback_floor(
                &damaged,
                &parameters().region_devices,
                &choose_system_configuration(&damaged).expect("系统配置").immutable.sizes,
                &choose_system_configuration(&damaged).expect("系统配置").immutable.filesystem_identifier,
            );
            let recovered = recover_and_judge(&damaged, &session.contents);
            // 在改坏之后的盘上发回退。
            let stream = SharedStream::retaining_contents();
            let mut devices = common::crash_state_devices(&damaged, &[], &[], &stream);
            let result = prototype_mount_rollback_forward(
                &parameters(),
                &mut devices,
                RollbackTarget {
                    instance: InstanceGeneration(target.0),
                    checkpoint_txg: CheckpointTxg(target.1),
                },
                arm,
            );
            let summary = match &result {
                Ok((mounted, _counts)) => {
                    let after = common::memory_pool_of_sparse_devices(&devices);
                    let root = mounted.current.root();
                    format!(
                        "accepted newest=({}, {}) read={:?} checker_reds={:?}",
                        root.instance.0,
                        root.checkpoint_txg.0,
                        walk_to_file(&after, root, &mut 0).map(|c| c == session.contents.get(&target).cloned()),
                        checker_reds(&after)
                    )
                }
                Err(error) => format!("refused {:?}", error).chars().take(300).collect(),
            };
            println!(
                "RBF t6 arm={arm:?} target={target:?} chain={:?} floor_roots={floor_roots:?} corrupted_on_dev1={on_device_one:?} effective_floor_after={} recover={recovered:?} rollback={summary}",
                txgs(&chain),
                effective_floor.0
            );
        }
    }
}

use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::prototype_mount_writable_with_shadow_ledger;
use singlefs_harness::fault_injection::{
    FaultInjectingBlockDevice, FaultSchedule, NamedRootRingSlots, RootRingSlotTarget,
    SharedFaultPlan,
};

/// 不带管理员回退的「被实例表抛弃的根」：A B（实例 1）→ 重开（实例 2）C → 重开时实例 2 的根暂时全读不出，恢复落到 B、
/// 实例 3 写行 (1, 4, W)、(2, 0, 0) 抛弃实例 2 的根，接着发 D E → 撤故障（实例 2 的根又读得出）→ 之后实例 3 的根全读不出。
/// 两臂：影子账开 / 关。交回 (会话, C 的 txg, 实例 3 的根 txg)。
fn crash_abandonment_history(shadow_ledger: ShadowLedger) -> (Session, u64, Vec<u64>, u64) {
    let (mut session, mut open) = start("t7");
    session.overwrite(&mut open, &content_of(1));
    drop(open);
    let mut second = session.mount();
    let c = session.overwrite(&mut second, &content_of(2));
    drop(second);
    let image = session.image();
    let instance_two: Vec<u64> = roots_of(&image)
        .iter()
        .filter(|root| root.instance == InstanceGeneration(2))
        .map(|root| root.checkpoint_txg.0)
        .collect();
    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
    let target = RootRingSlotTarget {
        named_slots: NamedRootRingSlots::naming(
            &instance_two
                .iter()
                .map(|txg| target_for_publish(CheckpointTxg(*txg), slots_per_region))
                .collect::<Vec<_>>(),
        ),
        region_devices: parameters().region_devices,
        fixed_structure_slot_spacing: parameters().geometry.fixed_structure_slot_spacing,
    };
    let plan = SharedFaultPlan::armed(
        common::geometry(),
        FaultSchedule::every_read_of_named_root_ring_slots_fails(target),
    );
    let devices = std::mem::take(&mut session.devices);
    let mut failing: Vec<_> = devices
        .into_iter()
        .map(|(identity, device)| (identity, FaultInjectingBlockDevice::new(identity, device, plan.clone())))
        .collect();
    let mounted = prototype_mount_writable_with_shadow_ledger(
        &parameters(),
        &mut failing,
        SpaceAdmission::JudgedByTheFormula,
        shadow_ledger,
    )
    .expect("带读故障的可写挂载");
    let effective = mounted.output.effective_root;
    assert_eq!(
        (effective.instance.0, effective.checkpoint_txg.0),
        (1, 4),
        "实例 2 的根读不出，恢复落到 B"
    );
    let b_content = session.contents.get(&(1, 4)).cloned().expect("B");
    for published in std::iter::once(&mounted.output.row_publish).chain(&mounted.output.warm_up_publishes) {
        let root = published.root();
        session.contents.insert((root.instance.0, root.checkpoint_txg.0), b_content.clone());
    }
    let mut third = Open {
        allocator: mounted.allocator,
        current: mounted.current.into_file_version().expect("带文件"),
        instance: mounted.output.instance,
    };
    let isolated = mounted.output.isolated_slots_per_device.clone();
    for seed in [3usize, 4] {
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, failing.as_mut_slice());
        let content = content_of(seed);
        let output = publish_overwrite(
            &mut writer,
            &mut third.allocator,
            &third.current,
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            third.instance,
        )
        .expect("D / E");
        session.contents.insert((output.root.instance.0, output.root.checkpoint_txg.0), content);
        third.current = output;
    }
    plan.disarm();
    session.devices = failing.into_iter().map(|(identity, device)| (identity, device.into_inner())).collect();
    let instance_three: Vec<u64> = roots_of(&session.image())
        .iter()
        .filter(|root| root.instance == third.instance)
        .map(|root| root.checkpoint_txg.0)
        .collect();
    println!("RBF t7-history shadow={shadow_ledger:?} instance_two={instance_two:?} isolated={isolated:?} instance_three={instance_three:?}");
    (session, c.root.checkpoint_txg.0, instance_three, u64::from(third.instance.0))
}

#[test]
fn t7_shadow_ledger_without_any_admin_rollback() {
    for shadow_ledger in [ShadowLedger::On, ShadowLedger::Off] {
        let (session, c_txg, instance_three, _) = crash_abandonment_history(shadow_ledger);
        let image = session.image();
        let under_c = walk_under(&image, 2, c_txg).map(|c| c == Some(content_of(2)));
        let mut damaged = image.clone();
        corrupt_roots(&mut damaged, &instance_three);
        println!(
            "RBF t7 shadow={shadow_ledger:?} checker_reds={:?} under_C_intact={under_c:?} after_killing_instance_three({})={:?} checker_reds_damaged={:?}",
            checker_reds(&image),
            instance_three.len(),
            recover_and_judge(&damaged, &session.contents),
            checker_reds(&damaged)
        );
    }
}

#[test]
fn t8_rollback_to_a_root_the_instance_table_abandoned() {
    for arm in [PrototypeForwardArm::Full, PrototypeForwardArm::FullWithoutTableValidityCheck] {
        let (mut session, c_txg, _instance_three, _) = crash_abandonment_history(ShadowLedger::On);
        let result = session.rollback_forward((2, c_txg), arm);
        match result {
            Ok((_open, counts, mounted)) => {
                let image = session.image();
                println!(
                    "RBF t8 arm={arm:?} accepted txgs={:?} revived={} released={} recover={:?} checker_reds={:?}",
                    mounted_txgs(&mounted),
                    counts.revived_placements,
                    counts.released_placements,
                    recover_and_judge(&image, &session.contents),
                    checker_reds(&image)
                );
            }
            Err(error) => println!("RBF t8 arm={arm:?} refused={error:?}"),
        }
    }
}

use singlefs_core::transaction::publish_sequential_write;

impl Session {
    pub fn sequential_write(&mut self, open: &mut Open, content: &[u8]) -> TransactionOutput {
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        let output = publish_sequential_write(
            &mut writer,
            &mut open.allocator,
            &open.current,
            FirstFile { content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 30 },
            open.instance,
        )
        .expect("顺序写");
        self.contents.insert((output.root.instance.0, output.root.checkpoint_txg.0), content.to_vec());
        open.current = output.clone();
        output
    }
}

/// F4：回退跨 k = 1..4 个非空状态，三档负载：每代覆盖写一个单元的文件（W1）、每代建 500 个 inode（W2）、每代顺序写一个 20 单元的文件（W3）。
#[test]
fn t9_cost_of_the_forward_rollback_publish() {
    for workload in ["W1-overwrite-one-unit", "W2-new-inodes-500", "W3-sequential-20-units"] {
        for k in 1..=4usize {
            let (mut session, mut open) = start("t9");
            for generation in 0..k {
                match workload {
                    "W1-overwrite-one-unit" => {
                        session.overwrite(&mut open, &content_of(generation + 1));
                    }
                    "W2-new-inodes-500" => {
                        session.new_inodes(&mut open, 500);
                    }
                    _ => {
                        let content: Vec<u8> = (0..20 * 32000 + generation)
                            .map(|index| u8::try_from((index * 13 + generation) % 251).expect("小"))
                            .collect();
                        session.sequential_write(&mut open, &content);
                    }
                }
            }
            let newest_txg = open.current.root.checkpoint_txg.0;
            drop(open);
            let (_open, counts, mounted) = session
                .rollback_forward((1, 3), PrototypeForwardArm::Full)
                .expect("向前回退");
            let image = session.image();
            let reds = checker_reds(&image);
            println!(
                "RBF t9 workload={workload} k={k} newest_before={newest_txg} rollback_txgs={:?} units_written={:?} released={}p/{}s revived={}p/{}s pruned_nodes_read={} pruned_data_units={} target_alloc_nodes={} newest_alloc_nodes={} born_after_by_family={:?} recover={:?} checker_red_names={:?}",
                mounted_txgs(&mounted),
                units_written(&mounted),
                counts.released_placements,
                counts.released_slots,
                counts.revived_placements,
                counts.revived_slots,
                counts.birth_pruned_nodes_read,
                counts.birth_pruned_data_units,
                counts.target_allocation_record_tree_nodes,
                counts.newest_allocation_record_tree_nodes,
                counts.newest_born_after_target_by_family,
                recover_and_judge(&image, &session.contents).is_ok(),
                reds.iter().map(|r| r.split(':').next().unwrap_or("").to_string()).collect::<Vec<_>>()
            );
        }
    }
}

/// F4：卸载那一串发几次、写几个单元，按上面三档负载各跑 4 代之后卸载。
#[test]
fn t10_cost_of_the_unmount_chain() {
    for workload in ["W1-overwrite-one-unit", "W2-new-inodes-500", "W3-sequential-20-units"] {
        let (mut session, mut open) = start("t10");
        for generation in 0..4usize {
            match workload {
                "W1-overwrite-one-unit" => {
                    session.overwrite(&mut open, &content_of(generation + 1));
                }
                "W2-new-inodes-500" => {
                    session.new_inodes(&mut open, 500);
                }
                _ => {
                    let content: Vec<u8> = (0..20 * 32000 + generation)
                        .map(|index| u8::try_from((index * 13 + generation) % 251).expect("小"))
                        .collect();
                    session.sequential_write(&mut open, &content);
                }
            }
        }
        let deferred_before: u64 = open.allocator.devices[0].deferred_slots();
        let chain = session.unmount_b(&mut open);
        let deferred_after: u64 = open.allocator.devices[0].deferred_slots();
        let image = session.image();
        println!(
            "RBF t10 workload={workload} chain_txgs={:?} units_written={:?} write_calls_bytes={:?} deferred_slots_dev0_before={deferred_before} after={deferred_after} checker_red_names={:?}",
            txgs(&chain),
            chain.iter().map(|o| o.rewritten.len()).collect::<Vec<_>>(),
            chain.iter().map(|o| { let t = o.writes.total(); (t.write_calls, t.written_bytes) }).collect::<Vec<_>>(),
            checker_reds(&image).iter().map(|r| r.split(':').next().unwrap_or("").to_string()).collect::<Vec<_>>()
        );
    }
}

impl Session {
    pub fn replace_image(&mut self, image: &MemoryPool) {
        self.devices = common::crash_state_devices(image, &[], &[], &self.stream);
    }

    pub fn open_from(&mut self, mounted: Mounted) -> Open {
        let effective = mounted.output.effective_root;
        let content = self
            .contents
            .get(&(effective.instance.0, effective.checkpoint_txg.0))
            .cloned()
            .expect("所选那一版的内容记着");
        self.note_mount(&mounted, &content);
        Open {
            instance: mounted.output.instance,
            allocator: mounted.allocator,
            current: mounted.current.into_file_version().expect("带文件"),
        }
    }
}

/// 影子账在没有管理员回退的历史上承不承重：A B C（实例 1）；C 那条记录两份都坏；重开时 C 的根暂时读不出 ⇒ 恢复落到 B、
/// 实例 2 写行 (1, 4, W) 抛弃 C（只写行与暖机，不发用户数据）；撤故障，C 的根又读得出；再重开（实例 3，影子账两臂）发 D E；
/// 最后实例 2、3 的根全读不出，恢复落在 C 上，读回的是不是 C。
#[test]
fn t7b_shadow_ledger_protects_a_root_the_instance_table_abandoned_without_any_admin_rollback() {
    for shadow_ledger in [ShadowLedger::On, ShadowLedger::Off] {
        let (mut session, mut open) = start("t7b");
        session.overwrite(&mut open, &content_of(1));
        let c = session.overwrite(&mut open, &content_of(2));
        drop(open);
        let mut image = session.image();
        let offset = singlefs_core::journal::record_offset(
            c.record.counter,
            parameters().geometry.journal_ring_bytes,
        );
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            image.flip_byte(device, offset, 300);
        }
        session.replace_image(&image);
        let slots_per_region = parameters().geometry.root_ring_slots_per_region;
        let plan = SharedFaultPlan::armed(
            common::geometry(),
            FaultSchedule::every_read_of_named_root_ring_slots_fails(RootRingSlotTarget {
                named_slots: NamedRootRingSlots::naming(&[target_for_publish(
                    c.root.checkpoint_txg,
                    slots_per_region,
                )]),
                region_devices: parameters().region_devices,
                fixed_structure_slot_spacing: parameters().geometry.fixed_structure_slot_spacing,
            }),
        );
        let devices = std::mem::take(&mut session.devices);
        let mut failing: Vec<_> = devices
            .into_iter()
            .map(|(identity, device)| (identity, FaultInjectingBlockDevice::new(identity, device, plan.clone())))
            .collect();
        let second = prototype_mount_writable_with_shadow_ledger(
            &parameters(),
            &mut failing,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
        )
        .expect("带读故障的可写挂载");
        plan.disarm();
        session.devices = failing.into_iter().map(|(identity, device)| (identity, device.into_inner())).collect();
        let second_effective = (second.output.effective_root.instance.0, second.output.effective_root.checkpoint_txg.0);
        let second_txgs = mounted_txgs(&second);
        drop(session.open_from(second));
        let third = prototype_mount_writable_with_shadow_ledger(
            &parameters(),
            &mut session.devices,
            SpaceAdmission::JudgedByTheFormula,
            shadow_ledger,
        )
        .expect("撤故障之后的可写挂载");
        let isolated = third.output.isolated_slots_per_device.clone();
        let third_txgs = mounted_txgs(&third);
        let mut open3 = session.open_from(third);
        let d = session.overwrite(&mut open3, &content_of(3));
        let e = session.overwrite(&mut open3, &content_of(4));
        let image = session.image();
        let newer_than_c: Vec<u64> = roots_of(&image)
            .iter()
            .filter(|root| root.checkpoint_txg > c.root.checkpoint_txg)
            .map(|root| root.checkpoint_txg.0)
            .collect();
        let mut damaged = image.clone();
        corrupt_roots(&mut damaged, &newer_than_c);
        println!(
            "RBF t7b shadow={shadow_ledger:?} second_effective={second_effective:?} second_txgs={second_txgs:?} third_txgs={third_txgs:?} isolated={isolated:?} C_data_slot={} D_data_slot={} E_data_slot={} checker_reds={:?} under_C_intact={:?} kill_newer_than_C({})={:?}",
            c.data_pointers[0].locations[0].slot.0,
            d.data_pointers[0].locations[0].slot.0,
            e.data_pointers[0].locations[0].slot.0,
            checker_reds(&image).iter().map(|r| r.chars().take(120).collect::<String>()).collect::<Vec<_>>(),
            walk_under(&image, 1, c.root.checkpoint_txg.0).map(|content| content == Some(content_of(2))),
            newer_than_c.len(),
            recover_and_judge(&damaged, &session.contents)
        );
    }
}

use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use std::cell::Cell;
use std::rc::Rc;

/// 读落在点名字节区间里就失败（暂时读不出）；`armed` 关掉之后照常读。
pub struct FailReads<Inner> {
    pub inner: Inner,
    pub device: DeviceIdentity,
    pub ranges: Vec<(DeviceIdentity, u64, u64)>,
    pub armed: Rc<Cell<bool>>,
}

impl<Inner: BlockDevice> BlockDevice for FailReads<Inner> {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let end = offset.0 + u64::try_from(buffer.len()).expect("长度");
        if self.armed.get()
            && self
                .ranges
                .iter()
                .any(|(device, start, stop)| *device == self.device && offset.0 < *stop && *start < end)
        {
            return Err(singlefs_harness::fault_injection::injected_block_device_error("读"));
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

/// 同 t7b，但 C 的记录完好、C 的根槽与 C 的数据单元暂时读不出：恢复落到 B（记录施加前验点名单元失败），
/// 新实例第一次发布的 txg 按环里读得出的记录取 max + 1（不盖 C 的根槽）。撤故障之后 C 的根又读得出、被实例表抛弃。
#[test]
fn t7c_shadow_ledger_protects_a_root_the_instance_table_abandoned_without_any_admin_rollback() {
    for shadow_ledger in [ShadowLedger::On, ShadowLedger::Off] {
        let (mut session, mut open) = start("t7c");
        session.overwrite(&mut open, &content_of(1));
        let c = session.overwrite(&mut open, &content_of(2));
        drop(open);
        let slots_per_region = parameters().geometry.root_ring_slots_per_region;
        let root_target = target_for_publish(c.root.checkpoint_txg, slots_per_region);
        let root_device = parameters().region_devices[usize::try_from(root_target.region).expect("区域")];
        let root_offset = slot_offset(root_target, 4096).0;
        let data_slot = c.data_pointers[0].locations[0].slot.0;
        let mut ranges = vec![(root_device, root_offset, root_offset + 4096)];
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            ranges.push((device, data_slot * 16384, data_slot * 16384 + 32768));
        }
        let armed = Rc::new(Cell::new(true));
        let devices = std::mem::take(&mut session.devices);
        let mut failing: Vec<_> = devices
            .into_iter()
            .map(|(identity, device)| {
                (identity, FailReads { inner: device, device: identity, ranges: ranges.clone(), armed: armed.clone() })
            })
            .collect();
        let second = prototype_mount_writable_with_shadow_ledger(
            &parameters(),
            &mut failing,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
        );
        armed.set(false);
        session.devices = failing.into_iter().map(|(identity, device)| (identity, device.inner)).collect();
        let second = match second {
            Ok(second) => second,
            Err(error) => {
                println!("RBF t7c shadow={shadow_ledger:?} second_mount_refused={error:?}");
                continue;
            }
        };
        let second_effective = (second.output.effective_root.instance.0, second.output.effective_root.checkpoint_txg.0);
        let second_txgs = mounted_txgs(&second);
        drop(session.open_from(second));
        let third = prototype_mount_writable_with_shadow_ledger(
            &parameters(),
            &mut session.devices,
            SpaceAdmission::JudgedByTheFormula,
            shadow_ledger,
        )
        .expect("撤故障之后的可写挂载");
        let isolated = third.output.isolated_slots_per_device.clone();
        let third_txgs = mounted_txgs(&third);
        let mut open3 = session.open_from(third);
        let d = session.overwrite(&mut open3, &content_of(3));
        let e = session.overwrite(&mut open3, &content_of(4));
        let image = session.image();
        let newer_than_c: Vec<u64> = roots_of(&image)
            .iter()
            .filter(|root| root.checkpoint_txg > c.root.checkpoint_txg)
            .map(|root| root.checkpoint_txg.0)
            .collect();
        let mut damaged = image.clone();
        corrupt_roots(&mut damaged, &newer_than_c);
        println!(
            "RBF t7c shadow={shadow_ledger:?} second_effective={second_effective:?} second_txgs={second_txgs:?} third_txgs={third_txgs:?} isolated={isolated:?} C_data_slot={data_slot} D_data_slot={} E_data_slot={} checker_reds={:?} under_C_intact={:?} kill_newer_than_C({})={:?}",
            d.data_pointers[0].locations[0].slot.0,
            e.data_pointers[0].locations[0].slot.0,
            checker_reds(&image).iter().map(|r| r.chars().take(120).collect::<String>()).collect::<Vec<_>>(),
            walk_under(&image, 1, c.root.checkpoint_txg.0).map(|content| content == Some(content_of(2))),
            newer_than_c.len(),
            recover_and_judge(&damaged, &session.contents)
        );
    }
}

/// t7c 那段历史（影子账开）走到「C 的根又读得出、被实例表抛弃」为止，交回会话与 C 的 txg。
fn history_with_a_readable_root_the_table_abandoned(tag: &str) -> (Session, u64) {
    let (mut session, mut open) = start(tag);
    session.overwrite(&mut open, &content_of(1));
    let c = session.overwrite(&mut open, &content_of(2));
    drop(open);
    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
    let root_target = target_for_publish(c.root.checkpoint_txg, slots_per_region);
    let root_device = parameters().region_devices[usize::try_from(root_target.region).expect("区域")];
    let root_offset = slot_offset(root_target, 4096).0;
    let data_slot = c.data_pointers[0].locations[0].slot.0;
    let mut ranges = vec![(root_device, root_offset, root_offset + 4096)];
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        ranges.push((device, data_slot * 16384, data_slot * 16384 + 32768));
    }
    let armed = Rc::new(Cell::new(true));
    let devices = std::mem::take(&mut session.devices);
    let mut failing: Vec<_> = devices
        .into_iter()
        .map(|(identity, device)| (identity, FailReads { inner: device, device: identity, ranges: ranges.clone(), armed: armed.clone() }))
        .collect();
    let second = prototype_mount_writable_with_shadow_ledger(&parameters(), &mut failing, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On)
        .expect("带读故障的可写挂载");
    armed.set(false);
    session.devices = failing.into_iter().map(|(identity, device)| (identity, device.inner)).collect();
    drop(session.open_from(second));
    (session, c.root.checkpoint_txg.0)
}

#[test]
fn t8b_rollback_to_a_readable_root_the_instance_table_abandoned() {
    for arm in [PrototypeForwardArm::Full, PrototypeForwardArm::FullWithoutTableValidityCheck] {
        let (mut session, c_txg) = history_with_a_readable_root_the_table_abandoned("t8b");
        match session.rollback_forward((1, c_txg), arm) {
            Ok((mut open, counts, mounted)) => {
                let image = session.image();
                let reds = checker_reds(&image);
                let more = session.try_overwrite(&mut open, &content_of(7)).map(|o| o.root.checkpoint_txg.0);
                println!(
                    "RBF t8b arm={arm:?} accepted txgs={:?} revived={} released={} recover={:?} checker_reds={:?} then_overwrite={more:?} reds_after={:?}",
                    mounted_txgs(&mounted),
                    counts.revived_placements,
                    counts.released_placements,
                    recover_and_judge(&image, &session.contents),
                    reds.iter().map(|r| r.chars().take(160).collect::<String>()).collect::<Vec<_>>(),
                    checker_reds(&session.image()).iter().map(|r| r.chars().take(160).collect::<String>()).collect::<Vec<_>>()
                );
            }
            Err(error) => println!("RBF t8b arm={arm:?} refused={error:?}"),
        }
    }
}

/// 两棵账有一个节点读不出：(a) R_old 那一版分配记录树的一片叶两份都坏；(b) 最新那一版的一片 inode 叶容器两份都坏。
/// 回退（与普通可写挂载）怎么处置：拒、还是漏。
#[test]
fn t12_one_unreadable_node_in_either_ledger() {
    for case in ["target-allocation-leaf", "newest-inode-leaf", "newest-allocation-leaf"] {
        let (mut session, mut open) = start("t12");
        let a = open.current.clone();
        session.new_inodes(&mut open, 300);
        let newest = session.overwrite(&mut open, &content_of(2));
        drop(open);
        let mut image = session.image();
        let pointer = match case {
            "target-allocation-leaf" => a.allocation_record_tree.nodes.first().expect("叶").1,
            "newest-allocation-leaf" => newest.allocation_record_tree.nodes.first().expect("叶").1,
            _ => newest.inode_leaf_containers.last().expect("叶容器").pointer,
        };
        for location in pointer.locations {
            image.flip_byte(location.device, DeviceOffsetInBytes(location.slot.0 * 16384), 200);
        }
        session.replace_image(&image);
        let rollback = session.rollback_forward((1, 3), PrototypeForwardArm::Full).map(|(_, counts, _)| counts.released_placements);
        let mut plain = session.image();
        let stream = SharedStream::retaining_contents();
        let mut devices = common::crash_state_devices(&mut plain, &[], &[], &stream);
        let mount = mount_writable(&parameters(), &mut devices).map(|m| m.output.instance.0);
        println!(
            "RBF t12 case={case} rollback={:?} plain_mount={:?}",
            rollback.map_err(|e| format!("{e:?}").chars().take(220).collect::<String>()),
            mount.map_err(|e| format!("{e:?}").chars().take(220).collect::<String>())
        );
    }
}

/// 崩溃恢复落到更旧的根之后再回退：t7c 那段历史（C 被实例表抛弃、又读得出）之后向前回退到 A，再发 D E，最后比 C 新的根全读不出。
#[test]
fn t11_rollback_after_recovery_fell_below_a_root() {
    let (mut session, c_txg) = history_with_a_readable_root_the_table_abandoned("t11");
    let (mut open, counts, mounted) = session.rollback_forward((1, 3), PrototypeForwardArm::Full).expect("回退到 A");
    session.overwrite(&mut open, &content_of(3));
    session.overwrite(&mut open, &content_of(4));
    let image = session.image();
    let newer: Vec<u64> = roots_of(&image).iter().filter(|r| r.checkpoint_txg.0 > c_txg).map(|r| r.checkpoint_txg.0).collect();
    let mut damaged = image.clone();
    corrupt_roots(&mut damaged, &newer);
    println!(
        "RBF t11 txgs={:?} isolated={:?} released={} revived={} checker_reds={:?} under_C={:?} kill_newer_than_C({})={:?} landing_first_bad={}",
        mounted_txgs(&mounted),
        mounted.output.isolated_slots_per_device,
        counts.released_placements,
        counts.revived_placements,
        checker_reds(&image).iter().map(|r| r.chars().take(120).collect::<String>()).collect::<Vec<_>>(),
        walk_under(&image, 1, c_txg).map(|c| c == Some(content_of(2))),
        newer.len(),
        recover_and_judge(&damaged, &session.contents),
        first_bad(&landing_table(&image, &session.contents))
    );
}

/// t6 的对照臂：不做 B（普通关掉），同样重开、D E、改坏盘 1 上 txg ≥ 6 的根，再回退到 (1,3)/(1,4)。
#[test]
fn t6c_control_without_b() {
    for arm in [PrototypeForwardArm::Full, PrototypeForwardArm::FullWithoutProtectionCheck] {
        for target in [(1u32, 3u64), (1, 4)] {
            let (mut session, mut open) = start("t6c");
            session.overwrite(&mut open, &content_of(1));
            session.overwrite(&mut open, &content_of(2));
            drop(open);
            let mut reopened = session.mount();
            session.overwrite(&mut reopened, &content_of(3));
            session.overwrite(&mut reopened, &content_of(4));
            drop(reopened);
            let mut damaged = session.image();
            corrupt_roots(&mut damaged, &[7, 10]);
            let stream = SharedStream::retaining_contents();
            let mut devices = common::crash_state_devices(&damaged, &[], &[], &stream);
            let result = prototype_mount_rollback_forward(
                &parameters(),
                &mut devices,
                RollbackTarget { instance: InstanceGeneration(target.0), checkpoint_txg: CheckpointTxg(target.1) },
                arm,
            );
            let summary = match &result {
                Ok((mounted, _)) => {
                    let after = common::memory_pool_of_sparse_devices(&devices);
                    let root = mounted.current.root();
                    format!("accepted newest=({}, {}) read={:?} checker_reds={:?}", root.instance.0, root.checkpoint_txg.0,
                        walk_to_file(&after, root, &mut 0).map(|c| c == session.contents.get(&target).cloned()),
                        checker_reds(&after).iter().map(|r| r.chars().take(100).collect::<String>()).collect::<Vec<_>>())
                }
                Err(error) => format!("refused {error:?}").chars().take(300).collect(),
            };
            println!("RBF t6c arm={arm:?} target={target:?} rollback={summary}");
        }
    }
}

/// t6 放开用户那几步扫一遍：卸载做不做 B × 重开之后写几次（0..=4）× 回退目标（A/B/C）；故障固定为「盘 1 上 txg ≥ 6 的根全改坏」。
/// 臂取 `FullWithoutProtectionCheck`（候选集照条款：按实例表有效 ∧ txg ≥ F_生效 ∧ 见证）。
#[test]
fn t6s_sweep_user_steps() {
    let mut rows = Vec::new();
    for with_b in [false, true] {
        for writes in 0..=4usize {
            for target in [(1u32, 3u64), (1, 4), (1, 5)] {
                let (mut session, mut open) = start("t6s");
                session.overwrite(&mut open, &content_of(1));
                session.overwrite(&mut open, &content_of(2));
                if with_b {
                    session.unmount_b(&mut open);
                }
                drop(open);
                let mut reopened = session.mount();
                for seed in 0..writes {
                    session.overwrite(&mut reopened, &content_of(10 + seed));
                }
                drop(reopened);
                let image = session.image();
                let slots_per_region = parameters().geometry.root_ring_slots_per_region;
                let on_device_one: Vec<u64> = roots_of(&image)
                    .iter()
                    .map(|root| root.checkpoint_txg.0)
                    .filter(|txg| *txg >= 6)
                    .filter(|txg| {
                        parameters().region_devices[usize::try_from(target_for_publish(CheckpointTxg(*txg), slots_per_region).region).expect("区域")]
                            == DeviceIdentity(1)
                    })
                    .collect();
                let mut damaged = image.clone();
                corrupt_roots(&mut damaged, &on_device_one);
                let stream = SharedStream::retaining_contents();
                let mut devices = common::crash_state_devices(&damaged, &[], &[], &stream);
                let result = prototype_mount_rollback_forward(
                    &parameters(),
                    &mut devices,
                    RollbackTarget { instance: InstanceGeneration(target.0), checkpoint_txg: CheckpointTxg(target.1) },
                    PrototypeForwardArm::FullWithoutProtectionCheck,
                );
                let verdict = match &result {
                    Ok((mounted, _)) => {
                        let after = common::memory_pool_of_sparse_devices(&devices);
                        let root = mounted.current.root();
                        let read_ok = walk_to_file(&after, root, &mut 0).map(|c| c == session.contents.get(&target).cloned()) == Ok(true);
                        let reds: Vec<String> = checker_reds(&after)
                            .iter()
                            .map(|r| r.split(':').next().unwrap_or("").to_string())
                            .filter(|name| name != "I-3.9" && name != "I-7.9")
                            .collect();
                        if read_ok && reds.is_empty() { "clean".to_string() } else { format!("HIT read_ok={read_ok} reds={reds:?}") }
                    }
                    Err(error) => format!("refused {}", format!("{error:?}").chars().take(80).collect::<String>()),
                };
                println!("RBF t6s b={with_b} writes={writes} target={target:?} corrupted_dev1={} verdict={verdict}", on_device_one.len());
                rows.push((with_b, verdict.starts_with("HIT")));
            }
        }
    }
    for with_b in [false, true] {
        let hits = rows.iter().filter(|(b, hit)| *b == with_b && *hit).count();
        let total = rows.iter().filter(|(b, _)| *b == with_b).count();
        println!("RBF t6s-summary b={with_b} hits={hits}/{total}");
    }
}
