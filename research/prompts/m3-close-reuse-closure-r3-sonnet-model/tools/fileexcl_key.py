#!/usr/bin/env python3
"""fileexcl_key.py <树> [额外排除文件…]：用 lib_verdict_library 现成的 excluded_files 参数算三个形式化入口与 e165 replay 的判法单元键（前 16 位）与单元数、拒算数。"""
import os, sys
tree = os.path.abspath(sys.argv[1]); extra = tuple(sys.argv[2:])
sys.path.insert(0, os.path.join(tree, "scripts/crash")); sys.path.insert(0, os.path.join(tree, "scripts/gate"))
import lib_verdict_library as L
for e in ("scripts/gate/verus-proof-check.py::check_proof", "scripts/gate/verus-proof-check.py::check_vacuity", "scripts/gate/verus-proof-check.py::check_must_red_worlds"):
    k = L.FunctionUnitIndex(tree).key([e], excluded_files=tuple(L.NOT_JUDGING_FILES) + extra, programs_keyed_elsewhere=True)
    print(f"{e.split('::')[1]}\t键={(k.key or 'None')[:16]}\t单元={len(k.units)}\t拒算={len(k.refusals)}")
