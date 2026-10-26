"""W3：第二台若跑「同一份 host-resources.py、由参数选机器」，它在那台上要读到哪些仓内文件。
① 在整棵树上照 HostLookup 的调法跑一次 host-resources.py --json -- true（多机开关关着，只查这一台），装 audit hook 记下 python 层打开的仓内文件与起的子进程里点名的仓内脚本；
② 只拷 scripts/hosts/ 下的 .py 与 .sh 进一棵空树，同一条命令再跑一次，报退出码与 stderr 末行。
用法：w3-needed-files.py <树> <草稿目录>"""
import os
import shutil
import subprocess
import sys

root, scratch = os.path.realpath(sys.argv[1]), sys.argv[2]
os.makedirs(scratch, exist_ok=True)
configuration = os.path.join(scratch, "multi-host-off.env")
with open(configuration, "w", encoding="utf-8") as handle:
    handle.write("PEER_SSH_HOST=selftest-peer-host-name\nPEER_REPOSITORY_DIRECTORY=/selftest/peer/home\nPEER_CARGO_BIN_DIRECTORY=/selftest/peer/bin\n"
                 "PEER_MEMORY_CAP=40G\nQUIESCE_STOP_COMMAND=\nQUIESCE_STOPPED_CHECK_COMMAND=\nQUIESCE_START_COMMAND=\nQUIESCE_STARTED_CHECK_COMMAND=\nENABLE_ACROSS_MACHINES=0\n")
environment = dict(os.environ, SINGLEFS_MULTI_HOST_CONFIG=configuration, HOST_RESOURCES_SAMPLE_SECONDS="0.2",
                   RUN_WITH_MEMORY_CAP_PEAKS=os.path.join(scratch, "peaks.tsv"), RUN_WITH_MEMORY_CAP_STATE_DIR=os.path.join(scratch, "state"))
for name in ("RUN_WITH_MEMORY_CAP_HOST_LOOKUP", "SINGLEFS_HEAVY_TESTS"):
    environment.pop(name, None)
tracer = r'''
import os, runpy, sys
root = os.path.realpath(sys.argv[1]); log = open(sys.argv[2], "w", encoding="utf-8"); script = sys.argv[3]
def hook(event, arguments):
    if event == "open" and isinstance(arguments[0], str):
        path = os.path.realpath(arguments[0])
        if path.startswith(root + os.sep):
            log.write("open\t" + os.path.relpath(path, root) + "\n"); log.flush()
    elif event == "subprocess.Popen":
        for word in arguments[1] or []:
            word = str(word)
            if os.path.exists(word) and os.path.realpath(word).startswith(root + os.sep):
                log.write("program\t" + os.path.relpath(os.path.realpath(word), root) + "\n"); log.flush()
sys.addaudithook(hook)
sys.argv = [script] + sys.argv[4:]
runpy.run_path(script, run_name="__main__")
'''
log_path = os.path.join(scratch, "opened.tsv")
script = os.path.join(root, "scripts/hosts/host-resources.py")
completed = subprocess.run([sys.executable, "-B", "-c", tracer, root, log_path, script, "--json", "--", "true"], cwd=root, env=environment,
                           capture_output=True, text=True, stdin=subprocess.DEVNULL)
seen = sorted({line.rstrip("\n") for line in open(log_path, encoding="utf-8")})
files = sorted({line.split("\t", 1)[1] for line in seen if "/__pycache__/" not in line and os.path.isfile(os.path.join(root, line.split("\t", 1)[1]))})
# 只记得到 python 层的打开与子进程参数里点名的脚本：bash 里 source 进来的不在这里（下界）
outside = [path for path in files if not path.startswith("scripts/hosts/")]
print(f"name=needed_files full_tree_rc={completed.returncode} repo_files={len(files)} outside_scripts_hosts={len(outside)}")
for path in outside:
    print(f"  outside={path}")
partial = os.path.join(scratch, "only-scripts-hosts")
shutil.rmtree(partial, ignore_errors=True)
os.makedirs(os.path.join(partial, "scripts", "hosts"))
for name in sorted(os.listdir(os.path.join(root, "scripts", "hosts"))):
    if name.endswith((".py", ".sh")):
        shutil.copy2(os.path.join(root, "scripts", "hosts", name), os.path.join(partial, "scripts", "hosts", name))
only = subprocess.run([sys.executable, "-B", os.path.join(partial, "scripts/hosts/host-resources.py"), "--json", "--", "true"], cwd=partial, env=environment,
                      capture_output=True, text=True, stdin=subprocess.DEVNULL)
tail = only.stderr.strip().splitlines()[-1] if only.stderr.strip() else ""
print(f"name=only_scripts_hosts rc={only.returncode} stdout_is_json={only.stdout.strip().startswith('{')} last_stderr_line={tail[:160]}")
