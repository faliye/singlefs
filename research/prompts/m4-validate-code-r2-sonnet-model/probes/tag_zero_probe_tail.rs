
// ───────── 以下是辩方腿 m4-validate-code-r2-sonnet 加的探针（上面是冻结副本里同名用例文件的辅助函数，逐字照搬）─────────
// 不断言产品行为，只打 PROBE 行：每条改动序列做完，报第一个文件在上段叶里的那一条、三处读者各读到什么、池级 checker 判红了哪几条。
// 会话里某一步 panic 或被拒，也打成一行，不让测试红。
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use std::panic::{catch_unwind, AssertUnwindSafe};

fn session_outcome(image: &MemoryPool, changes: Vec<UserChange<'_>>) -> Result<MemoryPool, String> {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let mut devices = writable_devices_of(image);
        let mounted = mount_writable(&parameters(), &mut devices)
            .map_err(|refusal| format!("mount_writable_refused {refusal:?}"))?;
        let (_, mut session) = MountedSession::of_the_mount(mounted);
        for (step, change) in changes.into_iter().enumerate() {
            let what = format!("{change:?}").chars().take(60).collect::<String>();
            session
                .publish_user_change(&mut devices, change)
                .map_err(|refusal| format!("step {step} {what} refused {refusal:?}").chars().take(300).collect::<String>())?;
        }
        Ok::<MemoryPool, String>(image_of_the_devices(&devices, image.device_size_in_bytes))
    }));
    match outcome {
        Ok(result) => result,
        Err(payload) => {
            let message = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).to_string()))
                .unwrap_or_else(|| "panic without a message".to_string());
            Err(format!("PANIC {message}"))
        }
    }
}

fn entry_text(image: &MemoryPool) -> String {
    match catch_unwind(AssertUnwindSafe(|| first_file_entry_of_the_extent_upper_segment(image))) {
        Ok(None) => "no_entry".to_string(),
        Ok(Some(target)) => match target {
            ExtentUpperLeafTarget::NoDataUnit => "tag0".to_string(),
            ExtentUpperLeafTarget::LowerSegmentRoot(_) => "tag1".to_string(),
            ExtentUpperLeafTarget::InlineDataUnit(_) => "tag2".to_string(),
            ExtentUpperLeafTarget::LowerSegmentRootFollowedByAHole(_) => "tag3".to_string(),
            ExtentUpperLeafTarget::InlineDataUnitFollowedByAHole(_) => "tag4".to_string(),
        },
        Err(_) => "entry_unreadable".to_string(),
    }
}

fn readers_text(image: &MemoryPool, read_the_content: bool) -> String {
    let cold = if read_the_content {
        let root = newest_root(image);
        let mut fallbacks = 0;
        match walk_to_file(image, &root, &mut fallbacks) {
            Ok(Some(bytes)) => format!("ok(len={},all_zero={})", bytes.len(), bytes.iter().all(|byte| *byte == 0)),
            Ok(None) => "ok(no_file)".to_string(),
            Err(failure) => format!("ERR({failure:?})").chars().take(160).collect(),
        }
    } else {
        "skipped".to_string()
    };
    let mut devices = writable_devices_of(image);
    let remount = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => match mounted.current {
            PoolVersion::WithFile(version) => format!(
                "ok(size={:?},units={})",
                version.inode_record.map(|record| record.size),
                version.data_pointers.len()
            ),
            _ => "ok(not_with_file)".to_string(),
        },
        Err(refusal) => format!("ERR({refusal:?})").chars().take(160).collect(),
    };
    let mounted_read = match mount_read_only(image) {
        Ok(read_only) => match read_only.mounted.open_file(image, FIRST_FILE) {
            Ok(file) => {
                let size = file.inode_record().size;
                let units = file.data_unit_count();
                if read_the_content {
                    match file.read_at(image, FileOffsetInBytes(0), size) {
                        Ok(read) => format!("ok(size={size},units={units},read_len={},all_zero={})", read.bytes.len(), read.bytes.iter().all(|byte| *byte == 0)),
                        Err(failure) => format!("size={size},units={units},read_ERR({failure:?})").chars().take(160).collect(),
                    }
                } else {
                    format!("ok(size={size},units={units})")
                }
            }
            Err(failure) => format!("open_ERR({failure:?})").chars().take(160).collect(),
        },
        Err(refusal) => format!("ERR({refusal:?})").chars().take(160).collect(),
    };
    format!("cold_walk={cold} writable_remount={remount} mounted_read={mounted_read}")
}

fn checker_text(image: &MemoryPool) -> String {
    let verdicts = check_pool_image(image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect();
    format!("checker_checked={} checker_violated={violated:?}", verdicts.len())
}

fn probe(name: &str, image: &MemoryPool, changes: Vec<UserChange<'_>>, read_the_content: bool) {
    match session_outcome(image, changes) {
        Ok(after) => eprintln!(
            "PROBE name={name} session=ok entry={} {} {}",
            entry_text(&after),
            readers_text(&after, read_the_content),
            checker_text(&after)
        ),
        Err(message) => eprintln!("PROBE name={name} session=FAILED {message}"),
    }
}

fn tag_zero(tag: &str) -> MemoryPool {
    image_whose_first_file_is_a_whole_file_hole(tag)
}

#[test]
fn probe_baseline_the_hand_built_image_itself() {
    let image = tag_zero("probe-baseline");
    eprintln!(
        "PROBE name=baseline entry={} {} {}",
        entry_text(&image),
        readers_text(&image, true),
        checker_text(&image)
    );
}

fn truncate(size: u64) -> UserChange<'static> {
    UserChange::Truncate { inode: FIRST_FILE, new_size: size, write_time_seconds: WRITE_TIME_SECONDS }
}

#[test]
fn probe_truncate_to_zero() {
    probe("truncate_to_zero", &tag_zero("probe-trunc0"), vec![truncate(0)], true);
}

#[test]
fn probe_truncate_to_zero_then_extend() {
    probe(
        "truncate_to_zero_then_extend_to_three_payloads",
        &tag_zero("probe-trunc0-extend"),
        vec![truncate(0), truncate(3 * payload_capacity_in_bytes())],
        true,
    );
}

#[test]
fn probe_truncate_to_zero_then_rewrite_the_upper_leaf_for_inode_two() {
    let content = vec![9u8; 100];
    probe(
        "truncate_to_zero_then_new_inode_and_write_inode_two",
        &tag_zero("probe-trunc0-leaf"),
        vec![
            truncate(0),
            UserChange::NewInodes { count: 1, write_time_seconds: WRITE_TIME_SECONDS },
            UserChange::OverwriteOfAnInode { inode: InodeNumber(2), content: &content, write_time_seconds: WRITE_TIME_SECONDS },
        ],
        true,
    );
}

#[test]
fn probe_truncate_to_zero_then_grow_the_upper_segment() {
    let content = vec![5u8; 100];
    let past = InodeNumber(INODES_PER_UPPER_LEAF);
    probe(
        "truncate_to_zero_then_write_an_inode_past_the_first_upper_leaf",
        &tag_zero("probe-trunc0-grow"),
        vec![
            truncate(0),
            UserChange::NewInodes { count: past.0 - 1, write_time_seconds: WRITE_TIME_SECONDS },
            UserChange::OverwriteOfAnInode { inode: past, content: &content, write_time_seconds: WRITE_TIME_SECONDS },
        ],
        true,
    );
}

#[test]
fn probe_first_data_unit_written_into_a_tag_zero_file() {
    let content = vec![7u8; 100];
    probe(
        "overwrite_inode_one_with_100_bytes",
        &tag_zero("probe-first-write-small"),
        vec![UserChange::OverwriteOfAnInode { inode: FIRST_FILE, content: &content, write_time_seconds: WRITE_TIME_SECONDS }],
        true,
    );
    let long = vec![3u8; usize::try_from(3 * payload_capacity_in_bytes() + 7).expect("装得进")];
    probe(
        "sequential_write_inode_one_three_payloads_plus_7",
        &tag_zero("probe-first-write-long"),
        vec![UserChange::SequentialWriteOfAnInode { inode: FIRST_FILE, content: &long, write_time_seconds: WRITE_TIME_SECONDS }],
        true,
    );
}

#[test]
fn probe_first_data_unit_after_shrink_and_extend() {
    let content = vec![7u8; 50];
    probe(
        "shrink_100_extend_5_payloads_then_overwrite_50_bytes",
        &tag_zero("probe-shrink-extend-write"),
        vec![
            truncate(100),
            truncate(5 * payload_capacity_in_bytes()),
            UserChange::OverwriteOfAnInode { inode: FIRST_FILE, content: &content, write_time_seconds: WRITE_TIME_SECONDS },
        ],
        true,
    );
}

#[test]
fn probe_unlink_the_tag_zero_first_file() {
    probe(
        "unlink_inode_one",
        &tag_zero("probe-unlink"),
        vec![UserChange::Unlink { inode: FIRST_FILE, write_time_seconds: WRITE_TIME_SECONDS }],
        false,
    );
}

#[test]
fn probe_unlink_after_truncate_to_zero() {
    probe(
        "truncate_to_zero_then_unlink_inode_one",
        &tag_zero("probe-trunc0-unlink"),
        vec![truncate(0), UserChange::Unlink { inode: FIRST_FILE, write_time_seconds: WRITE_TIME_SECONDS }],
        false,
    );
}

#[test]
fn probe_create_snapshot_then_truncate_the_tag_zero_file() {
    probe(
        "snapshot_then_truncate_100",
        &tag_zero("probe-snapshot"),
        vec![UserChange::CreateSnapshot, truncate(100)],
        true,
    );
}

#[test]
fn probe_extend_to_one_tebibyte_tag_zero_and_ordinary() {
    let one_tebibyte = 1u64 << 40;
    probe("tag_zero_extend_to_2_pow_40", &tag_zero("probe-huge-tag0"), vec![truncate(one_tebibyte)], false);
    let ordinary = common::build_pool("probe-huge-ordinary").memory_pool();
    probe("ordinary_3000_byte_file_extend_to_2_pow_40", &ordinary, vec![truncate(one_tebibyte)], false);
}

#[test]
fn probe_ordinary_file_truncated_to_zero_for_comparison() {
    let ordinary = common::build_pool("probe-ordinary-trunc0").memory_pool();
    probe("ordinary_3000_byte_file_truncate_to_zero", &ordinary, vec![truncate(0)], true);
}

// ───────── 第二批探针：截到 0 之后的边（第一批里 truncate_to_zero 之后重写上段叶的两条读不回，这一批把它拆开）─────────
#[test]
fn probe_n0_split_a_new_inodes_only() {
    probe(
        "split_a_truncate_to_zero_then_new_inodes_only",
        &tag_zero("probe-split-a"),
        vec![truncate(0), UserChange::NewInodes { count: 1, write_time_seconds: WRITE_TIME_SECONDS }],
        true,
    );
}

#[test]
fn probe_n0_split_b_truncate_to_zero_twice() {
    probe("split_b_truncate_to_zero_twice", &tag_zero("probe-split-b"), vec![truncate(0), truncate(0)], true);
}

#[test]
fn probe_n0_split_c_truncate_to_zero_then_overwrite_inode_one() {
    let content = vec![4u8; 100];
    probe(
        "split_c_truncate_to_zero_then_overwrite_inode_one_100_bytes",
        &tag_zero("probe-split-c"),
        vec![
            truncate(0),
            UserChange::OverwriteOfAnInode { inode: FIRST_FILE, content: &content, write_time_seconds: WRITE_TIME_SECONDS },
        ],
        true,
    );
}

#[test]
fn probe_n0_split_d_leaf_rewrite_first_then_truncate_to_zero() {
    let content = vec![9u8; 100];
    probe(
        "split_d_new_inode_write_inode_two_then_truncate_to_zero",
        &tag_zero("probe-split-d"),
        vec![
            UserChange::NewInodes { count: 1, write_time_seconds: WRITE_TIME_SECONDS },
            UserChange::OverwriteOfAnInode { inode: InodeNumber(2), content: &content, write_time_seconds: WRITE_TIME_SECONDS },
            truncate(0),
        ],
        true,
    );
}

#[test]
fn probe_n0_split_e_unlink_after_truncate_to_zero_read_everything() {
    probe(
        "split_e_truncate_to_zero_then_unlink_read_content",
        &tag_zero("probe-split-e"),
        vec![truncate(0), UserChange::Unlink { inode: FIRST_FILE, write_time_seconds: WRITE_TIME_SECONDS }],
        true,
    );
    probe(
        "split_e_unlink_without_truncate_read_content",
        &tag_zero("probe-split-e2"),
        vec![UserChange::Unlink { inode: FIRST_FILE, write_time_seconds: WRITE_TIME_SECONDS }],
        true,
    );
}

#[test]
fn probe_n0_split_f_unlink_then_new_inode_and_write_it() {
    let content = vec![6u8; 100];
    probe(
        "split_f_truncate_to_zero_unlink_then_new_inode_write_inode_two",
        &tag_zero("probe-split-f"),
        vec![
            truncate(0),
            UserChange::Unlink { inode: FIRST_FILE, write_time_seconds: WRITE_TIME_SECONDS },
            UserChange::NewInodes { count: 1, write_time_seconds: WRITE_TIME_SECONDS },
            UserChange::OverwriteOfAnInode { inode: InodeNumber(2), content: &content, write_time_seconds: WRITE_TIME_SECONDS },
        ],
        true,
    );
}

// ───────── 第三批探针：三种改 size 碰没碰 extent 树（根指针前后比）─────────
fn extent_root_slot_of(image: &MemoryPool) -> u64 {
    the_slot_of(&tree_table_entry_of_kind(image, TREE_KIND_EXTENT).root).0
}

#[test]
fn probe_extent_root_before_and_after_each_size_change() {
    let before = tag_zero("probe-extent-root");
    let slot_before = extent_root_slot_of(&before);
    for (name, size) in [
        ("shrink_to_100", 100u64),
        ("same_size_3000", SIZE_OF_THE_FIRST_FILE),
        ("extend_to_three_payloads", 3 * payload_capacity_in_bytes()),
        ("truncate_to_zero", 0),
    ] {
        match session_outcome(&before, vec![truncate(size)]) {
            Ok(after) => eprintln!(
                "PROBE name=extent_root_{name} extent_root_slot_before={slot_before} extent_root_slot_after={} entry_after={}",
                extent_root_slot_of(&after),
                entry_text(&after)
            ),
            Err(message) => eprintln!("PROBE name=extent_root_{name} session=FAILED {message}"),
        }
    }
}
