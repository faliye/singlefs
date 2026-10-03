#!/usr/bin/env python3
"""攻方第三轮模型：C2（版本内按覆盖挑测试）挑漏、C1（哪些变异算「这次改到的」）漏算。

用法（整条经内存包装跑）：
  run.py <工作目录> <llvm-tools 1.98.0 的 bin 目录>
工作目录里建 c2/、c1/ 两棵树与各自的编译目录；ws/ 本身不动。

C2：插桩编一次，每条测试单独一个进程跑两遍（LLVM_PROFILE_FILE 不带 %p / 每条测试一个目录带 %p），
按三种取法导出「执行到的函数」（入口计数 > 0），对每条变异挑「执行到被改函数」的测试；再在不插桩的树上施加变异，
挑出来的测试逐条跑（遇第一个失败即停）、全部测试（cargo test，含文档测试）跑一遍，比结局。
C1：每个场景造第二版（只动一处），按三种读法判被改函数「改没改」，再在两版上各施加同一条变异真跑全部测试，比结局。
"""
import json
import os
import re
import shutil
import subprocess
import sys

MODEL = os.path.dirname(os.path.abspath(__file__))
WS = os.path.join(MODEL, "ws")
WORK = os.path.abspath(sys.argv[1])
LLVM = os.path.abspath(sys.argv[2])


def sh(command, cwd, env=None, check=True):
    result = subprocess.run(command, cwd=cwd, env=env, capture_output=True, text=True)
    if check and result.returncode != 0:
        raise SystemExit(f"命令失败 {command} 退 {result.returncode}\n{result.stdout[-3000:]}\n{result.stderr[-3000:]}")
    return result


def fresh_tree(name):
    root = os.path.join(WORK, name)
    if os.path.exists(root):
        shutil.rmtree(root)
    shutil.copytree(WS, root)
    return root


def replace_once(root, relative, old, new):
    path = os.path.join(root, relative)
    text = open(path).read()
    if text.count(old) != 1:
        raise SystemExit(f"{relative} 里 {old!r} 出现 {text.count(old)} 次，不是 1 次")
    with open(path, "w") as handle:
        handle.write(text.replace(old, new))


def build(root, target, instrument=False):
    env = dict(os.environ, CARGO_TARGET_DIR=target)
    if instrument:
        env["RUSTFLAGS"] = "-C instrument-coverage"
        os.makedirs(os.path.join(target, "build-profiles"), exist_ok=True)
        env["LLVM_PROFILE_FILE"] = os.path.join(target, "build-profiles", "%p.profraw")
    out = sh(["cargo", "test", "--no-run", "--message-format=json"], root, env).stdout
    tests, bins = {}, []
    for line in out.splitlines():
        if not line.startswith("{"):
            continue
        message = json.loads(line)
        if message.get("reason") != "compiler-artifact" or not message.get("executable"):
            continue
        target_info = message["target"]
        if message["profile"]["test"]:
            tests[f"{target_info['name']}:{'+'.join(target_info['kind'])}"] = message["executable"]
        elif "bin" in target_info["kind"]:
            bins.append(message["executable"])
    return tests, bins


def list_tests(executable, cwd, env):
    out = sh([executable, "--list", "--format", "terse"], cwd, env).stdout
    return [line[: -len(": test")] for line in out.splitlines() if line.endswith(": test")]


# ── C2 ─────────────────────────────────────────────────────────────────────────────
C2_MUTANTS = [
    # 编号, 文件, 原文, 替换文, 形状
    ("X1", "mcore/src/sub.rs", "i * 7 + 3", "i + 7 + 3", "子进程（current_exe 跑子测试，正常退出）"),
    ("X1b", "mcore/src/sub.rs", "i * 5 + 2", "i + 5 + 2", "子进程（current_exe 跑子测试，打印完 abort）"),
    ("X1c", "mcore/src/sub.rs", "i * 9 + 4", "i + 9 + 4", "子进程（先后两个子进程，只在第一个里执行）"),
    ("X2", "mcore/src/tool.rs", "h * 31", "h + 31", "子进程（CARGO_BIN_EXE 另一个二进制）"),
    ("X2b", "mcore/src/tool.rs", "u64::from(b) * 3", "u64::from(b) + 3", "子进程（CARGO_BIN_EXE，函数只在那个二进制里）"),
    ("X3", "mcore/src/mixed.rs", "x * x", "x + x", "const fn 运行时与常量上下文都被叫"),
    ("X4", "mcore/src/inl.rs", "a * 3 + b", "a + 3 + b", "跨 crate #[inline(always)]"),
    ("X5", "mcore/src/fptr.rs", "x * 3", "x + 3", "static 函数指针表"),
    ("X6", "mcore/src/dynd.rs", "self.0 * self.0", "self.0 + self.0", "dyn 分派"),
    ("X7", "mcore/src/doc.rs", "x / 2", "x * 2", "只有文档测试抓得到"),
]


def executed_functions(profiles, objects, scratch, root):
    """profraw 合并后按 objects 导出，交回入口计数 > 0 的函数：[(仓内文件, 首行, 末行, 名字)]。"""
    if not profiles:
        return []
    data = os.path.join(scratch, "merged.profdata")
    sh([os.path.join(LLVM, "llvm-profdata"), "merge", "-sparse", *profiles, "-o", data], root)
    command = [os.path.join(LLVM, "llvm-cov"), "export", objects[0], f"-instr-profile={data}"]
    for extra in objects[1:]:
        command += ["-object", extra]
    export = json.loads(sh(command, root).stdout)
    found = []
    for function in export["data"][0]["functions"]:
        if function["count"] <= 0:
            continue
        regions = [region for region in function["regions"] if region[5] == 0]
        if not regions:
            continue
        name = os.path.relpath(function["filenames"][0], root)
        found.append((name, min(r[0] for r in regions), max(r[2] for r in regions), function["name"]))
    return found


def line_of(root, relative, old):
    text = open(os.path.join(root, relative)).read()
    return text[: text.index(old)].count("\n") + 1


def c2():
    root = fresh_tree("c2-src")
    target = os.path.join(WORK, "c2-target-cov")
    tests, bins = build(root, target, instrument=True)
    print(f"C2 instrumented test-targets={sorted(tests)} bins={[os.path.basename(b) for b in bins]}")
    profile_root = os.path.join(WORK, "c2-profiles")
    if os.path.exists(profile_root):
        shutil.rmtree(profile_root)
    os.makedirs(profile_root)
    throwaway = dict(os.environ, LLVM_PROFILE_FILE=os.path.join(profile_root, "listing-%p.profraw"))
    coverage = {}  # 测试 → 取法 → 执行到的函数
    for target_name, executable in sorted(tests.items()):
        for test in list_tests(executable, root, throwaway):
            key = f"{target_name}::{test}"
            stem = re.sub(r"[^A-Za-z0-9_]", "_", key)
            naive = os.path.join(profile_root, "naive", stem + ".profraw")
            pid_dir = os.path.join(profile_root, "pid", stem)
            os.makedirs(os.path.dirname(naive), exist_ok=True)
            os.makedirs(pid_dir, exist_ok=True)
            first = sh([executable, "--exact", test, "--test-threads=1"], root, dict(os.environ, LLVM_PROFILE_FILE=naive), check=False)
            second = sh([executable, "--exact", test, "--test-threads=1"], root, dict(os.environ, LLVM_PROFILE_FILE=os.path.join(pid_dir, "%p.profraw")), check=False)
            if first.returncode != 0 or second.returncode != 0:
                raise SystemExit(f"插桩基线红：{key} 退 {first.returncode} / {second.returncode}")
            pid_files = sorted(os.path.join(pid_dir, f) for f in os.listdir(pid_dir))
            scratch = os.path.join(profile_root, "merge", stem)
            os.makedirs(scratch, exist_ok=True)
            coverage[key] = {
                "naive+test-binary": executed_functions([naive] if os.path.exists(naive) else [], [executable], scratch, root),
                "pid+test-binary": executed_functions(pid_files, [executable], scratch, root),
                "pid+all-binaries": executed_functions(pid_files, [executable, *bins], scratch, root),
            }
            print(f"C2 coverage-run test={key} pid-profraw-files={len(pid_files)}")
    plain_root = fresh_tree("c2-plain")
    plain_target = os.path.join(WORK, "c2-target-plain")
    for number, relative, old, new, shape in C2_MUTANTS:
        line = line_of(root, relative, old)
        replace_once(plain_root, relative, old, new)
        plain_tests, _ = build(plain_root, plain_target)
        everything = sh(["cargo", "test", "--no-fail-fast"], plain_root, dict(os.environ, CARGO_TARGET_DIR=plain_target), check=False)
        failing = sorted(set(re.findall(r"^test (\S+) \.\.\. FAILED$", everything.stdout, re.M))
                         | set(re.findall(r"^    (\S+(?: - \S+)*(?: \(line \d+\))?)$", everything.stdout, re.M)))
        all_verdict = "caught" if everything.returncode != 0 else "missed"
        for policy in ("naive+test-binary", "pid+test-binary", "pid+all-binaries"):
            selected = [key for key in sorted(coverage)
                        if any(f == relative and start <= line <= end for f, start, end, _ in coverage[key][policy])]
            verdict = "zero-coverage=>missed"
            failed_at = None
            if selected:
                verdict = "missed"
                for key in selected:
                    target_name, test = key.split("::", 1)
                    run = sh([plain_tests[target_name], "--exact", test, "--test-threads=1"], plain_root, check=False)
                    if run.returncode != 0:
                        verdict, failed_at = "caught", key
                        break
            hole = "HOLE" if verdict != "caught" and all_verdict == "caught" else "same"
            print(f"C2 {number} shape={shape} policy={policy} mutated={relative}:{line} selected={selected} "
                  f"selected-verdict={verdict} first-failing={failed_at} all-tests-verdict={all_verdict} => {hole}")
        print(f"C2 {number} all-tests-failing={failing}")
        replace_once(plain_root, relative, new, old)


# ── C1 ─────────────────────────────────────────────────────────────────────────────
def tokens(source):
    """词法单元 [(文本, 起, 止)]：去空白与注释（行注释、可嵌套的块注释、文档注释）；串、原始串、字节串、字符、生命周期各一个单元。"""
    out, i, n = [], 0, len(source)
    while i < n:
        c = source[i]
        if c.isspace():
            i += 1
        elif source.startswith("//", i):
            j = source.find("\n", i)
            i = n if j < 0 else j
        elif source.startswith("/*", i):
            depth, i = 1, i + 2
            while i < n and depth:
                if source.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif source.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
        elif re.match(r'(b?r#*")', source[i:]):
            prefix = re.match(r'(b?r)(#*)"', source[i:])
            closing = '"' + prefix.group(2)
            j = source.index(closing, i + len(prefix.group(0))) + len(closing)
            out.append((source[i:j], i, j))
            i = j
        elif c == '"' or source.startswith('b"', i):
            j = i + (2 if c == "b" else 1)
            while source[j] != '"':
                j += 2 if source[j] == "\\" else 1
            out.append((source[i:j + 1], i, j + 1))
            i = j + 1
        elif c == "'":
            m = re.match(r"'(\\.[^']*|[^\\'])'", source[i:])
            if m:
                out.append((m.group(0), i, i + len(m.group(0))))
                i += len(m.group(0))
            else:
                m = re.match(r"'[A-Za-z_][A-Za-z0-9_]*", source[i:])
                out.append((m.group(0), i, i + len(m.group(0))))
                i += len(m.group(0))
        elif c.isalnum() or c == "_":
            m = re.match(r"[A-Za-z0-9_]+", source[i:])
            out.append((m.group(0), i, i + len(m.group(0))))
            i += len(m.group(0))
        else:
            out.append((c, i, i + 1))
            i += 1
    return out


def regex_normal(text):
    """读法甲用的「去注释与空白」：正则去块注释、行注释，空白折成一个空格（不认串）。"""
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    text = re.sub(r"//[^\n]*", "", text)
    return re.sub(r"\s+", " ", text).strip()


def token_normal(text):
    return " ".join(t for t, _, _ in tokens(text))


def functions(source):
    """带函数体的 fn 项：[(键, 项起, 体起, 体止)]；键 = 外层 impl / trait 块头的词法串 + 名字 + 同键序号。"""
    toks = tokens(source)
    found, seen = [], {}
    for index, (text, start, _) in enumerate(toks):
        if text != "fn" or index + 1 >= len(toks) or not re.match(r"[A-Za-z_]", toks[index + 1][0]):
            continue
        j, depth = index + 2, 0
        while j < len(toks):
            t = toks[j][0]
            if t in "([":
                depth += 1
            elif t in ")]":
                depth -= 1
            elif t == ";" and depth == 0:
                j = None
                break
            elif t == "{" and depth == 0:
                break
            j += 1
        if j is None or j >= len(toks):
            continue
        body_start, k, depth = toks[j][1], j, 0
        while k < len(toks):
            if toks[k][0] == "{":
                depth += 1
            elif toks[k][0] == "}":
                depth -= 1
                if depth == 0:
                    break
            k += 1
        body_end = toks[k][2]
        item = index
        while True:
            if item > 0 and toks[item - 1][0] in ("pub", "const", "unsafe", "async", "extern"):
                item -= 1
            elif item > 3 and toks[item - 1][0] == ")" and toks[item - 3][0] == "(" and toks[item - 4][0] == "pub":
                item -= 4
            else:
                break
        header = enclosing_header(source, toks, index)
        base = f"{header}::{toks[index + 1][0]}"
        seen[base] = seen.get(base, 0) + 1
        found.append((f"{base}#{seen[base]}", toks[item][1], body_start, body_end))
    return found


def enclosing_header(source, toks, index):
    """往回找没闭合的 `{`，取它前面那个 impl / trait 块头（到上一个 `}` `;` `{` 为止）。"""
    depth = 0
    for back in range(index - 1, -1, -1):
        t = toks[back][0]
        if t == "}":
            depth += 1
        elif t == "{":
            if depth == 0:
                head = back - 1
                while head >= 0 and toks[head][0] not in ("}", ";", "{"):
                    head -= 1
                return " ".join(x for x, _, _ in toks[head + 1:back])
            depth -= 1
    return "<file>"


def outermost(spans):
    spans = sorted(spans)
    kept = []
    for start, end in spans:
        if kept and start < kept[-1][1]:
            continue
        kept.append((start, end))
    return kept


def outside(source, spans):
    pieces, at = [], 0
    for start, end in outermost(spans):
        pieces.append(source[at:start])
        at = end
    pieces.append(source[at:])
    return "\n".join(pieces)


READINGS = {
    # 读法：函数文本取哪一段、用哪种「去注释与空白」
    "R1-regex-whole-item": ("item", regex_normal),
    "R2-lexer-whole-item": ("item", token_normal),
    "R3-lexer-body-only": ("body", token_normal),
}


def judge_c1(v1_root, v2_root, relative, old_v1, old_v2, changed_files):
    """每种读法下，被改函数所在文件与函数：这条变异算不算「这次改到的」。"""
    v1 = open(os.path.join(v1_root, relative)).read()
    v2 = open(os.path.join(v2_root, relative)).read()
    verdicts = {}
    for reading, (part, normal) in READINGS.items():
        def table(source):
            return {key: (item, body_start, body_end) for key, item, body_start, body_end in functions(source)}

        def text_of(source, entry):
            item, body_start, body_end = entry
            return normal(source[item:body_end] if part == "item" else source[body_start:body_end])

        def containing(source, offset):
            best = None
            for key, item, body_start, body_end in functions(source):
                if body_start <= offset < body_end and (best is None or body_start > best[1]):
                    best = (key, body_start)
            return best[0]

        key = containing(v1, v1.index(old_v1))
        assert key == containing(v2, v2.index(old_v2)), (key, reading)
        t1, t2 = table(v1), table(v2)
        reasons = []
        if text_of(v1, t1[key]) != text_of(v2, t2[key]):
            reasons.append("function-text-changed")
        if normal(outside(v1, [(i, e) for i, _, e in t1.values()])) != normal(outside(v2, [(i, e) for i, _, e in t2.values()])):
            reasons.append("outside-part-of-its-file-changed")
        verdicts[reading] = ("rerun:" + "+".join(reasons)) if reasons else "reuse"
    others = []
    for changed in changed_files:
        if changed == relative:
            continue
        if not changed.endswith(".rs"):
            others.append(f"{changed}:not-rust(C1-silent)")
            continue
        a = {k for k, *_ in functions(open(os.path.join(v1_root, changed)).read())}
        b = {k for k, *_ in functions(open(os.path.join(v2_root, changed)).read())}
        others.append(f"{changed}:added={sorted(b - a)} removed={sorted(a - b)}")
    return key, verdicts, others


C1_SCENARIOS = [
    # 编号, 形状, 第二版的改动 [(文件, 原文, 新文)], 变异 (文件, 原文, 替换文)
    ("Y1", "串字面量里的空白", [("mcore/src/y1.rs", '"a  "', '"a "')], ("mcore/src/y1.rs", "n == 0", "n != 0")),
    ("Y2", "串里的 /* */ 夹住代码", [("mcore/src/y2.rs", "let k = 3;", "let k = 2;")], ("mcore/src/y2.rs", "x * k", "x + k")),
    ("Y3", "串里的 // 吃掉同行代码", [("mcore/src/y3.rs", "let k = 3;", "let k = 2;")], ("mcore/src/y3.rs", "x * k", "x + k")),
    ("Y4", "别的文件里的 macro_rules! 展开进被改函数", [("mcore/src/y4_macros.rs", "$x * 3", "$x * 1")], ("mcore/src/y4.rs", "s - 1", "s + 1")),
    ("Y5", "约束挪进 where 并换 trait，函数体不变", [("mcore/src/y5.rs", "pub fn pick<T: A>(t: T, x: u32) -> u32 {", "pub fn pick<T>(t: T, x: u32) -> u32\nwhere\n    T: B,\n{")], ("mcore/src/y5.rs", "x * t.v()", "x + t.v()")),
    ("Y6", "另一个文件的 impl 覆盖 trait 默认方法", [("mcore/src/y6_impl.rs", "        3\n    }\n}", "        3\n    }\n\n    fn size(&self) -> u32 {\n        4\n    }\n}")], ("mcore/src/y6_trait.rs", "self.base() * 2", "self.base() + 2")),
    ("Y7", "Cargo.toml 的 overflow-checks", [("Cargo.toml", "overflow-checks = true", "overflow-checks = false")], ("mcore/src/y7.rs", "x + 1", "x - 1")),
    ("Y8", "include_str! 读的数据文件", [("mcore/src/y8_k.txt", "3", "2")], ("mcore/src/y8.rs", "x * include_str!", "x + include_str!")),
    ("Y9", "别的文件给返回类型加 derive(Default)，无效的变异变得编得过", [("mcore/src/y9_type.rs", "pub struct W(pub u32);", "#[derive(Default)]\npub struct W(pub u32);")], ("mcore/src/y9.rs", "crate::y9_type::W(x * 2)", "Default::default()")),
]


def all_tests(root, target):
    run = sh(["cargo", "test", "--no-fail-fast"], root, dict(os.environ, CARGO_TARGET_DIR=target), check=False)
    failing = sorted(set(re.findall(r"^test (\S+) \.\.\. FAILED$", run.stdout, re.M)))
    if run.returncode != 0 and "could not compile" in run.stderr:
        return "unviable", failing, run.returncode
    return ("caught" if run.returncode != 0 else "missed"), failing, run.returncode


def c1():
    target = os.path.join(WORK, "c1-target")
    for number, shape, edits, (relative, old, new) in C1_SCENARIOS:
        v1 = fresh_tree("c1-v1")
        v2 = fresh_tree("c1-v2")
        for edit_file, edit_old, edit_new in edits:
            replace_once(v2, edit_file, edit_old, edit_new)
        key, verdicts, others = judge_c1(v1, v2, relative, old, old, [e[0] for e in edits])
        print(f"C1 {number} shape={shape} mutated-function={relative}::{key} other-changed-files={others}")
        for reading, verdict in verdicts.items():
            print(f"C1 {number} reading={reading} => {verdict}")
        baseline, base_failing, _ = all_tests(v2, target)
        replace_once(v1, relative, old, new)
        on_v1, v1_failing, _ = all_tests(v1, target)
        replace_once(v2, relative, old, new)
        on_v2, v2_failing, _ = all_tests(v2, target)
        print(f"C1 {number} v2-baseline-failing={base_failing} v1+M={on_v1} {v1_failing} v2+M={on_v2} {v2_failing}")
        flip = on_v1 != on_v2
        for reading, verdict in verdicts.items():
            hit = "HOLE(reused-verdict-flips)" if verdict == "reuse" and flip else ("ok" if flip else "no-flip")
            print(f"C1 {number} reading={reading} c1={verdict} v1+M={on_v1} v2+M={on_v2} => {hit}")


if __name__ == "__main__":
    which = sys.argv[3:] or ["c2", "c1"]
    os.makedirs(WORK, exist_ok=True)
    if "c2" in which:
        c2()
    if "c1" in which:
        c1()
