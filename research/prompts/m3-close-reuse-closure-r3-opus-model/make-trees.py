"""第三轮攻方：从一棵树（被判九份 sha256 对得上）造几份变体树。每份 cp -al 硬链原树，只把点名的文件换成改过的（写临时文件再改名，不动原树 inode）。
  cut          HostLookup.query 起 host-resources.py 的 Popen 换成起 `python3 -c pass`：按调用边切掉 HostLookup -> host-resources.py（丙第 3 件的后一半）
  ml           cut 之上，闭包算法照丙第 3 件字面：一份 python 模块进闭包时，它模块级的 import、assign 单元一律进闭包（不收 class 头）
  mlh          ml 之上另收模块级 class 头本身（import 时执行的就是类头：装饰器、基类、类体里不是 def 的语句），但不因此把类当成「被点名」、不连它的方法
               （类以别的方式被点名时照旧连方法）。这是我提的写法，被攻过零轮
  *-classbreak 在 host-dispatch.py 的 class SelftestWorld 类体里加一行模块载入时就抛的类属性（predicted_bytes exec_module 整份 host-dispatch.py 时执行）
  *-defbreak   在 memory_peaks_lib.py 的 def headroom（不在丙第 3 件字面的闭包里）加一个缺省值载入时就抛的参数（包装排队脚本 import 整份 memory_peaks_lib 时执行）
  *-assignbreak 在 memory_peaks_lib.py 的模块级赋值加同样会抛的一个（对照：丙第 3 件字面收得到的那一种）
用法：python3 make-trees.py <原树> <放变体的目录>"""
import os
import shutil
import subprocess
import sys

source, target = sys.argv[1], sys.argv[2]
WRAPPER = "scripts/hosts/run-with-memory-cap.sh"
UNITS = "scripts/gate/lib_function_units.py"
DISPATCH = "scripts/hosts/host-dispatch.py"
PEAKS = "scripts/hosts/memory_peaks_lib.py"
CUT = ('subprocess.Popen([sys.executable, "-B", script, "--json", "--", *self.command], stdout=output, stderr=errors,',
       'subprocess.Popen([sys.executable, "-c", "pass"], stdout=output, stderr=errors,')
ENTERED = "    def module_entered(self, module):\n        self.modules_seen.add(module.path)\n        self.closure.refusals.update(module.refusals)\n"
ML_TAIL = ("        for unit in module.units:   # 第三轮攻方：丙第 3 件字面——模块级 import、assign 一律进闭包\n"
           "            if unit.kind in (\"import\", \"assign\") and not unit.data:\n"
           "                self.push(unit, module.module_body, \"module-level\")\n")
MLH_TAIL = ("        for unit in module.units:   # 第三轮攻方：模块级 import、assign 与类头本身一律进闭包（类头不算被点名）\n"
            "            if unit.kind in (\"import\", \"assign\") and not unit.data:\n"
            "                self.push(unit, module.module_body, \"module-level\")\n"
            "            elif unit.kind == \"class\" and unit.container is None and not unit.data:\n"
            "                self.push(unit, module.module_body, \"module-level-class\")\n")
EDGES_OLD = "        if unit.kind == \"class\":\n            self.class_entered(unit)\n"
EDGES_NEW = ("        if unit.kind == \"class\" and getattr(self, \"current_rule\", None) != \"module-level-class\":\n"
             "            self.class_entered(unit)\n")
RUN_OLD = ("            elif not self.closure.add(unit, via, rule):\n                continue\n            if unit.data:\n")
RUN_NEW = ("            elif not self.closure.add(unit, via, rule):\n"
           "                if unit.kind == \"class\" and rule != \"module-level-class\" and (unit.file, unit.first_line) in getattr(self, \"import_time_heads\", {}):\n"
           "                    stored = self.import_time_heads.pop((unit.file, unit.first_line))   # 先当类头收进来、后来被点名：照旧连方法\n"
           "                    self.class_entered(stored)\n"
           "                    if stored is not unit:\n"
           "                        self.class_entered(unit)\n"
           "                continue\n"
           "            if unit.kind == \"class\" and rule == \"module-level-class\":\n"
           "                self.import_time_heads[(unit.file, unit.first_line)] = unit   # 只当类头收进来：python_edges 不连方法\n"
           "            self.current_rule = rule\n"
           "            if unit.data:\n")
RUN_HEAD_OLD = "    def run(self, entries):\n        for entry in entries:\n"
RUN_HEAD_NEW = "    def run(self, entries):\n        self.import_time_heads = {}\n        for entry in entries:\n"
PUSH_OLD = ("    def push(self, unit, via, rule):\n        if unit is not None and id(unit) not in self.closure.members:\n")
PUSH_NEW = ("    def push(self, unit, via, rule):\n"
            "        if unit is not None and unit.kind == \"class\" and rule != \"module-level-class\" and id(unit) in self.closure.members \\\n"
            "                and (unit.file, unit.first_line) in getattr(self, \"import_time_heads\", {}):\n"
            "            self.import_time_heads.pop((unit.file, unit.first_line))   # 先当类头收进来、后来被点名：照旧连方法\n"
            "            self.class_entered(unit)\n"
            "        if unit is not None and id(unit) not in self.closure.members:\n")
CLASS_OLD = "class SelftestWorld:\n"
CLASS_NEW = "class SelftestWorld:\n    ATTACK_IMPORT_TIME = int(\"第三轮攻方：类头在载入时就抛\")\n"
DEF_OLD = "def headroom(environment):\n"
DEF_NEW = "def headroom(environment, attack_import_time=int(\"第三轮攻方：缺省值在载入时就抛\")):\n"
ASSIGN_OLD = "\nimport subprocess\n"
ASSIGN_NEW = "\nimport subprocess\nATTACK_IMPORT_TIME = int(\"第三轮攻方：模块级赋值在载入时就抛\")\n"


def once(old, new):
    return lambda text: text.replace(old, new) if text.count(old) == 1 else None


def chain(*edits):
    def apply(text):
        for edit in edits:
            text = edit(text)
            if text is None:
                return None
        return text
    return apply


CUT_EDIT = (WRAPPER, once(*CUT))
ML_EDIT = (UNITS, once(ENTERED, ENTERED + ML_TAIL))
MLH_EDIT = (UNITS, chain(once(PUSH_OLD, PUSH_NEW), once(ENTERED, ENTERED + MLH_TAIL), once(EDGES_OLD, EDGES_NEW), once(RUN_OLD, RUN_NEW), once(RUN_HEAD_OLD, RUN_HEAD_NEW)))
CLASS_EDIT = (DISPATCH, once(CLASS_OLD, CLASS_NEW))
DEF_EDIT = (PEAKS, once(DEF_OLD, DEF_NEW))
ASSIGN_EDIT = (PEAKS, once(ASSIGN_OLD, ASSIGN_NEW))
VARIANTS = {
    "cut": [CUT_EDIT],
    "ml": [CUT_EDIT, ML_EDIT],
    "mlh": [CUT_EDIT, MLH_EDIT],
    "ml-classbreak": [CUT_EDIT, ML_EDIT, CLASS_EDIT],
    "ml-defbreak": [CUT_EDIT, ML_EDIT, DEF_EDIT],
    "ml-assignbreak": [CUT_EDIT, ML_EDIT, ASSIGN_EDIT],
    "mlh-classbreak": [CUT_EDIT, MLH_EDIT, CLASS_EDIT],
    "mlh-defbreak": [CUT_EDIT, MLH_EDIT, DEF_EDIT],
}
os.makedirs(target, exist_ok=True)
for name, edits in VARIANTS.items():
    tree = os.path.join(target, name)
    if os.path.exists(tree):
        shutil.rmtree(tree)
    subprocess.run(["cp", "-al", source, tree], check=True)
    for relative, edit in edits:
        path = os.path.join(tree, relative)
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        changed = edit(text)
        if changed is None or changed == text:
            sys.exit(f"{name}: {relative} 里要换的那一处没恰好命中一次")
        temporary = path + ".variant"
        with open(temporary, "w", encoding="utf-8") as handle:
            handle.write(changed)
        os.chmod(temporary, os.stat(path).st_mode)
        os.replace(temporary, path)
        print(f"variant={name} file={relative}")
