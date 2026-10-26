//! 攻方原型（m4-partial-write-r1，只在攻方草稿副本里）：部分写的几面镜像与读法对照。盘是内存稀疏盘，每个起点状态只建一次池、之后克隆。
#![allow(clippy::all, clippy::pedantic, reason = "攻方草稿原型，不进仓")]

mod common;

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::{InvariantVerdict, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    DataUnitIndexInFile, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::PoolReader;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_overwrite,
    publish_partial_write_of_the_first_file_attack_prototype, publish_sequential_write,
    publish_size_change, warm_up, FirstFile, PoolWriter, TransactionOutput, TransactionUnit,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{data_unit_payload, data_unit_payload_capacity, parse_data_unit};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

const UNIT_BYTES: usize = 32768;
const INSTANCE: InstanceGeneration = InstanceGeneration(1);

struct SparsePool {
    devices: Devices,
    allocator: PoolAllocator,
    output: TransactionOutput,
    next_time: u64,
    instance: InstanceGeneration,
}

fn clone_devices(devices: &Devices) -> Devices {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copy = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}

impl SparsePool {
    fn duplicate(&self) -> SparsePool {
        SparsePool {
            devices: clone_devices(&self.devices),
            allocator: self.allocator.clone(),
            output: self.output.clone(),
            next_time: self.next_time,
            instance: self.instance,
        }
    }
    fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.image.clone()))
                .collect(),
            device_size_in_bytes: IMAGE_BYTES,
        }
    }
    fn time(&mut self) -> u64 {
        self.next_time += 1;
        self.next_time
    }
}

fn payload() -> usize {
    data_unit_payload_capacity()
}

/// mkfs → 取号 → 暖机 → 新池新建文件（3000 字节），两块 4 GiB 内存稀疏盘。
fn build_sparse_pool() -> SparsePool {
    let parameters = parameters();
    let mut devices: Devices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    let content = file_content();
    let output = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut pool).expect("取号");
        let warmed = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件")
    };
    SparsePool {
        devices,
        allocator,
        output,
        next_time: FIXED_WRITE_TIME_SECONDS,
        instance: INSTANCE,
    }
}

fn content_of_units(units: usize, seed: u8) -> Vec<u8> {
    (0..units * payload() - 7)
        .map(|index| u8::try_from((index * 7 + usize::from(seed)) % 251).expect("小于 256"))
        .collect()
}

fn sequential_write(pool: &mut SparsePool, content: &[u8]) {
    let parameters = parameters();
    let time = pool.time();
    let previous = pool.output.clone();
    let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
    pool.output = publish_sequential_write(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: time,
        },
        INSTANCE,
    )
    .expect("顺序写");
}

fn overwrite_whole(pool: &mut SparsePool, content: &[u8]) {
    let parameters = parameters();
    let time = pool.time();
    let previous = pool.output.clone();
    let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
    pool.output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: time,
        },
        INSTANCE,
    )
    .expect("覆盖写");
}

fn size_change(pool: &mut SparsePool, size: u64) -> Result<(), String> {
    let parameters = parameters();
    let time = pool.time();
    let previous = pool.output.clone();
    let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
    let published = publish_size_change(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(FIRST_INODE_NUMBER),
        size,
        time,
        INSTANCE,
    )
    .map_err(|error| format!("{error:?}"))?;
    pool.output = published;
    Ok(())
}

/// 读回旧单元的两种读法（R3 的两个候选）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OldUnitReading {
    /// 取内存里上一版的字节（截断尾单元今天的做法，`payload_of_the_rewritten_tail_data_unit`）。
    MemoryBytesOfThePreviousVersion,
    /// 读盘：按两条位置条目逐份读整单元、核位置条目里的整单元 CRC-32C，第一份对得上的取它；都对不上就失败。
    DiskWithTheWholeUnitChecksum,
}

/// 一次部分写的结果：发成了没有、这次发了几次块层读。
#[derive(Debug)]
struct PartialWriteOutcome {
    result: Result<(), String>,
    device_reads: u64,
}

fn old_payload_of_the_unit(
    pool: &SparsePool,
    unit: u64,
    reading: OldUnitReading,
    device_reads: &mut u64,
) -> Result<Vec<u8>, String> {
    match reading {
        OldUnitReading::MemoryBytesOfThePreviousVersion => {
            let bytes = &pool
                .output
                .unit(TransactionUnit::Data(DataUnitIndexInFile(unit)))
                .bytes;
            let header = parse_data_unit(bytes).map_err(|error| format!("内存里这一版的字节解不开：{error:?}"))?;
            Ok(data_unit_payload(bytes, header.declared_length)
                .map_err(|error| format!("{error:?}"))?
                .to_vec())
        }
        OldUnitReading::DiskWithTheWholeUnitChecksum => {
            let pointer = pool.output.data_pointers[usize::try_from(unit).expect("序号")];
            for location in pointer.locations {
                *device_reads += 1;
                let Some(copy) =
                    pool.devices
                        .read(location.device, location.slot.to_device_offset(), UNIT_BYTES)
                else {
                    continue;
                };
                if crc32_castagnoli(&copy) != location.unit_checksum {
                    continue;
                }
                let header = parse_data_unit(&copy).map_err(|error| format!("{error:?}"))?;
                return Ok(data_unit_payload(&copy, header.declared_length)
                    .map_err(|error| format!("{error:?}"))?
                    .to_vec());
            }
            Err("读回旧单元：两份都读不回或整单元校验和对不上 ⇒ 部分写失败（D4 已定项 4）".to_string())
        }
    }
}

/// 第一个文件单元 `unit` 里从净荷偏移 `at` 起写 `bytes`（不跨单元、不改 size）：先按 `reading` 读回旧单元，改完整单元重写。
fn partial_write(
    pool: &mut SparsePool,
    unit: u64,
    at: usize,
    bytes: &[u8],
    reading: OldUnitReading,
) -> PartialWriteOutcome {
    let mut device_reads = 0;
    let old = match old_payload_of_the_unit(pool, unit, reading, &mut device_reads) {
        Ok(old) => old,
        Err(reason) => {
            return PartialWriteOutcome {
                result: Err(reason),
                device_reads,
            }
        }
    };
    let mut new_payload = old;
    new_payload[at..at + bytes.len()].copy_from_slice(bytes);
    let parameters = parameters();
    let time = pool.time();
    let previous = pool.output.clone();
    let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
    let result = publish_partial_write_of_the_first_file_attack_prototype(
        &mut writer,
        &mut pool.allocator,
        &previous,
        unit,
        new_payload,
        time,
        pool.instance,
    )
    .map(|published| {
        pool.output = published;
    })
    .map_err(|error| format!("{error:?}"));
    PartialWriteOutcome {
        result,
        device_reads,
    }
}

fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(name, _)| *name == invariant)
        .expect("在清单里")
        .1
}

fn short(verdict: &InvariantVerdict) -> String {
    let text = format!("{verdict:?}");
    text.chars().take(260).collect()
}

/// 不是成立、也不是不适用的那几条（违例与只判了一部分）。
fn reddened(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| name.to_string())
        .collect()
}

fn birth_txgs(pool: &SparsePool) -> Vec<u64> {
    pool.output
        .data_pointers
        .iter()
        .map(|pointer| pointer.head.birth_txg.0)
        .collect()
}

/// 读法 Z 写序那一条（`rebuild.rs` 的 `newest_birth_txg_below_each_data_unit`）照字面算：有没有一个单元，偏移更小的单元里有出生 txg 更大的。
fn write_order_rule_kills_some_live_unit(births: &[u64]) -> bool {
    births
        .iter()
        .enumerate()
        .any(|(index, birth)| births[..index].iter().any(|below| below > birth))
}

fn whole_file_read_by_the_mounted_reader(image: &MemoryPool) -> Result<Vec<u8>, String> {
    let mounted = mount_read_only(image).map_err(|error| format!("{error:?}"))?;
    let file = mounted
        .mounted
        .open_file(image, InodeNumber(FIRST_INODE_NUMBER))
        .map_err(|error| format!("{error:?}"))?;
    let size = mounted
        .mounted
        .inode_records()
        .iter()
        .find(|record| record.inode == FIRST_INODE_NUMBER)
        .expect("记录")
        .size;
    file.read_at(image, FileOffsetInBytes(0), size)
        .map(|read| read.bytes)
        .map_err(|error| format!("{error:?}"))
}

fn three_unit_pool() -> SparsePool {
    let mut pool = build_sparse_pool();
    sequential_write(&mut pool, &content_of_units(3, 1));
    pool
}

/// R4：三个单元的文件，对单元 k 做一次 4 KiB 的部分写（只重写单元 k），再只走扫描重建。
#[test]
fn r4_mirror_a_partial_write_on_one_unit_then_the_scan_rebuild() {
    let base = three_unit_pool();
    println!("观测 R4 起点：三个单元的出生 txg {:?}，I-1.13 {}", birth_txgs(&base), short(&verdict_of(&base.image(), SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX)));
    let mut hits = 0;
    for unit in 0..3u64 {
        let mut pool = base.duplicate();
        let mut expected = content_of_units(3, 1);
        let patch = vec![0xA5u8; 4096];
        let at = 100usize;
        let outcome = partial_write(&mut pool, unit, at, &patch, OldUnitReading::MemoryBytesOfThePreviousVersion);
        outcome.result.expect("部分写发成");
        let start = usize::try_from(unit).expect("序号") * payload() + at;
        expected[start..start + patch.len()].copy_from_slice(&patch);
        let image = pool.image();
        let read_back = whole_file_read_by_the_mounted_reader(&image);
        let verdict = verdict_of(&image, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX);
        let violated = matches!(verdict, InvariantVerdict::Violated(_));
        hits += usize::from(violated);
        println!(
            "观测 R4 部分写单元 {unit}：出生 txg {:?}；挂载态读者读回整文件与期望相同 = {}；写序一条照字面判死某个活单元 = {}；I-1.13 {}；违例的不变量 {:?}",
            birth_txgs(&pool),
            read_back.as_deref() == Ok(expected.as_slice()),
            write_order_rule_kills_some_live_unit(&birth_txgs(&pool)),
            short(&verdict),
            reddened(&image)
        );
    }
    println!("观测 R4 汇总：三个单元各部分写一次，I-1.13 违例 {hits} 次");
    assert!(hits >= 1, "镜像造出来了");
}

/// R4 把用户动作放开扫：起点只固定「三个单元的文件」，之后由用户定的动作序列（对某个单元部分写、整份覆盖写、变长一个净荷）
/// 取长度 1 到 3 的全部组合；每段历史判 I-1.13，与「写序一条照字面判死某个活单元」比。
#[test]
fn r4_user_steps_opened_over_short_histories() {
    #[derive(Clone, Copy, Debug)]
    enum Step {
        PartialWrite(u64),
        OverwriteWhole,
        ExtendByOnePayload,
    }
    let steps = [
        Step::PartialWrite(0),
        Step::PartialWrite(1),
        Step::PartialWrite(2),
        Step::OverwriteWhole,
        Step::ExtendByOnePayload,
    ];
    let base = three_unit_pool();
    let started = std::time::Instant::now();
    let mut histories: Vec<Vec<Step>> = Vec::new();
    for length in 1..=3usize {
        let mut indexes = vec![0usize; length];
        loop {
            histories.push(indexes.iter().map(|index| steps[*index]).collect());
            let mut position = length;
            loop {
                if position == 0 {
                    break;
                }
                position -= 1;
                indexes[position] += 1;
                if indexes[position] < steps.len() {
                    break;
                }
                indexes[position] = 0;
            }
            if indexes.iter().all(|index| *index == 0) {
                break;
            }
        }
    }
    let limit: usize = std::env::var("ATTACK_HISTORY_LIMIT").ok().and_then(|text| text.parse().ok()).unwrap_or(histories.len());
    let mut violated = 0;
    let mut agree = 0;
    let mut judged = 0;
    for history in histories.iter().take(limit) {
        let mut pool = base.duplicate();
        let mut content = content_of_units(3, 1);
        for step in history {
            match step {
                Step::PartialWrite(unit) => {
                    let unit_count = pool.output.data_pointers.len() as u64;
                    let unit = (*unit).min(unit_count - 1);
                    partial_write(&mut pool, unit, 100, &[0x5Au8; 64], OldUnitReading::MemoryBytesOfThePreviousVersion)
                        .result
                        .expect("部分写发成");
                    let start = usize::try_from(unit).expect("序号") * payload() + 100;
                    if start + 64 <= content.len() {
                        content[start..start + 64].copy_from_slice(&[0x5Au8; 64]);
                    }
                }
                Step::OverwriteWhole => {
                    content = content_of_units(3, 9);
                    sequential_write(&mut pool, &content);
                }
                Step::ExtendByOnePayload => {
                    let size = content.len() + payload();
                    size_change(&mut pool, size as u64).expect("变长");
                    content.resize(size, 0);
                }
            }
        }
        let image = pool.image();
        let verdict = verdict_of(&image, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX);
        let is_violated = matches!(verdict, InvariantVerdict::Violated(_));
        let predicted = write_order_rule_kills_some_live_unit(&birth_txgs(&pool));
        violated += usize::from(is_violated);
        agree += usize::from(is_violated == predicted);
        judged += 1;
        println!(
            "观测 R4 扫：历史 {history:?} 出生 txg {:?} I-1.13 违例 = {is_violated} 照字面预测 = {predicted} 读者读回相同 = {}",
            birth_txgs(&pool),
            whole_file_read_by_the_mounted_reader(&image).as_deref() == Ok(content.as_slice())
        );
    }
    println!(
        "观测 R4 扫汇总：历史 {judged} 段，I-1.13 违例 {violated} 段，判定与照字面预测一致 {agree} 段，挂钟 {:?}",
        started.elapsed()
    );
}

/// R4 候选「写到 o 时把 o 之后的单元一起重写」的仿真（分几次发布，不是同一次发布）：部分写单元 0 之后，单元 1、2 原样各重写一次。
/// 另看「有洞的盘报不适用」那个候选的触发条件在这面镜像上成不成立：上段叶标签与单元数对不对得上 size。
#[test]
fn r4_candidates_on_the_mirror() {
    let base = three_unit_pool();
    let mut pool = base.duplicate();
    partial_write(&mut pool, 0, 100, &[0xA5u8; 4096], OldUnitReading::MemoryBytesOfThePreviousVersion).result.expect("部分写");
    let size = pool.output.inode_record.expect("记录").size;
    let units_the_size_needs = size.div_ceil(payload() as u64).max(1);
    println!(
        "观测 R4 候选：镜像上第一个文件 size {size}，存在的单元 {}，size 要的单元 {units_the_size_needs}，有洞 = {}",
        pool.output.data_pointers.len(),
        (pool.output.data_pointers.len() as u64) < units_the_size_needs
    );
    let mut data_units_written = 1;
    for unit in 1..3u64 {
        let length = if unit == 2 { payload() - 7 } else { payload() };
        let mut reads = 0;
        let old = old_payload_of_the_unit(&pool, unit, OldUnitReading::MemoryBytesOfThePreviousVersion, &mut reads).expect("旧净荷");
        assert_eq!(old.len(), length);
        partial_write(&mut pool, unit, 0, &old[..16], OldUnitReading::MemoryBytesOfThePreviousVersion).result.expect("原样重写");
        data_units_written += 1;
    }
    println!(
        "观测 R4 候选：部分写单元 0 再把单元 1、2 原样重写之后出生 txg {:?}，I-1.13 {}，这次部分写一共写了 {data_units_written} 个数据单元",
        birth_txgs(&pool),
        short(&verdict_of(&pool.image(), SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX))
    );
}

/// R3：旧单元两份都改坏（每份净荷里翻一个字节），再对它部分写。两种读法各跑一次；不改坏时同一个写各跑一次。
#[test]
fn r3_injected_corruption_of_the_old_unit_under_the_two_readings() {
    let base = three_unit_pool();
    for corrupt in [false, true] {
        for reading in [OldUnitReading::MemoryBytesOfThePreviousVersion, OldUnitReading::DiskWithTheWholeUnitChecksum] {
            let mut pool = base.duplicate();
            if corrupt {
                for location in pool.output.data_pointers[1].locations {
                    let device = &mut pool.devices.iter_mut().find(|(identity, _)| *identity == location.device).expect("盘").1;
                    let offset = location.slot.to_device_offset();
                    let mut sector = device.image.read(singlefs_core::address::DeviceOffsetInBytes(offset.0 + 4096), 512);
                    sector[17] ^= 0xFF;
                    device.image.write(singlefs_core::address::DeviceOffsetInBytes(offset.0 + 4096), &sector);
                }
            }
            let outcome = partial_write(&mut pool, 1, 100, &[0xC3u8; 4096], reading);
            let after = pool.image();
            println!(
                "观测 R3 改坏旧单元 = {corrupt} 读法 {reading:?}：部分写结果 {:?}，块层读 {} 次，之后读者读整文件 {}，违例的不变量 {:?}",
                outcome.result,
                outcome.device_reads,
                match whole_file_read_by_the_mounted_reader(&after) { Ok(_) => "读得出".to_string(), Err(error) => error.chars().take(120).collect() },
                reddened(&after)
            );
        }
    }
}

/// R3：今天的截断尾单元那条路（内存字节）在旧尾单元两份都改坏时照样发成；同一个截断在重开可写挂载之后（内存里没有字节）被拒。
#[test]
fn r3_the_truncation_tail_path_today() {
    let base = three_unit_pool();
    let target = 2 * payload() as u64 + 100;
    {
        let mut pool = base.duplicate();
        for location in pool.output.data_pointers[2].locations {
            let device = &mut pool.devices.iter_mut().find(|(identity, _)| *identity == location.device).expect("盘").1;
            let offset = location.slot.to_device_offset();
            let mut sector = device.image.read(singlefs_core::address::DeviceOffsetInBytes(offset.0 + 512), 512);
            sector[3] ^= 0xFF;
            device.image.write(singlefs_core::address::DeviceOffsetInBytes(offset.0 + 512), &sector);
        }
        let result = size_change(&mut pool, target);
        println!("观测 R3 截断尾单元：旧尾单元两份都改坏，同一进程里截到 {target}：{result:?}；违例的不变量 {:?}", reddened(&pool.image()));
    }
    {
        let mut pool = base.duplicate();
        let result = size_change(&mut pool, target);
        println!("观测 R3 截断尾单元：盘完好，同一进程里截到 {target}：{result:?}");
    }
    {
        let mut devices = clone_devices(&base.devices);
        let parameters = parameters();
        let mounted = singlefs_core::mount::mount_writable(&parameters, &mut devices).expect("可写挂载");
        let current = mounted.current.file_version().expect("带文件").clone();
        let empty_bytes = current.unit(TransactionUnit::Data(DataUnitIndexInFile(2))).bytes.is_empty();
        let mut allocator = mounted.allocator.clone();
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let result = publish_size_change(&mut writer, &mut allocator, &current, InodeNumber(FIRST_INODE_NUMBER), target, FIXED_WRITE_TIME_SECONDS + 999, mounted.output.instance);
        println!(
            "观测 R3 截断尾单元：盘完好，重开可写挂载之后（现行版本里单元 2 的字节为空 = {empty_bytes}）截到 {target}：{:?}",
            result.map(|_| "发成").map_err(|error| format!("{error:?}"))
        );
    }
}

/// R2：三种「空文件」各有几个数据单元，挂载态读者对它们 `read_at(0, 0)` 与 `read_at(0, size)` 发几次块层读；
/// 再看 R2/R5 的两格：写跨文件末尾时按 `read_at` 读被写区间、写进尾部洞里时 `read_at` 发不发读。
#[test]
fn r2_forms_of_an_empty_file_and_read_at_over_the_cells() {
    let reader_view = |image: &MemoryPool, inode: u64, offset: u64, length: u64| -> String {
        let mounted = mount_read_only(image).expect("只读挂载");
        let file = match mounted.mounted.open_file(image, InodeNumber(inode)) {
            Ok(file) => file,
            Err(error) => return format!("打开失败 {error:?}"),
        };
        let units = file.data_unit_count();
        match file.read_at(image, FileOffsetInBytes(offset), length) {
            Ok(read) => format!(
                "单元 {units} 个；read_at({offset}, {length}) 成功，{} 字节、全零 = {}、块层读 {} 次",
                read.bytes.len(),
                read.bytes.iter().all(|byte| *byte == 0),
                read.observation.device_reads_issued
            ),
            Err(error) => format!("单元 {units} 个；read_at({offset}, {length}) 失败 {error:?}"),
        }
    };
    // 形态一：publish_new_inodes 建的新 inode（size 0）。
    let mut pool = build_sparse_pool();
    {
        let parameters = parameters();
        let time = pool.time();
        let previous = pool.output.clone();
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        pool.output = publish_new_inodes(&mut writer, &mut pool.allocator, &previous, 1, time, INSTANCE).expect("建 inode");
    }
    let new_inode = pool
        .output
        .inode_leaf_container_contents()
        .iter()
        .flat_map(|container| container.records.iter())
        .map(|record| record.inode)
        .max()
        .expect("有号");
    println!("观测 R2 空文件形态一（新建 inode {new_inode}，size 0）：{}", reader_view(&pool.image(), new_inode, 0, 0));
    // 形态二：第一个文件截到 0（判决 K3：截到 0 留一个声明长度 0 的单元）。
    let mut truncated = build_sparse_pool();
    size_change(&mut truncated, 0).expect("截到 0 的意图");
    let mut steps = 0;
    while singlefs_core::logged_operations::live_truncation_intent_of_the_entries(&truncated.output.logged_operations).is_some() && steps < 8 {
        let parameters = parameters();
        let previous = truncated.output.clone();
        let mut writer = PoolWriter::new(&parameters, truncated.devices.as_mut_slice());
        truncated.output = singlefs_core::transaction::publish_truncation_step(
            &mut writer,
            &mut truncated.allocator,
            &previous,
            singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
            INSTANCE,
        )
        .expect("截断下一步");
        steps += 1;
    }
    println!(
        "观测 R2 空文件形态二（第一个文件截到 0，之后走了 {steps} 步）：存在的单元 {}，单元 0 声明长度 {:?}；{}",
        truncated.output.data_pointers.len(),
        truncated.output.data_pointers.first().map(|_| parse_data_unit(&truncated.output.unit(TransactionUnit::Data(DataUnitIndexInFile(0))).bytes).map(|header| header.declared_length)),
        reader_view(&truncated.image(), FIRST_INODE_NUMBER, 0, 0)
    );
    // 形态三：第一个文件覆盖写成 0 字节。
    let mut emptied = build_sparse_pool();
    let emptied_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| overwrite_whole(&mut emptied, &[])));
    println!(
        "观测 R2 空文件形态三（第一个文件整份覆盖写成 0 字节）：发成 = {}；存在的单元 {}；{}",
        emptied_result.is_ok(),
        emptied.output.data_pointers.len(),
        reader_view(&emptied.image(), FIRST_INODE_NUMBER, 0, 0)
    );
    // R2/R5：3000 字节的文件，写 [2000, 6096) 跨过文件末尾时按 read_at 读被写那一段。
    let base = build_sparse_pool();
    println!("观测 R5 跨末尾：size 3000 的文件 {}", reader_view(&base.image(), FIRST_INODE_NUMBER, 2000, 4096));
    println!("观测 R5 跨末尾：size 3000 的文件只读 size 之内那一段 {}", reader_view(&base.image(), FIRST_INODE_NUMBER, 2000, 1000));
    // R2/R5：变长到三个净荷（之后是洞，标签 4），在洞里 read_at。
    let mut extended = build_sparse_pool();
    size_change(&mut extended, 3 * payload() as u64).expect("变长");
    println!(
        "观测 R2 洞：变长到三个净荷之后存在的单元 {}，{}",
        extended.output.data_pointers.len(),
        reader_view(&extended.image(), FIRST_INODE_NUMBER, payload() as u64 + 100, 4096)
    );
}

fn corrupt_both_copies_of_the_data_unit(pool: &mut SparsePool, unit: usize) {
    for location in pool.output.data_pointers[unit].locations {
        let device = &mut pool.devices.iter_mut().find(|(identity, _)| *identity == location.device).expect("盘").1;
        let offset = location.slot.to_device_offset();
        let at = singlefs_core::address::DeviceOffsetInBytes(offset.0 + 4096);
        let mut sector = device.image.read(at, 512);
        sector[17] ^= 0xFF;
        device.image.write(at, &sector);
    }
}

fn remounted(pool: &SparsePool) -> SparsePool {
    let mut devices = clone_devices(&pool.devices);
    let mounted = singlefs_core::mount::mount_writable(&parameters(), &mut devices).expect("可写挂载");
    SparsePool {
        output: mounted.current.file_version().expect("带文件").clone(),
        allocator: mounted.allocator.clone(),
        instance: mounted.output.instance,
        devices,
        next_time: pool.next_time + 100,
    }
}

/// R3：「内存里上一版的字节」从哪来——可写挂载把每个数据单元读回、核过放进内存。改坏落在挂载之前与挂载之后，内存字节那一读法各怎样。
#[test]
fn r3_memory_bytes_come_from_the_mount_and_the_order_of_the_fault_decides() {
    let base = three_unit_pool();
    // 改坏在挂载之前。
    let mut before = base.duplicate();
    corrupt_both_copies_of_the_data_unit(&mut before, 1);
    let mut mounted_after_the_fault = remounted(&before);
    let empty = mounted_after_the_fault.output.unit(TransactionUnit::Data(DataUnitIndexInFile(1))).bytes.is_empty();
    for reading in [OldUnitReading::MemoryBytesOfThePreviousVersion, OldUnitReading::DiskWithTheWholeUnitChecksum] {
        let mut pool = mounted_after_the_fault.duplicate();
        let outcome = partial_write(&mut pool, 1, 100, &[0xC3u8; 4096], reading);
        println!("观测 R3 改坏在挂载之前（挂载后内存里单元 1 的字节为空 = {empty}）读法 {reading:?}：{:?}，块层读 {} 次", outcome.result.map_err(|e| e.chars().take(140).collect::<String>()), outcome.device_reads);
    }
    let _ = &mut mounted_after_the_fault;
    // 改坏在挂载之后（同一个会话里）。
    let healthy_mount = remounted(&base);
    for reading in [OldUnitReading::MemoryBytesOfThePreviousVersion, OldUnitReading::DiskWithTheWholeUnitChecksum] {
        let mut pool = healthy_mount.duplicate();
        corrupt_both_copies_of_the_data_unit(&mut pool, 1);
        let outcome = partial_write(&mut pool, 1, 100, &[0xC3u8; 4096], reading);
        println!("观测 R3 改坏在挂载之后 读法 {reading:?}：{:?}，块层读 {} 次", outcome.result.map_err(|e| e.chars().take(140).collect::<String>()), outcome.device_reads);
    }
}

/// R2：「首次写读那个将要写的新物理位置的残留」——那个位置上此刻是什么。三个单元的文件截到 100（单元 1、2 放掉），变长回三个净荷（洞），
/// 之后反复对单元 0 部分写：每次记新分配给单元 0 的落点在这次写之前的内容。
#[test]
fn r2_residue_at_the_newly_allocated_slot() {
    let mut pool = three_unit_pool();
    let released: Vec<_> = pool.output.data_pointers[1..].iter().map(|pointer| pointer.locations[0].slot).collect();
    size_change(&mut pool, 100).expect("截断意图");
    while singlefs_core::logged_operations::live_truncation_intent_of_the_entries(&pool.output.logged_operations).is_some() {
        let parameters = parameters();
        let previous = pool.output.clone();
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        pool.output = singlefs_core::transaction::publish_truncation_step(&mut writer, &mut pool.allocator, &previous, singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf, INSTANCE).expect("截断下一步");
    }
    size_change(&mut pool, 3 * payload() as u64).expect("变长");
    println!("观测 R2 残留：截断放掉的单元 1、2 的落点 {released:?}；变长之后存在的单元 {}", pool.output.data_pointers.len());
    let mut kinds = std::collections::BTreeMap::new();
    for attempt in 0..40 {
        let before = pool.image();
        partial_write(&mut pool, 0, 0, &[attempt as u8; 16], OldUnitReading::MemoryBytesOfThePreviousVersion).result.expect("部分写");
        let location = pool.output.data_pointers[0].locations[0];
        let residue = before.devices[&location.device].read(location.slot.to_device_offset(), UNIT_BYTES);
        let description = if residue.iter().all(|byte| *byte == 0) {
            "全零（从没写过）".to_string()
        } else {
            match parse_data_unit(&residue) {
                Ok(header) => format!(
                    "码 1 数据单元：对象 {}、对象出生代 {}、锚点偏移 {}、出生 txg {}、声明长度 {}；被截断放掉的落点 = {}",
                    header.identity.object, header.identity.object_birth.0, header.identity.anchor_offset, header.birth_txg.0, header.declared_length,
                    released.contains(&location.slot)
                ),
                Err(error) => format!("不是数据单元（{error:?}）"),
            }
        };
        if !kinds.contains_key(&description) {
            println!("观测 R2 残留：第 {attempt} 次部分写的新落点 {:?} 写之前是 {description}", location.slot);
        }
        *kinds.entry(description).or_insert(0) += 1;
    }
    println!("观测 R2 残留汇总：40 次里各类残留 {kinds:?}");
}

/// R4：几种写法各写了多少（这次发布交给设备的写，按结构种类）：部分写一个单元、整份顺序写三个单元。
#[test]
fn r4_writes_of_one_partial_write_and_of_a_whole_write() {
    let base = three_unit_pool();
    let mut partial = base.duplicate();
    partial_write(&mut partial, 0, 100, &[1u8; 4096], OldUnitReading::MemoryBytesOfThePreviousVersion).result.expect("部分写");
    println!("观测 R4 写账：部分写单元 0：合计 {:?}；分种类 {:?}", partial.output.writes.total(), partial.output.writes);
    let mut whole = base.duplicate();
    sequential_write(&mut whole, &content_of_units(3, 2));
    println!("观测 R4 写账：整份顺序写三个单元：合计 {:?}；分种类 {:?}", whole.output.writes.total(), whole.output.writes);
}

#[derive(Clone, Copy, Debug)]
enum UserStep {
    PartialWrite(u64),
    OverwriteWholeThreeUnits,
    OverwriteShorterToOneUnit,
    ExtendByOnePayload,
    TruncateIntoTheSecondUnit,
}

/// 改法「放掉单元的整份写照截断写字段、去掉写序那一条」开着时，整份写之前按这次放不放单元设截断字段。
fn whole_write_under_the_arm(pool: &mut SparsePool, content: &[u8], fix: bool) {
    if fix {
        let units_before = pool.output.data_pointers.len() as u64;
        let units_after = (content.len() as u64).div_ceil(payload() as u64).max(1);
        let next_txg = singlefs_core::transaction::checkpoint_txg_of_the_publish_after(pool.output.root.checkpoint_txg).expect("txg");
        let carried = pool.output.inode_record.expect("记录").truncation_field;
        let carry_when_nothing_is_released = std::env::var("ATTACK_FIX_CLEARS_WHEN_NOTHING_IS_RELEASED").is_err();
        singlefs_core::transaction::set_the_truncation_field_of_whole_writes_for_the_attack(
            if units_before > units_after {
                Some(singlefs_core::records::InodeRecordTruncationField::written_by_the_truncation_intent_of(next_txg, content.len() as u64))
            } else if carry_when_nothing_is_released {
                Some(carried)
            } else {
                None
            },
        );
    }
    sequential_write(pool, content);
    singlefs_core::transaction::set_the_truncation_field_of_whole_writes_for_the_attack(None);
}

fn truncate_to(pool: &mut SparsePool, size: u64) {
    size_change(pool, size).expect("截断意图");
    while singlefs_core::logged_operations::live_truncation_intent_of_the_entries(&pool.output.logged_operations).is_some() {
        let parameters = parameters();
        let previous = pool.output.clone();
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        pool.output = singlefs_core::transaction::publish_truncation_step(&mut writer, &mut pool.allocator, &previous, singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf, pool.instance).expect("截断下一步");
    }
}

/// 一段历史跑完：交回 (I-1.13 违例没有, 挂载态读者读回与期望相同没有, 出生 txg)。
fn run_history(base: &SparsePool, history: &[UserStep], fix: bool) -> (bool, bool, Vec<u64>, Vec<String>) {
    singlefs_checker::WRITE_ORDER_RULE_OFF_FOR_THE_ATTACK.store(fix, std::sync::atomic::Ordering::SeqCst);
    let mut pool = base.duplicate();
    let mut content = content_of_units(3, 1);
    for step in history {
        match step {
            UserStep::PartialWrite(unit) => {
                let unit = (*unit).min(pool.output.data_pointers.len() as u64 - 1);
                partial_write(&mut pool, unit, 0, &[0x5Au8; 16], OldUnitReading::MemoryBytesOfThePreviousVersion).result.expect("部分写");
                let start = usize::try_from(unit).expect("序号") * payload();
                content[start..start + 16].copy_from_slice(&[0x5Au8; 16]);
            }
            UserStep::OverwriteWholeThreeUnits => {
                content = content_of_units(3, 9);
                whole_write_under_the_arm(&mut pool, &content.clone(), fix);
            }
            UserStep::OverwriteShorterToOneUnit => {
                content = content_of_units(1, 5);
                whole_write_under_the_arm(&mut pool, &content.clone(), fix);
            }
            UserStep::ExtendByOnePayload => {
                let size = content.len() + payload();
                size_change(&mut pool, size as u64).expect("变长");
                content.resize(size, 0);
            }
            UserStep::TruncateIntoTheSecondUnit => {
                let size = (payload() + 100).min(content.len());
                truncate_to(&mut pool, size as u64);
                content.truncate(size);
            }
        }
    }
    let image = pool.image();
    let verdict = verdict_of(&image, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX);
    let red = reddened(&image);
    singlefs_checker::WRITE_ORDER_RULE_OFF_FOR_THE_ATTACK.store(false, std::sync::atomic::Ordering::SeqCst);
    (
        matches!(verdict, InvariantVerdict::Violated(_)),
        whole_file_read_by_the_mounted_reader(&image).as_deref() == Ok(content.as_slice()),
        birth_txgs(&pool),
        red,
    )
}

fn sweep(fix: bool) {
    let steps = [
        UserStep::PartialWrite(0),
        UserStep::PartialWrite(1),
        UserStep::PartialWrite(2),
        UserStep::OverwriteWholeThreeUnits,
        UserStep::OverwriteShorterToOneUnit,
        UserStep::ExtendByOnePayload,
        UserStep::TruncateIntoTheSecondUnit,
    ];
    let base = three_unit_pool();
    let started = std::time::Instant::now();
    let mut histories: Vec<Vec<UserStep>> = Vec::new();
    for first in steps {
        histories.push(vec![first]);
        for second in steps {
            histories.push(vec![first, second]);
            for third in steps {
                histories.push(vec![first, second, third]);
            }
        }
    }
    let limit: usize = std::env::var("ATTACK_HISTORY_LIMIT").ok().and_then(|text| text.parse().ok()).unwrap_or(histories.len());
    let (mut violated, mut misread, mut other_red, mut with_partial, mut violated_with_partial, mut violated_without_partial) = (0, 0, 0, 0, 0, 0);
    let skip: usize = std::env::var("ATTACK_HISTORY_SKIP").ok().and_then(|text| text.parse().ok()).unwrap_or(0);
    for history in histories.iter().skip(skip).take(limit) {
        let (is_violated, read_ok, births, red) = run_history(&base, history, fix);
        let has_partial = history.iter().any(|step| matches!(step, UserStep::PartialWrite(_)));
        violated += usize::from(is_violated);
        misread += usize::from(!read_ok);
        other_red += usize::from(red.iter().any(|name| name != SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX));
        with_partial += usize::from(has_partial);
        violated_with_partial += usize::from(is_violated && has_partial);
        violated_without_partial += usize::from(is_violated && !has_partial);
        if is_violated && !has_partial || red.iter().any(|name| name != SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX) || !read_ok {
            println!("观测 扫（改法开着 = {fix}）：历史 {history:?} 出生 txg {births:?} 违例 {red:?} 读者读回相同 = {read_ok}");
        }
    }
    println!(
        "观测 扫汇总（改法开着 = {fix}）：历史 {} 段（含部分写 {with_partial} 段），I-1.13 违例 {violated} 段（含部分写的 {violated_with_partial}、不含部分写的 {violated_without_partial}），I-1.13 之外有违例 {other_red} 段，读者读回不同 {misread} 段，挂钟 {:?}",
        histories.len().saturating_sub(skip).min(limit),
        started.elapsed()
    );
}

#[test]
fn sweep_today_rules() {
    sweep(false);
}

#[test]
fn sweep_with_the_candidate_fix() {
    sweep(true);
}

/// R4 判别子：变长留洞（覆盖写变短再变长）与中间部分写两段历史，造到 size 与各单元出生 txg 都相同，比扫描一侧看得到的输入。
#[test]
fn r4_hole_history_and_partial_write_history_look_alike_to_the_scan() {
    let base = three_unit_pool();
    let size = 3 * payload() - 7;
    let mut hole = base.duplicate();
    sequential_write(&mut hole, &content_of_units(1, 5));
    size_change(&mut hole, size as u64).expect("变长");
    let mut partial = base.duplicate();
    partial_write(&mut partial, 0, 0, &[1u8; 16], OldUnitReading::MemoryBytesOfThePreviousVersion).result.expect("部分写");
    partial_write(&mut partial, 0, 0, &[2u8; 16], OldUnitReading::MemoryBytesOfThePreviousVersion).result.expect("部分写");
    for (name, pool) in [("变长留洞", &hole), ("中间部分写", &partial)] {
        let image = pool.image();
        let record = pool.output.inode_record.expect("记录");
        println!(
            "观测 R4 判别子 {name}：记录 size {} 对象出生代 {} 截断字段 {:?}；遍历方向的单元数 {}；I-1.13 {}",
            record.size, record.object_birth.0, record.truncation_field, pool.output.data_pointers.len(),
            short(&verdict_of(&image, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX))
        );
        for unit in 0..3usize {
            let slot = base.output.data_pointers[unit].locations[0];
            let (location, from) = if unit < pool.output.data_pointers.len() { (pool.output.data_pointers[unit].locations[0], "这一版指着的") } else { (slot, "起点那一版的落点（遍历方向已不指）") };
            let bytes = image.devices[&location.device].read(location.slot.to_device_offset(), UNIT_BYTES);
            let header = parse_data_unit(&bytes).expect("码 1");
            println!(
                "观测 R4 判别子 {name} 单元 {unit}（{from} {:?}）：对象 {} 对象出生代 {} 锚点偏移 {} 出生 txg {} 声明长度 {} 写序 {:?}",
                location.slot, header.identity.object, header.identity.object_birth.0, header.identity.anchor_offset, header.birth_txg.0, header.declared_length, header.write_order
            );
        }
    }
}
