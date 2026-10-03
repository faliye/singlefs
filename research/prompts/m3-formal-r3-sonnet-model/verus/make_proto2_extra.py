#!/usr/bin/env python3
"""proto2 上再造几份：必红世界表第 36、448 行（lib.rs 上的两行）、spec 冒烟（链条改成 +2）、代码冒烟（expected_next 改成 +2）。"""
import pathlib, shutil, sys
src = pathlib.Path("proto2/src/lib.rs").read_text()
tsv = pathlib.Path("/tmp/claude-1000/m3-formal-r3/tree/crates/mutations.tsv").read_text().split("\n")
out = pathlib.Path("variants3")
if out.exists(): shutil.rmtree(out)
def put(name, text):
    d = out / name / "src"; d.mkdir(parents=True); (d / "lib.rs").write_text(text)
for row in (36, 448):
    f = tsv[row - 1].split("\t")
    old, new = f[2].replace("\\n", "\n"), f[3].replace("\\n", "\n")
    assert f[1].endswith("singlefs-replay-prefix/src/lib.rs")
    assert src.count(old) == 1, (row, src.count(old))
    put(f"tsv{row}", src.replace(old, new))
a = "== records[chain[position]].counter + 1"
assert src.count(a) == 1
put("spec_smoke_chain_plus2", src.replace(a, "== records[chain[position]].counter + 2"))
b = "        expected_next = Some((record.instance, record.counter + 1));"
assert src.count(b) == 1
put("code_smoke_expected_plus2", src.replace(b, "        expected_next = Some((record.instance, record.counter + 2));"))
c = "        Some(counter) => records[chain[0]].counter == counter + 1,"
assert src.count(c) == 1
put("spec_smoke_head_plus2", src.replace(c, "        Some(counter) => records[chain[0]].counter == counter + 2,"))
(out / "base" / "src").mkdir(parents=True); (out / "base/src/lib.rs").write_text(src)
print(sorted(p.name for p in out.iterdir()))
