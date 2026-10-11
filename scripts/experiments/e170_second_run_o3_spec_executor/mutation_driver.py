"""单测层变异：在执行器目录的副本上逐条替换、编译、跑单测、还原（副本每条重拷），三栏：抓到 / 无效 / 没红。
用法：python3 mutation_driver.py <执行器目录> <变异表> <草稿目录> —— 表三列：名字、原文、替换文（\\n 表示换行）；
草稿目录下要先有 testscratch 子目录（单测在那里建临时目录）。原文在执行器目录的 .py 里必须恰好命中一次，否则记 anchor_error、不跑那一条。
第 1 次重跑第一段只写了单测层（只跑 53 条单测）；批态与坏状态上的判定要第三段再接。"""
import os
import py_compile
import shutil
import subprocess
import sys

source_directory, table_path, scratch = sys.argv[1], sys.argv[2], sys.argv[3]
python_files = sorted(f for f in os.listdir(source_directory) if f.endswith(".py"))
rows = []
for line in open(table_path, encoding="utf-8").read().splitlines():
    if not line or line.startswith("#"):
        continue
    name, original, replacement = line.split("\t")
    rows.append((name, original.replace("\\n", "\n"), replacement.replace("\\n", "\n")))
counts = {"caught": 0, "invalid": 0, "survived": 0, "anchor_error": 0}
for name, original, replacement in rows:
    work = os.path.join(scratch, "mutation-copy")
    shutil.rmtree(work, ignore_errors=True)
    shutil.copytree(source_directory, work, ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))
    hits = []
    for file_name in python_files:
        text = open(os.path.join(work, file_name), encoding="utf-8").read()
        if original in text:
            hits.append((file_name, text.count(original)))
    if sum(count for _f, count in hits) != 1:
        print(f"anchor_error [{name}] hits={hits}")
        counts["anchor_error"] += 1
        continue
    file_name = hits[0][0]
    path = os.path.join(work, file_name)
    text = open(path, encoding="utf-8").read()
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text.replace(original, replacement))
    try:
        py_compile.compile(path, doraise=True, cfile=os.path.join(scratch, "mutation-compile-check.pyc"))
    except py_compile.PyCompileError as error:
        print(f"invalid [{name}] file={file_name} compile_error={str(error)[:120]!r}")
        counts["invalid"] += 1
        continue
    environment = dict(os.environ, E170_TEST_SCRATCH=os.path.join(scratch, "testscratch"), PYTHONDONTWRITEBYTECODE="1")
    result = subprocess.run([sys.executable, "-m", "unittest", "test_executor_on_synthetic_pool"], cwd=work, env=environment,
                            capture_output=True, text=True)
    summary = [line for line in result.stderr.splitlines() if line.startswith(("FAILED", "OK", "Ran "))]
    failing = sorted({line.split()[1] for line in result.stderr.splitlines() if line.startswith(("FAIL:", "ERROR:"))})
    if result.returncode == 0:
        print(f"survived [{name}] file={file_name} {summary}")
        counts["survived"] += 1
    else:
        print(f"caught [{name}] file={file_name} {summary} first_red={failing[:3]} red_count={len(failing)}")
        counts["caught"] += 1
shutil.rmtree(os.path.join(scratch, "mutation-copy"), ignore_errors=True)
print(f"counts rows={len(rows)} " + " ".join(f"{key}={value}" for key, value in counts.items()))
