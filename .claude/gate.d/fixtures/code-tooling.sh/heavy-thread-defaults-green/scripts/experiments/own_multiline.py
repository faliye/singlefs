"""样本：读和回退写在同一条跨行的语句里（括号续行），读的变量 capped.sh 不设，不点名。"""
import os
from heavy_threads import default_heavy_threads
workers = int(
    os.environ.get("SAMPLE_MULTILINE_WORKERS")
    or default_heavy_threads()
)
