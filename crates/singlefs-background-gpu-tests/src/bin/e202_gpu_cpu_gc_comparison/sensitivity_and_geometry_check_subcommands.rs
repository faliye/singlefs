//! 子命令 sensitivity（几何敏感性两点）与 geometry-check（只造池、核几何）。

use super::*;

// ---- 子命令 sensitivity（登记第八节「几何敏感性」两点：中档、批大小 16 的交替跑，前台间隔 500 毫秒的并发跑）----

/// 几何敏感性那两点取的档（中档）。
const SENSITIVITY_TIER: UnitAreaTier = UnitAreaTier::FourGibibytes;

/// 批大小 16 那一点：中档、CPU 臂与 GPU 点名一张卡臂各 5 次交替跑（量 S、量 W 的正负）。
fn sensitivity_lockstep_command(product: &Path, user_changes: u64) -> LockstepCommand {
    LockstepCommand {
        tier: SENSITIVITY_TIER,
        user_changes,
        batch_units: BATCH_UNITS_AT_THE_SENSITIVITY_POINT,
        arms_requested: vec![LOCKSTEP_ARMS[0], LOCKSTEP_ARMS[1]],
        cards_of_the_every_card_arm: CardsOfTheEveryCardArm::EveryHandedOverCard,
        product: product.to_path_buf(),
    }
}

/// 前台间隔 500 毫秒那一点：中档、三条臂各 5 次并发跑（量 F 的方向）。
fn sensitivity_concurrent_command(product: &Path) -> ConcurrentCommand {
    ConcurrentCommand {
        tier: SENSITIVITY_TIER,
        foreground_interval: Duration::from_millis(
            FOREGROUND_INTERVAL_MILLISECONDS_AT_THE_SENSITIVITY_POINT,
        ),
        arms_requested: CONCURRENT_ARMS.to_vec(),
        product: product.to_path_buf(),
    }
}

/// 两点串着跑、写进同一份产物：交替跑那一点不是 0 就不跑并发那一点、交回它的退出码。两点共用这个进程取到的那一份卡（[`CARDS_SEEN_BY_THIS_PROCESS`]）。
/// 不拆成先录后判：两点各是交替跑、并发跑，量的都是同一个进程里各臂挨着跑的读数（理由在 [`lockstep_command`]、[`concurrent_command`]）。
pub(super) fn sensitivity_command(product: &Path, user_changes: u64) -> i32 {
    let lockstep_exit = lockstep_command(&sensitivity_lockstep_command(product, user_changes));
    if lockstep_exit != EXIT_SUCCESS {
        return lockstep_exit;
    }
    concurrent_command(&sensitivity_concurrent_command(product))
}

// ---- 子命令 geometry-check（登记第五节「分段」第一段：只造池、不计时、不要卡；第一节未定第 6 项：量每档造池耗时与每次跑的批数）----

/// 一档几何核的结局：哪一档、起步镜像（造池停下的是停在哪），与按阶梯在 CPU 臂上不计时地交替跑出的每一级的批数（够 64 批就不再往上；
/// 造池停下或几何没造成的一级都不跑）。
struct GeometryCheckOfATier {
    tier: UnitAreaTier,
    image: Result<StartingImage, ConstructionStopped>,
    batches_by_user_changes: Vec<(u64, Result<u64, String>)>,
}

fn geometry_check_of_a_tier(tier: UnitAreaTier) -> GeometryCheckOfATier {
    let image_or_the_stop = construct_the_starting_image(tier);
    let mut batches_by_user_changes: Vec<(u64, Result<u64, String>)> = Vec::new();
    if let Some(image) = image_or_the_stop
        .as_ref()
        .ok()
        .filter(|image| image.construction.geometry.is_formed())
    {
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        for user_changes in LOCKSTEP_USER_CHANGES_LADDER {
            let setting = LockstepRunSetting {
                image,
                discrete_gpus: &[],
                discrete_gpus_of_the_every_card_arm: &[],
                memory_leaked: &memory_leaked,
                vulkan_instance: None,
                user_changes,
                batch_units: BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT,
            };
            let batches =
                run_lockstep_once(&setting, LOCKSTEP_ARMS[0], 0).map(|summary| summary.batches);
            let is_enough = batches
                .as_ref()
                .is_ok_and(|batches| *batches >= MINIMUM_BATCHES_PER_RUN);
            let is_failed = batches.is_err();
            batches_by_user_changes.push((user_changes, batches));
            if is_enough || is_failed {
                break;
            }
        }
    }
    GeometryCheckOfATier {
        tier,
        image: image_or_the_stop,
        batches_by_user_changes,
    }
}

pub(super) fn geometry_check_command(product_path: &Path) -> i32 {
    let mut product = match ProductWriter::appending_to(product_path) {
        Ok(product) => product,
        Err(reason) => {
            eprintln!("{reason}");
            return EXIT_RUN_CONDITION_NOT_MET;
        }
    };
    let opening = match opening_of_the_command(
        &mut product,
        product_path,
        "command=geometry-check reading=occupancy",
        CardsOfTheCommand::NotNeeded,
    ) {
        Ok(opening) => opening,
        Err(exit_code) => {
            product.finish();
            return exit_code;
        }
    };
    // mkfs 不认的那一档不造池（作废第 6 条，交主 agent），别的档照造。
    let mut is_every_tier_fine = true;
    let mut tiers_accepted: Vec<UnitAreaTier> = Vec::new();
    for tier in UnitAreaTier::EVERY_TIER {
        if emit_whether_the_make_filesystem_accepts_the_tier(&mut product, tier) {
            tiers_accepted.push(tier);
        } else {
            is_every_tier_fine = false;
        }
    }
    // 各档各一条线程、各建各的池（登记第十四节「独立单位」）；线程数取登记值，实际只用到档数那么多条。
    let threads = opening.registered_heavy_threads.min(tiers_accepted.len());
    let mut checks: Vec<GeometryCheckOfATier> = Vec::new();
    for tiers_of_a_wave in tiers_accepted.chunks(threads.max(1)) {
        let checks_of_the_wave: Vec<GeometryCheckOfATier> = std::thread::scope(|scope| {
            let handles: Vec<_> = tiers_of_a_wave
                .iter()
                .map(|tier| scope.spawn(move || geometry_check_of_a_tier(*tier)))
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("几何核那条线程没有 panic"))
                .collect()
        });
        checks.extend(checks_of_the_wave);
    }
    product.emit(
        "parallelism",
        &format!(
            "geometry_threads={threads} registered_heavy_threads={}",
            opening.registered_heavy_threads
        ),
    );
    for check in &checks {
        let image = match &check.image {
            Ok(image) => image,
            Err(stopped) => {
                emit_the_construction_stop(
                    &mut product,
                    check.tier,
                    check.tier.live_data_units(),
                    stopped,
                );
                is_every_tier_fine = false;
                continue;
            }
        };
        is_every_tier_fine &= emit_the_construction(&mut product, image);
        let mut has_enough_batches = false;
        for (user_changes, batches) in &check.batches_by_user_changes {
            match batches {
                Ok(batches) => {
                    has_enough_batches = *batches >= MINIMUM_BATCHES_PER_RUN;
                    product.emit(
                        "geometry_batches",
                        &format!(
                            "tier={} user_changes={user_changes} batches={batches} enough={has_enough_batches}",
                            check.tier.name()
                        ),
                    );
                }
                Err(reason) => product.emit(
                    "geometry_batches",
                    &format!(
                        "tier={} user_changes={user_changes} failed={}",
                        check.tier.name(),
                        field_value(reason)
                    ),
                ),
            }
        }
        is_every_tier_fine &= has_enough_batches;
    }
    product.emit(
        "verdict",
        &format!(
            "command=geometry-check every_tier_formed_with_enough_batches={is_every_tier_fine}"
        ),
    );
    product.finish();
    if is_every_tier_fine {
        EXIT_SUCCESS
    } else {
        EXIT_STOPPED
    }
}

#[cfg(test)]
mod tests;
