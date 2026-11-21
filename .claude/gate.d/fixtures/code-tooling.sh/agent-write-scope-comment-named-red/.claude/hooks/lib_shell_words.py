"""样本：被判仓里共用切词模块自己那一份，里面的 def 不算另写（agent-write-scope 格第 ⑬ 条按文件名排除它）；
文档串里写的 import 行不算载入同目录的 lib_shell_words_shadow.py：
    import lib_shell_words_shadow   # 别这样写
"""
# 别的钩子留下的 "lib_shell_words_shadow.py" 是影子，不归这一份；注释里点名它也不算载入。


def shell_tokens(text):
    return text.split()
