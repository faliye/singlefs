#!/usr/bin/env python3
"""攻方腿模型：把已跑过 setup.sh 的 library-version 样本根里的库目录改成「今天的驱动 version_key 算出来的指纹」。
样本 setup.sh 写的是 sha256(源码指纹、换行、工具链)（判法版本进指纹之前的公式）；今天的驱动是 sha256(源码指纹、换行、工具链、judge <判法版本> 一行)。
用法：fix-green.py <tree> <样本根>   只改样本根里 .broad-sample/home/（拷贝里的），打新旧指纹。"""
import importlib.util, json, os, shutil, sys
tree, root = sys.argv[1:3]
specification = importlib.util.spec_from_file_location("driver_for_model", os.path.join(tree, "research/scripts/broad-mutation.py"))
driver = importlib.util.module_from_spec(specification)
specification.loader.exec_module(driver)
home = os.path.join(root, ".broad-sample", "home")
source = json.load(open(os.path.join(home, "tree.source"), encoding="utf-8"))
old = source["fingerprint"]
toolchain = open(os.path.join(home, "versions", old, "toolchain.txt"), encoding="utf-8").read()
new = driver.version_key(home, source["source_fingerprint"], toolchain)
shutil.move(os.path.join(home, "versions", old), os.path.join(home, "versions", new))
source["fingerprint"] = new
json.dump(source, open(os.path.join(home, "tree.source"), "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print(f"old={old[:16]} new={new[:16]} judge={driver.JUDGE_RULES_VERSION}")
