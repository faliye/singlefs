//! m3-ring-r1 攻方原型 H1 / H3：不枚举崩溃状态，直接在前缀镜像上做被测的可写挂载，再做 K 次覆盖写，每一步量盘上门槛与多扣、跑池级 checker。
//! 用法：ring_hole_direct <S> <front|back> <挂点 0..5> <K>
//! 环境变量：CRASH_THE_ROW_PUBLISH=1 把洞造在新实例的写行那次发布上（不可补的那一类）；
//! ROW_CRASH_REPEATS=n 在被测挂载之前再连崩 n 次可写挂载（每次都崩在写行那次根槽写之前，挂点开关已开）；
//! ROLL_BACK_BY=d 在 K 次覆盖写之后回退到 最新 − d 那条根、再覆盖写 3 次；
//! HOLE_FILL_BYTES=zero|chosen 必红世界；I39_SINGLE_VALUE=1 把 I-3.9 收成单值判；SHOW_VIOLATION_DETAIL=1 打印红的细节。
use ring_hole_proto::{
    build_prefix, content, crash_a_mount_before_its_row_root, devices_of, disk_metrics, hole_txg,
    occupant_of_the_hole_slot, parameters, pool_of, readable_root_at_txg, violated, HolePosition,
    WRITE_TIME,
};
use singlefs_core::mount::{
    mount_writable, roll_back_by_a_forward_publish, set_hole_fill_point, HoleFillPoint,
    RollbackTarget,
};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let slots_per_region: u64 = arguments[1].parse().expect("S");
    let position = if arguments[2] == "front" { HolePosition::Front } else { HolePosition::Back };
    let point = match arguments[3].as_str() {
        "0" => HoleFillPoint::NotFilled,
        "1" => HoleFillPoint::P1AfterAcquisition,
        "2" => HoleFillPoint::P2BeforeRowRoot,
        "3" => HoleFillPoint::P2AfterRowRoot,
        "4" => HoleFillPoint::P3AfterWarmUp,
        "5" => HoleFillPoint::P2RecordedBeforeRowPublish,
        other => panic!("挂点 {other}"),
    };
    let steps: u64 = arguments[4].parse().expect("K");
    let parameters = parameters(slots_per_region);
    let prefix = build_prefix(slots_per_region, hole_txg(slots_per_region, position));
    let hole = prefix.hole;
    set_hole_fill_point(point);
    let mut image = prefix.image.clone();
    let repeats: u64 = std::env::var("ROW_CRASH_REPEATS").ok().map_or(0, |v| v.parse().expect("n"));
    let mut row_holes = Vec::new();
    for _ in 0..repeats {
        let (crashed, row_txg) = crash_a_mount_before_its_row_root(slots_per_region, &image);
        image = crashed;
        row_holes.push(row_txg);
    }
    let mut devices = devices_of(&image);
    let mounted = mount_writable(&parameters, &mut devices).expect("被测的挂载");
    let mut peak_window = 0u64;
    let mut red_steps = 0u64;
    let mut print = |step: String, image: &singlefs_harness::memory_pool::MemoryPool| {
        let m = disk_metrics(image).expect("盘面");
        let red = violated(image);
        if !red.is_empty() {
            red_steps += 1;
        }
        let window_over = m.withheld_by_floor - m.withheld_by_window_floor;
        peak_window = peak_window.max(window_over);
        println!(
            "PROTO name=direct s={slots_per_region} position={position:?} point={point:?} hole={hole} step={step} newest={} oldest_valid={} floor={} missing={:?} withheld={} over_withheld={} hole_slot={:?} window_floor={} withheld_under_window_floor={} outside_window={:?} candidates={} violated={:?}",
            m.newest_txg, m.oldest_valid, m.floor, m.missing_txgs, m.withheld_by_floor, m.over_withheld, occupant_of_the_hole_slot(image, hole), m.window_floor, m.withheld_by_window_floor, m.valid_roots_outside_the_window, m.candidate_txgs.len(), red
        );
        m.over_withheld
    };
    let mut peak = print("mounted".to_string(), &pool_of(&devices));
    println!(
        "PROTO name=direct_mount chosen={} effective={} row={} instance={} row_crash_holes={row_holes:?}",
        mounted.output.chosen_root.checkpoint_txg.0,
        mounted.output.effective_root.checkpoint_txg.0,
        mounted.output.row_publish.root().checkpoint_txg.0,
        mounted.output.instance.0
    );
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator;
    let mut current = mounted.current.into_file_version().expect("带文件");
    let mut salt = 3000u64;
    let mut overwrite = |label: String,
                         devices: &mut ring_hole_proto::Devices,
                         allocator: &mut singlefs_core::allocator::PoolAllocator,
                         current: &mut singlefs_core::transaction::TransactionOutput,
                         peak: &mut u64| {
        salt += 1;
        let bytes = content(salt);
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        *current = publish_overwrite(&mut pool, allocator, current, FirstFile { content: &bytes, write_time_seconds: WRITE_TIME + salt }, instance).expect("覆盖写");
        drop(pool);
        *peak = (*peak).max(print(label, &pool_of(devices)));
    };
    for step in 0..steps {
        overwrite(format!("overwrite{step}"), &mut devices, &mut allocator, &mut current, &mut peak);
    }
    if let Some(back) = std::env::var("ROLL_BACK_BY").ok().map(|v| v.parse::<u64>().expect("d")) {
        let target_txg = current.root.checkpoint_txg.0 - back;
        match readable_root_at_txg(&pool_of(&devices), target_txg) {
            None => println!("PROTO name=direct_rollback target={target_txg} outcome=no_such_root"),
            Some(target) => {
                let rolled = roll_back_by_a_forward_publish(&parameters, &mut devices, &mut allocator, &mut current, RollbackTarget { instance: target.instance, checkpoint_txg: target.checkpoint_txg });
                println!("PROTO name=direct_rollback target={target_txg} outcome={}", match &rolled { Ok(_) => "ok".to_string(), Err(e) => format!("{e:?}").chars().take(80).collect() });
                if rolled.is_ok() {
                    for step in 0..3 {
                        overwrite(format!("after_rollback{step}"), &mut devices, &mut allocator, &mut current, &mut peak);
                    }
                }
            }
        }
    }
    drop(overwrite);
    drop(print);
    println!("PROTO name=direct_peak s={slots_per_region} position={position:?} point={point:?} hole={hole} row_crash_repeats={repeats} peak_over_withheld={peak} peak_released_by_window_floor={peak_window} red_steps={red_steps}");
}
