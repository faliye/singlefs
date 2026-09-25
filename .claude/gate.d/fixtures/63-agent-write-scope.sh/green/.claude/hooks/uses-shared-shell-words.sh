#!/usr/bin/env bash
# 样本：从共用切词模块导入、只写自己的判定（63 号第 ⑬ 条该绿）；调用 shell_words.shell_tokens 不算另写一份
python3 - "$(dirname "$0")" <<'PY'
import importlib.util, os, sys
spec = importlib.util.spec_from_file_location("lib_shell_words", os.path.join(sys.argv[1], "lib_shell_words.py"))
shell_words = importlib.util.module_from_spec(spec)
spec.loader.exec_module(shell_words)
def command_names(command):
    return [found.words[0] for found in shell_words.commands_at_command_position(command).commands]
PY
