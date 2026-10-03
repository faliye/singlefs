#!/usr/bin/env python3
"""L1：把 harness 档的「一条用例一个场景」判法（research/scripts/crash-case-check.py 的 one_scenario_problems）原样用在 checker 档 tests/*.rs 上，
数它会判红几条。只读：只调判法函数，不起任何测试。"""
import glob, importlib.util, os, subprocess, sys
root = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip()
os.chdir(root)
sys.argv = ["crash-case-check.py"]
spec = importlib.util.spec_from_file_location("crash_case_check", "research/scripts/crash-case-check.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
tier = sorted(glob.glob(os.path.join(root, "crates", "singlefs-checker-tier", "tests", "*.rs")))
module.harness_test_paths = lambda _root: tier
checked, problems = module.one_scenario_problems(root)
print("checker-tier tests/*.rs 里查了", checked, "条用例，判红", len(problems), "条")
for problem in problems: print("  ", str(problem)[:200])
