#!/usr/bin/env python3
"""第二段：N2「同一条判据」一臂（rebuild_version 容下读不出的数据单元）与 harness 两处穷举 match。apply.py 之后跑。"""
import pathlib, sys
root = pathlib.Path(sys.argv[1])
def replace_once(path, old, new):
    text = path.read_text(); n = text.count(old)
    assert n == 1, (path, n, old[:80]); path.write_text(text.replace(old, new))
tx = root/"crates/singlefs-core/src/transaction.rs"
replace_once(tx, """        /// N3「重挂时现算」：挂载时把「分配记录已释放、却被所选根那一版引用着」的槽隔离。
        pub recompute_at_mount: bool,
    }""", """        /// N3「重挂时现算」：挂载时把「分配记录已释放、却被所选根那一版引用着」的槽隔离。
        pub recompute_at_mount: bool,
        /// N2「同一条判据」一臂：重建上一版时，数据单元（下一版不读它的字节、只会经映射释放它）读不出或对不上，
        /// 按释放核验的判定处置（不拒绝挂载、占位、交给释放核验去隔离），而不是今天的整次挂载失败。
        pub rebuild_tolerates_unreadable_data: bool,
    }""")
replace_once(tx, """            recompute_at_mount: false,
        };""", """            recompute_at_mount: false,
            rebuild_tolerates_unreadable_data: false,
        };""")
replace_once(tx, """        pub static ISOLATED_AT_MOUNT: RefCell<u64> = const { RefCell::new(0) };""", """        pub static ISOLATED_AT_MOUNT: RefCell<u64> = const { RefCell::new(0) };
        /// 重建时按「同一条判据」容下的数据单元个数。
        pub static REBUILD_TOLERATED: RefCell<u64> = const { RefCell::new(0) };""")
rec = root/"crates/singlefs-core/src/recovery.rs"
replace_once(rec, """    let data_unit_bytes = read_unit_via_locations(reader, &data_pointer.locations, data_bytes)?;""", """    let data_unit_bytes = match read_unit_via_locations(reader, &data_pointer.locations, data_bytes) {
        Ok(bytes) => bytes,
        Err(failure) => {
            if crate::transaction::release_check_model::arms().rebuild_tolerates_unreadable_data {
                crate::transaction::release_check_model::REBUILD_TOLERATED
                    .with(|count| *count.borrow_mut() += 1);
                vec![0u8; data_bytes]
            } else {
                return Err(failure.into());
            }
        }
    };""")
h = root/"crates/singlefs-harness/src/history.rs"
replace_once(h, '        PublishError::ReleaseSpanMismatch { .. } => "ReleaseSpanMismatch",\n',
 '        PublishError::ReleaseSpanMismatch { .. } => "ReleaseSpanMismatch",\n        PublishError::ReleaseCheckReadFailed { .. } => "ReleaseCheckReadFailed",\n        PublishError::ReleaseCheckReadFailedGoesToFailureTable { .. } => "ReleaseCheckReadFailedGoesToFailureTable",\n')
m = root/"crates/singlefs-harness/src/model_comparison.rs"
replace_once(m, "        | PublishError::ReleaseSpanMismatch { .. }\n",
 "        | PublishError::ReleaseSpanMismatch { .. }\n        | PublishError::ReleaseCheckReadFailed { .. }\n        | PublishError::ReleaseCheckReadFailedGoesToFailureTable { .. }\n")
print("patched2")
