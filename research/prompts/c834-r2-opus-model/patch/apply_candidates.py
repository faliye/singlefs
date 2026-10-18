"""C834 第二轮攻方：在仓副本的 mount.rs 上装候选开关，只改那一判与它的调用处（不改别的产品代码）。
用法：python3 apply_candidates.py <副本根>

那一判（`placements_in_the_accounts_of_newer_readable_roots` 里「一条记录都读不出、映射现算不成」那一支）每到一次，算齐下面几样凭据、记一行
`C834_JUDGE …`（线程局部，原型用 `c834_take_judge_lines` 取走），再按「强制」开关或候选名判拒 / 放过：
  chosen                  择到那一份系统配置的实例代号（今天的判法读它）
  later_root_or_record    根环里有实例代号比 R 大的根，或读阶段扫出的记录里有实例代号比 R 大的（K3）
  mapping_root_newer      R 点名的中央映射树根两个落点上，有一个按 16 KiB 解得开索引节点、实例代号比 R 大（第一轮正推 K6 补丁那一句，逐字同读法）
  pointer_r1              第一轮攻方「指针槽读法」原样：树表与映射根四个落点各读 32 KiB，依次试打包记录单元、数据单元、索引节点（索引节点也用 32 KiB 缓冲）
  pointer_per_kind        同上四个落点，索引节点按 16 KiB 解、数据单元与打包记录单元按 32 KiB 解（改掉缓冲长度那一处，别的不变）
  slots                   四个落点各读出什么（诊断）：zero / idx:<实例> / data:<实例> / packed:<实例> / other
候选（判拒的条件）：today = chosen <= R；k3 = !later_root_or_record；k6 = !(later_root_or_record || mapping_root_newer)；
pointer_r1 = !pointer_r1；pointer_per_kind = !pointer_per_kind；always_refuse。
强制开关 `c834_force(Some(true))` 一律拒、`Some(false)` 一律放过、`None` 照候选（`c834_set_candidate`；没设过时取环境变量 `C834_CANDIDATE`，再没有是 today）。
"""
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

/// 第一轮攻方原型原样：读 32 KiB，打包记录单元、数据单元、索引节点逐个试（索引节点同样拿 32 KiB 缓冲去解）。
fn c834_instance_in_the_unit_header_at_as_in_the_first_round<Device: BlockDevice>(
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

/// 那一槽读出什么：索引节点按 16 KiB 解，数据单元与打包记录单元按 32 KiB 解；交回（种类, 实例代号）。
fn c834_unit_at<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    location: &crate::pointer::LocationEntry,
) -> (&'static str, Option<InstanceGeneration>) {
    let node = PoolReader::read(
        devices,
        location.device,
        location.slot.to_device_offset(),
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    let Some(node) = node else {
        return ("unreadable", None);
    };
    if let Ok(header) = crate::unit::parse_index_node(&node) {
        return ("idx", Some(header.instance));
    }
    if let Some(unit) = PoolReader::read(
        devices,
        location.device,
        location.slot.to_device_offset(),
        usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768"),
    ) {
        if let Ok(header) = crate::unit::parse_data_unit(&unit) {
            return ("data", Some(header.write_order.instance));
        }
        if let Ok(header) = crate::unit::parse_packed_unit(&unit) {
            return ("packed", Some(header.write_order.instance));
        }
    }
    if node.iter().all(|byte| *byte == 0) {
        ("zero", None)
    } else {
        ("other", None)
    }
}

/// K6 那一句，逐字同第一轮正推补丁：映射根两个落点读 16 KiB、解得开索引节点、实例代号比 R 大。
fn c834_mapping_root_newer<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    root: &RootRecord,
) -> bool {
    root.mapping_root.locations.iter().any(|location| {
        PoolReader::read(
            devices,
            location.device,
            location.slot.to_device_offset(),
            usize::try_from(NODE_BYTES).expect("16384"),
        )
        .and_then(|bytes| crate::unit::parse_index_node(&bytes).ok())
        .is_some_and(|node| node.instance > root.instance)
    })
}

thread_local! {
    static C834_JUDGE_LINES: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    static C834_FORCE: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
    static C834_CANDIDATE: std::cell::RefCell<String> = std::cell::RefCell::new(std::env::var("C834_CANDIDATE").unwrap_or_else(|_| String::from("today")));
}

/// C834 攻方原型：`Some(true)` 那一判一律拒，`Some(false)` 一律放过，`None` 照候选。
pub fn c834_force(force: Option<bool>) {
    C834_FORCE.with(|cell| cell.set(force));
}

/// C834 攻方原型：选候选（today、k3、k6、pointer_r1、pointer_per_kind、always_refuse）。
pub fn c834_set_candidate(candidate: &str) {
    C834_CANDIDATE.with(|cell| *cell.borrow_mut() = candidate.to_string());
}

/// C834 攻方原型：取走这个线程上那一判记下的行（每判一次一行），记录清空。
pub fn c834_take_judge_lines() -> Vec<String> {
    C834_JUDGE_LINES.with(|lines| std::mem::take(&mut *lines.borrow_mut()))
}

/// C834 攻方原型：「一条记录都读不出、映射现算不成」那一判按强制开关或候选判拒；凭据全算一遍记一行。
fn c834_candidate_says_refuse<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    roots: &[RootRecord],
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    root: &RootRecord,
    chosen: InstanceGeneration,
    instances_of_each_device: &[(DeviceIdentity, InstanceGeneration)],
) -> bool {
    let later_root_or_record = roots.iter().any(|other| other.instance > root.instance)
        || records.values().any(|record| record.instance > root.instance);
    let mapping_root_newer = c834_mapping_root_newer(devices, root);
    let pointer_locations: Vec<&crate::pointer::LocationEntry> = root
        .tree_table
        .locations
        .iter()
        .chain(root.mapping_root.locations.iter())
        .collect();
    let pointer_r1 = pointer_locations
        .iter()
        .filter_map(|location| c834_instance_in_the_unit_header_at_as_in_the_first_round(devices, location))
        .any(|instance| instance > root.instance);
    let slots: Vec<(&'static str, Option<InstanceGeneration>)> = pointer_locations
        .iter()
        .map(|location| c834_unit_at(devices, location))
        .collect();
    let pointer_per_kind = slots
        .iter()
        .any(|(_, instance)| instance.is_some_and(|instance| instance > root.instance));
    let candidate = C834_CANDIDATE.with(|cell| cell.borrow().clone());
    let by_candidate = match candidate.as_str() {
        "today" => chosen <= root.instance,
        "k3" => !later_root_or_record,
        "k6" => !(later_root_or_record || mapping_root_newer),
        "pointer_r1" => !pointer_r1,
        "pointer_per_kind" => !pointer_per_kind,
        "always_refuse" => true,
        other => panic!("C834 候选认不出：{other}"),
    };
    let force = C834_FORCE.with(std::cell::Cell::get);
    let refuse = force.unwrap_or(by_candidate);
    let slot_text: Vec<String> = slots
        .iter()
        .map(|(kind, instance)| match instance {
            Some(instance) => format!("{kind}:{}", instance.0),
            None => (*kind).to_string(),
        })
        .collect();
    let line = format!(
        "C834_JUDGE root=({},{}) chosen={} per_device={:?} later_root_or_record={later_root_or_record} mapping_root_newer={mapping_root_newer} pointer_r1={pointer_r1} pointer_per_kind={pointer_per_kind} slots=tt0:{},tt1:{},mr0:{},mr1:{} force={:?} candidate={candidate} verdict={}",
        root.instance.0,
        root.checkpoint_txg.0,
        chosen.0,
        instances_of_each_device.iter().map(|(device, instance)| (device.0, instance.0)).collect::<Vec<_>>(),
        slot_text[0],
        slot_text[1],
        slot_text[2],
        slot_text[3],
        force,
        if refuse { "refuse" } else { "pass" }
    );
    C834_JUDGE_LINES.with(|lines| lines.borrow_mut().push(line));
    refuse
}
'''
path.write_text(text)
print("patched", path)
