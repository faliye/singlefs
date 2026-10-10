//! 贴边建满后 unlink 不超卖（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第五节 K5-2 那一行，用户 2026-10-09 定：
//! 「这个用例要做，做一个warning。内容是没测出来。」）。读法照两次攻方抽样（`research/prompts/m3-batch50-code-r1-opus-output.md`、
//! `research/prompts/m3-batch50-code-r1-opus-s2-output.md` 的 K5-2 一节）：
//! ① unlink 的每一次删除类发布（意图、各批，含开出第一片退役容器的最后一批）都发成，不因空间被拒；
//! ② 删除走完之后每块盘的稳态可用 = 准入读数的可用(d) + 这块盘此刻 defer 里的槽 × 16384 ≥ 0。
//! 删除类发布只判落点、不判式子（D3（空间分配） 已定项 2、D28（挂载期承诺量） 已定项 1），删完那一刻可用(d) 为负是环窗口在兜，不算超卖。
//!
//! 池是随机历史单元区 384 槽那一档小盘（两块内存稀疏盘，环窗口补足之后单元区每盘 1391 槽），第一个文件之外建 233 个空 inode：
//! 234 条记录装成 [1..=233]、[234] 两片叶。逐格把第一个文件顺序写成 `du` 个数据单元（准入判着），`du` 从 1 扫到第一个被拒的那一格；
//! 被拒之前的每一格（贴边那一格及以下每一格）各从同一个池拷一份，写完 unlink 2 号：左片吸收右片，同一次 unlink 里开出墓碑树与第一片退役容器。
//!
//! 没测出来的那一半：这条用例对退役预付项没有判别力。两次攻方抽样把准入里退役那几项（标签 6 的退役常驻 2 + H_退役 + 1 与 35 × n_记录、
//! ckpt_cost 的退役项与它的映射几把、环窗口 W 的 6 + H_退役）全部乘 0，同一读法照样绿（贴边那一格挪到更大的 du，删除照样发成、稳态可用照样为正）。
//! 所以用例跑完向 stderr 直接写一行（不经 libtest 的输出截获，`cargo test` 不带 `--nocapture` 也看得见）：
//! 前缀 `WARNING_NOT_DETECTED`，内容是「没测出来：去掉退役预付各项这条用例照样绿，退役项的预付由 `admission.rs` 单测钉值与变异行罩着」。
//! 这一行不让用例红；按前缀找：`grep -rn WARNING_NOT_DETECTED crates/`。

mod common_admission;

use std::io::Write;

use common_admission::{plain_devices_on, start_plain, PoolUnderTest};
use singlefs_core::address::InodeNumber;
use singlefs_core::admission::admission_reading_before_a_publish;
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::write_request_split::data_unit_count_of_a_sequential_write;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;

/// 第一个文件之外建的空 inode 数：连第一个文件 234 条记录，装成 [1..=233]、[234] 两片叶（两次攻方抽样的起点 C）。
const EMPTY_INODES_CREATED_BEFORE_THE_WRITE: u64 = 233;

/// 贴边之后 unlink 的那个空 inode：删掉它左片剩 232 条，吸收右片那 1 条，最后一批开出第一片退役容器。
const INODE_WHOSE_UNLINK_ABSORBS_THE_RIGHT_LEAF: InodeNumber = InodeNumber(2);

/// 扫的数据单元数上界：单元区每盘 1391 槽、一个数据单元占 2 槽，写到 696 个之前一定被拒；扫到这里还没被拒说明没找到贴边那一格。
const DATA_UNITS_SWEPT_AT_MOST: u64 = 700;

/// 用例跑完打的那一行的前缀（文件头写明）。
const WARNING_NOT_DETECTED_PREFIX: &str = "WARNING_NOT_DETECTED";

/// 同一个池拷一份：两块盘的镜像、会话（分配器、现行那一版）与每条根带的内容都拷，改拷贝不动原来那份。
fn copy_of_the_pool(pool: &PoolUnderTest<SparseBlockDevice>) -> PoolUnderTest<SparseBlockDevice> {
    let session = pool.session();
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

/// 删除走完之后每块盘的稳态可用（字节）：准入读数的可用(d) 加这块盘此刻 defer 里的槽 × 16384。
fn settled_available_bytes_on_each_device(pool: &PoolUnderTest<SparseBlockDevice>) -> Vec<i128> {
    let session = pool.session();
    let reading =
        admission_reading_before_a_publish(&session.allocator, session.current.file_version());
    reading
        .available_on_each_device()
        .into_iter()
        .map(|(device, available)| {
            let device_map = session
                .allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == device)
                .expect("准入读数的盘就是分配器的那几块盘");
            available.0 + i128::from(device_map.deferred_slots()) * i128::from(SLOT_BYTES)
        })
        .collect()
}

// harness-test-granularity:one-scenario 扫填盘程度找贴边那一格：每格从同一个建好 inode 的池拷一份，扫到第一个被拒的那一格为止合起来才是「贴边那一格及以下每一格」这一个场景
#[test]
#[ignore = "harness 慢档用例：贴边那一格 114 个数据单元，逐格顺序写到它、每格 unlink 一次；debug 下单跑量过 108.59 秒（2026-10-10，机器上另有负载）"]
fn unlink_after_writing_up_to_the_admission_edge_publishes_every_deletion_step_and_leaves_settled_room_on_every_device(
) {
    let mut pool_with_the_inodes = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    pool_with_the_inodes.write_time_seconds += 1;
    let write_time_seconds_of_the_new_inodes = pool_with_the_inodes.write_time_seconds;
    pool_with_the_inodes
        .publish(
            UserChange::NewInodes {
                count: EMPTY_INODES_CREATED_BEFORE_THE_WRITE,
                write_time_seconds: write_time_seconds_of_the_new_inodes,
            },
            None,
        )
        .expect("单元区 384 槽那一档小盘上建 233 个空 inode 放得进");
    assert_eq!(
        pool_with_the_inodes
            .current_file_version()
            .inode_leaf_containers
            .len(),
        2,
        "234 条记录装成两片叶（[1..=233]、[234]）：unlink 2 号才会让左片吸收右片"
    );

    let payload_bytes_of_one_data_unit =
        u64::try_from(data_unit_payload_capacity()).expect("一个数据单元的载荷装得进 u64");
    let mut violations: Vec<String> = Vec::new();
    let mut edge_data_units: Option<u64> = None;
    let mut smallest_settled_available_bytes: Option<i128> = None;
    // 迭代上界：DATA_UNITS_SWEPT_AT_MOST 格；提前出口：第一个被拒的那一格（贴边那一格是它的前一格）。
    for data_units in 1..=DATA_UNITS_SWEPT_AT_MOST {
        let mut pool = copy_of_the_pool(&pool_with_the_inodes);
        let content_length_in_bytes = data_units * payload_bytes_of_one_data_unit;
        assert_eq!(
            data_unit_count_of_a_sequential_write(content_length_in_bytes),
            data_units,
            "第 {data_units} 格：这么长的内容顺序写成 {data_units} 个数据单元"
        );
        match pool.sequential_write(
            usize::try_from(content_length_in_bytes).expect("几 MiB 的内容长度装得进 usize"),
        ) {
            Ok(_) => {}
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)) => {
                edge_data_units = Some(data_units - 1);
                break;
            }
            Err(refusal) => {
                panic!("第 {data_units} 格：顺序写只该因为空间不够被拒，实际 {refusal:?}")
            }
        }
        assert!(
            pool.current_file_version().retirement_containers.is_empty(),
            "第 {data_units} 格：unlink 之前还没有退役容器"
        );

        pool.write_time_seconds += 1;
        let write_time_seconds_of_the_unlink = pool.write_time_seconds;
        let unlink = pool.publish(
            UserChange::Unlink {
                inode: INODE_WHOSE_UNLINK_ABSORBS_THE_RIGHT_LEAF,
                write_time_seconds: write_time_seconds_of_the_unlink,
            },
            None,
        );
        if let Err(refusal) = unlink {
            violations.push(format!(
                "第 {data_units} 格：unlink 的删除类发布没发成：{refusal:?}"
            ));
            continue;
        }
        let version_after_the_unlink = pool.current_file_version();
        assert!(
            version_after_the_unlink.deleted_inodes.is_empty(),
            "第 {data_units} 格：会话一次调用走完意图与各批，删除队列空了"
        );
        assert_eq!(
            (
                version_after_the_unlink.inode_leaf_containers.len(),
                version_after_the_unlink.retirement_containers.len(),
            ),
            (1, 1),
            "第 {data_units} 格：左片吸收右片、最后一批开出第一片退役容器"
        );

        for (device_index, settled_available_bytes) in settled_available_bytes_on_each_device(&pool)
            .into_iter()
            .enumerate()
        {
            smallest_settled_available_bytes = Some(
                smallest_settled_available_bytes.map_or(settled_available_bytes, |smallest| {
                    smallest.min(settled_available_bytes)
                }),
            );
            if settled_available_bytes < 0 {
                violations.push(format!(
                    "第 {data_units} 格：删除走完之后第 {device_index} 块盘的稳态可用 {settled_available_bytes} 字节 < 0"
                ));
            }
        }
    }

    let edge_data_units = edge_data_units.unwrap_or_else(|| {
        panic!("扫到 {DATA_UNITS_SWEPT_AT_MOST} 个数据单元顺序写还没被拒：没找到贴边那一格")
    });
    assert!(
        edge_data_units >= 1,
        "贴边那一格至少写得进一个数据单元：一格都没写进，unlink 一次都没在贴边之下跑过"
    );
    assert!(
        violations.is_empty(),
        "贴边那一格（{edge_data_units} 个数据单元）及以下：每次删除类发布都发成、删完稳态可用 ≥ 0，实际：\n{}",
        violations.join("\n")
    );
    let smallest_settled_available_bytes =
        smallest_settled_available_bytes.expect("贴边那一格至少一格、每格两块盘都读过稳态可用");
    // 直接写 stderr、不经 eprintln!：libtest 截获的是 print! 一族，这一行要在不带 --nocapture 的 cargo test 里也看得见。
    writeln!(
        std::io::stderr().lock(),
        "{WARNING_NOT_DETECTED_PREFIX} test=unlink_after_writing_up_to_the_admission_edge_publishes_every_deletion_step_and_leaves_settled_room_on_every_device \
         没测出来：去掉退役预付各项这条用例照样绿，退役项的预付由 `admission.rs` 单测钉值与变异行罩着 \
         （贴边 {edge_data_units} 个数据单元，最小稳态可用 {smallest_settled_available_bytes} 字节）"
    )
    .expect("stderr 写得进");
}
