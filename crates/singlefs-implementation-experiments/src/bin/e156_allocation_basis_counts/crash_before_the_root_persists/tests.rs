//! 根落盘之前崩溃的两种造法、Q7c 的判别力自证与分配代的影子账的单测。

use super::new_pool_file_creation_state;
use super::{parameters, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::transaction::{FirstFile, PoolWriter};

/// 岔路 1 的分配代账本（G12 要的那 8 字节，真实记录里没有，登记「三」N2）：一次覆盖写释放的记录，账本里查到
/// 的分配代应等于释放前它在「仍分配」快照里的那个值——不是这次发布自己的释放代（真实记录的 `generation`
/// 字段释放那一刻被改写成释放代，登记「三」I1，装置只能另存一份）。
#[test]
fn allocation_generation_ledger_recovers_the_pre_release_generation() {
    let parameters = parameters();
    let (mut allocator, first, mut devices) = new_pool_file_creation_state();
    let before = super::snapshot_allocated_generations(&allocator);
    assert!(!before.is_empty(), "新池新建文件之后应当有「仍分配」的记录");
    let instance = singlefs_core::address::InstanceGeneration(1);
    let overwritten = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        singlefs_core::transaction::publish_overwrite(
            &mut pool,
            &mut allocator,
            &first,
            FirstFile {
                content: &super::overwrite_content(1),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 61,
            },
            instance,
        )
        .expect("一次覆盖写")
    };
    // 这次覆盖写自己释放的记录（device 0）：每一条在覆盖写之前必然是「仍分配」——它们理应全部出现在
    // `before` 这份「仍分配」快照里；`snapshot_allocated_generations` 认反方向（改成只收「已释放」的）
    // 就会漏光这些槽，这条断言必须抓到那种反向。
    let released_this_step: Vec<u64> = allocator
        .records()
        .iter()
        .filter(|record| {
            record.device == DeviceIdentity(0)
                && record.is_released
                && record.generation == overwritten.root.checkpoint_txg
        })
        .map(|record| record.slot.0)
        .collect();
    assert!(
        !released_this_step.is_empty(),
        "S1(e)：一次覆盖写应当释放至少一条记录"
    );
    for slot in &released_this_step {
        assert!(
            before.contains_key(slot),
            "这次覆盖写释放的槽 {slot} 在覆盖写之前必然『仍分配』，应当出现在 `before` 快照里"
        );
    }
    let mut ledger = std::collections::HashMap::new();
    super::record_release_generations(&mut ledger, &before, &allocator);
    assert!(
        !ledger.is_empty(),
        "一次覆盖写应当释放至少一条记录（S1(e)：应为 10 槽对应的记录）"
    );
    for slot in &released_this_step {
        assert!(
            ledger.contains_key(slot),
            "这次覆盖写释放的槽 {slot} 应当被记进分配代账本"
        );
    }
    for (slot, allocation_generation) in &ledger {
        assert_eq!(
            before.get(slot).copied(),
            Some(*allocation_generation),
            "账本记的分配代应与释放前『仍分配』快照里的那个值一致，槽 {slot}"
        );
    }
}
