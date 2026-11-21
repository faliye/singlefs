use super::*;

// ============================================================================
// 十、计数、三条臂（丙 / 甲一 / 甲二）与一个状态上的分类。
// ============================================================================

/// 一片（或整条历史）的计数：名字 → 个数，按名字排序；合并是逐名相加，所以与片的次序无关。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Counters(pub(super) BTreeMap<String, u64>);

impl Counters {
    pub(super) fn add(&mut self, name: &str, amount: u64) {
        *self.0.entry(name.to_string()).or_insert(0) += amount;
    }

    pub(super) fn count_named(&self, name: &str) -> u64 {
        self.0.get(name).copied().unwrap_or(0)
    }

    pub(super) fn merge(&mut self, other: &Counters) {
        for (name, amount) in &other.0 {
            self.add(name, *amount);
        }
    }

    pub(super) fn to_text(&self) -> String {
        self.0
            .iter()
            .map(|(name, amount)| format!("{name}\t{amount}\n"))
            .collect()
    }

    pub(super) fn from_text(text: &str) -> Result<Counters, String> {
        let mut counters = Counters::default();
        for line in text.lines().filter(|line| !line.starts_with('#')) {
            let (name, amount) = line
                .split_once('\t')
                .ok_or_else(|| format!("计数行没有制表符：{line}"))?;
            let amount = amount
                .parse::<u64>()
                .map_err(|error| format!("计数行 {name} 的值不是整数：{error}"))?;
            counters.add(name, amount);
        }
        Ok(counters)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Arm {
    /// 丙（今天，真实基线）：照 `recovery::recover` 择。
    Today,
    /// 甲一：剩下那块盘上读得出更新实例的记录、按它的新根段读得出的实例表判所选根被抛弃，就不择它。
    NewerInstanceRecord,
    /// 甲二：剩下那块盘上系统配置记着的实例代号比所选根新，就不择它。
    ConfigurationInstance,
}

impl Arm {
    pub(super) const EVERY_ARM: [Arm; 3] = [
        Arm::Today,
        Arm::NewerInstanceRecord,
        Arm::ConfigurationInstance,
    ];

    pub(super) fn name(self) -> &'static str {
        match self {
            Arm::Today => "today",
            Arm::NewerInstanceRecord => "newer_instance_record",
            Arm::ConfigurationInstance => "configuration_instance",
        }
    }
}

/// 甲二的判：系统配置里记着的实例代号严格大于所选根的实例代号才避开。
fn avoided_by_configuration_instance(configuration_instance: u32, root_instance: u32) -> bool {
    configuration_instance > root_instance
}

/// 剩下那块盘上自证过的系统配置槽里记着的实例代号的最大值（两个槽都读）。
pub(super) fn newest_configuration_instance_on_the_kept_disk(
    reader: &dyn PoolReader,
    device: DeviceIdentity,
) -> Option<u32> {
    (0..2u64)
        .filter_map(|slot_index| {
            let bytes = reader.read(
                device,
                DeviceOffsetInBytes(slot_index * u64::from(FIXED_STRUCTURE_SLOT_SPACING_LOCAL)),
                SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL,
            )?;
            SystemConfiguration::parse_slot(&bytes).ok()
        })
        .map(|configuration| configuration.quantities.journal_instance.0)
        .max()
}

/// 甲一的判：读者只给剩下那一块盘。找实例代号比所选根新、自证过的记录里最新的一条，读它新根段点名的实例表，按实例表的行判所选根被抛弃。
/// 看不见更新实例的记录、或表读不出，都不避开（与今天相同）。
fn avoided_by_newer_instance_record(reader: &dyn PoolReader, root: &RootRecord) -> bool {
    let Ok(configuration) = choose_system_configuration(reader) else {
        return false;
    };
    let records = scan_journal(reader, &configuration);
    let Some(newest_newer_record) = records
        .values()
        .filter(|record| record.instance.0 > root.instance.0)
        .max_by_key(|record| (record.instance.0, record.counter))
    else {
        return false;
    };
    let mut root_with_the_newer_instance_table = *root;
    root_with_the_newer_instance_table.instance_table = newest_newer_record.new_instance_table;
    match instance_table_chain_of_root(reader, &root_with_the_newer_instance_table) {
        Ok(chain) => root_is_abandoned_by_the_instance_table(root, &chain.records),
        Err(_) => false,
    }
}

pub(super) fn outcome_class(outcome: &RecoveryOutcome) -> &'static str {
    match outcome {
        RecoveryOutcome::FileRead { .. } => "file_read",
        RecoveryOutcome::NoFile { .. } => "no_file",
        RecoveryOutcome::Failed { root: Some(_), .. } => "failed_with_root",
        RecoveryOutcome::Failed { root: None, .. } => "failed_without_root",
    }
}

/// 恢复报告里的根（所选的那一条，或退回见证到的那一版时退到的那一条）。
pub(super) fn outcome_root(outcome: &RecoveryOutcome) -> Option<RootKey> {
    let key_of =
        |root: &(InstanceGeneration, singlefs_core::address::CheckpointTxg)| (root.0 .0, root.1 .0);
    match outcome {
        RecoveryOutcome::FileRead { root, .. } | RecoveryOutcome::NoFile { root } => {
            Some(key_of(root))
        }
        RecoveryOutcome::Failed { root, .. } => root.as_ref().map(key_of),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RootPick {
    /// 择到的是被藏根，它引用的某个单元在用到的盘上已被盖。
    HiddenRootCovered,
    /// 择到的是被藏根，单元都还是写下时的字节。
    HiddenRootIntact,
    /// 择到的不是被藏根，它引用的某个单元被盖了。
    OtherRootCovered,
    /// 合法单盘恢复：择到的根不是被抛弃根，引用的单元逐字节等于写下时的字节。
    Legal,
}

impl RootPick {
    pub(super) fn name(self) -> &'static str {
        match self {
            RootPick::HiddenRootCovered => "hidden_root_covered",
            RootPick::HiddenRootIntact => "hidden_root_intact",
            RootPick::OtherRootCovered => "other_root_covered",
            RootPick::Legal => "legal",
        }
    }
}

pub(super) fn classify_root_pick(
    key: RootKey,
    hidden_root: Option<RootKey>,
    covered: bool,
) -> RootPick {
    match (Some(key) == hidden_root, covered) {
        (true, true) => RootPick::HiddenRootCovered,
        (true, false) => RootPick::HiddenRootIntact,
        (false, true) => RootPick::OtherRootCovered,
        (false, false) => RootPick::Legal,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ArmResult {
    pub(super) key: Option<RootKey>,
    pub(super) outcome: &'static str,
    /// 这条臂连着避开了几条根（丙恒 0）。
    pub(super) avoided_roots: usize,
    /// 要避开却找不到那条根的根槽位置，没能避开。
    pub(super) could_not_mask: bool,
}

fn root_record_of(reader: &dyn PoolReader, key: RootKey) -> Option<RootRecord> {
    let configuration = choose_system_configuration(reader).ok()?;
    readable_roots(
        reader,
        &configuration.immutable.region_devices,
        &configuration.immutable.sizes,
        &configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .find(|root| root_key_of(root) == key)
}

/// 一条根的根槽在哪块盘的哪个偏移（装置自己的独立解码：checker 的 `valid_roots` 与 `root_slot_positions`），按字节区间 (偏移, 长度)。
fn root_slot_position(
    view: &KeptDiskView<'_, '_>,
    geometry: &PoolGeometry,
    key: RootKey,
) -> Option<(u64, u64)> {
    let positions = checker_image::root_slot_positions(geometry);
    checker_image::valid_roots(view, geometry)
        .into_iter()
        .find(|(_, _, root_view)| (root_view.instance, root_view.checkpoint_txg) == key)
        .and_then(|(region, slot, _)| {
            positions
                .iter()
                .find(|(candidate_region, candidate_slot, _, _)| {
                    *candidate_region == region && *candidate_slot == slot
                })
                .map(|(_, _, _, offset)| (*offset, u64::from(geometry.physical_block_size)))
        })
}

/// 一条臂在一块剩下的盘上的结局。`first_report` 是不避开任何根时 `recover` 的报告（丙就是它）；甲一、甲二避开一条根就把它的根槽读成全零、
/// 再跑一遍 `recover`（于是后面的施加记录、退回见证到的那一版都照今天的做），直到择到的根不再被避开、或择不到根。
pub(super) fn arm_result_on_the_kept_disk(
    image: &CrashImage<'_>,
    device: DeviceIdentity,
    geometry: &PoolGeometry,
    arm: Arm,
    first_report: &RecoveryReport,
) -> ArmResult {
    let mut masked_root_slots: Vec<(u64, u64)> = Vec::new();
    let mut avoided_roots = 0usize;
    let mut could_not_mask = false;
    let mut key = outcome_root(&first_report.outcome);
    let mut outcome = outcome_class(&first_report.outcome);
    while let Some(picked) = key {
        let view = KeptDiskView {
            image,
            device,
            masked_root_slots: masked_root_slots.clone(),
        };
        let avoid = match arm {
            Arm::Today => false,
            Arm::NewerInstanceRecord => match root_record_of(&view, picked) {
                Some(root) => avoided_by_newer_instance_record(&view, &root),
                None => false,
            },
            Arm::ConfigurationInstance => newest_configuration_instance_on_the_kept_disk(
                &view, device,
            )
            .is_some_and(|configuration_instance| {
                avoided_by_configuration_instance(configuration_instance, picked.0)
            }),
        };
        if !avoid || avoided_roots >= AVOIDED_ROOTS_AT_MOST {
            break;
        }
        let Some(position) = root_slot_position(&view, geometry, picked) else {
            could_not_mask = true;
            break;
        };
        masked_root_slots.push(position);
        avoided_roots += 1;
        let next_report = recover(
            &KeptDiskView {
                image,
                device,
                masked_root_slots: masked_root_slots.clone(),
            },
            JournalPolicy::Consult,
        );
        key = outcome_root(&next_report.outcome);
        outcome = outcome_class(&next_report.outcome);
    }
    ArmResult {
        key,
        outcome,
        avoided_roots,
        could_not_mask,
    }
}

#[cfg(test)]
mod tests;
