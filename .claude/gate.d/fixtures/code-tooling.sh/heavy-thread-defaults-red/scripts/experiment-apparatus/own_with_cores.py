"""样本：自己的线程变量没设时取核数：核数那一种照旧点名，自己的线程变量这一种加在同一行。"""
import os
workers = int(os.environ.get("SAMPLE_CORES_THREADS") or os.cpu_count())
