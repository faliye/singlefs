"""样本：文档串里的 os.cpu_count() // 2 与 capped.sh 16 不点名。"""
import os
import heavy_threads
SAMPLE = "bash scripts/hosts/capped.sh 16 cargo build"
threads = heavy_threads.default_heavy_threads()  # 注释里的 nproc 不点名
print(f"不按 nproc 取：{threads}")
cap = os.environ.get("SINGLEFS_THREAD_CAP")
timeout = int(os.environ.get("GATE_SAMPLE_TIMEOUT_SECONDS") or 600)  # 超时的回退值：不是线程数，线程变量名在上一行也不点名
