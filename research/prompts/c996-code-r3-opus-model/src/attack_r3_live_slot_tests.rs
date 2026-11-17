
    // ── 攻方腿模型（c996-code-r3）：接在 live_slot_kernels.rs 的测试模块末尾，只在冻结副本的拷贝里，不进仓 ──
    /// 乙（bggpu-audit-fix-code-r1 攻方腿那一条，在今天的代码上重跑）：丙段一批里同一个段出现两次。三种重复的摆法 × 两份核交成、交回「同一个键两次」。
    /// 原读数（任务 14625）：6 行都是 `outcome=panicked message=assertion…丙段交进来的是单元区的整段`。条款：交回出错、计数不动。
    /// c996-code-r3 另加一臂 `kernels_failed=gpu_failed`：两份核那一次是这张卡真出错了（等超过上限 4 次、报这张卡用不了）。
    #[test]
    fn attack_model_a_repeated_segment_meets_the_full_list_assertion_first() {
        use crate::background_gpu::{GpuFailureCause, LiveSlotBatchFailed};
        let unit_area = unit_area_of_three_segments();
        let full = cluster_segment_entry_keys_of_the_unit_area(unit_area);
        let arrangements: Vec<(&str, Vec<ClusterSegmentEntryKey>)> = vec![
            ("middle_segment_twice_in_place_of_the_last", vec![full[0], full[1], full[1]]),
            ("last_segment_appended_again", vec![full[0], full[1], full[2], full[2]]),
            ("first_segment_appended_again", vec![full[0], full[1], full[2], full[0]]),
        ];
        let mut rows_not_returning_the_repeated_key = 0;
        let mut gpu_failures_reported_as_the_repeated_key = 0;
        for (arrangement, segments) in arrangements {
            for kernels in ["reconciled", "failed", "gpu_failed"] {
                let both_kernel_live_slots: Result<Vec<(u64, u64)>, LiveSlotBatchFailed> =
                    match kernels {
                        "reconciled" => Ok(vec![(0, 0); segments.len()]),
                        "failed" => Err(LiveSlotBatchFailed::SameEntryKeyTwiceInOneBatch(
                            SameEntryKeyTwiceInOneBatch {
                                position_of_the_first_occurrence: 9,
                                position_of_the_second_occurrence: 9,
                            },
                        )),
                        _ => Err(LiveSlotBatchFailed::GpuFailed {
                            device: singlefs_core::background_compute::GpuDeviceIdentifier::of_opaque_bytes(
                                b"0000:01:00.0".to_vec(),
                            ),
                            cause: GpuFailureCause::UnusableAfterResubmissions {
                                resubmissions: 3,
                                last_cause: Box::new(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                                    wait_limit: std::time::Duration::from_secs(1),
                                }),
                            },
                        }),
                    };
                let mut uncertainty = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
                let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    reconciled_live_slots_of_the_segments(
                        &[],
                        unit_area,
                        &segments,
                        both_kernel_live_slots,
                        &mut uncertainty,
                    )
                }));
                let outcome = match caught {
                    Ok(Ok(_reconciled)) => "outcome=reconciled".to_string(),
                    Ok(Err(LiveSlotBatchFailed::SameEntryKeyTwiceInOneBatch(repeated))) => format!(
                        "outcome=returned_error first={} second={}",
                        repeated.position_of_the_first_occurrence,
                        repeated.position_of_the_second_occurrence
                    ),
                    Ok(Err(other)) => format!(
                        "outcome=returned_other_error {}",
                        format!("{other:?}").replace(char::is_whitespace, "_")
                    ),
                    Err(payload) => format!(
                        "outcome=panicked message={}",
                        payload
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| payload.downcast_ref::<&str>().map(|said| (*said).to_string()))
                            .unwrap_or_default()
                            .lines()
                            .next()
                            .unwrap_or("")
                            .replace(char::is_whitespace, "_")
                    ),
                };
                if !outcome.starts_with("outcome=returned_error") {
                    rows_not_returning_the_repeated_key += 1;
                }
                if kernels == "gpu_failed" && outcome.starts_with("outcome=returned_error") {
                    gpu_failures_reported_as_the_repeated_key += 1;
                }
                println!(
                    "name=attack_model_live_slot arrangement={arrangement} kernels={kernels} {outcome}"
                );
            }
        }
        println!(
            "name=attack_model_live_slot_summary rows=9 rows_not_returning_the_repeated_key={rows_not_returning_the_repeated_key} gpu_failures_reported_as_the_repeated_key={gpu_failures_reported_as_the_repeated_key}"
        );
        assert_eq!(rows_not_returning_the_repeated_key, 0, "乙：重复的段都交回出错");
    }
