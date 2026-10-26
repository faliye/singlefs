//! m3-close-w13-code-r1 攻方腿的模型（K2、K3）：用过的盘（旧池 fsid 取 harness 的 E142 那一个）上经内层入口拿一个前 8 字节不同的
//! fsid 重做 mkfs、写完全部 27 步（盘面与「作废持久之后、最后一道屏障之前、每一写都已持久」那个崩溃状态相同），
//! 再看：冷启动恢复落到哪；可写挂载在调用方给旧池 fsid、给新池 fsid、给第三个 fsid 时各报什么。
//! 对外入口只在成功时交回它取到的 fsid：中途崩溃时调用方手里只有旧池那一个（或什么都没有），这里把调用方给哪个 fsid 放开扫一遍。
//! 只在攻方腿的仓副本里跑（harness 档），不进库。

mod common;
mod common_pointer_head_corruption;

use common::{build_pool, parameters};
use common_pointer_head_corruption::writable_devices_of;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_harness::memory_pool::MemoryPool;

fn identifier_with_first_eight_bytes(first_eight: &[u8; 8]) -> [u8; 16] {
    let mut identifier = parameters().filesystem_identifier;
    identifier[..8].copy_from_slice(first_eight);
    identifier
}

#[test]
fn writable_mount_of_a_remade_pool_for_each_identifier_the_caller_may_hold() {
    let old_pool = build_pool("attack-w13-old-pool-for-the-remake").memory_pool();
    let remade_identifier = identifier_with_first_eight_bytes(b"attack-b");
    assert_ne!(remade_identifier[..8], parameters().filesystem_identifier[..8]);
    let remade_parameters = MakeFilesystemParameters {
        filesystem_identifier: remade_identifier,
        region_devices: parameters().region_devices,
        geometry: parameters().geometry,
    };
    let mut devices = writable_devices_of(&old_pool);
    make_filesystem(&remade_parameters, &mut devices).expect("用过的盘上重做 mkfs");
    let mut remade: MemoryPool = old_pool.clone();
    for (identity, device) in &devices {
        remade.devices.insert(*identity, device.image.clone());
    }
    let cold = match recover(&remade, JournalPolicy::Consult).outcome {
        RecoveryOutcome::Failed { failure, .. } => format!("恢复拒:{failure:?}"),
        RecoveryOutcome::FileRead { root, .. } => format!("恢复读出文件:{root:?}"),
        RecoveryOutcome::NoFile { root } => format!("恢复无文件:{root:?}"),
    };
    println!("name=remade cold={cold}");
    for (holder, identifier) in [
        ("old-pool-identifier", parameters().filesystem_identifier),
        ("remade-pool-identifier", remade_identifier),
        ("third-identifier", identifier_with_first_eight_bytes(b"attack-c")),
    ] {
        let caller_parameters = MakeFilesystemParameters {
            filesystem_identifier: identifier,
            ..remade_parameters.clone()
        };
        let outcome = match mount_writable(&caller_parameters, &mut writable_devices_of(&remade)) {
            Ok(mounted) => format!("挂上:实例{:?}", mounted.output.instance),
            Err(refusal) => format!("拒:{refusal:?}"),
        };
        println!("name=remade writable caller-holds={holder} outcome={}", outcome.chars().take(400).collect::<String>());
    }
}
