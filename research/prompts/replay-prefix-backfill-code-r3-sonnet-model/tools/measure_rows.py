#!/usr/bin/env python3
"""必红世界表若干行的独立复量：把表里这几行的原文换成替换文，在 crate 的一份拷贝里直接起 Verus，数证明错条数、取落点，与表里第五、六段并排打出。
与门禁 formal-verification 不共用代码：判法只照表头文字（verus-must-red-worlds.tsv 第 9 行起）自己写一份。

用法：measure_rows.py --repo <仓根> --crate <crate 目录，含 src、Cargo.toml、verus-must-red-worlds.tsv> --work <新的草稿目录> --threads <n>
                      [--verus-root <Verus 安装目录>] [--baseline] [--names-file <文件，每行一个变异名>]
      measure_rows.py --selftest
Verus 安装目录：--verus-root，再 SINGLEFS_VERUS_ROOT，再 <仓根>/.claude/hooks/lib_work_root.py 的 shared_data_path("formal-tools") 加登记表 release 行第五列里 formal-tools/ 之后那一段。
起法：cwd 是 crate 拷贝，参数 --crate-type=lib --edition=<Cargo.toml> --crate-name <包名，横线换下划线> --num-threads <n> --output-json --rlimit <登记表 rlimit 行> src/lib.rs。
证明错：stderr 与 stdout 里以 error 起头的行，除去含 "Resource limit (rlimit) exceeded" 的与 "aborting due to" 起头的。
落点：每条证明错取它之后第一处「--> 文件:行:列」，在改坏之后的文件文本里往上找最近一行 fn 的开头，取 fn 名；去重成集合。
输出：每行一条 ROW 行（name、表里第五第六段、实测条数与落点、是否相同、秒数），ROW 之后缩进列出每条证明错；末尾 SUMMARY 行。
退出码：0 每行落点都与第六段相同（条数不同只报）；1 有落点不同或原文没恰好命中一次或 Verus 没出判定；2 用法错。
"""
import argparse, importlib.util, json, os, re, shutil, signal, subprocess, sys, tempfile, time

HEADING = re.compile(r"^\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?(?:(?:open|closed|broadcast|proof|spec(?:\([a-z]+\))?|exec|axiom|const|unsafe)\s+)*fn\s+(\w+)")
LOCATION = re.compile(r"^\s*-->\s+(\S+?):(\d+):(\d+)")
RLIMIT_TEXT = "Resource limit (rlimit) exceeded"


def read_rows(table_path):
    rows = {}
    for number, line in enumerate(open(table_path, encoding="utf-8"), 1):
        if line.startswith("#") or not line.strip():
            continue
        fields = line.rstrip("\n").split("\t")
        rows[fields[0]] = (number, fields)
    return rows


def enclosing_function(text_rows, line):
    for index in range(min(line, len(text_rows)) - 1, -1, -1):
        found = HEADING.match(text_rows[index])
        if found:
            return found.group(1)
    return None


def proof_errors(diagnostics):
    """[(message, path, line)]：以 error 起头的行配上它之后第一处 --> 位置。"""
    found, pending = [], None
    for text in diagnostics.split("\n"):
        if text.startswith("error"):
            message = text.split(":", 1)[1].strip() if ":" in text else text
            if RLIMIT_TEXT in message or message.startswith("aborting due to"):
                pending = None
                continue
            pending = [message, None, 0]
            found.append(pending)
            continue
        place = LOCATION.match(text)
        if place and pending is not None and pending[1] is None:
            pending[1], pending[2] = place.group(1), int(place.group(2))
    return [(m, p, n) for m, p, n in found]


def landing_of(errors, texts):
    names = set()
    for message, path, line in errors:
        if path is None:
            names.add("(无位置)")
            continue
        rows = texts.get(path)
        if rows is None:
            names.add("(函数外)")
            continue
        names.add(enclosing_function(rows, line) or "(函数外)")
    return sorted(names)


def start_verus(binary, arguments, cwd):
    process = subprocess.Popen([binary, *arguments], cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
    try:
        out, err = process.communicate(timeout=7200)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGTERM)
        out, err = process.communicate()
        err += "\nerror: aborting due to the driver timeout"
    return process.returncode, out, err


def verus_binary(repo, override):
    root = override or os.environ.get("SINGLEFS_VERUS_ROOT")
    if not root:
        pins = open(os.path.join(repo, ".claude/gate.d/verus-pins.tsv"), encoding="utf-8").read().split("\n")
        release = next(line for line in pins if line.startswith("release\t")).split("\t")[4]
        prefix = "singlefs/formal-tools/"
        spec = importlib.util.spec_from_file_location("lib_work_root_for_rerun", os.path.join(repo, ".claude/hooks/lib_work_root.py"))
        library = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(library)
        root = os.path.join(library.shared_data_path("formal-tools"), release[len(prefix):])
    return os.path.join(os.path.abspath(root), "verus")


def registered_rlimit(repo):
    for line in open(os.path.join(repo, ".claude/gate.d/verus-pins.tsv"), encoding="utf-8"):
        if line.startswith("rlimit\t"):
            return line.split("\t")[1].strip()
    return "10"


def package_facts(crate):
    text = open(os.path.join(crate, "Cargo.toml"), encoding="utf-8").read()
    name = re.search(r'^name\s*=\s*"([^"]+)"', text, re.M).group(1)
    edition = re.search(r'^edition\s*=\s*"([^"]+)"', text, re.M)
    return name.replace("-", "_"), edition.group(1) if edition else "2021"


def run_once(binary, crate, copy, threads, rlimit):
    name, edition = package_facts(crate)
    arguments = ["--crate-type=lib", f"--edition={edition}", "--crate-name", name, "--num-threads", str(threads), "--output-json", "--rlimit", rlimit, "src/lib.rs"]
    started = time.time()
    code, out, err = start_verus(binary, arguments, copy)
    verified = errors = None
    try:
        results = json.loads(out)["verification-results"]
        verified, errors = results.get("verified"), results.get("errors")
    except (ValueError, KeyError, TypeError):
        pass
    return code, err + "\n" + out, verified, errors, time.time() - started


def measure(arguments):
    crate = os.path.abspath(arguments.crate)
    work = os.path.abspath(arguments.work)
    if os.path.exists(work) and os.listdir(work):
        print(f"工作目录 {work} 已存在且非空", file=sys.stderr)
        return 2
    os.makedirs(work, exist_ok=True)
    copy = os.path.join(work, "crate")
    shutil.copytree(crate, copy, ignore=shutil.ignore_patterns("target"))
    binary = verus_binary(arguments.repo, arguments.verus_root)
    rlimit = registered_rlimit(arguments.repo)
    rows = read_rows(os.path.join(copy, "verus-must-red-worlds.tsv"))
    names = [line.strip() for line in open(arguments.names_file, encoding="utf-8") if line.strip()] if arguments.names_file else []
    progress = os.path.join(work, "progress.md")
    failed = False
    if arguments.baseline:
        code, diagnostics, verified, errors, seconds = run_once(binary, crate, copy, arguments.threads, rlimit)
        print(f"BASELINE exit={code} verified={verified} errors={errors} seconds={seconds:.0f}", flush=True)
        failed |= errors != 0 or verified is None
        open(progress, "a", encoding="utf-8").write(f"baseline {seconds:.0f} 秒\n")
    landing_same = counts_same = 0
    for name in names:
        if name not in rows:
            print(f"ROW name={name} 表里没有这一行")
            failed = True
            continue
        number, fields = rows[name]
        # 表里第二段是从仓根起的路径：取 crate 目录之后的那一截（crates/<包>/src/x.rs → src/x.rs）
        relative = fields[1].split("/", 2)[2]
        path = os.path.join(copy, relative)
        original = open(path, encoding="utf-8").read()
        before = fields[2].replace("\\n", "\n")
        hits = original.count(before)
        if hits != 1:
            print(f"ROW name={name} line={number} 原文命中 {hits} 次（要恰好 1 次）")
            failed = True
            continue
        mutated = original.replace(before, fields[3].replace("\\n", "\n"), 1)
        open(path, "w", encoding="utf-8").write(mutated)
        try:
            code, diagnostics, verified, errors, seconds = run_once(binary, crate, copy, arguments.threads, rlimit)
        finally:
            open(path, "w", encoding="utf-8").write(original)
        found = proof_errors(diagnostics)
        texts = {relative_path: open(os.path.join(copy, relative_path), encoding="utf-8").read().split("\n")
                 for relative_path in {p for _, p, _ in found if p and os.path.isfile(os.path.join(copy, p))}}
        texts[relative] = mutated.split("\n")
        measured = landing_of(found, texts)
        table_errors = fields[4].split("=", 1)[1]
        table_landing = sorted(fields[5].split("=", 1)[1].split(","))
        same_landing = measured == table_landing
        same_count = str(len(found)) == table_errors
        landing_same += same_landing
        counts_same += same_count
        failed |= (not same_landing) or (verified is None)
        print(f"ROW name={name} line={number} table_errors={table_errors} measured_errors={len(found)} counts={'same' if same_count else 'differ'} "
              f"table_landing={','.join(table_landing)} measured_landing={','.join(measured)} landing={'same' if same_landing else 'DIFFER'} "
              f"json_errors={errors} verified={verified} seconds={seconds:.0f}", flush=True)
        for message, place, line in found:
            print(f"    ERR {message} @ {place}:{line} -> {enclosing_function(texts.get(place, []), line) if place else '(无位置)'}", flush=True)
        open(progress, "a", encoding="utf-8").write(f"{name} {seconds:.0f} 秒\n")
    print(f"SUMMARY rows={len(names)} landing_same={landing_same} counts_same={counts_same}")
    return 1 if failed else 0


def selftest():
    sample = "error: postcondition not satisfied\n  --> src/a.rs:5:3\n   |\nerror: Resource limit (rlimit) exceeded\n  --> src/a.rs:9:1\nerror: aborting due to 2 previous errors\nerror: assertion failed\n  --> src/a.rs:12:5\n"
    found = proof_errors(sample)
    assert [(m, p, n) for m, p, n in found] == [("postcondition not satisfied", "src/a.rs", 5), ("assertion failed", "src/a.rs", 12)], found
    text = ["pub proof fn alpha()", "{", "    x;", "}", "#[inline]", "pub open spec fn beta(a: int) -> int {", "    a", "}", "fn gamma() {", "    y"]
    assert enclosing_function(text, 3) == "alpha"
    assert enclosing_function(text, 7) == "beta"
    assert enclosing_function(text, 10) == "gamma"
    assert landing_of([("e", "src/a.rs", 3), ("e", "src/a.rs", 3), ("e", None, 0)], {"src/a.rs": text}) == ["(无位置)", "alpha"]
    assert landing_of([("e", "src/missing.rs", 3)], {}) == ["(函数外)"]
    with tempfile.TemporaryDirectory() as scratch:
        fake = os.path.join(scratch, "verus")
        open(fake, "w").write('#!/bin/sh\necho "error: postcondition not satisfied" >&2\necho "  --> src/lib.rs:2:1" >&2\necho \'{"verification-results":{"verified":3,"errors":1}}\'\nexit 1\n')
        os.chmod(fake, 0o755)
        os.makedirs(os.path.join(scratch, "src"))
        process_code, out, err = start_verus(fake, [], scratch)
        assert process_code == 1 and json.loads(out)["verification-results"]["verified"] == 3 and "postcondition" in err
    print("selftest ok")
    return 0


def main():
    if sys.argv[1:] == ["--selftest"]:
        return selftest()
    parser = argparse.ArgumentParser(add_help=False)
    for option in ("--repo", "--crate", "--work", "--threads", "--verus-root", "--names-file"):
        parser.add_argument(option)
    parser.add_argument("--baseline", action="store_true")
    try:
        arguments = parser.parse_args()
        assert arguments.repo and arguments.crate and arguments.work and arguments.threads
    except (SystemExit, AssertionError):
        print(__doc__, file=sys.stderr)
        return 2
    return measure(arguments)


sys.exit(main())
