#!/usr/bin/env python3
"""W2 自证格的原型：predicted_bytes 真正做的那件事（按文件路径载入 host-dispatch.py）在一棵树上实际 import 了仓内哪几份文件，
与某次 closure 输出里出现的文件比，闭包里少的列出来。两条路不共用代码：一边是 Python 自己的 import，一边是 lib_function_units.py 的静态闭包。
用法：import_trace.py <树根> <closure 输出（每行制表符分隔，第三列 文件:起-止）>；少了退 1。"""
import importlib.util
import os
import sys

root, closure_path = os.path.abspath(sys.argv[1]), sys.argv[2]
hosts = os.path.join(root, "scripts", "hosts")
sys.dont_write_bytecode = True
sys.path.insert(0, hosts)
before = set(sys.modules)
spec = importlib.util.spec_from_file_location("host_dispatch", os.path.join(hosts, "host-dispatch.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
loaded = sorted({os.path.relpath(os.path.abspath(sys.modules[name].__file__), root) for name in set(sys.modules) - before
                 if getattr(sys.modules[name], "__file__", None) and os.path.abspath(sys.modules[name].__file__).startswith(root + os.sep)})
loaded.append("scripts/hosts/host-dispatch.py")
in_closure = set()
for line in open(closure_path, encoding="utf-8"):
    columns = line.rstrip("\n").split("\t")
    if columns[0] not in ("via", "key", "refused") and len(columns) >= 4:
        in_closure.add(columns[2].rsplit(":", 1)[0])
missing = [path for path in loaded if path not in in_closure]
print(f"TRACE 载入 host-dispatch.py 实际 import 的仓内文件 {len(loaded)} 份：{' '.join(loaded)}")
print(f"TRACE 闭包里没有的 {len(missing)} 份：{' '.join(missing) or '无'}")
sys.exit(1 if missing else 0)
