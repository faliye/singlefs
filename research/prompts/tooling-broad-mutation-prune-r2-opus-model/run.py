#!/usr/bin/env python3
"""第二轮攻方模型：照第二版形态（正文第三节 P1–P3、B4 并集、B8）判「可复用」，再在第二版上真跑，看结论翻不翻。
用法：python3 run.py <工作目录> <llvm 工具目录>   （整条经 run-with-memory-cap.sh 与 capped.sh 起；只编、只跑本目录 ws/ 下的模型 crate）

每个场景（scenarios.tsv 一行）：第一版树 = ws/（v1_file 不是 - 时换掉那一个文件），第二版树 = ws/ 换掉 v2_file（v2_source 写 ws 就取 ws/ 里那一份）。
两版各跑：没改过的全部测试、施加变异后的全部测试；两版没改过的树各逐条测试单独起进程量覆盖；第一版加变异的树只量抓到它的测试 T（看变异体自己走到哪些文件）。
判「可复用」（抓到的那一种）：
  关联集 = {T 的测试文件} ∪ T 在两版没改过的树上执行到的文件（B4 并集） ∪ 常量闭包与宏、derive 定义文件（P3、B8，按标识符名字过近似：
          起点文件里出现过的任何标识符，只要工作区里有同名的 const / static / const fn / macro_rules! / 带 derive 的 struct、enum，就把定义它的文件并进来，到不动点）。
  读法甲：被改文件照样按文件进关联集（正文 P2「T 执行到的源文件」的字面）。
  读法乙：被改文件从关联集里拿掉，只比被改函数（连所在 impl 块的头）的 sha256 与「函数外面的部分」（整份文件去掉每一个 fn 项）的 sha256；
          乙内取包住变异的最内层 fn，乙外取最外层 fn。
  读法丙（攻方提的改法，被攻过零轮）：同读法甲，只是覆盖换成「第一版加变异的树上」T（没红的：在变异体上执行到被改文件的每条测试）的覆盖 ∪ 第二版没改过的树上的覆盖。
  读法丁（攻方提的改法，被攻过零轮）：同读法丙，名字闭包再收类型别名、struct、enum、union、trait 与 pub use 再导出的定义文件。
  关联集里没有一个文件变了、被改文件那一份也没变 ⇒ 可复用。
"""
import hashlib, json, os, re, shutil, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
WORK, LLVM = sys.argv[1], sys.argv[2]
TEST_LINE = re.compile(r"^test (\S+) \.\.\. (ok|FAILED)$")
ITEM_AT_LINE_START = re.compile(r"^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?(impl|trait)\b", re.M)


def sh(command, cwd, env=None, check=True, merged=False):
    result = subprocess.run(command, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT if merged else subprocess.PIPE, text=True)
    if check and result.returncode not in (0, 101):
        sys.stderr.write(result.stdout + (result.stderr or ""))
        raise SystemExit(f"命令退 {result.returncode}：{' '.join(command)}")
    return result


# ---------- 只凭语法切函数（注释与字符串先抹成空格，保留偏移） ----------
def strip(text, strings=True):
    """把注释抹成空格；strings 为真时连字符串、字符字面量的内容一起抹（切函数用），为假时留着（算 sha256 用）。"""
    out, i, n = list(text), 0, len(text)
    def blank(a, b):
        for k in range(a, b):
            if text[k] != "\n":
                out[k] = " "
    while i < n:
        if text.startswith("//", i):
            j = text.find("\n", i); j = n if j < 0 else j; blank(i, j); i = j
        elif text.startswith("/*", i):
            j = text.find("*/", i + 2); j = n if j < 0 else j + 2; blank(i, j); i = j
        elif text[i] == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            if strings:
                blank(i + 1, j)
            i = j + 1
        elif text[i] == "'":
            m = re.match(r"'(\\.|[^\\'])'", text[i:])
            if m:
                if strings:
                    blank(i + 1, i + m.end() - 1)
                i += m.end()
            else:
                i += 1
        else:
            i += 1
    return "".join(out)


def match_brace(s, at):
    depth = 0
    for j in range(at, len(s)):
        if s[j] == "{":
            depth += 1
        elif s[j] == "}":
            depth -= 1
            if depth == 0:
                return j
    raise SystemExit("括号不配对")


def line_start(text, at):
    start = text.rfind("\n", 0, at) + 1
    while True:  # 连上紧挨着的属性行与文档注释行
        previous = text.rfind("\n", 0, max(start - 1, 0)) + 1
        line = text[previous:start].strip()
        if start > 0 and (line.startswith("#[") or line.startswith("///")):
            start = previous
        else:
            return start


def items(text):
    s = strip(text)
    containers = []
    for m in ITEM_AT_LINE_START.finditer(s):
        j = m.end()
        while j < len(s) and s[j] not in "{;":
            j += 1
        if j < len(s) and s[j] == "{":
            containers.append({"header": " ".join(text[m.start():j].split()), "body": j, "end": match_brace(s, j) + 1})
    functions = []
    for m in re.finditer(r"\bfn\s+([A-Za-z_]\w*)", s):
        j, depth = m.end(), 0
        while j < len(s):
            if s[j] in "([":
                depth += 1
            elif s[j] in ")]":
                depth -= 1
            elif depth == 0 and s[j] in "{;":
                break
            j += 1
        if j >= len(s) or s[j] == ";":
            continue
        enclosing = [c for c in containers if c["body"] < m.start() < c["end"]]
        header = min(enclosing, key=lambda c: c["end"] - c["body"])["header"] if enclosing else ""
        functions.append({"name": m.group(1), "start": line_start(text, m.start()), "end": match_brace(s, j) + 1, "header": header})
    return functions


def outside_of_functions(text):
    keep, cursor = [], 0
    for f in sorted(items(text), key=lambda f: f["start"]):
        if f["start"] >= cursor:
            keep.append(text[cursor:f["start"]]); cursor = f["end"]
        else:
            cursor = max(cursor, f["end"])
    keep.append(text[cursor:])
    return "".join(keep)


def locate(text, spec, old):
    """spec 是「函数名」或「impl 头里的一段::函数名」；交（变异在全文的偏移, 最内层 fn, 最外层 fn）。"""
    header, _, name = spec.rpartition("::")
    candidates = [f for f in items(text) if f["name"] == name and header in f["header"]]
    if len(candidates) != 1:
        raise SystemExit(f"{spec} 命中 {len(candidates)} 个 fn")
    target = candidates[0]
    body = text[target["start"]:target["end"]]
    if body.count(old) != 1:
        raise SystemExit(f"{spec} 里 {old!r} 出现 {body.count(old)} 次")
    offset = target["start"] + body.index(old)
    around = [f for f in items(text) if f["start"] <= offset < f["end"]]
    return offset, min(around, key=lambda f: f["end"] - f["start"]), max(around, key=lambda f: f["end"] - f["start"])


def digest(value):
    """语法工具（syn）比的是记号流：注释与空白不算，字符串内容算。"""
    return hashlib.sha256(" ".join(strip(value, strings=False).split()).encode()).hexdigest()[:12]


# ---------- 树、测试、覆盖 ----------
def make_tree(name, replacements):
    root = os.path.join(WORK, name)
    if os.path.exists(root):
        shutil.rmtree(root)
    shutil.copytree(os.path.join(HERE, "ws"), root)
    for relative, source in replacements:
        if relative != "-":
            shutil.copyfile(os.path.join(HERE, "ws", relative) if source == "ws" else os.path.join(HERE, source), os.path.join(root, relative))
    return root


def apply_mutation(root, relative, spec, old, new):
    path = os.path.join(root, relative)
    text = open(path, encoding="utf-8").read()
    offset, _, _ = locate(text, spec, old)
    open(path, "w", encoding="utf-8").write(text[:offset] + new + text[offset + len(old):])


def run_tests(root, target):
    env = dict(os.environ, CARGO_TARGET_DIR=target)
    result = sh(["cargo", "test", "--offline", "--manifest-path", "harness/Cargo.toml", "--tests", "--no-fail-fast"], root, env, merged=True)
    verdicts, binary = {}, None
    for line in result.stdout.splitlines():
        found = re.search(r"Running tests/(\S+)\.rs", line)
        if found:
            binary = found.group(1)
        found = TEST_LINE.match(line.strip())
        if found and binary:
            verdicts[f"harness/tests/{binary}.rs::{found.group(1)}"] = found.group(2)
    return verdicts


def coverage(root, target, only=None):
    """逐条测试单独起进程量覆盖，交 {测试全名: 执行到的源文件集合（相对树根）}。only 给了就只量那一条。"""
    env = dict(os.environ, CARGO_TARGET_DIR=target, RUSTFLAGS="-C instrument-coverage")
    profiles = os.path.join(target, "profiles")
    os.makedirs(profiles, exist_ok=True)
    built = sh(["cargo", "test", "--offline", "--manifest-path", "harness/Cargo.toml", "--tests", "--no-run", "--message-format=json"], root,
               dict(env, LLVM_PROFILE_FILE=os.path.join(profiles, "build-%p.profraw")))
    executed = {}
    for line in built.stdout.splitlines():
        if not line.startswith("{"):
            continue
        message = json.loads(line)
        if message.get("reason") != "compiler-artifact" or not message.get("executable") or message["target"]["kind"] != ["test"]:
            continue
        binary, source = message["executable"], message["target"]["src_path"]
        listing = sh([binary, "--list", "--format", "terse"], root, dict(os.environ, LLVM_PROFILE_FILE=os.path.join(profiles, "listing-%p.profraw"))).stdout
        for test in [l.split(": ")[0] for l in listing.splitlines() if l.endswith(": test")]:
            key = f"{os.path.relpath(source, root)}::{test}"
            if only and key != only:
                continue
            for leftover in os.listdir(profiles):
                os.remove(os.path.join(profiles, leftover))
            stem = re.sub(r"[^A-Za-z0-9]", "_", key)
            sh([binary, "--exact", test], root, dict(os.environ, LLVM_PROFILE_FILE=os.path.join(profiles, stem + "-%p.profraw")))
            data = os.path.join(target, stem + ".profdata")
            sh([os.path.join(LLVM, "llvm-profdata"), "merge", "-sparse", *[os.path.join(profiles, f) for f in os.listdir(profiles)], "-o", data], root)
            export = json.loads(sh([os.path.join(LLVM, "llvm-cov"), "export", binary, f"-instr-profile={data}", "-summary-only"], root).stdout)
            executed[key] = {os.path.relpath(os.path.normpath(e["filename"]), root) for e in export["data"][0]["files"] if e["summary"]["lines"]["covered"] > 0}
    return executed


def tree_files(root):
    out = {}
    for base, dirs, files in os.walk(root):
        dirs[:] = [d for d in dirs if d != "target"]
        for name in files:
            if name.endswith(".rs") or name == "Cargo.toml":
                path = os.path.join(base, name)
                out[os.path.relpath(path, root)] = open(path, encoding="utf-8").read()
    return out


DEFINITION = re.compile(r"\b(?:const|static)\s+(?:mut\s+)?(?:fn\s+)?([A-Za-z_]\w*)|macro_rules!\s*([A-Za-z_]\w*)")
DERIVED = re.compile(r"#\[derive\([^)]*\)\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum)\s+([A-Za-z_]\w*)")


ANY_ITEM = re.compile(r"\b(?:const|static|type|struct|enum|union|trait)\s+(?:mut\s+)?(?:fn\s+)?([A-Za-z_]\w*)|macro_rules!\s*([A-Za-z_]\w*)|\bpub(?:\([^)]*\))?\s+use\s+[^;]*?\b([A-Za-z_]\w*)\s*;")


def closure(start, trees, any_item=False):
    """P3、B8 按名字过近似：两版的文件一起看。any_item 为真时（读法丁）连类型别名、struct、enum、trait 与 pub use 再导出的定义文件一起收。"""
    definitions = {}
    for files in trees:
        for path, text in files.items():
            s = strip(text)
            for m in (ANY_ITEM if any_item else DEFINITION).finditer(s):
                definitions.setdefault(m.group(1) or m.group(2) or m.group(len(m.groups())), set()).add(path)
            for m in DERIVED.finditer(s):
                definitions.setdefault(m.group(1), set()).add(path)
    result, frontier = set(start), set(start)
    while frontier:
        identifiers = set()
        for path in frontier:
            for files in trees:
                if path in files:
                    identifiers |= set(re.findall(r"[A-Za-z_]\w*", strip(files[path])))
        added = set().union(*[definitions.get(i, set()) for i in identifiers]) - result
        result |= added
        frontier = added
    return result


def main():
    os.makedirs(WORK, exist_ok=True)
    rows = [r.rstrip("\n").split("\t") for r in list(open(os.path.join(HERE, "scenarios.tsv"), encoding="utf-8"))[1:]]
    for scenario, v1_file, v1_source, v2_file, v2_source, mutated, spec, old, new in rows:
        v1 = make_tree(f"{scenario}-v1", [(v1_file, v1_source)])
        v2 = make_tree(f"{scenario}-v2", [(v1_file, v1_source), (v2_file, v2_source)])
        v1m = make_tree(f"{scenario}-v1m", [(v1_file, v1_source)]); apply_mutation(v1m, mutated, spec, old, new)
        v2m = make_tree(f"{scenario}-v2m", [(v1_file, v1_source), (v2_file, v2_source)]); apply_mutation(v2m, mutated, spec, old, new)
        plain = os.path.join(WORK, "target-plain")
        base1, base2 = run_tests(v1, plain), run_tests(v2, plain)
        fail1 = sorted(k for k, v in run_tests(v1m, plain).items() if v != "ok")
        fail2 = sorted(k for k, v in run_tests(v2m, plain).items() if v != "ok")
        green = lambda b: "green" if b and all(v == "ok" for v in b.values()) else "RED"
        print(f"SCENARIO {scenario} v1-baseline={green(base1)}({len(base1)}) v1+M failing={fail1 or 'none'} => {'caught' if fail1 else 'missed'}")
        cov1 = coverage(v1, os.path.join(WORK, f"target-cov-{scenario}-v1"))
        cov2 = coverage(v2, os.path.join(WORK, f"target-cov-{scenario}-v2"))
        f1, f2 = tree_files(v1), tree_files(v2)
        changed = sorted(p for p in set(f1) | set(f2) if f1.get(p) != f2.get(p))
        if fail1:
            tests = [fail1[0]]
            kind = "caught"
        else:  # 没红的：覆盖到被改文件的每条测试
            tests = sorted(t for t in set(cov1) | set(cov2) if mutated in cov1.get(t, set()) | cov2.get(t, set()))
            kind = "missed"
        union = set().union(*[cov1.get(t, set()) | cov2.get(t, set()) for t in tests]) if tests else set()
        start = {mutated} | {t.split("::")[0] for t in tests} | union
        associated = closure(start, [f1, f2])
        mutant_all = coverage(v1m, os.path.join(WORK, f"target-cov-{scenario}-v1m"))
        if fail1:
            mutant = mutant_all.get(tests[0], set())
            print(f"SCENARIO {scenario} T={tests[0]} T-on-v1+M-executed={','.join(sorted(mutant))} not-in-v1-or-v2-baseline-coverage-of-T={','.join(sorted(mutant - union)) or 'none'}")
        print(f"SCENARIO {scenario} {kind}-tests={tests} union-coverage={','.join(sorted(union))} P3-B8-closure-adds={','.join(sorted(associated - start)) or 'none'} changed={','.join(changed)}")
        t1, t2 = f1[mutated], f2[mutated]
        o1, inner1, outer1 = locate(t1, spec, old)
        o2, inner2, outer2 = locate(t2, spec, old)
        outside_same = digest(outside_of_functions(t1)) == digest(outside_of_functions(t2))
        # 读法丙（攻方提的改法，被攻过零轮）：覆盖取「第一版加变异」上的覆盖 ∪ 第二版没改过的覆盖；没红的取在变异体上执行到被改文件的每条测试；被改文件照样按文件
        mutant_tests = tests if fail1 else sorted(t for t in mutant_all if mutated in mutant_all[t])
        mutant_union = set().union(*[mutant_all.get(t, set()) | cov2.get(t, set()) for t in mutant_tests]) if mutant_tests else set()
        associated_c = closure({mutated} | {t.split("::")[0] for t in mutant_tests} | mutant_union, [f1, f2])
        associated_d = closure({mutated} | {t.split("::")[0] for t in mutant_tests} | mutant_union, [f1, f2], any_item=True)
        added_by_d = ",".join(sorted(associated_d - associated_c)) or "none"
        print(f"SCENARIO {scenario} reading-丁-closure-adds={added_by_d}")
        for reading in ("甲", "乙内", "乙外", "丙", "丁"):
            if reading in ("甲", "丙", "丁"):
                files = {"甲": associated, "丙": associated_c, "丁": associated_d}[reading]
                key_same = True
            else:
                files = associated - {mutated}
                a, b = (inner1, inner2) if reading == "乙内" else (outer1, outer2)
                key_same = digest(a["header"] + t1[a["start"]:a["end"]]) == digest(b["header"] + t2[b["start"]:b["end"]]) and outside_same
            hit = sorted(set(changed) & files)
            verdict = "reuse " + kind if key_same and not hit else "rerun"
            print(f"SCENARIO {scenario} reading={reading} mutated-function-and-outside-unchanged={key_same} changed-associated={','.join(hit) or 'none'} => {verdict}")
        print(f"SCENARIO {scenario} v2-baseline={green(base2)}({len(base2)}) v2+M failing={fail2 or 'none'} => rerun-verdict={'caught' if fail2 else 'missed'}")


main()
