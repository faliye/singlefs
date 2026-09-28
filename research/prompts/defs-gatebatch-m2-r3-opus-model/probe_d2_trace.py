"""D2（判法摘要：准入模块按 ast 从判法入口求模块级闭包）漏看什么，三件：
  ① 今天：在小 git 仓上真跑一遍判法那几个子命令（crash-case-command / judge（单机日志与 merge 日志各一次）/ record / marker-check / marker-path / shardable），
     sys.settrace 记下执行到的 admission.py 里的函数与它们读的模块级名字，与静态闭包比：运行时用到而闭包里没有的，改了它判法变、摘要不变。
  ② 以后：几种 ast 闭包看不见的写法（元组解包赋值、if 块里的 def、分派表的值不是名字、判法挪进 import 的别的模块），在准入模块原文的副本上
     两步改：先改成那种写法（摘要变一次），再改判法本身，看摘要变不变。只改字符串、不落盘，调被判模块的 crash_case_judging_digest_text。
  ③ 54 号不进指纹：同一份日志，54 号转给 crash-case-judge 的 --threads / --threads-origin 不同，判的结论不同（文件头说 54 号「还定着结论的只剩流程的次序」）。
复跑：python3 probe_d2_trace.py"""
import ast
import hashlib
import io
import contextlib
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common as pc  # noqa: E402

cells = pc.Cells("D2 判法摘要的闭包")
admission = pc.load(pc.ADMISSION, "admission_r3_probe")
ADMISSION_REAL = os.path.realpath(pc.ADMISSION)
source = open(pc.ADMISSION, encoding="utf-8").read()
digest_text, closure_count = admission.crash_case_judging_digest_text(source)
closure = {line[3:] for line in digest_text.decode("utf-8").split("\n") if line.startswith("## ")} - {"分派", "入口", "main"}
definitions = set(admission.top_level_definitions(ast.parse(source)))
print(f"INFO\t静态闭包 {closure_count} 个定义（摘要里 ## 节 {len(closure)} 个），模块级定义共 {len(definitions)} 个")

# ── ① 运行时 vs 静态闭包 ──
work = pc.scratch("d2-trace-")
pc.write(os.path.join(work, "Cargo.toml"), '[workspace]\nmembers = ["crates/pkg"]\n')
pc.write(os.path.join(work, "crates/pkg/Cargo.toml"), '[package]\nname = "pkg"\nversion = "0.0.0"\nedition = "2021"\n')
pc.write(os.path.join(work, "crates/pkg/src/lib.rs"), "pub fn one() -> u32 { 1 }\n")
pc.write(os.path.join(work, "crates/pkg/tests/own_case.rs"), "#[test]\n#[ignore]\nfn the_case() {}\n")
pc.write(os.path.join(work, ".claude/gate.d/stage-inputs.tsv"),
         "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case count-line=LAYER0 count-line=CHECKER exhaustive=LAYER0 threads=LAYER0 shard=across-machines\t# 样本\n")
pc.run(["git", "init", "-q", "-b", "master", work])
manifest = os.path.join(work, ".manifest")
with contextlib.redirect_stdout(io.StringIO()) as captured:
    admission.main(["crash-case-manifest", work, "crash-case:own", manifest])
fingerprint = captured.getvalue().split()[0]
pc.write(os.path.join(work, "single.log"), admission.synthetic_layer0_log())
pc.write(os.path.join(work, "merge.log"), admission.synthetic_merge_log())
executed, referenced = set(), set()


def tracer(frame, event, _argument):
    code = frame.f_code
    if event == "call" and os.path.realpath(code.co_filename) == ADMISSION_REAL:
        top = code.co_qualname.split(".")[0]
        if top not in ("<module>", "main"):  # main 是摘要里原文进、不往下顺的那一格（CRASH_CASE_JUDGING_LEAVES），它引的 COMMANDS 只有判法那几项进摘要
            executed.add(top)
            referenced.update(name for name in code.co_names if name in definitions)
    return None


THREADS = ["--machine-cores", "32", "--threads", "32", "--threads-origin", "default"]
RUNS = [["crash-case-command", work, "crash-case:own", fingerprint],
        ["crash-case-judge", work, "crash-case:own", os.path.join(work, "single.log"), os.path.join(work, "judged"), *THREADS],
        ["crash-case-judge", work, "crash-case:own", os.path.join(work, "merge.log"), os.path.join(work, "judged"), *THREADS],
        ["crash-case-record", work, "crash-case:own", fingerprint, manifest, os.path.join(work, "judged"), "--files", "4", "--excluded", "0",
         "--started", "2026-09-27T00:00:00Z", "--judged-root", work, *THREADS],  # clock-times:allow 传给被测脚本的开跑时间戳参数
        ["crash-case-marker-check", work, "crash-case:own", fingerprint, manifest],
        ["crash-case-marker-path", work, "crash-case:own", fingerprint],
        ["crash-case-shardable", work, "crash-case:own"]]
exits = []
for arguments in RUNS:
    sys.settrace(tracer)
    try:
        with contextlib.redirect_stdout(io.StringIO()):
            exits.append(admission.main(arguments))
    finally:
        sys.settrace(None)
print(f"INFO\t七趟子命令的退出码 {exits}（command、judge 单机、judge merge、record、marker-check、marker-path、shardable；0 是走到了判绿 / 作数那一支）")
used = (executed | referenced) - {"main"}
outside = sorted(used - closure)
print(f"INFO\t运行时执行到的函数 {len(executed - {'main'})} 个、读到的模块级名字 {len(referenced)} 个；合起来 {len(used)} 个，其中闭包外的：{outside or '无'}")
cells.expect("ATTACK", "①今天：判法子命令运行时用到的模块级定义（main 之下）都在静态闭包里", not outside, f"闭包外的 {outside or '无'}；执行到的函数 {sorted(executed)}")


# ── ② 以后：ast 闭包看不见的写法，两步改 ──
def digest(text):
    return hashlib.sha256(admission.crash_case_judging_digest_text(text)[0]).hexdigest()[:16]


def replace_once(text, old, new):
    assert text.count(old) == 1, old[:70]
    return text.replace(old, new)


PASSED = 'PASSED_ONE_TEST_FORM = re.compile(r"^test result: ok\\. 1 passed; 0 failed; ")'
FIELDS = 'def fields_of_line(line):\n    return dict(token.split("=", 1) for token in line.split() if "=" in token)\n'
JUDGE_ROW = '    "crash-case-judge": command_crash_case_judge,\n'
STEPS = [
    ("元组解包赋值：PASSED_ONE_TEST_FORM 挪进 `A, B = …`，再把判绿的正则放宽成「test result: 」开头都算（FAILED 也判绿）",
     replace_once(source, PASSED, 'PASSED_ONE_TEST_FORM, _SPARE_FORM = re.compile(r"^test result: ok\\. 1 passed; 0 failed; "), None'),
     lambda text: replace_once(text, 're.compile(r"^test result: ok\\. 1 passed; 0 failed; "), None', 're.compile(r"^test result: "), None')),
    ("if 块里的 def：fields_of_line 挪进 `if True:` 块，再让它把 exhaustive=false 读成 true",
     replace_once(source, FIELDS, "if True:\n    def fields_of_line(line):\n        return dict(token.split(\"=\", 1) for token in line.split() if \"=\" in token)\n"),
     lambda text: replace_once(text, '        return dict(token.split("=", 1) for token in line.split() if "=" in token)\n',
                               '        return {**dict(token.split("=", 1) for token in line.split() if "=" in token), "exhaustive": "true"}\n')),
    ("分派表的值不是名字：crash-case-judge 那一项写成 lambda，再改 command_crash_case_judge 让它一律退 0",
     replace_once(source, JUDGE_ROW, '    "crash-case-judge": lambda arguments: command_crash_case_judge(arguments),\n'),
     lambda text: replace_once(text, "    problems, recorded_lines, thread_notes = judge_crash_case_log(case, log_text, int(values[\"--machine-cores\"]), threads_explicitly_one)\n",
                               "    problems, recorded_lines, thread_notes = judge_crash_case_log(case, log_text, int(values[\"--machine-cores\"]), threads_explicitly_one)\n    problems = []\n")),
    ("判法挪进 import 的别的模块：`from judging_helpers import fields_of_line`（再改那份模块，准入模块原文一个字不变）",
     replace_once(source, FIELDS, "from judging_helpers import fields_of_line  # noqa: E402\n"),
     lambda text: text),
]
for label, first, second_edit in STEPS:
    second = second_edit(first)
    d0, d1, d2 = digest(source), digest(first), digest(second)
    cells.expect("ATTACK", f"②以后：{label} ⇒ 第二步摘要应当变", d1 != d2, f"原样 {d0}… 第一步 {d1}… 第二步 {d2}…")

# ── ③ 54 号转给 crash-case-judge 的线程参数定判不判绿 ──
one_thread = os.path.join(work, "one-thread.log")
pc.write(one_thread, admission.synthetic_layer0_log(worker_threads=1))
pc.write(os.path.join(work, ".claude/gate.d/stage-inputs.tsv"),
         "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case count-line=LAYER0 count-line=CHECKER exhaustive=LAYER0 threads=LAYER0\t# 样本\n")
verdicts = {}
for label, threads in (("配的是 32（default）", ["--threads", "32", "--threads-origin", "default"]),
                       ("转成显式 1（explicit）", ["--threads", "1", "--threads-origin", "explicit"])):
    code, out, _ = pc.run([sys.executable, pc.ADMISSION, "crash-case-judge", work, "crash-case:own", one_thread, os.path.join(work, "judged3"),
                           "--machine-cores", "32", *threads])
    verdicts[label] = (code, out.strip().splitlines()[0][:90] if out.strip() else "")
same_log_differs = verdicts["配的是 32（default）"][0] == 1 and verdicts["转成显式 1（explicit）"][0] == 0
cells.expect("ATTACK", "③同一份「64 片只起 1 个线程」的日志：判不判绿只取决于 54 号转过来的 --threads / --threads-origin（54 号不进指纹）",
             not same_log_differs, "；".join(f"{label} 退 {code}（{line}）" for label, (code, line) in verdicts.items()))
sys.exit(cells.finish())
