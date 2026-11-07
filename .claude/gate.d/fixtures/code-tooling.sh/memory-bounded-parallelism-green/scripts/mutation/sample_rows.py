"""样本判法模块：文档串里提到 memory_bound_for 不算，下面的函数定义才算。"""


def memory_bound_for(cap, peak):
    return max(1, cap // peak)
