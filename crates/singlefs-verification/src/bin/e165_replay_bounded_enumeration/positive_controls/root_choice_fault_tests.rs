use super::*;

/// C8a 的钉值（登记 r5 5.4、第七节乙类）：`even` 下 L 域 N ≤ 5 上 J8a 三格 762、762、620，J8b、J8c 0，r1 档那种生成器上三格 0；
/// `odd` 下 L 域同值（第 j 版与第 0 版的记号不同 ⟺ j 为奇数，两种奇偶都如此），C 域 Nc ≤ 3 两条指针那两格大于 0。
/// 9.2「r5 档时间线记录的后三项写回 r1 常量」点名这一条。
#[test]
fn the_chosen_record_fields_control_reads_762_762_620_on_the_root_fields_in_records_generator_and_0_on_the_constant_fields_generator(
) {
    assert_eq!(
        FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
        FARTHEST_IS_ODD_STATES_WITHIN_THE_CONTROL_BOUND
            + FARTHEST_IS_EVEN_AND_AT_LEAST_TWO_STATES_WITHIN_THE_CONTROL_BOUND,
        "登记 r5 第七节乙类：620 + 142 = 762"
    );
    for marker_parity in [
        MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
        MarkerParity::OddVersionsWrittenByTheUnmountSequence,
    ] {
        let readings = control_readings_of(
            Criterion::J8aFieldsFromTheNewRootSegment,
            RunProfile::root_fields_in_records(marker_parity),
            1,
        );
        assert_eq!(
            readings
                .iter()
                .take(8)
                .map(|reading| reading.value)
                .collect::<Vec<u64>>(),
            vec![762, 762, 620, 0, 0, 0, 0, 0],
            "C8a（{}）的对照读数：{readings:?}",
            marker_parity.label()
        );
        assert!(
            readings.iter().all(|reading| reading.is_red_as_required),
            "C8a（{}）的对照都照登记红：{readings:?}",
            marker_parity.label()
        );
    }
}

/// C8b 的钉值与第八节奇偶的判别力自证（登记 r5 5.4、第七节乙类、第八节）：`even` 142、`odd` 620，r1 档那种生成器上 0；把门槛挪到
/// 142 与 620 之间（判「卸载记号那一格 ≥ 400」），`even` 那一点由绿转红、`odd` 那一点不动。9.2「C8b 的卸载记号写 Written（写反）」点名这一条。
#[test]
fn the_c8b_control_reads_142_on_even_and_620_on_odd_and_a_threshold_of_400_parts_the_two_parities()
{
    let readings_on = |marker_parity: MarkerParity| {
        control_readings_of(
            Criterion::J8aUnmountMarkerFromTheNewRootSegment,
            RunProfile::root_fields_in_records(marker_parity),
            1,
        )
    };
    let even = readings_on(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    let odd = readings_on(MarkerParity::OddVersionsWrittenByTheUnmountSequence);
    assert_eq!(
        even.iter()
            .map(|reading| reading.value)
            .collect::<Vec<u64>>(),
        vec![142, 0],
        "C8b even：{even:?}"
    );
    assert_eq!(
        odd.iter()
            .map(|reading| reading.value)
            .collect::<Vec<u64>>(),
        vec![620, 0],
        "C8b odd：{odd:?}"
    );
    assert!(
        even.iter()
            .chain(&odd)
            .all(|reading| reading.is_red_as_required),
        "C8b 两种奇偶都照登记红"
    );
    let threshold_between_the_two_points = 400;
    assert!(
        even[0].value < threshold_between_the_two_points
            && odd[0].value >= threshold_between_the_two_points,
        "判别力自证：门槛 400 上 even 那一点红、odd 那一点不动（{} 与 {}）",
        even[0].value,
        odd[0].value
    );
}

/// C8c 的钉值（登记 r5 5.4、第七节乙类）：L 域 N ≤ 5 上 J8b 762、J8a 三格 0，r1 档那种生成器上 J8b 0；`odd` 下 C 域 Nc ≤ 3 上 J8b 大于 0。
/// 9.2「J8b 拿落到的那一版的 fsid 当期望」点名这一条。
#[test]
fn the_landed_filesystem_identifier_control_reads_762_on_j8b_and_0_on_the_three_j8a_cells_and_on_the_constant_fields_generator(
) {
    for marker_parity in [
        MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
        MarkerParity::OddVersionsWrittenByTheUnmountSequence,
    ] {
        let readings = control_readings_of(
            Criterion::J8bFilesystemIdentifierOfTheChosenRoot,
            RunProfile::root_fields_in_records(marker_parity),
            1,
        );
        assert_eq!(
            readings
                .iter()
                .take(5)
                .map(|reading| reading.value)
                .collect::<Vec<u64>>(),
            vec![762, 0, 0, 0, 0],
            "C8c（{}）：{readings:?}",
            marker_parity.label()
        );
        assert!(
            readings.iter().all(|reading| reading.is_red_as_required),
            "C8c（{}）都照登记红：{readings:?}",
            marker_parity.label()
        );
    }
}

/// C8d 的钉值（登记 r5 5.4、第七节乙类）：`even` 下 L 域 N ≤ 5 上 J8c 2 348、`odd` 下 0，r1 档那种生成器上 0；`odd` 下 C 域 Nc ≤ 3 上大于 0。
#[test]
fn the_c8d_control_reads_2348_on_even_0_on_odd_and_above_0_on_the_monotone_domain_under_odd() {
    let even = control_readings_of(
        Criterion::J8cNothingAppliedReturnsTheChosenRoot,
        RunProfile::root_fields_in_records(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
        1,
    );
    let odd = control_readings_of(
        Criterion::J8cNothingAppliedReturnsTheChosenRoot,
        RunProfile::root_fields_in_records(MarkerParity::OddVersionsWrittenByTheUnmountSequence),
        1,
    );
    assert_eq!(
        even.iter()
            .map(|reading| reading.value)
            .collect::<Vec<u64>>(),
        vec![2_348, 0],
        "C8d even：{even:?}"
    );
    assert_eq!(
        odd.iter()
            .take(2)
            .map(|reading| reading.value)
            .collect::<Vec<u64>>(),
        vec![0, 0],
        "C8d odd：{odd:?}"
    );
    assert_eq!(odd.len(), 3, "odd 另有 C 域那一条：{odd:?}");
    assert!(
        even.iter()
            .chain(&odd)
            .all(|reading| reading.is_red_as_required),
        "C8d 两种奇偶都照登记红"
    );
}
