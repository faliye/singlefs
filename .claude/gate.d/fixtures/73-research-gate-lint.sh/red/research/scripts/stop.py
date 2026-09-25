"""样本：python 里循环 os.kill 与 os.killpg（这个文件只拿来扫，不执行）。"""
import os
import signal
for process_id in []:
    os.kill(process_id, signal.SIGTERM)
os.killpg(os.getpgid(0), signal.SIGTERM)
