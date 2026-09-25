#!/usr/bin/env bash
# 样本：自己手抄一份共用重型测试判定模块里的 classify_process，不从 lib_heavy_tests.py 导入（63 号第 ⑬ 条该红）
python3 - <<'PY'
def classify_process(argv, cwd):
    return []
PY
