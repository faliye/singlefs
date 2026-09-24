#!/usr/bin/env python3
"""第三段（接手腿加的）：写者错注入（映射里数据单元那条的槽号写错、节点照常封好落盘）与
N2「重建也核位置项」一臂（重建时把映射条目的位置项与父指针里的位置项逐条比，不一致就数一次；装了拒绝臂就整次挂载失败）。
apply.py、apply2.py 之后跑。"""
import pathlib, sys
root = pathlib.Path(sys.argv[1])
def replace_once(path, old, new):
    text = path.read_text(); n = text.count(old)
    assert n == 1, (path, n, old[:80]); path.write_text(text.replace(old, new))
tx = root/"crates/singlefs-core/src/transaction.rs"
replace_once(tx, """        pub rebuild_tolerates_unreadable_data: bool,
    }""", """        pub rebuild_tolerates_unreadable_data: bool,
        /// N2「同一条判据」另一读法：重建时核映射条目的位置项（与父指针的位置项逐条比），不一致就拒绝挂载。
        pub rebuild_refuses_on_mapping_mismatch: bool,
    }""")
replace_once(tx, """            rebuild_tolerates_unreadable_data: false,
        };""", """            rebuild_tolerates_unreadable_data: false,
            rebuild_refuses_on_mapping_mismatch: false,
        };""")
replace_once(tx, """        pub static REBUILD_TOLERATED: RefCell<u64> = const { RefCell::new(0) };""", """        pub static REBUILD_TOLERATED: RefCell<u64> = const { RefCell::new(0) };
        /// 重建时映射条目位置项与父指针不一致的条数（核开着就数，不论拒不拒）。
        pub static REBUILD_MAPPING_MISMATCH: RefCell<u64> = const { RefCell::new(0) };
        /// 写者错注入：下一次发布装映射节点时，把数据单元那条的槽号改成这个（一次性）。
        pub static WRITER_BUG_DATA_ENTRY_TO: RefCell<Option<SlotNumber>> = const { RefCell::new(None) };""")
replace_once(tx, """    let mapped_units: Vec<(TransactionUnit, Vec<u8>)> = mapped_units_with_locations""", """    // 副本（m2-newq-r1 攻方接手腿）：写者错注入，数据单元那条映射条目的槽号写错，其余照常（节点照常封、照常落盘）。
    if let Some(target) = release_check_model::WRITER_BUG_DATA_ENTRY_TO.with(|bug| bug.borrow_mut().take()) {
        let data_entry = mapped_units_with_locations
            .iter_mut()
            .find(|(identity, _, _)| *identity == TransactionUnit::Data)
            .expect("数据单元那条");
        for location in &mut data_entry.2 {
            location.slot = target;
        }
    }
    let mapped_units: Vec<(TransactionUnit, Vec<u8>)> = mapped_units_with_locations""")
rec = root/"crates/singlefs-core/src/recovery.rs"
replace_once(rec, """        mapping_keys.push(key);
    }
""", """        mapping_keys.push(key);
    }
    // 副本（m2-newq-r1 攻方接手腿）：N2「重建也核位置项」——映射条目的位置项与父指针里的逐条比（不另读盘）。
    if crate::transaction::release_check_model::arms().enabled {
        let mut hints: Vec<(Vec<u8>, [LocationEntry; 2])> = vec![
            (mapping_key_for_data(data_pointer.head, data_pointer.write_order), data_pointer.locations),
            (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer), extent_pointer.locations),
            (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer), inode_root_pointer.locations),
            (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer), allocation_pointer.locations),
            (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, accounting_pointer), accounting_pointer.locations),
        ];
        for container in &inode_leaf_containers {
            hints.push((mapping_key_for_node(UNIT_CLASS_PACKED, container.pointer), container.pointer.locations));
        }
        for entry in &mapping_node.entries {
            let (key, locations) = parse_mapping_entry(entry).expect("上面刚解过");
            if let Some((_, hint)) = hints.iter().find(|(hint_key, _)| *hint_key == key) {
                if *hint != locations {
                    crate::transaction::release_check_model::REBUILD_MAPPING_MISMATCH
                        .with(|count| *count.borrow_mut() += 1);
                    if crate::transaction::release_check_model::arms().rebuild_refuses_on_mapping_mismatch {
                        return Err(RecoveryFailure::UnitUnreadable { slot: locations[0].slot }.into());
                    }
                }
            }
        }
    }
""")
print("patched3")
