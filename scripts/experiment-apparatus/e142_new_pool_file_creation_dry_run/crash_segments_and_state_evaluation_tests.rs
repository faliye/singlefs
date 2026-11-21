use super::*;

/// E142 第十九次跑登记 7.4 C5 / C12（臂 N19C）：发布尾那道屏障（⑧）把暖机第二次的系统配置槽写与新池新建文件的
/// 24 个单元写隔开，整条流 `2+2+1+2+2+1+2+24+2+1+2`、两态 16777240；便宜（纯算术）。
/// 整轮枚举在下面 `layer0_state_count_is_16777240_with_zero_violations` 那条（登记 R19C-8 第二段）。
#[test]
fn layer0_segment_sizes_and_closed_form_match_the_new_layout() {
    let BuiltPool { recording, mkfs_operation_count, .. } = built_pool();
    let (_, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], true);
    assert_eq!(segments.iter().map(Vec::len).collect::<Vec<_>>(), vec![2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]);
    assert_eq!(closed_form_state_count(&segments), 16_777_240, "登记 7.4 C5 / C12「layer0 closed_form=16777240」");
}

/// 整轮枚举 16777240 个状态（登记 7.4 C12），并进默认那一趟，不再标 `#[ignore]`。
#[test]
fn layer0_state_count_is_16777240_with_zero_violations() {
    let BuiltPool { recording, mkfs_operation_count, .. } = built_pool();
    let (base, _) = mkfs(&PoolParameters::settled_two_devices());
    let (writes, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], true);
    let tally = enumerate_layer0(&base.pool, &writes, &segments, &sample_file(), ReaderMode::Clause);
    assert_eq!(tally.states, 16_777_240);
    assert_eq!(tally.violations, 0, "{:?}", tally.first_violation);
}

/// FUA 不当边界（C313 已判掉的另一读法，只钉它给的数不同）：登记 7.4 C13 N19C `2+2+3+2+3+24+2+3`、16777249；
/// G11（⑧ 关）`2+2+3+2+27+2+3`、134217754。
#[test]
fn fua_not_a_boundary_gives_16777249_states_and_134217754_without_the_trailing_barrier() {
    let BuiltPool { recording, mkfs_operation_count, .. } = built_pool();
    let (_, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], false);
    assert_eq!(segments.iter().map(Vec::len).collect::<Vec<_>>(), vec![2, 2, 3, 2, 3, 24, 2, 3]);
    assert_eq!(closed_form_state_count(&segments), 16_777_249);
    let stream_without_trailing_barrier = record_new_pool_file_creation_stream(&PoolParameters::settled_two_devices_without_trailing_publish_barrier(), &sample_file());
    let (_, segments_without_trailing_barrier) = split_into_segments(&stream_without_trailing_barrier.operations[stream_without_trailing_barrier.mkfs_operation_count..], false);
    assert_eq!(segments_without_trailing_barrier.iter().map(Vec::len).collect::<Vec<_>>(), vec![2, 2, 3, 2, 27, 2, 3]);
    assert_eq!(closed_form_state_count(&segments_without_trailing_barrier), 134_217_754);
}

/// 段序列的五条路径由这条钉住，绝对值取 E142 第十九次跑登记 7.4 D1–D3 与 R19E-4 第 4 条（臂 N19M：屏障按设备各记一步、
/// 同一块盘上连着的屏障并成一道、按设备放行切段、每次发布末尾一道屏障、种类串 `zero_fill` 排最前）：
/// mkfs 14+1+1+1+4（25 次操作、16402 个状态；E142 第二十五次跑臂 N25M：每块盘多一段 [0, 根环基址) 清零）、取号 2（2 次操作）、暖机 2+1+2+2+1+2（20 次操作）、
/// 事务 24+2+1+2（35 次操作）、整条流 2+2+1+2+2+1+2+24+2+1+2（57 次操作）。
/// 改任何一条路径里屏障或 FUA 的位置都红——mkfs 那一行不进层 0 枚举，这里是它唯一的会红检查。
///
/// D17（实现分层与第三方管道） 已定项 2 的结构等价类要的是「段边界位置 + 每段步骤种类集合」，
/// 所以这里连每段的步骤种类多重集一起钉死，四条路径各钉一个**绝对值**（不是拿几条路径互相比）：
/// 把某一步录成别的种类——例如系统配置槽写录成单元写——段边界与状态数一个都不变，只有这几行会红（C316 ②）。
#[test]
fn registered_segment_sequences_match_every_recorded_path() {
    let BuiltPool { recording, mkfs_operation_count, acquisition_operation_count, warm_up_operation_count, .. } = built_pool();
    let sizes = |operations: &[RecordedOperation]| split_into_segments(operations, true).1.iter().map(Vec::len).collect::<Vec<_>>();
    let kinds = |operations: &[RecordedOperation]| format_segment_kinds(&segment_step_kinds(operations, true));
    let mkfs_operations = &recording.operations[..mkfs_operation_count];
    let acquisition_operations = &recording.operations[mkfs_operation_count..acquisition_operation_count];
    let warm_up_operations = &recording.operations[acquisition_operation_count..warm_up_operation_count];
    let transaction_operations = &recording.operations[warm_up_operation_count..];
    let post_mkfs_operations = &recording.operations[mkfs_operation_count..];
    assert_eq!(mkfs_operations.len(), 25, "mkfs：21 次写（两块盘各清 [0, 根环基址)、根环三区域与 journal 环共 10、m1/m2 各两盘、三个第 0 代根、两盘各两个系统配置槽）+ 2 道池屏障 × 2 盘（E142 第二十五次跑登记 7.1 K1）");
    assert_eq!(sizes(mkfs_operations), vec![14, 1, 1, 1, 4]);
    assert_eq!(closed_form_state_count(&split_into_segments(mkfs_operations, true).1), 16402);
    assert_eq!(acquisition_operations.len(), 2, "取号：两盘各写一次系统配置槽，不另加屏障");
    assert_eq!(sizes(acquisition_operations), vec![2]);
    assert_eq!(warm_up_operations.len(), 20, "暖机：10 次写 + 每次空发布三道池屏障 × 2 次 × 2 盘，减去第二次开头那道与第一次发布尾那道连着、两块盘各并掉的一步（登记 R19E-4 第 4 条）");
    assert_eq!(sizes(warm_up_operations), vec![2, 1, 2, 2, 1, 2]);
    assert_eq!(transaction_operations.len(), 35, "新池新建文件：29 次写 + 三道池屏障 × 2 盘（登记 7.4 D2）");
    assert_eq!(sizes(transaction_operations), vec![24, 2, 1, 2]);
    assert_eq!(post_mkfs_operations.len(), 57, "整条流：取号 2 + 暖机 20 + 新池新建文件 35（登记 R19E-4 第 4 条）");
    assert_eq!(sizes(post_mkfs_operations), vec![2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]);

    assert_eq!(
        kinds(mkfs_operations),
        "[zero_fill×10,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]",
        "mkfs：两块盘各清 [0, 根环基址)、根环三区域与 journal 环（共 10）与 m1/m2 两个单元各两盘同一段、三个第 0 代根各自 FUA 一段、两盘各两个系统配置槽收尾"
    );
    assert_eq!(kinds(acquisition_operations), "[system_configuration_slot×2]", "取号那一段只有两次系统配置槽写");
    assert_eq!(
        kinds(warm_up_operations),
        "[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]",
        "暖机两次空发布：每次「空记录两盘 → 根 FUA → 系统配置槽两盘 → 屏障」；第一次的发布尾屏障关掉它那一段，第二次开头那道与它之间没有写、两块盘各并掉（R46）"
    );
    assert_eq!(
        kinds(transaction_operations),
        "[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]",
        "新池新建文件：12 个单元各两盘（t1..t12）→ journal 记录两盘 → 根 FUA → 系统配置槽两盘 → 屏障"
    );
    assert_eq!(
        kinds(post_mkfs_operations),
        "[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]",
        "整条流：取号那两次系统配置槽写自成一段（收段的是暖机第一次开头那道屏障），暖机第二次的发布尾屏障把它的系统配置槽写与事务的 24 个单元写隔开"
    );

    // 每一步都恰好落在一个段里：各段的步骤数加起来等于录到的操作数。
    for operations in [mkfs_operations, acquisition_operations, warm_up_operations, transaction_operations, post_mkfs_operations] {
        assert_eq!(segment_step_kinds(operations, true).iter().map(Vec::len).sum::<usize>(), operations.len());
    }
    // mkfs 之后的种类字母表仍是这五个：新的 zero_fill 只出现在 mkfs 自己的段里。
    let alphabet: std::collections::BTreeSet<&str> = segment_step_kinds(post_mkfs_operations, true).concat().iter().map(|kind| kind.tag()).collect();
    assert_eq!(alphabet.into_iter().collect::<Vec<_>>(), vec!["barrier", "journal_record", "root_record_fua", "system_configuration_slot", "unit_write"]);
    // 整条录制流（含 mkfs）的字母表六种（D22 已定项 8 第 3 条，含 zero_fill），
    // 与 layout/01-first-txn.md 八「mkfs 种根」那一行登记的真值（六种）同形。
    let global_alphabet: std::collections::BTreeSet<&str> = segment_step_kinds(&recording.operations, true).concat().iter().map(|kind| kind.tag()).collect();
    assert_eq!(
        global_alphabet.into_iter().collect::<Vec<_>>(),
        vec!["barrier", "journal_record", "root_record_fua", "system_configuration_slot", "unit_write", "zero_fill"],
        "步骤种类五种改六种（D22 已定项 8 第 3 条：mkfs 新增 zero_fill）"
    );
}

/// 一条路径的三态那一行的各个量，逐格对登记里的锚点。
fn assert_three_state(summary: &ThreeStateSummary, anchor: (usize, &str, &str, u64, u64), label: &str) {
    let (writes, in_place_per_segment, in_place_kinds, two_state, three_state) = anchor;
    assert_eq!(summary.writes, writes, "{label} writes");
    assert_eq!(summary.in_place_per_segment, in_place_per_segment, "{label} in_place_per_segment");
    assert_eq!(summary.in_place_kinds, in_place_kinds, "{label} in_place_kinds");
    assert_eq!(summary.closed_form_two_state, two_state, "{label} closed_form_two_state");
    assert_eq!(summary.closed_form_three_state, three_state, "{label} closed_form_three_state");
    let in_place_total: usize = in_place_per_segment.split('+').map(|count| count.parse::<usize>().expect("整数")).sum();
    assert_eq!(summary.in_place_overwrites, in_place_total, "{label} in_place_overwrites");
}

/// 臂 N19C 主几何五条路径的三态（登记 7.4 C1–C5、D4）：取三态的只有系统配置槽写，mkfs 那四次写之前镜像是空的。
#[test]
fn three_state_rows_match_the_registered_anchors() {
    let BuiltPool { recording, mkfs_operation_count, acquisition_operation_count, warm_up_operation_count, .. } = built_pool();
    let anchors: [(usize, &str, &str, u64, u64); 5] = [
        (21, "0+0+0+0+0", "none", 16402, 16402),
        (2, "2", "[system_configuration_slot×2]", 4, 9),
        (10, "0+0+2+0+0+2", "[system_configuration_slot×4]", 15, 25),
        (29, "0+0+0+2", "[system_configuration_slot×2]", 16_777_223, 16_777_228),
        (41, "2+0+0+2+0+0+2+0+0+0+2", "[system_configuration_slot×8]", 16_777_240, 16_777_260),
    ];
    let paths = new_pool_file_creation_stream_paths(&recording.operations, mkfs_operation_count, acquisition_operation_count, warm_up_operation_count);
    for ((path_name, earlier_operations, operations), anchor) in paths.into_iter().zip(anchors) {
        assert_three_state(&three_state_summary(earlier_operations, operations), anchor, path_name);
    }
}

/// 取样点 G11（⑧ 关，就是臂 N19，登记 R44）：五条路径逐格对登记 7.1 A1–A4 与 7.2 B1–B5。
#[test]
fn trailing_barrier_off_rows_match_the_anchors_of_the_stream_without_the_trailing_barrier() {
    let stream = record_new_pool_file_creation_stream(&PoolParameters::settled_two_devices_without_trailing_publish_barrier(), &sample_file());
    let expected: [(&str, usize, &str, u64, u64, &str, &str); 5] = [
        ("mkfs", 25, "14+1+1+1+4", 16402, 16402, "0+0+0+0+0", "[zero_fill×10,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]"),
        ("instance_acquisition", 2, "2", 4, 9, "2", "[system_configuration_slot×2]"),
        ("warm_up", 18, "2+1+2+2+1+2", 15, 25, "0+0+2+0+0+2", "[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]"),
        ("transaction", 33, "24+2+1+2", 16_777_223, 16_777_228, "0+0+0+2", "[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]"),
        (
            "post_mkfs_stream",
            53,
            "2+2+1+2+2+1+26+2+1+2",
            67_108_885,
            150_994_980,
            "2+0+0+2+0+0+2+0+0+2",
            "[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[unit_write×24,system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]",
        ),
    ];
    let paths = new_pool_file_creation_stream_paths(&stream.operations, stream.mkfs_operation_count, stream.acquisition_operation_count, stream.warm_up_operation_count);
    for ((path_name, earlier_operations, operations), (expected_path, operation_count, sizes, two_state, three_state, in_place, kinds)) in paths.into_iter().zip(expected) {
        assert_eq!(path_name, expected_path);
        assert_eq!(
            segments_sensitivity_line("point=G11 trailing_barrier=off", path_name, earlier_operations, operations),
            format!("name=segments_sensitivity point=G11 trailing_barrier=off path={path_name} operations={operation_count} segments={sizes} closed_form={two_state} closed_form_three_state={three_state} in_place_per_segment={in_place} kinds={kinds}"),
        );
    }
}

/// 取样点 G10a / G10b（⑧ 开的主几何）与 G12a / G12b（⑧ 关）：吞掉新池新建文件「记录 → 根槽」之间一块盘那一步屏障。
/// 绝对值取登记 7.4 C6–C9、D6〔G10〕与 7.2 B6–B9、7.1 A6〔G12〕。
#[test]
fn swallowed_barrier_points_match_the_registered_anchors() {
    type Anchor = (&'static str, u32, &'static str, usize, &'static str, u64, u64, &'static str, &'static str);
    let with_trailing = built_pool();
    let without_trailing = record_new_pool_file_creation_stream(&PoolParameters::settled_two_devices_without_trailing_publish_barrier(), &sample_file());
    let with_trailing_counts = (with_trailing.mkfs_operation_count, with_trailing.warm_up_operation_count);
    let without_trailing_counts = (without_trailing.mkfs_operation_count, without_trailing.warm_up_operation_count);
    let anchors: [(&[RecordedOperation], (usize, usize), Anchor); 8] = [
        (&with_trailing.recording.operations, with_trailing_counts, ("point=G10a swallowed_device=1", 1, "transaction", 34, "24+5", 16_777_247, 16_777_287, "0+2", "[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,system_configuration_slot×2,barrier×3]")),
        (&with_trailing.recording.operations, with_trailing_counts, ("point=G10a swallowed_device=1", 1, "post_mkfs_stream", 56, "2+2+1+2+2+1+2+24+5", 16_777_264, 16_777_319, "2+0+0+2+0+0+2+0+2", "[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,system_configuration_slot×2,barrier×3]")),
        (&with_trailing.recording.operations, with_trailing_counts, ("point=G10b swallowed_device=0", 0, "transaction", 34, "24+3+2", 16_777_226, 16_777_231, "0+0+2", "[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,barrier]|[system_configuration_slot×2,barrier×2]")),
        (&with_trailing.recording.operations, with_trailing_counts, ("point=G10b swallowed_device=0", 0, "post_mkfs_stream", 56, "2+2+1+2+2+1+2+24+3+2", 16_777_243, 16_777_263, "2+0+0+2+0+0+2+0+0+2", "[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,barrier]|[system_configuration_slot×2,barrier×2]")),
        (&without_trailing.operations, without_trailing_counts, ("point=G12a trailing_barrier=off swallowed_device=1", 1, "transaction", 32, "24+5", 16_777_247, 16_777_287, "0+2", "[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,system_configuration_slot×2,barrier]")),
        (&without_trailing.operations, without_trailing_counts, ("point=G12a trailing_barrier=off swallowed_device=1", 1, "post_mkfs_stream", 52, "2+2+1+2+2+1+26+5", 67_108_909, 150_995_039, "2+0+0+2+0+0+2+2", "[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[unit_write×24,system_configuration_slot×2,barrier×2]|[journal_record×2,root_record_fua,system_configuration_slot×2,barrier]")),
        (&without_trailing.operations, without_trailing_counts, ("point=G12b trailing_barrier=off swallowed_device=0", 0, "transaction", 32, "24+3+2", 16_777_226, 16_777_231, "0+0+2", "[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,barrier]|[system_configuration_slot×2]")),
        (&without_trailing.operations, without_trailing_counts, ("point=G12b trailing_barrier=off swallowed_device=0", 0, "post_mkfs_stream", 52, "2+2+1+2+2+1+26+3+2", 67_108_888, 150_994_983, "2+0+0+2+0+0+2+0+2", "[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[unit_write×24,system_configuration_slot×2,barrier×2]|[journal_record×2,root_record_fua,barrier]|[system_configuration_slot×2]")),
    ];
    for (operations, (mkfs_operation_count, warm_up_operation_count), (point_fields, expected_swallowed_device, path_name, operation_count, sizes, two_state, three_state, in_place, kinds)) in anchors {
        // 吞哪块盘取 `main()` 用的那两张表（变异 M193、M201 改的就是它们），期望值是登记里的字面。
        let point_name = point_fields.split_whitespace().next().and_then(|field| field.strip_prefix("point=")).expect("point= 打头");
        let (_, swallowed_device) = SWALLOWED_BARRIER_POINTS_WITH_TRAILING_BARRIER
            .iter()
            .chain(SWALLOWED_BARRIER_POINTS_WITHOUT_TRAILING_BARRIER.iter())
            .find(|(name, _)| *name == point_name)
            .copied()
            .expect("登记过的取样点");
        assert_eq!(swallowed_device, DeviceIdentity(expected_swallowed_device), "{point_name} 吞的那块盘");
        let swallowed_transaction = swallow_one_device_barrier_step_before_the_root(&operations[warm_up_operation_count..], swallowed_device);
        let (earlier_operations, path_operations): (&[RecordedOperation], Vec<RecordedOperation>) = match path_name {
            "transaction" => (&operations[..warm_up_operation_count], swallowed_transaction),
            _ => (&operations[..mkfs_operation_count], operations[mkfs_operation_count..warm_up_operation_count].iter().cloned().chain(swallowed_transaction).collect()),
        };
        assert_eq!(
            segments_sensitivity_line(point_fields, path_name, earlier_operations, &path_operations),
            format!("name=segments_sensitivity {point_fields} path={path_name} operations={operation_count} segments={sizes} closed_form={two_state} closed_form_three_state={three_state} in_place_per_segment={in_place} kinds={kinds}"),
        );
    }
}

/// E142 第二十五次跑臂 N25M 的 M④ (i)：D22（单元原子性怎么合成） 已定项 8 第 3 条「每块盘上 [0, 根环基址)、根环三个区域与
/// journal 环各一次调用……每块盘 5 步、两块盘共 10 步」——主几何两块盘各恰 5 步整段清零，其中恰一步落在偏移 0
/// （[0, 根环基址) 那一段）；mkfs 之后的流里不再有整段清零（变异 M232、M233）。
#[test]
fn mkfs_zero_fills_five_areas_on_each_device_and_exactly_one_of_them_starts_at_offset_zero() {
    let BuiltPool { recording, mkfs_operation_count, .. } = built_pool();
    let zero_fills_in = |operations: &[RecordedOperation]| -> Vec<(DeviceIdentity, DeviceOffset)> {
        operations
            .iter()
            .filter_map(|operation| match operation {
                RecordedOperation::Write(write) if write.kind == StepKind::ZeroFill => Some((write.device, write.offset)),
                RecordedOperation::Write(_) | RecordedOperation::Barrier(_) => None,
            })
            .collect()
    };
    let mkfs_zero_fills = zero_fills_in(&recording.operations[..mkfs_operation_count]);
    assert_eq!(mkfs_zero_fills.len(), 10, "两块盘共 10 步整段清零");
    for device in PoolParameters::settled_two_devices().devices() {
        let offsets_on_device: Vec<DeviceOffset> = mkfs_zero_fills.iter().filter(|(zero_fill_device, _)| *zero_fill_device == device).map(|(_, offset)| *offset).collect();
        assert_eq!(offsets_on_device.len(), 5, "设备 {} 上恰 5 步整段清零", device.0);
        assert_eq!(offsets_on_device.iter().filter(|offset| **offset == DeviceOffset(0)).count(), 1, "设备 {} 上恰一步落在偏移 0（[0, 根环基址)）", device.0);
    }
    assert!(zero_fills_in(&recording.operations[mkfs_operation_count..]).is_empty(), "整段清零只在 mkfs 里发");
}

/// E142 第二十五次跑臂 N25M 的 M④ (ii)：取样点 G1B（一块盘、带屏障）那一行的四个字段钉登记 7.2 A-M2
/// （第一段 5 次清零 + 2 个单元写 = 7 写、1 步屏障；三次根 FUA 各一段；两个系统配置槽 + 1 步屏障）。A-M2 是把「每块盘 5 步」
/// 搬到一块盘上推的，条款的射程是两块盘的第一版。
#[test]
fn mkfs_on_one_device_with_barriers_matches_five_zero_fills_on_that_device() {
    assert_eq!(
        segments_geometry_line_of_mkfs_on_one_device_with_barriers(),
        "name=segments_geometry point=G1B path=mkfs operations=14 segments=7+1+1+1+2 closed_form=134 kinds=[zero_fill×5,unit_write×2,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×2,barrier]"
    );
}

/// E142 第二十五次跑臂 N25M 的 M④ (iii)：取样点 G11（不发发布末尾那道屏障）的 `path=mkfs` 那一行，`operations=`、`segments=`、
/// `closed_form=`、`kinds=` 四个字段与主几何 `name=segments path=mkfs` 那一行逐个相同——mkfs 不随发布这一侧的屏障旋钮变。
/// 主几何那一行的绝对值另由 `registered_segment_sequences_match_every_recorded_path` 钉住。
#[test]
fn mkfs_segment_fields_without_the_trailing_barrier_equal_the_main_geometry() {
    let BuiltPool { recording, mkfs_operation_count, .. } = built_pool();
    let main_mkfs_operations = &recording.operations[..mkfs_operation_count];
    let (_, main_segments) = split_into_segments(main_mkfs_operations, true);
    let main_fields = [
        format!("operations={}", main_mkfs_operations.len()),
        format!("segments={}", main_segments.iter().map(|segment| segment.len().to_string()).collect::<Vec<_>>().join("+")),
        format!("closed_form={}", closed_form_state_count(&main_segments)),
        format!("kinds={}", format_segment_kinds(&segment_step_kinds(main_mkfs_operations, true))),
    ];
    assert_eq!(main_fields[0], "operations=25");
    let stream_without_trailing_barrier = record_new_pool_file_creation_stream(&PoolParameters::settled_two_devices_without_trailing_publish_barrier(), &sample_file());
    let [(path_name, earlier_operations, operations), ..] =
        new_pool_file_creation_stream_paths(&stream_without_trailing_barrier.operations, stream_without_trailing_barrier.mkfs_operation_count, stream_without_trailing_barrier.acquisition_operation_count, stream_without_trailing_barrier.warm_up_operation_count);
    assert_eq!(path_name, "mkfs");
    let line_without_trailing_barrier = segments_sensitivity_line("point=G11 trailing_barrier=off", path_name, earlier_operations, operations);
    for field in &main_fields {
        let field_name = field.split('=').next().expect("字段名");
        let field_without_trailing_barrier = line_without_trailing_barrier.split(' ').find(|token| token.split('=').next() == Some(field_name)).expect("G11 那一行有这个字段");
        assert_eq!(field_without_trailing_barrier, field, "G11 的 path=mkfs 那一行 {field_name} 与主几何不同");
    }
}

/// 窗口之前与窗口的计数（登记 7.4 C10、C11、D5）与判决格 `write_list_ok`（⑤ ⑨；变异 M192、M198）。
#[test]
fn before_window_and_window_counts_match_the_registered_anchors() {
    let BuiltPool { recording, warm_up_operation_count, .. } = built_pool();
    let before_window = &recording.operations[..warm_up_operation_count];
    let window = &recording.operations[warm_up_operation_count..];
    let count_writes = |operations: &[RecordedOperation]| operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(_))).count();
    let count_barrier_steps = |operations: &[RecordedOperation]| operations.iter().filter(|operation| matches!(operation, RecordedOperation::Barrier(_))).count();
    let count_fua = |operations: &[RecordedOperation]| operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(write) if write.is_fua())).count();
    assert_eq!((before_window.len(), count_writes(before_window)), (47, 33), "C10（R19E-4 第 4 条：暖机两道连着的屏障两块盘各并掉一步；E142 第二十五次跑登记 7.2 A-M4：mkfs 每块盘多一步写、没有多屏障）");
    assert_eq!((window.len(), count_writes(window), count_barrier_steps(window), count_fua(window)), (35, 29, 6, 1), "C11");
    assert!(new_pool_file_creation_write_list_matches(count_writes(window), count_barrier_steps(window), count_fua(window), 2));
    assert!(!new_pool_file_creation_write_list_matches(29, 4, 1, 2), "C577 之前的两道池屏障（每盘各一步）不算过");
    assert!(!new_pool_file_creation_write_list_matches(29, 2, 1, 2), "按池屏障数（一道一步）不算过");
    assert!(new_pool_file_creation_write_list_matches(29, 3, 1, 1), "一块盘时三道屏障各一步");
    assert!(!new_pool_file_creation_write_list_matches(28, 6, 1, 2) && !new_pool_file_creation_write_list_matches(29, 6, 2, 2));
}

/// 臂 N19 的 ③（登记 R39）：原地覆写三条都成立才算；合成的写流上逐条各缺一条（变异 M186、M187、M188 各落一个点）。
#[test]
fn in_place_overwrite_needs_all_three_conditions_on_synthetic_writes() {
    let write = |kind: StepKind, offset: u64, length: usize, byte: u8| RecordedOperation::Write(WriteRequest { device: DeviceIdentity(0), offset: DeviceOffset(offset), bytes: vec![byte; length], kind });
    let operations = vec![
        write(StepKind::SystemConfigurationSlot, 0, 4096, 0x5a), // 镜像是空的：不是
        write(StepKind::SystemConfigurationSlot, 0, 4096, 0x5b), // 同一路径里更早的写落过：是（M188）
        write(StepKind::JournalRecord, 0, 512, 0x5c),             // 只有一个扇区长：不是（M186）
        write(StepKind::UnitWrite, 0, 32768, 0x5d),               // 单元写：不是（M187）
        write(StepKind::RootRecordFua, 65536, 4096, 0x5e),        // 罩住的范围原来全是 0：不是
        write(StepKind::JournalRecord, 65536, 1024, 0x5f),        // 罩住上一次写：是
        RecordedOperation::Write(WriteRequest { device: DeviceIdentity(1), offset: DeviceOffset(0), bytes: Vec::new(), kind: StepKind::ZeroFill }),
        RecordedOperation::Write(WriteRequest { device: DeviceIdentity(1), offset: DeviceOffset(0), bytes: vec![0x60; 4096], kind: StepKind::SystemConfigurationSlot }), // 清零过的一段：不是
    ];
    assert_eq!(in_place_overwrite_flags(&[], &operations), vec![false, true, false, false, false, true, false, false]);
    assert_eq!(in_place_overwrite_flags(&operations[..1], &operations[1..2]), vec![true], "路径之前的写也算进镜像");
    // 八次写（两块盘，没有屏障，FUA 不当边界）同一段，其中两次原地覆写：1 + (3^2 · 2^6 − 1) = 576。
    let (_, segments) = split_into_segments(&operations, false);
    assert_eq!(segments.iter().map(Vec::len).collect::<Vec<_>>(), vec![8]);
    assert_eq!(closed_form_three_state_count(&segments, &in_place_overwrite_flags(&[], &operations)), 576);
    assert_eq!(closed_form_state_count(&segments), 256);
}

/// 臂 N19 的 ②（D13 已定项 4）：合成写流上逐条看放行规则（变异 M181–M184 各有一处落点）。
#[test]
fn segment_closing_rule_waits_for_every_device_with_writes() {
    let write = |device: u32, kind: StepKind| RecordedOperation::Write(WriteRequest { device: DeviceIdentity(device), offset: DeviceOffset(0), bytes: vec![1; 512], kind });
    let barrier = |device: u32| RecordedOperation::Barrier(DeviceIdentity(device));
    let sizes = |operations: &[RecordedOperation], fua_is_boundary: bool| split_into_segments(operations, fua_is_boundary).1.iter().map(Vec::len).collect::<Vec<_>>();
    // 两块盘有写，只放行一块盘不关段；有写的盘都放行了才关。
    let released_one_by_one = [
        write(0, StepKind::UnitWrite),
        write(1, StepKind::UnitWrite),
        barrier(0),
        write(0, StepKind::JournalRecord),
        barrier(1),
        barrier(0),
        write(1, StepKind::UnitWrite),
        barrier(1),
    ];
    assert_eq!(sizes(&released_one_by_one, true), vec![3, 1]);
    // FUA 只放行它自己那块盘：盘 1 还没放行时它不关段；只有它那块盘有写时它关段。
    let fua_on_one_device = [write(1, StepKind::JournalRecord), write(0, StepKind::RootRecordFua), barrier(1), write(0, StepKind::RootRecordFua), write(0, StepKind::SystemConfigurationSlot)];
    assert_eq!(sizes(&fua_on_one_device, true), vec![2, 1, 1]);
    assert_eq!(sizes(&fua_on_one_device, false), vec![4], "FUA 不当边界时与普通写一样，盘 0 一直没放行");
    // FUA 不放行别的盘：盘 1 的记录要等盘 1 的屏障。
    let fua_does_not_release_other_devices = [write(1, StepKind::JournalRecord), write(0, StepKind::RootRecordFua), write(0, StepKind::SystemConfigurationSlot), barrier(1), barrier(0)];
    assert_eq!(sizes(&fua_does_not_release_other_devices, true), vec![3]);
    // 段里没写时的屏障并进即将开始的一段；流尾只有屏障的一串并进上一段。
    let kinds = format_segment_kinds(&segment_step_kinds(&[barrier(0), barrier(1), write(0, StepKind::UnitWrite), barrier(0), barrier(1)], true));
    assert_eq!(kinds, "[unit_write,barrier×4]");
}

/// 枚举上限（`LARGEST_ENUMERATED_SEGMENT_WRITES`）：恰好 24 个写的一段放行；25 个写的一段当场断言失败，不进枚举。
/// 变异 M180（池屏障只记盘 0）、M184（屏障不放行它那块盘）下盘放不了行、整条流并成一个 41 个写的段，
/// 没有这条上限那段要枚举 2^41 个状态、测试跑不完；有了它，逐态枚举、分组枚举与多线程整轮枚举的测试在入口断言失败（变异 M246、M247、M248 守这三个入口）。
#[test]
fn segments_up_to_the_enumeration_limit_pass_and_longer_ones_are_refused() {
    assert_segments_are_enumerable(&[(0..LARGEST_ENUMERATED_SEGMENT_WRITES).collect()]);
    let result = std::panic::catch_unwind(|| assert_segments_are_enumerable(&[vec![0], (0..=LARGEST_ENUMERATED_SEGMENT_WRITES).collect()]));
    let message = *result.expect_err("超过上限的段必须断言失败").downcast::<String>().expect("断言信息是字符串");
    assert!(message.contains("段号 1 有 25 个写"), "{message}");
}

/// 逐态枚举入口先核段长：25 个写的一段在 `for_each_layer0_state` 里当场断言失败，访问回调一次都没调。
#[test]
#[should_panic(expected = "超过枚举上限 24")]
fn per_state_enumeration_refuses_a_segment_longer_than_the_limit() {
    let writes_length = LARGEST_ENUMERATED_SEGMENT_WRITES + 1;
    for_each_layer0_state(writes_length, &[(0..writes_length).collect()], |_| panic!("上限没拦住，枚举开始了"));
}

/// 分组枚举入口同样先核段长（它不经 `for_each_layer0_state`，按全局下标自己切）。根槽下标故意越界：
/// 上限没拦住时第一个状态的判定就在越界处 panic、信息对不上，不会真去枚举 2^25 个状态。
#[test]
#[should_panic(expected = "超过枚举上限 24")]
fn grouped_enumeration_refuses_a_segment_longer_than_the_limit() {
    let (base, _) = mkfs(&PoolParameters::control_one_device_no_barriers());
    let writes_length = LARGEST_ENUMERATED_SEGMENT_WRITES + 1;
    let writes = vec![WriteRequest { device: DeviceIdentity(0), offset: DeviceOffset(0), bytes: vec![1; 512], kind: StepKind::UnitWrite }; writes_length];
    enumerate_layer0_grouped(1, &base.pool, &writes, &[(0..writes_length).collect()], &sample_file(), ReaderMode::Clause, writes_length, &[0], &[vec![0]], &[]);
}

/// 整轮枚举入口同样先核段长：它经 `layer0_state_at_index` 按下标自己切，不经 `for_each_layer0_state`。写流里故意没有根槽那一条：
/// 上限没拦住时紧接着找根槽的断言就失败、信息对不上，不会真去枚举 2^25 个状态。
#[test]
#[should_panic(expected = "超过枚举上限 24")]
fn threaded_enumeration_refuses_a_segment_longer_than_the_limit() {
    let (base, _) = mkfs(&PoolParameters::control_one_device_no_barriers());
    let writes_length = LARGEST_ENUMERATED_SEGMENT_WRITES + 1;
    let writes = vec![WriteRequest { device: DeviceIdentity(0), offset: DeviceOffset(0), bytes: vec![1; 512], kind: StepKind::UnitWrite }; writes_length];
    enumerate_crash_states_with_threads(2, &base.pool, &writes, &[(0..writes_length).collect()], &sample_file(), ReaderMode::Clause);
}

/// 下标区间的切法：任意总数与线程数下，各段从 0 起首尾相接、按起点升序、都不空、段数不超过线程数、末段止于总数。
#[test]
fn index_ranges_cover_every_index_exactly_once_in_order() {
    for closed_form in [1u64, 2, 7, 100, 8193, 16_777_240] {
        for threads in [0usize, 1, 2, 3, 4, 31, 200] {
            let ranges = split_index_ranges(closed_form, threads);
            assert!(!ranges.is_empty() && ranges.len() <= threads.max(1), "总数 {closed_form}、线程 {threads}：{ranges:?}");
            assert_eq!(ranges.first().map(|range| range.start), Some(0), "总数 {closed_form}、线程 {threads}：{ranges:?}");
            assert_eq!(ranges.last().map(|range| range.end), Some(closed_form), "总数 {closed_form}、线程 {threads}：{ranges:?}");
            for range in &ranges {
                assert!(!range.is_empty(), "总数 {closed_form}、线程 {threads}：{ranges:?}");
            }
            for pair in ranges.windows(2) {
                assert_eq!(pair[0].end, pair[1].start, "总数 {closed_form}、线程 {threads}：{ranges:?}");
            }
        }
    }
}

/// 整轮枚举在阳性对照、屏障补回的 G6、FUA 当边界的 G7 三条小流上，线程数 1 与 4（再加不整除的 7）的 `Layer0Tally` 逐格相等，
/// 并且都等于逐态访问（`for_each_layer0_state`）直接累计出来的那一份。阳性对照至少有一处违例，好让 `first_violation` 取哪一处也被比到。
#[test]
fn crash_state_enumeration_agrees_between_one_and_four_threads_on_the_positive_control_barriers_restored_and_fua_boundary() {
    let flows: [(&str, PoolParameters, bool); 3] = [
        ("positive_control", PoolParameters::control_one_device_no_barriers(), false),
        ("barriers_restored", PoolParameters::one_device_with_barriers(), false),
        ("fua_boundary", PoolParameters::control_one_device_no_barriers(), true),
    ];
    for (label, parameters, fua_is_boundary) in flows {
        let flow = build_single_publish_flow(&parameters, &sample_file(), fua_is_boundary);
        let root_index = flow.writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("写流里有根槽那一条");
        let mut reference = Layer0Tally::default();
        for_each_layer0_state(flow.writes.len(), &flow.segments, |persisted| {
            evaluate_state(&flow.base, &flow.writes, persisted, root_index, &sample_file(), ReaderMode::Clause, &mut reference);
        });
        assert_eq!(reference.states, flow.closed_form, "{label}：逐态访问的状态数");
        let serial = enumerate_crash_states_with_threads(1, &flow.base, &flow.writes, &flow.segments, &sample_file(), ReaderMode::Clause);
        assert_eq!(serial, reference, "{label}：一个线程的整轮枚举与逐态累计必须逐格相等");
        for threads in [4, 7] {
            let parallel = enumerate_crash_states_with_threads(threads, &flow.base, &flow.writes, &flow.segments, &sample_file(), ReaderMode::Clause);
            assert_eq!(parallel, serial, "{label}：{threads} 个线程与一个线程的 Layer0Tally 必须逐格相等");
        }
        if label == "positive_control" {
            assert!(serial.violations > 0, "阳性对照要有违例");
            assert!(serial.first_violation.is_some(), "阳性对照要有第一处违例");
        }
    }
}
