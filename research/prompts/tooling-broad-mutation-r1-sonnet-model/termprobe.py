# 同构的 mock：外层 bash 设 TERM 的 trap，前台等一个子进程（结构照 run-with-memory-cap.sh 第 800、812 行）；驱动照 broad-mutation.py 第 688–690 行发 TERM、等 3 秒。
import os, subprocess, time
work = "/tmp/claude-1000/tooling-broad-mutation-r1-sonnet"
os.chdir(work)
if os.path.exists("termprobe.out"):
    os.remove("termprobe.out")
p = subprocess.Popen(["bash", "/home/fy5090/code/singlefs/research/prompts/tooling-broad-mutation-r1-sonnet-model/termprobe.sh"])
time.sleep(1)
p.terminate()
t0 = time.time()
try:
    p.wait(timeout=3)
    print("wrapper exited within 3s after terminate:", p.returncode)
except subprocess.TimeoutExpired:
    print("wrapper still alive 3s after TERM (trap deferred)")
p.wait()
print("total wait %.1fs rc=%s" % (time.time() - t0, p.returncode))
print(open("termprobe.out").read().strip())
