//! C322（取号那一步的屏障怎么放没有条款） 第三轮攻方腿（Opus）的模型 L：在仓里今天的实现与 checker（crates/，只读、不改）上，
//! 量候选包 P 第 2 句（世代号 G2）在第一次挂载那条流上写不写出同样的世代号，第 4 句（I-7.7 换成 ① ②）在 checker 上判什么。
//! 流：mkfs → 取号（实例 1）→ 暖机 → 第一个事务，与 crates/singlefs-harness/tests/common/mod.rs 同一串调用（内存盘）；
//! 再以实例 2 借 publish_first_file 发一次替身，只取它第一道屏障之前的 16 个单元写，拿来拼点名镜像。
//! 输出一 G2_AUDIT：那条流里每一次超级块槽写的世代号，与「这块盘上自证过的槽里最大的世代号 + 1」逐条比。
//! 输出二 LAYER0：层 0 全部崩溃状态（与 crates/singlefs-harness/src/crash.rs 的 enumerate_layer0 同一条枚举规则），
//!   每个状态判今天 checker 的 I-7.7、①（全部槽读法）、② 的三种读法（每盘最大、每盘择到的、每个槽），
//!   以及按全部槽读法与按择到的读法，下一次取号撞不撞号。
//! 输出三 IMAGE：点名镜像，逐个判同一组检查。

use std::collections::BTreeSet;

use singlefs_checker::image::{chosen_superblocks, valid_roots, ImageReader, InvariantVerdict};
use singlefs_checker::walk::check_pool_image;
use singlefs_checker::{check_journal_record, check_superblock_slot, crc32_castagnoli_bitwise};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, MKFS_INSTANCE_GENERATION,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::recovery::PoolReader;
use singlefs_core::superblock::Superblock;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter,
};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES};
use singlefs_harness::crash::{
    writes_and_segments, CrashImage, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

/// 与 tests/common/mod.rs 同：4 GiB 两块盘、物理块 512、槽距 4096、默认 768 MiB 环、E142 的 fsid。
const IMAGE_BYTES: u64 = 4 << 30;
const SUPERBLOCK_SLOT_SPACING: u64 = 4096;

struct Built {
    parameters: MakeFilesystemParameters,
    base_after_mkfs: MemoryPool,
    layer0_writes: Vec<RetainedWrite>,
    layer0_segments: Vec<Vec<usize>>,
    final_image: MemoryPool,
    stand_in_unit_writes: Vec<RetainedWrite>,
}

fn fixed_geometry() -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
    }
}

fn build() -> Built {
    let parameters = e142_parameters(512, 512);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(device_number),
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mkfs_operation_count = stream.operations().len();
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(&[
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    ]);
    let content = first_file_content();
    let first_transaction_end;
    {
        let mut pool = PoolWriter {
            parameters: &parameters,
            devices: &mut devices,
        };
        let instance = acquire_instance(&mut pool, MKFS_INSTANCE_GENERATION).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        let first_file = FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        };
        publish_first_file(&mut pool, &mut allocator, &genesis.root, first_file, instance, &warm.last_record_bytes)
            .expect("第一个事务");
        first_transaction_end = stream.operations().len();
        let stand_in = FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        };
        let second_instance = InstanceGeneration(instance.0 + 1);
        publish_first_file(&mut pool, &mut allocator, &genesis.root, stand_in, second_instance, &warm.last_record_bytes)
            .expect("实例 2 的替身发布");
    }
    let operations = stream.retained_operations();
    let mut base_after_mkfs =
        MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    base_after_mkfs.apply(&operations[..mkfs_operation_count]);
    let (layer0_writes, layer0_segments) = writes_and_segments(
        &operations[mkfs_operation_count..first_transaction_end],
        &fixed_geometry(),
    );
    let mut final_image = base_after_mkfs.clone();
    final_image.apply(&operations[mkfs_operation_count..first_transaction_end]);
    let stand_in_first_barrier = (first_transaction_end..operations.len())
        .find(|position| operations[*position].operation.kind == RecordedOperationKind::Barrier)
        .expect("替身的第一道屏障");
    let (stand_in_writes, _) = writes_and_segments(
        &operations[first_transaction_end..stand_in_first_barrier],
        &fixed_geometry(),
    );
    let stand_in_unit_writes: Vec<RetainedWrite> = stand_in_writes
        .into_iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .collect();
    Built {
        parameters,
        base_after_mkfs,
        layer0_writes,
        layer0_segments,
        final_image,
        stand_in_unit_writes,
    }
}

/// 一个自证过（校验和过、fsid 与本池相同）的超级块槽。
#[derive(Clone, Copy, Debug)]
struct SlotReading {
    device: u32,
    slot_index: u64,
    generation: u64,
    instance: u32,
}

/// 一个镜像上 ① ② 要的输入：自证过的超级块槽、根环里读得出的根、带实例代号的根 / journal 记录 / 单元写序（都按 fsid 过滤）。
struct Evaluation {
    slots: Vec<SlotReading>,
    root_instances: BTreeSet<u32>,
    carriers: BTreeSet<u32>,
    today_seven_seven_violated: bool,
}

impl Evaluation {
    fn every_slot_highest(&self) -> u32 {
        self.slots.iter().map(|slot| slot.instance).max().unwrap_or(0)
    }
    fn devices(&self) -> BTreeSet<u32> {
        self.slots.iter().map(|slot| slot.device).collect()
    }
    fn per_disk_highest(&self) -> Vec<u32> {
        let slots_of = |device: u32| self.slots.iter().filter(move |slot| slot.device == device);
        self.devices().into_iter().map(|device| slots_of(device).map(|slot| slot.instance).max().unwrap_or(0)).collect()
    }
    /// 择槽取世代号大的；相等取槽 0（与 recovery::choose_superblock、image::chosen_superblocks 同）。
    fn per_disk_chosen(&self) -> Vec<u32> {
        let slots_of = |device: u32| self.slots.iter().filter(move |slot| slot.device == device);
        self.devices()
            .into_iter()
            .map(|device| {
                slots_of(device)
                    .max_by_key(|slot| (slot.generation, std::cmp::Reverse(slot.slot_index)))
                    .map_or(0, |slot| slot.instance)
            })
            .collect()
    }
    fn per_slot(&self) -> Vec<u32> {
        self.slots.iter().map(|slot| slot.instance).collect()
    }
    /// ①：每个根 / 记录 / 单元写序的实例代号 ≤ 全部自证过的槽里最大的实例代号。
    fn first_sentence_violated(&self) -> bool {
        self.carriers.iter().any(|instance| *instance > self.every_slot_highest())
    }
    /// ②：给定读法下各盘（或各槽）的实例代号不等时，较大者不出现在任何根 / 记录 / 单元写序里。
    fn second_sentence_violated(&self, values: &[u32]) -> bool {
        let larger = values.iter().copied().max().unwrap_or(0);
        values.iter().any(|value| *value != larger) && self.carriers.contains(&larger)
    }
    fn highest_root(&self) -> u32 {
        self.root_instances.iter().copied().max().unwrap_or(0)
    }
    fn next_by_every_slot(&self) -> u32 {
        self.every_slot_highest().max(self.highest_root()) + 1
    }
    fn next_by_chosen(&self) -> u32 {
        self.per_disk_chosen().into_iter().max().unwrap_or(0).max(self.highest_root()) + 1
    }
}

fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 字节"))
}

fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

/// 头校验和（D18（块里携带什么信息） 已定项 17）：32 字节字段在偏移 10，罩 [0, 明文头末尾)，字段按 0 参与；
/// 前 4 字节小端 CRC32C、其余 28 字节恒 0。
fn header_checksum_holds(bytes: &[u8], header_end: usize) -> bool {
    let mut covered = bytes[..header_end].to_vec();
    covered[10..42].fill(0);
    bytes[10..14] == crc32_castagnoli_bitwise(&covered).to_le_bytes()
        && bytes[14..42].iter().all(|byte| *byte == 0)
}

/// 候选槽上一个单元头的写序实例代号（D18（块里携带什么信息） 已定项 18 的偏移表）：码 1 在 91、码 2 在 68 + 2k、码 3 在 93；
/// fsid 低 8 字节码 1 在 83、码 2 在 60 + 2k、码 3 在 81。头校验和不过、fsid 不同的不数。
fn unit_write_order_instance(
    image: &dyn ImageReader,
    device: u32,
    slot: u64,
    filesystem_identifier_low: u64,
) -> Option<u32> {
    let head = image.read(device, slot * SLOT_BYTES, 512)?;
    if &head[..4] != b"SFSU" || head[7] != 0 {
        return None;
    }
    let (header_end, identifier_offset, instance_offset) = match head[6] {
        1 => (105, 83, 91),
        2 => {
            let key_width = usize::from(head[51]);
            (86 + 2 * key_width, 60 + 2 * key_width, 68 + 2 * key_width)
        }
        3 => (107, 81, 93),
        _ => return None,
    };
    if header_end > head.len() || !header_checksum_holds(&head, header_end) {
        return None;
    }
    (read_u64_at(&head, identifier_offset) == filesystem_identifier_low)
        .then(|| read_u32_at(&head, instance_offset))
}

/// 读一个镜像：每盘两个超级块槽（槽距 4096）、根环、journal 环里提示的记录槽、单元区的候选槽。
fn evaluate<Image: PoolReader + ImageReader>(
    image: &Image,
    filesystem_identifier: &[u8; 16],
    with_today_checker: bool,
) -> Evaluation {
    let mut slots = Vec::new();
    for device in ImageReader::devices(image) {
        for slot_index in 0..2u64 {
            let offset = slot_index * SUPERBLOCK_SLOT_SPACING;
            let view = ImageReader::read(image, device, offset, 4096)
                .and_then(|bytes| check_superblock_slot(&bytes).ok())
                .filter(|view| view.filesystem_identifier == *filesystem_identifier);
            if let Some(view) = view {
                slots.push(SlotReading {
                    device,
                    slot_index,
                    generation: view.slot_generation,
                    instance: view.journal_instance,
                });
            }
        }
    }
    let geometry = chosen_superblocks(image)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
        .expect("至少一块盘有自证过的超级块槽");
    let root_instances: BTreeSet<u32> = valid_roots(image, &geometry)
        .iter()
        .map(|(_, _, root)| root.instance)
        .collect();
    let mut carriers = root_instances.clone();
    let filesystem_identifier_low =
        u64::from_le_bytes(filesystem_identifier[..8].try_into().expect("8 字节"));
    let ring_start = DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES);
    for device in PoolReader::device_identities(image) {
        let offsets = PoolReader::journal_record_offsets_hint(
            image,
            device,
            ring_start,
            geometry.journal_ring_bytes,
        )
        .expect("内存镜像给得出记录槽提示");
        for offset in offsets {
            let record = PoolReader::read(image, device, offset, 4096)
                .and_then(|bytes| check_journal_record(&bytes, filesystem_identifier_low).ok());
            if let Some(record) = record {
                carriers.insert(record.instance);
            }
        }
    }
    for device in ImageReader::devices(image) {
        for slot in ImageReader::candidate_unit_slots(image, device).expect("内存镜像给得出候选槽") {
            if let Some(instance) =
                unit_write_order_instance(image, device, slot, filesystem_identifier_low)
            {
                carriers.insert(instance);
            }
        }
    }
    let today_seven_seven_violated = with_today_checker
        && check_pool_image(image).iter().any(|(name, verdict)| {
            *name == "I-7.7" && matches!(verdict, InvariantVerdict::Violated(_))
        });
    Evaluation {
        slots,
        root_instances,
        carriers,
        today_seven_seven_violated,
    }
}

/// G2 审计：第一次挂载那条流里每一次超级块槽写，写之前这块盘上自证过的槽里最大的世代号 + 1，与写出的世代号、写的槽逐条比。
fn generation_audit(built: &Built) -> Vec<String> {
    let mut image = built.base_after_mkfs.clone();
    let mut lines = Vec::new();
    for write in &built.layer0_writes {
        if write.kind == StepKind::SuperblockSlot {
            let device = write.device.0;
            let highest_before = (0..2u64)
                .filter_map(|slot_index| {
                    ImageReader::read(&image, device, slot_index * SUPERBLOCK_SLOT_SPACING, 4096)
                })
                .filter_map(|bytes| check_superblock_slot(&bytes).ok())
                .map(|view| view.slot_generation)
                .max()
                .unwrap_or(0);
            let written = check_superblock_slot(&write.bytes).expect("写出的超级块槽自证");
            let by_rule = highest_before + 1;
            let offset_by_rule = (by_rule % 2) * SUPERBLOCK_SLOT_SPACING;
            lines.push(format!(
                "G2_AUDIT device={device} offset={} written_generation={} written_instance={} highest_self_verified_before={highest_before} generation_by_G2={by_rule} offset_by_G2={offset_by_rule} matches={}",
                write.offset.0,
                written.slot_generation,
                written.journal_instance,
                written.slot_generation == by_rule && write.offset.0 == offset_by_rule
            ));
        }
        image
            .devices
            .get_mut(&write.device)
            .expect("池里的盘")
            .write(write.offset, &write.bytes);
    }
    lines
}

/// 层 0 每个状态判的七件事；按全部槽读法与按择到的读法各算一次「下一次取号撞不撞号」。
const LAYER0_CHECKS: [&str; 7] = [
    "today_I-7.7",
    "first_sentence_every_slot",
    "second_sentence_by_disk_highest",
    "second_sentence_by_disk_chosen",
    "second_sentence_by_every_slot",
    "next_acquisition_collides_by_every_slot",
    "next_acquisition_collides_by_chosen",
];

#[derive(Default)]
struct Layer0Counts {
    states: u64,
    red: [u64; 7],
    first_red: [Option<String>; 7],
}

fn slots_text(evaluation: &Evaluation) -> String {
    let parts: Vec<String> = evaluation
        .slots
        .iter()
        .map(|slot| format!("d{}s{}:g{}i{}", slot.device, slot.slot_index, slot.generation, slot.instance))
        .collect();
    format!("[{}]", parts.join(" "))
}

impl Layer0Counts {
    fn observe(&mut self, built: &Built, persisted: Vec<bool>) {
        let landed: Vec<String> = persisted
            .iter()
            .zip(&built.layer0_writes)
            .filter(|(is_landed, _)| **is_landed)
            .map(|(_, write)| format!("{}@disk{}", write.kind.name(), write.device.0))
            .collect();
        let image = CrashImage {
            base: &built.base_after_mkfs,
            writes: &built.layer0_writes,
            persisted,
        };
        let evaluation = evaluate(&image, &built.parameters.filesystem_identifier, true);
        let verdicts = [
            evaluation.today_seven_seven_violated,
            evaluation.first_sentence_violated(),
            evaluation.second_sentence_violated(&evaluation.per_disk_highest()),
            evaluation.second_sentence_violated(&evaluation.per_disk_chosen()),
            evaluation.second_sentence_violated(&evaluation.per_slot()),
            evaluation.carriers.contains(&evaluation.next_by_every_slot()),
            evaluation.carriers.contains(&evaluation.next_by_chosen()),
        ];
        self.states += 1;
        for (position, is_red) in verdicts.iter().enumerate() {
            if *is_red {
                self.red[position] += 1;
                if self.first_red[position].is_none() {
                    self.first_red[position] = Some(format!(
                        "LAYER0_FIRST_RED check={} landed_writes=[{}] slots={} carriers={:?}",
                        LAYER0_CHECKS[position],
                        landed.join(" "),
                        slots_text(&evaluation),
                        evaluation.carriers
                    ));
                }
            }
        }
    }
}

/// 与 crates/singlefs-harness/src/crash.rs 的 enumerate_layer0 同一条规则：前面的段全持久 + 当前段任意真子集，
/// 最后再加全部持久那一个状态。
fn enumerate_layer0(built: &Built) -> Layer0Counts {
    let mut counts = Layer0Counts::default();
    let mut persisted_before = vec![false; built.layer0_writes.len()];
    for segment in &built.layer0_segments {
        for mask in 0..(1u64 << segment.len()) - 1 {
            let mut persisted = persisted_before.clone();
            for (bit_position, write_index) in segment.iter().enumerate() {
                if mask & (1 << bit_position) != 0 {
                    persisted[*write_index] = true;
                }
            }
            counts.observe(built, persisted);
        }
        for write_index in segment {
            persisted_before[*write_index] = true;
        }
    }
    counts.observe(built, persisted_before);
    counts
}

/// 在一块盘上按 G2 的槽（世代号 mod 2）写一份超级块，字段除世代号、tail、实例代号外与 PoolWriter::perform 写的相同。
fn put_superblock(
    image: &mut MemoryPool,
    parameters: &MakeFilesystemParameters,
    device: u32,
    generation: u64,
    journal_tail: u64,
    instance: u32,
) {
    let slot = Superblock {
        filesystem_identifier: parameters.filesystem_identifier,
        this_device: DeviceIdentity(device),
        device_count: 2,
        slot_generation: generation,
        region_devices: parameters.region_devices,
        geometry: parameters.geometry,
        journal_tail,
        journal_instance: InstanceGeneration(instance),
    }
    .to_slot();
    let offset = DeviceOffsetInBytes((generation % 2) * SUPERBLOCK_SLOT_SPACING);
    image.devices.get_mut(&DeviceIdentity(device)).expect("池里的盘").write(offset, &slot);
}

fn put_units(image: &mut MemoryPool, writes: &[RetainedWrite]) {
    for write in writes {
        image.devices.get_mut(&write.device).expect("池里的盘").write(write.offset, &write.bytes);
    }
}

fn report_image(name: &str, image: &MemoryPool, built: &Built) -> String {
    let evaluation = evaluate(image, &built.parameters.filesystem_identifier, true);
    let next_every = evaluation.next_by_every_slot();
    let next_chosen = evaluation.next_by_chosen();
    format!(
        "IMAGE name={name} slots={} carriers={:?} today_I-7.7_violated={} first_sentence_violated={} second_violated(by_disk_highest/by_disk_chosen/by_every_slot)={}/{}/{} next_by_every_slot={next_every} collides={} next_by_chosen={next_chosen} collides={}",
        slots_text(&evaluation),
        evaluation.carriers,
        evaluation.today_seven_seven_violated,
        evaluation.first_sentence_violated(),
        evaluation.second_sentence_violated(&evaluation.per_disk_highest()),
        evaluation.second_sentence_violated(&evaluation.per_disk_chosen()),
        evaluation.second_sentence_violated(&evaluation.per_slot()),
        evaluation.carriers.contains(&next_every),
        evaluation.carriers.contains(&next_chosen)
    )
}

/// 点名镜像：都从第一个事务写完的镜像（两盘槽 0 = (4, 1)、槽 1 = (5, 1)）起，按模型 M 里对应的历史写超级块与替身单元。
fn named_images(built: &Built) -> Vec<String> {
    let parameters = &built.parameters;
    let clean = &built.final_image;
    let mut lines = vec![report_image("clean_after_first_transaction", clean, built)];
    // 甲的撞号形态：带实例 2 的单元落了，取号那两次超级块写没落。
    // 两盘的屏障都报错并丢了缓存、实现重发屏障成功就继续发单元时，盘上是同一份字节。
    let mut jia = clean.clone();
    put_units(&mut jia, &built.stand_in_unit_writes);
    lines.push(report_image("units_of_instance_2_without_acquisition", &jia, built));
    // T1：第二次挂载取号 (6, 2) 两盘都落、单元带 2 落了、根没落；第三次挂载取号 (7, 3) 两盘的屏障都报错丢了缓存，
    // 全或无失败回卷，回卷写世代号 8 落槽 0；实例代号取「被取号写盖掉的那一槽原来的号」= 1，或「新号 − 1」「这块盘择到的旧号」= 2。
    for (label, rollback_instance) in [
        ("rollback_code_is_overwritten_slot_before_1", 1u32),
        ("rollback_code_is_new_minus_one_or_disk_chosen_2", 2),
    ] {
        let mut image = clean.clone();
        for device in 0..2 {
            put_superblock(&mut image, parameters, device, 6, 3, 2);
        }
        put_units(&mut image, &built.stand_in_unit_writes);
        for device in 0..2 {
            put_superblock(&mut image, parameters, device, 8, 3, rollback_instance);
        }
        lines.push(report_image(label, &image, built));
    }
    // 只有盘 1 的屏障报错丢了缓存、重发屏障就继续：盘 0 有 (6, 2)，盘 1 没有，单元带 2。
    let mut single = clean.clone();
    put_superblock(&mut single, parameters, 0, 6, 3, 2);
    put_units(&mut single, &built.stand_in_unit_writes);
    lines.push(report_image("disk1_barrier_error_then_proceed", &single, built));
    // 回卷路径上的合法态：盘 1 的取号写报错，盘 0 的 (6, 2) 落了，回卷写 (7, 1) 落在盘 0 槽 1；没有单元。
    let mut rolled = clean.clone();
    put_superblock(&mut rolled, parameters, 0, 6, 3, 2);
    put_superblock(&mut rolled, parameters, 0, 7, 3, 1);
    lines.push(report_image("disk1_write_error_rollback_landed", &rolled, built));
    // 坏镜像语料里 I-7.7 那一份（crates/singlefs-harness/tests/checker_known_bad_images.rs）：盘 1 被择的槽 1（世代 5）实例代号改成 0。
    let mut corpus = clean.clone();
    put_superblock(&mut corpus, parameters, 1, 5, 3, 0);
    lines.push(report_image("known_bad_corpus_disk1_chosen_slot_instance_0", &corpus, built));
    // 今天的写法：第二次取号写常量世代 2（transaction.rs 第 49 行）落槽 0，单元带 2（第二轮模型 R 那一格）。
    let mut today = clean.clone();
    for device in 0..2 {
        put_superblock(&mut today, parameters, device, 2, 0, 2);
    }
    put_units(&mut today, &built.stand_in_unit_writes);
    lines.push(report_image("today_constant_generation_2_then_units", &today, built));
    lines
}

fn main() {
    let built = build();
    let segment_sizes: Vec<usize> = built.layer0_segments.iter().map(Vec::len).collect();
    let stand_in_instances: BTreeSet<u32> = built
        .stand_in_unit_writes
        .iter()
        .filter_map(|write| unit_write_order_instance(&built.final_image, write.device.0, write.offset.0 / SLOT_BYTES, 0).or_else(|| {
            let bytes = &write.bytes;
            match bytes[6] {
                1 => Some(read_u32_at(bytes, 91)),
                2 => Some(read_u32_at(bytes, 68 + 2 * usize::from(bytes[51]))),
                3 => Some(read_u32_at(bytes, 93)),
                _ => None,
            }
        }))
        .collect();
    println!(
        "CONFIG layer0_writes={} segments={segment_sizes:?} stand_in_unit_writes={} stand_in_unit_instances={stand_in_instances:?}",
        built.layer0_writes.len(),
        built.stand_in_unit_writes.len()
    );
    for line in generation_audit(&built) {
        println!("{line}");
    }
    let counts = enumerate_layer0(&built);
    let per_check: Vec<String> =
        LAYER0_CHECKS.iter().zip(counts.red).map(|(name, red)| format!("{name}={red}")).collect();
    println!("LAYER0 states={} red_states {}", counts.states, per_check.join(" "));
    for first in counts.first_red.iter().flatten() {
        println!("{first}");
    }
    for line in named_images(&built) {
        println!("{line}");
    }
}
