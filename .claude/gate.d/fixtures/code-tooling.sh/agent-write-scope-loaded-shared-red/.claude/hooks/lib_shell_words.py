"""样本：被判仓里共用切词模块自己那一份，里面的 def 不算另写（agent-write-scope 格第 ⑬ 条按文件名排除它）；它 import 的 shared_text_tools 是几份脚本共用的库，不算它拆出的部分。"""
import shared_text_tools


def shell_tokens(text):
    return text.split()
