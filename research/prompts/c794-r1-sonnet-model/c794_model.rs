//! C794 正推腿的模型：瞬时隔离、持久坏、写者漏释放、活单元记成已释放，四种镜像上池级 checker 与四个候选处置各判成什么。
//! 候选（checker 里 `C794_CANDIDATE` 选）：0 现行（核出坏的才豁免）、1 已分配而没有根引用的记录一律豁免、
//! 2 标记（盘上隔离位的语义，模型里由写者的隔离清单供给）、3 曾被某条走过的根引用就豁免（旁证）。
//! 只在草稿目录的副本里跑；输出行都以 `C794RESULT ` 开头。
mod common;

use common::{
    build_pool, parameters, publish_overwrite_in_process, BuiltPool, Recorded,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{check_pool_image, C794_CANDIDATE, C794_MARKED_SLOTS};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::Placement;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{raise_rollback_floor, ShadowLedger};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
    C794_SKIP_RELEASE_SLOT,
};
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::memory_pool::MemoryPool;
use std::cell::Cell;
use std::sync::atomic::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Form {
    Error,
    Corrupted,
}

/// 某个偏移上的读先坏 `to_refuse` 次（一次连续的坏读），之后放行；盘上的字节不动。
struct BurstAtOneOffset<Inner: BlockDevice> {
    inner: Inner,
    offset: Option<DeviceOffsetInBytes>,
    form: Form,
    to_refuse: Cell<u64>,
    seen: Cell<u64>,
}

impl<Inner: BlockDevice> BlockDevice for BurstAtOneOffset<Inner> {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if self.offset == Some(offset) {
            self.seen.set(self.seen.get() + 1);
            if self.to_refuse.get() > 0 {
                self.to_refuse.set(self.to_refuse.get() - 1);
                match self.form {
                    Form::Error => return Err(injected_block_device_error("读")),
                    Form::Corrupted => {
                        self.inner.read_at(offset, buffer)?;
                        buffer[4096] ^= 0xff;
                        return Ok(());
                    }
                }
            }
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], d: WriteDurability) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, d)
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

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 11 + seed) % 247).expect("小于 256"))
        .collect()
}

fn corrupt_on_disk(pool: &mut BuiltPool, device: DeviceIdentity, slot: SlotNumber) {
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let (_, block_device) = devices.iter_mut().find(|(i, _)| *i == device).expect("盘");
    let offset = DeviceOffsetInBytes(slot.to_device_offset().0 + 4096);
    let mut sector = vec![0u8; 512];
    block_device.read_at(offset, &mut sector).expect("读");
    sector[0] ^= 0xff;
    block_device.write_at(offset, &sector, WriteDurability::Plain).expect("写");
}

/// 覆盖写一次；`burst` 给了就让这个槽起的那一份在两块盘上各先坏 `burst.1` 次。
fn overwrite(
    pool: &mut BuiltPool,
    previous: &TransactionOutput,
    seed: usize,
    step: u64,
    burst: Option<(SlotNumber, u64, Form)>,
) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let content = content_of(2700, seed);
    let devices = pool.devices.take().expect("镜像还开着");
    let mut wrapped: Vec<(DeviceIdentity, BurstAtOneOffset<Recorded>)> = devices
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                BurstAtOneOffset {
                    inner: device,
                    offset: burst.map(|(slot, _, _)| slot.to_device_offset()),
                    form: burst.map_or(Form::Error, |(_, _, form)| form),
                    to_refuse: Cell::new(burst.map_or(0, |(_, times, _)| times)),
                    seen: Cell::new(0),
                },
            )
        })
        .collect();
    let result = {
        let mut writer = PoolWriter::new(&publish_parameters, wrapped.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 * step,
            },
            InstanceGeneration(1),
        )
    };
    pool.devices = Some(wrapped.into_iter().map(|(i, d)| (i, d.inner)).collect());
    result
}

struct Verdicts {
    status: String,
    red: Vec<&'static str>,
    detail_i311: String,
    detail_i31: String,
}

fn judge(image: &MemoryPool) -> Verdicts {
    let mut red = Vec::new();
    let mut detail_i311 = String::new();
    let mut detail_i31 = String::new();
    let mut status = String::new();
    for (name, verdict) in check_pool_image(image) {
        if name == "I-3.1" || name == "I-3.11" || name == "I-5.2" {
            let word = match &verdict {
                InvariantVerdict::Holds => "holds".to_string(),
                InvariantVerdict::Violated(_) => "violated".to_string(),
                InvariantVerdict::NotApplicable(why) => format!("not_applicable({why})"),
                InvariantVerdict::PartiallyJudged(_) => "partial".to_string(),
            };
            status.push_str(&format!(" {name}={word}"));
        }
        if let InvariantVerdict::Violated(detail) = verdict {
            if name == "I-3.11" {
                detail_i311 = detail.clone();
            }
            if name == "I-3.1" {
                detail_i31 = detail.clone();
            }
            red.push(name);
        }
    }
    Verdicts { status, red, detail_i311, detail_i31 }
}

/// 一张镜像在四个候选下各红哪几条。
fn print_candidates(mirror: &str, stage: &str, image: &MemoryPool, marked: &[u64], show_detail: bool) -> [Vec<&'static str>; 4] {
    *C794_MARKED_SLOTS.lock().unwrap() = marked.to_vec();
    let mut out: [Vec<&'static str>; 4] = Default::default();
    for (index, label) in ["keep_current", "exempt_every_unreferenced_record", "on_disk_quarantine_mark", "once_referenced_evidence"].iter().enumerate() {
        C794_CANDIDATE.store(u8::try_from(index).unwrap(), Ordering::SeqCst);
        let verdicts = judge(image);
        println!(
            "C794RESULT mirror={mirror} stage={stage} candidate={label} red={:?}{}",
            verdicts.red, verdicts.status
        );
        if show_detail && index == 0 {
            if !verdicts.detail_i311.is_empty() {
                println!("C794DETAIL mirror={mirror} stage={stage} I-3.11 {}", verdicts.detail_i311);
            }
            if !verdicts.detail_i31.is_empty() {
                println!("C794DETAIL mirror={mirror} stage={stage} I-3.1 {}", verdicts.detail_i31);
            }
        }
        out[index] = verdicts.red;
    }
    C794_CANDIDATE.store(0, Ordering::SeqCst);
    out
}

fn data_slot(pool: &BuiltPool) -> SlotNumber {
    pool.output.data_pointers[0].locations[0].slot
}

#[derive(Clone, Copy, Debug)]
enum Mirror {
    Healthy,
    /// 释放核那一刻连着坏两读、之后两份都完好（瞬时隔离）。
    TransientTwoBadReads(Form),
    /// 盘 0 上那一份盘上字节坏、盘 1 那一份好（持久坏、一份坏）。
    PersistentOneCopyBad,
    /// 写者漏释放：A 的数据单元那一槽的释放整个跳过。
    WriterForgetsTheRelease,
    /// 一个从来没有根引用过的已分配单元（发布前分配、发布不引用），那一槽里什么也没写。
    OrphanNeverReferenced,
    /// 同上，但那一槽里写着一个完整的单元（拷自 A 的数据单元，自证过得去）。
    OrphanNeverReferencedButWrittenAsAValidUnit,
}

impl Mirror {
    fn name(self) -> String {
        match self {
            Mirror::Healthy => "healthy".into(),
            Mirror::TransientTwoBadReads(Form::Error) => "transient_two_read_errors".into(),
            Mirror::TransientTwoBadReads(Form::Corrupted) => "transient_two_corrupted_reads".into(),
            Mirror::PersistentOneCopyBad => "persistent_one_copy_bad".into(),
            Mirror::WriterForgetsTheRelease => "writer_forgets_the_release".into(),
            Mirror::OrphanNeverReferenced => "orphan_never_referenced".into(),
            Mirror::OrphanNeverReferencedButWrittenAsAValidUnit => "orphan_never_referenced_written_valid".into(),
        }
    }
}

/// 发布 B（txg 4）按镜像造；之后接着覆盖写到 `last_txg`，每一步交一张镜像；返回每一步的镜像、写者隔离的槽。
fn run(mirror: Mirror, last_txg: u64, tag: &str) -> (BuiltPool, TransactionOutput, Vec<(String, MemoryPool)>, Vec<u64>) {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let a_slot = data_slot(&pool);
    let mut stages = Vec::new();
    let mut marked: Vec<u64> = Vec::new();
    C794_SKIP_RELEASE_SLOT.store(u64::MAX, Ordering::SeqCst);
    let mut current = match mirror {
        Mirror::Healthy => overwrite(&mut pool, &first, 5, 1, None).expect("B"),
        Mirror::TransientTwoBadReads(form) => {
            let b = overwrite(&mut pool, &first, 5, 1, Some((a_slot, 2, form))).expect("B");
            assert_eq!(b.quarantined_after_release_checksum_mismatch.len(), 2, "两盘那一份都隔离");
            marked.push(a_slot.0);
            b
        }
        Mirror::PersistentOneCopyBad => {
            corrupt_on_disk(&mut pool, DeviceIdentity(0), a_slot);
            let b = overwrite(&mut pool, &first, 5, 1, None).expect("B");
            assert_eq!(b.quarantined_after_release_checksum_mismatch.len(), 2);
            marked.push(a_slot.0);
            b
        }
        Mirror::WriterForgetsTheRelease => {
            C794_SKIP_RELEASE_SLOT.store(a_slot.0, Ordering::SeqCst);
            let b = overwrite(&mut pool, &first, 5, 1, None).expect("B");
            C794_SKIP_RELEASE_SLOT.store(u64::MAX, Ordering::SeqCst);
            assert!(b.quarantined_after_release_checksum_mismatch.is_empty());
            b
        }
        Mirror::OrphanNeverReferenced | Mirror::OrphanNeverReferencedButWrittenAsAValidUnit => {
            // 写进去的是 A 的数据单元（头里诞生代号 3）：分配代也取 3，I-3.10 不因为模型自己的拷贝而红。
            let generation = if matches!(mirror, Mirror::OrphanNeverReferencedButWrittenAsAValidUnit) { 3 } else { 4 };
            let orphan = pool.allocator.allocate_user_data(CheckpointTxg(generation)).expect("分得到");
            println!("C794RESULT mirror={} orphan_slot={}", mirror.name(), orphan.slot.0);
            if matches!(mirror, Mirror::OrphanNeverReferencedButWrittenAsAValidUnit) {
                let devices = pool.devices.as_mut().expect("镜像还开着");
                for (_, device) in devices.iter_mut() {
                    let mut unit = vec![0u8; 32768];
                    device.read_at(a_slot.to_device_offset(), &mut unit).expect("读 A");
                    device
                        .write_at(orphan.slot.to_device_offset(), &unit, WriteDurability::Plain)
                        .expect("写孤儿槽");
                }
            }
            overwrite(&mut pool, &first, 5, 1, None).expect("B")
        }
    };
    stages.push(("txg4_after_B".to_string(), pool.memory_pool()));
    let mut txg = 4;
    while txg < last_txg {
        txg += 1;
        current = overwrite(&mut pool, &current, 5 + usize::try_from(txg).unwrap(), txg - 3, None).expect("接着覆盖写");
        stages.push((format!("txg{txg}"), pool.memory_pool()));
    }
    (pool, current, stages, marked)
}

#[test]
fn c794_without_a_floor_raise() {
    for mirror in [
        Mirror::Healthy,
        Mirror::TransientTwoBadReads(Form::Error),
        Mirror::TransientTwoBadReads(Form::Corrupted),
        Mirror::PersistentOneCopyBad,
        Mirror::WriterForgetsTheRelease,
        Mirror::OrphanNeverReferenced,
        Mirror::OrphanNeverReferencedButWrittenAsAValidUnit,
    ] {
        let (_pool, _current, stages, marked) = run(mirror, 12, &format!("c794-nf-{}", mirror.name()));
        for (stage, image) in &stages {
            print_candidates(&mirror.name(), stage, image, &marked, stage == "txg4_after_B");
        }
    }
}

#[test]
fn c794_with_a_floor_raise_to_4() {
    for mirror in [
        Mirror::Healthy,
        Mirror::TransientTwoBadReads(Form::Error),
        Mirror::TransientTwoBadReads(Form::Corrupted),
        Mirror::PersistentOneCopyBad,
        Mirror::WriterForgetsTheRelease,
        Mirror::OrphanNeverReferenced,
        Mirror::OrphanNeverReferencedButWrittenAsAValidUnit,
    ] {
        let (mut pool, mut current, _stages, marked) = run(mirror, 7, &format!("c794-fr-{}", mirror.name()));
        let publish_parameters = parameters();
        let raised = {
            let devices = pool.devices.as_mut().expect("镜像还开着");
            raise_rollback_floor(&publish_parameters, devices, &mut pool.allocator, &mut current, CheckpointTxg(4), ShadowLedger::On)
                .expect("抬 F 到 4")
        };
        println!("C794RESULT mirror={} floor_raise_publishes={}", mirror.name(), raised.publishes.len());
        print_candidates(&mirror.name(), "txg7_floor_raised_to_4", &pool.memory_pool(), &marked, true);
    }
}

/// 抬 F 那一串空发布也读盘核：上一版记账树那一单元的读在抬 F 里连坏两次，该单元被隔离（可达性的第二个站点）。
#[test]
fn c794_transient_inside_the_floor_raise_publishes() {
    let (mut pool, mut current, _stages, _marked) = run(Mirror::Healthy, 7, "c794-fr-site");
    let accounting_slot = current.unit(TransactionUnit::AccountingTree).slot;
    let publish_parameters = parameters();
    let devices = pool.devices.take().expect("镜像还开着");
    let mut wrapped: Vec<(DeviceIdentity, BurstAtOneOffset<Recorded>)> = devices
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                BurstAtOneOffset {
                    inner: device,
                    offset: Some(accounting_slot.to_device_offset()),
                    form: Form::Error,
                    to_refuse: Cell::new(2),
                    seen: Cell::new(0),
                },
            )
        })
        .collect();
    let raised = raise_rollback_floor(&publish_parameters, &mut wrapped, &mut pool.allocator, &mut current, CheckpointTxg(4), ShadowLedger::On)
        .expect("抬 F");
    let quarantined: usize = raised.publishes.iter().map(|p| p.quarantined_after_release_checksum_mismatch.len()).sum();
    println!(
        "C794RESULT mirror=transient_inside_floor_raise accounting_slot={} publishes={} quarantined_copies_in_the_publishes={quarantined}",
        accounting_slot.0,
        raised.publishes.len()
    );
    pool.devices = Some(wrapped.into_iter().map(|(i, d)| (i, d.inner)).collect());
    let marked = [accounting_slot.0];
    print_candidates("transient_inside_floor_raise", "after_floor_raise", &pool.memory_pool(), &marked, true);
}

/// 活单元记成已释放、放进 defer（Z10 那一种）：A 之后第五版的数据单元由写者记成已释放，再发空发布。
#[test]
fn c794_live_unit_recorded_as_released() {
    for with_a_quarantined_unit_present in [false, true] {
        let mirror = if with_a_quarantined_unit_present {
            Mirror::TransientTwoBadReads(Form::Error)
        } else {
            Mirror::Healthy
        };
        let (mut pool, mut current, _stages, marked) = run(mirror, 7, "c794-z10");
        let live = current.data_pointers[0].locations[0].slot;
        pool.allocator.release(Placement { slot: live, span: 2 }, CheckpointTxg(8));
        let publish_parameters = parameters();
        let raised = {
            let devices = pool.devices.as_mut().expect("镜像还开着");
            raise_rollback_floor(&publish_parameters, devices, &mut pool.allocator, &mut current, CheckpointTxg(4), ShadowLedger::On)
                .expect("抬 F")
        };
        println!("C794RESULT mirror=z10 with_quarantine={with_a_quarantined_unit_present} publishes={} live_slot={}", raised.publishes.len(), live.0);
        print_candidates(if with_a_quarantined_unit_present { "z10_plus_transient" } else { "z10" }, "txg7_after_empty_publishes", &pool.memory_pool(), &marked, true);
    }
}

/// 瞬时隔离与写者漏释放：两条完全不同的代码路径造出来的镜像是不是逐字节相同（发布 B 之后；再覆盖写到 txg 7 并抬 F 到 4 之后）。
#[test]
fn c794_transient_quarantine_and_forgotten_release_leave_the_same_bytes() {
    let mut images: Vec<(String, MemoryPool, MemoryPool)> = Vec::new();
    for (label, mirror) in [
        ("transient_two_read_errors", Mirror::TransientTwoBadReads(Form::Error)),
        ("transient_two_corrupted_reads", Mirror::TransientTwoBadReads(Form::Corrupted)),
        ("writer_forgets_the_release", Mirror::WriterForgetsTheRelease),
    ] {
        let (mut pool, mut current, stages, _marked) = run(mirror, 7, &format!("c794-same-{label}"));
        let after_b = stages[0].1.clone();
        let publish_parameters = parameters();
        {
            let devices = pool.devices.as_mut().expect("镜像还开着");
            raise_rollback_floor(&publish_parameters, devices, &mut pool.allocator, &mut current, CheckpointTxg(4), ShadowLedger::On).expect("抬 F");
        }
        images.push((label.to_string(), after_b, pool.memory_pool()));
    }
    for index in 1..images.len() {
        for (stage, left, right) in [
            ("txg4_after_B", &images[0].1, &images[index].1),
            ("txg7_floor_raised_to_4", &images[0].2, &images[index].2),
        ] {
            let mut differing_sectors = 0usize;
            for (device, left_device) in &left.devices {
                let right_device = &right.devices[device];
                let left_sectors = left_device.written_sectors();
                let right_sectors = right_device.written_sectors();
                let keys: std::collections::BTreeSet<_> = left_sectors.keys().chain(right_sectors.keys()).collect();
                for key in keys {
                    let zero = vec![0u8; 512];
                    let a = left_sectors.get(key).unwrap_or(&zero);
                    let b = right_sectors.get(key).unwrap_or(&zero);
                    if a != b {
                        differing_sectors += 1;
                    }
                }
            }
            println!(
                "C794RESULT same_bytes stage={stage} left={} right={} identical={} differing_sectors={differing_sectors}",
                images[0].0, images[index].0, differing_sectors == 0
            );
        }
    }
}

/// 根环轮转：瞬时隔离与漏释放在 A 的根被环覆写之后（不抬 F），A / C / D 各判什么。
#[test]
fn c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit() {
    for mirror in [Mirror::TransientTwoBadReads(Form::Error), Mirror::WriterForgetsTheRelease] {
        let (_pool, _current, stages, marked) = run(mirror, 44, &format!("c794-ring-{}", mirror.name()));
        for (stage, image) in &stages {
            let txg: u64 = stage.trim_start_matches("txg").trim_end_matches("_after_B").parse().unwrap();
            if [4, 20, 24, 26, 27, 28, 29, 30, 32, 36, 40, 44].contains(&txg) {
                print_candidates(&mirror.name(), stage, image, &marked, false);
            }
        }
    }
}
