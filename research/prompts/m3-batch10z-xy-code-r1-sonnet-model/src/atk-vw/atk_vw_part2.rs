
// ---------- 要出路与照出路二真走 ----------

/// 预演结局的短名，带拒绝的成员名（PushRefused 里是哪一种、ChangeStillRefused 里的拒绝是哪一种）。
fn atk_rehearsal_head(rehearsal: &RootRingPushRehearsal) -> String {
    match rehearsal {
        RootRingPushRehearsal::ChangeAdmittedAfterThePush { .. } => "adm".to_string(),
        RootRingPushRehearsal::ChangeStillRefusedAfterThePush { refusal, .. } => {
            let inner = match refusal.as_ref() {
                UserChangeRefused::Publish { cause, .. } => format!("Publish:{}", atk_head(format!("{cause:?}"))),
                UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(failed) => {
                    format!("FloorRaiseFailed:{}", atk_head(format!("{:?}", failed.cause)))
                }
                other => atk_head(format!("{other:?}")),
            };
            format!("still[{inner}]")
        }
        RootRingPushRehearsal::PushRefused(refusal) => {
            let inner = match refusal.as_ref() {
                RootRingPushRefused::DevicesHandedIn(error) => format!("DevicesHandedIn:{}", atk_head(format!("{error:?}"))),
                RootRingPushRefused::RefusedByTheRehearsalBeforeAnyWrite { cause, refused_publish_in_the_push, publishes_in_the_push } => {
                    format!("RefusedByTheRehearsal:{}@{refused_publish_in_the_push}/{publishes_in_the_push}", atk_head(format!("{cause:?}")))
                }
                RootRingPushRefused::PublishFailedAfterTheRehearsal(failed) => {
                    format!("PublishFailedAfterTheRehearsal:{}", atk_head(format!("{:?}", failed.cause)))
                }
                other => atk_head(format!("{other:?}")),
            };
            format!("pushref[{inner}]")
        }
    }
}

/// 预演的结局：(adm / still / pushref, 预演里每块盘放出来的隔离槽数, 预演里交回的拒绝原样)。
fn atk_rehearsal_parts(rehearsal: &RootRingPushRehearsal) -> (&'static str, Vec<u64>, String) {
    match rehearsal {
        RootRingPushRehearsal::ChangeAdmittedAfterThePush { isolated_slots_released_by_the_push_on_each_device } => (
            "adm",
            isolated_slots_released_by_the_push_on_each_device.iter().map(|(_, n)| *n).collect(),
            String::new(),
        ),
        RootRingPushRehearsal::ChangeStillRefusedAfterThePush { isolated_slots_released_by_the_push_on_each_device, refusal } => (
            "still",
            isolated_slots_released_by_the_push_on_each_device.iter().map(|(_, n)| *n).collect(),
            format!("{refusal:?}"),
        ),
        RootRingPushRehearsal::PushRefused(refusal) => ("pushref", Vec::new(), format!("{refusal:?}")),
    }
}

/// 一次要出路：结局与这一次对底下的盘做了什么（探物理块大小、读、改盘调用的次数），盘面、分配器、现行那一版动没动。
struct AtkAsk {
    result: Result<WaysOutOfTheWideProtection, WaysOutOfTheWideProtectionRefused>,
    probes: u64,
    reads: u64,
    changing_calls: u64,
    untouched: bool,
}

impl AtkAsk {
    fn head(&self) -> String {
        match &self.result {
            Ok(ways) => atk_rehearsal_head(&ways.pushing_the_root_ring),
            Err(WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
                instance_refused_on,
                checkpoint_txg_refused_on,
                instance_of_the_current_version,
                checkpoint_txg_of_the_current_version,
            }) => format!(
                "moved[{}.{}->{}.{}]",
                instance_refused_on.0, checkpoint_txg_refused_on.0, instance_of_the_current_version.0, checkpoint_txg_of_the_current_version.0
            ),
            Err(WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount { device_table_of_the_mount, device_table_handed_in }) => format!(
                "devtable[{:?}->{:?}]",
                device_table_of_the_mount.iter().map(|d| d.0).collect::<Vec<_>>(),
                device_table_handed_in.iter().map(|d| d.0).collect::<Vec<_>>()
            ),
        }
    }
    fn io(&self) -> String {
        format!("ask_io=({},{},{}) ask_untouched={}", self.probes, self.reads, self.changing_calls, self.untouched)
    }
}

/// 在这一刻的一份数调用的拷贝上要出路（`&self`，盘表交这一刻池里的那一份）。
fn atk_ask(pool: &PoolUnderTest<SparseBlockDevice>, stuck: &StuckBehindTheWideProtection) -> AtkAsk {
    let (counting, calls) = pool_failing_one_changing_call_copied_from(pool, None);
    let image_before = counting.image();
    let allocator_before = format!("{:?}", counting.session().allocator);
    let current_before = format!("{:?}", counting.session().current);
    let result = counting.session().ways_out_of_the_wide_protection(&counting.devices, stuck);
    let untouched = counting.image() == image_before
        && format!("{:?}", counting.session().allocator) == allocator_before
        && format!("{:?}", counting.session().current) == current_before;
    AtkAsk {
        result,
        probes: calls.probes_of_the_physical_block_size.get(),
        reads: calls.reads.get(),
        changing_calls: calls.calls.get(),
        untouched,
    }
}

/// 任何盘上直接要出路（不数调用；只比盘面、分配器、现行那一版动没动）。
fn atk_ask_direct<Device: DeviceOverASparseImage>(pool: &PoolUnderTest<Device>, stuck: &StuckBehindTheWideProtection) -> AtkAsk {
    let image_before = pool.image();
    let allocator_before = format!("{:?}", pool.session().allocator);
    let current_before = format!("{:?}", pool.session().current);
    let result = pool.session().ways_out_of_the_wide_protection(&pool.devices, stuck);
    let untouched = pool.image() == image_before
        && format!("{:?}", pool.session().allocator) == allocator_before
        && format!("{:?}", pool.session().current) == current_before;
    AtkAsk { result, probes: u64::MAX, reads: u64::MAX, changing_calls: u64::MAX, untouched }
}

/// 照出路二真走一遍（真推环、再发同一个改动一次）。交回 (推环短名, 真推前后每块盘隔离数之差, 推环拒绝原样, 再发结局短名, 再发拒绝原样, 再发做成)。
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
fn atk_compare<Device: DeviceOverASparseImage>(
    ask: &AtkAsk,
    pool_to_follow: &mut PoolUnderTest<Device>,
    change: &AtkChange,
) -> (String, Option<bool>) {
    if let Err(WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount { .. }) = &ask.result {
        // 盘表不对被拒在预演之前：照同一份盘表真走（推环、再发），真结局不能是「放得下」；没读盘、没动。
        let (push_text, _released, _push_detail, retry_text, retry_detail, retry_ok, pushed) = atk_real_follow(pool_to_follow, change);
        let truth_admitted = pushed && retry_ok;
        let retry_is_devtable = retry_detail.contains("DeviceTableOtherThanTheOneOfTheMount");
        let io_clean = ask.probes == 0 && ask.reads == 0 && ask.changing_calls == 0;
        let ok = !truth_admitted && ask.untouched && (io_clean || ask.reads == u64::MAX);
        return (
            format!(
                "ask={} {} truth_push={push_text} truth_retry={retry_text} retry_is_devtable={retry_is_devtable} truth_admitted={truth_admitted} io_clean={io_clean}{}",
                ask.head(), ask.io(), if ok { "" } else { " !MISMATCH" }
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
