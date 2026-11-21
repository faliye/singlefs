"""样本判法模块 sample_rows.py 的 Python 拆法部分文件：入口代码行点名的是 sample_rows.py，这一份随它的族一起罩着。"""


def memory_bound_for_pool(cap, peak):
    return max(1, cap // peak)
