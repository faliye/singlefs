
/// 照出路二走（真推环、再发同一个改动一次），与那一报的预演逐格比。交回一行字与「对不对得上」。
/// 比的是：预演 adm ⇔ 推环做成且再发放得下；still ⇔ 推环做成且再发被拒（另比两份拒绝的 Debug 是不是逐字相同）；
/// pushref ⇔ 推环被拒（另比 Debug）；预演放出来的隔离槽数 ⇔ 真推前后每块盘隔离数之差。
fn atk_follow_way_two(
    after_report: &AtkSnap,
    stuck: &StuckBehindTheWideProtection,
    write: &AtkWrite,
) -> (String, bool) {
    let (kind, released_rehearsed, detail_rehearsed) = atk_rehearsal_of(stuck);
    let mut pool = after_report.pool();
    let before = isolated_slots_on_each_device_of(&pool.session().allocator);
    let pushed = push_the_root_ring_of(&mut pool);
    let (push_text, released_real, push_detail) = match &pushed {
        Ok(done) => {
            let after = isolated_slots_on_each_device_of(&pool.session().allocator);
            let released: Vec<u64> = before
                .iter()
                .zip(&after)
                .map(|((_, b), (_, a))| b.saturating_sub(*a))
                .collect();
            (format!("pushed({})", done.versions_published.len()), released, String::new())
        }
        Err(refusal) => ("push_refused".to_string(), Vec::new(), format!("{refusal:?}")),
    };
    let (retry_text, retry_detail, retry_ok) = if pushed.is_ok() {
        let retry = atk_apply(&mut pool, write);
        let detail = match &retry {
            Ok(_) => String::new(),
            Err(refusal) => format!("{refusal:?}"),
        };
        (atk_kind(&retry), detail, retry.is_ok())
    } else {
        ("-".to_string(), String::new(), false)
    };
    let agree = match kind {
        "adm" => pushed.is_ok() && retry_ok,
        "still" => pushed.is_ok() && !retry_ok,
        _ => pushed.is_err(),
    };
    let released_same = kind == "pushref" || released_rehearsed == released_real;
    let detail_same = match kind {
        "still" => (detail_rehearsed == retry_detail).to_string(),
        "pushref" => (detail_rehearsed == push_detail).to_string(),
        _ => "-".to_string(),
    };
    let ways = atk_ways(stuck).ways_out.len();
    let ways_consistent = (kind == "adm") == (ways == 2);
    (
        format!(
            "reh={kind} ways={ways} push={push_text} retry={retry_text} agree={} n_reh={released_rehearsed:?} n_real={released_real:?} n_same={released_same} dbg_same={detail_same} ways_ok={ways_consistent}",
            if agree { "yes" } else { "NO" }
        ),
        agree && released_same && ways_consistent && detail_same != "false",
    )
}

/// 被拒那一报之后再原样发一次（不推环）：这一次推了几串抬 F、发了几个改盘调用，没推也没发成时盘面、分配器、现行那一版是不是逐字不变，
/// 与第二次报的预演结局。
fn atk_repeat_without_push(after_report: &AtkSnap, write: &AtkWrite) -> String {
    let (mut pool, stream) = after_report.recording_pool();
    let image_before = pool.image();
    let allocator_before = format!("{:?}", pool.session().allocator);
    let current_before = format!("{:?}", pool.session().current);
    let operations_before = stream.operation_count();
    let again = atk_apply(&mut pool, write);
    let operations = stream.operation_count() - operations_before;
    let raises = match &again {
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => no_space.floor_raises.len(),
        _ => usize::MAX,
    };
    let unchanged = pool.image() == image_before
        && format!("{:?}", pool.session().allocator) == allocator_before
        && format!("{:?}", pool.session().current) == current_before;
    let reh = atk_stuck_of(&again).map_or("-", |stuck| atk_rehearsal_of(stuck).0);
    format!("again={} again_reh={reh} again_raises={raises} again_ops={operations} again_unchanged={unchanged}", atk_kind(&again))
}

/// 用户在被拒之后先做 k 次小动作（建 inode / 覆盖写 2999 字节），再发同一个长度的顺序写；那一次若报被宽护卡住，照它的出路二走、逐格比。
fn atk_small_actions_then_report_and_follow(
    after_report: &AtkSnap,
    units: usize,
    action: &str,
    k: usize,
) -> (String, Option<bool>) {
    let mut pool = after_report.pool();
    let mut refused = 0;
    for _ in 0..k {
        let outcome = if action == "inode" { pool.create_one_inode() } else { pool.overwrite(OVERWRITE_BYTES) };
        if outcome.is_err() {
            refused += 1;
        }
    }
    let write = atk_write_of(&mut pool, units);
    let outcome = atk_apply(&mut pool, &write);
    let kind = atk_kind(&outcome);
    let Some(stuck) = atk_stuck_of(&outcome) else {
        return (format!("{action}{k}:{kind}(small_refused={refused})"), None);
    };
    let snap = AtkSnap::of(&pool);
    let (line, ok) = atk_follow_way_two(&snap, stuck, &write);
    (format!("{action}{k}:stuck(small_refused={refused}) {line}"), Some(ok))
}

/// 一份镜像上新起一个池（两块普通稀疏盘），可写挂载一次（空间准入判着）。
fn atk_mounted_pool_from(
    parameters: &MakeFilesystemParameters,
    device_bytes: u64,
    image: &MemoryPool,
    contents: &BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
    write_time_seconds: u64,
) -> Result<(PoolUnderTest<SparseBlockDevice>, MountOutput), MountError> {
    let mut pool = PoolUnderTest {
        parameters: parameters.clone(),
        device_bytes,
        devices: plain_devices_on(image),
        session: None,
        write_time_seconds,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: contents.clone(),
    };
    let output = pool.crash_and_mount_writable()?;
    Ok((pool, output))
}
