    #[test]
    fn layer0_main_arm_verdict_flags_every_threshold_independently() {
        let mut tally = Layer0Tally { states: 67_108_885, violations: 0, root_persisted_states: 4, file_read_states: 7, journal_differing_states: 3, ..Layer0Tally::default() };
        let comparison = PredictionComparison::default();
        let baseline = layer0_main_arm_verdict(&tally, 67_108_885, &comparison);
        assert!(baseline.layer0_states_ok && baseline.root_persisted_ok && baseline.file_read_ok && baseline.file_read_wrong_content_zero && baseline.main_outcome_matrix_ok);
        assert_eq!(baseline.layer0_violations, "zero");
        assert_eq!(baseline.journal_reading, "j_jia");

        let wrong_states = layer0_main_arm_verdict(&tally, 67_108_884, &comparison);
        assert!(!wrong_states.layer0_states_ok);
        assert!(wrong_states.root_persisted_ok, "只应该翻 layer0_states_ok 这一格");

        tally.violations = 1;
        let with_violation = layer0_main_arm_verdict(&tally, 67_108_885, &comparison);
        assert_eq!(with_violation.layer0_violations, "nonzero");
        assert!(with_violation.layer0_states_ok, "违例数不该影响 layer0_states_ok");
        tally.violations = 0;

        tally.root_persisted_states = 5;
        assert!(!layer0_main_arm_verdict(&tally, 67_108_885, &comparison).root_persisted_ok);
        tally.root_persisted_states = 4;

        tally.file_read_states = 6;
        assert!(!layer0_main_arm_verdict(&tally, 67_108_885, &comparison).file_read_ok);
        tally.file_read_states = 7;

        tally.file_read_wrong_content_states = 1;
        assert!(!layer0_main_arm_verdict(&tally, 67_108_885, &comparison).file_read_wrong_content_zero);
        tally.file_read_wrong_content_states = 0;

        tally.journal_differing_states = 9;
        assert_eq!(layer0_main_arm_verdict(&tally, 67_108_885, &comparison).journal_reading, "j_yi");
        tally.journal_differing_states = 4;
        assert_eq!(layer0_main_arm_verdict(&tally, 67_108_885, &comparison).journal_reading, "neither");
        tally.journal_differing_states = 3;

        let mut dirty_comparison = PredictionComparison::default();
        dirty_comparison.off_diagonal = 1;
        assert!(!layer0_main_arm_verdict(&tally, 67_108_885, &dirty_comparison).main_outcome_matrix_ok);
