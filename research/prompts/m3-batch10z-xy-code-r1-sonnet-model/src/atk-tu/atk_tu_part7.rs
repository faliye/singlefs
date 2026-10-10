
/// 一格（前缀覆盖写、文件长到几个单元、g）的 K1 形：挂上之后那一刻的池与那次写的长度（单元数）。
fn atk_k1_cell(pre: u64, fill: usize, grow: usize) -> (AtkSnap, usize) {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let (_base, crashed, contents, write_time_seconds) = atk_k1_record_on(384, pre, fill, 5).expect("录得下");
    let (pool, _) = atk_mounted_pool_from(
        &parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        &crashed,
        &contents,
        write_time_seconds,
    )
    .expect("挂");
    let size_now = pool.content_of_the_current_version.len().div_ceil(unit_bytes);
    (AtkSnap::of(&pool), size_now + grow)
}

fn atk_cells_from_env() -> Vec<(u64, usize, usize)> {
    std::env::var("ATK_CELLS")
        .unwrap_or_else(|_| "4:16:13,4:16:19,4:16:20,2:16:17,8:11:40".to_string())
        .split(',')
        .map(|cell| {
            let parts: Vec<usize> = cell.split(':').map(|text| text.parse().expect("数")).collect();
            (u64::try_from(parts[0]).expect("数"), parts[1], parts[2])
        })
        .collect()
}

/// K1-2 一：被拒一次之后再原样发一次（第二报），第 n 次读报错一次（n 扫过这一次调用的每一次读）：报成什么、会话动没动；
/// 真结局（不带故障照出路二走）另跑一次。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_transient_read_error_while_reporting() {
    let step: usize = std::env::var("ATK_STEP_N").map_or(1, |text| text.parse().expect("数"));
    for (pre, fill, grow) in atk_cells_from_env() {
        let (mounted, units) = atk_k1_cell(pre, fill, grow);
        let mut pool = mounted.pool();
        let write = atk_write_of(&mut pool, units);
        let first = atk_apply(&mut pool, &write);
        assert!(atk_stuck_of(&first).is_some(), "这一格第一次报被宽护卡住：{}", atk_kind(&first));
        let after_report = AtkSnap::of(&pool);
        let truth = atk_follow_on(&mut after_report.pool(), &write);
        let dry_plan = AtkReadPlan::new(None);
        let mut dry = after_report.pool_with(&dry_plan);
        let dry_outcome = atk_apply(&mut dry, &write);
        let total_reads = dry_plan.reads.get();
        let base_head = atk_stuck_of(&dry_outcome).map_or_else(|| atk_kind(&dry_outcome), atk_rehearsal_head);
        let base_debug = format!("{dry_outcome:?}");
        let mut tally: BTreeMap<String, usize> = BTreeMap::new();
        let mut session_changed = 0usize;
        let mut first_n_of: BTreeMap<String, u64> = BTreeMap::new();
        for n in (1..=total_reads).step_by(step) {
            let plan = AtkReadPlan::new(Some(n));
            let mut faulted = after_report.pool_with(&plan);
            let allocator_before = format!("{:?}", faulted.session().allocator);
            let outcome = atk_apply(&mut faulted, &write);
            let same_debug = format!("{outcome:?}") == base_debug;
            let head = match atk_stuck_of(&outcome) {
                Some(stuck) => format!("stuck:{} ways={} fired={} same_as_without_fault={same_debug}", atk_rehearsal_head(stuck), atk_ways(stuck).ways_out.len(), plan.fired.get()),
                None => format!("{} fired={}", atk_kind(&outcome).chars().take(70).collect::<String>(), plan.fired.get()),
            };
            if atk_stuck_of(&outcome).is_some() && format!("{:?}", faulted.session().allocator) != allocator_before {
                session_changed += 1;
            }
            first_n_of.entry(head.clone()).or_insert(n);
            *tally.entry(head).or_insert(0) += 1;
        }
        println!("ATK k1rd pre={pre} fill={fill} g={grow} truth={truth} base={base_head} reads={total_reads} step={step} session_changed={session_changed}");
        for (head, count) in &tally {
            println!("ATK k1rd pre={pre} fill={fill} g={grow}   {count:5} × {head} (first n={})", first_n_of[head]);
        }
    }
}

/// K1-2 二：C 那一圈罩着的、隔离着的槽从挂上之后就读不出（写也不撤，`DeviceWithReadFaultsOnChosenSlots` 那一种故障模型），
/// 发那次写：报什么、预演说什么；照出路二在同一组故障上真走一遍。对照：不带故障。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_persistent_read_faults_on_the_isolated_c_slots() {
    for (pre, fill, grow) in atk_cells_from_env() {
        let (mounted, units) = atk_k1_cell(pre, fill, grow);
        let probe = mounted.pool();
        let allocator = &probe.session().allocator;
        let occupancy = allocator.root_ring_occupancy().expect("根环表");
        let placements: Vec<PlacementOnDevice> = occupancy
            .ring_slots_holding_placements_named_by_unapplied_journal_records()
            .into_values()
            .flatten()
            .collect();
        let mut ranges = Vec::new();
        for placement in &placements {
            for slot in placement.slot.0..placement.slot.0 + placement.span {
                let map = allocator.devices.iter().find(|map| map.device == placement.device).expect("盘");
                if map.is_isolated(SlotNumber(slot)) {
                    let start = SlotNumber(slot).to_device_offset().0;
                    ranges.push((placement.device, start, start + singlefs_format::SLOT_BYTES));
                }
            }
        }
        let plan = AtkReadPlan::new(None);
        *plan.ranges.borrow_mut() = ranges.clone();
        plan.active.set(true);
        let mut pool = mounted.pool_with(&plan);
        let write = atk_write_of(&mut pool, units);
        let outcome = atk_apply(&mut pool, &write);
        let fired_before_follow = plan.fired.get();
        let report = match atk_stuck_of(&outcome) {
            Some(stuck) => format!("stuck:{} ways={}", atk_rehearsal_head(stuck), atk_ways(stuck).ways_out.len()),
            None => atk_kind(&outcome),
        };
        let follow = if atk_stuck_of(&outcome).is_some() { atk_follow_on(&mut pool, &write) } else { "-".to_string() };
        let mut control = mounted.pool();
        let control_write = write.clone();
        let control_outcome = atk_apply(&mut control, &control_write);
        let control_report = atk_stuck_of(&control_outcome).map_or_else(|| atk_kind(&control_outcome), atk_rehearsal_head);
        println!(
            "ATK k1pf pre={pre} fill={fill} g={grow} faulted_slots={} report={report} fired_in_report={fired_before_follow} follow={follow} fired_total={} control_report={control_report}",
            ranges.len(),
            plan.fired.get()
        );
    }
}

/// K1-2 三：被拒一次之后再原样发一次（第二报），第 n 次读读的那一段从这一次起一直读不出（写也不撤）：第二报的预演说什么，
/// 照出路二在同一组故障上真走一遍是什么（n 扫过这一次调用的每一次读）。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_location_turning_unreadable_while_reporting() {
    let step: usize = std::env::var("ATK_STEP_N").map_or(1, |text| text.parse().expect("数"));
    for (pre, fill, grow) in atk_cells_from_env() {
        let (mounted, units) = atk_k1_cell(pre, fill, grow);
        let mut pool = mounted.pool();
        let write = atk_write_of(&mut pool, units);
        let first = atk_apply(&mut pool, &write);
        assert!(atk_stuck_of(&first).is_some(), "这一格第一次报被宽护卡住");
        let after_report = AtkSnap::of(&pool);
        let dry_plan = AtkReadPlan::new(None);
        let mut dry = after_report.pool_with(&dry_plan);
        let _ = atk_apply(&mut dry, &write);
        let total_reads = dry_plan.reads.get();
        let mut tally: BTreeMap<String, usize> = BTreeMap::new();
        let mut first_n_of: BTreeMap<String, u64> = BTreeMap::new();
        for n in (1..=total_reads).step_by(step) {
            let plan = AtkReadPlan::new(None);
            plan.pin_at.set(Some(n));
            let mut faulted = after_report.pool_with(&plan);
            let outcome = atk_apply(&mut faulted, &write);
            let fired_in_report = plan.fired.get();
            let cell = match atk_stuck_of(&outcome) {
                Some(stuck) => {
                    let head = atk_rehearsal_head(stuck);
                    let follow = atk_follow_on(&mut faulted, &write);
                    let admitted = follow.ends_with("retry=admitted");
                    let agree = (head == "adm") == admitted;
                    format!("stuck:{head} | follow:{} agree={agree}", follow.chars().take(60).collect::<String>())
                }
                None => format!("{}", atk_kind(&outcome).chars().take(70).collect::<String>()),
            };
            let cell = format!("{cell} fired_in_report={}", fired_in_report.min(2));
            first_n_of.entry(cell.clone()).or_insert(n);
            *tally.entry(cell).or_insert(0) += 1;
        }
        println!("ATK k1pin pre={pre} fill={fill} g={grow} reads={total_reads} step={step}");
        for (cell, count) in &tally {
            println!("ATK k1pin pre={pre} fill={fill} g={grow}   {count:5} × {cell} (first n={})", first_n_of[cell]);
        }
    }
}
