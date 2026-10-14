import os
EXPERIMENT = 999
threads = int(os.environ.get(f"E{EXPERIMENT}_THREADS", "8"))
