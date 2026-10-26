"""从一棵树（被判脚本在里面、sha256 对得上）造几份变体树：每份硬链原树（cp -al），只把点名的文件换成改过的（先写临时文件再改名，不动原树的 inode）。
  cut               HostLookup.query 起 host-resources.py 那一处换成起字面内联 python（"pass"）：按调用边把 HostLookup -> host-resources.py 那一条切掉，别的不动（丙第 3 件的前一半）
  cut-mpl-import    cut 之上：memory_peaks_lib.py 的模块级「import subprocess」多带一个不存在的模块（包装自己 import 这一份时就失败）
  full-mpl-import   原树之上同一处改动（对照：切边之前键变不变）
  cut-long-run-unit cut 之上：.claude/hooks/lib_long_run.py 的模块级赋值 CANONICAL_HOSTS_SENTENCE 末尾多一个句号（丙第 3 件自证那一格：lib_long_run.py 的一个单元改了）
  hr-int-cap        host-resources.py 的 --json 里这条命令的预测内存改报给人看的写法（「8.0 GiB」，一次重构的样子：host-resources.py 在丙之下不进键）
  cut-hr-int-cap    cut 之上同 hr-int-cap 那一处改动（算键用：丙把那条边排掉之后，host-resources.py 这一改键变不变）
  cut-ml            cut 之上：闭包算法改一处（我提的改法，被攻过零轮）——一份 python 模块有单元进闭包时，它模块级的 import、assign 单元一律进闭包（import 时都执行）
  cut-mlc           同 cut-ml，另把模块级 class 头也一律连上（看类头连带的方法会不会把拒算带回来）
  cut-ml-long-run   cut-ml 之上同 cut-long-run-unit 那一处；cut-ml-mpl 同 cut-mpl-import 那一处；cut-ml-hr 同 hr-int-cap 那一处
  item1-literal     hr-int-cap 之上：HostLookup 照丙第 1 件的字面补 try——起线程、前台建临时文件到读完输出那一段；别处不动
  item1-catchall    hr-int-cap 之上：HostLookup.run 整个包住（begin 起的那一趟与 finish 前台那一趟都过它）
用法：python3 make-variants.py <原树> <放变体的目录>"""
import os
import shutil
import subprocess
import sys

source, target = sys.argv[1], sys.argv[2]
WRAPPER = "scripts/hosts/run-with-memory-cap.sh"
CUT = ('process = self.process = subprocess.Popen([sys.executable, "-B", script, "--json", "--", *self.command], stdout=output, stderr=errors,',
       'process = self.process = subprocess.Popen([sys.executable, "-c", "pass"], stdout=output, stderr=errors,')
MPL = ("scripts/hosts/memory_peaks_lib.py", "\nimport subprocess\n", "\nimport subprocess, attack_w2_missing_module\n")
LONG_RUN = (".claude/hooks/lib_long_run.py", "不现算、不写常量\"\n", "不现算、不写常量。\"\n")
HR_INT = ("scripts/hosts/host-resources.py",
          '"prediction": {"memory": prediction.memory_predicted,',
          '"prediction": {"memory": peaks.human(prediction.memory_predicted),')
BEGIN_OLD = ('        self.thread = threading.Thread(target=self.run, args=("queued", local), daemon=True)\n'
             '        self.thread.start()\n')
BEGIN_NEW = ('        try:\n'
             '            self.thread = threading.Thread(target=self.run, args=("queued", local), daemon=True)\n'
             '            self.thread.start()\n'
             '        except Exception as error:\n'
             '            self.thread = None\n'
             '            say(f"run-with-memory-cap: 没查到别的机器：起不了线程（{type(error).__name__}）")\n')
QUERY_HEAD = '        with tempfile.TemporaryFile(mode="w+", encoding="utf-8") as output, tempfile.TemporaryFile(mode="w+", encoding="utf-8") as errors:\n'
QUERY_TAIL = '            stdout, stderr = output.read(), errors.read().strip()\n'
RUN_OLD = ('    def run(self, phase, local):\n'
           '        problem, peer = self.query()\n')
RUN_NEW = ('    def run(self, phase, local):\n'
           '        try:\n'
           '            self.run_unguarded(phase, local)\n'
           '        except Exception as error:\n'
           '            say(f"run-with-memory-cap: 没查到别的机器：查的那一支出了异常（{type(error).__name__}）")\n'
           '\n'
           '    def run_unguarded(self, phase, local):\n'
           '        problem, peer = self.query()\n')


ML = ("scripts/gate/lib_function_units.py",
      "    def module_entered(self, module):\n        self.modules_seen.add(module.path)\n        self.closure.refusals.update(module.refusals)\n",
      "    def module_entered(self, module):\n        self.modules_seen.add(module.path)\n        self.closure.refusals.update(module.refusals)\n"
      "        for unit in module.units:   # 攻方腿的改法：模块级会在 import 时执行的单元一律进闭包\n"
      "            if unit.kind in (\"import\", \"assign\") and not unit.data:\n"
      "                self.push(unit, module.module_body, \"module-level\")\n")
CUT_EDIT = (WRAPPER, lambda text: text.replace(*CUT) if text.count(CUT[0]) == 1 else None)
ML_EDIT = (ML[0], lambda text: text.replace(ML[1], ML[2]) if text.count(ML[1]) == 1 else None)
LONG_RUN_EDIT = (LONG_RUN[0], lambda text: text.replace(LONG_RUN[1], LONG_RUN[2]) if text.count(LONG_RUN[1]) == 1 else None)
MPL_EDIT = (MPL[0], lambda text: text.replace(MPL[1], MPL[2]) if text.count(MPL[1]) == 1 else None)
HR_EDIT = (HR_INT[0], lambda text: text.replace(HR_INT[1], HR_INT[2]) if text.count(HR_INT[1]) == 1 else None)


def literal_item1(text):
    """丙第 1 件的字面：起线程那两行、前台建临时文件到读完输出那一段各包一层 try；HostLookup 别处不动。"""
    assert text.count(BEGIN_OLD) == 1 and text.count(QUERY_HEAD) == 1 and text.count(QUERY_TAIL) == 1
    text = text.replace(BEGIN_OLD, BEGIN_NEW)
    head, rest = text.split(QUERY_HEAD)
    body, tail = rest.split(QUERY_TAIL)
    block = QUERY_HEAD + body + QUERY_TAIL
    indented = "".join("    " + line if line.strip() else line for line in block.splitlines(keepends=True))
    guarded = ("        try:\n" + indented +
               "        except Exception as error:\n"
               "            return f\"查的那一支建临时文件或读输出出了异常（{type(error).__name__}）\", None\n")
    return head + guarded + tail


def catchall_item1(text):
    assert text.count(RUN_OLD) == 1
    return text.replace(RUN_OLD, RUN_NEW)


VARIANTS = {
    "cut": [(WRAPPER, lambda text: text.replace(*CUT) if text.count(CUT[0]) == 1 else None)],
    "cut-mpl-import": [(WRAPPER, lambda text: text.replace(*CUT) if text.count(CUT[0]) == 1 else None),
                       (MPL[0], lambda text: text.replace(MPL[1], MPL[2]) if text.count(MPL[1]) == 1 else None)],
    "full-mpl-import": [(MPL[0], lambda text: text.replace(MPL[1], MPL[2]) if text.count(MPL[1]) == 1 else None)],
    "cut-long-run-unit": [(WRAPPER, lambda text: text.replace(*CUT) if text.count(CUT[0]) == 1 else None),
                          (LONG_RUN[0], lambda text: text.replace(LONG_RUN[1], LONG_RUN[2]) if text.count(LONG_RUN[1]) == 1 else None)],
    "hr-int-cap": [(HR_INT[0], lambda text: text.replace(HR_INT[1], HR_INT[2]) if text.count(HR_INT[1]) == 1 else None)],
    "cut-hr-int-cap": [(WRAPPER, lambda text: text.replace(*CUT) if text.count(CUT[0]) == 1 else None),
                       (HR_INT[0], lambda text: text.replace(HR_INT[1], HR_INT[2]) if text.count(HR_INT[1]) == 1 else None)],
    "cut-ml": [CUT_EDIT, ML_EDIT],
    "cut-mlc": [CUT_EDIT, (ML[0], lambda text: text.replace(ML[1], ML[2].replace('(\"import\", \"assign\")', '(\"import\", \"assign\", \"class\")')) if text.count(ML[1]) == 1 else None)],
    "cut-ml-long-run": [CUT_EDIT, ML_EDIT, LONG_RUN_EDIT],
    "cut-ml-mpl": [CUT_EDIT, ML_EDIT, MPL_EDIT],
    "cut-ml-hr": [CUT_EDIT, ML_EDIT, HR_EDIT],
    "item1-literal": [(HR_INT[0], lambda text: text.replace(HR_INT[1], HR_INT[2]) if text.count(HR_INT[1]) == 1 else None),
                      (WRAPPER, literal_item1)],
    "item1-catchall": [(HR_INT[0], lambda text: text.replace(HR_INT[1], HR_INT[2]) if text.count(HR_INT[1]) == 1 else None),
                       (WRAPPER, catchall_item1)],
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
