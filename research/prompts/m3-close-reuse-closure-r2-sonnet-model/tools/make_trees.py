#!/usr/bin/env python3
"""W2 的试验树：从冻结副本拷出的树 <源树> 出发，在 <目标目录> 下各建一棵改过的树。这些改动只用来量，不是实现。
用法：make_trees.py <源树> <目标目录>；建好的树：
  cut-lookup        只把 HostLookup.query 起 host-resources.py 的那一条边切掉（Popen 换成 None）——丙第 3 件的字面做法
  load-edge         闭包算法：按文件路径载入（spec_from_file_location 一类 load 规则）的模块，它的模块体一执行，它的每条模块级 import 与引来的模块跟着收
  cut-load-edge     load-edge 加 cut-lookup
  cut-lookup-syn    cut-lookup 加：.claude/hooks/lib_long_run.py 末尾加一行语法错
  cut-load-edge-syn cut-load-edge 加同一行语法错
  edge-decl         闭包算法：源文件里 `# closure-excluded-edge: <调用方单元> -> <被调文件>` 声明过的调用边不收；声明没被用上的拒算；包装里加一条声明
每一处旧串必须恰好命中一次。"""
import os
import shutil
import subprocess
import sys

source, target = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
LIB = "scripts/gate/lib_function_units.py"
WRAPPER = "scripts/hosts/run-with-memory-cap.sh"
SYNTAX = ".claude/hooks/lib_long_run.py"


def copy(name, parent=None):
    destination = os.path.join(target, name)
    if os.path.exists(destination):
        shutil.rmtree(destination)
    subprocess.run(["rsync", "-a", (parent or source).rstrip("/") + "/", destination + "/"], check=True)
    return destination


def edit(tree, relative, old, new):
    path = os.path.join(tree, relative)
    text = open(path, encoding="utf-8").read()
    assert text.count(old) == 1, f"{relative}：旧串不是恰好一处：{old[:60]!r}"
    with open(path + ".new", "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new))
    os.replace(path + ".new", path)


def append(tree, relative, text):
    with open(os.path.join(tree, relative), "a", encoding="utf-8") as handle:
        handle.write(text)


CUT_OLD = ('process = self.process = subprocess.Popen([sys.executable, "-B", script, "--json", "--", *self.command], stdout=output, stderr=errors,\n'
           '                                                              stdin=subprocess.DEVNULL, env=environment)')


def cut(tree):
    edit(tree, WRAPPER, CUT_OLD, "process = self.process = None")


def load_edge(tree):
    edit(tree, LIB, '''        if unit.kind != "module-body":
            self.push(module.module_body, unit, "module-body")
        if unit.kind == "class":''', '''        if unit.kind != "module-body":
            self.push(module.module_body, unit, "module-body")
        elif module.path in self.executing_modules:
            # 按文件路径载入的模块：模块体一执行，它的每一条模块级 import 跟着执行
            for sibling in module.units:
                if sibling.kind == "import" and not sibling.data:
                    self.push(sibling, unit, "load-import")
        if unit.kind == "import" and module.path in self.executing_modules:
            # 这些 import 引来的模块同样执行它们的模块体与模块级 import
            for target in facts.import_targets:
                if target.module.inline is None and target.module.path not in self.executing_modules:
                    self.executing_modules.add(target.module.path)
                    self.push(target.module.module_body, unit, "load-import")
        if unit.kind == "class":''')
    edit(tree, LIB, '            self.push(loaded.module_body, unit, "load")', '            self.executing_modules.add(loaded.path)\n            self.push(loaded.module_body, unit, "load")')
    edit(tree, LIB, '        self.todo = []\n        # 内联脚本切出来的单元', '        self.todo = []\n        self.executing_modules = set()\n        # 内联脚本切出来的单元')


def edge_declaration(tree):
    edit(tree, LIB, '''        for file in facts.program_links:
            self.push_program(file, unit)
        for script in facts.inline_scripts:''', '''        for file in facts.program_links:
            if (unit.name, file) in self.declared_exclusions(unit.file):
                self.used_exclusions.add((unit.file, unit.name, file))
                continue
            self.push_program(file, unit)
        for script in facts.inline_scripts:''')
    edit(tree, LIB, '    def push_inline(self, script, via):', '''    EDGE_DECLARATION = re.compile(r"^\\s*#\\s*closure-excluded-edge:\\s*(\\S+)\\s*->\\s*([^\\s：:]+)")

    def declared_exclusions(self, file):
        if file not in self.exclusion_cache:
            found = {}
            try:
                with open(os.path.join(self.root, file), encoding="utf-8") as handle:
                    for number, line in enumerate(handle, 1):
                        match = self.EDGE_DECLARATION.search(line)
                        if match:
                            found[(match.group(1), match.group(2))] = (number, line.strip())
            except (OSError, UnicodeDecodeError):
                pass
            self.exclusion_cache[file] = found
        return self.exclusion_cache[file]

    def stale_exclusions(self):
        for file in sorted({unit.file for unit in self.closure.order}):
            for (caller, callee), (number, line) in sorted(self.declared_exclusions(file).items()):
                if (file, caller, callee) not in self.used_exclusions:
                    self.closure.refusals.add(Refusal("closure-exclusion-stale", file, number, snippet_of(line)))

    def push_inline(self, script, via):''')
    edit(tree, LIB, '        self.todo = []\n        # 内联脚本切出来的单元', '        self.todo = []\n        self.exclusion_cache, self.used_exclusions = {}, set()\n        # 内联脚本切出来的单元')
    edit(tree, LIB, '                self.shell_edges(unit)\n        return self.closure', '                self.shell_edges(unit)\n        self.stale_exclusions()\n        return self.closure')
    edit(tree, WRAPPER, '        # 输出落进临时文件：到点停它时不被它留下的子进程（ssh）握着的管道卡住\n',
         '        # closure-excluded-edge: HostLookup.query -> scripts/hosts/host-resources.py：只打提示，出错只等于没有提示（试验声明）\n'
         '        # 输出落进临时文件：到点停它时不被它留下的子进程（ssh）握着的管道卡住\n')


os.makedirs(target, exist_ok=True)
tree = copy("cut-lookup"); cut(tree)
tree = copy("load-edge"); load_edge(tree)
tree = copy("cut-load-edge"); load_edge(tree); cut(tree)
tree = copy("cut-lookup-syn", os.path.join(target, "cut-lookup")); append(tree, SYNTAX, "def broken(:\n")
tree = copy("cut-load-edge-syn", os.path.join(target, "cut-load-edge")); append(tree, SYNTAX, "def broken(:\n")
tree = copy("edge-decl"); edge_declaration(tree)
print("name=make_trees built=" + ",".join(sorted(name for name in os.listdir(target))))
