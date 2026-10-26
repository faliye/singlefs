"""W2：包装排队那一步真执行到的仓内 python 模块（照内联脚本的写法：sys.path 插 scripts/hosts、import memory_peaks_lib，再照 predicted_bytes 按路径 exec_module host-dispatch.py），
逐份列它们模块级会在 import 时执行的单元（import、assign、class），比一个入口的闭包：哪些不在闭包里（改它们、键不变，而 import 时照样执行）。
用法：w2-import-census.py <树> <闭包输出（closure --explain 的原样输出）> <dump 输出（dump --language python 的原样输出）> <标签>"""
import importlib.util
import os
import subprocess
import sys

root, closure_path, dump_path, label = sys.argv[1:5]
probe = ("import importlib.util, os, sys\n"
         "sys.dont_write_bytecode = True\n"
         "sys.path.insert(0, 'scripts/hosts')\n"
         "before = set(sys.modules)\n"
         "import memory_peaks_lib\n"
         "spec = importlib.util.spec_from_file_location('host_dispatch', 'scripts/hosts/host-dispatch.py')\n"
         "module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)\n"
         "root = os.path.realpath('.')\n"
         "for name in sorted(set(sys.modules) - before):\n"
         "    path = getattr(sys.modules[name], '__file__', None)\n"
         "    if path and os.path.realpath(path).startswith(root + os.sep):\n"
         "        print(os.path.relpath(os.path.realpath(path), root))\n"
         "print('scripts/hosts/host-dispatch.py')\n")
completed = subprocess.run([sys.executable, "-c", probe], cwd=root, capture_output=True, text=True)
if completed.returncode != 0:
    tail = completed.stderr.strip().splitlines()[-1] if completed.stderr.strip() else ""
    print(f"name=import_census tree={label} import_failed=1 rc={completed.returncode} last_line={tail[:160]}")
    sys.exit(0)
executed = completed.stdout.split()


def rows(path):
    found = set()
    for line in open(path, encoding="utf-8"):
        fields = line.rstrip("\n").split("\t")
        if len(fields) >= 4 and fields[0] not in ("via", "refused", "key"):
            found.add((fields[0], fields[1], fields[2].rsplit(":", 1)[0]))
    return found


closure, dumped = rows(closure_path), rows(dump_path)
total_missing = 0
for file in executed:
    module_level = sorted(row for row in dumped if row[2] == file and row[0] in ("import", "assign", "class"))
    missing = [row for row in module_level if row not in closure]
    total_missing += len(missing)
    print(f"name=import_census tree={label} file={file} module_level={len(module_level)} not_in_closure={len(missing)} "
          f"examples={','.join(f'{kind}:{name}' for kind, name, _ in missing[:6])}")
print(f"name=import_census_total tree={label} executed_files={len(executed)} not_in_closure={total_missing}")
