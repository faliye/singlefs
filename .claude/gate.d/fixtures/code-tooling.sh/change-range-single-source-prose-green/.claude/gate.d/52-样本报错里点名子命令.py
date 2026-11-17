# 绿样本：报错里点名失败的是哪条子命令，那一行不跑 git；行尾带豁免标记与理由
import subprocess

GIT = ["git", "-c", "core.quotepath=false"]


class ListingProblem(Exception):
    pass


listed = subprocess.run(GIT + ["ls-files", "-z"], capture_output=True)
if listed.returncode != 0:
    raise ListingProblem("索引", "ls-files", listed.returncode)  # change-range:exempt 报错里点名失败的是哪条子命令，这一行不跑 git
