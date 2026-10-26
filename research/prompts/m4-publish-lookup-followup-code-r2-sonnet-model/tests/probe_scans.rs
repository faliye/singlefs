//! 第二轮正推腿（m4-publish-lookup-followup-code-r2）：以第一轮攻方腿 attack_followup_differential.rs 为底（该文件在 research/prompts/m4-publish-lookup-followup-code-r1-opus-model/tests/，入库），
//! 加两样：随机历史里夹重新挂载（恢复路径调 take_the_mapping_positions_over_the_location_hints），每步摘要加这一版 units 的槽位摘要。
//! 原说明：改前、改后两份副本各编这一份，按种子跑同一串随机的合法操作，
//! 每一步之后打一行（操作、结局、两块盘镜像的 sha256、留存条目数与重复 key 数、搬了几个留存的单元），两份输出逐行比。
//! 种子范围与步数从环境变量取：DIFF_SEED_FIRST、DIFF_SEEDS、DIFF_STEPS。
mod common;

use std::collections::BTreeSet;
use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_core::address::{DeviceIdentity, InodeNumber, TreeIdentifier};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{CompactionBatchSize, CompactionBatchUnits, CompactionSwitches};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable_with_compaction_switches, raise_rollback_floor_to_the_admission_ceiling,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::records::TREE_KIND_SNAPSHOT_INODE_ROOT;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolVersion, PoolWriter,
    RelocatedUnitHolder, TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::SparseBlockDevice;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn sparse_device() -> SparseBlockDevice {
    SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512))
}

fn switches_with_the_batch_size(batch_units: usize) -> CompactionSwitches {
    let mut switches = CompactionSwitches::PRODUCT;
    switches.batch_size = CompactionBatchSize::ForcedForTests(
        CompactionBatchUnits::new(batch_units).expect("批大小至少 1"),
    );
    switches
}

fn formatted_and_mounted(switches: CompactionSwitches) -> (Devices, MountedSession) {
    let parameters = common::parameters();
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| (identity, sparse_device()))
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_file = common::file_content();
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile { content: &first_file, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    let mounted = mount_writable_with_compaction_switches(&parameters, &mut devices, switches)
        .expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (devices, session)
}

fn digest_of_the_images(devices: &Devices) -> String {
    let mut written = Vec::new();
    for (identity, device) in devices {
        written.extend_from_slice(&identity.0.to_le_bytes());
        for (sector, bytes) in device.image.written_sectors() {
            written.extend_from_slice(&sector.to_le_bytes());
            written.extend_from_slice(bytes);
        }
    }
    singlefs_harness::sha256::sha256_digest(&written)
        .iter()
        .take(12)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

fn content(data_units: usize, seed: u64, short_by: usize) -> Vec<u8> {
    let mut random = Random(seed | 1);
    (0..(data_units * data_unit_payload_capacity()).saturating_sub(short_by).max(1))
        .map(|_| u8::try_from(random.next() & 0xff).expect("低 8 位"))
        .collect()
}

fn version_of(session: &MountedSession) -> Option<&TransactionOutput> {
    session.current.version_of_trees()
}

/// 这一版的摘要：留存条目数、留存条目里重复的 key 数、第一个文件之外的文件数、其中重复 inode 号的个数、活快照数。
fn summary_of(session: &MountedSession) -> String {
    let Some(version) = version_of(session) else {
        return "no-trees".to_string();
    };
    let kept = &version.mapping_entries_kept_for_the_snapshots;
    let distinct_kept: BTreeSet<&[u8]> = kept.iter().map(|(key, _)| key.as_slice()).collect();
    let files = &version.files_other_than_the_first;
    let distinct_files: BTreeSet<u64> = files.iter().map(|file| file.inode.0).collect();
    format!(
        "kept={} dupkept={} files={} dupfiles={} units={} snapshots={} slots={}",
        kept.len(),
        kept.len() - distinct_kept.len(),
        files.len(),
        files.len() - distinct_files.len(),
        version.units.len(),
        live_snapshots(version).len(),
        slots_digest_of(version)
    )
}

fn slots_digest_of(version: &TransactionOutput) -> String {
    let mut text = String::new();
    for unit in &version.units {
        text.push_str(&format!("{:?}@{:?}/{};", unit.identity, unit.slot, unit.bytes.len()));
    }
    for (identity, key) in &version.mapped_units {
        text.push_str(&format!("{identity:?}={key:?};"));
    }
    singlefs_harness::sha256::sha256_digest(text.as_bytes()).iter().take(6).map(|byte| format!("{byte:02x}")).collect()
}

fn live_snapshots(version: &TransactionOutput) -> Vec<TreeIdentifier> {
    version
        .tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .map(|entry| entry.tree)
        .filter(|tree| !version.snapshots_being_destroyed.contains(tree))
        .collect()
}

fn environment_number(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|text| text.parse().ok()).unwrap_or(default)
}

fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).to_string()))
        .unwrap_or_else(|| "<非文本>".to_string())
}

/// 结局只取变体名那一段（Debug 的头一个词），连同整段 Debug 文本的长度：改前改后同一个错，这两样相同。
fn outcome_of<T, E: std::fmt::Debug>(result: &Result<T, E>) -> String {
    match result {
        Ok(_) => "ok".to_string(),
        Err(error) => {
            let text = format!("{error:?}");
            let head: String = text.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            format!("err:{head}:{}", text.len())
        }
    }
}

fn run_one_history(seed: u64, steps: u64) -> Vec<String> {
    let mut lines = Vec::new();
    let mut random = Random(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1);
    let batch = [1usize, 2, 3, 8][usize::try_from(random.below(4)).expect("下标")];
    let parameters = common::parameters();
    let (mut devices, mut session) = formatted_and_mounted(switches_with_the_batch_size(batch));
    let mut write_time = common::FIXED_WRITE_TIME_SECONDS + 600;
    let mut inodes: Vec<InodeNumber> = Vec::new();
    let mut kept_relocated_total = 0usize;
    lines.push(format!("seed={seed} batch={batch} start {}", digest_of_the_images(&devices)));
    for step in 0..steps {
        write_time += 1;
        let choice = if step < 2 { 0 } else { random.below(100) };
        let live = version_of(&session).map(live_snapshots).unwrap_or_default();
        let (label, outcome) = match choice {
            0..=7 => {
                let count = 1 + random.below(6);
                let first = version_of(&session).map_or(0, TransactionOutput::inode_number_watermark);
                let result = catch_unwind(AssertUnwindSafe(|| {
                    session.publish_user_change(&mut devices, UserChange::NewInodes { count, write_time_seconds: write_time })
                }));
                if matches!(result, Ok(Ok(_))) {
                    inodes.extend((first..first + count).map(InodeNumber));
                }
                (format!("new_inodes({count})"), result.map(|r| outcome_of(&r)))
            }
            8..=39 if !inodes.is_empty() => {
                let inode = inodes[usize::try_from(random.below(inodes.len() as u64)).expect("下标")];
                let units = 1 + usize::try_from(random.below(9)).expect("单元数");
                let bytes = content(units, random.next(), usize::try_from(random.below(7)).expect("短"));
                let result = catch_unwind(AssertUnwindSafe(|| {
                    session.publish_user_change(&mut devices, UserChange::SequentialWriteOfAnInode { inode, content: &bytes, write_time_seconds: write_time })
                }));
                (format!("write({},{units})", inode.0), result.map(|r| outcome_of(&r)))
            }
            40..=45 if !inodes.is_empty() => {
                let inode = inodes[usize::try_from(random.below(inodes.len() as u64)).expect("下标")];
                let bytes = content(1, random.next(), 1 + usize::try_from(random.below(300)).expect("短"));
                let result = catch_unwind(AssertUnwindSafe(|| {
                    session.publish_user_change(&mut devices, UserChange::OverwriteOfAnInode { inode, content: &bytes, write_time_seconds: write_time })
                }));
                (format!("overwrite({})", inode.0), result.map(|r| outcome_of(&r)))
            }
            46..=55 if live.len() < 3 => {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    session.publish_user_change(&mut devices, UserChange::CreateSnapshot)
                }));
                ("create_snapshot".to_string(), result.map(|r| outcome_of(&r)))
            }
            56..=62 if !live.is_empty() => {
                let snapshot = live[usize::try_from(random.below(live.len() as u64)).expect("下标")];
                let result = catch_unwind(AssertUnwindSafe(|| {
                    session.publish_user_change(&mut devices, UserChange::DestroySnapshot { snapshot })
                }));
                (format!("destroy_snapshot({})", snapshot.0), result.map(|r| outcome_of(&r)))
            }
            63..=65 if inodes.len() > 2 => {
                let position = usize::try_from(random.below(inodes.len() as u64)).expect("下标");
                let inode = inodes.remove(position);
                let result = catch_unwind(AssertUnwindSafe(|| {
                    session.publish_user_change(&mut devices, UserChange::Unlink { inode, write_time_seconds: write_time })
                }));
                (format!("unlink({})", inode.0), result.map(|r| outcome_of(&r)))
            }
            66..=80 => {
                let result = catch_unwind(AssertUnwindSafe(|| session.run_compaction(&mut devices)));
                let text = match &result {
                    Ok(Ok(run)) => {
                        let kept: usize = run
                            .versions_published
                            .iter()
                            .flat_map(|version| version.units_relocated_by_this_publish.iter())
                            .filter(|unit| unit.holder == RelocatedUnitHolder::KeptForTheSnapshots)
                            .count();
                        kept_relocated_total += kept;
                        format!("ok:batches={}:kept_relocated={kept}:{:?}", run.batches_published, std::mem::discriminant(&run.outcome))
                    }
                    Ok(error) => outcome_of(error),
                    Err(_) => String::new(),
                };
                ("compaction".to_string(), result.map(|_| text))
            }
            91..=94 => {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    let mounted = mount_writable_with_compaction_switches(&parameters, &mut devices, switches_with_the_batch_size(batch));
                    match mounted {
                        Ok(mounted) => {
                            let (_, remounted) = MountedSession::of_the_mount(mounted);
                            session = remounted;
                            "ok".to_string()
                        }
                        Err(error) => format!("err:{}", format!("{error:?}").chars().take(60).collect::<String>()),
                    }
                }));
                ("remount".to_string(), result)
            }
            81..=90 => {
                // 直接发一批整理（不经选段）：从这一版进映射的数据单元与留存的数据单元映射条目里随机取至多 batch 项，源落点取映射位置。
                use singlefs_core::compaction_plan::PlannedRelocation;
                use singlefs_core::transaction::{
                    central_mapping_locations_in_the_version, publish_relocation_batch, MappingLookup, TransactionUnit,
                };
                use singlefs_core::unit::UNIT_CLASS_DATA;
                let previous = version_of(&session).expect("有树").clone();
                let mut candidates: Vec<PlannedRelocation> = previous
                    .mapped_units
                    .iter()
                    .filter(|(role, _)| matches!(role, TransactionUnit::DataOfAFileOtherThanTheFirst { .. } | TransactionUnit::Data(_)))
                    .filter_map(|(role, key)| match central_mapping_locations_in_the_version(&previous, key) {
                        MappingLookup::Found(locations) => Some(PlannedRelocation {
                            mapping_key: key.clone(),
                            source: Placement { slot: locations[0].slot, span: role.span_slots() },
                        }),
                        _ => None,
                    })
                    .collect();
                let mut kept_candidates: Vec<PlannedRelocation> = Vec::new();
                kept_candidates.extend(
                    previous
                        .mapping_entries_kept_for_the_snapshots
                        .iter()
                        .filter(|(key, _)| key.first() == Some(&UNIT_CLASS_DATA))
                        .map(|(key, locations)| PlannedRelocation {
                            mapping_key: key.clone(),
                            source: Placement { slot: locations[0].slot, span: 2 },
                        }),
                );
                let mut planned = Vec::new();
                // 有留存条目时每一项一半机会取留存的那一类（留存条目的搬迁走七·6 后续批新加的按 key 索引）。
                while planned.len() < batch && !(candidates.is_empty() && kept_candidates.is_empty()) {
                    let from_kept = !kept_candidates.is_empty() && (candidates.is_empty() || random.below(2) == 0);
                    let pool = if from_kept { &mut kept_candidates } else { &mut candidates };
                    let position = usize::try_from(random.below(pool.len() as u64)).expect("下标");
                    planned.push(pool.swap_remove(position));
                }
                let kept_planned = planned
                    .iter()
                    .filter(|item| previous.mapping_entries_kept_for_the_snapshots.iter().any(|(key, _)| *key == item.mapping_key))
                    .count();
                if planned.is_empty() {
                    ("relocate(0)".to_string(), Ok("skip".to_string()))
                } else {
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        let mut writer = PoolWriter::new(&parameters, &mut devices);
                        publish_relocation_batch(&mut writer, &mut session.allocator, &previous, &planned, session.instance)
                    }));
                    let text = match &result {
                        Ok(Ok(version)) => {
                            let kept = version
                                .units_relocated_by_this_publish
                                .iter()
                                .filter(|unit| unit.holder == RelocatedUnitHolder::KeptForTheSnapshots)
                                .count();
                            kept_relocated_total += kept;
                            format!("ok:kept_relocated={kept}")
                        }
                        Ok(error) => outcome_of(error),
                        Err(_) => String::new(),
                    };
                    if let Ok(Ok(version)) = result.as_ref().map(|r| r.as_ref()) {
                        session.current = PoolVersion::of_a_version_of_trees(version.clone());
                    }
                    (format!("relocate({},{kept_planned})", planned.len()), result.map(|_| text))
                }
            }
            _ => {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    let mut current = version_of(&session).expect("有树").clone();
                    let raised = raise_rollback_floor_to_the_admission_ceiling(
                        &parameters, &mut devices, &mut session.allocator, &mut current, session.shadow_ledger,
                    );
                    if raised.is_ok() {
                        session.current = PoolVersion::of_a_version_of_trees(current);
                    }
                    outcome_of(&raised)
                }));
                ("raise_floor".to_string(), result)
            }
        };
        match outcome {
            Ok(text) => lines.push(format!(
                "seed={seed} step={step} {label} {text} {} {}",
                digest_of_the_images(&devices),
                summary_of(&session)
            )),
            Err(payload) => {
                lines.push(format!("seed={seed} step={step} {label} panic:{}", panic_text(&*payload)));
                return lines;
            }
        }
    }
    lines.push(format!("seed={seed} end kept_relocated_total={kept_relocated_total}"));
    lines
}

#[test]
fn diff_history_remount_history() {
    let first = environment_number("DIFF_SEED_FIRST", 1);
    let seeds = environment_number("DIFF_SEEDS", 4);
    let steps = environment_number("DIFF_STEPS", 40);
    // 各段历史互不依赖：种子按轮转分给 DIFF_THREADS 条线程（默认 SINGLEFS_THREAD_CAP，再没有取 1），每段各自建池；
    // 输出按种子的次序合并再打，与线程数无关。
    let threads = environment_number("DIFF_THREADS", environment_number("SINGLEFS_THREAD_CAP", 1)).max(1);
    let all_seeds: Vec<u64> = (first..first + seeds).collect();
    let mut outputs: Vec<(u64, Vec<String>)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|worker| {
                let mine: Vec<u64> = all_seeds
                    .iter()
                    .copied()
                    .skip(usize::try_from(worker).expect("线程号"))
                    .step_by(usize::try_from(threads).expect("线程数"))
                    .collect();
                scope.spawn(move || mine.into_iter().map(|seed| (seed, run_one_history(seed, steps))).collect::<Vec<_>>())
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| worker.join().expect("一条线程没 panic 出来"))
            .collect()
    });
    outputs.sort_by_key(|(seed, _)| *seed);
    for (_, lines) in outputs {
        for line in lines {
            println!("{line}");
        }
    }
}


// ===== 第二轮正推腿的计数探针（只在探针副本里编：核心层打了 probe_patch.py，数线性查找的比较次数） =====
use singlefs_core::mounted_session::UserChange as ProbeUserChange;
use singlefs_core::transaction::lookup_work::lookup_work_while;
use singlefs_core::transaction::probe_take;

fn build_probe_pool(first_file_units: Option<usize>, sizes: &[usize]) -> (Devices, MountedSession, Vec<InodeNumber>) {
    let (mut devices, mut session) = formatted_and_mounted(CompactionSwitches::PRODUCT);
    let mut write_time = common::FIXED_WRITE_TIME_SECONDS + 600;
    if let Some(units) = first_file_units {
        let bytes = content(units, 99, 0);
        session
            .publish_user_change(
                &mut devices,
                ProbeUserChange::SequentialWrite(singlefs_core::transaction::FirstFile {
                    content: &bytes,
                    write_time_seconds: write_time,
                }),
            )
            .unwrap_or_else(|refusal| panic!("写第一个文件 {units} 单元：{refusal:?}"));
    }
    let first = version_of(&session).map_or(0, TransactionOutput::inode_number_watermark);
    write_time += 1;
    session
        .publish_user_change(&mut devices, ProbeUserChange::NewInodes { count: sizes.len() as u64, write_time_seconds: write_time })
        .unwrap_or_else(|refusal| panic!("建 inode：{refusal:?}"));
    let inodes: Vec<InodeNumber> = (first..first + sizes.len() as u64).map(InodeNumber).collect();
    for (inode, units) in inodes.iter().zip(sizes) {
        write_time += 1;
        let bytes = content(*units, inode.0, 0);
        session
            .publish_user_change(
                &mut devices,
                ProbeUserChange::SequentialWriteOfAnInode { inode: *inode, content: &bytes, write_time_seconds: write_time },
            )
            .unwrap_or_else(|refusal| panic!("写 inode {}：{refusal:?}", inode.0));
    }
    (devices, session, inodes)
}

fn rewrite_and_count(devices: &mut Devices, session: &mut MountedSession, inode: InodeNumber, units: usize, tag: &str) {
    let bytes = content(units, 4242 + inode.0, 0);
    let _ = probe_take();
    let (result, work) = lookup_work_while(|| {
        session.publish_user_change(
            devices,
            ProbeUserChange::SequentialWriteOfAnInode { inode, content: &bytes, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 90_000 },
        )
    });
    result.unwrap_or_else(|refusal| panic!("整份重写：{refusal:?}"));
    let counts = probe_take();
    let version = version_of(session).expect("有树");
    println!(
        "name=probe {tag} first_file_units={} units_in_version={} files_other_than_first={} alloc_nodes={} rewritten_roles={} \
         cmp[first_file_contains={} alloc_find={} has_placement_any={} carried_lower_find={} lower_children_find={} reloc_contains={} unit_find={} file_find={} upper_leaf_filter={}] none_site={:?} all_sites_index_builds={} all_sites_lookups={}",
        version.data_pointers.len(),
        version.units.len(),
        version.files_other_than_the_first.len(),
        version.allocation_record_tree.nodes.len(),
        0,
        counts[0], counts[1], counts[2], counts[3], counts[4], counts[5], counts[6], counts[7], counts[8],
        work.work_of_each_site.get(&None),
        work.of_every_site().index_builds,
        work.of_every_site().index_lookups,
    );
}

#[test]
fn probe_scan_counts() {
    // 场景 A：一次写的单元数 n（池里另有两个 8 单元的文件）
    for n in [8usize, 32, 128] {
        let (mut devices, mut session, inodes) = build_probe_pool(None, &[n, 8, 8]);
        rewrite_and_count(&mut devices, &mut session, inodes[0], n, &format!("scenario=write_size n={n}"));
    }
    // 场景 B：第一个文件的数据单元数 d（写的是别的文件的 8 个单元）
    for d in [8usize, 64, 256] {
        let (mut devices, mut session, inodes) = build_probe_pool(Some(d), &[8, 8]);
        rewrite_and_count(&mut devices, &mut session, inodes[0], 8, &format!("scenario=first_file d={d}"));
    }
    // 场景 C：池里文件数 p（各 4 单元；写的是其中一个的 8 单元）
    for p in [16usize, 64, 256] {
        let sizes = vec![4usize; p];
        let (mut devices, mut session, inodes) = build_probe_pool(None, &sizes);
        rewrite_and_count(&mut devices, &mut session, inodes[0], 8, &format!("scenario=pool_files p={p}"));
    }
    // 场景 D：重新挂载（恢复路径）：池里文件数 p，数 take_the_mapping_positions 与 inode 记录查找
    let parameters = common::parameters();
    for p in [64usize, 256, 1024] {
        let sizes = vec![4usize; p];
        let (mut devices, session, _) = build_probe_pool(None, &sizes);
        let units = version_of(&session).expect("有树").units.len();
        drop(session);
        let _ = probe_take();
        let (mounted, work) = lookup_work_while(|| {
            mount_writable_with_compaction_switches(&parameters, &mut devices, CompactionSwitches::PRODUCT).expect("挂载")
        });
        let counts = probe_take();
        drop(mounted);
        println!("name=probe scenario=remount files={p} units={units} cmp[unit_find={} file_find={} inode_record_find={}] positions_site={:?} all_sites={:?}", counts[6], counts[7], counts[9], work.of_the_site(singlefs_core::transaction::lookup_work::LookupSite::UnitsOfTheMappedRolesTakingTheirMappingPositions), work.of_every_site());
    }
}
