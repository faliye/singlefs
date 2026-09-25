#!/usr/bin/env bash
# 样本：自己手抄一份共用切词模块里的 shell_tokens，不从 lib_shell_words.py 导入（63 号第 ⑬ 条该红）
python3 - <<'PY'
def shell_tokens(text):
    return text.split()
PY
