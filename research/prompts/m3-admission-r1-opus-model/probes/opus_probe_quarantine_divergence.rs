//! 攻方腿 m3-admission-r1-opus，A5（C574「各盘同槽」那一半）：一块盘上一个数据单元坏了一个字节，覆盖写释放它时读盘核出对不上，
//! 那一份的分配记录留在已分配（D19（块指针的结构与宽度预算） 已定项 5）；另一块盘上同一槽照常释放、回收。之后用户数据的落点各盘答的
//! 是不是同一槽、写会不会被拒、拒成什么、推抬 F 与重挂之后还拒不拒。对照：同一段历史不弄坏。只在冻结副本的拷贝上跑。
mod common_admission;
#[path = "opus_probe/common.rs"]
mod probe_common;

use probe_common::{counts_of, line_of, start_pool};
use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mounted_session::{UserChangePublished, UserChangeRefused};
use singlefs_core::transaction::{PublishError, TransactionUnit};

fn outcome_text(outcome: &Result<UserChangePublished, UserChangeRefused>) -> String {
    let refusal_text = |refusal: &PublishError| match refusal {
        PublishError::PlacementRefused { unit, refusal } => format!("PlacementRefused(unit={unit:?} refusal={refusal:?})"),
        PublishError::SpaceAdmissionRefused(short) => format!(
            "SpaceAdmissionRefused({})",
            short.short_devices.iter().map(|d| format!("d{} avail_slots={} demand_slots={}", d.device.0, d.available.0 / 16384, d.demand.0 / 16384)).collect::<Vec<_>>().join(",")
        ),
        other => format!("{}", format!("{other:?}").chars().take(100).collect::<String>()),
    };
    match outcome {
        Ok(published) => format!(
            "Ok raises={} quarantined={}",
            published.floor_raises.len(),
            published.versions_published.iter().map(|v| v.quarantined_after_release_checksum_mismatch.len()).sum::<usize>()
        ),
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => format!(
            "NoSpaceAfterRaisingTheFloor raises={} first_refusal={} last_refusal={} stop={}",
            no_space.floor_raises.len(),
            no_space.refusals_that_pushed_the_floor_raises.first().map_or("none".to_string(), refusal_text),
            refusal_text(&no_space.last_refusal),
            format!("{:?}", no_space.stop).chars().take(80).collect::<String>()
        ),
        Err(UserChangeRefused::Publish { cause, .. }) => format!("Publish {}", refusal_text(cause)),
        Err(other) => format!("Other {}", format!("{other:?}").chars().take(160).collect::<String>()),
    }
}

fn run(corrupt: bool) {
    println!("==== QD corrupt={corrupt}");
    let mut pool = start_pool([1391, 1391], 6 << 20);
    let first = pool.sequential_write(3 * singlefs_core::unit::data_unit_payload_capacity());
    println!("QD step=write3 outcome={}", outcome_text(&first));
    let version = pool.session().current.file_version().expect("带文件").clone();
    let data_slot = version
        .units
        .iter()
        .find(|unit| matches!(unit.identity, TransactionUnit::Data(_)))
        .expect("这一版写了数据单元")
        .slot;
    println!("QD data_slot_of_the_current_version={}", data_slot.0);
    if corrupt {
        // 盘 0 上那一份数据单元的第 200 个字节翻一下（载荷里），盘 1 上那一份不动。
        let offset = data_slot.0 * 16384 + 512;
        let mut sector = vec![0u8; 512];
        pool.devices[0].1.read_at(DeviceOffsetInBytes(offset), &mut sector).expect("读得出");
        sector[200] ^= 0xff;
        pool.devices[0].1.write_at(DeviceOffsetInBytes(offset), &sector, WriteDurability::Plain).expect("写得进");
    }
    for step in 1..=60 {
        let outcome = pool.sequential_write(3 * singlefs_core::unit::data_unit_payload_capacity());
        let counts = counts_of(&pool.session().allocator, &pool.session().current);
        let free_on_each: Vec<bool> = pool.session().allocator.devices.iter().map(|map| map.is_free(data_slot)).collect();
        let text = outcome_text(&outcome);
        let failed = outcome.is_err();
        if failed || step <= 2 || step % 10 == 0 {
            println!("QD step={step} outcome={text} data_slot_free_on_each_device={free_on_each:?}");
            println!("{}", line_of(&format!("qd step={step}"), &counts));
        }
        if failed {
            // 用户这时能做的：再试一次同样的写、写小一点、重挂之后再写。
            let retry = pool.sequential_write(3 * singlefs_core::unit::data_unit_payload_capacity());
            println!("QD after_refusal retry_same={}", outcome_text(&retry));
            let small = pool.overwrite(2999);
            println!("QD after_refusal overwrite_small={}", outcome_text(&small));
            let remount = pool.crash_and_mount_writable();
            println!("QD after_refusal crash_and_remount ok={}", remount.is_ok());
            if remount.is_ok() {
                let after = pool.sequential_write(3 * singlefs_core::unit::data_unit_payload_capacity());
                println!("QD after_remount write3={}", outcome_text(&after));
                let small_after = pool.overwrite(2999);
                println!("QD after_remount overwrite_small={}", outcome_text(&small_after));
            }
            return;
        }
    }
    println!("QD no refusal within 60 steps");
}

#[test]
fn opus_probe_c574_a_checksum_mismatch_on_one_device_and_user_data_placements_across_devices() {
    run(false);
    run(true);
}
