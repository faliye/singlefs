//! 攻方腿（m3-close-w5-code-r1）的历史：按 inode 号写数据与多文件删除在交错写删、回退、快照、续做、上段长高之后删、删到只剩一个上的样子。
//! 盘是内存稀疏盘（`SparseBlockDevice`），录制流开内容保留。每条用例照条款写期望：断言红就是打中。
mod common;

use std::collections::BTreeSet;

use common::{crash_state_devices, file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::extent_tree::ExtentUpperNodePosition;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::transaction::{
    acquire_instance, publish_create_snapshot, publish_first_file, publish_new_inodes,
    publish_sequential_write_of_an_inode, publish_unlink_intent, publish_unlink_step, warm_up,
    FirstFile, PoolVersion, PoolWriter, PublishError, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type Device = RecordingBlockDevice<SparseBlockDevice>;
const INSTANCE: InstanceGeneration = InstanceGeneration(1);
const LATER: u64 = FIXED_WRITE_TIME_SECONDS + 600;

pub struct MemPool {
    pub devices: Vec<(DeviceIdentity, Device)>,
    pub stream: SharedStream,
    pub output: TransactionOutput,
    pub allocator: PoolAllocator,
}

impl MemPool {
    pub fn build() -> MemPool {
        let parameters = parameters();
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<(DeviceIdentity, Device)> = (0..2u32)
            .map(|number| {
                (
                    DeviceIdentity(number),
                    RecordingBlockDevice::with_shared_stream(
                        DeviceIdentity(number),
                        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect();
        let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
            DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
        ]);
        allocator.mark_format_time_units(
            Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
            Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
        );
        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
            &parameters,
            &genesis,
        ));
        let content = file_content();
        let output = {
            let mut pool = PoolWriter::new(&parameters, &mut devices);
            let instance = acquire_instance(&mut pool).expect("取号");
            let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
            publish_first_file(
                &mut pool,
                &mut allocator,
                warm.roots.last().expect("暖机两代根"),
                FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
                instance,
                &warm.last_record_bytes,
            )
            .expect("新池新建文件")
        };
        MemPool { devices, stream, output, allocator }
    }

    pub fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                .collect(),
            device_size_in_bytes: IMAGE_BYTES,
        }
    }

    pub fn create_inodes(&mut self, count: u64) {
        let mut left = count;
        while left > 0 {
            let now = left.min(200);
            let previous = self.output.clone();
            let parameters = parameters();
            let mut writer = PoolWriter::new(&parameters, &mut self.devices);
            self.output = publish_new_inodes(&mut writer, &mut self.allocator, &previous, now, LATER, INSTANCE)
                .expect("建 inode");
            left -= now;
        }
    }

    pub fn try_write(&mut self, inode: u64, content: &[u8]) -> Result<(), PublishError> {
        let previous = self.output.clone();
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, &mut self.devices);
        self.output = publish_sequential_write_of_an_inode(
            &mut writer,
            &mut self.allocator,
            &previous,
            InodeNumber(inode),
            content,
            LATER,
            INSTANCE,
        )?;
        Ok(())
    }

    pub fn write(&mut self, inode: u64, content: &[u8]) {
        self.try_write(inode, content)
            .unwrap_or_else(|refusal| panic!("写 inode {inode}：{refusal:?}"));
    }

    /// 意图那次加全部批，每一版交池级 checker；交回每一版。
    pub fn unlink(&mut self, inode: u64, batch: UnlinkBatchSize, what: &str) -> Vec<TransactionOutput> {
        let parameters = parameters();
        let mut versions = Vec::new();
        {
            let previous = self.output.clone();
            let mut writer = PoolWriter::new(&parameters, &mut self.devices);
            self.output = publish_unlink_intent(&mut writer, &mut self.allocator, &previous, InodeNumber(inode), LATER, INSTANCE)
                .unwrap_or_else(|refusal| panic!("{what}：unlink {inode} 的意图：{refusal:?}"));
        }
        assert_checker_green(&self.image(), &format!("{what}：unlink {inode} 意图之后"));
        versions.push(self.output.clone());
        let mut steps = 0;
        while self.output.deleted_inodes.contains(&InodeNumber(inode)) {
            steps += 1;
            assert!(steps < 400, "{what}：unlink {inode} 走不到最后一批");
            let previous = self.output.clone();
            let mut writer = PoolWriter::new(&parameters, &mut self.devices);
            self.output = publish_unlink_step(&mut writer, &mut self.allocator, &previous, InodeNumber(inode), batch, INSTANCE)
                .unwrap_or_else(|refusal| panic!("{what}：unlink {inode} 第 {steps} 步：{refusal:?}"));
            assert_checker_green(&self.image(), &format!("{what}：unlink {inode} 第 {steps} 步之后"));
            versions.push(self.output.clone());
        }
        versions
    }

    pub fn roll_back_to(&mut self, target: &TransactionOutput) -> Result<(), String> {
        let parameters = parameters();
        roll_back_by_a_forward_publish(
            &parameters,
            &mut self.devices,
            &mut self.allocator,
            &mut self.output,
            RollbackTarget { instance: target.root.instance, checkpoint_txg: target.root.checkpoint_txg },
        )
        .map(|_| ())
        .map_err(|error| format!("{error:?}"))
    }

    pub fn snapshot(&mut self) {
        let previous = self.output.clone();
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, &mut self.devices);
        self.output = publish_create_snapshot(&mut writer, &mut self.allocator, &previous, INSTANCE).expect("建快照");
    }
}

pub fn batches_of(units: u64) -> UnlinkBatchSize {
    UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(units).expect("1..=144"))
}

pub fn content_of(length: usize, seed: u64) -> Vec<u8> {
    let mut state = seed | 1;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).expect("低 8 位")
        })
        .collect()
}

pub fn units(count: usize, seed: u64) -> Vec<u8> {
    content_of(count * data_unit_payload_capacity() - 5, seed)
}

pub fn violations(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            _ => None,
        })
        .collect()
}

pub fn assert_checker_green(image: &MemoryPool, what: &str) {
    let found = violations(image);
    assert!(found.is_empty(), "{what}：池级 checker 红 {found:?}");
}

pub fn read_back(image: &MemoryPool, inode: u64) -> Result<Vec<u8>, String> {
    let mounted = mount_read_only(image).map_err(|failure| format!("只读挂载 {failure:?}"))?;
    let file = mounted
        .mounted
        .open_file(image, InodeNumber(inode))
        .map_err(|failure| format!("打开 {inode}：{failure:?}"))?;
    let size = file.inode_record().size;
    file.read_at(image, FileOffsetInBytes(0), size)
        .map(|read| read.bytes)
        .map_err(|failure| format!("读 {inode}：{failure:?}"))
}

pub fn upper_positions(version: &TransactionOutput) -> Vec<ExtentUpperNodePosition> {
    version.extent_tree.upper_nodes.iter().map(|(position, _)| *position).collect()
}

/// 这一份镜像可写挂载一次（续做按 `batch` 走），交回挂载之后的现行那一版。
pub fn mount_a_copy(image: &MemoryPool, batch: UnlinkBatchSize) -> Result<(PoolVersion, MemoryPool), String> {
    let stream = SharedStream::new();
    let mut devices = crash_state_devices(image, &[], &[], &stream);
    let switches = DeletionSwitches { batch_size: batch, ..DeletionSwitches::PRODUCT };
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches,
    )
    .map_err(|error| format!("可写挂载 {error:?}"))?;
    let after = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    };
    Ok((mounted.current, after))
}

pub fn set_of(values: &[u64]) -> BTreeSet<u64> {
    values.iter().copied().collect()
}

pub const FIRST: u64 = FIRST_INODE_NUMBER;

/// W4 β「取死亡 txg 最大」：墓碑随现行那一版、回退不取目标的。删 1（d1）→ 建 3（b3 > d1，写 3 被拒）→ 回退到删 1 之前 → 再删 1（d2 > b3）
/// → 回退到建 3 那一版（3 在、1 不在，这条时间线上 3 是删完 1 之后才建的）→ 写 3。β 照这条时间线该拒；取最大那条墓碑（d2）就放过。
#[test]
fn beta_after_two_rollbacks_lets_a_write_to_an_inode_born_after_the_deletion_through() {
    let mut pool = MemPool::build();
    pool.create_inodes(1);
    pool.write(2, &units(2, 0x22));
    let before_deleting_the_first_file = pool.output.clone();
    let first_deletion = pool.unlink(FIRST, UnlinkBatchSize::OneExtentLowerLeaf, "第一次删 1");
    let d1 = first_deletion[0].root.checkpoint_txg;
    pool.create_inodes(1);
    let version_with_inode_three = pool.output.clone();
    let b3 = version_with_inode_three.inode_record_of(InodeNumber(3)).expect("3 在").object_birth;
    eprintln!("d1 = {d1:?}，b3 = {b3:?}");
    let refused_before = pool.try_write(3, &units(1, 0x33));
    eprintln!("回退之前写 3：{refused_before:?}");
    assert!(refused_before.is_err(), "删完 1 之后才建的 3：β 拒");
    pool.roll_back_to(&before_deleting_the_first_file).expect("回退到删 1 之前");
    let second_deletion = pool.unlink(FIRST, UnlinkBatchSize::OneExtentLowerLeaf, "第二次删 1");
    let d2 = second_deletion[0].root.checkpoint_txg;
    eprintln!("d2 = {d2:?}");
    match pool.roll_back_to(&version_with_inode_three) {
        Ok(()) => {}
        Err(error) => panic!("回退到建 3 那一版被拒（这条历史造不出）：{error}"),
    }
    assert!(pool.output.inode_record.is_none(), "回退之后 1 不在");
    let b3_now = pool.output.inode_record_of(InodeNumber(3)).expect("回退之后 3 在").object_birth;
    eprintln!(
        "回退之后：3 的出生 {b3_now:?}，β 读到的死亡 txg {:?}",
        pool.output.death_txg_of_the_deleted_first_file()
    );
    let attempt = pool.try_write(3, &units(1, 0x34));
    eprintln!("回退到建 3 那一版之后写 3：{attempt:?}");
    if attempt.is_ok() {
        let image = pool.image();
        eprintln!("写成之后读回 3：{:?} 字节；checker 红 {:?}", read_back(&image, 3).map(|bytes| bytes.len()), violations(&image));
    }
    assert!(
        attempt.is_err(),
        "这条时间线上 3 生于 1 删完（d1 = {d1:?}）之后（b3 = {b3:?}），β 该拒；实际放过（取了别的时间线上的 d2 = {d2:?}）"
    );
}

/// W2 / W3：上段长到层级 2 之后删到只剩一个。S = {1, 150, 21100}（21100 要层级 2：层级 1 一个节点罩 143 × 147 = 21021 个号），
/// 删 1 → {150, 21100}，删 21100 → {150}（降两层到层级 1，(1, 0) 照抄成根），删 150 → 空。每一步 checker 全绿（含 I-9.40），
/// 剩下的文件逐字节读回，每一版可写挂载一次照常。
#[test]
fn the_upper_segment_grown_to_level_two_is_deleted_down_to_one_file_and_then_to_none() {
    let mut pool = MemPool::build();
    pool.create_inodes(21_100);
    let content_150 = units(3, 0x150);
    let content_21100 = units(2, 0x21100);
    pool.write(150, &content_150);
    pool.write(21_100, &content_21100);
    eprintln!("写完：根层级 {}，上段 {:?}", pool.output.extent_tree.upper_root().level, upper_positions(&pool.output));
    assert_checker_green(&pool.image(), "写完 150 与 21100");
    pool.unlink(FIRST, batches_of(1), "删 1");
    eprintln!("删 1 之后：上段 {:?}", upper_positions(&pool.output));
    assert_eq!(read_back(&pool.image(), 150).expect("读 150"), content_150);
    assert_eq!(read_back(&pool.image(), 21_100).expect("读 21100"), content_21100);
    mount_a_copy(&pool.image(), batches_of(1)).expect("删 1 之后可写挂载");
    pool.unlink(21_100, batches_of(1), "删 21100");
    eprintln!("删 21100 之后：上段 {:?}", upper_positions(&pool.output));
    assert_eq!(read_back(&pool.image(), 150).expect("读 150"), content_150);
    mount_a_copy(&pool.image(), batches_of(1)).expect("删 21100 之后可写挂载");
    pool.unlink(150, batches_of(1), "删 150");
    eprintln!("删 150 之后：上段 {:?}，成员 {}", upper_positions(&pool.output),
        if pool.output.is_a_version_with_trees_and_no_file() { "有树无文件" } else { "带文件" });
    mount_a_copy(&pool.image(), batches_of(1)).expect("删 150 之后可写挂载");
}

/// W4 / D5 已定项 19：活快照罩着别的文件时按批删它（批大小 1），每一版 checker 全绿；删完之后可写挂载照常。
#[test]
fn deleting_another_file_in_batches_under_a_live_snapshot_keeps_every_version_green() {
    let mut pool = MemPool::build();
    pool.create_inodes(2);
    pool.write(3, &units(3, 0x31));
    pool.write(2, &units(2, 0x21));
    pool.snapshot();
    assert_checker_green(&pool.image(), "建快照之后");
    pool.unlink(3, batches_of(1), "快照之下删 3");
    pool.write(2, &units(1, 0x22));
    assert_checker_green(&pool.image(), "快照之下改写 2");
    pool.unlink(FIRST, batches_of(1), "快照之下删 1");
    pool.unlink(2, batches_of(1), "快照之下删 2");
    mount_a_copy(&pool.image(), batches_of(1)).expect("快照之下删空之后可写挂载");
}

/// W4：回退到别的文件删除的意图那一版（号回到队列），可写挂载续做把它走完；再回退到它还活着的那一版、再删一次（同一对象两条墓碑）。
#[test]
fn rolling_back_into_the_middle_of_the_deletion_of_another_file_is_continued_by_the_mount() {
    let mut pool = MemPool::build();
    pool.create_inodes(1);
    let content_two = units(3, 0x2a);
    pool.write(2, &content_two);
    let alive = pool.output.clone();
    let versions = pool.unlink(2, batches_of(1), "删 2");
    let intent = versions[0].clone();
    let after_first_batch = versions[1].clone();
    pool.write(FIRST, &units(2, 0x1a));
    pool.roll_back_to(&after_first_batch).expect("回退到删 2 的第一批之后");
    assert_checker_green(&pool.image(), "回退到删 2 的中途");
    let (continued, image) = mount_a_copy(&pool.image(), batches_of(1)).expect("回退到中途之后可写挂载续做");
    let continued = continued.file_version().expect("带文件").clone();
    assert!(continued.inode_record_of(InodeNumber(2)).is_none() && continued.deleted_inodes.is_empty(), "续做删完 2");
    assert_checker_green(&image, "续做删完 2 之后");
    pool.roll_back_to(&intent).expect("回退到删 2 的意图那一版");
    assert_checker_green(&pool.image(), "回退到删 2 的意图");
    pool.roll_back_to(&alive).expect("回退到 2 活着那一版");
    assert_eq!(read_back(&pool.image(), 2).expect("读 2"), content_two);
    pool.unlink(2, UnlinkBatchSize::OneExtentLowerLeaf, "再删一次 2");
    let tombstones_of_two = pool
        .output
        .tombstone_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .filter(|tombstone| tombstone.object.0 == 2)
        .count();
    eprintln!("2 的墓碑条数：{tombstones_of_two}");
    mount_a_copy(&pool.image(), batches_of(1)).expect("再删 2 之后可写挂载");
}

/// W1 / W4：按 inode 号写长度 0 的内容（第一个文件长度 0 也写一个单元），之后可写挂载、按批删它。
#[test]
fn an_empty_write_to_another_inode_mounts_and_is_deleted() {
    let mut pool = MemPool::build();
    pool.create_inodes(1);
    pool.write(2, &[]);
    eprintln!(
        "写空内容之后：2 的单元数 {:?}，size {:?}，有条目的号 {:?}",
        pool.output.file_other_than_the_first(InodeNumber(2)).map(|file| file.data_pointers.len()),
        pool.output.inode_record_of(InodeNumber(2)).map(|record| record.size),
        pool.output.inodes_with_an_extent_upper_leaf_entry()
    );
    assert_checker_green(&pool.image(), "写空内容之后");
    mount_a_copy(&pool.image(), batches_of(1)).expect("写空内容之后可写挂载");
    assert_eq!(read_back(&pool.image(), 2).expect("读 2"), Vec::<u8>::new());
    pool.unlink(2, batches_of(1), "删空内容的 2");
    mount_a_copy(&pool.image(), batches_of(1)).expect("删之后可写挂载");
}

/// W3：两种删序的终态（条款不要求逐字节同；这里只报上段与逻辑内容是否同，量一下差在哪）。删别的文件时第一个文件读回不变。
#[test]
fn two_deletion_orders_of_three_files_report_their_final_upper_segments() {
    let mut shapes = Vec::new();
    for order in [[FIRST, 2, 150], [150, 2, FIRST], [2, FIRST, 150]] {
        let mut pool = MemPool::build();
        pool.create_inodes(149);
        pool.write(2, &units(2, 0x2b));
        pool.write(150, &units(3, 0x15b));
        for inode in order {
            pool.unlink(inode, batches_of(1), &format!("删序 {order:?}"));
        }
        shapes.push((order, upper_positions(&pool.output), pool.output.is_a_version_with_trees_and_no_file()));
        mount_a_copy(&pool.image(), batches_of(1)).expect("删空之后可写挂载");
    }
    eprintln!("各删序终态：{shapes:?}");
    assert!(shapes.windows(2).all(|pair| pair[0].1 == pair[1].1 && pair[0].2 == pair[1].2));
}

/// 从录制流第 `start` 步起（之前那一段施加成基线）切段：交回基线、写表与段（写表下标）。
fn segments_after(pool: &MemPool, start: usize) -> (MemoryPool, Vec<singlefs_harness::memory_pool::RetainedWrite>, Vec<Vec<usize>>) {
    let operations = pool.stream.retained_operations();
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    base.apply(&operations[..start]);
    let (writes, segments) =
        singlefs_harness::memory_pool::writes_and_segments(&operations[start..], &common::geometry());
    (base, writes, segments)
}

/// 每一段展开（前面各段全持久、这一段写的每个整写子集），逐个交给 `judge`；不先攒成表。只取每次写两态，原地覆写的撕裂第三态不在里面。
const SHORT_SEGMENT_WRITES: usize = 2;

fn for_each_crash_state(writes: usize, segments: &[Vec<usize>], mut judge: impl FnMut(u64, &[bool])) -> u64 {
    let mut persisted_before = vec![false; writes];
    let mut index = 0u64;
    judge(index, &persisted_before);
    for segment in segments {
        // 段边界那几个状态，加短段（至多 SHORT_SEGMENT_WRITES 次写）全展开；长段只取全持久那一个（不是崩溃点重放的全量）。
        let subsets = if segment.len() <= SHORT_SEGMENT_WRITES { 1u64 << segment.len() } else { 2 };
        for mask in 1..subsets {
            let mask = if segment.len() <= SHORT_SEGMENT_WRITES { mask } else { (1u64 << segment.len()) - 1 };
            let mut persisted = persisted_before.clone();
            for (bit, write) in segment.iter().enumerate() {
                if mask & (1 << bit) != 0 {
                    persisted[*write] = true;
                }
            }
            index += 1;
            judge(index, &persisted);
        }
        for write in segment {
            persisted_before[*write] = true;
        }
    }
    index + 1
}

/// 每个崩溃状态：可写挂载（续做按批大小 1 走完），挂载做成、续做之后队列空、第一个文件与 `kept` 里的文件逐字节读回、`target` 要么整份在要么没了，
/// 池级 checker 全绿。交回判过的状态数。
fn judge_every_crash_state(pool: &MemPool, start: usize, what: &str, target: u64, target_content: &[u8], kept: &[(u64, Vec<u8>)], states_at_most: u64) -> u64 {
    let (base, writes, segments) = segments_after(pool, start);
    let count = singlefs_harness::memory_pool::closed_form_state_count(&segments);
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    eprintln!("{what}：写 {} 次，段长 {sizes:?}，闭式状态数 {count}", writes.len());
    let _ = states_at_most;
    let judged = for_each_crash_state(writes.len(), &segments, |index, persisted| {
        let stream = SharedStream::new();
        let mut devices = crash_state_devices(&base, &writes, persisted, &stream);
        let switches = DeletionSwitches { batch_size: batches_of(1), ..DeletionSwitches::PRODUCT };
        let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
            &parameters(),
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            switches,
        )
        .unwrap_or_else(|error| panic!("{what}：状态 {index} 可写挂载被拒：{error:?}"));
        let current = mounted.current.file_version().unwrap_or_else(|| panic!("{what}：状态 {index} 不是带文件的一版")).clone();
        assert!(current.deleted_inodes.is_empty(), "{what}：状态 {index} 续做之后队列不空");
        let image = MemoryPool {
            devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(),
            device_size_in_bytes: IMAGE_BYTES,
        };
        for (inode, content) in kept {
            assert_eq!(&read_back(&image, *inode).unwrap_or_else(|failure| panic!("{what}：状态 {index} 读 {inode}：{failure}")), content, "{what}：状态 {index} {inode} 读回");
        }
        match current.inode_record_of(InodeNumber(target)) {
            Some(_) => assert_eq!(read_back(&image, target).expect("读删的那个"), target_content, "{what}：状态 {index} 意图没落、{target} 整份在"),
            None => assert!(current.file_other_than_the_first(InodeNumber(target)).is_none() && (target != FIRST || current.data_pointers.is_empty()), "{what}：状态 {index} {target} 删完了却还有单元"),
        }
        let found = violations(&image);
        assert!(found.is_empty(), "{what}：状态 {index} 续做之后 checker 红 {found:?}");
    });
    eprintln!("{what}：判了 {judged} 个状态（段边界加短段全展开）");
    judged
}

/// W4 删一半崩溃：S = {1, 150}（上段层级 1），按批大小 1 删 150 的四次发布（意图、留 2、留 1、最后一批降层）每一个崩溃状态可写挂载续做到底。
#[test]
fn every_crash_state_of_deleting_another_file_that_lowers_the_upper_segment_is_continued() {
    let mut pool = MemPool::build();
    pool.create_inodes(149);
    let content_150 = units(3, 0x150c);
    pool.write(150, &content_150);
    let start = pool.stream.retained_operations().len();
    pool.unlink(150, batches_of(1), "删 150");
    judge_every_crash_state(&pool, start, "删 150（降层）", 150, &content_150, &[(FIRST, file_content())], 20_000);
}

/// W4 删一半崩溃：S = {1, 150}，按批大小 1 删第一个文件（三个单元）到底，最后一批去掉第 0 片叶；每一个崩溃状态可写挂载续做到底，150 读回不变。
#[test]
fn every_crash_state_of_deleting_the_first_file_beside_another_file_is_continued() {
    let mut pool = MemPool::build();
    let content_first = units(3, 0x1c);
    pool.write(FIRST, &content_first);
    pool.create_inodes(149);
    let content_150 = units(2, 0x150d);
    pool.write(150, &content_150);
    let start = pool.stream.retained_operations().len();
    pool.unlink(FIRST, batches_of(1), "删 1");
    judge_every_crash_state(&pool, start, "删 1（150 留着）", FIRST, &content_first, &[(150, content_150.clone())], 20_000);
}

/// W5 β「只在 inode 1 的记录不在时判」那一句的取样点：删 1（d1）→ 回退到删 1 之前（1 活着，墓碑随现行那一版留着）→ 建 3（b3 > d1）→ 写 3。
/// 1 活着，β 不判，照写。把那一句拿掉（墓碑在就判）这里拒。
#[test]
fn a_write_after_rolling_back_past_the_deletion_of_the_first_file_is_not_judged_by_beta() {
    let mut pool = MemPool::build();
    pool.create_inodes(1);
    let before = pool.output.clone();
    pool.unlink(FIRST, UnlinkBatchSize::OneExtentLowerLeaf, "删 1");
    pool.roll_back_to(&before).expect("回退到删 1 之前");
    assert!(pool.output.inode_record.is_some(), "回退之后 1 活着");
    pool.create_inodes(1);
    let birth = pool.output.inode_record_of(InodeNumber(3)).expect("3 在").object_birth;
    eprintln!("3 的出生 {birth:?}，墓碑里 1 的死亡 txg 在");
    pool.try_write(3, &units(2, 0x3c)).expect("1 活着时写 3 照写");
    assert_checker_green(&pool.image(), "写 3 之后");
}

/// 这条模型自己的一版：活着的 inode 与内容（没写过数据的是空）、出生代、这条时间线上 inode 1 的死亡 txg。
#[derive(Clone, Debug)]
struct OwnModel {
    alive: std::collections::BTreeMap<u64, (Vec<u8>, u64)>,
    death_of_the_first_file_on_this_line: Option<u64>,
}

/// W1–W4 交错历史（攻方自己的小模型，不用实现员的理想模型）：建 inode、按号写（号落在上段几片叶上）、删（批大小 1）、写第一个文件、
/// 回退到更早的一版、可写挂载一次。每次发布之后池级 checker 全绿；每一步之后活着的每个 inode 只读挂载读回与模型相同；
/// 按号写的结局与这条时间线上的 β 比（拒 ⇔ 第一个文件这条时间线上删完了而目标生于那之后）。交回（步数、β 对不上的次数）。
fn interleaved_history(seed: u64, steps: usize) -> (usize, Vec<String>) {
    let mut pool = MemPool::build();
    let mut model = OwnModel {
        alive: std::collections::BTreeMap::from([(FIRST, (file_content(), 0))]),
        death_of_the_first_file_on_this_line: None,
    };
    let mut saved: Vec<(TransactionOutput, OwnModel)> = vec![(pool.output.clone(), model.clone())];
    let mut state = seed | 1;
    let mut draw = |bound: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % bound
    };
    let mut beta_disagreements = Vec::new();
    let mut done = 0;
    for step in 0..steps {
        let choice = if model.alive.is_empty() { draw(15) } else { draw(100) };
        let what = format!("种子 {seed} 第 {step} 步");
        let trace = std::env::var("ATTACK_TRACE").is_ok();
        if choice < 15 {
            let count = 1 + draw(3) * draw(80);
            pool.create_inodes(count);
            let txg = pool.output.root.checkpoint_txg.0;
            if trace { eprintln!("{what}：建 {count} 个 inode，txg {txg}"); }
            // 新号：树里出生代是这次（最后一次建）的那几条；一次建多于 200 个分几次发布，前几次的按各自的出生代收。
            for record in pool.output.inode_leaf_containers.iter().flat_map(|container| container.contents.records.iter()) {
                if record.inode != FIRST && !model.alive.contains_key(&record.inode) && record.object_birth.0 + 2 >= txg {
                    model.alive.insert(record.inode, (Vec::new(), record.object_birth.0));
                }
            }
        } else if choice < 50 {
            let candidates: Vec<u64> = model.alive.keys().copied().collect();
            let inode = candidates[usize::try_from(draw(u64::try_from(candidates.len()).expect("个数"))).expect("下标")];
            let content = units(1 + usize::try_from(draw(4)).expect("小"), seed ^ u64::try_from(step).expect("步"));
            let born = model.alive[&inode].1;
            let beta_refuses = inode != FIRST && model.alive.get(&FIRST).is_none()
                && model.death_of_the_first_file_on_this_line.is_some_and(|death| born > death);
            if trace { eprintln!("{what}：写 {inode}（生于 {born}），这条时间线上 1 死于 {:?}，现行 txg {}", model.death_of_the_first_file_on_this_line, pool.output.root.checkpoint_txg.0); }
            match pool.try_write(inode, &content) {
                Ok(()) => {
                    if beta_refuses {
                        beta_disagreements.push(format!("{what}：写 {inode}（生于 {born}，这条时间线上 1 死于 {:?}）做成了", model.death_of_the_first_file_on_this_line));
                    }
                    model.alive.get_mut(&inode).expect("活着").0 = content;
                    assert_checker_green(&pool.image(), &format!("{what} 写 {inode} 之后"));
                }
                Err(refusal) => {
                    if !beta_refuses {
                        beta_disagreements.push(format!("{what}：写 {inode} 被拒 {refusal:?}，这条时间线上 β 不拒"));
                    }
                }
            }
        } else if choice < 70 {
            let candidates: Vec<u64> = model.alive.keys().copied().collect();
            if candidates.len() <= 1 && draw(2) == 0 {
                continue;
            }
            let inode = candidates[usize::try_from(draw(u64::try_from(candidates.len()).expect("个数"))).expect("下标")];
            let versions = pool.unlink(inode, batches_of(1), &what);
            if trace { eprintln!("{what}：删 {inode}，意图 txg {}", versions[0].root.checkpoint_txg.0); }
            if inode == FIRST {
                model.death_of_the_first_file_on_this_line = Some(versions[0].root.checkpoint_txg.0);
            }
            model.alive.remove(&inode);
        } else if choice < 85 {
            let index = usize::try_from(draw(u64::try_from(saved.len()).expect("个数"))).expect("下标");
            let (target, target_model) = saved[index].clone();
            let outcome = pool.roll_back_to(&target);
            if trace { eprintln!("{what}：回退到 txg {}：{}", target.root.checkpoint_txg.0, if outcome.is_ok() { "做成".to_string() } else { format!("{outcome:?}").chars().take(80).collect() }); }
            if outcome.is_ok() {
                model = target_model;
                assert_checker_green(&pool.image(), &format!("{what} 回退之后"));
            }
        } else {
            mount_a_copy(&pool.image(), batches_of(1)).unwrap_or_else(|failure| panic!("{what}：可写挂载一份拷贝：{failure}"));
        }
        let image = pool.image();
        for (inode, (content, _)) in &model.alive {
            if *inode == FIRST {
                continue;
            }
            assert_eq!(&read_back(&image, *inode).unwrap_or_else(|failure| panic!("{what}：读 {inode}：{failure}")), content, "{what}：{inode} 读回");
        }
        if let Some((content, _)) = model.alive.get(&FIRST) {
            assert_eq!(&read_back(&image, FIRST).expect("读 1"), content, "{what}：1 读回");
        }
        saved.push((pool.output.clone(), model.clone()));
        done += 1;
    }
    (done, beta_disagreements)
}

#[test]
fn interleaved_histories_against_the_attack_model() {
    let seeds: u64 = std::env::var("ATTACK_SEEDS").ok().and_then(|value| value.parse().ok()).unwrap_or(4);
    let steps: usize = std::env::var("ATTACK_STEPS").ok().and_then(|value| value.parse().ok()).unwrap_or(30);
    let first_seed: u64 = std::env::var("ATTACK_FIRST_SEED").ok().and_then(|value| value.parse().ok()).unwrap_or(1);
    let mut all = Vec::new();
    for seed in first_seed..first_seed + seeds {
        let (done, disagreements) = interleaved_history(seed * 0x9e37_79b9, steps);
        eprintln!("种子 {seed}：走了 {done} 步，β 对不上 {} 次", disagreements.len());
        all.extend(disagreements);
    }
    for line in &all {
        eprintln!("{line}");
    }
    assert!(all.is_empty(), "β 对不上 {} 次", all.len());
}

/// W3 / D8 已定项 16 第 4 条「从意图提交起只读挂载……答没有文件」对别的号：删 2 的意图之后、每一批之后只读挂载打开 2 都拒。
#[test]
fn a_read_only_mount_answers_no_file_for_another_inode_from_its_intent_on() {
    let mut pool = MemPool::build();
    pool.create_inodes(1);
    pool.write(2, &units(3, 0x2d));
    let versions = pool.unlink(2, batches_of(1), "删 2");
    let operations = pool.stream.retained_operations();
    let _ = operations;
    for (index, version) in versions.iter().enumerate() {
        let _ = version;
        let _ = index;
    }
    // 只读挂载读的是盘上最新那一版：分别在意图之后与第一批之后的镜像上读。
    let mut pool = MemPool::build();
    pool.create_inodes(1);
    pool.write(2, &units(3, 0x2d));
    let parameters = parameters();
    let previous = pool.output.clone();
    {
        let mut writer = PoolWriter::new(&parameters, &mut pool.devices);
        pool.output = publish_unlink_intent(&mut writer, &mut pool.allocator, &previous, InodeNumber(2), LATER, INSTANCE).expect("意图");
    }
    let after_intent = read_back(&pool.image(), 2);
    eprintln!("意图之后只读挂载打开 2：{:?}", after_intent.as_ref().map(Vec::len));
    {
        let previous = pool.output.clone();
        let mut writer = PoolWriter::new(&parameters, &mut pool.devices);
        pool.output = publish_unlink_step(&mut writer, &mut pool.allocator, &previous, InodeNumber(2), batches_of(1), INSTANCE).expect("第一批");
    }
    let after_batch = read_back(&pool.image(), 2);
    eprintln!("第一批之后只读挂载打开 2：{:?}", after_batch.as_ref().map(Vec::len));
    assert!(after_intent.is_err() && after_batch.is_err(), "删除中的别的号只读挂载答没有文件");
}
