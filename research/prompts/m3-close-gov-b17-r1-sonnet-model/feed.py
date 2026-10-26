#!/usr/bin/env python3
"""G4：把一批命令文本喂给 .claude/hooks/lib_heavy_tests.py 的 heavy_tests_in_command_text（包装剥掉后逐条 classify），
打每条的判结果；期望列由规则字面推出（lib_heavy_tests.py 文件头「实验复跑入口」一段：只有摘掉 --force 之后恰好一个 --list-items 的不算，别的都按重型算）。
用法：python3 -I feed.py <仓根>   只读：不起 cargo、不写仓。"""
import hashlib, importlib.util, os, sys

sys.dont_write_bytecode = True
repo = os.path.abspath(sys.argv[1])
hooks = os.path.join(repo, ".claude", "hooks")
sys.path.insert(0, hooks)
spec = importlib.util.spec_from_file_location("lib_heavy_tests", os.path.join(hooks, "lib_heavy_tests.py"))
lib = importlib.util.module_from_spec(spec)
spec.loader.exec_module(lib)

R = "scripts/experiment-apparatus/replay.sh"
# (编号, 命令文本, 当前目录相对仓根, 期望: light=放行 / heavy=拒 / note=规则字面没有期望，只记实际)
CASES = [
    ("a01", f"bash {R} --list-items", ".", "light"),
    ("a02", f"bash {R} --force --list-items", ".", "light"),
    ("a03", f"bash {R} --list-items --force", ".", "light"),
    ("a04", f"bash scripts/hosts/capped.sh 31 bash {R} --list-items", ".", "light"),
    ("a05", f"bash scripts/hosts/run-with-memory-cap.sh 8G bash {R} --list-items", ".", "light"),
    ("a06", f"nice -n 19 bash {R} --list-items", ".", "light"),
    ("a07", f"bash -c 'bash {R} --list-items'", ".", "light"),
    ("a08", f"env REPLAY_MEMORY_CAP=1G bash {R} --list-items", ".", "light"),
    ("a09", "bash replay.sh --list-items", "scripts/experiment-apparatus", "light"),
    ("a10", f"bash {R} --list-items | head -3", ".", "light"),
    ("a11", f"/usr/bin/time -v bash {R} --list-items", ".", "light"),
    ("a12", f"SINGLEFS_HEAVY_TESTS=commit bash {R} --list-items", ".", "light"),
    ("a13", f"cd scripts/experiment-apparatus && bash ./replay.sh --list-items", ".", "light"),
    ("b01", f"bash {R}", ".", "heavy"),
    ("b02", f"bash {R} E142", ".", "heavy"),
    ("b03", f"bash {R} --list-items E142", ".", "heavy"),
    ("b04", f"bash {R} --admission-only", ".", "heavy"),
    ("b05", f"bash {R} --full", ".", "heavy"),
    ("b06", f"bash {R} --list-items --full", ".", "heavy"),
    ("b07", f"bash {R} --list-items --list-items", ".", "heavy"),
    ("b08", f"bash {R} --list-items; bash {R} E142", ".", "heavy"),
    ("b09", f"bash {R} --list-items && bash {R}", ".", "heavy"),
    ("b10", f"bash scripts/hosts/capped.sh 31 bash {R} E142", ".", "heavy"),
    ("b11", f"bash scripts/hosts/run-with-memory-cap.sh 8G bash {R}", ".", "heavy"),
    ("b12", f"bash -c 'bash {R} E142'", ".", "heavy"),
    ("b13", f"X=E142; bash {R} $X", ".", "heavy"),
    ("b14", f"args=(--list-items E142); bash {R} \"${{args[@]}}\"", ".", "heavy"),
    ("b15", f"bash {R} --list-items=E142", ".", "heavy"),
    ("b16", f"bash {R} --list-items --selftest", ".", "heavy"),
    ("b17", f"bash {R} -- --list-items", ".", "heavy"),
    ("b18", f"source {R} E142", ".", "heavy"),
    ("b19", f"sh {R} E142", ".", "heavy"),
    ("b20", f"./{R} E142", ".", "heavy"),
    ("b21", f"exec bash {R} E142", ".", "heavy"),
    ("b22", f"echo E142 | xargs bash {R}", ".", "heavy"),
    ("b23", f"find . -maxdepth 0 -exec bash {R} E142 \;", ".", "heavy"),
    ("b24", f"cp {R} /tmp/claude-1000/r.sh; bash /tmp/claude-1000/r.sh E142", ".", "heavy"),
    ("b25", f"bash {R} --force", ".", "heavy"),
    ("b26", f"bash {R} --list-items --force E142", ".", "heavy"),
    ("c01", f"bash /tmp/claude-1000/copy/{R} --list-items", ".", "note"),
    ("c02", "bash /tmp/claude-1000/copy/replay.sh --list-items", ".", "note"),
    ("c03", "bash scripts/experiments/replay.sh --list-items", ".", "note"),
    ("c04", f"bash {R} --selftest", ".", "note"),
    ("c05", f"REPLAY_BREAK=guard-off bash {R} --list-items", ".", "note"),
    ("c06", "bash replay.sh --list-items", "scripts", "note"),
    ("c07", f"bash scripts/crash/../experiment-apparatus/replay.sh --list-items", ".", "note"),
]

print(f"lib_heavy_tests.py sha256={hashlib.sha256(open(os.path.join(hooks, 'lib_heavy_tests.py'), 'rb').read()).hexdigest()}", file=sys.stderr)   # 别的会话在改这份库：sha 不进比对的输出
print(f"EXPERIMENT_REPLAY_SCRIPT={lib.EXPERIMENT_REPLAY_SCRIPT!r} KNOWN_SCRIPT_LOCATIONS[replay.sh]={lib.KNOWN_SCRIPT_LOCATIONS.get('replay.sh')!r}")
bad = 0
for number, text, relative, expect in CASES:
    directory = os.path.normpath(os.path.join(repo, relative))
    found = lib.heavy_tests_in_command_text(text, directory)
    actual = "light" if not found else "heavy:" + ",".join(sorted({test.kind for test in found}))
    verdict = "-" if expect == "note" else ("same" if actual.split(":")[0] == expect else "DIFF")
    bad += verdict == "DIFF"
    print(f"{number}\t{actual}\texpect={expect}\t{verdict}\tcwd={relative}\t{text}")
print(f"cases={len(CASES)} diff={bad}")
