#!/usr/bin/env python3
"""第四段（接手腿加的）：N1 候选丁「读失败先只读复核一次（同一份镜像再读一遍），复核读得出按读出的字节判，仍读不出按对不上处置」。
apply.py、apply2.py、apply3.py 之后跑。"""
import pathlib, sys
root = pathlib.Path(sys.argv[1])
def replace_once(path, old, new):
    text = path.read_text(); n = text.count(old)
    assert n == 1, (path, n, old[:80]); path.write_text(text.replace(old, new))
tx = root/"crates/singlefs-core/src/transaction.rs"
replace_once(tx, """        pub rebuild_refuses_on_mapping_mismatch: bool,
    }""", """        pub rebuild_refuses_on_mapping_mismatch: bool,
        /// N1 候选丁：一份镜像读失败时对同一落点再读一次（只读），读得出就按读出的字节判。
        pub reread_once_on_read_failure: bool,
    }""")
replace_once(tx, """            rebuild_refuses_on_mapping_mismatch: false,
        };""", """            rebuild_refuses_on_mapping_mismatch: false,
            reread_once_on_read_failure: false,
        };""")
replace_once(tx, """            let verdict = match device.read_at(location.slot.to_device_offset(), &mut buffer) {
                Err(_) => MirrorVerdict::ReadFailed,""", """            let mut first = device.read_at(location.slot.to_device_offset(), &mut buffer);
            if first.is_err() && arms().reread_once_on_read_failure {
                READS.with(|reads| {
                    let mut reads = reads.borrow_mut();
                    reads.0 += 1;
                    reads.1 += u64::try_from(length).expect("字节数");
                });
                first = device.read_at(location.slot.to_device_offset(), &mut buffer);
            }
            let verdict = match first {
                Err(_) => MirrorVerdict::ReadFailed,""")
print("patched4")
