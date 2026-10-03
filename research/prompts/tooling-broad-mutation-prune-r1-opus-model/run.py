#!/usr/bin/env python3
"""攻方腿的模型：照 P1–P3 判「可复用」，再在第二版上真跑，看结论翻不翻。
用法：python3 run.py <工作目录> <llvm 工具目录>   （整条经 run-with-memory-cap.sh 与 capped.sh 起；只编、只跑本目录 ws/ 下的模型 crate）
每个场景：第一版树 = ws/；第二版树 = ws/ 只换 scenarios.tsv 里那一个文件。两版各跑：没改过的全部测试、施加变异后的全部测试、
逐条测试单独起进程量覆盖（-C instrument-coverage，一条测试一个 profraw）。复用判定照正文第三节 P2「抓到的」那一条：
被改文件、T 所在的测试文件、T 执行到的全部源文件（读法甲取第一版的覆盖、读法乙取第二版的覆盖）、P3 被改文件引用到的常量的定义文件，都没变 ⇒ 可复用。
"""
import hashlib, json, os, re, shutil, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
WORK, LLVM = sys.argv[1], sys.argv[2]
PACKAGES = ["harness", "tier"]
MARK_LINE = re.compile(r"^test (\S+) \.\.\. (ok|FAILED)$")


def sh(command, cwd, env=None, check=True, merged=False):
    result = subprocess.run(command, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT if merged else subprocess.PIPE, text=True)
    if check and result.returncode not in (0, 101):
        sys.stderr.write(result.stdout + (result.stderr or ""))
        raise SystemExit(f"命令退 {result.returncode}：{' '.join(command)}")
    return result


def tree_digest(root):
    out = {}
    for base, dirs, files in os.walk(root):
        dirs[:] = [d for d in dirs if d != "target"]
        for name in files:
            if name == "Cargo.lock" or name.endswith(".profraw"):
                continue
            path = os.path.join(base, name)
            out[os.path.relpath(path, root)] = hashlib.sha256(open(path, "rb").read()).hexdigest()
    return out


def make_tree(name, replace=None):
    root = os.path.join(WORK, name)
    if os.path.exists(root):
        shutil.rmtree(root)
    shutil.copytree(os.path.join(HERE, "ws"), root)
    if replace:
        relative, source = replace
        shutil.copyfile(os.path.join(HERE, source), os.path.join(root, relative))
    return root


def run_tests(root, target):
    """跑两个包的全部测试，交 {测试全名: ok/FAILED}。"""
    env = dict(os.environ, CARGO_TARGET_DIR=target)
    verdicts = {}
    for package in PACKAGES:
        result = sh(["cargo", "test", "--offline", "--manifest-path", f"{package}/Cargo.toml", "--tests", "--no-fail-fast"], root, env, merged=True)
        binary = None
        for line in result.stdout.splitlines():
            found = re.search(r"Running tests/(\S+)\.rs", line)
            if found:
                binary = found.group(1)
            found = MARK_LINE.match(line.strip())
            if found and binary:
                verdicts[f"{package}/tests/{binary}.rs::{found.group(1)}"] = found.group(2)
    return verdicts


def apply_mutation(root, relative, old, new):
    path = os.path.join(root, relative)
    text = open(path, encoding="utf-8").read()
    if text.count(old) != 1:
        raise SystemExit(f"{relative} 里 {old!r} 出现 {text.count(old)} 次")
    open(path, "w", encoding="utf-8").write(text.replace(old, new))


def coverage_per_test(root, target):
    """逐条测试单独起进程量覆盖，交 {测试全名: 执行到的源文件集合（相对树根；llvm-cov 原样给的名字另记）}。"""
    env = dict(os.environ, CARGO_TARGET_DIR=target, RUSTFLAGS="-C instrument-coverage")
    executed, raw_names = {}, {}
    profiles = os.path.join(target, "profiles")
    os.makedirs(profiles, exist_ok=True)
    for package in PACKAGES:
        result = sh(["cargo", "test", "--offline", "--manifest-path", f"{package}/Cargo.toml", "--tests", "--no-run", "--message-format=json"], root, env)
        for line in result.stdout.splitlines():
            if not line.startswith("{"):
                continue
            message = json.loads(line)
            if message.get("reason") != "compiler-artifact" or not message.get("executable") or message["target"]["kind"] != ["test"]:
                continue
            binary, source = message["executable"], message["target"]["src_path"]
            listing = sh([binary, "--list", "--format", "terse"], root, dict(os.environ, LLVM_PROFILE_FILE=os.path.join(profiles, "listing-%p.profraw"))).stdout
            for test in [l.split(": ")[0] for l in listing.splitlines() if l.endswith(": test")]:
                key = f"{os.path.relpath(source, root)}::{test}"
                stem = re.sub(r"[^A-Za-z0-9]", "_", key)
                for leftover in os.listdir(profiles):
                    os.remove(os.path.join(profiles, leftover))
                run_env = dict(os.environ, LLVM_PROFILE_FILE=os.path.join(profiles, stem + "-%p.profraw"))
                sh([binary, "--exact", test], os.path.dirname(source), run_env)
                raws = [os.path.join(profiles, f) for f in os.listdir(profiles)]
                data = os.path.join(target, stem + ".profdata")
                sh([os.path.join(LLVM, "llvm-profdata"), "merge", "-sparse", *raws, "-o", data], root)
                export = sh([os.path.join(LLVM, "llvm-cov"), "export", binary, f"-instr-profile={data}", "-summary-only"], root).stdout
                files, names = set(), []
                for entry in json.loads(export)["data"][0]["files"]:
                    if entry["summary"]["lines"]["covered"] > 0:
                        names.append(entry["filename"])
                        files.add(os.path.relpath(os.path.normpath(entry["filename"]), root))
                executed[key], raw_names[key] = files, sorted(names)
    return executed, raw_names


def main():
    os.makedirs(WORK, exist_ok=True)
    rows = list(open(os.path.join(HERE, "scenarios.tsv"), encoding="utf-8"))[1:]
    v1 = make_tree("v1")
    base_v1 = run_tests(v1, os.path.join(WORK, "target-plain"))
    cov_v1, raw_v1 = coverage_per_test(v1, os.path.join(WORK, "target-cov"))
    print("BASELINE v1", "green" if all(v == "ok" for v in base_v1.values()) else "RED", len(base_v1), "tests")
    for key in sorted(raw_v1):
        print("COVERAGE v1", key, "files=" + ",".join(sorted(cov_v1[key])), "llvm-cov-names=" + ",".join(raw_v1[key]))
    for row in rows:
        scenario, changed, v2_source, mutated, old, new, const_files = row.rstrip("\n").split("\t")
        v1m = make_tree(f"{scenario}-v1m")
        apply_mutation(v1m, mutated, old, new)
        failing_v1 = sorted(k for k, v in run_tests(v1m, os.path.join(WORK, "target-plain")).items() if v != "ok")
        print(f"SCENARIO {scenario} v1+M failing={failing_v1 or 'none'} => {'caught' if failing_v1 else 'missed'}")
        if not changed:
            mutated_line_hit = sorted(k for k, files in cov_v1.items() if mutated in files)
            print(f"SCENARIO {scenario} tests-whose-coverage-includes-the-mutated-file={mutated_line_hit or 'none'}")
            continue
        v2 = make_tree(f"{scenario}-v2", (changed, v2_source))
        base_v2 = run_tests(v2, os.path.join(WORK, "target-plain"))
        v2m = make_tree(f"{scenario}-v2m", (changed, v2_source))
        apply_mutation(v2m, mutated, old, new)
        failing_v2 = sorted(k for k, v in run_tests(v2m, os.path.join(WORK, "target-plain")).items() if v != "ok")
        cov_v2, _ = coverage_per_test(v2, os.path.join(WORK, f"target-cov-{scenario}"))
        d1, d2 = tree_digest(v1), tree_digest(v2)
        changed_files = sorted(f for f in set(d1) | set(d2) if d1.get(f) != d2.get(f))
        test = failing_v1[0] if failing_v1 else None
        test_file = test.split("::")[0] if test else "(none)"
        constants = set(filter(None, const_files.split(",")))
        for reading, coverage in (("A-first-version-coverage", cov_v1), ("B-second-version-coverage", cov_v2)):
            associated = {mutated, test_file} | coverage.get(test, set()) | constants
            reuse = not (set(changed_files) & associated)
            print(f"SCENARIO {scenario} reading={reading} T={test} associated={','.join(sorted(associated))} changed={','.join(changed_files)} "
                  f"P1-P3={'reuse caught' if reuse else 'rerun'}")
        print(f"SCENARIO {scenario} v2-baseline={'green' if all(v == 'ok' for v in base_v2.values()) else 'RED'} v2+M failing={failing_v2 or 'none'} "
              f"=> rerun-verdict={'caught' if failing_v2 else 'missed'}")


main()
