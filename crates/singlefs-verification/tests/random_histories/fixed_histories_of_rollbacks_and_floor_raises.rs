//! 写死操作的历史：空覆盖写、挂载中回退、抬回退下限、回退到被丢弃的根、冷启动读回、同一个种子跑两遍逐字节相同。

use super::*;

/// 镜像上最新那条根（按 checker 的读法，(txg, 实例) 最大）的身份。
fn newest_root_on_the_image(image: &MemoryPool) -> ModelRootKey {
    let geometry = chosen_system_configurations(image)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
        .expect("历史里的盘上至少一块系统配置自证过");
    let (_, _, newest) = valid_roots(image, &geometry)
        .into_iter()
        .max_by_key(|(_, _, view)| (view.checkpoint_txg, view.instance))
        .expect("根环里至少有 mkfs 的第 0 代根");
    ModelRootKey {
        checkpoint_txg: ModelCheckpointTxg(newest.checkpoint_txg),
        instance: ModelInstanceGeneration(u64::from(newest.instance)),
    }
}

/// 执行器每一步量模型写进根环几条根（`HistoryTally::most_roots_written_by_the_model_in_one_step`，断言不多于根环槽数；
/// `IdealModel::committed_versions` 那一段注释）：从 mkfs 起可写挂载那一步写行与暖机两条（txg 1、2），第一个文件那一步一条（txg 3），
/// 起点（只做过 mkfs）一条都不写，最多的那一步是两条。判别力：模型写根不计数时这里是 0；断言的界压到 1 时挂载那一步就停在断言上、历史跑不完。
#[test]
fn the_executor_measures_the_roots_the_model_writes_in_each_step_and_the_mount_writes_two() {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 2999 },
                fill_seed: 3,
            }),
        ],
    };
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_observation| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert_eq!(
        run.tally.most_roots_written_by_the_model_in_one_step, 2,
        "可写挂载那一步写行与暖机两条根，是三步里最多的"
    );
}

/// 分配记录条数按 checker 的解析从镜像上数（`allocation_records_on_the_image_under`，不看分配器）：一条记录记一个单元、每盘一条、
/// 释放只改写不删（D3（空间分配） 已定项 7）。从 mkfs 起：第 0 代根与零单元写行、暖机那几版树表 0 条、没有分配记录树，数出 0 条；
/// 第一个文件那一版 28 条：七个单元（数据、extent 根、inode 叶、inode 根、记账、映射、树表）加分配记录树五个节点
/// （D8（核心索引结构） 已定项 14：4 GiB 两块盘上根在第 2 层，每块盘第 1 层一个、单元区起点那片叶一个），加 mkfs 的实例表与第 0 版树表，
/// 每盘各一条。从第一个文件起连着覆盖写（回收之前不复用；这几次的落点都还在单元区起点那片叶里，分配记录树每次重写那五个节点），
/// 每次每盘加 12 条：52、76、100。
#[test]
fn allocation_records_counted_on_the_image_are_one_per_unit_per_device_and_zero_without_a_file() {
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed: 5,
    });
    let count_after_each_step = |history: &GeneratedHistory| {
        let mut counted: Vec<Option<u64>> = Vec::new();
        let run = execute_history_with(
            history,
            HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            &SharedStream::new(),
            &mut |observation| {
                counted.push(allocation_records_on_the_image_under(
                    observation.image,
                    newest_root_on_the_image(observation.image),
                ));
            },
        );
        assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
        counted
    };
    let from_make_filesystem = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 2999 },
                fill_seed: 3,
            }),
        ],
    };
    assert_eq!(
        count_after_each_step(&from_make_filesystem),
        vec![Some(0), Some(0), Some(28)],
        "起点（txg 0）、挂载的零单元写行与暖机（txg 1、2）、第一个文件（txg 3）"
    );
    let from_the_first_file = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![overwrite; 3],
    };
    assert_eq!(
        count_after_each_step(&from_the_first_file),
        vec![Some(28), Some(52), Some(76), Some(100)]
    );
}

/// 内容为空的覆盖写：每次重写一个数据单元与提交内生块（extent 根、inode 叶与根、记账树、映射树、树表，加分配记录树按位置寻址
/// 重写的根与装着改了的记录的那几片叶，D8（核心索引结构） 已定项 14），小盘上每盘占 12 至 14 槽。
const EMPTY_CONTENT_OVERWRITE: HistoryOperation =
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::Empty,
        fill_seed: 0,
    });

/// 两块单元区 240 槽的小盘（准入关着，只供测试的开关 `SpaceAdmission::SkippedByTheTestOnlySwitch`，执行器与直接调挂载两处都关），
/// 从第一个文件起在 mkfs 那条会话里覆盖写 18 次空内容（txg 4–21），再可写挂载：挂载做成（实例 2），写行与一次暖机两次发布、
/// 执行器逐次比过分配记录、一条都没复用；同一份盘面（第 18 次覆盖写之后的镜像）另起两块内存盘直接调 `mount_writable`，
/// 同样做成、取号 2、暖机一次。这段历史原是增补 2 收口表第 39 行那一族（取号之前的准入不算落点）钉的那一形：240 槽的盘上挂载自己那一串
/// 在取号之前就取不到落点；小盘为环窗口补足之后单元区宽了，这里走不到那道墙了。
/// （Q-T「乙 照现值改钉并改名」）：覆盖写次数与盘宽不动，钉今天的结局；落点墙那一格只靠写死的那 5 条用例罩着。
/// 判别力：暖机计划多推一次时挂载那一步报三次发布、直接挂载暖机两次。
#[test]
fn eighteen_empty_overwrites_on_240_slot_devices_with_the_admission_skipped_leave_room_for_a_writable_mount_with_one_warm_up(
) {
    const OVERWRITES: usize = 18;
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::repeat_n(EMPTY_CONTENT_OVERWRITE, OVERWRITES)
            .chain(std::iter::once(HistoryOperation::CloseAndMountWritable))
            .collect(),
    };
    let device_width = HistoryDeviceWidth::UnitAreaOf240Slots;
    let mut images_after_each_step: Vec<MemoryPool> = Vec::new();
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
        },
        &SharedStream::new(),
        &mut |observation| {
            images_after_each_step.push(observation.image.clone());
        },
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        run.outcomes[..OVERWRITES].iter().all(|outcome| matches!(
            outcome,
            StepOutcome::Applied(AppliedEffect::Published { .. })
        )),
        "{OVERWRITES} 次覆盖写都做成：{:?}",
        &run.outcomes[..OVERWRITES]
    );
    assert_eq!(
        run.outcomes[OVERWRITES],
        StepOutcome::Applied(AppliedEffect::Mounted {
            instance: InstanceGeneration(2),
            publishes: 2,
            allocation_records_compared: MountAllocationComparison::Compared { publishes: 2 },
            reuse: RecordReuse::default(),
            space_admission: MountSpaceAdmissionOutcome::NotJudgedByTheTestOnlySwitch,
        }),
        "挂载做成：取号 2，写行与一次暖机逐次比过分配记录、一条都没复用"
    );
    assert_eq!(
        images_after_each_step.len(),
        OVERWRITES + 2,
        "起点与每一步各一份镜像"
    );
    let image_before_the_mount = &images_after_each_step[OVERWRITES];
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image_before_the_mount
        .devices
        .iter()
        .map(|(identity, image)| {
            let mut device = SparseBlockDevice::new(
                image_before_the_mount.device_size_in_bytes,
                PhysicalBlockSizeInBytes(512),
            );
            device.image = image.clone();
            (*identity, device)
        })
        .collect();
    let mounted = mount_writable_with_space_admission(
        &device_width.parameters(),
        &mut devices,
        SpaceAdmission::SkippedByTheTestOnlySwitch,
    )
    .unwrap_or_else(|error| panic!("同一份盘面直接挂载做成：{error:?}"));
    assert_eq!(
        (
            mounted.output.instance,
            mounted.output.warm_up_publishes.len()
        ),
        (InstanceGeneration(2), 1),
        "直接挂载取号 2、暖机一次"
    );
}

/// 挂着的时候回退到环里最旧的那条根，在单元区 384 槽的小盘上做成、之后照常覆盖写（C558（回退目标是环里最旧的根时回退那次发布会写坏它）
/// 那一格在随机历史执行器上的对照：回退那次发布的根槽就是回退目标住的那一槽）：第一个文件之后可写挂载（实例 2），覆盖写 24 次，
/// 此时根环 24 槽里最旧的那条是挂载之后第二次暖机的根；回退到它（一次向前发布）、再覆盖写一次，每一步之后池级 checker 判绿、模型逐步比。
/// 空间准入关掉（只供测试的开关 `SpaceAdmission::SkippedByTheTestOnlySwitch`）：判着准入时 384 槽的盘上挂载之后第 11 次覆盖写就被式子拒，
/// 根环凑不满。旧形态（挂载时回退）下这一格护的是「取号之前的预演连写行换下的落点一起释放」；向前回退不取号、不预演，那一半不再有对象。
#[test]
fn rolling_back_while_mounted_to_the_oldest_ring_root_on_a_small_device_and_overwriting_again_all_succeed(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(std::iter::repeat_n(EMPTY_CONTENT_OVERWRITE, 24))
            .chain([
                HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                    index_from_newest: 23,
                }),
                EMPTY_CONTENT_OVERWRITE,
            ])
            .collect(),
    };
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
        },
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        matches!(
            run.outcomes[25],
            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
        ),
        "回退到环里最旧的根做成：{:?}",
        run.outcomes[25]
    );
    assert!(
        matches!(run.outcomes[26], StepOutcome::Applied(_)),
        "回退之后覆盖写做成：{:?}",
        run.outcomes[26]
    );
}

/// 抬到现行的 F（0）：只推空发布、一个落点都不回收（释放代 ≤ 0 的一条都没有）。
const RAISE_TO_THE_CURRENT_FLOOR: HistoryOperation =
    HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
        steps_above_current_floor: 0,
    });

/// 越过原分配记录墙那一条写死用例的覆盖写：内容长度固定在一个数据单元之内。
const WALL_OVERWRITE: HistoryOperation = HistoryOperation::PublishOverwrite(ContentChoice {
    length: ContentLength::InsideOneDataUnit { selector: 2999 },
    fill_seed: 1,
});

/// 越过原分配记录墙那一条写死用例的前缀（逼近墙那三条共用的）：从第一个文件（txg 3）起在 mkfs 同一个进程那条会话里覆盖写 20 次
/// （txg 4–23，这一段根环还没转过），再连着可写挂载 `mounts` 次。根环转过之后，按可再分配谓词回收的槽被之后的发布复用改写
/// （D3（空间分配） 已定项 7：释放只改写不删），每一步加几条随复用走。
fn wall_prefix(mounts: usize) -> impl Iterator<Item = HistoryOperation> {
    std::iter::repeat_n(WALL_OVERWRITE, 20).chain(std::iter::repeat_n(
        HistoryOperation::CloseAndMountWritable,
        mounts,
    ))
}

/// 分配记录墙那三格（覆盖写正好到 812 条、抬 F 的第二次空发布正好到 812 条、回退的第二次暖机正好到 812 条；
/// 增补 3 第 2 件）拆墙之后（D8（核心索引结构） 已定项 14，K1）并成这一条：
/// 同样的前缀（`wall_prefix`，挂载 9 次）之后覆盖写 46 次、抬到现行的 F、挂着的时候回退到最新那条根（一次向前发布；旧形态是
/// 挂载时回退，它的第二次暖机正好到 812 条）、再覆盖写 5 次，一步都不拒，
/// 镜像上按 checker 的解析数的分配记录条数越过 812；每一步之后跑池级 checker、模型逐步比（跑完即都对得上）。
/// 一段历史每一步接着上一步的盘面，次序本身就是被测对象，不切片并行。
#[test]
fn overwrites_raising_the_floor_and_rolling_back_past_812_allocation_records_all_succeed() {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: wall_prefix(9)
            .chain(std::iter::repeat_n(WALL_OVERWRITE, 46))
            .chain(std::iter::once(RAISE_TO_THE_CURRENT_FLOOR))
            .chain(std::iter::once(HistoryOperation::RollBackWhileMounted(
                RollbackTargetChoice::RingRoot {
                    index_from_newest: 0,
                },
            )))
            .chain(std::iter::repeat_n(WALL_OVERWRITE, 5))
            .collect(),
    };
    let (run, counted_after_each_step) = run_counting_allocation_records_after_each_step(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    let refused: Vec<&StepOutcome> = run
        .outcomes
        .iter()
        .filter(|outcome| !matches!(outcome, StepOutcome::Applied(_)))
        .collect();
    assert_eq!(refused, Vec::<&StepOutcome>::new(), "一步都不拒");
    let most_counted = counted_after_each_step
        .iter()
        .map(|counted| counted.expect("每一步之后镜像上都数得出最新那条根下的分配记录"))
        .max()
        .expect("至少起点那一次");
    assert!(
        most_counted > 812,
        "镜像上数的分配记录条数要越过 812，最多 {most_counted} 条"
    );
    assert!(
        run.tally.step_images_handed_out > 0,
        "每一步之后跑池级 checker"
    );
}

/// 一段写死的历史在 4 GiB 的盘上每一步之后跑池级 checker、跑完，交回每一步之后（含起点）镜像上最新那条根下的分配记录条数。
fn run_counting_allocation_records_after_each_step(
    history: &GeneratedHistory,
) -> (HistoryRun, Vec<Option<u64>>) {
    let mut counted_after_each_step: Vec<Option<u64>> = Vec::new();
    let run = execute_history_with(
        history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            counted_after_each_step.push(allocation_records_on_the_image_under(
                observation.image,
                newest_root_on_the_image(observation.image),
            ));
        },
    );
    (run, counted_after_each_step)
}

/// 回退到 txg 低于 F_生效 的根要拒，理由是「低于 F」（D16（发布语义） 已定项 1「回退候选集」：txg ≥ F_生效）；它是候选排除，不是
/// 「树表 0 条、第一版不支持」（两者互报的变异里，「低于 F」报成「树表 0 条」那一条在门禁
/// 五段里各只红 1 段，这一条把它钉死）。第一个文件（txg 3）之后可写挂载（实例 2：txg 4、5）、覆盖写四次（6–9）、抬 F（选择子 6 ⇒ F = 6，
/// 推 txg 10、11）、挂着的时候回退到根环从新到旧第 6 条 (5, 2)：它带文件、没有被抛弃，只因 5 < 6 被挡。今天的代码上这段跑完、
/// 最后一步报 `BelowEffectiveFloor`，模型要求拒、理由对上。一段历史每一步接着上一步的盘面，不切片并行。
#[test]
fn rolling_back_below_the_effective_floor_is_refused_for_being_below_the_floor() {
    let overwrite = |fill_seed: u64| {
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 2999 },
            fill_seed,
        })
    };
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            overwrite(6),
            overwrite(7),
            overwrite(8),
            overwrite(9),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 6,
            }),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 6,
            }),
        ],
    };
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        matches!(
            run.outcomes[5],
            StepOutcome::Applied(AppliedEffect::RaisedFloor {
                new_floor: CheckpointTxg(6),
                ..
            })
        ),
        "抬到 6：{:?}",
        run.outcomes[5]
    );
    assert_eq!(
        run.outcomes[6],
        StepOutcome::Refused {
            member: "RollbackError::TargetNotACandidate(BelowEffectiveFloor)".to_string()
        }
    );
    assert_eq!(
        run.tally.model_counts.required_refusals_matched, 1,
        "模型要求拒、理由对上"
    );
}

/// 回退到 txg = F_生效 的根要做成（D16（发布语义） 已定项 1「回退候选集」：txg ≥ F_生效），模型按 B2 那一格判（攻方变异「回退到
/// txg = F_生效 的根也拒」）。第一个文件（txg 3）之后可写挂载（实例 2：txg 4、5）、覆盖写四次（6–9）、抬 F（选择子 6 ⇒ F = 6：上限是
/// 第 4 新的非空根 6 与盘 1 上最新的有效根 7 取小，推 txg 10、11 两次）、挂着的时候回退到候选集的下沿 (6, 2)（一次向前发布 txg 12）、
/// 冷启动读回。今天的代码上这段跑完：回退做成一次、落在 F 上，冷启动读回的是 txg 6 那次覆盖写的内容（模型比过）。取样点里这一格 48 个种子一窗只有
/// 0–5 段（在 [0, 960) 上逐窗数），这一条把它钉死。
#[test]
fn rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version() {
    let overwrite = |fill_seed: u64| {
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 2999 },
            fill_seed,
        })
    };
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            overwrite(6),
            overwrite(7),
            overwrite(8),
            overwrite(9),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 6,
            }),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRootAtTheNewestFloor),
            HistoryOperation::ColdStartRecover,
        ],
    };
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        matches!(
            run.outcomes[5],
            StepOutcome::Applied(AppliedEffect::RaisedFloor {
                new_floor: CheckpointTxg(6),
                publishes: 2,
                ..
            })
        ),
        "抬到 6、推 txg 10（盘 1）与 11（盘 0）：{:?}",
        run.outcomes[5]
    );
    assert!(
        matches!(
            run.outcomes[6],
            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
        ),
        "回退到 (6, 2) 要做成：{:?}",
        run.outcomes[6]
    );
    assert_eq!(
        run.tally
            .model_counts
            .rollbacks_accepted_at_the_effective_floor,
        1,
        "模型数到一次落在 F_生效 上的回退"
    );
    assert_eq!(
        run.tally.model_counts.cold_start_contents_compared, 1,
        "冷启动读回的内容模型比过"
    );
}

/// 增补 2 收口表第 ② 行（一次挂载转过一整圈根环时 checker 在合法状态上判 I-3.1 红）在一次挂载里那一形，C518（一次挂载之内环转过一圈之后不回收） 修好之后跑完：
/// 第一个文件之后可写挂载一次（写行 txg 4、暖机 txg 5），同一次挂载里连着覆盖写 25 次（txg 6–30）。根环 R × S = 24 槽：txg 24、25、26
/// 依次盖掉第 0 代根与两条暖机根，txg 26 盖掉之后再没有一条有效根引用 mkfs 的第 0 版树表单元（1 槽，txg 3 释放），它在这一次发布里
/// 按谓词回收、记账按回收之后的数写——修之前它没回收、记账仍算已分配，checker 在这一步判 I-3.1 红（记账比遍历多 16384 字节）。
/// 现在这段历史跑完，起点、挂载、25 次覆盖写之后各交出一张镜像（这个二进制不调池级 checker；同一段历史带池级 checker 判、
/// 挂载内回收那一槽由 `root_ring_turn_in_one_mount.rs` 的 `turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it`
/// 与 同名的给定状态用例判）。
#[test]
fn turning_the_root_ring_with_overwrites_in_one_mount_runs_to_the_end_handing_out_an_image_after_every_step(
) {
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed: 1,
    });
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(std::iter::repeat_n(overwrite, 25))
            .collect(),
    };
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert_eq!(
        run.tally.step_images_handed_out, 27,
        "起点、挂载、25 次覆盖写之后各交出一张镜像"
    );
}

/// 增补 2 收口表第 ② 行在 mkfs 同一个进程那条会话里那一形，那条会话也装上根环表（`make_filesystem::allocator_after_make_filesystem`）
/// 之后跑完：同样 25 次覆盖写（txg 4–28），一次挂载都不做。txg 26 那次覆盖写（第 22 步）盖掉 txg 2 的暖机根，环里再没有一条有效根
/// 引用 mkfs 的第 0 版树表单元（1 槽，第一个文件版本在 txg 3 换下），它在这一次发布里按谓词回收、记账按回收之后的数写——
/// 不装那张表时它不回收、记账仍算已分配，checker 在这一步判 I-3.1 红（记账比遍历多 16384 字节）。
/// 装了表，这段历史跑完、根环转过一圈，起点与 25 次覆盖写之后各交出一张镜像（这个二进制不调池级 checker；mkfs 那条会话里覆盖写
/// 转过根环、按谓词回收、池级 checker 不红，由 `root_ring_turn_in_the_make_filesystem_session.rs` 的
/// `overwrites_in_the_make_filesystem_session_past_one_turn_of_the_root_ring_reclaim_and_keep_every_invariant` 与 同名的给定状态用例判，
/// 覆盖写到 txg 30）。
#[test]
fn turning_the_root_ring_with_overwrites_in_the_make_filesystem_process_runs_to_the_end_handing_out_an_image_after_every_step(
) {
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed: 1,
    });
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::repeat_n(overwrite, 25).collect(),
    };
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert_eq!(
        run.tally.step_images_handed_out, 26,
        "起点与 25 次覆盖写之后各交出一张镜像"
    );
    assert_eq!(
        run.tally.histories_that_turned_the_root_ring, 1,
        "最后一条根 txg 28 ≥ R × S = 24：根环转过了一圈"
    );
}

/// 「已知红」清单那一条（增补 2 收口表第 43 行）的复现是随机历史那一批种子 80 收缩出来的那一段，向前回退之后跑完：
/// 可写挂载（实例 2，txg 4、5）、覆盖写两次（6、7）、挂着的时候回退到最新那条根 (2, 7)（txg 8）、再回退到从新到旧第 4 条 (2, 5)（txg 9）、
/// 覆盖写（10）、可写挂载（实例 3，txg 11、12）、覆盖写三次（13–15）、抬 F（选择子 8）。旧形态下两次回退各开一个实例、抛弃前一段，
/// F = 8 落进回退留下的空档，I-3.1 记账多算 12 槽；向前回退不抛弃任何根，这一段没有空档，每一步之后 checker 判绿、跑完。
/// 清单那一条的形态今天只剩崩溃恢复抛弃的时间线造得出（实四乙第 7 件）；C554 乙之后只剩系统配置没见证到被抛弃那条根的那一形，
/// 那一形的时间线由 `crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units`
/// 造（C583（双故障形：每盘一槽系统配置坏加最新根一时读不出））：可写挂载时的预隔离护住被抛弃那条根的单元，抬 F 之前池级 checker 不红。
#[test]
fn the_history_that_raised_the_floor_into_a_rollback_gap_completes_under_the_forward_rollback() {
    let empty = ContentChoice {
        length: ContentLength::Empty,
        fill_seed: 0,
    };
    let history = GeneratedHistory {
        seed: HistorySeed(80),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 0,
            }),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 3,
            }),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 8,
            }),
        ],
    };
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        matches!(
            (&run.outcomes[3], &run.outcomes[4]),
            (
                StepOutcome::Applied(AppliedEffect::RolledBack { .. }),
                StepOutcome::Applied(AppliedEffect::RolledBack { .. })
            )
        ),
        "两次回退都做成：{:?}",
        &run.outcomes[3..5]
    );
    assert!(
        matches!(
            run.outcomes[10],
            StepOutcome::Applied(AppliedEffect::RaisedFloor { .. })
        ),
        "抬 F 做成：{:?}",
        run.outcomes[10]
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_random_histories::run_the_scenario_where_crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units(
) {
    scenarios_of_random_histories::run_the_scenario_where_crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units(&mut PoolCheckerImages::left_to_the_given_states_engine());
}

/// 崩溃恢复抛弃的那条根不在回退的候选集里：照 [`pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version`] 造出被抛弃的 C
/// （实例 1、txg 5；系统配置没见证到它），挂在实例 2 上回退到 C：被拒成「在被抛弃的时间线上」，拒之前盘上逐字节不变
/// （两盘的系统配置槽、根环里读得出的根、录制流步数），胶水把这个成员映射成模型的「回退目标在被抛弃的时间线上」
/// （随机历史里模型对这一格要求拒的就是这一条）。
#[test]
fn rolling_back_to_a_root_crash_recovery_abandoned_without_a_system_configuration_witness_is_refused_on_the_abandoned_timeline(
) {
    let (mut pool, abandoned) = pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version(
        "random-history-rollback-onto-the-abandoned",
    );
    assert_eq!(
        (abandoned.root.instance, abandoned.root.checkpoint_txg),
        (InstanceGeneration(1), CheckpointTxg(5))
    );
    let disk_before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refused = roll_back_by_a_forward_publish(
        &common::parameters(),
        pool.devices.as_mut().expect("镜像还开着"),
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: abandoned.root.instance,
            checkpoint_txg: abandoned.root.checkpoint_txg,
        },
    )
    .expect_err("C 在被抛弃的时间线上");
    assert!(
        matches!(
            refused,
            RollbackError::TargetNotACandidate {
                exclusion: RollbackCandidateExclusion::OnAbandonedTimeline,
                ..
            }
        ),
        "被拒成「在被抛弃的时间线上」：{refused:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        disk_before,
        "拒之前一个字节都不写"
    );
    assert_eq!(
        refusal_reason_of_rollback_error(&refused),
        ObservedRefusalReason::Explained(ModelRefusalReason::RollbackTargetOnAbandonedTimeline)
    );
}

/// 冷启动读回报的根取施加 journal 之后实际走的那条（实四丙交回 ④ offset 330，实七）：第一个文件之后覆盖写一次（txg 4），
/// 它的根槽 FUA 写被说谎的设备吞掉——journal 记录与单元都落了盘，根槽没落；接着冷启动。恢复择到 txg 3、施加 txg 4 的记录、
/// 读回 txg 4 那一版的内容，模型要的也是 txg 4。按施加之前所选的根报（`observed_read_back`），对拍报「冷启动读回」对不上。
#[test]
fn cold_start_after_a_swallowed_root_slot_write_reads_back_the_version_its_applied_record_rebuilt_and_the_model_agrees(
) {
    let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
    let parameters = execution.device_width.parameters();
    let root_slot_of_the_overwrite = singlefs_core::root_ring::target_for_publish(
        CheckpointTxg(4),
        parameters.geometry.root_ring_slots_per_region,
    );
    let plan = SharedFaultPlan::armed(
        execution.device_width.fixed_geometry(),
        FaultSchedule {
            fault: InjectedFault::WriteIsSwallowed,
            device: FaultDeviceSelector::OnlyDevice(
                parameters.region_devices
                    [usize::try_from(root_slot_of_the_overwrite.region).expect("区域号小于 3")],
            ),
            placement: FaultPlacement::OffsetExactly(singlefs_core::root_ring::slot_offset(
                root_slot_of_the_overwrite,
                parameters.geometry.fixed_structure_slot_spacing,
            )),
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::TheNthMatchingCall(1),
        },
    );
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 2999 },
                fill_seed: 7,
            }),
            HistoryOperation::ColdStartRecover,
        ],
    };
    let run = execute_history_with_faults(
        &history,
        execution,
        &SharedStream::new(),
        &plan,
        &mut |_| {},
    );
    let fired = plan.fired();
    assert_eq!(fired.len(), 1, "只吞掉覆盖写那一次根槽 FUA 写：{fired:?}");
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        matches!(
            &run.outcomes[1],
            StepOutcome::Applied(AppliedEffect::Recovered {
                read_back: ColdStartReadBack::FileRead,
                ..
            })
        ),
        "冷启动读回文件：{:?}",
        run.outcomes[1]
    );
}

/// 不经回退的抬 F 之后 I-3.1 记账多算是新发现，不是清单那一条（收口表第 43 行）。历史照攻方的 `opus_z2_history.rs`：
/// 第一个文件之后可写挂载（实例 2，txg 4、5）、覆盖写三次（6、7、8）、抬 F（选择子 3 ⇒ F = 3），一次回退都没有。今天的代码上这段跑完，
/// 抬 F 之前的镜像上 txg 3 那条根是实例 1 的有效根 ⇒ 「F 那个 txg 上的根全属于被抛弃的实例」为假；拿它配上攻方在回收门槛差一
/// （A1）下量到的那一步违例（`盘 0：记账的已分配 Some(999424)，遍历全部有效根得到 983040`），分类是新发现。同一个判定在那一条的复现
/// （种子 80 那一段，F = 8 落在被抛弃的实例 3 上）为真，那一条由上面的用例钉着。
#[test]
fn an_allocated_statistic_over_count_after_raising_the_floor_without_a_rollback_is_a_new_finding() {
    let three_overwrites = std::iter::repeat_n(
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 2999 },
            fill_seed: 7,
        }),
        3,
    );
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(three_overwrites)
            .chain(std::iter::once(HistoryOperation::RaiseRollbackFloor(
                FloorTargetChoice {
                    steps_above_current_floor: 3,
                },
            )))
            .collect(),
    };
    let mut image_before_raising: Option<MemoryPool> = None;
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            if observation.position == StepPosition::Operation(3) {
                image_before_raising = Some(observation.image.clone());
            }
        },
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "今天的代码上这段跑完");
    let StepOutcome::Applied(AppliedEffect::RaisedFloor { new_floor, .. }) = run.outcomes[4] else {
        panic!("第 4 步抬 F 要成：{:?}", run.outcomes[4]);
    };
    assert_eq!(new_floor, CheckpointTxg(3));
    let lands_only_on_abandoned = raised_floor_lands_only_on_abandoned_roots(
        &image_before_raising.expect("观察者见过第 3 步之后的镜像"),
        new_floor,
    );
    assert_eq!(
        lands_only_on_abandoned,
        Some(false),
        "txg 3 那条根是实例 1 的有效根，不在任何回退留下的空档里"
    );
    let observation = FailureObservation {
        position: StepPosition::Operation(4),
        operation_kind: Some(HistoryOperationKind::RaiseRollbackFloor),
        violations: vec![(
            "I-3.1",
            // checker 在 I-3.1 的说明文字里带的机理标识：这里是「F 把 4 条读得到的根挡在了遍历之外」那一种
            // （`singlefs-checker` 的 `walk.rs`）——机理对得上而 F 没落在空档里，所以照样是新发现。
            "盘 0：记账的已分配 Some(999424)，遍历全部有效根得到 983040；机理：根环槽数 24、最新根 txg 10、环里自证过的根槽 11 个、最老的自证过的根 txg 0、遍历的候选根槽 7 个、被实例表判抛弃的根槽 0 个、回退下界 F 3、低于 F 的根槽 4 个".to_string(),
        )],
        panic: None,
        newest_ring_root_txg: Some(10),
        root_ring_slot_count: Some(24),
        harness_judgement: None,
        model_disagreement: None,
        raised_floor_lands_only_on_abandoned_roots: lands_only_on_abandoned,
        record_check: RecordCheck::default(),
    };
    let ending = classify_failure(observation);
    assert!(
        matches!(
            &ending,
            HistoryEnding::NewFinding {
                signature: FailureSignature::CheckerViolations { invariants },
                ..
            } if invariants == &vec!["I-3.1"]
        ),
        "不经回退的抬 F 之后 I-3.1 多算要是新发现：{ending:?}"
    );
}

/// 挂载写出的写行与暖机那几次发布也比分配代。历史是攻方的形态 a
/// （只在挂载里复用改写已回收记录时不改分配代）在专门取样点的比重下种子 2 收缩出来的 11 步：从 mkfs 起，可写挂载、第一个文件、
/// 可写挂载、覆盖写、抬 F（到 0）、覆盖写四次、抬 F 到 6（回收 18 个落点）、再可写挂载——最后这次挂载的写行与暖机复用改写了已回收的记录。
/// 今天的代码上这段跑完；最后一步的挂载逐次比了两次带文件的发布、复用改写 14 条。形态 a / b 下最后一步是执行器判出的失败，本用例红。
#[test]
fn row_and_warm_up_publishes_that_reuse_reclaimed_records_have_their_allocation_generations_checked(
) {
    let empty = ContentChoice {
        length: ContentLength::Empty,
        fill_seed: 0,
    };
    let history = GeneratedHistory {
        seed: HistorySeed(2),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(empty),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 0,
            }),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 6,
            }),
            HistoryOperation::CloseAndMountWritable,
        ],
    };
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "今天的代码上这段跑完：挂载写出的发布里复用改写的记录，代都是那一次的 txg"
    );
    let StepOutcome::Applied(AppliedEffect::Mounted {
        allocation_records_compared,
        reuse,
        ..
    }) = &run.outcomes[10]
    else {
        panic!("最后一步挂载要成：{:?}", run.outcomes[10]);
    };
    assert_eq!(
        *allocation_records_compared,
        MountAllocationComparison::Compared { publishes: 2 },
        "写行与一次暖机都带文件，逐次比过"
    );
    assert!(
        reuse.rewritten_from_released >= 1,
        "这次挂载的发布真的复用改写了已回收的记录：{reuse:?}"
    );
}

/// 同一个种子跑两次，每一步的结局、收尾、计数逐项相同，每一步之后的整份镜像逐字节相同（随机源手写、写入时间是参数）。
#[test]
fn the_same_seed_runs_to_the_same_outcomes_and_the_same_bytes_twice() {
    let history = generate_history(HistorySeed(7), 24);
    let run_keeping_every_image = || {
        let mut images: Vec<MemoryPool> = Vec::new();
        let run = execute_history_with(
            &history,
            HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            &SharedStream::new(),
            &mut |observation| {
                images.push(observation.image.clone());
            },
        );
        (run, images)
    };
    let (first, first_images) = run_keeping_every_image();
    let (second, second_images) = run_keeping_every_image();
    assert_eq!(first, second);
    assert!(
        first.tally.step_images_handed_out >= 2,
        "起点之后与至少一步写之后都跑了 checker：{}",
        first.tally.step_images_handed_out
    );
    assert!(
        first_images.len() >= 2,
        "观察者至少见到起点与一步之后的镜像：{}",
        first_images.len()
    );
    assert!(first_images == second_images, "每一步之后的镜像逐字节相同");
}

/// 代码三方 `research/prompts/m2-final-code-r3-main-verification.md` 第三节「越格线索」那两段历史（攻方
/// `research/prompts/m2-final-code-r3-opus-model/rerun.sh` 里 `z13_one_seed` 那一格：比重 `REUSE_AFTER_RAISING_THE_FLOOR`、60 步、
/// 两块单元区 240 槽的小盘、每一步之后跑池级 checker），空间准入关掉：在 `step_of_the_raise` 那一步抬 F，结局是 `raise_outcome`，
/// 60 步跑完、每一步之后池级 checker 判绿、模型逐步对得上。那两段里抬 F 那一串的第二次在预演里取得到落点、做成
/// （C546 第二次起那一半）；小盘为环窗口补足之后单元区宽了。
/// （Q-T「乙 照现值改钉并改名」）：种子与几何都不动，钉今天的结局；预演里取不到落点那一形只靠写死的那 5 条用例罩着。
fn the_floor_raise_on_narrow_devices_with_the_admission_skipped_has_its_outcome_and_the_history_runs_to_the_end(
    seed: u64,
    step_of_the_raise: usize,
    raise_outcome: &StepOutcome,
) {
    let history = generate_history_with_weights(
        HistorySeed(seed),
        60,
        &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
    );
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf240Slots,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
        },
        &SharedStream::new(),
        &mut |_| {},
    );
    assert!(
        matches!(
            history.operations[step_of_the_raise],
            HistoryOperation::RaiseRollbackFloor(_)
        ),
        "种子 {seed} 第 {step_of_the_raise} 步是抬 F"
    );
    assert_eq!(
        run.outcomes.get(step_of_the_raise),
        Some(raise_outcome),
        "种子 {seed} 第 {step_of_the_raise} 步的抬 F（历史停在：{:?}）",
        run.ending
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "种子 {seed}：60 步跑完，每一步之后池级 checker 判绿"
    );
    assert_eq!(run.outcomes.len(), 60);
}

/// 种子 4000000045 第 27 步：F 抬到 19，三次空发布，回收 135 个落点。
/// 判别力：抬 F 生效之后不回收时回收数是 0；上限少算一档时新 F 不是 19。
#[test]
fn seed_4000000045_on_240_slot_devices_with_the_admission_skipped_raises_the_floor_to_19_reclaiming_135_placements(
) {
    the_floor_raise_on_narrow_devices_with_the_admission_skipped_has_its_outcome_and_the_history_runs_to_the_end(
        4_000_000_045,
        27,
        &StepOutcome::Applied(AppliedEffect::RaisedFloor {
            new_floor: CheckpointTxg(19),
            publishes: 3,
            reclaimed_placements: 135,
            reuse: RecordReuse::default(),
        }),
    );
}

/// 种子 4000000001 第 30 步：F 抬到 7，两次空发布，一个落点都没回收。
/// 判别力：上限少算一档时新 F 不是 7。
#[test]
fn seed_4000000001_on_240_slot_devices_with_the_admission_skipped_raises_the_floor_to_7_in_two_publishes(
) {
    the_floor_raise_on_narrow_devices_with_the_admission_skipped_has_its_outcome_and_the_history_runs_to_the_end(
        4_000_000_001,
        30,
        &StepOutcome::Applied(AppliedEffect::RaisedFloor {
            new_floor: CheckpointTxg(7),
            publishes: 2,
            reclaimed_placements: 0,
            reuse: RecordReuse::default(),
        }),
    );
}
