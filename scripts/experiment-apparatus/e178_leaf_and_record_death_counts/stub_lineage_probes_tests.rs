use super::*;

/// 第九节修订 2 的 M17：H51 F1 n = 300 升序、容量 16、θ = 0、W = ∞，B-stub × Z × E-I 期末容器 r = 0、读到的桩 18 个（跳过桩就多 18）。
#[test]
fn stub_readers_select_the_stub_and_read_no_dead_container() {
    let history = History::BuildAndDeleteEmptyInodes { inode_count: 300, order: DeletionOrder::Ascending };
    let (simulation, _) = stub_path_simulation(history, write_path_cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 0), true);
    let readers = segment_four_stub_path_readers();
    let reader = position_in(&readers, reader_with(BaseArm::ZeroRecordStubs, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever));
    let last = simulation.scans.last().expect("有扫描").counts[reader];
    assert_eq!((last.container_resurrected, last.container_missed, last.stubs_read), (0, 0, 17 + 1));
    assert!(simulation.scans.iter().zip(&simulation.publish_records).all(|(scan, record)| scan.counts[reader].stubs_read == record.stubs_in_the_leaf_order));
}

/// A12 与第九节修订 2 的 M19：同一取样点上 B-tsz × Z × E-I 期末容器 r = 18（= B-off-ret，A8）、inode r = 0（不施行整对象墓碑就多 18）。
#[test]
fn tombstones_only_reads_every_dead_container_and_drops_its_deleted_records() {
    let history = History::BuildAndDeleteEmptyInodes { inode_count: 300, order: DeletionOrder::Ascending };
    let simulation = original_path_simulation(history, write_path_cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 0), true);
    let readers = segment_four_original_path_readers();
    let tombstones_only = position_in(&readers, reader_with(BaseArm::TombstonesOnly, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever));
    let without_retirement = position_in(&readers, reader_with(BaseArm::WithoutRetirement, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever));
    let last = simulation.scans.last().expect("有扫描");
    assert_eq!(last.counts[tombstones_only].container_resurrected, 17 + 1);
    assert_eq!(last.counts[tombstones_only].inode_resurrected, 0);
    assert!(simulation.scans.iter().all(|scan| scan.counts[tombstones_only].container_resurrected == scan.counts[without_retirement].container_resurrected));
    // PC-Bt1：删空那 18 片的最后一版装着最后删掉的那条（nlink = 0）；拿掉它的整对象墓碑，B-tsz inode r + 1、B-on + 0。
    for window in READABLE_WINDOWS {
        let controls = simulation.dead_record_positive_control(simulation.publish_sequence, window);
        assert_eq!(controls.len(), 1, "{window:?}");
        assert!(controls[0].paths_agree && controls[0].closed_form_holds, "{}", controls[0].line(""));
    }
}

/// 第九节修订 2 的 M16：F11 吸收缺口（容量 16、θ = 8、N1），W = ∞，B-stub × Z × E-I 从那次吸收到回退之前的扫描里容器 r 都是 0
/// （吸收时不落桩，被吸收那片的最后一版就被读回）。
#[test]
fn an_absorbed_leaf_leaves_a_stub_that_hides_its_last_version() {
    let (simulation, outcome) = stub_path_simulation(History::RollBackOverAGap { event: GapEvent::Absorb }, write_path_cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 8), true);
    let outcome = outcome.expect("F11 交回回退的结局");
    let gap_start = simulation.gap_start.clone().expect("吸收缺口出现了");
    let readers = segment_four_stub_path_readers();
    let reader = position_in(&readers, reader_with(BaseArm::ZeroRecordStubs, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever));
    let in_the_gap: Vec<&ScanRecord> = simulation
        .scans
        .iter()
        .filter(|scan| scan.publish_sequence >= gap_start.event_publish_sequence && scan.publish_sequence < outcome.rollback_publish_sequence)
        .collect();
    assert!(!in_the_gap.is_empty());
    assert!(in_the_gap.iter().all(|scan| scan.counts[reader].container_resurrected == 0));
    assert!(in_the_gap.iter().any(|scan| scan.counts[reader].stubs_read > 0));
}

/// PC-Bt2、PC-Bt2-mid（登记修订 3）与第九节修订 2 的 M18：F10、N2、容量 16、θ = 4、W = ∞。期末那次扫描（原 PC-Bt2）手算值是 0
/// （F10 删光 2..n）；最后一次「有被挪过、之后变短的活对象」那次扫描上，倒置供出规则之后 B-tsz × Z 的 extent r 增量（手算）> 0，
/// 而 P-C 下那次扫描 B-tsz × Z × E-I 的 extent r 是 0（供出规则取写序最小的副本就变成那个增量）。
#[test]
fn reversed_provenance_resurrects_the_tail_of_a_moved_then_shortened_object() {
    let write_path = write_path_cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 4);
    let readers = segment_four_original_path_readers();
    let tombstones_only = position_in(&readers, reader_with(BaseArm::TombstonesOnly, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever));
    let hand_of = |control: &PositiveControlOutcome| -> i64 { control.detail.trim_start_matches("hand_extent_r_z=").parse().expect("手算值是数") };
    let mut found = false;
    for order in deletion_orders() {
        let history = History::ManyObjectsWithData { inode_count: 300, order, batch: PRODUCT_UNLINK_BATCH_DATA_UNITS };
        let run = run_segment_four_path(history, write_path, ContainerDeathWritePath::RetirementRecords, &readers);
        let at_the_end = run.positive_controls.iter().find(|control| control.control_identifier == "PC-Bt2").expect("期末那次做了 PC-Bt2");
        assert!(at_the_end.paths_agree && at_the_end.closed_form_holds && hand_of(at_the_end) == 0, "{}", at_the_end.line(""));
        let Some(mid) = run.positive_controls.iter().find(|control| control.control_identifier == "PC-Bt2-mid") else {
            continue;
        };
        assert!(mid.paths_agree && mid.closed_form_holds, "{}", mid.line(""));
        if hand_of(mid) > 0 {
            let scan = run.simulation.scans.iter().find(|scan| scan.publish_sequence == mid.at_publish).expect("那次扫描在");
            assert_eq!(scan.counts[tombstones_only].extent_resurrected, 0, "{order:?}");
            found = true;
            break;
        }
    }
    assert!(found, "F10 n = 300、N2、容量 16、θ = 4、批 144 的七段里没有一段 PC-Bt2-mid 手算值 > 0");
}

/// 第二段 PC-B1 在三段 F11 上不成立（产物里那三行 `holds=false`）的成因：消失过的身份里期末没被 B-off-ret × Z × E-I × ∞ 读回的，
/// 盘上每一版的诞生 txg 都不在期末那一版的父链上（在缺口里生、又在缺口里消失）；登记 PC-B1 的期望数的是消失过的身份，没扣这一类。
/// (消失过的身份数, 期末容器 r) 照第二段产物那三行钉：(4, 3)、(6, 4)、(5, 4)。
#[test]
fn identities_born_and_retired_inside_the_abandoned_gap_are_what_the_gap_positive_control_overcounts() {
    let without_retirement_forever = position_of(BaseArm::WithoutRetirement, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever);
    for (event, threshold, expected) in [(GapEvent::Borrow, 4, (4, 3)), (GapEvent::Borrow, 8, (6, 4)), (GapEvent::MovedObjectShortened, 8, (5, 4))] {
        let (simulation, _) = simulate(History::RollBackOverAGap { event }, write_path_cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, threshold), false);
        let final_identities: HashSet<ContainerIdentity> = simulation.version.leaves.iter().map(|leaf| leaf.identity).collect();
        let disappeared: BTreeSet<ContainerIdentity> = simulation
            .publish_records
            .iter()
            .flat_map(|record| record.retired_identities.iter().copied())
            .filter(|identity| !final_identities.contains(identity))
            .collect();
        let chain = ParentChain::of(simulation.version.txg, &simulation.version.lineage);
        let born_off_the_chain_only = disappeared
            .iter()
            .filter(|identity| {
                simulation.disk.units.iter().all(|unit| match &unit.body {
                    UnitBody::InodeLeaf { identity: unit_identity, .. } => unit_identity != *identity || !chain.contains(unit.write_order.birth_txg),
                    UnitBody::Data { .. } | UnitBody::Tombstones { .. } | UnitBody::Retirements { .. } => true,
                })
            })
            .count();
        let counts = simulation.read_every_reader(&Overlay::default(), simulation.publish_sequence, SelectionPath::GroupIndex);
        let final_container_resurrected = counts[without_retirement_forever].container_resurrected;
        assert_eq!((length_as_u64(disappeared.len()), final_container_resurrected), expected, "{event:?} θ {threshold}");
        assert!(born_off_the_chain_only > 0, "{event:?} θ {threshold}");
        assert_eq!(final_container_resurrected + length_as_u64(born_off_the_chain_only), length_as_u64(disappeared.len()), "{event:?} θ {threshold}");
    }
}
