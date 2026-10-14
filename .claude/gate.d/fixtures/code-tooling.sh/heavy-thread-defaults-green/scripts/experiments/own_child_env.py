"""样本：只往子进程的环境里写线程变量，不是从环境读，不点名。"""
import subprocess
environment = {}
environment["SAMPLE_CHILD_THREADS"] = "4"
subprocess.run(["true"], env=environment, check=True)
