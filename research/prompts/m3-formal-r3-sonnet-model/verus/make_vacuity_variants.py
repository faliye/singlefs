#!/usr/bin/env python3
"""七·5 空真样本：在冻结核上给 replay_core 的 requires 加一条 `false`（variants4），再加一个真调 replay_core 的见证函数（variants5）。
用法：make_vacuity_variants.py <冻结核 lib.rs> <输出目录的上一级>
"""
import pathlib, shutil, sys
base = pathlib.Path(sys.argv[1]).read_text()
root = pathlib.Path(sys.argv[2])
old = "    requires\n        forall|record_index: int|\n            0 <= record_index < records@.len() ==> (#[trigger] records@[record_index]).counter\n                < u64::MAX,\n        forall|record_index: usize|\n            record_index < records@.len() ==> #[trigger] back_chain_broken.requires((record_index,)),"
assert base.count(old) == 1
vacuous = base.replace(old, "    requires\n        false,\n" + old.split("    requires\n", 1)[1])
witness = '''
// 前置条件可满足的见证：真调一次 replay_core（空记录表）。replay_core 的 requires 写成恒假时，这一处的前置条件证不出，核红。
fn the_preconditions_of_replay_core_are_satisfiable() {
    let records: [CoreRecord<u64>; 0] = [];
    let _outcome = replay_core(1u32, 1u64, 0u64, &records, 16usize, false, |_index: usize| false, |_index: usize| true);
}
'''
marker = "// ===== spec 自己的冒烟（防 spec 恒真或恒假）====="
assert base.count(marker) == 1
with_witness = base.replace(marker, witness + "\n" + marker)
vacuous_with_witness = with_witness.replace(old, "    requires\n        false,\n" + old.split("    requires\n", 1)[1])
for variant_set, items in (("variants4", [("vacuous_requires_false", vacuous)]),
                           ("variants5", [("base_plus_witness", with_witness), ("vacuous_requires_false_plus_witness", vacuous_with_witness)])):
    out = root / variant_set
    if out.exists():
        shutil.rmtree(out)
    for name, text in items:
        d = out / name / "src"; d.mkdir(parents=True); (d / "lib.rs").write_text(text)
