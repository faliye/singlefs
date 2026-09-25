#!/usr/bin/env bash
# 样本：从共用重型测试判定模块导入、只写自己的判定（63 号第 ⑬ 条该绿）；调用 heavy_tests.classify_process 不算另写一份，
# 自己的 selftest 入口与 load_sibling_module 这个导入样板登记在 NOT_SHARED_JUDGMENT 里，同名也不算
python3 - "$(dirname "$0")" <<'PY'
import importlib.util, os, sys
def load_sibling_module(module_name):
    spec = importlib.util.spec_from_file_location(module_name, os.path.join(sys.argv[1], module_name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module
heavy_tests = load_sibling_module("lib_heavy_tests")
def refused(argv, cwd):
    return bool(heavy_tests.classify_process(argv, cwd))
def selftest():
    return 0 if not refused(["ls"], "/") else 1
PY
