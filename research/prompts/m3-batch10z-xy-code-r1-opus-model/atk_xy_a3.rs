fn atk_real_follow<Device: DeviceOverASparseImage>(
    pool: &mut PoolUnderTest<Device>,
    change: &AtkChange,
) -> (String, Vec<u64>, String, String, String, bool, bool) {
    let before = isolated_slots_on_each_device_of(&pool.session().allocator);
    let pushed = push_the_root_ring_of(pool);
    let (push_text, released, push_detail) = match &pushed {
        Ok(done) => {
            let after = isolated_slots_on_each_device_of(&pool.session().allocator);
            let released: Vec<u64> = before.iter().zip(&after).map(|((_, b), (_, a))| b.saturating_sub(*a)).collect();
            (format!("pushed({})", done.versions_published.len()), released, String::new())
        }
        Err(refusal) => (format!("push_refused[{}]", atk_head(format!("{refusal:?}"))), Vec::new(), format!("{refusal:?}")),
    };
    if pushed.is_err() {
        return (push_text, released, push_detail, "-".to_string(), String::new(), false, false);
    }
    let retry = atk_apply(pool, change);
    let detail = match &retry {
        Ok(_) => String::new(),
        Err(refusal) => format!("{refusal:?}"),
    };
    (push_text, released, push_detail, atk_kind(&retry), detail, retry.is_ok(), true)
}

/// 把一次要出路与照出路二真走逐格比：预演 adm ⇔ 推环做成且再发放得下；still ⇔ 推环做成且再发被拒（另比两份拒绝的 Debug）；
/// pushref ⇔ 推环被拒（另比 Debug）；预演放出来的隔离槽数 ⇔ 真推前后每块盘隔离数之差；出路条数与结局对得上。
/// 要出路被拒成 moved 的不比真走（交回 None）。
/// 把一次要出路与照出路二真走逐格比：预演 adm ⇔ 推环做成且再发放得下；still ⇔ 推环做成且再发被拒（另比两份拒绝的 Debug）；
/// pushref ⇔ 推环被拒（另比 Debug）；预演放出来的隔离槽数 ⇔ 真推前后每块盘隔离数之差；出路条数与结局对得上。
/// 第二十四块：要出路拒成盘表那一种（tableref）⇔ 照出路二拿同一份盘表推环、再发同一个改动，再发拒成 `DeviceTableOtherThanTheOneOfTheMount`；
/// 另要这一次一个盘都没读（数调用那一层的数为 0；没数的记 u64::MAX，不算）、会话与盘不动。要出路被拒成 moved 的不比真走（交回 None）。
fn atk_compare<Device: DeviceOverASparseImage>(
    ask: &AtkAsk,
    pool_to_follow: &mut PoolUnderTest<Device>,
    change: &AtkChange,
) -> (String, Option<bool>) {
    if let Err(WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount { .. }) = &ask.result {
        let (push_text, _released_real, _push_detail, retry_text, retry_detail, _retry_ok, pushed) = atk_real_follow(pool_to_follow, change);
        // 照同一份盘表走：推环被拒（盘表那一套核不过），或推得过、再发拒成盘表那一种，都是「这份盘表走不通」。
        let agree = !pushed || retry_detail.starts_with("DeviceTableOtherThanTheOneOfTheMount");
        let no_read = (ask.probes, ask.reads, ask.changing_calls) == (0, 0, 0)
            || (ask.probes, ask.reads, ask.changing_calls) == (u64::MAX, u64::MAX, u64::MAX);
        let ok = agree && no_read && ask.untouched;
        return (
            format!(
                "ask={} {} push={push_text} retry={retry_text} agree={}{}",
                ask.head(),
                ask.io(),
                if agree { "yes" } else { "NO" },
                if ok { "" } else { " !MISMATCH" }
            ),
            Some(ok),
        );
    }
    let Ok(ways) = &ask.result else {
        return (format!("ask={} {}", ask.head(), ask.io()), None);
    };
    let (kind, released_rehearsed, detail_rehearsed) = atk_rehearsal_parts(&ways.pushing_the_root_ring);
    let (push_text, released_real, push_detail, retry_text, retry_detail, retry_ok, pushed) = atk_real_follow(pool_to_follow, change);
    let agree = match kind {
        "adm" => pushed && retry_ok,
        "still" => pushed && !retry_ok,
        _ => !pushed,
    };
    let released_same = kind == "pushref" || released_rehearsed == released_real;
    let detail_same = match kind {
        "still" => (detail_rehearsed == retry_detail).to_string(),
        "pushref" => (detail_rehearsed == push_detail).to_string(),
        _ => "-".to_string(),
    };
    let ways_count = ways.ways_out.len();
    let ways_consistent = (kind == "adm") == (ways_count == 2);
    let ok = agree && released_same && ways_consistent && detail_same != "false" && ask.untouched;
    (
        format!(
            "ask={} {} ways={ways_count} push={push_text} retry={retry_text} agree={} n_reh={released_rehearsed:?} n_real={released_real:?} n_same={released_same} dbg_same={detail_same} ways_ok={ways_consistent}{}",
            ask.head(),
            ask.io(),
            if agree { "yes" } else { "NO" },
            if ok { "" } else { " !MISMATCH" }
        ),
        Some(ok),
    )
}

/// 报「被宽护卡住」之后立刻要出路，再在另一份拷贝上照出路二真走。
fn atk_follow_way_two(after_report: &AtkSnap, stuck: &StuckBehindTheWideProtection, change: &AtkChange) -> (String, bool) {
    let ask = atk_ask(&after_report.pool(), stuck);
    let mut follow = after_report.pool();
    let (line, ok) = atk_compare(&ask, &mut follow, change);
    (line, ok == Some(true))
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

fn atk_env_list<T: std::str::FromStr>(name: &str, default: &str) -> Vec<T>
where
    T::Err: std::fmt::Debug,
{
    std::env::var(name).unwrap_or_else(|_| default.to_string()).split(',').filter(|t| !t.is_empty()).map(|text| text.parse().expect("数")).collect()
}

fn atk_env_or<T: std::str::FromStr>(name: &str, default: T) -> T
where
    T::Err: std::fmt::Debug,
{
    std::env::var(name).map_or(default, |text| text.parse().expect("数"))
}
