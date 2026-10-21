//! m3-multi-file-unlink-r1 攻方腿的原型（只在攻方副本上跑）：两块 4 GiB 内存稀疏盘上 mkfs、取号、暖机、第一个文件，
//! 之后经会话或直接经 transaction 的入口走多文件的写与删。三种副本读法由环境变量 OPUS_VARIANT 选：
//! 不设 = 那棵树今天的代码；lift = 只撤掉 `refuse_an_unlink_the_deletion_steps_do_not_cover`；
//! lift_count = 再让 `next_unlink_step_after` 对 inode N 取它自己的剩余单元数（D8 已定项 16 第 1、2 条原样套到 N）。
//! 每条用例只打印 `OPUS|` 开头的观测行，不断言结论。
#[path = "common_admission/mod.rs"]
mod common_admission;

use std::panic::{catch_unwind, AssertUnwindSafe};

use common_admission::{content_of, plain_device, PoolUnderTest};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{FileOffsetInBytes, InodeNumber};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::UserChange;
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

type Pool = PoolUnderTest<SparseBlockDevice>;

fn start() -> Pool {
    PoolUnderTest::start_after_the_first_file(HistoryDeviceWidth::FourGibibytes, plain_device)
}

fn member(version: &PoolVersion) -> &'static str {
    match version {
        PoolVersion::WithoutFile(_) => "WithoutFile",
        PoolVersion::WithFile(_) => "WithFile",
        PoolVersion::WithTreesAndNoFile(_) => "WithTreesAndNoFile",
    }
}

fn trees(version: &PoolVersion) -> &TransactionOutput {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => output,
        PoolVersion::WithoutFile(_) => panic!("这几段历史里现行那一版都有树"),
    }
}

/// 一版的摘要：inode 记录（号, nlink, size）、第一个文件的单元数、别的文件（号, 单元数）、上段节点（层, 序号）、deleted_inodes。
fn summary(version: &TransactionOutput) -> String {
    let records: Vec<(u64, u32, u64)> = version
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .map(|record| (record.inode, u32::from(record.nlink), record.size))
        .collect();
    let others: Vec<(u64, usize)> = version
        .files_other_than_the_first
        .iter()
        .map(|file| (file.inode.0, file.data_pointers.len()))
        .collect();
    let upper: Vec<(u8, u64)> = version
        .extent_tree
        .upper_nodes
        .iter()
        .map(|(position, _)| (position.level, position.index))
        .collect();
    let deleted: Vec<u64> = version.deleted_inodes.iter().map(|inode| inode.0).collect();
    let records_shown = if records.len() > 6 {
        format!("{:?} 之后还有 {} 条（nlink 都是 1、size 都是 0：{}）", &records[..3], records.len() - 3,
            records[3..].iter().all(|(_, nlink, size)| *nlink == 1 && *size == 0))
    } else {
        format!("{records:?}")
    };
    format!(
        "records(inode,nlink,size)={records_shown} first_file_units={} others(inode,units)={others:?} upper(level,index)={upper:?} deleted_inodes={deleted:?} txg={}",
        version.data_pointers.len(),
        version.root.checkpoint_txg.0
    )
}

fn violations(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
            _ => None,
        })
        .collect()
}

fn verdict_of(image: &MemoryPool, wanted: &str) -> String {
    check_pool_image(image)
        .into_iter()
        .filter(|(invariant, _)| *invariant == wanted)
        .map(|(invariant, verdict)| format!("{invariant}: {verdict:?}"))
        .collect::<Vec<_>>()
        .join(" ; ")
}

fn read_back(image: &MemoryPool, inode: u64) -> Result<Vec<u8>, String> {
    let mounted = mount_read_only(image).map_err(|failure| format!("只读挂载：{failure:?}"))?;
    let file = mounted
        .mounted
        .open_file(image, InodeNumber(inode))
        .map_err(|failure| format!("打开 inode {inode}：{failure:?}"))?;
    let size = file.inode_record().size;
    file.read_at(image, FileOffsetInBytes(0), size)
        .map(|read| read.bytes)
        .map_err(|failure| format!("读 inode {inode}：{failure:?}"))
}

fn write_inode(pool: &mut Pool, inode: u64, content: &[u8]) -> String {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    match pool.publish(
        UserChange::SequentialWriteOfAnInode {
            inode: InodeNumber(inode),
            content,
            write_time_seconds,
        },
        None,
    ) {
        Ok(_) => "Ok".to_string(),
        Err(refusal) => format!("Err({refusal:?})").chars().take(400).collect(),
    }
}

fn unlink(pool: &mut Pool, inode: u64) -> String {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        pool.publish(
            UserChange::Unlink {
                inode: InodeNumber(inode),
                write_time_seconds,
            },
            None,
        )
    }));
    match outcome {
        Ok(Ok(_)) => "Ok".to_string(),
        Ok(Err(refusal)) => format!("Err({refusal:?})").chars().take(400).collect(),
        Err(panic) => format!(
            "PANIC({})",
            panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_string()))
                .unwrap_or_default()
        ),
    }
}

fn current(pool: &Pool) -> &PoolVersion {
    &pool.session().current
}

fn report(tag: &str, pool: &Pool) {
    let version = current(pool);
    println!("OPUS|{tag}|member={}|{}", member(version), summary(trees(version)));
}

fn remount_and_report(tag: &str, pool: &mut Pool) {
    match pool.crash_and_mount_writable() {
        Ok(_) => report(&format!("{tag}-after-remount"), pool),
        Err(error) => println!("OPUS|{tag}-after-remount|mount_writable=Err({error:?})"),
    }
}

/// U3 / K-U5：删空之后建 inode 2 再按 inode 号写它（今天的代码，不改）。
#[test]
fn today_writes_data_to_an_inode_created_after_the_pool_was_emptied() {
    let mut pool = start();
    println!("OPUS|A1-unlink-1|{}", unlink(&mut pool, 1));
    report("A1-after-unlink-1", &pool);
    let created = pool.create_one_inode().map(|_| "Ok").unwrap_or("Err");
    println!("OPUS|A1-new-inode|{created}");
    report("A1-after-new-inode", &pool);
    let new_inode = trees(current(&pool))
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .map(|record| record.inode)
        .max()
        .expect("建了一个");
    let content = content_of(3 * data_unit_payload_capacity() - 7, 11);
    println!("OPUS|A1-write-inode-{new_inode}|{}", write_inode(&mut pool, new_inode, &content));
    report("A1-after-write", &pool);
    let image = pool.image();
    println!("OPUS|A1-checker-violations|{:?}", violations(&image));
    println!(
        "OPUS|A1-read-back-equal|{:?}",
        read_back(&image, new_inode).map(|bytes| bytes == content)
    );
    println!("OPUS|A1-unlink-{new_inode}|{}", unlink(&mut pool, new_inode));
    remount_and_report("A1", &mut pool);
}

/// U3：第一个文件之外先有一个 size 0 的 inode，再删第一个文件，再按 inode 号写那个 inode（今天的代码）。
#[test]
fn today_writes_data_to_an_inode_that_outlived_the_first_file() {
    let mut pool = start();
    let created = pool.create_one_inode().map(|_| "Ok").unwrap_or("Err");
    println!("OPUS|A2-new-inode|{created}");
    println!("OPUS|A2-unlink-1|{}", unlink(&mut pool, 1));
    report("A2-after-unlink-1", &pool);
    let content = content_of(3 * data_unit_payload_capacity() - 7, 12);
    println!("OPUS|A2-write-inode-2|{}", write_inode(&mut pool, 2, &content));
    report("A2-after-write", &pool);
    let image = pool.image();
    println!("OPUS|A2-checker-violations|{:?}", violations(&image));
    println!("OPUS|A2-read-back-equal|{:?}", read_back(&image, 2).map(|bytes| bytes == content));
    println!("OPUS|A2-unlink-2|{}", unlink(&mut pool, 2));
}

/// U1：只撤掉拒绝（OPUS_VARIANT=lift），删一个带 3 个单元的 inode 2。
#[test]
fn lift_only_unlinks_inode_two_holding_three_data_units() {
    if std::env::var("OPUS_VARIANT").as_deref() != Ok("lift") {
        return;
    }
    let mut pool = start();
    pool.create_one_inode().expect("建 inode 2");
    let content = content_of(3 * data_unit_payload_capacity() - 7, 13);
    println!("OPUS|B-write-inode-2|{}", write_inode(&mut pool, 2, &content));
    println!("OPUS|B-unlink-2|{}", unlink(&mut pool, 2));
    report("B-after-unlink-2", &pool);
    let image = pool.image();
    println!("OPUS|B-checker-violations|{:?}", violations(&image));
    remount_and_report("B", &mut pool);
}

/// U3：只撤掉拒绝（OPUS_VARIANT=lift），inode 2 带数据（上段仍只有根兼叶）时删第一个文件；inode 150 带数据（上段长到层 1）时删第一个文件。
#[test]
fn lift_only_unlinks_the_first_file_while_another_file_holds_data_units() {
    if std::env::var("OPUS_VARIANT").as_deref() != Ok("lift") {
        return;
    }
    for (tag, inode) in [("D-leaf0", 2_u64), ("E-level1", 150_u64)] {
        let mut pool = start();
        pool.write_time_seconds += 1;
        let write_time_seconds = pool.write_time_seconds;
        pool.publish(
            UserChange::NewInodes {
                count: inode - 1,
                write_time_seconds,
            },
            None,
        )
        .expect("建 inode 2 到 inode");
        let content = content_of(2 * data_unit_payload_capacity() - 5, 14);
        println!("OPUS|{tag}-write-inode-{inode}|{}", write_inode(&mut pool, inode, &content));
        report(&format!("{tag}-before-unlink-1"), &pool);
        let outcome = unlink(&mut pool, 1);
        println!("OPUS|{tag}-unlink-1|{outcome}");
        if pool.session.is_some() {
            report(&format!("{tag}-after-unlink-1"), &pool);
        }
        let image = pool.image();
        println!("OPUS|{tag}-checker-violations|{:?}", violations(&image));
        println!(
            "OPUS|{tag}-read-back-inode-{inode}-equal|{:?}",
            read_back(&image, inode).map(|bytes| bytes == content)
        );
        let remount = catch_unwind(AssertUnwindSafe(|| remount_and_report(tag, &mut pool)));
        if remount.is_err() {
            println!("OPUS|{tag}-after-remount|PANIC（可写挂载的续做里）");
        }
    }
}

/// U1：第 1、2 条原样套到 inode N（OPUS_VARIANT=lift_count）：第一个文件 3 个单元、inode 2 三个单元、批大小压成 1，
/// 直接经 transaction 的入口发意图与第一批，看这一批截的是谁。
#[test]
fn lift_count_first_batch_of_inode_two_with_a_three_unit_first_file() {
    if std::env::var("OPUS_VARIANT").as_deref() != Ok("lift_count") {
        return;
    }
    let mut pool = start();
    let first_content = content_of(3 * data_unit_payload_capacity() - 7, 21);
    pool.sequential_write(first_content.len()).expect("第一个文件写成 3 个单元");
    let first_content = pool.content_of_the_current_version.clone();
    pool.create_one_inode().expect("建 inode 2");
    let content = content_of(3 * data_unit_payload_capacity() - 7, 22);
    println!("OPUS|C-write-inode-2|{}", write_inode(&mut pool, 2, &content));
    report("C-before-intent", &pool);
    let previous = trees(current(&pool)).clone();
    let parameters = pool.parameters.clone();
    let mut session = pool.session.take().expect("会话");
    let instance = session.instance;
    let switches = session.allocator.deletion_switches();
    let capped = UnlinkBatchSize::CappedForTests(
        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1..=144"),
    );
    let (intent, batch) = {
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        writer.set_container_tree_capacities(switches.container_tree_capacities());
        let intent = publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &previous,
            InodeNumber(2),
            pool.write_time_seconds + 1,
            instance,
        )
        .expect("意图");
        let batch = catch_unwind(AssertUnwindSafe(|| {
            publish_unlink_step(
                &mut writer,
                &mut session.allocator,
                &intent,
                InodeNumber(2),
                capped,
                instance,
            )
        }));
        (intent, batch)
    };
    println!("OPUS|C-after-intent|{}", summary(&intent));
    match batch {
        Ok(Ok(version)) => println!("OPUS|C-after-first-batch|{}", summary(&version)),
        Ok(Err(error)) => println!("OPUS|C-first-batch|Err({error:?})"),
        Err(_) => println!("OPUS|C-first-batch|PANIC"),
    }
    let image = pool.image();
    println!("OPUS|C-checker-violations|{:?}", violations(&image));
    let payload = data_unit_payload_capacity();
    match read_back(&image, 1) {
        Ok(bytes) => println!(
            "OPUS|C-read-back-first-file|len={} equal={} first-two-units-equal={} third-unit-all-zero={} third-unit-was-all-zero={}",
            bytes.len(),
            bytes == first_content,
            bytes[..2 * payload] == first_content[..2 * payload],
            bytes[2 * payload..].iter().all(|byte| *byte == 0),
            first_content[2 * payload..].iter().all(|byte| *byte == 0)
        ),
        Err(error) => println!("OPUS|C-read-back-first-file|{error}"),
    }
    let remount = catch_unwind(AssertUnwindSafe(|| pool.crash_and_mount_writable().map(|_| ())));
    match remount {
        Ok(Ok(())) => {
            report("C-after-remount", &pool);
            let image = pool.image();
            println!("OPUS|C-after-remount-violations|{:?}", violations(&image));
        }
        Ok(Err(error)) => println!("OPUS|C-after-remount|mount_writable=Err({error:?})"),
        Err(_) => println!("OPUS|C-after-remount|PANIC"),
    }
}

/// U4：删第一个文件的中途（意图 + 一批，批大小 1），inode 2 带数据：扫描重建 I-1.13 对整份镜像报什么（OPUS_VARIANT=lift）。
#[test]
fn lift_only_scan_rebuild_verdict_while_the_first_file_is_half_deleted_and_inode_two_holds_data() {
    if std::env::var("OPUS_VARIANT").as_deref() != Ok("lift") {
        return;
    }
    let mut pool = start();
    pool.sequential_write(3 * data_unit_payload_capacity() - 7).expect("第一个文件 3 个单元");
    pool.create_one_inode().expect("建 inode 2");
    let content = content_of(2 * data_unit_payload_capacity() - 5, 31);
    println!("OPUS|F-write-inode-2|{}", write_inode(&mut pool, 2, &content));
    let clean = pool.image();
    println!("OPUS|F-clean-I-1.13|{}", verdict_of(&clean, "I-1.13"));
    let previous = trees(current(&pool)).clone();
    let parameters = pool.parameters.clone();
    let mut session = pool.session.take().expect("会话");
    let instance = session.instance;
    let switches = session.allocator.deletion_switches();
    let capped = UnlinkBatchSize::CappedForTests(
        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1..=144"),
    );
    {
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        writer.set_container_tree_capacities(switches.container_tree_capacities());
        let intent = publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &previous,
            InodeNumber(1),
            pool.write_time_seconds + 1,
            instance,
        )
        .expect("意图");
        let batch = publish_unlink_step(
            &mut writer,
            &mut session.allocator,
            &intent,
            InodeNumber(1),
            capped,
            instance,
        )
        .expect("第一批");
        println!("OPUS|F-after-first-batch|{}", summary(&batch));
    }
    let half = pool.image();
    println!("OPUS|F-half-deleted-I-1.13|{}", verdict_of(&half, "I-1.13"));
    println!("OPUS|F-half-deleted-I-4.7|{}", verdict_of(&half, "I-4.7"));
    println!("OPUS|F-half-deleted-violations|{:?}", violations(&half));
}
