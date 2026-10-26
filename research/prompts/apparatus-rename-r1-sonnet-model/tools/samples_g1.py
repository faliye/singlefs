"""G1 误放：登记表一行假搬迁（旧前缀磁盘上没有、新前缀是已有的别的目录）能不能把「改指到另一个目录」洗成「只差搬迁」。
用法：python3 samples_g1.py --repo <仓根> --work <新目录>。每个样本一个临时 git 仓，产物一行 name=… 。"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import git, load_library, make_repository, register_move, run_library, say, write  # noqa: E402

EXEMPT_ROW = "{path}:os.cpu_count()  # 这一处只量核数、不起线程，理由写够字数\n"


def exempt_keys(text):
    return sorted(line.split("  #")[0] for line in text.split("\n") if line and not line.startswith("#"))


def sample_exempt_table(library, library_path, work):
    """A1：排除表（路径:锚串）只缩不涨。基准里有 scripts/gone/runner.py 的一行；删掉 scripts/gone，表里改成已有的 scripts/gate/runner.py。"""
    root = os.path.join(work, "a1-exempt-table")
    make_repository(root, {".claude/heavy-thread-exempt": EXEMPT_ROW.format(path="scripts/gone/runner.py"),
                           "scripts/gone/runner.py": "os.cpu_count()\n", "scripts/gate/runner.py": "workers = os.cpu_count() * 2\n",
                           "records/2026-10-23-sample.md": "# 样本\n"})
    git(root, "rm", "-rq", "scripts/gone")
    write(root, ".claude/heavy-thread-exempt", EXEMPT_ROW.format(path="scripts/gate/runner.py"))
    base_without = run_library(library_path, root, "show", "HEAD", ".claude/heavy-thread-exempt")[1]
    current = open(os.path.join(root, ".claude/heavy-thread-exempt"), encoding="utf-8").read()
    say("a1-exempt-growth-without-registry", grown=[key for key in exempt_keys(current) if key not in exempt_keys(base_without)])
    register_move(root, "scripts/gone", "scripts/gate")
    code, check_out, _ = run_library(library_path, root, "path-moves-check", root)
    base_with = run_library(library_path, root, "show", "HEAD", ".claude/heavy-thread-exempt")[1]
    say("a1-registry-check", exit=code, out=check_out.strip())
    say("a1-exempt-growth-with-fiction-row", grown=[key for key in exempt_keys(current) if key not in exempt_keys(base_with)], base_after_rewrite=exempt_keys(base_with))


def sample_change_range(library, library_path, work):
    """A2：改动范围。crates 文件里 Path::new("scripts/gone/run.sh") 改成 "scripts/gate/run.sh"，同一行假登记之后是不是不再算改了。"""
    root = os.path.join(work, "a2-change-range")
    make_repository(root, {"crates/x/src/lib.rs": 'let p = "scripts/gone/run.sh";\nlet q = 1;\n', "scripts/gone/run.sh": "echo\n",
                           "scripts/gate/run.sh": "echo gate\n", "records/2026-10-23-sample.md": "# 样本\n"})
    git(root, "rm", "-rq", "scripts/gone")
    write(root, "crates/x/src/lib.rs", 'let p = "scripts/gate/run.sh";\nlet q = 1;\n')
    paths = "crates/x/src/lib.rs\n"
    say("a2-without-registry", moved_only_filter_kept=run_library(library_path, root, "moved-only-filter", "HEAD", data=paths)[1].split())
    register_move(root, "scripts/gone", "scripts/gate")
    say("a2-with-fiction-row", moved_only_filter_kept=run_library(library_path, root, "moved-only-filter", "HEAD", data=paths)[1].split())
    lines = "crates/x/src/lib.rs\tlet p = \"scripts/gate/run.sh\";\n"
    say("a2-added-line-with-fiction-row", moved_lines_filter_kept=run_library(library_path, root, "moved-lines-filter", "HEAD", data=lines)[1].split("\n")[:-1])


def sample_word_row(library, library_path, work):
    """A3：旧前缀不必是目录名——路径前缀的写法只禁空白、开头 / 与 ./、结尾 /、空段与 . .. 段，单个词也过。
    登记 deny → scripts（scripts 在磁盘上、deny 不在），一行 `exit deny` 改成 `exit scripts`。"""
    root = os.path.join(work, "a3-word-row")
    make_repository(root, {"hook.sh": "exit deny\n", "scripts/placeholder": "x\n", "records/2026-10-23-sample.md": "# 样本\n"})
    write(root, "hook.sh", "exit scripts\n")
    register_move(root, "deny", "scripts")
    code, check_out, _ = run_library(library_path, root, "path-moves-check", root)
    say("a3-registry-check", exit=code, out=check_out.strip())
    say("a3-moved-only-filter-kept", kept=run_library(library_path, root, "moved-only-filter", "HEAD", data="hook.sh\n")[1].split())


def sample_long_run_pin(repo, work, identical_copy=False):
    """A4：按「路径 + sha256」钉的豁免表挡不挡得住同一种假登记：基准里 scripts/old-rig/live.sh 有一行，删掉它，表里改成已有的另一份 scripts/other/live.sh。"""
    import subprocess
    sources = os.path.join(repo, ".claude/gate.d/fixtures/code-tooling.sh/long-run-header-exempt-moved-green/scripts/new-rig/live.sh")
    body = open(sources, encoding="utf-8").read()
    root = os.path.join(work, "a5-long-run-pin-identical-copy" if identical_copy else "a4-long-run-pin")
    import hashlib
    digest = lambda text: hashlib.sha256(text.encode()).hexdigest()  # noqa: E731
    other = body if identical_copy else body + "# 另一份脚本\n"
    make_repository(root, {"scripts/old-rig/live.sh": body, "scripts/other/live.sh": other, "records/2026-10-23-sample.md": "# 样本\n",
                           ".claude/long-run-live-hosts-exempt": f"scripts/old-rig/live.sh  {digest(body)}  # 样本：文件头按现算写、不能改\n"})
    git(root, "rm", "-rq", "scripts/old-rig")
    write(root, ".claude/long-run-live-hosts-exempt", f"scripts/other/live.sh  {digest(other)}  # 样本：文件头按现算写、不能改\n")
    register_move(root, "scripts/old-rig", "scripts/other")
    done = subprocess.run([sys.executable, "-I", os.path.join(repo, ".claude/hooks/lib_long_run.py"), "exempt-check", root, "--base", "HEAD"],
                          cwd=root, capture_output=True, text=True)
    say("a5-exempt-check-identical-copy" if identical_copy else "a4-exempt-check", exit=done.returncode, stderr=done.stderr.strip()[:900], stdout=done.stdout.strip()[:200])


def sample_duplicate_line(library_path, work):
    """A6：新增行逐条去抵「基准里因搬迁才变了的行」，不核基准那一行在此刻是不是真被换掉了：基准的旧路径行原样留着，再多写一行换成新前缀的拷贝，新增的那一行被剔。"""
    old, new = "scripts/experiments", "scripts/experiment-apparatus"
    root = os.path.join(work, "a6-duplicate-line")
    make_repository(root, {"docs/a.md": f"bash {old}/run.sh\n", f"{new}/run.sh": "echo\n", "records/2026-10-23-sample.md": "# 样本\n"})
    write(root, "docs/a.md", f"bash {old}/run.sh\nbash {new}/run.sh\n")
    register_move(root, old, new)
    lines = f"docs/a.md\tbash {new}/run.sh\n"
    say("a6-duplicate-line-dropped", added_line_kept=run_library(library_path, root, "moved-lines-filter", "HEAD", data=lines)[1].split("\n")[:-1],
        baseline_old_line_still_in_file=True)


def sample_old_prefix_still_on_disk(library_path, work):
    """A7：「旧前缀不该还在磁盘上」有个例外（后面的登记又搬进它的）。两行登记：scripts/gate → scripts/hosts（scripts/gate 此刻还在、还有真文件），
    再 scripts/zzz → scripts/gate/zzz（让第一行的旧前缀落进例外）；一条命令里的 scripts/gate 改成 scripts/hosts 被当成只差搬迁。"""
    root = os.path.join(work, "a7-old-prefix-on-disk")
    make_repository(root, {"docs/a.md": "bash scripts/gate/run.sh\n", "scripts/gate/run.sh": "echo gate\n", "scripts/hosts/run.sh": "echo hosts\n",
                           "scripts/gate/zzz/keep.txt": "x\n", "records/2026-10-23-sample.md": "# 样本\n"})
    write(root, "docs/a.md", "bash scripts/hosts/run.sh\n")
    write(root, ".claude/gate.d/path-moves.tsv", "scripts/gate\tscripts/hosts\t2026-10-23\trecords/2026-10-23-sample.md\nscripts/zzz\tscripts/gate/zzz\t2026-10-23\trecords/2026-10-23-sample.md\n")
    code, check_out, _ = run_library(library_path, root, "path-moves-check", root)
    say("a7-registry-check", exit=code, out=check_out.strip(), old_prefix_exists=os.path.exists(os.path.join(root, "scripts/gate/run.sh")))
    say("a7-moved-only-filter-kept", kept=run_library(library_path, root, "moved-only-filter", "HEAD", data="docs/a.md\n")[1].split())


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    arguments.repo, arguments.work = os.path.abspath(arguments.repo), os.path.abspath(arguments.work)
    library, library_path = load_library(arguments.repo)
    os.makedirs(arguments.work, exist_ok=True)
    for sample in (sample_exempt_table, sample_change_range, sample_word_row):
        sample(library, library_path, arguments.work)
    sample_duplicate_line(library_path, arguments.work)
    sample_old_prefix_still_on_disk(library_path, arguments.work)
    sample_long_run_pin(arguments.repo, arguments.work)
    sample_long_run_pin(arguments.repo, arguments.work, identical_copy=True)


if __name__ == "__main__":
    main()
