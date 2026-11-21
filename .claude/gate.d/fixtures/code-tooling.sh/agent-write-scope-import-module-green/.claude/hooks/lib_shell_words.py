"""样本：被判仓里共用切词模块自己那一份，里面的 def 不算另写（agent-write-scope 格第 ⑬ 条按文件名排除它）；它按字面量参数的 importlib.import_module 载入同目录的部分文件 lib_shell_words_core.py。"""
import importlib

brace_expand = importlib.import_module("lib_shell_words_core").brace_expand


def shell_tokens(text):
    return text.split()
