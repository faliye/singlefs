# 照 59 号的判法在草稿副本里跑这 6 行（与第 41 行的候选写法）：先不施加跑基线，再逐行施加、跑点名的测试与它所在的整个测试二进制、还原。
# 副本：/tmp/claude-1000/m2-anchor-fix/copy（rsync -a --exclude target --exclude .git），编译产物只放 copy-target。
import json, os, re, subprocess, sys, time
draft = "/tmp/claude-1000/m2-anchor-fix"
copy = os.path.join(draft, "copy")
logs = os.path.join(draft, "logs")
os.makedirs(logs, exist_ok=True)
environment = dict(os.environ)
environment["CARGO_TARGET_DIR"] = os.path.join(draft, "copy-target")
rows = {int(key): value for key, value in json.load(open(os.path.join(draft, "new-rows.json"), encoding="utf-8")).items()}
alternative = json.load(open(os.path.join(draft, "alternative-41.json"), encoding="utf-8"))
def unescape(text): return text.replace("\\n", "\n")
def cargo_test(label, args):
    started = time.time()
    run = subprocess.run(["nice", "-n", "19", "cargo", "test", "--offline"] + args, cwd=copy, env=environment, capture_output=True, text=True)
    output = run.stdout + run.stderr
    open(os.path.join(logs, label + ".log"), "w", encoding="utf-8").write(output)
    return run.returncode, output, time.time() - started
def whole_binary_args(args):
    # "-p singlefs-harness --test X -- filter" → 去掉 "--" 之后的过滤串，跑整个测试二进制
    words = args.split()
    return words[:words.index("--")] if "--" in words else words
def test_lines(output):
    return re.findall(r"^test \S+ \.\.\. \S+$", output, re.M)
def failed_tests(output):
    return re.findall(r"^test (\S+) \.\.\. FAILED$", output, re.M)
def panic_lines(output, test_name):
    block = re.search(r"^---- (\S+::)?" + re.escape(test_name) + r" stdout ----\n(.*?)(?=^---- |^failures:$|\Z)", output, re.M | re.S)
    return block.group(2).strip().splitlines()[:12] if block else []
phase = sys.argv[1]
if phase == "baseline":
    binaries = sorted({" ".join(whole_binary_args(fields[4])) for fields in rows.values()})
    for binary in binaries:
        label = "baseline-whole-" + binary.split()[-1]
        code, output, seconds = cargo_test(label, binary.split())
        print(f"== 基线整个二进制 `cargo test --offline {binary}`：退出码 {code}，{seconds:.0f} 秒")
        for line in test_lines(output):
            print("   " + line)
        if code != 0 and not test_lines(output):
            print("\n".join(output.splitlines()[-15:]))
    for line_number, fields in sorted(rows.items()):
        label = f"baseline-row-{line_number}"
        code, output, seconds = cargo_test(label, fields[4].split())
        print(f"== 基线第 {line_number} 行 `cargo test --offline {fields[4]}`：退出码 {code}，{seconds:.0f} 秒")
        for line in test_lines(output):
            print("   " + line)
elif phase == "mutate":
    targets = [(str(line_number), fields) for line_number, fields in sorted(rows.items())]
    if len(sys.argv) > 2:
        wanted = sys.argv[2].split(",")
        targets = [(label, fields) for label, fields in targets if label in wanted]
        if "41-alternative" in wanted:
            targets.append(("41-alternative", alternative))
    for label, fields in targets:
        path = os.path.join(copy, fields[1])
        pristine = open(path, encoding="utf-8").read()
        old, new = unescape(fields[2]), unescape(fields[3])
        assert pristine.count(old) == 1, (label, pristine.count(old))
        open(path, "w", encoding="utf-8").write(pristine.replace(old, new))
        try:
            code, output, seconds = cargo_test(f"mutated-row-{label}-filtered", fields[4].split())
            red = code != 0 and re.search(r"^test (\S+::)?" + re.escape(fields[5]) + r" \.\.\. FAILED$", output, re.M)
            print(f"== 第 {label} 行施加替换，`cargo test --offline {fields[4]}`：退出码 {code}，{seconds:.0f} 秒，59 号判法：{'红' if red else '没红'}")
            for line in test_lines(output):
                print("   " + line)
            if not test_lines(output):
                print("\n".join(output.splitlines()[-20:]))
            for line in panic_lines(output, fields[5]):
                print("   | " + line)
            whole = whole_binary_args(fields[4])
            code, output, seconds = cargo_test(f"mutated-row-{label}-whole", whole)
            print(f"   整个二进制 `cargo test --offline {' '.join(whole)}`：退出码 {code}，{seconds:.0f} 秒；红的：{failed_tests(output)}")
            total = re.findall(r"^test result: .*$", output, re.M)
            print("   " + " / ".join(total))
        finally:
            open(path, "w", encoding="utf-8").write(pristine)
            os.utime(path, None)
        assert open(path, encoding="utf-8").read() == pristine
