//! 攻方腿原型（m4-segment-code-r2）：整理、回退、删完之后的快照混在一段随机历史里，执行器逐步与理想模型对拍，
//! 另外每一步之后的镜像过池级 checker（执行器自己不跑它）。比重与种子段见 `attack_common/weights_r2.rs`。
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, AppliedEffect, GenerationWeights,
    HistoryEnding, HistoryExecution, HistoryOperationKind as K, HistorySeed,
    HistoryStartingPoint, NamedGenerationWeights, OverwriteContentDraw, RollbackTargetDraw,
    StepOutcome, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::SharedStream;

include!("attack_common/weights_r2.rs");

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// ATTACK_DEVICE_WIDTH：384 / 256 / 240 取小盘那三档，别的取两块 4 GiB 的盘。
fn execution() -> HistoryExecution {
    use singlefs_harness::history::HistoryDeviceWidth;
    let device_width = match std::env::var("ATTACK_DEVICE_WIDTH").as_deref() {
        Ok("384") => HistoryDeviceWidth::UnitAreaOf384Slots,
        Ok("256") => HistoryDeviceWidth::UnitAreaOf256Slots,
        Ok("240") => HistoryDeviceWidth::UnitAreaOf240Slots,
        _ => HistoryDeviceWidth::FourGibibytes,
    };
    HistoryExecution {
        device_width,
        ..HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES
    }
}

/// 固定点那一族（ckpt_cost 的 Σ 名单：分配记录树、中央映射树、记账树、树表，加谱系、退役两项）。
fn is_a_fixed_point_role(identity: singlefs_core::transaction::TransactionUnit) -> bool {
    use singlefs_core::transaction::TransactionUnit as U;
    matches!(
        identity,
        U::AllocationTreeNodeBelowTheRoot(_)
            | U::AllocationTree
            | U::MappingTreeNodeBelowTheRoot(_)
            | U::MappingTree
            | U::AccountingTreeNodeBelowTheRoot(_)
            | U::AccountingTree
            | U::TreeTable
            | U::LineageLeafContainer(_)
            | U::LineageRoot
            | U::LineageNodeBelowTheRoot(_)
            | U::RetirementLeafContainer(_)
            | U::RetirementRoot
            | U::RetirementNodeBelowTheRoot(_)
    )
}

/// ATTACK_CKPT_MARGIN=1：这一步镜像根环里每一对相邻、同实例、txg 差一的根，后一版里固定点那一族新写的单元占的槽数
/// （角色与槽都不在前一版里的）与前一版的 ckpt_cost（`admission::checkpoint_cost_of_the_version_to_build_on`）比，交回（txg, 写的, ckpt_cost）。
/// 只在两块 4 GiB 的盘上算（分配器按默认单元区起点建）。
fn checkpoint_cost_margins(image: &singlefs_harness::memory_pool::MemoryPool, measured: &mut std::collections::BTreeSet<u64>) -> Vec<(u64, u64, u64)> {
    use singlefs_core::address::DeviceIdentity;
    use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
    use singlefs_core::recovery::{choose_system_configuration, readable_roots, rebuild_version_with_its_snapshot_entries, RebuiltVersion};
    let Ok(configuration) = choose_system_configuration(image) else { return Vec::new() };
    let mut roots = readable_roots(
        image,
        &configuration.immutable.region_devices,
        &configuration.immutable.sizes,
        &configuration.immutable.filesystem_identifier,
    );
    roots.sort_by_key(|root| (root.checkpoint_txg, root.instance));
    let allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), image.device_size_in_bytes),
        DeviceFreeMap::new(DeviceIdentity(1), image.device_size_in_bytes),
    ]);
    let rebuild = |root| match rebuild_version_with_its_snapshot_entries(image, root, None).map(|rebuilt| rebuilt.version) {
        Ok(RebuiltVersion::WithFile(output) | RebuiltVersion::WithTreesAndNoFile(output)) => Some(output),
        other => {
            if std::env::var("ATTACK_CKPT_DEBUG").is_ok() {
                eprintln!("CKPT_DEBUG 重建不出 {:?}", other.map(|_| "WithoutFile"));
            }
            None
        }
    };
    let mut margins = Vec::new();
    for pair in roots.windows(2) {
        let (earlier, later) = (&pair[0], &pair[1]);
        if later.checkpoint_txg.0 != earlier.checkpoint_txg.0 + 1 || later.instance != earlier.instance || measured.contains(&later.checkpoint_txg.0) {
            continue;
        }
        let (Some(previous), Some(next)) = (rebuild(earlier), rebuild(later)) else { continue };
        measured.insert(later.checkpoint_txg.0);
        let carried: std::collections::BTreeSet<(String, u64)> = previous
            .units
            .iter()
            .map(|unit| (format!("{:?}", unit.identity), unit.slot.0))
            .collect();
        let written: u64 = next
            .units
            .iter()
            .filter(|unit| is_a_fixed_point_role(unit.identity))
            .filter(|unit| !carried.contains(&(format!("{:?}", unit.identity), unit.slot.0)))
            .map(|unit| unit.identity.span_slots())
            .sum();
        let cost = singlefs_core::admission::checkpoint_cost_of_the_version_to_build_on(Some(&previous), &allocator).0;
        margins.push((later.checkpoint_txg.0, written, cost));
    }
    margins
}

#[test]
fn attack_r2_every_step_image_passes_the_pool_checker_and_the_model_agrees() {
    let seed_start = env_u64("ATTACK_SEED_START", 0);
    let seeds = env_u64("ATTACK_SEEDS", 2);
    let ops = usize::try_from(env_u64("ATTACK_OPS", 30)).expect("步数");
    let verbose = env_u64("ATTACK_VERBOSE", 0) == 1;
    let mut problems: Vec<String> = Vec::new();
    let mut steps_checked = 0u64;
    let mut kinds: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let measure_margins = env_u64("ATTACK_CKPT_MARGIN", 0) == 1;
    let mut smallest_margin: (i128, String) = (i128::MAX, String::new());
    let mut margins_measured = 0u64;
    for index in seed_start..seed_start + seeds {
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + ATTACK_SEED_OFFSET + index;
        let history = generate_history_with_weights(HistorySeed(seed), ops, attack_weights());
        let mut seen: Vec<String> = Vec::new();
        let mut measured: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
        let run = execute_history_with(
            &history,
            execution(),
            &SharedStream::new(),
            &mut |observation| {
                if let Some(outcome) = observation.outcome {
                    let key = match outcome {
                        StepOutcome::Applied(AppliedEffect::Compacted {
                            batches_published,
                            units_relocated,
                            stop,
                            ..
                        }) => {
                            if batches_published > units_relocated {
                                seen.push(format!(
                                    "种子偏移 {index} 位置 {:?}：整理报 {batches_published} 批、只搬 {units_relocated} 个",
                                    observation.position
                                ));
                            }
                            format!("Compacted stop={stop} batches>0={}", *batches_published > 0)
                        }
                        other => {
                            let text = format!("{other:?}");
                            let head: String =
                                text.chars().take_while(|c| *c != '{' && *c != '(').collect();
                            let inner: String = text.chars().skip(head.len()).take(48).collect();
                            format!("{head}{inner}")
                        }
                    };
                    *kinds.entry(key).or_insert(0) += 1;
                }
                if verbose {
                    eprintln!(
                        "STEP {:?} op={:?} outcome={}",
                        observation.position,
                        observation.operation,
                        observation
                            .outcome
                            .map(|o| format!("{o:?}").chars().take(400).collect::<String>())
                            .unwrap_or_default()
                    );
                }
                if measure_margins {
                    for (txg, written, cost) in checkpoint_cost_margins(observation.image, &mut measured) {
                        let margin = i128::from(cost) - i128::from(written);
                        if margin < smallest_margin.0 {
                            smallest_margin = (margin, format!("种子偏移 {index} txg {txg} 写 {written} 槽 ckpt_cost {cost} 操作 {:?}", observation.operation.map(|o| format!("{o:?}").chars().take(80).collect::<String>())));
                        }
                        margins_measured += 1;
                    }
                }
                steps_checked += 1;
                for (name, verdict) in check_pool_image(observation.image) {
                    if let InvariantVerdict::Violated(why) = verdict {
                        if verbose {
                            eprintln!(
                                "RED {:?} {name}: {}",
                                observation.position,
                                why.chars().take(1500).collect::<String>()
                            );
                        }
                        seen.push(format!(
                            "种子偏移 {index} 位置 {:?} 操作 {:?}：{name} 红：{}",
                            observation.position,
                            observation.operation,
                            why.chars().take(600).collect::<String>()
                        ));
                    }
                }
            },
        );
        if run.ending != HistoryEnding::Completed {
            problems.push(format!(
                "种子偏移 {index}：结局 {}；操作 {:?}",
                format!("{:?}", run.ending).chars().take(1500).collect::<String>(),
                history.operations
            ));
        }
        if let Some(earliest) = seen.first() {
            problems.push(format!("{earliest}（这一段共 {} 处）", seen.len()));
        }
    }
    eprintln!(
        "ATTACK_SUMMARY weights={} seeds={seeds} seed_start={seed_start} ops={ops} steps_checked={steps_checked} problems={}",
        attack_weights().name,
        problems.len()
    );
    if measure_margins {
        eprintln!("ATTACK_CKPT_MARGIN publishes={margins_measured} smallest_margin={} at={}", smallest_margin.0, smallest_margin.1);
    }
    for problem in &problems {
        eprintln!("ATTACK_PROBLEM {problem}");
    }
    for (kind, count) in &kinds {
        eprintln!("ATTACK_KIND {count} {kind}");
    }
    assert!(problems.is_empty(), "{} 段出问题", problems.len());
}
