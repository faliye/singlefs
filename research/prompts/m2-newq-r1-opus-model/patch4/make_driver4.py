#!/usr/bin/env python3
"""从第三版驱动派生第四版：加故障 F11（写者错：映射里数据单元那条指到一个没分配的槽）与 N1 候选丁、候选丁 + 记录留在已分配两臂。"""
import pathlib
here = pathlib.Path(__file__).parent
src = (here.parent / "patch3" / "m2_newq_attack3.rs").read_text()
def rep(old, new):
    global src
    n = src.count(old); assert n == 1, (n, old[:90]); src = src.replace(old, new)
rep("//! m2-newq-r1 云端攻方腿的驱动第三版（接手腿，只在副本 repo3 里跑）", "//! m2-newq-r1 云端攻方腿的驱动第四版（接手腿，只在副本 repo4 里跑；第三版之上加 F11 与 N1 候选丁）")
rep("""    WriterBugOnDiskToCarriedLeaf,
}""", """    WriterBugOnDiskToCarriedLeaf,
    /// F11 写者错（落盘）：注入那次发布把数据单元那条的槽号写成一个没分配的槽（数据槽往后 1000 槽）。
    WriterBugOnDiskToUnallocatedSlot,
}""")
rep("const ALL: [Fault; 10] = [", "const ALL: [Fault; 11] = [")
rep("""        Fault::WriterBugOnDiskToCarriedLeaf,
    ];""", """        Fault::WriterBugOnDiskToCarriedLeaf,
        Fault::WriterBugOnDiskToUnallocatedSlot,
    ];""")
rep("""            Fault::WriterBugOnDiskToCarriedLeaf => "F10-writer-bug-disk-carried-leaf",""", """            Fault::WriterBugOnDiskToCarriedLeaf => "F10-writer-bug-disk-carried-leaf",
            Fault::WriterBugOnDiskToUnallocatedSlot => "F11-writer-bug-disk-unallocated-slot",""")
rep("""rebuild_refuses_on_mapping_mismatch: false }, persisted }""", """rebuild_refuses_on_mapping_mismatch: false, reread_once_on_read_failure: false }, persisted }""")
rep("""            refuse.arms.rebuild_refuses_on_mapping_mismatch = true;
            refuse
        },
    ]""", """            refuse.arms.rebuild_refuses_on_mapping_mismatch = true;
            refuse
        },
        {
            let mut reread = arm("N1D-reread-then-mismatch-iso-mem", true, A, Any, All, Iso, false, false);
            reread.arms.reread_once_on_read_failure = true;
            reread
        },
        {
            let mut reread = arm("N1D-reread-then-mismatch-keep-allocated", true, A, Any, All, Keep, false, false);
            reread.arms.reread_once_on_read_failure = true;
            reread
        },
    ]""")
rep("""            _ => self.note_target(TransactionUnit::Data, data[0].slot),
        }
        if matches!(fault, Fault::WriterBugOnDiskToInstanceTable | Fault::WriterBugOnDiskToCarriedLeaf) {
            let target = self.target.expect("刚登记").0 .0;""", """            _ => self.note_target(TransactionUnit::Data, data[0].slot),
        }
        if fault == Fault::WriterBugOnDiskToUnallocatedSlot {
            let target = SlotNumber(data[0].slot.0 + 1000);
            assert!(self.allocator.devices.iter().all(|device| device.is_free(target)), "目标槽没分配");
            rc::WRITER_BUG_DATA_ENTRY_TO.with(|bug| *bug.borrow_mut() = Some(target));
            let o = self.overwrite();
            assert_eq!(o, "ok", "注入那次发布");
            rc::reset_counters();
            return;
        }
        if matches!(fault, Fault::WriterBugOnDiskToInstanceTable | Fault::WriterBugOnDiskToCarriedLeaf) {
            let target = self.target.expect("刚登记").0 .0;""")
rep("""            Fault::WriterBugOnDiskToInstanceTable | Fault::BadMediaReadAndWriteForever | Fault::WriterBugOnDiskToCarriedLeaf => unreachable!("上面处置过"),""",
"""            Fault::WriterBugOnDiskToInstanceTable | Fault::BadMediaReadAndWriteForever | Fault::WriterBugOnDiskToCarriedLeaf
            | Fault::WriterBugOnDiskToUnallocatedSlot => unreachable!("上面处置过"),""")
(here / "m2_newq_attack4.rs").write_text(src)
print("driver4 written")
