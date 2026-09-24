# 绿样本：调用前缀收进一个带着选项的列表
import subprocess
GIT = ["git", "-c", "core.quotepath=false"]
names = subprocess.run(GIT + ["ls-files", "--others", "--exclude-standard"], capture_output=True, text=True).stdout
