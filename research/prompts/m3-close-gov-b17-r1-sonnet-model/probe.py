#!/usr/bin/env python3
"""G4 旁证两样：①xargs / find -exec 起重型脚本在 lib_heavy_tests.py 这一层判不判得出——对 replay.sh 与已有的两类（崩溃点重放、崩溃放量）同判，
证明是对每一类重型都在的老口子（lib_heavy_tests.py 文件头登记的「罩不住的」），不是这次加 replay.sh 才有的；
②真跑 replay.sh 的三种写法（--list-items、--force --list-items、--list-items --force）：退出码、行数、三份输出是否逐字节相同、是否留下本轮输出目录。
用法：python3 -I probe.py <仓根>"""
import glob, importlib.util, os, subprocess, sys

sys.dont_write_bytecode = True
repo = os.path.abspath(sys.argv[1])
hooks = os.path.join(repo, ".claude", "hooks")
sys.path.insert(0, hooks)
spec = importlib.util.spec_from_file_location("lib_heavy_tests", os.path.join(hooks, "lib_heavy_tests.py"))
lib = importlib.util.module_from_spec(spec)
spec.loader.exec_module(lib)
for script in ("scripts/experiment-apparatus/replay.sh", "scripts/crash/crash-point-replay.sh", "scripts/crash/crash-amplification.sh"):
    for shape in ("echo x | xargs bash {}", "find . -maxdepth 0 -exec bash {} x ';'"):
        text = shape.format(script)
        found = lib.heavy_tests_in_command_text(text, repo)
        print(f"gap\t{'light' if not found else 'heavy'}\t{text}")
before = set(glob.glob("/tmp/singlefs-replay-*"))
outputs = []
for arguments in (["--list-items"], ["--force", "--list-items"], ["--list-items", "--force"]):
    done = subprocess.run(["nice", "-n", "19", "bash", "scripts/experiment-apparatus/replay.sh", *arguments], cwd=repo, capture_output=True, text=True)
    outputs.append(done.stdout)
    print(f"list\texit={done.returncode}\tstdout_lines={len(done.stdout.splitlines())}\tstderr_bytes={len(done.stderr)}\t{' '.join(arguments)}")
print(f"list\tthree_outputs_identical={len(set(outputs)) == 1}\tnew_output_directories={len(set(glob.glob('/tmp/singlefs-replay-*')) - before)}")
