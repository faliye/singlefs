#!/usr/bin/env python3
"""攻方腿 m3-close-gov-b17-r2 H3：判法闭包里一处「getattr(对象, 变量)」（lib_function_units.py 的拒算形态 getattr-variable）
写在 NOT_JUDGING_FILES 那份里与写在别的判法文件里，crates-mutation-rows.py judging-units 各怎么判。只算 python 闭包，不起 cargo。
用法：python3 refusal_probe.py --base <仓副本根> --work <新的草稿目录>
--work 下建两份拷贝：lfu-dynamic（改 scripts/gate/lib_function_units.py 的 RustFile.row）与 units-dynamic（改 scripts/mutation/lib_mutation_units.py 的 UnitKeys.executed_units）。"""
import argparse, os, shutil, subprocess, sys
parser = argparse.ArgumentParser()
parser.add_argument("--base", required=True); parser.add_argument("--work", required=True)
arguments = parser.parse_args()
base, work = os.path.realpath(arguments.base), os.path.realpath(arguments.work)
os.makedirs(work, exist_ok=False)
EDITS = {
    "lfu-dynamic": ("scripts/gate/lib_function_units.py",
                    '    def row(self, line):\n        return snippet_of(',
                    '    def row(self, line):\n        _hook = getattr(self, os.environ.get("ROW_HOOK", "path"))\n        return snippet_of('),
    "units-dynamic": ("scripts/mutation/lib_mutation_units.py",
                      '        executed, unplaced = {}, 0\n',
                      '        executed, unplaced = {}, 0\n        _hook = getattr(self, os.environ.get("ROW_HOOK", "root"))\n'),
}
def judging_units(root, extra):
    environment = {key: value for key, value in os.environ.items() if not key.endswith("_BREAK")}
    environment.update(extra)
    done = subprocess.run([sys.executable, "-B", os.path.join(root, "scripts/mutation/crates-mutation-rows.py"), "judging-units", root],
                          capture_output=True, text=True, env=environment)
    tail = (done.stdout.strip().splitlines() or [""])[0] if done.returncode == 0 else (done.stderr.strip().splitlines() or [""])[0]
    return done.returncode, tail.replace(work, "<work>")[:200]
print("case=base default exit=%s out=%s" % judging_units(base, {}))
for name, (path, old, new) in EDITS.items():
    root = os.path.join(work, name)
    shutil.copytree(base, root, symlinks=True)
    target = os.path.join(root, path)
    text = open(target, encoding="utf-8").read()
    assert text.count(old) >= 1, (name, text.count(old))   # 取头一处（lib_function_units.py 里头一处是 RustFile.row）
    with open(target + ".tmp", "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new, 1))
    os.replace(target + ".tmp", target)
    print(f"case={name} default exit=%s out=%s" % judging_units(root, {}))
    print(f"case={name} judging-files-included exit=%s out=%s" % judging_units(root, {"CRATES_MUTATION_ROWS_BREAK": "judging-files-included"}))
