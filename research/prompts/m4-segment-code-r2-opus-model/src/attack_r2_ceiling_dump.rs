//! 攻方腿原型（m4-segment-code-r2，R1 / R8）：整理之后回退、再抬 F 时模型与实现报的上限不同。跑缩出来的那一段历史，
//! 每一步之后把根环里每一条读得出的根的 inode 树、extent 树根指针字节（实现判「非空」与「4 个不同状态」比的就是它们，
//! `mount::user_visible_trees_changed`）摘成短指纹打出来，看回退那条根与它的目标、与它之前那条根的字节是否相同。
use singlefs_core::recovery::{choose_system_configuration, readable_roots, user_visible_tree_root_pointers};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GeneratedHistory, GenerationWeights,
    HistoryExecution, HistoryOperationKind as K, HistorySeed, HistoryStartingPoint,
    NamedGenerationWeights, OverwriteContentDraw, RollbackTargetDraw,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::SharedStream;

include!("attack_common/weights_r2.rs");

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// ATTACK_DEVICE_WIDTH：384 / 256 / 240 取小盘那三档，别的取两块 4 GiB 的盘（同随机历史原型）。
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

/// 字节串的短指纹（FNV-1a 64 位）：只用来看两条根的指针字节相同还是不同。
fn fingerprint(bytes: &Option<Vec<u8>>) -> String {
    match bytes {
        None => "无".to_string(),
        Some(bytes) => {
            let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
            for byte in bytes {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x0100_0000_01b3);
            }
            format!("{:04x}", hash & 0xffff)
        }
    }
}

#[test]
fn dump_the_root_pointers_of_every_ring_root_after_each_step() {
    let index = env_u64("ATTACK_SEED_INDEX", 476);
    let ops = usize::try_from(env_u64("ATTACK_OPS", 40)).expect("步数");
    let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + ATTACK_SEED_OFFSET + index;
    let full = generate_history_with_weights(HistorySeed(seed), ops, attack_weights());
    let keep: Vec<usize> = std::env::var("ATTACK_KEEP")
        .expect("ATTACK_KEEP")
        .split(',')
        .map(|v| v.parse().expect("下标"))
        .collect();
    let history = GeneratedHistory {
        seed: full.seed,
        starting_point: full.starting_point,
        operations: keep.iter().map(|i| full.operations[*i]).collect(),
    };
    let run = execute_history_with(
        &history,
        execution(),
        &SharedStream::new(),
        &mut |observation| {
            eprintln!(
                "STEP {:?} op={:?} outcome={}",
                observation.position,
                observation.operation,
                observation
                    .outcome
                    .map(|o| format!("{o:?}").chars().take(300).collect::<String>())
                    .unwrap_or_default()
            );
            let image = observation.image;
            let Ok(configuration) = choose_system_configuration(image) else {
                eprintln!("ROOTS 系统配置择不出");
                return;
            };
            let mut roots = readable_roots(
                image,
                &configuration.immutable.region_devices,
                &configuration.immutable.sizes,
                &configuration.immutable.filesystem_identifier,
            );
            roots.sort_by_key(|root| (root.checkpoint_txg, root.instance));
            let described: Vec<String> = roots
                .iter()
                .map(|root| match user_visible_tree_root_pointers(image, root) {
                    Ok(pointers) => format!(
                        "{}@{}:i{}e{}",
                        root.checkpoint_txg.0,
                        root.instance.0,
                        fingerprint(&pointers.inode_tree),
                        fingerprint(&pointers.extent_tree)
                    ),
                    Err(_) => format!("{}@{}:读不出", root.checkpoint_txg.0, root.instance.0),
                })
                .collect();
            eprintln!("ROOTS {}", described.join(" "));
        },
    );
    eprintln!("ENDING {}", format!("{:?}", run.ending).chars().take(1200).collect::<String>());
}
