import importlib.util, os, sys
hooks = "/home/fy5090/code/singlefs/.claude/hooks"
def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path); m = importlib.util.module_from_spec(spec)
    sys.modules[name] = m; spec.loader.exec_module(m); return m
lib = load("lib_heavy_tests", os.path.join(hooks, "lib_heavy_tests.py"))
root = "/home/fy5090/code/singlefs"
S = "research/scripts/broad-mutation.py"
cases = [
 ("整趟", f"python3 {S}", root),
 ("--summary+--shard", f"python3 {S} --summary --shard 0/2", root),
 ("--sync-tree+--item 1（无 skip）", f"python3 {S} --sync-tree --item 1", root),
 ("--list-items+--from", f"python3 {S} --list-items --from 1", root),
 ("--item 单条+skip", f"python3 {S} --item 12 --skip-stage-two", root),
 ("--item 三条+skip", f"python3 {S} --item 1 --item 2 --item 3 --skip-stage-two", root),
 ("--item 两个+skip+stage-two-only", f"python3 {S} --item 1 --skip-stage-two --stage-two-only", root),
 ("--item=1 写法+skip", f"python3 {S} --item=1 --skip-stage-two", root),
 ("--item 加 --package+skip", f"python3 {S} --item 1 --package singlefs-core --skip-stage-two", root),
 ("bash -c", f"bash -c 'python3 {S}'", root),
 ("env", f"env X=1 python3 {S}", root),
 ("nice", f"nice -n 19 python3 {S}", root),
 ("./ 直接执行", f"./{S}", root),
 ("cd 后 ./", f"cd research/scripts && ./broad-mutation.py", root),
 ("cd 后 python3 相对", f"cd research/scripts && python3 broad-mutation.py", root),
 ("python3 -m runpy", f"python3 -m runpy {S}", root),
 ("python3 -W ignore", f"python3 -W ignore {S}", root),
 ("python3 -X dev", f"python3 -X dev {S}", root),
 ("python3 -u", f"python3 -u {S}", root),
 ("python3 -c runpy", f"python3 -c 'import runpy; runpy.run_path(\"{S}\", run_name=\"__main__\")'", root),
 ("python3 - < 脚本", f"python3 - < {S}", root),
 ("符号链接 bm.py", f"python3 /tmp/claude-1000/tooling-broad-mutation-r1-sonnet/elsewhere/bm.py", root),
 ("符号链接 bm.py 带 --item+skip", f"python3 /tmp/claude-1000/tooling-broad-mutation-r1-sonnet/elsewhere/bm.py --shard 0/2", root),
 ("xargs", f"echo | xargs python3 {S}", root),
 ("find -exec", f"find . -maxdepth 0 -exec python3 {S} \;", root),
 ("eval", f"eval 'python3 {S}'", root),
 ("仓外同名", f"python3 /tmp/x/broad-mutation.py", root),
 ("仓外 cwd 相对路径", f"python3 research/scripts/broad-mutation.py", "/tmp"),
 ("sh -c 里 ./", f"sh -c './{S}'", root),
 ("timeout", f"timeout 10 python3 {S}", root),
 ("python3.12", f"python3.12 {S}", root),
 ("python3 -B -O", f"python3 -B -O {S}", root),
 ("时间 /usr/bin/time", f"/usr/bin/time -v python3 {S}", root),
 ("systemd-run", f"systemd-run --user --scope python3 {S}", root),
 ("--help 与整趟", f"python3 {S} -h --shard 0/2", root),
 ("--selftest", f"python3 {S} --selftest", root),
 ("--selftest+--item", f"python3 {S} --selftest --item 1", root),
]
for label, text, cwd in cases:
    try:
        r = lib.heavy_tests_in_command_text(text, cwd)
        print(f"{label}\t{text}\t=> {[t.kind for t in r]}")
    except Exception as e:
        print(f"{label}\t{text}\t=> EXC {e!r}")
