
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
            // 第二十四块新成员：盘表（按次序）与挂载时核过的那一份不同，预演之前就拒。
            Err(WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount { device_table_of_the_mount, device_table_handed_in }) => format!(
                "tableref[{:?}->{:?}]",
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
