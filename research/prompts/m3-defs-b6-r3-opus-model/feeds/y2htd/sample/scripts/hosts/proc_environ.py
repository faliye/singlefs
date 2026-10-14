# 攻方样本（Y2）：从 /proc/self/environ 读自己的线程变量（名字拼在 bytes 前缀里）
raw = open("/proc/self/environ", "rb").read().split(b"\0")
threads = int(next((item.split(b"=", 1)[1] for item in raw if item.startswith(b"SAMPLE_PROC_THREADS=")), b"8"))
