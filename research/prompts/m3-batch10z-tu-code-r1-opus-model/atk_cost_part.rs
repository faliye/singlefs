
// ---------- K1-3：预演的代价（只在攻方的量法副本里拼进来：它读副本 core 里加的 ATK_SKIP_REHEARSAL 与 ATK_LAST_REHEARSAL） ----------

fn atk_proc_status_kb(field: &str) -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .expect("读 /proc/self/status")
        .lines()
        .find(|line| line.starts_with(field))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|text| text.parse().ok())
        .expect("那一行")
}

/// 在 `snap` 上发一次长 `units` 个单元的顺序写：交回（结局短名, 挂钟秒, 这一次调用里常驻内存峰值比调用前多了几 MiB）。
fn atk_timed_write(snap: &AtkSnap, units: usize, skip: bool) -> (String, f64, f64) {
    singlefs_core::mounted_session::ATK_SKIP_REHEARSAL.store(skip, std::sync::atomic::Ordering::SeqCst);
    let mut pool = snap.pool();
    let write = atk_write_of(&mut pool, units);
    std::fs::write("/proc/self/clear_refs", "5").expect("重置峰值");
    let rss_before = atk_proc_status_kb("VmRSS:");
    let started = std::time::Instant::now();
    let outcome = atk_apply(&mut pool, &write);
    let seconds = started.elapsed().as_secs_f64();
    let peak = atk_proc_status_kb("VmHWM:");
    let kind = match atk_stuck_of(&outcome) {
        Some(stuck) => format!("stuck:{}", atk_rehearsal_of(stuck).0),
        None => atk_kind(&outcome).chars().take(40).collect(),
    };
    singlefs_core::mounted_session::ATK_SKIP_REHEARSAL.store(false, std::sync::atomic::Ordering::SeqCst);
    (kind, seconds, (peak.saturating_sub(rss_before)) as f64 / 1024.0)
}

/// 第一个让 `pred` 成立的单元数（假定单调）：先倍增、再二分，至多 `limit`。
fn atk_first_units(low: usize, limit: usize, mut pred: impl FnMut(usize) -> bool) -> Option<usize> {
    let mut high = low.max(1);
    while !pred(high) {
        if high >= limit {
            return None;
        }
        high = (high * 2).min(limit);
    }
    let mut low = high / 2;
    while low + 1 < high {
        let mid = (low + high) / 2;
        if pred(mid) { high = mid } else { low = mid }
    }
    Some(high)
}

#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_cost_by_geometry() {
    let areas: Vec<u64> = atk_env_list("ATK_AREAS", "384,4096");
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    for area in areas {
        let started = std::time::Instant::now();
        let (_base, crashed, contents, write_time_seconds) = atk_k1_record_on(area, 4, 16, 5).expect("录得下");
        let (pool, _) = atk_mounted_pool_from(
            &parameters_of_the_384_slot_small_devices(),
            atk_device_bytes_with_unit_area(area),
            &crashed,
            &contents,
            write_time_seconds,
        )
        .expect("挂");
        let snap = AtkSnap::of(&pool);
        let size_now = snap.content_now.len().div_ceil(unit_bytes);
        let c = atk_c_ring(&pool.session().allocator);
        let limit = usize::try_from(area).expect("数") * 2;
        let classify = |units: usize| atk_timed_write(&snap, units, true).0;
        let g_refused = atk_first_units(size_now + 1, limit, |units| classify(units) != "admitted");
        let g_still = g_refused.and_then(|first| {
            atk_first_units(first, limit, |units| {
                let (kind, _, _) = atk_timed_write(&snap, units, false);
                kind != "stuck:adm"
            })
        });
        println!("ATK cost area={area} c={c:?} size_now={size_now} first_refused_units={g_refused:?} first_not_admitted_after_push_units={g_still:?} setup_s={:.1}", started.elapsed().as_secs_f64());
        let (Some(first), Some(still)) = (g_refused, g_still) else { continue };
        let mut cells = vec![first, (first + still) / 2, still - 1, still, still * 2];
        cells.dedup();
        for units in cells {
            let (kind_skip, seconds_skip, peak_skip) = atk_timed_write(&snap, units, true);
            let (kind, seconds, peak) = atk_timed_write(&snap, units, false);
            let (nanos, blocks, bytes, count) = *singlefs_core::mounted_session::ATK_LAST_REHEARSAL.lock().expect("锁");
            println!(
                "ATK cost area={area} units={units} write_mib={:.1} without_rehearsal={kind_skip} {seconds_skip:.3}s peak+{peak_skip:.1}MiB | with={kind} {seconds:.3}s peak+{peak:.1}MiB rehearsal={:.3}s overlay_blocks={blocks} overlay_mib={:.1} rehearsals_so_far={count}",
                (units * unit_bytes) as f64 / 1048576.0,
                nanos as f64 / 1e9,
                bytes as f64 / 1048576.0
            );
        }
    }
}

/// K1-3 二：盘放大（单元区 384 槽到几百万槽），同一个 K1 形（C 一槽、隔离 102 槽）：一次普通覆盖写、一次真推环、一次预演推环加覆盖写各要多久、
/// 这一次调用里常驻内存峰值多了多少，与分配器拷一份要多久。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_rehearsal_cost_by_device_size() {
    let areas: Vec<u64> = atk_env_list("ATK_AREAS", "384,4096,65536");
    for area in areas {
        let started = std::time::Instant::now();
        let (_base, crashed, contents, write_time_seconds) = atk_k1_record_on(area, 4, 16, 5).expect("录得下");
        let (pool, _) = atk_mounted_pool_from(&parameters_of_the_384_slot_small_devices(), atk_device_bytes_with_unit_area(area), &crashed, &contents, write_time_seconds).expect("挂");
        let snap = AtkSnap::of(&pool);
        let setup = started.elapsed().as_secs_f64();
        let clone_started = std::time::Instant::now();
        for _ in 0..5 {
            std::hint::black_box(pool.session().allocator.clone());
        }
        let clone_seconds = clone_started.elapsed().as_secs_f64() / 5.0;
        let measure = |what: &str| -> (f64, f64, String) {
            let mut fresh = snap.pool();
            fresh.write_time_seconds += 1;
            let content = common_unwitnessed_publish::content_of(OVERWRITE_BYTES, fresh.write_time_seconds);
            let change = AtkUserChange::Overwrite(AtkFirstFile { content: &content, write_time_seconds: fresh.write_time_seconds });
            std::fs::write("/proc/self/clear_refs", "5").expect("重置峰值");
            let rss_before = atk_proc_status_kb("VmRSS:");
            let t = std::time::Instant::now();
            let note = match what {
                "overwrite" => atk_kind(&fresh.publish(change, Some(&content))),
                "push" => format!("{:?}", push_the_root_ring_of(&mut fresh).map(|pushed| pushed.versions_published.len())).chars().take(30).collect(),
                _ => {
                    let session = fresh.session.take().expect("会话");
                    let rehearsal = singlefs_core::mounted_session::atk_rehearse_for(&session, &fresh.devices, change);
                    fresh.session = Some(session);
                    match rehearsal {
                        RootRingPushRehearsal::ChangeAdmittedAfterThePush { .. } => "adm".to_string(),
                        RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. } => "still".to_string(),
                        RootRingPushRehearsal::PushRefused(refusal) => format!("pushref({refusal:?})").chars().take(40).collect(),
                    }
                }
            };
            let seconds = t.elapsed().as_secs_f64();
            let peak = atk_proc_status_kb("VmHWM:");
            (seconds, (peak.saturating_sub(rss_before)) as f64 / 1024.0, note)
        };
        let (ow_s, ow_mib, ow_note) = measure("overwrite");
        let (push_s, push_mib, push_note) = measure("push");
        let (reh_s, reh_mib, reh_note) = measure("rehearsal");
        let (_, blocks, bytes, _) = *singlefs_core::mounted_session::ATK_LAST_REHEARSAL.lock().expect("锁");
        println!(
            "ATK costsize area={area} device_gib={:.2} setup_s={setup:.1} allocator_clone_s={clone_seconds:.4} overwrite={ow_note} {ow_s:.3}s peak+{ow_mib:.1}MiB | push={push_note} {push_s:.3}s peak+{push_mib:.1}MiB | rehearsal={reh_note} {reh_s:.3}s peak+{reh_mib:.1}MiB overlay_blocks={blocks} overlay_mib={:.1}",
            atk_device_bytes_with_unit_area(area) as f64 / (1u64 << 30) as f64,
            bytes as f64 / 1048576.0
        );
    }
}
