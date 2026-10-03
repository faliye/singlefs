#!/usr/bin/env python3
"""用法：edit2.py <目标 crate 目录> <源 crate 目录> <编号>：拷源，再在 verus! 末尾加第 <编号> 种没人用的 spec / proof 定义（与核无关）。"""
import shutil, sys, pathlib
dst, src, k = sys.argv[1], sys.argv[2], int(sys.argv[3])
shutil.rmtree(dst, ignore_errors=True); shutil.copytree(src, dst)
lib = pathlib.Path(dst) / "src/lib.rs"; t = lib.read_text(); end = "} // verus!"
probes = [
 "pub open spec fn attack_probe_a(x: int) -> bool { x == x }",
 "pub open spec fn attack_probe_b(x: int) -> int { x + 1 }",
 "pub open spec fn attack_probe_c(s: Seq<int>) -> bool { s.len() >= 0 }",
 "pub open spec fn attack_probe_d(x: u64) -> bool { x <= u64::MAX }",
 "pub open spec fn attack_probe_e<R>(r: Seq<CoreRecord<R>>) -> bool { r.len() == r.len() }",
 "pub closed spec fn attack_probe_f(x: int) -> bool { x > 0 }",
 "pub open spec fn attack_probe_g(a: int, b: int) -> bool { a <= b || b <= a }",
 "pub struct AttackProbeH { pub v: u64 }",
]
t = t.replace(end, probes[k] + "\n\n" + end); lib.write_text(t); print(probes[k])
