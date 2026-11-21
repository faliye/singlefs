use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::journal::record_offset;
use singlefs_core::transaction::{
    publish_sequential_write, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_verification::memory_pool::RetainedWrite;
use singlefs_verification::segments::StepKind;

use super::experiment_pool::{parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};

/// 顺序写比第一个文件晚几秒（多记录发布流的 B、C 与覆盖写链的多单元那两档都取这个写时刻）。
const SECONDS_FROM_THE_FIRST_FILE_TO_A_SEQUENTIAL_WRITE: u64 = 60;
/// 顺序写都在新池新建文件那个实例里发。
const INSTANCE_OF_THE_SEQUENTIAL_WRITES: InstanceGeneration = InstanceGeneration(1);
/// 内容的第 i 个字节是 (11·i + 3·seed + 7) mod 251：多记录流与删除流的崩溃枚举用例取的同一份字节。
const CONTENT_POSITION_FACTOR: usize = 11;
const CONTENT_SEED_FACTOR: usize = 3;
const CONTENT_OFFSET: usize = 7;
const CONTENT_MODULUS: usize = 251;

/// 恰好要 `data_units` 个数据单元的内容：前面的单元装满、最后一个装一半。
///
/// # Panics
/// `data_units` 是 0。
pub fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
    assert!(data_units >= 1, "内容至少占一个数据单元");
    let payload_capacity = data_unit_payload_capacity();
    let byte_count = (data_units - 1) * payload_capacity + payload_capacity / 2;
    let offset_of_the_seed = seed * CONTENT_SEED_FACTOR + CONTENT_OFFSET;
    (0..byte_count)
        .map(|position| {
            u8::try_from(
                (position * CONTENT_POSITION_FACTOR + offset_of_the_seed) % CONTENT_MODULUS,
            )
            .expect("对 251 取余小于 256")
        })
        .collect()
}

/// 一次顺序写：把现行那一版的内容换成 `content`，写出的那一版成了池的现行那一版、也交回给调用方。
///
/// # Panics
/// 镜像已经关掉；顺序写被拒。
pub fn sequential_write(pool: &mut BuiltPool, content: &[u8]) -> TransactionOutput {
    let parameters_of_the_pool = parameters();
    let previous_version = pool.output.clone();
    let open_devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters_of_the_pool, open_devices.as_mut_slice());
    let written = publish_sequential_write(
        &mut writer,
        &mut pool.allocator,
        &previous_version,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS
                + SECONDS_FROM_THE_FIRST_FILE_TO_A_SEQUENTIAL_WRITE,
        },
        INSTANCE_OF_THE_SEQUENTIAL_WRITES,
    )
    .expect("顺序写");
    pool.output = written.clone();
    written
}

/// 一次多条记录的发布在写表里的样子：每条记录（按计数器升序）两盘各一份写在写表的哪几格，它与上一版的身份，上一版的内容。
#[allow(
    dead_code,
    reason = "照抄的多记录流建法造它、这个装置不读：只为照抄的段原样编得过"
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiRecordPublish {
    pub record_writes_in_order: Vec<Vec<usize>>,
    pub version: (InstanceGeneration, CheckpointTxg),
    pub version_before: (InstanceGeneration, CheckpointTxg),
    pub content_before: Vec<u8>,
}

/// 多记录发布流里 B、C 两次多条记录的发布（照抄的建法造它）。
#[allow(
    dead_code,
    reason = "照抄的多记录流建法造它、这个装置不读：只为照抄的段原样编得过"
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiRecordPublishBoundaryInputs {
    pub publishes: Vec<MultiRecordPublish>,
}

/// 写表里写到计数器 `counter` 那一格 journal 记录的写（两盘同偏移各一份）。
fn journal_record_writes_at(writes: &[RetainedWrite], counter: u64) -> Vec<usize> {
    let offset_of_the_record = record_offset(counter, parameters().geometry.journal_ring_bytes);
    (0..writes.len())
        .filter(|write_index| {
            writes[*write_index].kind == StepKind::JournalRecord
                && writes[*write_index].offset == offset_of_the_record
        })
        .collect()
}

/// 发布 `output`（上一版是 `before`、它的内容 `content_before`）的每条记录在写表里的那几格。
///
/// # Panics
/// 有一条记录不是两盘各恰好一份写。
pub fn multi_record_publish(
    writes: &[RetainedWrite],
    output: &TransactionOutput,
    before: &TransactionOutput,
    content_before: Vec<u8>,
) -> MultiRecordPublish {
    let counters_in_order = output
        .earlier_records_of_this_publish
        .iter()
        .map(|earlier| earlier.record.counter)
        .chain(std::iter::once(output.record.counter));
    let record_writes_in_order: Vec<Vec<usize>> = counters_in_order
        .map(|counter| journal_record_writes_at(writes, counter))
        .collect();
    assert!(
        record_writes_in_order
            .iter()
            .all(|copies| copies.len() == 2),
        "每条记录两盘各一份写：{record_writes_in_order:?}"
    );
    MultiRecordPublish {
        record_writes_in_order,
        version: (output.root.instance, output.root.checkpoint_txg),
        version_before: (before.root.instance, before.root.checkpoint_txg),
        content_before,
    }
}
