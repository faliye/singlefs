
// ---------- K2-1：推环真发半路块设备错、推环中间崩溃 ----------

/// 一块盘外面包一层：数改盘的调用（写、写零、屏障），第 `fail_at` 次（整池合计，从 1 数）报块设备错、不落盘，只这一次。
struct AtkFaultPlan {
    calls: Cell<u64>,
    fail_at: Cell<Option<u64>>,
    fired: Cell<Option<String>>,
}

struct AtkDevice {
    inner: SparseBlockDevice,
    plan: Rc<AtkFaultPlan>,
}

impl AtkDevice {
    fn tick(&self, what: &str) -> Result<(), BlockDeviceError> {
        let call = self.plan.calls.get() + 1;
        self.plan.calls.set(call);
        if self.plan.fail_at.get() == Some(call) {
            self.plan.fired.set(Some(what.to_string()));
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("攻方注入的写错")));
        }
        Ok(())
    }
}

impl BlockDevice for AtkDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.tick(if matches!(durability, WriteDurability::ForceUnitAccess) { "fua" } else { "write" })?;
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.tick("zeroes")?;
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.tick("barrier")?;
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

impl DeviceOverASparseImage for AtkDevice {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.image
    }
}

fn atk_clone_session(session: &MountedSession) -> MountedSession {
    MountedSession {
        allocator: session.allocator.clone(),
        current: session.current.clone(),
        instance: session.instance,
        shadow_ledger: session.shadow_ledger,
        parameters_and_device_table: session.parameters_and_device_table.clone(),
    }
}

/// 卡住那一刻的盘面与会话的一份快照：每段历史从它拷。
struct AtkStuckSnapshot {
    image: MemoryPool,
    session: MountedSession,
    contents: BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
    content_now: Vec<u8>,
    write_time_seconds: u64,
}

impl AtkStuckSnapshot {
    fn of(pool: &PoolUnderTest<SparseBlockDevice>) -> Self {
        AtkStuckSnapshot {
            image: pool.image(),
            session: atk_clone_session(pool.session()),
            contents: pool.content_of_each_root.clone(),
            content_now: pool.content_of_the_current_version.clone(),
            write_time_seconds: pool.write_time_seconds,
        }
    }

    fn pool_with_faults(&self, fail_at: Option<u64>) -> (PoolUnderTest<AtkDevice>, Rc<AtkFaultPlan>) {
        let plan = Rc::new(AtkFaultPlan { calls: Cell::new(0), fail_at: Cell::new(fail_at), fired: Cell::new(None) });
        let devices = plain_devices_on(&self.image)
            .into_iter()
            .map(|(identity, inner)| (identity, AtkDevice { inner, plan: plan.clone() }))
            .collect();
        (
            PoolUnderTest {
                parameters: parameters_of_the_384_slot_small_devices(),
                device_bytes: device_bytes_of_the_384_slot_small_devices(),
                devices,
                session: Some(atk_clone_session(&self.session)),
                write_time_seconds: self.write_time_seconds,
                content_of_the_current_version: self.content_now.clone(),
                content_of_each_root: self.contents.clone(),
            },
            plan,
        )
    }

    fn recording_pool(&self) -> (PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>, SharedStream) {
        let stream = SharedStream::retaining_contents();
        let devices = plain_devices_on(&self.image)
            .into_iter()
            .map(|(identity, inner)| (identity, RecordingBlockDevice::with_shared_stream(identity, inner, stream.clone())))
            .collect();
        (
            PoolUnderTest {
                parameters: parameters_of_the_384_slot_small_devices(),
                device_bytes: device_bytes_of_the_384_slot_small_devices(),
                devices,
                session: Some(atk_clone_session(&self.session)),
                write_time_seconds: self.write_time_seconds,
                content_of_the_current_version: self.content_now.clone(),
                content_of_each_root: self.contents.clone(),
            },
            stream,
        )
    }
}

/// 一份盘面崩了再可写挂载（普通盘），挂上之后池级 checker 全部违例数、再覆盖写 `overwrites` 次、每次之后的违例数合计。
fn atk_remount_and_check(
    image: &MemoryPool,
    contents: &BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
    write_time_seconds: u64,
    overwrites: usize,
) -> String {
    match atk_mounted_pool_from(image, contents, write_time_seconds) {
        Err(refusal) => format!("remount_refused({})", format!("{refusal:?}").chars().take(70).collect::<String>()),
        Ok((mut pool, output)) => {
            let at_mount = checker_violations_on(&pool.image());
            let mut later = 0usize;
            let mut refused = 0usize;
            let mut first_later = String::new();
            for _ in 0..overwrites {
                if pool.overwrite(OVERWRITE_BYTES).is_err() {
                    refused += 1;
                }
                let violations = checker_violations_on(&pool.image());
                if first_later.is_empty() {
                    if let Some(first) = violations.first() {
                        first_later = first.chars().take(120).collect();
                    }
                }
                later += violations.len();
            }
            format!(
                "remount_ok(sel={}:{} iso={} c={:?} viol_mount={} viol_later={later} refused={refused}{}{})",
                output.effective_root.instance.0,
                output.effective_root.checkpoint_txg.0,
                isolated_slots_of(&pool.session().allocator),
                atk_c_ring(&pool.session().allocator),
                at_mount.len(),
                at_mount.first().map(|first| format!(" first_mount={}", first.chars().take(120).collect::<String>())).unwrap_or_default(),
                if first_later.is_empty() { String::new() } else { format!(" first_later={first_later}") }
            )
        }
    }
}

fn atk_form_from(name: &str) -> FormStuckBehindTheWideProtectionOnThe384SlotDevices {
    match name {
        "b6o20" => FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack { overwrites_before: 20 },
        "b6o40" => FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack { overwrites_before: 40 },
        "no40" => FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation { overwrites_before: 40 },
        "d160" => FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion { overwrites_before: 160 },
        other => panic!("没有这一形：{other}"),
    }
}

/// K2-1 块设备错：推环真发时第 j 次改盘调用报错（只这一次），j 扫遍整串推环的改盘调用（步长 ATK_STEP_J）。
/// 每个 j：推环交回什么、会话的现行那一版与分配器上冻结着没有；再推一次、覆盖写一次、正常卸载各交回什么；
/// 之后照盘面崩了再挂，池级 checker 全部判定的违例数（挂上那一刻与之后覆盖写三次）。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k2_block_device_error_sweep() {
    let form_name = std::env::var("ATK_FORM").unwrap_or_else(|_| "no40".to_string());
    let step: u64 = std::env::var("ATK_STEP_J").map_or(1, |text| text.parse().expect("数"));
    let PoolStuckBehindTheWideProtection { pool, .. } = pool_stuck_behind_the_wide_protection(atk_form_from(&form_name));
    let snapshot = AtkStuckSnapshot::of(&pool);
    let (mut dry, plan) = snapshot.pool_with_faults(None);
    let dry_outcome = atk_push(&mut dry);
    let total_calls = plan.calls.get();
    println!("ATK k2err form={form_name} dry={dry_outcome} calls={total_calls} start_txg={}", snapshot.session.current.root().checkpoint_txg.0);
    let mut j = 1;
    while j <= total_calls {
        let (mut pool, plan) = snapshot.pool_with_faults(Some(j));
        let pushed = atk_push(&mut pool);
        let fired = plan.fired.take().unwrap_or_else(|| "-".to_string());
        let frozen = pool.session().allocator.frozen_publish().map(|frozen| frozen.checkpoint_txg().0);
        let current_txg = pool.session().current.root().checkpoint_txg.0;
        let again = atk_push(&mut pool);
        let overwrite = atk_outcome(&pool.overwrite(OVERWRITE_BYTES));
        let mut session = pool.session.take().expect("会话");
        let unmounted = match unmount(&pool.parameters, &mut pool.devices, &mut session.allocator, &mut session.current, ShadowLedger::On) {
            Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => format!("ok({})", raised.publishes.len()),
            Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "nothing".to_string(),
            Err(refusal) => format!("refused({})", format!("{refusal:?}").chars().take(60).collect::<String>()),
        };
        let image = pool.image();
        let after_crash = atk_remount_and_check(&image, &snapshot.contents, snapshot.write_time_seconds + 50, 3);
        println!(
            "ATK k2err form={form_name} j={j} fired={fired} push={pushed} current_txg={current_txg} frozen={frozen:?} again={again} overwrite={overwrite} unmount={unmounted} crash_remount={after_crash}"
        );
        j += step;
    }
}

/// K2-1 崩溃点：录下整串推环的写流，崩在第 k 个写之后（k 从 0 到全长，前缀；不是层 0 的整写子集枚举），
/// 普通盘崩了再挂、池级 checker 全部判定、之后覆盖写三次；另一臂挂载时 R 那几段照旧读回全零（故障没撤）。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k2_crash_prefix_sweep() {
    let form_name = std::env::var("ATK_FORM").unwrap_or_else(|_| "no40".to_string());
    let step: usize = std::env::var("ATK_STEP_K").map_or(1, |text| text.parse().expect("数"));
    let form = atk_form_from(&form_name);
    let PoolStuckBehindTheWideProtection { pool, .. } = pool_stuck_behind_the_wide_protection(form);
    let (_recorded, ranges) = recorded_r_and_ranges_reading_back_zero_of(form);
    let snapshot = AtkStuckSnapshot::of(&pool);
    let (mut recording, stream) = snapshot.recording_pool();
    let operations_before = stream.operation_count();
    let pushed = atk_push(&mut recording);
    let operations = stream.retained_operations();
    let (writes, segments, _indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let roots = writes.iter().filter(|write| write.kind == StepKind::RootRecordFua).count();
    println!("ATK k2crash form={form_name} push={pushed} writes={} segments={} roots={roots}", writes.len(), segments.len());
    let mut summary: BTreeMap<String, usize> = BTreeMap::new();
    let mut k = 0;
    while k <= writes.len() {
        let mut image = snapshot.image.clone();
        image.apply_writes(&writes[..k]);
        let plain = atk_remount_and_check(&image, &snapshot.contents, snapshot.write_time_seconds + 50, 3);
        let mut faulty_devices = devices_reading_back_all_zero_on(&image, &ranges);
        let faulty = match mount_writable_with_space_admission(&parameters_of_the_384_slot_small_devices(), &mut faulty_devices, SpaceAdmission::JudgedByTheFormula) {
            Ok(mounted) => format!("ok(sel_txg={})", mounted.output.effective_root.checkpoint_txg.0),
            Err(refusal) => format!("refused({})", format!("{refusal:?}").chars().take(50).collect::<String>()),
        };
        let roots_landed = writes[..k].iter().filter(|write| write.kind == StepKind::RootRecordFua).count();
        println!("ATK k2crash form={form_name} k={k} roots_landed={roots_landed} plain={plain} with_fault={faulty}");
        let key = format!(
            "plain_{} with_fault_{}",
            if plain.starts_with("remount_ok") {
                if plain.contains("viol_mount=0 viol_later=0") { "ok_clean" } else { "ok_VIOLATION" }
            } else { "refused" },
            if faulty.starts_with("ok") { "ok" } else { "refused" }
        );
        *summary.entry(key).or_insert(0) += 1;
        k += step;
    }
    println!("ATK k2crash form={form_name} summary={summary:?}");
}
