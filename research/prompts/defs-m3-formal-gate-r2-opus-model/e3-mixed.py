#!/usr/bin/env python3
"""E3：同一格里既有「没跑成」（包装退 252）又有真红时，判法打出来的是什么。函数级探针：在草稿拷贝里把 verus-proof-check.py 当模块载入，
Verus 换成它自证用的假 Verus（FAKE_VERUS），内存包装换成一个桩（副本源码里有 STUB_252 就退 252、不跑；没有就照原样起那条命令）。
不起真 Verus、不编、不起真包装、不跑门禁。用法：python3 e3-mixed.py <草稿拷贝根> <草稿目录>"""
import contextlib, importlib.util, io, json, os, sys

copy, work = sys.argv[1], sys.argv[2]
os.makedirs(work, exist_ok=True)
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("vpc", os.path.join(copy, "research", "scripts", "verus-proof-check.py"))
vpc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(vpc)
stub = os.path.join(work, "stub-wrapper.sh")
with open(stub, "w") as handle:
    handle.write('#!/usr/bin/env bash\nif grep -rqs STUB_252 src; then echo "  ✗ 桩：内存不够排不上" >&2; exit 252; fi\nshift\nexec "$@"\n')
vpc.MEMORY_CAP_RUNNER = stub
install = os.path.join(work, "verus-install")
os.makedirs(install, exist_ok=True)
open(os.path.join(install, "verus"), "w").write(vpc.FAKE_VERUS)
os.chmod(os.path.join(install, "verus"), 0o755)
commit = "a" * 40
json.dump({"verus": {"version": "0.9.9.tag", "commit": commit, "toolchain": "stable"}}, open(os.path.join(install, "version.json"), "w"))


def make_root(name, kernel, rows):
    root = os.path.join(work, name)
    os.makedirs(os.path.join(root, ".claude", "gate.d"), exist_ok=True)
    os.makedirs(os.path.join(root, "crates", "core", "src"), exist_ok=True)
    open(os.path.join(root, vpc.REGISTRY), "w").write(
        f"release\t0.9.9.tag\t{commit}\t{'d' * 64}\tunused\npackage\tvstd\t0.0.1\t{'c' * 40}\tsource/vstd\t{'e' * 64}\n"
        "crate\tcrates/core\t2\nkernel\tcrates/core/src/kernel.rs\nmust-red-worlds\tcrates/core/verus-mutations.tsv\nrlimit\t7\n")
    open(os.path.join(root, "crates", "core", "Cargo.toml"), "w").write('[package]\nname = "core-sample"\nedition = "2021"\n\n[dependencies]\nvstd = "=0.0.1"\n')
    open(os.path.join(root, "crates", "core", "src", "lib.rs"), "w").write("pub mod kernel;\n")
    open(os.path.join(root, "crates", "core", "src", "kernel.rs"), "w").write(kernel)
    open(os.path.join(root, "crates", "core", "verus-mutations.tsv"), "w").write("# 探针\n" + "".join(row + "\n" for row in rows))
    return root


options = {"verus_root": install, "cargo_home": None, "launcher": None, "timeout": 30, "threads": 2, "jobs": 2, "memory_cap": "1G"}
kernel = vpc.SELFTEST_KERNEL
survives = "改坏了也验得过\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\tif value > limit { limit } else { value } // 无害\tpostcondition not satisfied"
queued = "这一行排不上\tcrates/core/src/kernel.rs\t            candidate <= 10,\t            candidate <= 10, // STUB_252\tpostcondition not satisfied"
cases = [
    ("M1 必红世界：一行真没红、一行包装退 252（同一格）", "must-red-worlds", make_root("m1", kernel, [survives, queued])),
    ("M2 证明格：核里有 assume（作弊标记），同一趟 Verus 包装退 252（同一格）", "proof",
     make_root("m2", kernel.replace("if value > limit { limit }", "assume(false); // STUB_252\n    if value > limit { limit }"), [survives])),
    ("M3 对照：空真格每一处都排不上，别的都没事", "vacuity", make_root("m3", kernel + "// STUB_252\n", [survives])),
]
for label, check, root in cases:
    buffer = io.StringIO()
    with contextlib.redirect_stdout(buffer):
        code = vpc.run_check(check, root, options)
    print(f"== {label}：{check} 退 {code}")
    for line in buffer.getvalue().rstrip("\n").split("\n"):
        print(line.replace(work, "<草稿>"))
