//! 探针（正推腿，K3）：在「写行那次崩在根槽之前」造出的合法洞上，逐版看 I-3.5（与别的违例）判成什么。
//! 盘面搭法照 `raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs`（同一个 common_root_ring_holes）。
mod common;
mod common_root_ring_holes;

use common_root_ring_holes::{
    content_of_the_version, crash_a_mount_before_its_row_root, devices_of,
    parameters_of_the_ring_hole_pools, pool_of, pool_overwritten_through, publish_empty,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::MemoryPool;

fn summary(label: &str, image: &MemoryPool) -> (bool, bool) {
    let verdicts = check_pool_image(image);
    let mut i35 = "缺".to_string();
    let mut reds: Vec<String> = Vec::new();
    for (invariant, verdict) in &verdicts {
        match verdict {
            InvariantVerdict::Violated(detail) => {
                if *invariant == "I-3.5" {
                    i35 = format!("违例：{}", detail.chars().take(160).collect::<String>());
                }
                reds.push((*invariant).to_string());
            }
            InvariantVerdict::Holds => {
                if *invariant == "I-3.5" {
                    i35 = "成立".to_string();
                }
            }
            InvariantVerdict::NotApplicable(reason) => {
                if *invariant == "I-3.5" {
                    i35 = format!("不适用：{}", reason.chars().take(40).collect::<String>());
                }
            }
        }
    }
    println!("K3PROBE {label} :: I-3.5={i35} :: 红={reds:?}");
    (i35.starts_with("违例"), !reds.is_empty())
}

#[test]
fn probe_i35_over_histories_with_holes_from_a_crashed_row_publish() {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut i35_red = 0usize;
    let mut any_red = 0usize;
    let mut images = 0usize;
    for last in [8u64, 11, 14, 17, 20] {
        for hole_count in 0..=3usize {
            let mut image = pool_overwritten_through(last);
            let mut hole_txgs = Vec::new();
            for _ in 0..hole_count {
                let (crashed, row_txg) = crash_a_mount_before_its_row_root(&image);
                hole_txgs.push(row_txg);
                image = crashed;
                let (a, b) = summary(&format!("last={last} holes={hole_txgs:?} 崩溃之后"), &image);
                images += 1;
                i35_red += usize::from(a);
                any_red += usize::from(b);
            }
            let mut devices = devices_of(&image);
            let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
            let instance = mounted.output.instance;
            let mut allocator = mounted.allocator.clone();
            let mut current = mounted.current.into_file_version().expect("这一版带文件");
            let (a, b) = summary(&format!("last={last} holes={hole_txgs:?} 挂载之后 txg={}", current.root.checkpoint_txg.0), &pool_of(&devices));
            images += 1;
            i35_red += usize::from(a);
            any_red += usize::from(b);
            for step in 0..14u64 {
                {
                    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                    if step % 3 == 2 {
                        let salt = 9000 + step;
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
                let (a, b) = summary(&format!("last={last} holes={hole_txgs:?} 之后第 {step} 步 txg={}", current.root.checkpoint_txg.0), &pool_of(&devices));
                images += 1;
                i35_red += usize::from(a);
                any_red += usize::from(b);
            }
        }
    }
    println!("K3PROBE_TOTAL images={images} i35_red={i35_red} any_red={any_red}");
}
