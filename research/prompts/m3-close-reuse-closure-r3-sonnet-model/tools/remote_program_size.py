#!/usr/bin/env python3
"""remote_program_size.py <树>：今天送到第二台的探测程序（host-resources.py 的 remote_probe_program：memory_peaks_lib.py 源码 + probe_this_host 的源码 + 驱动）的字节数与 base64 长度，
对比 Linux 单个参数 131071 字节的上限；另用 ast 看那几份文件的语法最低版本与较新标准库。"""
import ast, base64, os, sys
tree = os.path.abspath(sys.argv[1])
sys.path.insert(0, os.path.join(tree, "scripts", "hosts"))
import importlib.util
text = open(os.path.join(tree, "scripts/hosts/host-resources.py"), encoding="utf-8").read()
library = open(os.path.join(tree, "scripts/hosts/memory_peaks_lib.py"), encoding="utf-8").read()
node = next(n for n in ast.parse(text).body if isinstance(n, ast.FunctionDef) and n.name == "probe_this_host")
probe = "\n".join(text.splitlines()[node.lineno - 1:node.end_lineno]) + "\n"
program = library + "\n" + probe + "\nimport json, os, sys\nprint(...)\n"
encoded = base64.b64encode(program.encode()).decode()
print(f"name=remote_program_today bytes={len(program.encode())} base64_bytes={len(encoded)} single_argument_limit=131071 fits={len(encoded) <= 131071}")
NEWER = {"tomllib": "3.11", "zoneinfo": "3.9", "graphlib": "3.9"}
for relative in (".claude/hooks/lib_heavy_tests.py", ".claude/hooks/lib_long_run.py", ".claude/hooks/lib_shell_words.py", ".claude/scripts/project_preflight.py",
                 "scripts/hosts/host-resources.py", "scripts/hosts/memory_peaks_lib.py", "scripts/hosts/host-dispatch.py", ".claude/singlefs-ai-sop/scripts/preflight.py",
                 "scripts/hosts/core_usage.py", "scripts/hosts/heavy_threads.py"):
    source = open(os.path.join(tree, relative), encoding="utf-8").read()
    minimum = None
    for minor in range(7, 14):
        try:
            ast.parse(source, feature_version=(3, minor)); minimum = minor; break
        except SyntaxError:
            continue
    names = set()
    for item in ast.walk(ast.parse(source)):
        if isinstance(item, ast.Import): names |= {a.name.split(".")[0] for a in item.names}
        elif isinstance(item, ast.ImportFrom) and item.module: names.add(item.module.split(".")[0])
    print(f"name=python_version file={relative} syntax_min=3.{minimum} newer_stdlib={ {n: NEWER[n] for n in names if n in NEWER} }")
