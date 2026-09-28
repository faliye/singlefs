# 红样本：参数列表换了行，续行里取 merge-base
import subprocess
base = subprocess.run(["git", "-c", "core.quotepath=false",
                       "merge-base", "HEAD", "@{upstream}"], capture_output=True, text=True).stdout
