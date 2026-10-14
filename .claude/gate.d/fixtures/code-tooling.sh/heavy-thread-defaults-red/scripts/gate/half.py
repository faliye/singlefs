"""样本：文档串里的 os.cpu_count() 与 capped.sh 16 不点名。"""
import os
workers = max(1, min(16, (os.cpu_count() or 4) // 2))
by_processor = processor_count // 2
print("nproc 只在出路里")
cap = os.environ.get("SINGLEFS_THREAD_CAP")
fallback_workers = int(cap or 8)
