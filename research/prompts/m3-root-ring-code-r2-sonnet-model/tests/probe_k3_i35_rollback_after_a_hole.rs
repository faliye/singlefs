//! 探针（正推腿，K3）：写行崩溃留下的洞之后，回退到洞之前的根，再接着发布，逐版看 I-3.5（缺口那一对上回退带回旧树）。
mod common;
mod common_root_ring_holes;

use common_root_ring_holes::{
    content_of_the_version, crash_a_mount_before_its_row_root, devices_of,
    parameters_of_the_ring_hole_pools, pool_of, pool_overwritten_through, publish_empty,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::MemoryPool;

fn summary(label: &str, image: &MemoryPool) -> (String, Vec<String>) {
    let mut i35 = "缺".to_string();
    let mut reds: Vec<String> = Vec::new();
    for (invariant, verdict) in &check_pool_image(image) {
        match verdict {
            InvariantVerdict::Violated(_) => {
                if *invariant == "I-3.5" {
                    i35 = "违例".to_string();
                }
                reds.push((*invariant).to_string());
            }
            InvariantVerdict::Holds => {
                if *invariant == "I-3.5" {
                    i35 = "成立".to_string();
                }
            }
            InvariantVerdict::NotApplicable(_) => {
                if *invariant == "I-3.5" {
                    i35 = "不适用".to_string();
                }
            }
        }
    }
    println!("K3RB {label} :: I-3.5={i35} :: 红={reds:?}");
    (i35, reds)
}

#[test]
fn probe_i35_after_a_rollback_to_a_root_before_a_hole() {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut judged = 0usize;
    let mut i35_red = 0usize;
    let mut any_red = 0usize;
    let mut rollbacks_done = 0usize;
    for last in [10u64, 12, 14] {
        for hole_count in 1..=2usize {
            let mut image = pool_overwritten_through(last);
            let mut hole_txgs = Vec::new();
            for _ in 0..hole_count {
                let (crashed, row_txg) = crash_a_mount_before_its_row_root(&image);
                hole_txgs.push(row_txg);
                image = crashed;
            }
            for back in [1u64, 3] {
                let target_txg = last - back;
                let mut devices = devices_of(&image);
                let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
                let instance = mounted.output.instance;
                let mut allocator = mounted.allocator.clone();
                let mut current = mounted.current.into_file_version().expect("这一版带文件");
                for _ in 0..2 {
                    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                    current = publish_empty(&mut writer, &mut allocator, &current, instance).expect("空发布");
                }
                let rolled = roll_back_by_a_forward_publish(
                    &parameters,
                    &mut devices,
                    &mut allocator,
                    &mut current,
                    RollbackTarget {
                        instance: InstanceGeneration(1),
                        checkpoint_txg: CheckpointTxg(target_txg),
                    },
                );
                let label0 = format!("last={last} holes={hole_txgs:?} 回退到 {target_txg}");
                if let Err(error) = &rolled {
                    println!("K3RB {label0} :: 回退被拒 {error:?}");
                    continue;
                }
                rollbacks_done += 1;
                let (v, reds) = summary(&format!("{label0} 回退之后 txg={}", current.root.checkpoint_txg.0), &pool_of(&devices));
                judged += 1;
                i35_red += usize::from(v == "违例");
                any_red += usize::from(!reds.is_empty());
                for step in 0..6u64 {
                    {
                        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                        if step % 2 == 1 {
                            let salt = 7000 + step;
                            current = publish_overwrite(
                                &mut writer,
                                &mut allocator,
                                &current,
                                FirstFile {
                                    content: &content_of_the_version(salt),
                                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
                                },
                                instance,
                            )
                            .expect("覆盖写");
                        } else {
                            current = publish_empty(&mut writer, &mut allocator, &current, instance).expect("空发布");
                        }
                    }
                    let (v, reds) = summary(&format!("{label0} 之后第 {step} 步 txg={}", current.root.checkpoint_txg.0), &pool_of(&devices));
                    judged += 1;
                    i35_red += usize::from(v == "违例");
                    any_red += usize::from(!reds.is_empty());
                }
            }
        }
    }
    println!("K3RB_TOTAL judged={judged} rollbacks_done={rollbacks_done} i35_red={i35_red} any_red={any_red}");
}
