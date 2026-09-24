//! m2-wave3-code-r1 云端攻方腿（Opus）Y3 / Y4 的驱动：记录已持久、根槽没落盘（以及同一次发布里每个前缀）处崩溃 →
//! 可写挂载（施加记录）→ 放开扫挂载之后的用户动作（会话内连续覆盖写 0..N 次，每一步判；之后关掉重开再判），
//! 看候选 b 四条认法与 I-3.10 读集在这些合法历史上判出什么。只打印，不断言；副本上跑，不进仓。

mod common;
#[path = "opus_attack_y1.rs"]
#[allow(dead_code)]
mod y1;

use common::{parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::make_filesystem::{make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion, PoolWriter,
    TransactionOutput,
};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::SharedStream;
use y1::{content, devices_of, image_of, kind_name, memory_of};

fn summary(image: &MemoryPool) -> (Vec<String>, String, String) {
    let verdicts = check_pool_image(image);
    let mut red = Vec::new();
    let mut i310 = String::new();
    let mut mech = String::new();
    for (name, verdict) in &verdicts {
        match verdict {
            InvariantVerdict::Violated(text) => {
                red.push(format!("{name}: {text:?}"));
            }
            _ => {}
        }
        if *name == "I-3.10" {
            i310 = format!("{verdict:?}").chars().take(60).collect();
        }
    }
    // 机理标识只在 I-3.1 红时带；这里另从 I-3.1 的说明里抽「并进遍历的由记录施加出来的版本」
    for (name, verdict) in &verdicts {
        if *name == "I-3.1" {
            mech = format!("{verdict:?}").chars().take(40).collect();
        }
    }
    (red, i310, mech)
}

/// mkfs → 取号 → 暖机 → A（txg 3）→ B（txg 4，覆盖写）；交回流与 B 在流里的起止。
fn stream_with_overwrite() -> (SharedStream, usize, usize) {
    let stream = SharedStream::retaining_contents();
    let mut devices = devices_of(&MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES), &stream);
    let params = parameters();
    let genesis = make_filesystem(&params, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let mut pool = PoolWriter::new(&params, &mut devices);
    let instance = acquire_instance(&mut pool).expect("取号");
    let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
    let a = publish_first_file(
        &mut pool,
        &mut allocator,
        warm.roots.last().unwrap(),
        FirstFile { content: &content(3000, 1), write_time_seconds: FIXED_WRITE_TIME_SECONDS },
        instance,
        &warm.last_record_bytes,
    )
    .expect("A");
    let start = stream.operation_count();
    publish_overwrite(
        &mut pool,
        &mut allocator,
        &a,
        FirstFile { content: &content(4100, 3), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
        instance,
    )
    .expect("B");
    let end = stream.operation_count();
    (stream, start, end)
}

fn overwrite(devices: &mut Vec<(DeviceIdentity, y1::Dev)>, allocator: &mut PoolAllocator, previous: &TransactionOutput, instance: InstanceGeneration, seed: usize) -> TransactionOutput {
    let params = parameters();
    let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        allocator,
        previous,
        FirstFile { content: &content(2000 + seed, seed), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 100 + seed as u64 },
        instance,
    )
    .expect("覆盖写")
}

fn main_sweep(stream: &SharedStream, start: usize, end: usize, max_overwrites: usize) {
    for k in start..=end {
        let crash = memory_of(stream, k);
        let recovered = recover(&crash, JournalPolicy::Consult);
        let (red0, i310_0, _) = summary(&crash);
        println!(
            "k={k} last={} crash-image: recovery prefix_applied={} red={:?} I-3.10={}",
            if k > start { kind_name(stream, k - 1) } else { "-".into() },
            recovered.journal.prefix_applied,
            red0,
            i310_0
        );
        let side = SharedStream::retaining_contents();
        let mut devices = devices_of(&crash, &side);
        let mounted = match mount_writable(&parameters(), &mut devices) {
            Ok(m) => m,
            Err(e) => { println!("k={k} mount err {e:?}"); continue; }
        };
        let (red, i310, _) = summary(&image_of(&devices));
        println!("k={k} step=mount root=({},{}) red={:?} I-3.10={}", mounted.current.root().instance.0, mounted.current.root().checkpoint_txg.0, red, i310);
        let PoolVersion::WithFile(mut current) = mounted.current.clone() else { continue };
        let mut allocator = mounted.allocator.clone();
        let instance = mounted.output.instance;
        for n in 1..=max_overwrites {
            current = overwrite(&mut devices, &mut allocator, &current, instance, n);
            let (red, i310, _) = summary(&image_of(&devices));
            if !red.is_empty() || n == max_overwrites {
                println!("k={k} step=overwrite{n} txg={} red={:?} I-3.10={}", current.root.checkpoint_txg.0, red, i310);
            }
        }
        let remounted = mount_writable(&parameters(), &mut devices);
        let (red, i310, _) = summary(&image_of(&devices));
        println!("k={k} step=remount ok={} red={:?} I-3.10={}", remounted.is_ok(), red, i310);
    }
}

#[test]
fn y3_crash_inside_overwrite_then_mount_then_overwrites() {
    let (stream, start, end) = stream_with_overwrite();
    println!("=== overwrite B ops [{start}, {end}) ===");
    main_sweep(&stream, start, end, 26);
}

/// Y4：同一段历史（B 的记录已持久、根槽没落盘），只看崩溃镜像与挂载之后那一张。
/// 配 `repo-mutY4` 那份副本（分配器把 txg 4 新记的分配记录的分配代写成 5）跑：崩溃镜像上 I-3.10 判不判得出。
#[test]
fn y4_residual_record_version_then_mount() {
    let (stream, start, end) = stream_with_overwrite();
    println!("=== Y4 overwrite B ops [{start}, {end}) ===");
    main_sweep(&stream, start, end, 1);
}

/// 只做过 mkfs 的池：第一次可写挂载（写行 + 暖机，树表 0 条那一版上写实例表与一片分配记录节点）里每个前缀崩溃 →
/// 再可写挂载 → 放开扫：什么都不做 / 发第一个文件版本再覆盖写 3 次，每一步判。C533 那两格（记录新根段里没有实例表指针与
/// 分配记录树根）在这条流上走得到。
#[test]
fn y3_crash_inside_first_writable_mount_of_a_formatted_pool() {
    let stream = SharedStream::retaining_contents();
    let mut devices = devices_of(&MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES), &stream);
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let start = stream.operation_count();
    mount_writable(&parameters(), &mut devices).expect("第一次可写挂载");
    let end = stream.operation_count();
    println!("=== first writable mount ops [{start}, {end}) ===");
    for k in start..=end {
        let crash = memory_of(&stream, k);
        let recovered = recover(&crash, JournalPolicy::Consult);
        let (red0, i310_0, _) = summary(&crash);
        println!(
            "k={k} last={} crash-image: prefix_applied={} red={:?} I-3.10={}",
            if k > start { kind_name(&stream, k - 1) } else { "-".into() },
            recovered.journal.prefix_applied, red0, i310_0
        );
        let side = SharedStream::retaining_contents();
        let mut devs = devices_of(&crash, &side);
        let mounted = match mount_writable(&parameters(), &mut devs) {
            Ok(m) => m,
            Err(e) => { println!("k={k} mount err {e:?}"); continue; }
        };
        let (red, i310, _) = summary(&image_of(&devs));
        println!("k={k} step=mount root=({},{}) red={:?} I-3.10={}", mounted.current.root().instance.0, mounted.current.root().checkpoint_txg.0, red, i310);
        let PoolVersion::WithoutFile(version) = &mounted.current else { println!("k={k} with file?"); continue };
        let mut allocator = mounted.allocator.clone();
        let params = parameters();
        let mut current = {
            let mut writer = PoolWriter::new(&params, devs.as_mut_slice());
            publish_first_file(&mut writer, &mut allocator, &version.root,
                FirstFile { content: &content(3000, 1), write_time_seconds: FIXED_WRITE_TIME_SECONDS },
                mounted.output.instance, &version.record_bytes).expect("第一个文件版本")
        };
        let (red, i310, _) = summary(&image_of(&devs));
        println!("k={k} step=first-file txg={} red={:?} I-3.10={}", current.root.checkpoint_txg.0, red, i310);
        for n in 1..=3 {
            current = overwrite(&mut devs, &mut allocator, &current, mounted.output.instance, n);
            let (red, i310, _) = summary(&image_of(&devs));
            println!("k={k} step=overwrite{n} txg={} red={:?} I-3.10={}", current.root.checkpoint_txg.0, red, i310);
        }
        let again = mount_writable(&parameters(), &mut devs);
        let (red, i310, _) = summary(&image_of(&devs));
        println!("k={k} step=remount ok={} red={:?} I-3.10={}", again.is_ok(), red, i310);
    }
}
