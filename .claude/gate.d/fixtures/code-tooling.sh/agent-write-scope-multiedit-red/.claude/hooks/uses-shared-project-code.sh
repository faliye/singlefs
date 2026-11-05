#!/usr/bin/env bash
# 样本：从共用项目代码判定模块导入、只写自己的判定（agent-write-scope 格第 ⑬ 条该绿）；调用 project_code_runs.judged_by_name 不算另写一份，
# 自己的 selftest 入口与 load_sibling_module 这个导入样板登记在 NOT_SHARED_JUDGMENT 里，同名也不算
python3 - "$(dirname "$0")" <<'PY'
import importlib.util, os, sys
def load_sibling_module(module_name):
    spec = importlib.util.spec_from_file_location(module_name, os.path.join(sys.argv[1], module_name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module
project_code_runs = load_sibling_module("lib_project_code_runs")
def refused(argv, cwd):
    return bool(project_code_runs.judged_by_name(argv, cwd))
def selftest():
    return 0 if not refused(["ls"], "/") else 1
PY
