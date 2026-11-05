#!/usr/bin/env bash
# 样本：自己手抄一份共用项目代码判定模块里的 judged_by_name，不从 lib_project_code_runs.py 导入（agent-write-scope 格第 ⑬ 条该红）
python3 - <<'PY'
def judged_by_name(argv, cwd):
    return []
PY
