import subprocess
# 字典实参：往子进程环境里写，不是读
subprocess.run(["true"], env={"SAMPLE_DICT_THREADS": "4"})
