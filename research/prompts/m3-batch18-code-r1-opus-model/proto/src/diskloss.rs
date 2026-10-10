//! 丢哪块盘：已发布那一组里进程内删除的历史（4 GiB、时间与记录同、删完不再建），每次发布之后的镜像各丢一块盘（那块盘整块读出全 0），
//! 跑池级 checker，看删除线五条与 I-9.6 有没有报不成立。别的条因为丢盘红是预期的（系统配置、根环那几条），只记数、不算误红。
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_core::address::DeviceIdentity;
use singlefs_harness::memory_pool::SparseDevice;

use crate::judge::{judge, WATCHED};
use crate::sweep::{histories, run_history, Mode, TimeChoice};

pub fn run(threads: usize) {
    let all: Vec<_> = histories(false)
        .into_iter()
        .filter(|history| {
            matches!(history.mode, Mode::InProcess)
                && matches!(history.time, TimeChoice::SameAsRecord)
                && history.device_bytes == 4 << 30
                && history.new_inodes_after_and_unlink_one == 0
        })
        .collect();
    let next = AtomicUsize::new(0);
    let lines: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::SeqCst);
                let Some(history) = all.get(index) else { break };
                let images = run_history(history).expect("进程内那一形都走得到");
                let mut judged_images = 0u64;
                let mut watched_red = Vec::new();
                let mut other_red = std::collections::BTreeMap::<&'static str, u64>::new();
                let mut watched_kinds = std::collections::BTreeMap::<String, u64>::new();
                for (label, image) in &images {
                    for lost in [DeviceIdentity(0), DeviceIdentity(1)] {
                        let mut degraded = image.clone();
                        degraded.devices.insert(lost, SparseDevice::default());
                        let judged = judge(&degraded);
                        judged_images += 1;
                        for (invariant, kind, _) in &judged.watched {
                            *watched_kinds.entry(format!("{invariant}={kind}")).or_insert(0) += 1;
                        }
                        for (invariant, text) in &judged.violated {
                            if WATCHED.contains(invariant) {
                                watched_red.push(format!("[{label} 丢盘{}] {invariant}：{text}", lost.0));
                            } else {
                                *other_red.entry(*invariant).or_insert(0) += 1;
                            }
                        }
                    }
                }
                let line = format!(
                    "DISKLOSS index={index} name={} images={judged_images} watched_red={} other_red={other_red:?} watched={watched_kinds:?}{}",
                    history.name(),
                    watched_red.len(),
                    watched_red.first().map_or(String::new(), |first| format!(" first_watched_red={first}"))
                );
                lines.lock().expect("锁").push((index, line));
            });
        }
    });
    let mut lines = lines.into_inner().expect("锁");
    lines.sort();
    for (_, line) in &lines {
        println!("{line}");
    }
    let images: u64 = lines.iter().filter_map(|(_, line)| line.split(" images=").nth(1)?.split(' ').next()?.parse::<u64>().ok()).sum();
    let red: usize = lines.iter().filter(|(_, line)| !line.contains(" watched_red=0 ")).count();
    println!("DISKLOSS_SUMMARY histories={} images_judged={images} histories_with_watched_red={red}", lines.len());
}
