//! 攻方原型（m3-unlink-code-r1，opus 腿）：E10 小盘准入开着，覆盖写到墙边；同一份盘面两条路：
//! 甲 先 unlink 第一个文件（意图 + 最后一批），再建一个 size 0 的 inode；乙 不删，直接建一个 inode。看两条路建 inode 的结局与准入读数。
//! 只在草稿副本上跑。

mod common_admission;

use common_admission::{plain_devices_on, start_plain, PoolUnderTest};
use singlefs_core::address::InodeNumber;
use singlefs_core::admission::admission_reading_before_a_publish;
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;

fn copy_of(pool: &PoolUnderTest<SparseBlockDevice>) -> PoolUnderTest<SparseBlockDevice> {
    let session = pool.session.as_ref().unwrap();
    PoolUnderTest {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        devices: plain_devices_on(&pool.image()),
        session: Some(MountedSession {
            allocator: session.allocator.clone(),
            current: session.current.clone(),
            instance: session.instance,
            shadow_ledger: session.shadow_ledger,
            parameters_and_device_table: session.parameters_and_device_table.clone(),
        }),
        write_time_seconds: pool.write_time_seconds,
        content_of_the_current_version: pool.content_of_the_current_version.clone(),
        content_of_each_root: pool.content_of_each_root.clone(),
    }
}

fn available_slots(pool: &PoolUnderTest<SparseBlockDevice>) -> Vec<i128> {
    let session = pool.session.as_ref().unwrap();
    admission_reading_before_a_publish(&session.allocator, session.current.version_of_trees())
        .available_on_each_device()
        .iter()
        .map(|(_, a)| a.0 / 16384)
        .collect()
}

fn outcome(r: &Result<singlefs_core::mounted_session::UserChangePublished, UserChangeRefused>) -> String {
    match r {
        Ok(p) => format!("Ok(raises={})", p.floor_raises.len()),
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(n)) => format!("NoSpace(raises={}, stop={})", n.floor_raises.len(), format!("{:?}", n.stop).chars().take(60).collect::<String>()),
        Err(e) => format!("{e:?}").chars().take(120).collect(),
    }
}

fn case(width: HistoryDeviceWidth, overwrites: usize, creates_after: usize) {
    let mut pool = start_plain(width);
    let mut done = 0;
    for _ in 0..overwrites {
        if pool.overwrite(2999).is_ok() { done += 1; } else { break; }
    }
    let before = available_slots(&pool);
    // 乙：不删，直接建 inode。
    let mut keep = copy_of(&pool);
    let keep_results: Vec<String> = (0..creates_after).map(|_| outcome(&keep.create_one_inode())).collect();
    // 甲：先 unlink，再建 inode。
    let unlinked = pool.publish(UserChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER) }, None);
    let after_unlink = available_slots(&pool);
    let delete_results: Vec<String> = (0..creates_after).map(|_| outcome(&pool.create_one_inode())).collect();
    println!(
        "E10 width={width:?} overwrites_done={done} available_before={before:?} | keep: creates={keep_results:?} | unlink={} available_after_unlink={after_unlink:?} creates={delete_results:?}",
        outcome(&unlinked)
    );
}

#[test]
fn e10_create_after_unlink_against_create_without_unlink() {
    for width in [HistoryDeviceWidth::UnitAreaOf240Slots, HistoryDeviceWidth::UnitAreaOf256Slots, HistoryDeviceWidth::UnitAreaOf384Slots] {
        for overwrites in [0usize, 3, 6, 10, 20, 40] {
            case(width, overwrites, 4);
        }
    }
}
