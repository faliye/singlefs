//! C659（fsid 前 8 字节相同时重做 mkfs 实例代号不变式红） 认下的已知形态（用户 2026-10-20 定「认作已知形态，钉坏镜像」，
//! 三方判决 `research/prompts/m3-close-c656-r3-main-verification.md` 第二节 S4 那一行与第三节）：
//! 用过的盘（旧池取号到实例 2）上经内层入口、拿一个与旧池 fsid 前 8 字节相同的 fsid 把 mkfs 做完，钉住三件——
//! 池级 checker 报 I-7.7（系统配置实例代号不低于根环） 红；读者读对（恢复读回的是新池，不是旧池的任何一版）；
//! 红随取号自愈（旧池单元里最大的实例代号 K = 2，新池第 K 次挂载之后转绿）。
//!
//! 根因（C659 那一行）：mkfs 不清单元区，单元头只带 fsid 前 8 字节（`singlefs_core::unit::unit_filesystem_identifier`），
//! I-7.7 扫整个单元区，旧池写的单元带的实例代号高于新池系统配置的就红。对外入口从随机源取 fsid 之后，这一形只剩
//! 前 8 字节随机碰撞与内层入口两种来路。前 8 字节不同的对照（同一个旧池）I-7.7 判成立。

mod common;

use common::{
    build_pool, crash_state_devices, file_content, memory_pool_of_sparse_devices, parameters,
    E142_FILESYSTEM_IDENTIFIER, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// I-7.7（系统配置实例代号不低于根环） 在池级 checker 里的名字。
const SYSTEM_CONFIGURATION_INSTANCE_INVARIANT: &str = "I-7.7";

/// 旧池用到的最大实例代号：mkfs 那条会话取号 1（暖机、第一个文件），重开可写挂载取号 2（写行、暖机，写行那次发布写单元）。
const HIGHEST_INSTANCE_OF_THE_OLD_POOL: InstanceGeneration = InstanceGeneration(2);

type RemadeDevices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 旧池用过之后的整份盘面：mkfs → 取号 1 → 暖机 → 第一个文件（txg 3）→ 进程退出、重开可写挂载（取号 2、写行、暖机）。
fn image_of_an_old_pool_used_up_to_instance_two(tag: &str) -> MemoryPool {
    let mut pool = build_pool(tag);
    let mut devices = pool.reopen_recorded();
    let remount = mount_writable(&parameters(), &mut devices).expect("旧池重开可写挂载");
    assert_eq!(
        remount.output.instance, HIGHEST_INSTANCE_OF_THE_OLD_POOL,
        "旧池重开取号 2"
    );
    pool.devices = Some(devices);
    pool.memory_pool()
}

/// 与旧池（[`E142_FILESYSTEM_IDENTIFIER`]）前 8 字节相同、后 8 字节不同的 fsid。
fn identifier_sharing_the_first_eight_bytes_of_the_old_pool() -> [u8; 16] {
    let mut filesystem_identifier = E142_FILESYSTEM_IDENTIFIER;
    filesystem_identifier[8..].copy_from_slice(b"remade-x");
    filesystem_identifier
}

/// 与旧池前 8 字节不同、后 8 字节相同的 fsid（对照）。
fn identifier_differing_in_the_first_eight_bytes_from_the_old_pool() -> [u8; 16] {
    let mut filesystem_identifier = E142_FILESYSTEM_IDENTIFIER;
    filesystem_identifier[..8].copy_from_slice(b"remade-x");
    filesystem_identifier
}

fn parameters_with(filesystem_identifier: [u8; 16]) -> MakeFilesystemParameters {
    let common_parameters = parameters();
    MakeFilesystemParameters {
        filesystem_identifier,
        region_devices: common_parameters.region_devices,
        geometry: common_parameters.geometry,
    }
}

/// 在旧池的盘面上经内层入口做完 mkfs：交回新池的参数与那两块内存盘。
fn remade_over(
    old_image: &MemoryPool,
    filesystem_identifier: [u8; 16],
) -> (MakeFilesystemParameters, RemadeDevices) {
    let remade_parameters = parameters_with(filesystem_identifier);
    let stream = SharedStream::new();
    let mut devices = crash_state_devices(old_image, &[], &[], &stream);
    make_filesystem(&remade_parameters, &mut devices).expect("用过的盘上经内层入口重做 mkfs");
    (remade_parameters, devices)
}

/// 池级 checker 对 I-7.7 的全部判定（同一条不变量可以有几句各判一次）。
fn instance_invariant_verdicts(devices: &RemadeDevices) -> Vec<InvariantVerdict> {
    check_pool_image(&memory_pool_of_sparse_devices(devices))
        .into_iter()
        .filter(|(invariant, _)| *invariant == SYSTEM_CONFIGURATION_INSTANCE_INVARIANT)
        .map(|(_, verdict)| verdict)
        .collect()
}

fn instance_invariant_is_violated(verdicts: &[InvariantVerdict]) -> bool {
    verdicts.iter().any(|verdict| match verdict {
        InvariantVerdict::Violated(_) => true,
        InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_) => false,
    })
}

/// I-7.7 判过而且成立：至少一句判成成立，没有一句判红、报不适用或部分没判。
fn instance_invariant_holds(verdicts: &[InvariantVerdict]) -> bool {
    !verdicts.is_empty()
        && verdicts.iter().all(|verdict| match verdict {
            InvariantVerdict::Holds => true,
            InvariantVerdict::Violated(_)
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => false,
        })
}

fn recovered_outcome(devices: &RemadeDevices) -> RecoveryOutcome {
    recover(
        &memory_pool_of_sparse_devices(devices),
        JournalPolicy::Consult,
    )
    .outcome
}

#[test]
fn the_pool_checker_reports_the_instance_invariant_red_right_after_a_remake_sharing_the_first_eight_identifier_bytes(
) {
    let old_image = image_of_an_old_pool_used_up_to_instance_two("c659-red-after-the-remake");
    let shared_prefix = identifier_sharing_the_first_eight_bytes_of_the_old_pool();
    assert_eq!(shared_prefix[..8], E142_FILESYSTEM_IDENTIFIER[..8]);
    assert_ne!(shared_prefix[8..], E142_FILESYSTEM_IDENTIFIER[8..]);
    let (_, devices) = remade_over(&old_image, shared_prefix);
    let verdicts = instance_invariant_verdicts(&devices);
    assert!(
        instance_invariant_is_violated(&verdicts),
        "前 8 字节相同的 fsid 在用过的盘上做完 mkfs：旧池写的单元带实例代号 2、高于新池系统配置的 0，I-7.7 该红：{verdicts:?}"
    );
}

/// 读者读对：做完 mkfs 恢复读回新池的第 0 代空池；挂载一次（实例 1）、发第一个文件版本之后读回的是新池这一版的内容，
/// 不是旧池第一个文件的那一版（两边内容故意不同）。
#[test]
fn the_reader_reads_the_remade_pool_and_not_the_old_one_when_the_first_eight_identifier_bytes_are_shared(
) {
    let old_image = image_of_an_old_pool_used_up_to_instance_two("c659-reader");
    let (remade_parameters, mut devices) = remade_over(
        &old_image,
        identifier_sharing_the_first_eight_bytes_of_the_old_pool(),
    );
    assert_eq!(
        recovered_outcome(&devices),
        RecoveryOutcome::NoFile {
            root: (InstanceGeneration(0), CheckpointTxg(0))
        },
        "做完 mkfs 恢复读回新池的第 0 代空池"
    );
    let mounted = mount_writable(&remade_parameters, &mut devices).expect("新池第一次可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(1));
    assert_eq!(
        (
            mounted.output.chosen_root.instance,
            mounted.output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(0), CheckpointTxg(0)),
        "挂载择到的是新池的第 0 代根"
    );
    let content_of_the_remade_pool: Vec<u8> =
        file_content().iter().map(|byte| byte ^ 0x5a).collect();
    assert_ne!(content_of_the_remade_pool, file_content());
    let mut allocator = mounted.allocator;
    let published = {
        let mut writer = PoolWriter::new(&remade_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            mounted.current.root(),
            FirstFile {
                content: &content_of_the_remade_pool,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(1),
            mounted.current.record_bytes(),
        )
        .expect("新池第一个文件版本")
    };
    assert_eq!(
        recovered_outcome(&devices),
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), published.root.checkpoint_txg),
            content: content_of_the_remade_pool,
        },
        "恢复读回新池第一个文件那一版的内容"
    );
}

/// 红随取号自愈：新池每挂载一次取一个号（系统配置的实例代号跟着涨），第 1 次挂载（实例 1 < 2）照旧红，
/// 第 2 次挂载（实例 2，追上旧池单元里最大的那个）之后 I-7.7 判成立。全程恢复读回新池的空池。
// harness-test-granularity:one-scenario 同一个新池上依次挂载两次，循环里不新建池
#[test]
fn the_instance_invariant_turns_green_at_the_mount_whose_instance_reaches_the_old_pools_highest() {
    let old_image = image_of_an_old_pool_used_up_to_instance_two("c659-self-healing");
    let (remade_parameters, mut devices) = remade_over(
        &old_image,
        identifier_sharing_the_first_eight_bytes_of_the_old_pool(),
    );
    for mount_count in 1..=HIGHEST_INSTANCE_OF_THE_OLD_POOL.0 {
        let mounted = mount_writable(&remade_parameters, &mut devices).expect("新池可写挂载");
        assert_eq!(mounted.output.instance, InstanceGeneration(mount_count));
        let RecoveryOutcome::NoFile { root } = recovered_outcome(&devices) else {
            panic!("第 {mount_count} 次挂载之后恢复没读回新池的空池");
        };
        assert_eq!(
            root.0,
            InstanceGeneration(mount_count),
            "恢复落在这次挂载写的根上"
        );
        let verdicts = instance_invariant_verdicts(&devices);
        if mount_count < HIGHEST_INSTANCE_OF_THE_OLD_POOL.0 {
            assert!(
                instance_invariant_is_violated(&verdicts),
                "第 {mount_count} 次挂载之后新池实例 {mount_count} 还低于旧池单元的 2，I-7.7 该照旧红：{verdicts:?}"
            );
        } else {
            assert!(
                instance_invariant_holds(&verdicts),
                "第 {mount_count} 次挂载之后新池实例追上旧池单元的 2，I-7.7 该判成立：{verdicts:?}"
            );
        }
    }
}

/// 对照（判别力）：同一种旧池，换一个前 8 字节不同的 fsid 重做 mkfs，旧池的单元不算本池的载体，做完就判成立——
/// 上面三条的红出在前 8 字节相同，不出在用过的盘本身。
#[test]
fn a_remake_with_different_first_eight_identifier_bytes_keeps_the_instance_invariant_green() {
    let old_image = image_of_an_old_pool_used_up_to_instance_two("c659-control");
    let differing_prefix = identifier_differing_in_the_first_eight_bytes_from_the_old_pool();
    assert_ne!(differing_prefix[..8], E142_FILESYSTEM_IDENTIFIER[..8]);
    let (_, devices) = remade_over(&old_image, differing_prefix);
    let verdicts = instance_invariant_verdicts(&devices);
    assert!(
        instance_invariant_holds(&verdicts),
        "前 8 字节不同的 fsid 做完 mkfs，I-7.7 该判成立：{verdicts:?}"
    );
}
