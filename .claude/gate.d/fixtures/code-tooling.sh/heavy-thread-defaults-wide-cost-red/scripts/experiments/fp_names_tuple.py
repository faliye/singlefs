# 名字元组：列着名字给别的函数核对，不读环境
KNOWN = ("SAMPLE_TUPLE_THREADS", "SAMPLE_TUPLE_JOBS")
def check(name):
    return name in KNOWN
