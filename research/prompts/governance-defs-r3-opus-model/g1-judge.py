#!/usr/bin/env python3
"""G1：59 号 judge_one 的真实判档 vs 变异分诊员照定义第 5 步的读法。

judge_one 从今天的 .claude/gate.d/59-crates-mutation-replay.sh 里按函数名用 ast 现抽，
subprocess.run 换成桩，喂合成的 cargo 输出（形状是推的：cargo 1.94 二进制里有
「test failed」「, to rerun pass `」「process didn't exit successfully: 」「test exited abnormally」这几个串，
没有真跑 cargo——这个容器里内存包装退 251）。
用法：python3 g1-judge.py <仓根>
"""
import ast, os, re, sys, tempfile, types

root = sys.argv[1]
stage = os.path.join(root, ".claude/gate.d/59-crates-mutation-replay.sh")
text = open(stage, encoding="utf-8").read()
py = text.split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0]
tree = ast.parse(py)
judge = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == "judge_one")
consts = [n for n in tree.body if isinstance(n, ast.Assign) and isinstance(n.targets[0], ast.Name) and n.targets[0].id.endswith("_EXIT")]
module = ast.Module(body=consts + [judge], type_ignores=[])

class Run:
    def __init__(self, rc, out, err):
        self.returncode, self.stdout, self.stderr = rc, out, err

current = {}
def fake_run(*a, **k):
    return current["run"]
ns = {"re": re, "os": os, "subprocess": types.SimpleNamespace(run=fake_run),
      "table": "crates/mutations.tsv", "timeout_seconds": 1800, "memory_max": "4G",
      "memory_cap_runner": "/dev/null", "TIMEOUT_KILL_GRACE_SECONDS": 30}
exec(compile(module, stage, "exec"), ns)
judge_one = ns["judge_one"]

NAME = "accounting_after_first_transaction_matches_the_registered_anchor"
BIN = "-p singlefs-harness --bin e156_allocation_basis_counts"
RUNNING = ("   Compiling singlefs-harness v0.1.0 (/w/crates/singlefs-harness)\n"
           "    Finished `test` profile [unoptimized + debuginfo] target(s) in 20.00s\n"
           "     Running unittests src/bin/e156_allocation_basis_counts.rs (target/debug/deps/e156_allocation_basis_counts-0f)\n")
TEST_FAILED = f"error: test failed, to rerun pass `{BIN}`\n"
KILLED = (TEST_FAILED + "\nCaused by:\n  process didn't exit successfully: `/w/target/debug/deps/e156_allocation_basis_counts-0f` (signal: 11, SIGSEGV: invalid memory reference)\n"
          "note: test exited abnormally; to see the full output pass --no-capture to the harness.\n")
FAIL_BLOCK = ("\nfailures:\n\n---- tests::{n} stdout ----\n\nthread 'tests::{n}' panicked at src/bin/e156_allocation_basis_counts.rs:3830:9:\nassertion `left == right` failed\n  left: 13\n right: 14\n\n"
              "failures:\n    tests::{n}\n\ntest result: FAILED. 40 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s\n\n")

cases = [
    # 名字, 点名测试名（表里第 6 列）, 退出码, stdout, stderr, 真实发生了什么
    ("compile-error", NAME, 101, "",
     "   Compiling singlefs-harness v0.1.0 (/w/crates/singlefs-harness)\nerror[E0308]: mismatched types\n --> crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:1:1\n\nFor more information about this error, try `rustc --explain E0308`.\nerror: could not compile `singlefs-harness` (bin \"e156_allocation_basis_counts\" test) due to 1 previous error\n",
     "替换文编不过"),
    ("named-test-failed", NAME, 101, f"\nrunning 41 tests\ntest tests::{NAME} ... FAILED\n" + FAIL_BLOCK.format(n=NAME), RUNNING + TEST_FAILED,
     "点名测试红了（抓到）"),
    ("named-test-failed-dollar-row", NAME + "$", 101, f"\nrunning 41 tests\ntest tests::{NAME} ... FAILED\n" + FAIL_BLOCK.format(n=NAME), RUNNING + TEST_FAILED,
     "点名测试红了（抓到），表里测试名尾巴带 $"),
    ("other-test-failed-named-absent", NAME + "_renamed", 101, f"\nrunning 41 tests\ntest tests::{NAME} ... FAILED\n" + FAIL_BLOCK.format(n=NAME), RUNNING + TEST_FAILED,
     "别的测试红了，点名的那个名字在输出里不存在（测试改过名）"),
    ("crash-after-named-test-passed", NAME, 101, f"\nrunning 41 tests\ntest tests::{NAME} ... ok\n", RUNNING + KILLED,
     "点名测试跑完没红，之后测试进程被信号杀"),
    ("crash-before-named-test", NAME, 101, "\nrunning 41 tests\n", RUNNING + KILLED,
     "点名测试没跑完，测试进程被信号杀"),
    ("named-test-not-run", NAME, 0, "\nrunning 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 41 filtered out; finished in 0.00s\n\n", RUNNING,
     "点名测试没跑到"),
]

def definition_reading(verdict, message, stdout, stderr):
    """mutation-triage.md 第 5 步 / mutation-sampling.md 第 82 行的读法。"""
    out = stdout + stderr
    process_unfinished = "process didn't exit successfully" in out or ("running " in out and "test result:" not in out and "could not compile" not in out)
    predicted = None
    if process_unfinished:
        predicted = "invalid" if (re.search(r"^error", out, re.M) or "could not compile" in out) else "failure"
    reading = ""
    if verdict == "invalid":
        reading = "进程被杀" if "process didn't exit successfully" in message else "替换文编不过（第八类）"
    return predicted, reading

work = tempfile.mkdtemp(prefix="g1-judge-")
os.makedirs(os.path.join(work, "src"))
target = os.path.join(work, "src/x.rs")
print("case\t59 号判档\t定义第 5 步对「测试进程没跑完」预言的判档\t分诊员照定义读「无效」\t真实发生的")
for name, expected, rc, out, err, truth in cases:
    open(target, "w").write("let a = 1;\n")
    current["run"] = Run(rc, out, err)
    _, verdict, message = judge_one(work, {}, (7, "合成变异", "src/x.rs", "let a = 1;", "let a = 2;", BIN, expected))
    predicted, reading = definition_reading(verdict, message, out, err)
    flag = ""
    if predicted is not None and predicted != verdict:
        flag = "  <- 定义预言与 59 号不符"
    print(f"{name}\t{verdict}\t{predicted or '（进程跑完了，不适用）'}\t{reading or '—'}\t{truth}{flag}")
    if verdict == "invalid":
        print("    59 号打给分诊员的那条：" + message.splitlines()[0])

print()
print("crates/mutations.tsv 里第 6 列（必须红的测试名）带正则元字符的行：")
rows = 0
hits = []
for line_number, line in enumerate(open(os.path.join(root, "crates/mutations.tsv"), encoding="utf-8"), 1):
    line = line.rstrip("\n")
    if not line or line.startswith("#"):
        continue
    fields = line.split("\t")
    if len(fields) != 6:
        continue
    rows += 1
    expected = fields[5]
    if re.search(r"[][$^*+?(){}|\\.]", expected):
        bare = expected.rstrip("$")
        red = re.compile(r"^test (\S+::)?" + re.escape(expected) + r" \.\.\. FAILED$", re.M)
        source = open(os.path.join(root, fields[1]), encoding="utf-8").read()
        has_fn = re.search(r"fn\s+" + re.escape(bare) + r"\s*\(", source) is not None
        matched = bool(red.search(f"test tests::{bare} ... FAILED\n"))
        hits.append(f"  crates/mutations.tsv:{line_number}\t{expected}\t源文件里有 fn {bare}：{'是' if has_fn else '否'}\t59 号的 red_pattern 认不认「test tests::{bare} ... FAILED」：{'认' if matched else '不认'}")
print("\n".join(hits))
print(f"  共 {len(hits)} 行 / 表里成形的 {rows} 行")
