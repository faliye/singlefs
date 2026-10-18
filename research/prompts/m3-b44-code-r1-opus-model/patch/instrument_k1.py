"""m3-b44 代码轮攻方（K1）：在冻结副本拷贝的 mount.rs 上只加一处记录，不改判法。
那一判（`one_pointer_has_both_copies_overwritten_by_a_later_instance`）每被调一次，记一行 `K1_JUDGE pass=<它交回的 bool> r=(i,t) slots=…`
进线程局部，原型用 `k1_take_judge_lines` 取走。slots 是树表、映射根四个落点上 `instance_in_the_self_verified_unit_header_at` 交回的值（none 或实例代号）。
用法：python3 instrument_k1.py <副本根>
"""
import pathlib, sys

path = pathlib.Path(sys.argv[1]) / "crates/singlefs-core/src/mount.rs"
text = path.read_text()
old = """                            None
                                if !one_pointer_has_both_copies_overwritten_by_a_later_instance(
                                    devices, root,
                                ) =>"""
new = """                            None
                                if !k1_record_judge(
                                    devices,
                                    root,
                                    one_pointer_has_both_copies_overwritten_by_a_later_instance(
                                        devices, root,
                                    ),
                                ) =>"""
assert text.count(old) == 1, text.count(old)
text = text.replace(old, new)
text += r'''

thread_local! {
    static K1_JUDGE_LINES: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// m3-b44 攻方原型（只在攻方副本里）：取走那一判记下的行。
pub fn k1_take_judge_lines() -> Vec<String> {
    K1_JUDGE_LINES.with(|lines| std::mem::take(&mut *lines.borrow_mut()))
}

#[allow(clippy::ptr_arg)]
fn k1_record_judge<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    root: &RootRecord,
    pass: bool,
) -> bool {
    let mut slots = Vec::new();
    for (name, pointer) in [("tt", &root.tree_table), ("mr", &root.mapping_root)] {
        for (index, location) in pointer.locations.iter().enumerate() {
            let seen = instance_in_the_self_verified_unit_header_at(devices, location)
                .map_or_else(|| "none".to_string(), |instance| instance.0.to_string());
            slots.push(format!("{name}{index}@d{}s{}:{seen}", location.device.0, location.slot.0));
        }
    }
    K1_JUDGE_LINES.with(|lines| {
        lines.borrow_mut().push(format!(
            "K1_JUDGE pass={pass} r=({},{}) slots={}",
            root.instance.0,
            root.checkpoint_txg.0,
            slots.join(",")
        ));
    });
    pass
}
'''
path.write_text(text)
print("patched", path)
