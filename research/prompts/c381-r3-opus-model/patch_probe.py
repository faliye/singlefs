import pathlib
root = pathlib.Path("/tmp/claude-1000/c381-r3-opus/repo")

# ---------- core：让臂 6/7/8 也走失败表驱动 ----------
p = root/"crates/singlefs-core/src/transaction.rs"
s = p.read_text()
def once(old, new, text):
    assert text.count(old) == 1, (text.count(old), old[:90])
    return text.replace(old, new)
if "state.arm >= 5" not in s:
    s = once("if (state.arm == 2 || state.arm == 5) && (state.stopped || state.read_only) {",
             "if (state.arm == 2 || state.arm >= 5) && (state.stopped || state.read_only) {", s)
    s = once("                5 => {\n", "                5 | 6 | 7 | 8 => {\n", s)
p.write_text(s)
print("core patched")

# ---------- 探针装置 ----------
p = root/"crates/singlefs-harness/tests/c381r3_probe_common.rs"
s = p.read_text()

# A. Control 新字段
s = once("""    /// 每一次写的（盘, 偏移, 长度）：两遍法的第一遍用它取 C 写到哪几个落点。
    pub log: RefCell<Vec<(u32, u64, usize)>>,""",
"""    /// 每一次写的（盘, 偏移, 长度）：两遍法的第一遍用它取 C 写到哪几个落点。
    pub log: RefCell<Vec<(u32, u64, usize)>>,
    /// c381-r3：候选乙要的「这次失败的那个落点」（盘, 偏移, 长度）。屏障报错时置 None——`barrier()` 不带偏移参数。
    pub last_failed_landing: RefCell<Option<(u32, u64, usize)>>,
    /// c381-r3：候选乙第二种读法（把那次写的原样字节再写一遍）要的字节。只有装置拿得到——
    /// `publish_version` 失败之后没有任何对象保存这次发布要写什么（c381-r2 K5 第七节）。
    pub last_failed_bytes: RefCell<Option<Vec<u8>>>,
    /// 每次探针写落在哪、盖掉的那一段原来是不是全零。
    pub probe_notes: RefCell<Vec<String>>,
    /// 探针写的落点没有定义（失败发生在屏障上）的次数。
    pub probe_undefined: Cell<u32>,""", s)

# B. Device::note_failure
s = once("""impl BlockDevice for Device {
    fn read_at""",
"""impl Device {
    /// 一次写报错：记下报错的盘与这次失败的那个落点（候选乙的判别子要它）。
    fn note_failure(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        self.control.last_failed_device.set(Some(self.number));
        *self.control.last_failed_landing.borrow_mut() =
            Some((self.number, offset.0, bytes.len()));
        *self.control.last_failed_bytes.borrow_mut() = Some(bytes.to_vec());
    }
}

impl BlockDevice for Device {
    fn read_at""", s)

# C. 屏障报错：落点置 None
s = once("""        if self.control.barrier_faults.borrow().contains(&index) {
            self.control.last_failed_device.set(Some(self.number));""",
"""        if self.control.barrier_faults.borrow().contains(&index) {
            self.control.last_failed_device.set(Some(self.number)); // 屏障：报错的是哪块盘知道
            // 屏障没有落点：`barrier()` 不带偏移（`block_device.rs` 的 `fn barrier(&mut self)`），
            // 它刷的是整块盘。候选乙在这一格没有定义。
            *self.control.last_failed_landing.borrow_mut() = None;""", s)

# D. write_at 里六处报错各记一次落点
old = "            self.control.last_failed_device.set(Some(self.number));\n"
n = s.count(old)
assert n == 6, n
s = s.replace(old, "            self.note_failure(offset, bytes);\n")

# E. 探针写三种读法
s = once("""/// 探针写：往目标盘的固定落点写 512 字节。写得进去 ⇒ 瞬时失败，写不进去 ⇒ 持续失败（D23（journal 的角色与格式） 已定项 14）。
fn probe_write(devices: &mut [(DeviceIdentity, Device)], target: u32, control: &Control) -> bool {
    control.probe_writes.set(control.probe_writes.get() + 1);
    let Some((_, device)) = devices.iter_mut().find(|(identity, _)| identity.0 == target) else {
        return false;
    };
    device
        .write_at(
            DeviceOffsetInBytes(PROBE_OFFSET),
            &[0xC3u8; 512],
            WriteDurability::ForceUnitAccess,
        )
        .is_ok()
}""",
"""/// 探针写的落点按哪一种读法取。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ProbeMode {
    /// D23（journal 的角色与格式） 已定项 14 今天的字面：目标设备的固定落点（用户 2026-09-21 第 1 题定的「地址空间表登记一行」就是给它的）。
    #[default]
    Fixed,
    /// 候选乙：写这次失败的那个落点（写探针图样 0xC3）。
    FailedLanding,
    /// 候选乙的第二种读法：把那次写的原样字节再写一遍（它其实是重试，不是探针写）。
    FailedLandingRetry,
}

/// 落点落在盘面地图的哪一段（`.claude/kb/layout/01-first-txn.md` 零节的地址空间表）。
#[must_use]
pub fn region_name(offset: u64) -> &'static str {
    match offset {
        0..=8_191 => "系统配置槽",
        1_048_576..=8_388_607 => "根环区域",
        16_777_216..=822_083_583 => "journal 环",
        _ => "单元区",
    }
}

/// 探针写：写得进去 ⇒ 瞬时失败，写不进去 ⇒ 持续失败（D23（journal 的角色与格式） 已定项 14）。
fn probe_write(
    devices: &mut [(DeviceIdentity, Device)],
    target: u32,
    control: &Control,
    mode: ProbeMode,
) -> bool {
    control.probe_writes.set(control.probe_writes.get() + 1);
    let landing = *control.last_failed_landing.borrow();
    let (target, offset, length, bytes): (u32, u64, usize, Vec<u8>) = match (mode, landing) {
        (ProbeMode::Fixed, _) => (target, PROBE_OFFSET, 512, vec![0xC3u8; 512]),
        (_, None) => {
            control.probe_undefined.set(control.probe_undefined.get() + 1);
            control
                .probe_notes
                .borrow_mut()
                .push("落点没有定义（失败发生在屏障上，barrier() 不带偏移）→ 装置按最宽松的读法退回固定落点".to_string());
            (target, PROBE_OFFSET, 512, vec![0xC3u8; 512])
        }
        (ProbeMode::FailedLanding, Some((device, offset, length))) => {
            (device, offset, length, vec![0xC3u8; length])
        }
        (ProbeMode::FailedLandingRetry, Some((device, offset, length))) => {
            let saved = control
                .last_failed_bytes
                .borrow()
                .clone()
                .unwrap_or_else(|| vec![0xC3u8; length]);
            (device, offset, length, saved)
        }
    };
    let Some((_, device)) = devices.iter_mut().find(|(identity, _)| identity.0 == target) else {
        return false;
    };
    if !matches!(mode, ProbeMode::Fixed) {
        let mut before = vec![0u8; length];
        let read = device
            .read_at(DeviceOffsetInBytes(offset), &mut before)
            .is_ok();
        let nonzero = read && before.iter().any(|byte| *byte != 0);
        control.probe_notes.borrow_mut().push(format!(
            "探针落点 盘{target} 偏移{offset} 长{length}（{}，盖掉之前{}）",
            region_name(offset),
            if nonzero { "那一段非零" } else { "那一段全零" }
        ));
    }
    device
        .write_at(
            DeviceOffsetInBytes(offset),
            &bytes,
            WriteDurability::ForceUnitAccess,
        )
        .is_ok()
}

/// 盘面上今天还读得出的权威结构：每条自证过的根记录、每份自证过的系统配置槽。
/// 探针写前后各取一次，差集就是这一次探针写抹掉的东西。
#[must_use]
pub fn disk_census(
    parameters: &MakeFilesystemParameters,
    devices: &Vec<(DeviceIdentity, Device)>,
) -> BTreeSet<String> {
    let mut census = BTreeSet::new();
    for root in readable_roots(
        devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    ) {
        census.insert(format!(
            "根记录(实例 {}, txg {})",
            root.instance.0, root.checkpoint_txg.0
        ));
    }
    for (identity, _) in devices.iter() {
        for slot in verified_superblock_slots(
            devices,
            *identity,
            u64::from(parameters.geometry.fixed_structure_slot_spacing),
            &parameters.filesystem_identifier,
        ) {
            census.insert(format!(
                "系统配置(盘 {}, 世代 {})",
                identity.0, slot.slot_generation
            ));
        }
    }
    census
}""", s)

# F. 驱动：探针写前后各取一次盘面普查
s = once("""        let probe_ok = probe_write(devices, target, control);
        outcome.steps.push(format!(
            "探针写(盘 {target}) {}",
            if probe_ok { "过" } else { "不过" }
        ));""",
"""        let census_before = disk_census(parameters, devices);
        let probe_ok = probe_write(devices, target, control, spec.probe_mode);
        let census_after = disk_census(parameters, devices);
        for lost in census_before.difference(&census_after) {
            outcome
                .destroyed
                .push(format!("探针写抹掉了 {lost}"));
        }
        outcome.steps.push(format!(
            "探针写(盘 {target}, {:?}) {}{}",
            spec.probe_mode,
            if probe_ok { "过" } else { "不过" },
            if census_before.difference(&census_after).count() == 0 {
                String::new()
            } else {
                format!(
                    "，抹掉 {:?}",
                    census_before
                        .difference(&census_after)
                        .cloned()
                        .collect::<Vec<_>>()
                )
            }
        ));""", s)

# G. Spec 与 Outcome 新字段
s = once("""    /// 坏扇区：这几段字节在这块盘上永远写不进去。
    pub bad_ranges: Vec<(u32, u64, u64)>,
}""",
"""    /// 坏扇区：这几段字节在这块盘上永远写不进去。
    pub bad_ranges: Vec<(u32, u64, u64)>,
    /// c381-r3：探针写按哪一种读法取落点。
    pub probe_mode: ProbeMode,
    /// c381-r3：这段历史走完之后，试着按 (实例, txg) 做一次管理员回退挂载，看那个根还在不在。
    pub rollback_probe: Option<(u32, u64)>,
}""", s)

s = once("""    /// 最后一次重开之后，最新根指着的那一版实例表里有几行（一片装 369 行，D18（块里携带什么信息） 已定项 11）。
    pub instance_rows: Option<usize>,
}""",
"""    /// 最后一次重开之后，最新根指着的那一版实例表里有几行（一片装 369 行，D18（块里携带什么信息） 已定项 11）。
    pub instance_rows: Option<usize>,
    /// c381-r3：探针写抹掉的权威结构（根记录、系统配置槽），一次一行。
    pub destroyed: Vec<String>,
    /// c381-r3：每次探针写落在哪。
    pub probe_notes: Vec<String>,
    /// c381-r3：探针写的落点没有定义（失败在屏障上）的次数。
    pub probe_undefined: u32,
    /// c381-r3：用户动作跑完、下次挂载之前，盘上还读得出的根记录与系统配置槽。
    pub census_before_remount: Vec<String>,
    /// c381-r3：`rollback_probe` 那个根还回退得上去吗。
    pub rollback_result: String,
}""", s)

# H. run()：下次挂载之前取普查、试一次回退
s = once("""    // 最后这一次是「下次挂载」：丙的「重开」与戊的「转只读到下次挂载」都在这里解除。
    c381_clear_stop();""",
"""    outcome.census_before_remount = disk_census(parameters, &devices).into_iter().collect();
    outcome.probe_notes = control.probe_notes.borrow().clone();
    outcome.probe_undefined = control.probe_undefined.get();
    if let Some((instance, txg)) = spec.rollback_probe {
        // 在这段历史的盘面副本上做一次管理员回退挂载（回退会写盘，所以另拿一份镜像跑）。
        let fresh = Rc::new(Control::default());
        let mut copy = devices_from(&images_of(&devices), &fresh);
        let attempt = catch_unwind(AssertUnwindSafe(|| {
            mount_rollback(
                parameters,
                &mut copy,
                RollbackTarget {
                    instance: InstanceGeneration(instance),
                    checkpoint_txg: CheckpointTxg(txg),
                },
                ShadowLedger::On,
            )
        }));
        outcome.rollback_result = match attempt {
            Ok(Ok(mounted)) => format!(
                "ok(回退到 txg {}，新实例 {})",
                mounted.output.chosen_root.checkpoint_txg.0,
                mounted.output.instance.0
            ),
            Ok(Err(error)) => format!(
                "REFUSED {}",
                format!("{error:?}").chars().take(100).collect::<String>()
            ),
            Err(_) => "PANIC".to_string(),
        };
    }
    // 最后这一次是「下次挂载」：丙的「重开」与戊的「转只读到下次挂载」都在这里解除。
    c381_clear_stop();""", s)

# I. 引用
s = once("use singlefs_core::recovery::{choose_root, choose_superblock, instance_table_of_root, walk_to_file};",
"""use singlefs_core::recovery::{
    choose_root, choose_superblock, instance_table_of_root, readable_roots,
    verified_superblock_slots, walk_to_file,
};""", s)

# J. 臂名
s = once("""        5 => "戊-A",
        6 => "戊-B",""",
"""        5 => "戊-A",
        6 => "戊-B",
        7 => "戊-乙a（探针写这次失败的那个落点）",
        8 => "戊-乙b（把那次写的原样字节再写一遍）",""", s)

# K. 分类里带上「探针写抹掉了别的数据」
s = once("""    if outcome.read_only {
        tags.push("转只读");
    }""",
"""    if outcome.read_only {
        tags.push("转只读");
    }
    if !outcome.destroyed.is_empty() {
        tags.push("探针写抹掉权威结构");
    }""", s)

p.write_text(s)
print("probe patched")

# ---------- harness：两处穷尽匹配（编译器报出来的，r2 第七节也记过） ----------
p = root/"crates/singlefs-harness/src/history.rs"
s = p.read_text()
if "C381WriterStopped" not in s:
    s = once('        PublishError::AllocationRecordsExceedOneNode { .. } => "AllocationRecordsExceedOneNode",',
             '        PublishError::C381WriterStopped => "C381WriterStopped",\n'
             '        PublishError::AllocationRecordsExceedOneNode { .. } => "AllocationRecordsExceedOneNode",', s)
    p.write_text(s)
p = root/"crates/singlefs-harness/src/model_comparison.rs"
s = p.read_text()
if "C381WriterStopped" not in s:
    s = once("        | PublishError::BlockDevice(_) => ObservedRefusalReason::Unexplained,",
             "        | PublishError::C381WriterStopped\n        | PublishError::BlockDevice(_) => ObservedRefusalReason::Unexplained,", s)
    p.write_text(s)
print("harness patched")
