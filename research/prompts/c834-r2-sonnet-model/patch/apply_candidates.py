"""C834 第二轮辩方腿：在仓副本的 mount.rs 上装候选开关（环境变量 C834_CANDIDATE），只改那一判与它的调用处。
用法：python3 apply_candidates.py <副本根>
候选：today、later_root_or_record（第一轮的 K3）、k6（第一轮正推的 K6）、pointer_slots_overwritten（第一轮攻方原型的读法，原样）、
pointer_slots_fixed（同一读法，读盘长度按单元类别取：索引节点 16 KiB，打包记录/数据单元 32 KiB）、always_refuse。
判决行里另带 diag=：R 的两个指针落点上读到什么（只读，不影响判）。"""
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

/// C834 第二轮辩方原型（只在副本里）：每块盘两槽里自证过的系统配置的实例代号取最大，按盘表次序。
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

/// 第一轮攻方原型的读法，原样：读 32 KiB，打包记录单元、数据单元、索引节点逐个试。
fn c834_instance_in_the_unit_header_at_as_in_round_one<Device: BlockDevice>(
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

/// 索引节点按 16 KiB 读，打包记录单元、数据单元按 32 KiB 读；返回（类别，实例代号）。
fn c834_unit_header_at_by_class<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    device: DeviceIdentity,
    offset: crate::address::DeviceOffsetInBytes,
) -> Option<(&'static str, InstanceGeneration)> {
    if let Some(bytes) = PoolReader::read(devices, device, offset, usize::try_from(NODE_BYTES).expect("16384")) {
        if let Ok(header) = crate::unit::parse_index_node(&bytes) {
            return Some(("index_node", header.instance));
        }
    }
    if let Some(bytes) = PoolReader::read(devices, device, offset, usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768")) {
        if let Ok(header) = crate::unit::parse_packed_unit(&bytes) {
            return Some(("packed", header.write_order.instance));
        }
        if let Ok(header) = crate::unit::parse_data_unit(&bytes) {
            return Some(("data", header.write_order.instance));
        }
    }
    None
}

fn c834_instance_in_the_unit_header_at_by_class<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    location: &crate::pointer::LocationEntry,
) -> Option<InstanceGeneration> {
    c834_unit_header_at_by_class(devices, location.device, location.slot.to_device_offset()).map(|(_, instance)| instance)
}

/// 这一槽上读到什么的文字（无空白）：槽首八字节、16 KiB 读的索引节点、32 KiB 读的单元、前一槽起 32 KiB 读的单元。
fn c834_describe_slot<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    location: &crate::pointer::LocationEntry,
) -> String {
    let offset = location.slot.to_device_offset();
    let head: String = PoolReader::read(devices, location.device, offset, 8)
        .map(|bytes| bytes.iter().map(|byte| format!("{byte:02x}")).collect())
        .unwrap_or_else(|| "unreadable".to_string());
    let here = c834_unit_header_at_by_class(devices, location.device, offset)
        .map_or("none".to_string(), |(class, instance)| format!("{class}@{}", instance.0));
    let before = if location.slot.0 > 0 {
        let previous = crate::address::DeviceOffsetInBytes(offset.0 - SLOT_BYTES);
        PoolReader::read(devices, location.device, previous, usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768"))
            .and_then(|bytes| {
                crate::unit::parse_packed_unit(&bytes)
                    .map(|header| format!("packed@{}", header.write_order.instance.0))
                    .or_else(|_| crate::unit::parse_data_unit(&bytes).map(|header| format!("data@{}", header.write_order.instance.0)))
                    .ok()
            })
            .unwrap_or_else(|| "none".to_string())
    } else {
        "none".to_string()
    };
    format!("slot{}:head={head}:here={here}:from_previous_slot={before}", location.slot.0)
}

/// C834 第二轮辩方原型：「一条记录都读不出、映射现算不成」那一判按哪个候选判拒。
fn c834_candidate_says_refuse<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    roots: &[RootRecord],
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    root: &RootRecord,
    chosen: InstanceGeneration,
    instances_of_each_device: &[(DeviceIdentity, InstanceGeneration)],
) -> bool {
    let candidate = std::env::var("C834_CANDIDATE").unwrap_or_else(|_| "today".to_string());
    let later_root_or_record = roots.iter().any(|other| other.instance > root.instance)
        || records.values().any(|record| record.instance > root.instance);
    let mapping_root_slot_holds_a_newer_index_node = root.mapping_root.locations.iter().any(|location| {
        PoolReader::read(
            devices,
            location.device,
            location.slot.to_device_offset(),
            usize::try_from(NODE_BYTES).expect("16384"),
        )
        .and_then(|bytes| crate::unit::parse_index_node(&bytes).ok())
        .is_some_and(|node| node.instance > root.instance)
    });
    let k6 = later_root_or_record || mapping_root_slot_holds_a_newer_index_node;
    let pointer_locations = || root.tree_table.locations.iter().chain(root.mapping_root.locations.iter());
    let pointer_slots_as_in_round_one = pointer_locations()
        .filter_map(|location| c834_instance_in_the_unit_header_at_as_in_round_one(devices, location))
        .any(|instance| instance > root.instance);
    let pointer_slots_fixed = pointer_locations()
        .filter_map(|location| c834_instance_in_the_unit_header_at_by_class(devices, location))
        .any(|instance| instance > root.instance);
    let refuse = match candidate.as_str() {
        "today" => chosen <= root.instance,
        "later_root_or_record" => !later_root_or_record,
        "k6" => !k6,
        "mapping_root_only" => !mapping_root_slot_holds_a_newer_index_node,
        "pointer_slots_overwritten" => !pointer_slots_as_in_round_one,
        "pointer_slots_fixed" => !pointer_slots_fixed,
        "always_refuse" => true,
        other => panic!("C834_CANDIDATE 认不出：{other}"),
    };
    let per_device = instances_of_each_device.iter().map(|(device, instance)| (device.0, instance.0)).collect::<Vec<_>>();
    let diag: Vec<String> = root
        .tree_table
        .locations
        .iter()
        .map(|location| format!("tree.d{}.{}", location.device.0, c834_describe_slot(devices, location)))
        .chain(
            root.mapping_root
                .locations
                .iter()
                .map(|location| format!("map.d{}.{}", location.device.0, c834_describe_slot(devices, location))),
        )
        .collect();
    let line = format!(
        "C834_JUDGE candidate={candidate} root=({},{}) chosen={} per_device={per_device:?} later_root_or_record={later_root_or_record} mapping_newer_index_node={mapping_root_slot_holds_a_newer_index_node} k6={k6} pointer_round_one={pointer_slots_as_in_round_one} pointer_fixed={pointer_slots_fixed} diag={} verdict={}",
        root.instance.0,
        root.checkpoint_txg.0,
        chosen.0,
        diag.join("|"),
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

/// C834 第二轮辩方原型：取走这个线程上那一判记下的行（每判一次一行），记录清空。
pub fn c834_take_judge_lines() -> Vec<String> {
    C834_JUDGE_LINES.with(|lines| std::mem::take(&mut *lines.borrow_mut()))
}
'''
path.write_text(text)
print("patched", path)
