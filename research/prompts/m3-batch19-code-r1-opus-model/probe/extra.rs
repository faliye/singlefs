//! MODE=qf：同一次挂载里先 unlink 第一个文件（给 T1）、再建一个 inode、再 unlink 它（给 T2 < T1），读第二次 unlink 意图那一版两块盘上（这里读发布交回的那一版）
//!   那条记录的三个时间秒应等于 T2。B19_MUT=q-unlink-mono 时写下的是 T1。
//! MODE=c2：`next_unlink_step` 在几个单元数上的第一步；再在 4 GiB 稀疏盘上把第一个文件顺序写到 UNITS 个单元、经会话 unlink，报每一次发布换下的数据单元数。
use crate::common_admission::*;
use singlefs_core::address::InodeNumber;
use singlefs_core::mounted_session::UserChange;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{next_unlink_step, UnlinkBatchSize};
use singlefs_harness::history::HistoryDeviceWidth;

pub fn qf() {
    let mut p = start_plain(HistoryDeviceWidth::FourGibibytes);
    let base = p.write_time_seconds;
    let t1 = base + 600;
    let t2 = base + 300;
    p.publish(UserChange::Unlink { inode: InodeNumber(1), write_time_seconds: t1 }, None).expect("unlink 1");
    p.publish(UserChange::NewInodes { count: 1, write_time_seconds: base + 100 }, None).expect("建 inode");
    let inode = p.session().current.version_of_trees().unwrap().inode_leaf_container_contents().iter()
        .flat_map(|c| c.records.iter().map(|r| r.inode)).max().unwrap();
    let published = p.publish(UserChange::Unlink { inode: InodeNumber(inode), write_time_seconds: t2 }, None).expect("unlink 新建的");
    let times: Vec<(u64, u64)> = published.versions_published.iter().flat_map(|v| v.inode_leaf_container_contents().into_iter()
        .flat_map(|c| c.records.into_iter()).filter(|r| r.inode == inode).map(|r| (r.nlink as u64, r.write_time_seconds)).collect::<Vec<_>>()).collect();
    println!("QF mut={} T1={t1} T2={t2} inode={inode} (nlink, 写入时间秒) 每一版={times:?} 意图那一版写的是给的 T2={}",
        std::env::var("B19_MUT").unwrap_or_default(), times.first().map_or(false, |(_, t)| *t == t2));
}

pub fn c2() {
    for n in [1u64, 144, 145, 432, 433, 576, 864, 4320] {
        println!("C2STEP mut={} n={n} first_step={:?}", std::env::var("B19_MUT").unwrap_or_default(), next_unlink_step(n, UnlinkBatchSize::OneExtentLowerLeaf));
    }
    let units = crate::env_u64("UNITS", 433) as usize;
    let mut p = start_plain(HistoryDeviceWidth::FourGibibytes);
    p.sequential_write((units - 1) * data_unit_payload_capacity() + 100).expect("顺序写");
    let before = p.current_file_version().data_pointers.len();
    let published = p.publish(UserChange::Unlink { inode: InodeNumber(1), write_time_seconds: p.write_time_seconds + 1 }, None).expect("unlink");
    let mut previous = before;
    let mut released = Vec::new();
    for v in &published.versions_published {
        let now = v.data_pointers.len();
        released.push(previous.saturating_sub(now));
        previous = now;
    }
    println!("C2RUN mut={} units={before} 每一次发布换下的数据单元={released:?} 最多={}", std::env::var("B19_MUT").unwrap_or_default(), released.iter().max().unwrap());
}
