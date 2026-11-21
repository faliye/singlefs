use super::*;

/// 删除态扫描里一条 nlink 1、size `size_in_bytes` 的记录（C871 那几条用例用）。
fn live_record_of_the_deletion_scan(size_in_bytes: u64) -> InodeRecordOfTheDeletionScan {
    InodeRecordOfTheDeletionScan {
        object_birth: 4,
        nlink: 1,
        size_in_bytes,
        truncation_field_txg: 0,
        truncation_field_size_in_bytes: 0,
    }
}

/// C871（记录 inode 号等于 key 没有 checker 判定）里 I-9.8 的那张镜像：一片叶里 inode 20、30、40
/// 各一条记录、各一个数据单元（size 5000），只把 30 那条记录偏移 0 的号改成 35（路由区间之内，I-9.4、I-9.12 不红）。
/// extent 树里 30 的单元没有 nlink ≥ 1 的记录 ⇒ 30 算删除中、不在队列里、单元还在 ⇒ I-4.7 红；记录 35 size 5000 而没有上段条目 ⇒ I-4.6 红。
/// 没改号的那一份两条都不红。
#[test]
fn renumbering_a_live_record_with_a_data_unit_inside_its_routing_range_reddens_the_inode_being_deleted_and_the_unit_count_invariants(
) {
    let data_units_of_each_inode = BTreeMap::from([(20u64, 1u64), (30, 1), (40, 1)]);
    let records_as_written = BTreeMap::from([
        (20u64, live_record_of_the_deletion_scan(5000)),
        (30, live_record_of_the_deletion_scan(5000)),
        (40, live_record_of_the_deletion_scan(5000)),
    ]);
    let records_renumbered = BTreeMap::from([
        (20u64, live_record_of_the_deletion_scan(5000)),
        (35, live_record_of_the_deletion_scan(5000)),
        (40, live_record_of_the_deletion_scan(5000)),
    ]);
    let violations_of_the_inode_being_deleted_invariant =
        |records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>| {
            let mut judgements = Judgements::default();
            judge_inodes_being_deleted_against_the_queue_and_their_data_units(
                "测试版",
                &BTreeSet::new(),
                records,
                &data_units_of_each_inode,
                &mut judgements,
            );
            judgements.violation_count(INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT)
        };
    assert_eq!(
        violations_of_the_inode_being_deleted_invariant(&records_as_written),
        0,
        "没改号：三个号都有 nlink 1 的记录，没有删除中的号"
    );
    assert_eq!(
            violations_of_the_inode_being_deleted_invariant(&records_renumbered),
            1,
            "30 改号成 35：extent 树里 30 的单元没有 nlink ≥ 1 的记录、号不在队列里、单元还在，I-4.7 红一处"
        );
    let one_data_unit_without_a_hole = || DataUnitsOfAnInodeOfTheDeletionScan {
        declared_by_the_upper_leaf_entry: Some(
            DataUnitsDeclaredByTheUpperLeafEntry::WithDataUnits(TailAfterTheLastDataUnit::NoHole),
        ),
        data_pointers_by_offset: vec![(0, parse_node_pointer(&[0u8; 86]))],
    };
    let units_of_each_inode = BTreeMap::from([
        (20u64, one_data_unit_without_a_hole()),
        (30, one_data_unit_without_a_hole()),
        (40, one_data_unit_without_a_hole()),
    ]);
    let violations_of_the_unit_count_invariant =
        |records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>| {
            let mut judgements = Judgements::default();
            judge_data_unit_counts_of_the_inodes_against_their_size(
                "测试版",
                records,
                &units_of_each_inode,
                &[],
                &mut judgements,
            );
            judgements.violation_count(DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE)
        };
    assert_eq!(
        violations_of_the_unit_count_invariant(&records_as_written),
        0,
        "没改号：每条记录 size 5000 对一个单元"
    );
    assert_eq!(
        violations_of_the_unit_count_invariant(&records_renumbered),
        1,
        "改号之后记录 35 size 5000 而 extent 树里没有它的条目，I-4.6 红一处"
    );
}

/// I-4.6 的单元数对 size 那一句里一条 size 为 `size_in_bytes`、nlink 1 的记录，上段里怎么声明交 `declared`、没有单元，判一次交回违例数。
fn violations_of_the_unit_count_of_a_record_without_data_units(
    size_in_bytes: u64,
    declared: Option<DataUnitsDeclaredByTheUpperLeafEntry>,
) -> u64 {
    let records = BTreeMap::from([(
        2,
        InodeRecordOfTheDeletionScan {
            object_birth: 4,
            nlink: 1,
            size_in_bytes,
            truncation_field_txg: 0,
            truncation_field_size_in_bytes: 0,
        },
    )]);
    let data_units_of_each_inode = match declared {
        Some(declared_by_the_upper_leaf_entry) => BTreeMap::from([(
            2,
            DataUnitsOfAnInodeOfTheDeletionScan {
                declared_by_the_upper_leaf_entry: Some(declared_by_the_upper_leaf_entry),
                data_pointers_by_offset: Vec::new(),
            },
        )]),
        None => BTreeMap::new(),
    };
    let mut judgements = Judgements::default();
    judge_data_unit_counts_of_the_inodes_against_their_size(
        "测试版",
        &records,
        &data_units_of_each_inode,
        &[],
        &mut judgements,
    );
    judgements.violation_count(DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE)
}

///  与第三节第 4 句：「有记录、没有条目、size > 0」I-4.6 红，
/// 没有条目而 size 0（`publish_new_inodes` 建出来、没改过 size 的）与标签 0 配 size > 0（粘住的标签 0：整个文件是洞）都不红。
#[test]
fn record_without_an_upper_leaf_entry_reddens_the_unit_count_only_when_its_size_is_not_zero() {
    assert_eq!(
        violations_of_the_unit_count_of_a_record_without_data_units(5000, None),
        1,
        "inode 2 size 5000、上段里没有它的条目：读者拒的那一形，I-4.6 要红"
    );
    assert_eq!(
        violations_of_the_unit_count_of_a_record_without_data_units(0, None),
        0,
        "inode 2 size 0、没有条目：建出来没改过 size 的空 inode，不红"
    );
    assert_eq!(
        violations_of_the_unit_count_of_a_record_without_data_units(
            5000,
            Some(DataUnitsDeclaredByTheUpperLeafEntry::NoDataUnit)
        ),
        0,
        "inode 2 size 5000、标签 0：整个文件是洞，不红"
    );
}

/// I-9.40 的规范形读到的样子：S = {1, 500}（产品几何：500 落在上段第 3 片叶、要层级 1）。
fn upper_shape_of_inodes_one_and_five_hundred(
    root_level: u8,
    positions: &[(u8, u64)],
) -> ExtentUpperSegmentShapeRead {
    ExtentUpperSegmentShapeRead {
        root_level,
        positions: positions.iter().copied().collect(),
        inodes_with_an_entry: BTreeSet::from([1, 500]),
    }
}

/// I-9.40 的规范形本身成立：S = {1, 500} 是 (0, 0)、(0, 3)、(1, 0)，根层级 1；S 空是 (0, 0) 一个、层级 0。
#[test]
fn the_canonical_upper_shape_of_the_inodes_with_an_entry_holds() {
    assert_eq!(
        departure_of_the_extent_upper_segment_from_its_canonical_shape(
            &upper_shape_of_inodes_one_and_five_hundred(1, &[(0, 0), (0, 3), (1, 0)])
        ),
        None
    );
    assert_eq!(
        departure_of_the_extent_upper_segment_from_its_canonical_shape(
            &ExtentUpperSegmentShapeRead {
                root_level: 0,
                positions: BTreeSet::from([(0, 0)]),
                inodes_with_an_entry: BTreeSet::new(),
            }
        ),
        None,
        "删空的一版：层级 0 的零条目根兼叶"
    );
}

/// I-9.40 必红世界：多一个脊线节点（层级 2 的根，只升不降那种形状）。
#[test]
fn an_extra_spine_node_above_the_canonical_root_departs_from_the_canonical_upper_shape() {
    assert!(
        departure_of_the_extent_upper_segment_from_its_canonical_shape(
            &upper_shape_of_inodes_one_and_five_hundred(2, &[(0, 0), (0, 3), (1, 0), (2, 0)])
        )
        .is_some()
    );
}

/// I-9.40 必红世界：根层级高一层（只剩 inode 1 时根留在层级 1，第 0 片叶挂在它下面）。
#[test]
fn root_one_level_higher_than_needed_departs_from_the_canonical_upper_shape() {
    assert!(
        departure_of_the_extent_upper_segment_from_its_canonical_shape(
            &ExtentUpperSegmentShapeRead {
                root_level: 1,
                positions: BTreeSet::from([(0, 0), (1, 0)]),
                inodes_with_an_entry: BTreeSet::from([1]),
            }
        )
        .is_some()
    );
}

/// I-9.40 必红世界：少一个祖先（S = {1, 30000} 要层级 2，30000 那片叶 (0, 209) 与根 (2, 0) 都在，它们之间那一层的 (1, 1) 缺了）。
#[test]
fn missing_ancestor_departs_from_the_canonical_upper_shape() {
    assert!(
        departure_of_the_extent_upper_segment_from_its_canonical_shape(
            &ExtentUpperSegmentShapeRead {
                root_level: 2,
                positions: BTreeSet::from([(0, 0), (0, 209), (1, 0), (2, 0)]),
                inodes_with_an_entry: BTreeSet::from([1, 30000]),
            }
        )
        .is_some()
    );
}
