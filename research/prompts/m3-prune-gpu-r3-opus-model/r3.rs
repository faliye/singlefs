//! m3-prune-gpu-r3 云端攻方腿的原型（只在草稿副本里，不入库）：挂在第二轮攻方原型 r2.rs 下面的子模块，复用它与第一轮 attack.rs 的历史与判定。
//! 每个子命令打一组 `name=r3_*` 行；`must_be_nonzero=` 是这个世界要报非 0 的数（`-` 是对照或回归）。
//! 历史一律在内存稀疏盘上录，起点池建一次；每个世界跑前现算状态数（`r3_state_budget` 行），一次不超过约 10⁶。
//! `r3-dump` 只打原料（每一步的三种内容哈希、每个状态的身份与判定），「身份 + 属性」下漏没漏由 `compare_dumps.py` 拿两份编译的原料比。

use super::*;

use singlefs_core::transaction::publish_sequential_write;

pub(super) fn run(output: &mut ResultLines, arguments: &[String]) -> i32 {
    match arguments.first().map(String::as_str) {
        Some("r3-dump") => world_dump_identities_and_attributes(output),
        Some("r3-panic-location") => world_panic_location(output),
        Some("r3-f1-torn") => world_f1_fix_under_a_torn_covering_write(output),
        Some("r3-multi-record") => world_multi_record_publish(output, arguments.get(1)),
        Some("r3-messages-by-batch") => world_violation_text_from_a_class_representative(output),
        _ => {
            eprintln!("用法：attack r3-dump|r3-panic-location|r3-f1-torn|r3-multi-record [单元数]|r3-messages-by-batch");
            2
        }
    }
}

// ───────────────────────────── 按操作分步录流：每一步（用户级操作）在写表里从哪一格起 ─────────────────────────────

/// 第一条流之后接的操作。
enum LaterOperation {
    Overwrite(Vec<u8>),
    SequentialWrite(Vec<u8>),
}

/// 一段历史与它每一步的起点：`step_starts[k]` 是第 k 步的第一次写在写表里的下标（步 0 = 取号，步 1 = 两次暖机，步 2 = 新池新建文件，之后逐个接的操作）。
struct SteppedHistory {
    history: History,
    step_names: Vec<String>,
    step_starts: Vec<usize>,
    mkfs_writes: Vec<RetainedWrite>,
}

fn record_stepped(content: &[u8], later: &[LaterOperation]) -> SteppedHistory {
    let parameters = common::parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, MemoryRecorded)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(number),
                    SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut boundaries = vec![stream.operations().len()];
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(Placement { slot: INSTANCE_TABLE_SLOT, span: 2 }, Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 });
    let mut step_names = vec!["acquire".to_string(), "warm_up_twice".to_string(), "first_file".to_string()];
    let mut versions_contents: Vec<Vec<u8>> = vec![content.to_vec()];
    let mut output = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        boundaries.push(stream.operations().len());
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        boundaries.push(stream.operations().len());
        let first = publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile { content, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS },
            instance,
            &warm.last_record_bytes,
        )
        .expect("新池新建文件");
        boundaries.push(stream.operations().len());
        first
    };
    for operation in later {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let file = |bytes: &'_ [u8]| -> Vec<u8> { bytes.to_vec() };
        output = match operation {
            LaterOperation::Overwrite(bytes) => {
                step_names.push("overwrite".to_string());
                versions_contents.push(file(bytes));
                publish_overwrite(&mut writer, &mut allocator, &output, FirstFile { content: bytes, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 60 }, InstanceGeneration(1)).expect("覆盖写")
            }
            LaterOperation::SequentialWrite(bytes) => {
                step_names.push("sequential_write".to_string());
                versions_contents.push(file(bytes));
                publish_sequential_write(&mut writer, &mut allocator, &output, FirstFile { content: bytes, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 60 }, InstanceGeneration(1)).expect("顺序写")
            }
        };
        boundaries.push(stream.operations().len());
    }
    let operations = stream.retained_operations();
    let mkfs_end = boundaries[0];
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], common::IMAGE_BYTES);
    base.apply(&operations[..mkfs_end]);
    let (mkfs_writes, _) = singlefs_harness::memory_pool::writes_and_segments(&operations[..mkfs_end], &common::geometry());
    let (writes, segments, stream_indexes) = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes(&operations[mkfs_end..], &common::geometry());
    let step_starts: Vec<usize> = boundaries[..boundaries.len() - 1]
        .iter()
        .map(|boundary| stream_indexes.iter().position(|index| index + mkfs_end >= *boundary).unwrap_or(writes.len()))
        .collect();
    // 版本表：每条根槽写的（实例，txg）按发布次序对上这一步写进去的内容；暖机与取号没有文件内容，只取文件那几步。
    let root_identities: Vec<(InstanceGeneration, CheckpointTxg)> = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .map(root_identity_of_root_write)
        .collect();
    let file_roots = &root_identities[root_identities.len() - versions_contents.len()..];
    let versions = file_roots
        .iter()
        .zip(versions_contents)
        .map(|((instance, checkpoint_txg), content)| PublishedVersion { instance: *instance, checkpoint_txg: *checkpoint_txg, content })
        .collect();
    SteppedHistory { history: History { base, writes, segments, versions }, step_names, step_starts, mkfs_writes }
}

fn step_of_write(stepped: &SteppedHistory, write_index: usize) -> usize {
    stepped.step_starts.iter().rposition(|start| *start <= write_index).expect("第一步从 0 起")
}

// ───────────────────────────── 一步的三种「写出的内容哈希」 ─────────────────────────────

fn contents_bytes(write: &RetainedWrite) -> Vec<u8> {
    match &write.contents {
        WrittenContents::Bytes(bytes) => bytes.clone(),
        WrittenContents::Zeros { length } => vec![0; usize::try_from(*length).expect("长")],
    }
}

/// 候选甲「写出的内容」：这一步的写按次序叠完、每个被写过的扇区最后留下的字节（次序、屏障、FUA、种类都不进）。
fn step_hash_final_sectors(writes: &[&RetainedWrite]) -> String {
    let mut sectors: BTreeMap<(u32, u64), Vec<u8>> = BTreeMap::new();
    for write in writes {
        let bytes = contents_bytes(write);
        for (chunk_index, chunk) in bytes.chunks(512).enumerate() {
            let sector = write.offset.0 / 512 + u64::try_from(chunk_index).expect("扇区号");
            sectors.insert((write.device.0, sector), chunk.to_vec());
        }
    }
    let mut message = b"r3 final sectors".to_vec();
    for ((device, sector), bytes) in &sectors {
        message.extend_from_slice(&device.to_le_bytes());
        message.extend_from_slice(&sector.to_le_bytes());
        message.extend_from_slice(bytes);
    }
    sha256_hex(&message)[..16].to_string()
}

/// 候选乙「写出的内容」：这一步的写按录制次序逐次（盘、偏移、长度、字节），不带种类、FUA 与屏障。
fn step_hash_write_list(writes: &[&RetainedWrite]) -> String {
    let mut message = b"r3 write list".to_vec();
    for write in writes {
        message.extend_from_slice(&write.device.0.to_le_bytes());
        message.extend_from_slice(&write.offset.0.to_le_bytes());
        let bytes = contents_bytes(write);
        message.extend_from_slice(&u64::try_from(bytes.len()).expect("长").to_le_bytes());
        message.extend_from_slice(&bytes);
    }
    sha256_hex(&message)[..16].to_string()
}

/// 收严的写法：整张写表（盘、种类、FUA、偏移、字节）连同段边界（每次写在这一步里落在第几段，也就是屏障切在哪）。
fn step_hash_write_table_with_barriers(writes: &[(usize, &RetainedWrite)]) -> String {
    let mut message = b"r3 write table with barriers".to_vec();
    let first_segment = writes.first().map_or(0, |(segment, _)| *segment);
    for (segment, write) in writes {
        message.extend_from_slice(&u64::try_from(segment - first_segment).expect("段").to_le_bytes());
        message.extend_from_slice(&write.device.0.to_le_bytes());
        message.extend_from_slice(write.kind.name().as_bytes());
        message.push(u8::from(write.is_force_unit_access));
        message.extend_from_slice(&write.offset.0.to_le_bytes());
        message.extend_from_slice(&contents_bytes(write));
    }
    sha256_hex(&message)[..16].to_string()
}

fn segment_of_write(segments: &[Vec<usize>], write_index: usize) -> usize {
    segments.iter().position(|segment| segment.contains(&write_index)).expect("每次写都在某一段里")
}

// ───────────────────────────── r3-dump：每一步的属性、每个状态的（流程步号身份，判定） ─────────────────────────────

/// 身份按 A1 被验的形态：`<代码文件名>::<验证目标名>::<流程步号>::<哈希串>`。这个世界里文件名、目标名、路径号、哈希串（流程定义）
/// 在两份编译上逐字相同（同一份原型、同一串操作与参数），只打流程步号那几段：步号、步内第几段、段内各次写的落法（0 没落、1 撕裂、2 落了；
/// 段内序号取这串落法本身，不取枚举次序，所以换展开方式不改它）。全部持久那一个记 `end`。
fn world_dump_identities_and_attributes(output: &mut ResultLines) -> i32 {
    let content = common::file_content();
    let overwrite_content: Vec<u8> = (0..4100).map(|index| u8::try_from((index * 7 + 3) % 253).expect("字节")).collect();
    let stepped = record_stepped(&content, &[LaterOperation::Overwrite(overwrite_content)]);
    let history = &stepped.history;
    let mkfs_refs: Vec<&RetainedWrite> = stepped.mkfs_writes.iter().collect();
    output.line(format!(
        "name=r3_dump_base mkfs_writes={} final_sectors={} write_list={} segments={:?}",
        stepped.mkfs_writes.len(),
        step_hash_final_sectors(&mkfs_refs),
        step_hash_write_list(&mkfs_refs),
        history.segments.iter().map(Vec::len).collect::<Vec<_>>()
    ));
    for (step, name) in stepped.step_names.iter().enumerate() {
        let indexes: Vec<usize> = (0..history.writes.len()).filter(|index| step_of_write(&stepped, *index) == step).collect();
        let refs: Vec<&RetainedWrite> = indexes.iter().map(|index| &history.writes[*index]).collect();
        let with_segments: Vec<(usize, &RetainedWrite)> = indexes.iter().map(|index| (segment_of_write(&history.segments, *index), &history.writes[*index])).collect();
        let segment_lengths: Vec<usize> = history
            .segments
            .iter()
            .filter(|segment| step_of_write(&stepped, segment[0]) == step)
            .map(Vec::len)
            .collect();
        output.line(format!(
            "name=r3_dump_step step={step} operation={name} writes={} segment_lengths={segment_lengths:?} final_sectors={} write_list={} write_table_with_barriers={}",
            refs.len(),
            step_hash_final_sectors(&refs),
            step_hash_write_list(&refs),
            step_hash_write_table_with_barriers(&with_segments)
        ));
    }
    budget(output, "r3_dump_first_plus_overwrite", history, &small);
    let table = TornWriteTable::of(&history.prepared());
    let mut states = 0u64;
    enumerate_observing(history, &small, &mut |image, _report| {
        let (segment, landings) = segment_and_landings(history, &table, &image.persisted);
        let identity = if segment < history.segments.len() {
            let step = step_of_write(&stepped, history.segments[segment][0]);
            let first_segment_of_step = history.segments.iter().position(|candidate| step_of_write(&stepped, candidate[0]) == step).expect("这一步至少一段");
            let code: String = landings.iter().map(|landing| char::from(b'0' + landing)).collect();
            format!("{step}.{}.{code}", segment - first_segment_of_step)
        } else {
            "end".to_string()
        };
        let verdict = verdict_of_image(image, &history.versions);
        states += 1;
        output.line(format!("name=r3_dump_state identity={identity} status={} red={:?} full={:?}", serialized_status(&verdict.status), verdict.red, verdict.full));
    });
    output.line(format!("name=r3_dump_done states={states}"));
    0
}

// ───────────────────────────── A9：panic 的位置进输出（只挪行、词法单元不变，输出就变） ─────────────────────────────

/// 用坏盘输入与崩溃注入同一个接 panic 的入口（`singlefs_harness::history::with_panic_capture`，它记 `文件:行`）接一处 core 里的 expect：
/// 盘比单元区起点还小时建空闲图（`DeviceFreeMap::new`）。输出只打它记下的位置与消息开头；两份编译的词法单元摘要相同而这一行不同，就是反向检查的假红。
fn world_panic_location(output: &mut ResultLines) -> i32 {
    let captured = singlefs_harness::history::with_panic_capture(|| DeviceFreeMap::new(DeviceIdentity(0), 1024)).expect_err("盘比单元区起点小要 panic");
    let message_head: String = captured.message.chars().take(12).collect();
    output.line(format!("name=r3_panic_location location={} message_head={message_head}", captured.location));
    0
}

// ───────────────────────────── A3：F1 的改法（候选槽按「这一槽上最后一次持久单元写从这里开头」认）碰上撕裂的覆盖写 ─────────────────────────────

/// 收严的候选：只有盖过这一槽的那次后写在这一槽上的字节确实在盘上（撕裂镜像落在这一槽上的是旧字节就不算盖过），才把这一槽拿掉。
struct CurrentUnitStartInPlaceReader<'reader, 'base> {
    image: &'reader CrashImage<'base>,
}

impl PoolReader for CurrentUnitStartInPlaceReader<'_, '_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        PoolReader::device_identities(self.image)
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        PoolReader::device_size_in_bytes(self.image, device)
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: usize) -> Option<Vec<u8>> {
        PoolReader::read(self.image, device, offset, length)
    }
    fn journal_record_offsets_hint(&self, device: DeviceIdentity, ring_start: DeviceOffsetInBytes, ring_bytes: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        PoolReader::journal_record_offsets_hint(self.image, device, ring_start, ring_bytes)
    }
}

impl ImageReader for CurrentUnitStartInPlaceReader<'_, '_> {
    fn devices(&self) -> Vec<u32> {
        ImageReader::devices(self.image)
    }
    fn device_bytes(&self, device: u32) -> Option<u64> {
        ImageReader::device_bytes(self.image, device)
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        ImageReader::read(self.image, device, offset, length)
    }
    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>> {
        let slots = ImageReader::candidate_unit_slots(self.image, device)?;
        Some(
            slots
                .into_iter()
                .filter(|slot| {
                    let slot_start = slot * LOCAL_SLOT_BYTES;
                    let last_covering = self
                        .image
                        .writes
                        .iter()
                        .zip(&self.image.persisted)
                        .filter(|(write, persisted)| {
                            **persisted
                                && write.kind == StepKind::UnitWrite
                                && write.device.0 == device
                                && write.offset.0 < slot_start + LOCAL_SLOT_BYTES
                                && slot_start < write.offset.0 + write.length_in_bytes()
                        })
                        .last();
                    let Some((write, _)) = last_covering else { return true };
                    if write.offset.0 == slot_start {
                        return true;
                    }
                    let from = usize::try_from(slot_start - write.offset.0).expect("槽内偏移");
                    let bytes = contents_bytes(write);
                    let covering_bytes_in_place = ImageReader::read(self.image, device, slot_start, 512).is_some_and(|on_disk| on_disk == bytes[from..from + 512]);
                    !covering_bytes_in_place
                })
                .collect(),
        )
    }
    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
        ImageReader::candidate_journal_slots(self.image, device)
    }
}

/// 改坏的实现在槽 60001 写一个树 ID 越过水位的孤儿节点（第一轮 p5 的那种），随后一个 32K 数据单元从槽 60000 写起盖过它。
/// 数据单元这次写罩着更早写过字节的范围，能撕（`crash.rs` 的可撕判据）；撕裂那一态前一半新、后一半旧，槽 60001 上留着完整的孤儿节点。
/// 数：孤儿节点确实完整在盘上的状态里，今天的叠加候选判红、F1 改法的候选判绿的个数（漏判）。
fn world_f1_fix_under_a_torn_covering_write(output: &mut ResultLines) -> i32 {
    let full = first_history();
    let node = full.segments[7]
        .iter()
        .map(|index| &full.writes[*index])
        .find(|write| write.device == DeviceIdentity(0) && write.length_in_bytes() == LOCAL_NODE_BYTES && write.bytes().is_some_and(|bytes| bytes[6] == 2))
        .expect("新建文件那段里有一个码 2 节点")
        .bytes()
        .expect("字节")
        .to_vec();
    let orphan = orphan_node_that_the_i78_scan_counts(&node, 1 << 40);
    for (label, data_unit_byte) in [("orphan_at_60001_then_data_unit_at_60000", 0x5Au8)] {
        let mut history = full.clone();
        let node_start = history.writes.len();
        history.writes.push(unit_write(0, 60_001, orphan.clone()));
        history.segments.push(vec![node_start]);
        let unit_start = history.writes.len();
        history.writes.push(RetainedWrite {
            device: DeviceIdentity(0),
            kind: StepKind::UnitWrite,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(60_000 * LOCAL_SLOT_BYTES),
            contents: WrittenContents::Bytes(vec![data_unit_byte; 32_768]),
        });
        history.segments.push(vec![unit_start]);
        budget(output, label, &history, &small);
        let table = TornWriteTable::of(&history.prepared());
        let first_new_segment = full.segments.len();
        let mut rows: Vec<String> = Vec::new();
        let (mut missed_by_f1, mut missed_by_in_place, mut orphan_present_states) = (0u64, 0u64, 0u64);
        enumerate_observing(&history, &small, &mut |image, _report| {
            if segment_of_state(&history.segments, &image.persisted) < first_new_segment {
                return;
            }
            let (segment, landings) = segment_and_landings(&history, &table, &image.persisted);
            let orphan_present = ImageReader::read(image, 0, 60_001 * LOCAL_SLOT_BYTES, orphan.len()).is_some_and(|bytes| bytes == orphan);
            let overlay = verdict_of_image(image, &history.versions);
            let f1 = verdict_of(&CurrentUnitStartReader { image }, image.writes, &image.persisted, &history.versions);
            let in_place = verdict_of(&CurrentUnitStartInPlaceReader { image }, image.writes, &image.persisted, &history.versions);
            let i78 = |verdict: &R2Verdict| verdict.red.iter().any(|name| name == "checker:I-7.8");
            orphan_present_states += u64::from(orphan_present);
            missed_by_f1 += u64::from(orphan_present && i78(&overlay) && !i78(&f1));
            missed_by_in_place += u64::from(orphan_present && i78(&overlay) && !i78(&in_place));
            rows.push(format!("seg{segment}:{landings:?}:orphan_on_disk={orphan_present}:overlay={}:f1={}:in_place={}", i78(&overlay), i78(&f1), i78(&in_place)));
        });
        output.line(format!(
            "name=r3_f1_torn world={label} states_from_the_new_segments={} orphan_intact_on_disk={orphan_present_states} i78_missed_by_the_f1_candidates={missed_by_f1} i78_missed_by_the_in_place_candidates={missed_by_in_place} rows={rows:?} must_be_nonzero={missed_by_f1}",
            rows.len()
        ));
    }
    f1_shifted_under_three_candidate_readers(output, &full, &node, &orphan);
    0
}

/// 第二轮 r2-f1-shifted 那一形（槽 60001 上是正常节点，数据单元后半是一段像节点头的用户数据），三种候选下各几个状态 I-7.8 红：今天的叠加候选假红，两种改法都该是 0。
fn f1_shifted_under_three_candidate_readers(output: &mut ResultLines, full: &History, node: &[u8], orphan: &[u8]) {
    let header_end = 86 + 2 * usize::from(orphan[51]);
    let mut data_unit = vec![0u8; 32_768];
    data_unit[16_384..16_384 + header_end].copy_from_slice(&orphan[..header_end]);
    let mut history = full.clone();
    let node_start = history.writes.len();
    history.writes.push(unit_write(0, 60_001, node.to_vec()));
    history.segments.push(vec![node_start]);
    let unit_start = history.writes.len();
    history.writes.push(RetainedWrite {
        device: DeviceIdentity(0),
        kind: StepKind::UnitWrite,
        is_force_unit_access: false,
        offset: DeviceOffsetInBytes(60_000 * LOCAL_SLOT_BYTES),
        contents: WrittenContents::Bytes(data_unit),
    });
    history.segments.push(vec![unit_start]);
    budget(output, "f1_shifted_one_write_per_segment", &history, &small);
    let first_new_segment = full.segments.len();
    let (mut states, mut red_overlay, mut red_f1, mut red_in_place) = (0u64, 0u64, 0u64, 0u64);
    enumerate_observing(&history, &small, &mut |image, _report| {
        if segment_of_state(&history.segments, &image.persisted) < first_new_segment {
            return;
        }
        let i78 = |verdict: &R2Verdict| verdict.red.iter().any(|name| name == "checker:I-7.8");
        states += 1;
        red_overlay += u64::from(i78(&verdict_of_image(image, &history.versions)));
        red_f1 += u64::from(i78(&verdict_of(&CurrentUnitStartReader { image }, image.writes, &image.persisted, &history.versions)));
        red_in_place += u64::from(i78(&verdict_of(&CurrentUnitStartInPlaceReader { image }, image.writes, &image.persisted, &history.versions)));
    });
    output.line(format!(
        "name=r3_f1_torn world=legit_node_at_60001_then_data_unit_with_a_node_like_second_half states_from_the_new_segments={states} i78_red_under_todays_overlay={red_overlay} i78_red_under_the_f1_candidates={red_f1} i78_red_under_the_in_place_candidates={red_in_place} must_be_nonzero=-"
    ));
}

// ───────────────────────────── A3 / A4：多记录发布（顺序写 k 个数据单元）的单元段与记录段 ─────────────────────────────

/// 记录核对器符号键的收严版（我加的，零轮）：记录写的在不在盘上只在「按记录自己的（实例，txg）认出的那条根在盘上」时进键；
/// 其余同第二轮符号版（看 journal 那一遍恢复落到的根、每条根槽写在不在盘上、过不了回收谓词的后写落没落）。
fn record_checker_key_records_only_under_their_landed_root(image: &CrashImage<'_>, consult_root: Option<(InstanceGeneration, CheckpointTxg)>) -> Digest128 {
    let in_place = |index: usize| {
        let write = &image.writes[index];
        let length = usize::try_from(write.length_in_bytes()).expect("长");
        PoolReader::read(image, write.device, write.offset, length).is_some_and(|bytes| write.contents.still_on_disk(&bytes))
    };
    let roots_in_place: BTreeSet<(u32, u64)> = (0..image.writes.len())
        .filter(|index| image.writes[*index].kind == StepKind::RootRecordFua && in_place(*index))
        .map(|index| {
            let (instance, txg) = root_identity_of_root_write(&image.writes[index]);
            (instance.0, txg.0)
        })
        .collect();
    let roots: Vec<(usize, bool)> = (0..image.writes.len()).filter(|index| image.writes[*index].kind == StepKind::RootRecordFua).map(|index| (index, in_place(index))).collect();
    let records: Vec<(usize, bool)> = (0..image.writes.len())
        .filter(|index| image.writes[*index].kind == StepKind::JournalRecord && record_identity(&image.writes[*index]).is_some_and(|identity| roots_in_place.contains(&identity)))
        .map(|index| (index, in_place(index)))
        .collect();
    let mut failing_bits: BTreeSet<(usize, usize, bool)> = BTreeSet::new();
    if let Some((_, landed_txg)) = consult_root {
        for publish in publishes_in(image.writes, &image.persisted, RecordStreamContinuity::OneRecording) {
            if publish.checkpoint_txg > landed_txg.0 {
                continue;
            }
            for unit in &publish.units {
                for later in unit + 1..image.writes.len() {
                    if writes_overlap(&image.writes[*unit], &image.writes[later]) && !attack_later_write_passes_the_reclaim_predicate(image.writes, *unit, later) {
                        failing_bits.insert((*unit, later, image.persisted[later]));
                    }
                }
            }
        }
    }
    fingerprint_of_debug_text(&(consult_root, roots, records, failing_bits))
}

fn world_multi_record_publish(output: &mut ResultLines, units_argument: Option<&String>) -> i32 {
    let units: usize = units_argument.map_or(6, |text| text.parse().expect("单元数"));
    let content = common::file_content();
    let sequential: Vec<u8> = (0..units * 30_000).map(|index| u8::try_from((index * 11 + 7) % 251).expect("字节")).collect();
    let stepped = record_stepped(&content, &[LaterOperation::SequentialWrite(sequential)]);
    let history = &stepped.history;
    let step = stepped.step_names.len() - 1;
    let step_segments: Vec<usize> = (0..history.segments.len()).filter(|segment| step_of_write(&stepped, history.segments[*segment][0]) == step).collect();
    let shape: Vec<String> = step_segments.iter().map(|segment| format!("{}x{}", history.segments[*segment].len(), kind_letter(history.writes[history.segments[*segment][0]].kind))).collect();
    output.line(format!("name=r3_multi_record_shape units={units} sequential_step_segments={shape:?}"));
    // A4：记录段（这一步里全是 journal 记录写的那一段），段内全展开。
    let record_segment = *step_segments
        .iter()
        .find(|segment| history.segments[**segment].iter().all(|index| history.writes[*index].kind == StepKind::JournalRecord))
        .expect("顺序写那一步有一段全是记录写");
    budget(output, "multi_record_record_segment", history, &|index, _| index == record_segment);
    let mut symbolic_pairs: Vec<(Digest128, (bool, bool))> = Vec::new();
    let mut tightened_pairs: Vec<(Digest128, (bool, bool))> = Vec::new();
    let mut tightened_status_pairs: Vec<(Digest128, StatusVector)> = Vec::new();
    let mut statuses: BTreeSet<StatusVector> = BTreeSet::new();
    let started = Instant::now();
    enumerate_observing(history, &|index, _| index == record_segment, &mut |image, _report| {
        if segment_of_state(&history.segments, &image.persisted) != record_segment {
            return;
        }
        let verdict = verdict_of_image(image, &history.versions);
        let pair = (verdict.status.root_without_record, verdict.status.claimed_state_missing_unit);
        symbolic_pairs.push((record_checker_symbolic_key(image, verdict.consult_root), pair));
        let tightened = record_checker_key_records_only_under_their_landed_root(image, verdict.consult_root);
        tightened_pairs.push((tightened, pair));
        tightened_status_pairs.push((tightened, verdict.status.clone()));
        statuses.insert(verdict.status);
    });
    let distinct_record_verdicts: BTreeSet<(bool, bool)> = symbolic_pairs.iter().map(|(_, verdict)| *verdict).collect();
    let (symbolic_classes, symbolic_inconsistent) = inconsistent_by(&symbolic_pairs);
    let (tightened_classes, tightened_inconsistent) = inconsistent_by(&tightened_pairs);
    let (_, tightened_status_inconsistent) = inconsistent_by(&tightened_status_pairs);
    output.line(format!(
        "name=r3_a4_record_segment units={units} record_writes={} states={} distinct_record_verdicts={} distinct_status_vectors={} symbolic_key_classes={symbolic_classes} symbolic_key_inconsistent={symbolic_inconsistent} tightened_key_classes={tightened_classes} tightened_key_inconsistent={tightened_inconsistent} tightened_key_inconsistent_on_the_whole_status_vector={tightened_status_inconsistent} seconds={:.1} must_be_nonzero={}",
        history.segments[record_segment].len(),
        symbolic_pairs.len(),
        distinct_record_verdicts.len(),
        statuses.len(),
        started.elapsed().as_secs_f64(),
        symbolic_classes.saturating_sub(distinct_record_verdicts.len())
    ));
    // A3：这一步的单元段，录制流前缀的前 12 次写（缩历史长度），段内 4095 个状态一个不落；P7 键与三种候选下的 I-7.8。
    let unit_segment = step_segments[0];
    let mut prefix = history.prefix_of_segments(unit_segment);
    let mut segment = Vec::new();
    for index in history.segments[unit_segment].iter().take(12) {
        segment.push(prefix.writes.len());
        prefix.writes.push(history.writes[*index].clone());
    }
    prefix.segments.push(segment);
    let prefix_unit_segment = prefix.segments.len() - 1;
    budget(output, "multi_record_unit_segment_k12", &prefix, &|index, _| index == prefix_unit_segment);
    let mut tally = KeyTally::default();
    let (mut red_f1, mut red_in_place) = (0u64, 0u64);
    let started = Instant::now();
    enumerate_observing(&prefix, &|index, _| index == prefix_unit_segment, &mut |image, _report| {
        if segment_of_state(&prefix.segments, &image.persisted) != prefix_unit_segment {
            return;
        }
        tally.add(image, &prefix.versions);
        let i78 = |verdict: &R2Verdict| verdict.red.iter().any(|name| name == "checker:I-7.8");
        red_f1 += u64::from(i78(&verdict_of(&CurrentUnitStartReader { image }, image.writes, &image.persisted, &prefix.versions)));
        red_in_place += u64::from(i78(&verdict_of(&CurrentUnitStartInPlaceReader { image }, image.writes, &image.persisted, &prefix.versions)));
    });
    output.line(format!(
        "{} i78_red_under_the_f1_candidates={red_f1} i78_red_under_the_in_place_candidates={red_in_place} seconds={:.1}",
        tally.line(&format!("a3_multi_record_unit_segment_k12_units{units}")).replacen("name=r2_b4_keys", "name=r3_a3_keys", 1),
        started.elapsed().as_secs_f64()
    ));
    0
}

// ───────────────────────────── A2 / A6：违例正文不进向量之后，从类的代表状态现算正文 ─────────────────────────────

/// 第二轮 r2-b4-messages 那段历史（单元段里两个写序实例代号过高的节点，接新建文件那段的前 4 次单元写）。
/// 每个状态取 P7 键、按项红绿与 I-7.7 的正文；正文按「类的代表状态现算」给出时，三种挑代表的办法下各有几个状态拿到的不是自己的正文、
/// 单机与两台（按块交错分、各自按批挑代表）给同一个状态的正文有几个不同。按项红绿从代表取，逐个核是不是与自己相同。
fn world_violation_text_from_a_class_representative(output: &mut ResultLines) -> i32 {
    let full = first_history();
    let prefix = full.prefix_of_segments(7);
    let node = full.segments[7]
        .iter()
        .map(|index| &full.writes[*index])
        .find(|write| write.device == DeviceIdentity(0) && write.length_in_bytes() == LOCAL_NODE_BYTES && write.bytes().is_some_and(|bytes| bytes[6] == 2))
        .expect("码 2 节点")
        .bytes()
        .expect("字节")
        .to_vec();
    let changed = node_with_write_order_instance(&node, 7);
    let mut history = prefix.clone();
    let start = history.writes.len();
    history.writes.push(unit_write(0, 60_000, changed.clone()));
    history.writes.push(unit_write(0, 60_002, changed));
    for index in full.segments[7].iter().take(4) {
        history.writes.push(full.writes[*index].clone());
    }
    history.segments.push((start..history.writes.len()).collect());
    let unit_segment = history.segments.len() - 1;
    budget(output, "messages_by_batch", &history, &|_, _| true);
    let mut states: Vec<(Digest128, StatusVector, String)> = Vec::new();
    enumerate_observing(&history, &|_, _| true, &mut |image, _report| {
        if segment_of_state(&history.segments, &image.persisted) != unit_segment {
            return;
        }
        let reads = state_reads(image);
        let summary = scan_summary(image);
        let recovery_key = pair_digest(sequence_key(&reads.consult_calls), sequence_key(&reads.ignore_calls));
        let p7_key = pair_digest(pair_digest(recovery_key, sequence_key(&reads.walk_calls)), fingerprint_of_debug_text(&summary));
        let text = check_pool_image(image).iter().find(|(name, _)| *name == "I-7.7").map_or("-".to_string(), |(_, verdict)| format!("{verdict:?}"));
        states.push((p7_key, verdict_of_image(image, &history.versions).status, text));
    });
    const BLOCK_STATES: usize = 16;
    const BATCH_BLOCKS: usize = 2;
    // 挑代表：machine_of(state) 与 batch_of(state) 定「在哪一批里挑」，同一批同一类取次序最前的那个。
    let representatives = |machine_of: &dyn Fn(usize) -> usize, batch_of: &dyn Fn(usize) -> usize| -> Vec<usize> {
        let mut first: BTreeMap<(usize, usize, Digest128), usize> = BTreeMap::new();
        (0..states.len())
            .map(|state| *first.entry((machine_of(state), batch_of(state), states[state].0)).or_insert(state))
            .collect()
    };
    let single_whole = representatives(&|_| 0, &|_| 0);
    let single_batched = representatives(&|_| 0, &|state| state / (BLOCK_STATES * BATCH_BLOCKS));
    let two_machines = representatives(&|state| (state / BLOCK_STATES) % 2, &|state| (state / BLOCK_STATES) / 2 / BATCH_BLOCKS);
    let wrong_text = |chosen: &[usize]| (0..states.len()).filter(|state| states[chosen[*state]].2 != states[*state].2).count();
    let wrong_status = |chosen: &[usize]| (0..states.len()).filter(|state| states[chosen[*state]].1 != states[*state].1).count();
    let differs = |left: &[usize], right: &[usize]| (0..states.len()).filter(|state| states[left[*state]].2 != states[right[*state]].2).count();
    output.line(format!(
        "name=r3_text_from_representative states={} p7_classes={} distinct_texts={} wrong_text_single_machine_one_batch={} wrong_text_single_machine_batches_of_{}={} wrong_text_two_machines_interleaved_blocks={} text_differs_single_versus_two_machines={} wrong_status_single={} wrong_status_two_machines={} must_be_nonzero={}",
        states.len(),
        states.iter().map(|(key, _, _)| *key).collect::<BTreeSet<_>>().len(),
        states.iter().map(|(_, _, text)| text.clone()).collect::<BTreeSet<_>>().len(),
        wrong_text(&single_whole),
        BLOCK_STATES * BATCH_BLOCKS,
        wrong_text(&single_batched),
        wrong_text(&two_machines),
        differs(&single_whole, &two_machines),
        wrong_status(&single_whole),
        wrong_status(&two_machines),
        wrong_text(&single_whole)
    ));
    0
}
