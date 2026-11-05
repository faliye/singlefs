
    /// 攻方探针（m4-gpugc-code-r1，T1/T3）：一批切成 4 段（各一片），每段头一次提交等超过上限、重交一次做完。
    /// 条款「一批等 GPU 超过上限，放弃的那一批在同一张卡上重交，至多 3 次」；代码按段重交，这一批合起来超时 4 次、重交 4 次仍照常交回。
    #[test]
    fn attack_probe_a_batch_of_four_chunks_each_timing_out_once_returns_after_four_resubmissions() {
        use crate::checksum_kernels::{
            checksum_outcomes_of_the_units_cut_into_pieces, ChecksumKernelOutcome,
            UnitChecksumToVerify,
        };
        use crate::memory_limit_and_chunks::PieceOfAnEntry;
        let unit_bytes: Vec<Vec<u8>> = (0..4_u8).map(|value| vec![value; 8]).collect();
        let units: Vec<UnitChecksumToVerify<'_, usize>> = unit_bytes
            .iter()
            .enumerate()
            .map(|(entry_key, bytes)| UnitChecksumToVerify {
                entry_key,
                unit_bytes: bytes,
                expected_crc32c: 0,
            })
            .collect();
        let chunks_of_pieces: Vec<Vec<PieceOfAnEntry>> = (0..4)
            .map(|entry_position| {
                vec![PieceOfAnEntry {
                    entry_position,
                    input_bytes: 0..8,
                }]
            })
            .collect();
        let mut gpu = GpuWithScriptedSubmissions::scripted(
            vec![
                Err(this_submission_timed_out()),
                Ok(0),
                Err(this_submission_timed_out()),
                Ok(0),
                Err(this_submission_timed_out()),
                Ok(0),
                Err(this_submission_timed_out()),
                Ok(0),
            ],
            vec![Ok(()), Ok(()), Ok(()), Ok(())],
        );
        let outcome = ChecksumKernelOutcome {
            computed_crc32c: 0,
            matches_the_expected_crc32c: true,
        };
        let batch = checksum_outcomes_of_the_units_cut_into_pieces(
            &units,
            &chunks_of_pieces,
            |piece_units| {
                let pieces = piece_units.len();
                outcome_resubmitted_after_timeouts(
                    &mut gpu,
                    GpuWithScriptedSubmissions::wait_for_the_timed_out_submission,
                    |scripted| {
                        scripted
                            .submit_once()
                            .map(|_| vec![(outcome, outcome); pieces])
                    },
                )
            },
        );
        let timeouts_in_the_batch = gpu.submissions_made - chunks_of_pieces.len();
        println!(
            "ATTACK name=per_chunk_resubmission batch_ok={} submissions={} waits_for_earlier={} timeouts_in_the_batch={} resubmissions_allowed_per_submission={}",
            batch.is_ok(),
            gpu.submissions_made,
            gpu.waits_made,
            timeouts_in_the_batch,
            RESUBMISSIONS_AFTER_A_TIMEOUT
        );
        assert!(batch.is_ok(), "按段重交：这一批照常交回");
        assert!(
            u32::try_from(timeouts_in_the_batch).expect("小数") > RESUBMISSIONS_AFTER_A_TIMEOUT,
            "这一批里超时与重交的次数多于条款的「至多 3 次」"
        );
    }
}
