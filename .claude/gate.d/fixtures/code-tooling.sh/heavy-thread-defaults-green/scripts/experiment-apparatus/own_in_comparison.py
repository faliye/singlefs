"""样本：线程变量名只出现在字符串比较里，不是从环境读；引号里的名字出现就算读，所以点了名，排除表按锚串放行。"""
registry_table = "AGENTS_TABLE = {'a', 'b'}"
if "SAMPLE_COMPARED_THREADS" in registry_table:
    print("字符串比较，不读环境变量")
