#!/usr/bin/env python3
"""T1 / T2 探针：把命令文本喂给副本的 lib_heavy_tests.heavy_tests_in_command_text（只读判定，不起任何命令），
再把同一串参数喂给副本驱动的 parse_arguments 看驱动会走哪一支。用法：classify_probe.py <副本根> <当作当前目录的仓根>"""
import importlib.util, os, sys, io, contextlib

copy_root, repository = sys.argv[1], sys.argv[2]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


heavy = load("lib_heavy_tests", os.path.join(copy_root, ".claude", "hooks", "lib_heavy_tests.py"))
driver = load("broad_mutation_copy", os.path.join(copy_root, "drv", "bin", "broad-mutation.py"))
D = "research/scripts/broad-mutation.py"
many = " ".join(f"--item {n}" for n in range(1, 5092))

# (编号, 命令文本, 驱动参数（None = 不是参数层面的事）)
CASES = [
    ("A1 点名一条只跑第一段（对照）", f"python3 {D} --item 12 --skip-stage-two", ["--item", "12", "--skip-stage-two"]),
    ("A2 点名 5091 条只跑第一段", f"python3 {D} {many} --skip-stage-two", [w for n in range(1, 5092) for w in ("--item", str(n))] + ["--skip-stage-two"]),
    ("A3 A2 带前缀（驱动要它）", f"SINGLEFS_HEAVY_TESTS=user-request nice -n 19 python3 {D} {many} --skip-stage-two", None),
    ("A4 点名 + 只跑第一段 + 放大线程与内存", f"python3 {D} --item 1 --skip-stage-two --threads 64 --memory-cap 200G", ["--item", "1", "--skip-stage-two", "--threads", "64", "--memory-cap", "200G"]),
    ("A5 假步驱动写真库", f"BROAD_MUTATION_STEP_DRIVER=/bin/true python3 {D} --item 7 --skip-stage-two", None),
    ("A6 弄坏开关进真跑", f"BROAD_MUTATION_BREAK=interrupted-as-done python3 {D} --item 7 --skip-stage-two", None),
    ("B1 --summary 加 --shard", f"python3 {D} --summary --shard 0/2", ["--summary", "--shard", "0/2"]),
    ("B2 --sync-tree 加 --shard", f"python3 {D} --sync-tree --shard 0/2", ["--sync-tree", "--shard", "0/2"]),
    ("B3 --sync-tree 换提交", f"python3 {D} --sync-tree --commit HEAD~1", ["--sync-tree", "--commit", "HEAD~1"]),
    ("B4 --list-items 加 --from", f"python3 {D} --list-items --from 1", ["--list-items", "--from", "1"]),
    ("B5 -h 在后面", f"python3 {D} --shard 0/2 -h", None),
    ("B6 --summary 当 --home 的值", f"python3 {D} --home --summary", ["--home", "--summary"]),
    ("B7 -- 之后的 --summary", f"python3 {D} --shard 0/2 -- --summary", ["--shard", "0/2", "--", "--summary"]),
    ("B8 两段开关并用", f"python3 {D} --item 1 --skip-stage-two --stage-two-only", ["--item", "1", "--skip-stage-two", "--stage-two-only"]),
    ("B9 --item 加 --stage-two-only", f"python3 {D} --item 1 --stage-two-only", ["--item", "1", "--stage-two-only"]),
    ("B10 缩写 --skip（argparse 认前缀）", f"python3 {D} --item 1 --skip", ["--item", "1", "--skip"]),
    ("B11 缩写 --list", f"python3 {D} --list", ["--list"]),
    ("B12 缩写 --sum", f"python3 {D} --sum", ["--sum"]),
    ("B13 --item=1 贴着写", f"python3 {D} --item=1 --skip-stage-two", ["--item=1", "--skip-stage-two"]),
    ("C1 bash -c", f"bash -c 'python3 {D} --shard 0/2'", None),
    ("C2 env", f"env X=1 python3 {D}", None),
    ("C3 nice + 绝对路径", f"nice -n 19 python3 {repository}/{D}", None),
    ("C4 cd 进去相对路径", f"cd research/scripts && python3 ./broad-mutation.py", None),
    ("C5 直接执行（有执行位）", f"./{D} --shard 1/2", None),
    ("C6 python3 -u", f"python3 -u {D}", None),
    ("C7 python3 -W ignore（带值的解释器选项）", f"python3 -W ignore {D} --shard 0/2", None),
    ("C8 python3 -X dev", f"python3 -X dev {D}", None),
    ("C9 python3 -c runpy", f"python3 -c \"import runpy; runpy.run_path('{D}', run_name='__main__')\" --shard 0/2", None),
    ("C10 python3 -c exec", f"python3 -c \"__file__='{D}'; exec(open(__file__).read())\"", None),
    ("C11 python3 - < 脚本", f"python3 - --shard 0/2 < {D}", None),
    ("C12 符号链接", f"ln -s broad-mutation.py research/scripts/bm.py && python3 research/scripts/bm.py", None),
    ("C13 仓内改名拷贝", f"cp {D} research/scripts/bm2.py && python3 research/scripts/bm2.py", None),
    ("C14 变量间接", f"S={D}; python3 $S", None),
    ("C15 python3.12 全路径", f"/usr/bin/python3.12 {D}", None),
    ("C16 多余斜杠与 ..", f"python3 research//scripts/../scripts/broad-mutation.py", None),
    ("C17 capped.sh 包一层", f"bash research/scripts/capped.sh 31 python3 {D}", None),
    ("C18 run-with-memory-cap 包一层", f"bash research/scripts/run-with-memory-cap.sh 24G python3 {D}", None),
    ("C19 xargs", f"echo --shard 0/2 | xargs python3 {D}", None),
    ("C20 setsid / stdbuf", f"stdbuf -oL python3 {D}", None),
    ("C21 command / exec", f"exec python3 {D}", None),
    ("C22 前一条放过、后一条整趟", f"python3 {D} --summary; python3 {D}", None),
    ("C23 pypy3 / uv run", f"uv run {D}", None),
    ("D1 T2 py_compile 语法检查", f"python3 -m py_compile {D}", None),
    ("D2 T2 grep 读它", f"grep -n main {D}", None),
    ("D3 T2 sha256sum", f"sha256sum {D}", None),
    ("D4 T2 草稿目录的仓副本里起（路径尾是 research/scripts）", "python3 /tmp/claude-1000/some-leg/copy/research/scripts/broad-mutation.py --home /tmp/claude-1000/some-leg/home", None),
    ("D5 T2 主 agent 全量", f"SINGLEFS_HEAVY_TESTS=user-request nice -n 19 python3 {D} --threads 31", None),
    ("D6 T2 --selftest", f"python3 {D} --selftest", None),
    ("D7 T2 --selftest 混运行参数", f"python3 {D} --selftest --shard 0/2", ["--selftest", "--shard", "0/2"]),
    ("D8 T2 --stage-two-only 全量", f"python3 {D} --stage-two-only", None),
]


def driver_branch(argv):
    sink = io.StringIO()
    try:
        with contextlib.redirect_stderr(sink), contextlib.redirect_stdout(sink):
            a = driver.parse_arguments(argv)
    except SystemExit as stop:
        return f"argparse 退 {stop.code}"
    if a.selftest:
        return "走 --selftest"
    if a.list_items or a.summary:
        return "只列项/汇总后返回"
    if a.sync_tree:
        return f"只 sync_tree(commit={a.commit}) 后返回"
    return (f"真跑 run_items：item={len(a.item)} 条 from={a.start_from} package={a.package} shard={a.shard} "
            f"skip_two={a.skip_stage_two} two_only={a.stage_two_only} threads={a.threads} memory_cap={a.memory_cap}")


for label, text, argv in CASES:
    found = heavy.heavy_tests_in_command_text(text, repository)
    kinds = ",".join(test.kind for test in found) or "None"
    shown = text if len(text) < 200 else text[:90] + f" …（共 {len(text)} 字）… " + text[-40:]
    print(f"{label}\n    命令: {shown}\n    分类: {kinds}" + (f"\n    驱动: {driver_branch(argv)}" if argv is not None else ""))
