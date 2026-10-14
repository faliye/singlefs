"""在仓副本的 mount.rs 上装 C834 候选开关（环境变量 C834_CANDIDATE），只改那一判与它的调用处。用法：python3 apply_candidates.py <副本根>"""
import pathlib, sys

root = pathlib.Path(sys.argv[1])
path = root / "crates/singlefs-core/src/mount.rs"
text = path.read_text()

def once(old, new):
    global text
    assert text.count(old) == 1, (old[:80], text.count(old))
    text = text.replace(old, new)

once("""    journal_instance_in_the_system_configuration: InstanceGeneration,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<PlacementsInTheAccountsOfNewerReadableRoots, MountError> {""",
"""    journal_instance_in_the_system_configuration: InstanceGeneration,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
    c834_instances_of_each_device: &[(DeviceIdentity, InstanceGeneration)],
) -> Result<PlacementsInTheAccountsOfNewerReadableRoots, MountError> {""")

once("""                            None if journal_instance_in_the_system_configuration
                                <= root.instance =>
                            {""",
"""                            None if c834_candidate_says_refuse(
                                devices,
                                roots,
                                records,
                                root,
                                journal_instance_in_the_system_configuration,
                                c834_instances_of_each_device,
                            ) =>
                            {""")

once("""                system_configuration.quantities.journal_instance,
                before_the_one_reread,
            )?;
            for (ring_slot, placements_in_the_newer_roots_account) in""",
"""                system_configuration.quantities.journal_instance,
                before_the_one_reread,
                &c834_instances_of_each_device(devices, system_configuration),
            )?;
            for (ring_slot, placements_in_the_newer_roots_account) in""")

text += r'''

/// C834 攻方原型（只在攻方副本里）：每块盘两槽里自证过的系统配置的实例代号取最大，按盘表次序。
fn c834_instances_of_each_device<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    system_configuration: &crate::system_configuration::SystemConfiguration,
) -> Vec<(DeviceIdentity, InstanceGeneration)> {
    let spacing = u64::from(system_configuration.immutable.sizes.fixed_structure_slot_spacing);
    let filesystem_identifier = system_configuration.immutable.filesystem_identifier;
    PoolReader::device_identities(devices)
        .into_iter()
        .map(|device| {
            let instance = verified_system_configuration_slots(
                devices,
                device,
                spacing,
                &filesystem_identifier,
            )
            .iter()
            .map(|configuration| configuration.quantities.journal_instance)
            .max()
            .unwrap_or(InstanceGeneration(0));
            (device, instance)
        })
        .collect()
}

/// C834 攻方原型：读 `location` 那一槽的单元头（打包记录单元、索引节点、数据单元三种逐个试），交回解得开的那一种头里的实例代号。
fn c834_instance_in_the_unit_header_at<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    location: &crate::pointer::LocationEntry,
) -> Option<InstanceGeneration> {
    let bytes = PoolReader::read(
        devices,
        location.device,
        location.slot.to_device_offset(),
        usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768"),
    )?;
    if let Ok(header) = crate::unit::parse_packed_unit(&bytes) {
        return Some(header.write_order.instance);
    }
    if let Ok(header) = crate::unit::parse_data_unit(&bytes) {
        return Some(header.write_order.instance);
    }
    if let Ok(header) = crate::unit::parse_index_node(&bytes) {
        return Some(header.instance);
    }
    None
}

/// C834 攻方原型：「一条记录都读不出、映射现算不成」那一判按哪个候选判拒。`C834_CANDIDATE` 没设或设成 `today` 时与冻结副本逐字同判。
fn c834_candidate_says_refuse<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    roots: &[RootRecord],
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    root: &RootRecord,
    chosen: InstanceGeneration,
    instances_of_each_device: &[(DeviceIdentity, InstanceGeneration)],
) -> bool {
    let candidate = std::env::var("C834_CANDIDATE").unwrap_or_else(|_| "today".to_string());
    let largest = instances_of_each_device.iter().map(|(_, instance)| *instance).max().unwrap_or(chosen);
    let smallest = instances_of_each_device.iter().map(|(_, instance)| *instance).min().unwrap_or(chosen);
    let later_root_or_record = roots.iter().any(|other| other.instance > root.instance)
        || records.values().any(|record| record.instance > root.instance);
    let pointer_slots_written_by_a_later_instance = root
        .tree_table
        .locations
        .iter()
        .chain(root.mapping_root.locations.iter())
        .filter_map(|location| c834_instance_in_the_unit_header_at(devices, location))
        .any(|instance| instance > root.instance);
    let refuse = match candidate.as_str() {
        "today" => chosen <= root.instance,
        "largest" => largest <= root.instance,
        "smallest" => smallest <= root.instance,
        "later_root_or_record" => !later_root_or_record,
        "pointer_slots_overwritten" => !pointer_slots_written_by_a_later_instance,
        "always_refuse" => true,
        other => panic!("C834_CANDIDATE 认不出：{other}"),
    };
    let line = format!(
        "C834_JUDGE candidate={candidate} root=({},{}) chosen={} per_device={:?} later_root_or_record={later_root_or_record} pointer_slots_overwritten={pointer_slots_written_by_a_later_instance} verdict={}",
        root.instance.0,
        root.checkpoint_txg.0,
        chosen.0,
        instances_of_each_device.iter().map(|(device, instance)| (device.0, instance.0)).collect::<Vec<_>>(),
        if refuse { "refuse" } else { "pass" }
    );
    if std::env::var("C834_VERBOSE").is_ok() {
        eprintln!("{line}");
    }
    C834_JUDGE_LINES.with(|lines| lines.borrow_mut().push(line));
    refuse
}

thread_local! {
    static C834_JUDGE_LINES: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// C834 攻方原型：取走这个线程上那一判记下的行（每判一次一行），记录清空。
pub fn c834_take_judge_lines() -> Vec<String> {
    C834_JUDGE_LINES.with(|lines| std::mem::take(&mut *lines.borrow_mut()))
}
'''
path.write_text(text)
print("patched", path)
