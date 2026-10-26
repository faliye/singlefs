#!/usr/bin/env python3
"""make_variants_all.py <基准树> <变体目录>：从基准树拷出全部试验变体树，只用来量，不是实现。每一处旧串必须恰好命中一次。
  cut-src             源码层切边：run-with-memory-cap.sh 里 HostLookup.query 起 host-resources.py 的 Popen 换成 None（与原腿同法，作对照）
  cut-algo            算法层切边：python_edges 里 HostLookup.query 的 program 连边中被连到 host-resources.py 的那一条不推
  stub-hr             host-resources.py 整份换成一行（边还在，被连的文件没有内容）
  modlevel            算法层：任何模块的模块体进闭包时，它的模块级 import 与 assign 单元也进闭包（宽读法；不切边）
  cut-modlevel        cut-algo 加 modlevel
  cut-modlevel-class  cut-modlevel，环境变量 R3_ML_KINDS=import,assign,class 时连 class 头（扫描时设）
  narrow              算法层：只有按路径载入的模块、与它们 import 引来的仓内模块，才收模块级 import（窄读法；不切边）
  cut-narrow          cut-algo 加 narrow（R3_NARROW_KINDS=import,assign 时另收 assign）
  mut-a / mut-b       基准树上各改一处：lib_long_run.py 的 import re 写成 import re as re；memory_peaks_lib.py 的 import subprocess 写成 import subprocess as subprocess"""
import os, shutil, subprocess, sys
base, target = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
LIB = "scripts/gate/lib_function_units.py"
WRAP = "scripts/hosts/run-with-memory-cap.sh"
os.makedirs(target, exist_ok=True)

def clone(src, name):
    dest = os.path.join(target, name)
    if os.path.exists(dest): shutil.rmtree(dest)
    subprocess.run(["rsync", "-a", src.rstrip("/") + "/", dest + "/"], check=True)
    return dest

def edit(tree, rel, old, new):
    p = os.path.join(tree, rel); t = open(p, encoding="utf-8").read()
    assert t.count(old) == 1, f"{rel}: 旧串命中 {t.count(old)} 次：{old[:70]!r}"
    open(p + ".new", "w", encoding="utf-8").write(t.replace(old, new)); os.replace(p + ".new", p)

CUT_ALGO_OLD = """        for file in facts.program_links:
            self.push_program(file, unit)
        for script in facts.inline_scripts:
            self.push_inline(script, unit)
        for file in facts.whole_modules:"""
CUT_ALGO_NEW = """        for file in facts.program_links:
            if unit.name == "HostLookup.query" and file.endswith("host-resources.py"):
                continue
            self.push_program(file, unit)
        for script in facts.inline_scripts:
            self.push_inline(script, unit)
        for file in facts.whole_modules:"""
CUT_SRC_OLD = ('process = self.process = subprocess.Popen([sys.executable, "-B", script, "--json", "--", *self.command], stdout=output, stderr=errors,\n'
               '                                                              stdin=subprocess.DEVNULL, env=environment)')
ML_OLD = """        if unit.kind != "module-body":
            self.push(module.module_body, unit, "module-body")
        if unit.kind == "class":"""
ML_NEW = """        if unit.kind != "module-body":
            self.push(module.module_body, unit, "module-body")
        elif module.inline is None:
            # 模块体一执行，模块级 import 与赋值跟着执行（不收 class 头）
            for sibling in module.units:
                if sibling.kind in KINDS_MODULE_LEVEL and not sibling.data:
                    self.push(sibling, unit, "module-level")
        if unit.kind == "class":"""
NARROW_NEW = """        if unit.kind != "module-body":
            self.push(module.module_body, unit, "module-body")
        elif module.inline is None and module.path in self.executing_modules:
            for sibling in module.units:
                if sibling.kind in NARROW_KINDS and not sibling.data:
                    self.push(sibling, unit, "narrow-module-level")
        if unit.kind == "import" and module.path in self.executing_modules:
            for target in facts.import_targets:
                if target.module.inline is None and target.module.path not in self.executing_modules:
                    self.executing_modules.add(target.module.path)
                    self.push(target.module.module_body, unit, "narrow-import")
        if unit.kind == "class":"""

cut_src = clone(base, "cut-src"); edit(cut_src, WRAP, CUT_SRC_OLD, "process = self.process = None")
cut_algo = clone(base, "cut-algo"); edit(cut_algo, LIB, CUT_ALGO_OLD, CUT_ALGO_NEW)
stub = clone(base, "stub-hr"); open(os.path.join(stub, "scripts/hosts/host-resources.py"), "w").write("# stub\n")
for name, src in (("modlevel", base), ("cut-modlevel", cut_algo), ("cut-modlevel-class", None)):
    if name == "cut-modlevel-class":
        clone(os.path.join(target, "cut-modlevel"), name); continue
    t = clone(src, name); edit(t, LIB, ML_OLD, ML_NEW)
    edit(t, LIB, "class ScriptClosure:\n", "KINDS_MODULE_LEVEL = os.environ.get('R3_ML_KINDS', 'import,assign').split(',')\n\n\nclass ScriptClosure:\n")
for name, src in (("cut-narrow", cut_algo), ("narrow", base)):
    t = clone(src, name); edit(t, LIB, ML_OLD, NARROW_NEW)
    edit(t, LIB, '            self.push(loaded.module_body, unit, "load")', '            self.executing_modules.add(loaded.path)\n            self.push(loaded.module_body, unit, "load")')
    edit(t, LIB, '        self.todo = []\n        # 内联脚本切出来的单元', '        self.todo = []\n        self.executing_modules = set()\n        # 内联脚本切出来的单元')
    edit(t, LIB, "class ScriptClosure:\n", "NARROW_KINDS = os.environ.get('R3_NARROW_KINDS', 'import').split(',')\n\n\nclass ScriptClosure:\n")
a = clone(base, "mut-a"); edit(a, ".claude/hooks/lib_long_run.py", "\nimport re\n", "\nimport re as re\n")
b = clone(base, "mut-b"); edit(b, "scripts/hosts/memory_peaks_lib.py", "\nimport subprocess\n", "\nimport subprocess as subprocess\n")
print("name=variants built=" + ",".join(sorted(os.listdir(target))))
