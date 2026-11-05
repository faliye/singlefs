"""样本：库名只当名字；没给判定库就退出（模块文档串，不判）。"""
import os
def library_name():
    """没有判定库时交空名就退出，不照跑（函数文档串，不判）。"""
    return "singlefs-sample-verdicts"
common = os.environ.get("SAMPLE_COMMON", ".")  # 判定库没处放就退出（注释，不判）
print(os.path.join(common, "runs"), library_name())
