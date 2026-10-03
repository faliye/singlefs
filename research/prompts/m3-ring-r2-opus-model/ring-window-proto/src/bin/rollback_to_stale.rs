//! m3-ring-r2 攻方原型 G2：一圈窗口只改分配器（与 checker）、回退候选集不改时，K1 那一类洞里那条比一圈还旧的根 R 还能不能被回退选中、
//! 回退之后读到的是什么。SINGLEFS_ROLLBACK_NO_WINDOW=1 关掉副本里回退候选集那一道窗口。
//!
//! 用法：rollback_to_stale <S> <最多覆盖写次数>
use ring_window_proto::{build_prefix, content, devices_of, parameters, pool_of, violated, WRITE_TIME};
use singlefs_core::address::CheckpointTxg;
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::recovery::{choose_system_configuration, readable_roots_with_ring_slots};
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let max_k: u64 = a[2].parse().expect("K");
    let parameters = parameters(s);
    let hole = 3 * s + 4;
    std::env::set_var("CRASH_THE_ROW_PUBLISH", "1");
    let prefix = build_prefix(s, hole);
    std::env::remove_var("CRASH_THE_ROW_PUBLISH");
    let c = choose_system_configuration(&prefix.image).expect("系统配置");
    let target_slot = target_for_publish(CheckpointTxg(hole), c.immutable.sizes.root_ring_slots_per_region);
    let stale = readable_roots_with_ring_slots(&prefix.image, &c.immutable.region_devices, &c.immutable.sizes, &c.immutable.filesystem_identifier)
        .into_iter().find(|(slot, _)| *slot == target_slot).map(|(_, r)| r).expect("R");
    for k in 0..=max_k {
        let mut devices = devices_of(&prefix.image);
        let mounted = mount_writable(&parameters, &mut devices).expect("挂载");
        let instance = mounted.output.instance;
        let mut allocator = mounted.allocator.clone();
        let mut current = mounted.current.clone().into_file_version().expect("带文件");
        for step in 0..k {
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(400 + step), write_time_seconds: WRITE_TIME + 400 + step }, instance).expect("覆盖写");
        }
        let outcome = roll_back_by_a_forward_publish(&parameters, &mut devices, &mut allocator, &mut current, RollbackTarget { instance: stale.instance, checkpoint_txg: stale.checkpoint_txg });
        let text = match &outcome {
            Ok(_) => {
                let red = violated(&pool_of(&devices));
                format!("ok checker={red:?}")
            }
            Err(e) => format!("refused {}", format!("{e:?}").chars().take(90).collect::<String>()),
        };
        println!("ROLLBACK s={s} anchor={:?} rollback_window_off={} stale_txg={} overwrites_before={k} current_txg={} outcome={text}",
            singlefs_core::mount::window_anchor(), std::env::var("SINGLEFS_ROLLBACK_NO_WINDOW").unwrap_or_default(), stale.checkpoint_txg.0, current.root.checkpoint_txg.0);
    }
}
