import os
# 样本：以 crash-amplification- 起头的运行目录、expanduser("~") 拼出来的数据根本身（两处都该点名）
RUNS = os.path.expanduser("~/.local/share/singlefs/crash-amplification-two-hosts/runs")
ROOT = os.path.join(os.path.expanduser("~"), ".local", "share", "singlefs")
print(ROOT, RUNS)
