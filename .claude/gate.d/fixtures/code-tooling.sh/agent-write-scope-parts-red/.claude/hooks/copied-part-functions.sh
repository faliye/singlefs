#!/usr/bin/env bash
# 样本：自己手抄两份住在共用模块部分文件里的函数，不从入口导入（agent-write-scope 格第 ⑬ 条该红）：brace_expand 住在 lib_shell_words_core.py，classify 住在 lib_project_code_runs_launchers.py
python3 - <<'PY'
def brace_expand(word, mask):
    return [word]
def classify(words, directory):
    return None
PY
