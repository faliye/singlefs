"""样本：线程变量名只出现在字符串比较、没人用的常量里，也算读（认不出读的是哪一个），必须点名；确实不读环境变量的进排除表（见 heavy-thread-defaults-green）。"""
registry_table = "AGENTS_TABLE = {'a', 'b'}"
if "SAMPLE_COMPARED_THREADS" in registry_table:
    print("字符串比较，不读环境变量")
UNUSED_NAME = "SAMPLE_UNUSED_THREADS"
