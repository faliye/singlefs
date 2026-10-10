//! s2 攻方原型 K7：「先右后左」在产品写法走得到的一格上分得出。经会话（内存稀疏盘）建 698 个空 inode（三片满叶 [1..=233]、[234..=466]、
//! [467..=699]），三片各删到 117 条（2..=117、234..=349、467..=582；每片都 ≥ 117、相邻和 234，一次都不并不借），再删中间那片的 350：
//! 中间片 116 条，与左、右两邻都合得进一片。打出这一次身份消失的是哪一片、幸存的几片。实现（先右后左）应是中间片吸收右片；
//! 改成先左后右的副本应是左片吸收中间片。只在攻方副本上跑。
mod common_admission_pool;

use common_admission_pool::{
    checker_violations_on, parameters_of_the_384_slot_small_devices, plain_device, PoolUnderTest,
};
use singlefs_core::address::InodeNumber;
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::mounted_session::UserChange;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::SparseBlockDevice;

type Pool = PoolUnderTest<SparseBlockDevice>;

fn unlink(pool: &mut Pool, inode: u64) {
    pool.write_time_seconds += 1;
    let t = pool.write_time_seconds;
    pool.publish(UserChange::Unlink { inode: InodeNumber(inode), write_time_seconds: t }, None)
        .expect("unlink 做得成");
}

fn leaves(pool: &Pool) -> Vec<(u64, usize)> {
    pool.current_file_version()
        .inode_leaf_containers
        .iter()
        .map(|c| (c.contents.identity.container, c.contents.records.len()))
        .collect()
}

#[test]
fn both_neighbours_fit_after_deleting_from_the_middle_of_three_leaves_of_117() {
    let start = UnitAreaStart::following_the_journal_ring(6 << 20).expect("1408").slot().0;
    let mut pool: Pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        (start + 1007 + 20_000) * SLOT_BYTES,
        plain_device,
    );
    pool.write_time_seconds += 1;
    let t = pool.write_time_seconds;
    pool.publish(UserChange::NewInodes { count: 698, write_time_seconds: t }, None).expect("建 698 个");
    println!("name=direction before-trim leaves={:?}", leaves(&pool));
    // 迭代上界：三段各 116 个号。
    for inode in (2..=117).chain(234..=349).chain(467..=582) {
        unlink(&mut pool, inode);
    }
    println!("name=direction trimmed leaves={:?}", leaves(&pool));
    let before = pool.current_file_version().retirement_containers.iter().map(|c| c.contents.records.len()).sum::<usize>();
    unlink(&mut pool, 350);
    let records: Vec<(u64, u64)> = pool
        .current_file_version()
        .retirement_containers
        .iter()
        .flat_map(|c| c.contents.records.iter().map(|r| (r.retired.container, r.death_txg.0)))
        .collect();
    println!(
        "name=direction after-unlink-350 leaves={:?} new_retirement_records={:?} checker_red={}",
        leaves(&pool),
        &records[before..],
        checker_violations_on(&pool.image()).len()
    );
}
