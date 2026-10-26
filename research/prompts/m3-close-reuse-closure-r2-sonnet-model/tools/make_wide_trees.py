#!/usr/bin/env python3
"""W2 里「更宽的写法」的两棵试验树（不在 rerun.sh 里；out/ 下 scan-import-edge-draft.out、scan-exec-all-draft.out 就是在这两棵树上跑 scan_entries.sh 的单次产物）。
用法：make_wide_trees.py <源树> <目标目录>；建 import-edge（每个模块体都连它的模块级 import）与 exec-all（再连模块级赋值与类头）。
这两棵树的补丁与 make_trees.py 的 load_edge 同一个落点（ScriptClosure.python_edges 里模块体单元那一支），只是不限于按路径载入的模块。"""
import os
import shutil
import subprocess
import sys

source, target = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
LIB = "scripts/gate/lib_function_units.py"
OLD = '''        if unit.kind != "module-body":
            self.push(module.module_body, unit, "module-body")
        if unit.kind == "class":'''


def build(name, kinds):
    destination = os.path.join(target, name)
    if os.path.exists(destination):
        shutil.rmtree(destination)
    subprocess.run(["rsync", "-a", source + "/", destination + "/"], check=True)
    path = os.path.join(destination, LIB)
    text = open(path, encoding="utf-8").read()
    assert text.count(OLD) == 1
    new = f'''        if unit.kind != "module-body":
            self.push(module.module_body, unit, "module-body")
        elif module.inline is None:
            for sibling in module.units:
                if sibling.kind in {kinds!r} and not sibling.data:
                    self.push(sibling, unit, "module-executed")
        if unit.kind == "class":'''
    with open(path + ".new", "w", encoding="utf-8") as handle:
        handle.write(text.replace(OLD, new))
    os.replace(path + ".new", path)


os.makedirs(target, exist_ok=True)
build("import-edge", ("import",))
build("exec-all", ("import", "assign", "class"))
print("name=make_wide_trees built=import-edge,exec-all")
