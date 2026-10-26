"""G2 读法一致：同一份读法库里的几处（正着换、倒着换、路径一级、文本一级、串起来的次序、写坏的登记表）对同一个输入是不是同判。
用法：python3 samples_g2.py --repo <仓根> --work <新目录>。产物一行 name=… 。"""
import argparse
import hashlib
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import git, load_library, make_repository, run_library, say, write  # noqa: E402

OLD, NEW = "scripts/experiments", "scripts/experiment-apparatus"
CASES = [
    ("plain", "bash scripts/experiments/a.sh"), ("dot-slash", "bash ./scripts/experiments/a.sh"), ("root-var", "bash $ROOT/scripts/experiments/a.sh"),
    ("absolute", "bash /home/u/code/singlefs/scripts/experiments/a.sh"), ("nested-other-dir", "bash tests/fixtures/scripts/experiments/a.sh"),
    ("sibling-name", "scripts/experiments2/a.sh"), ("dash-suffix", "scripts/experiments-old/a.sh"), ("dot-md", "scripts/experiments.md"),
    ("sentence-end", "见 scripts/experiments。"), ("trailing-dot-space", "放在 scripts/experiments. 下"), ("quote", "'scripts/experiments'"),
    ("diff-minus", "-scripts/experiments/a.sh"), ("diff-plus", "+scripts/experiments/a.sh"), ("default-expansion", "${X:-scripts/experiments}"),
    ("after-newline-escape", "echo \\nscripts/experiments/a"), ("after-tab-escape", "echo \\tscripts/experiments/a"),
    ("split-literal", '"scripts", "experiments"'), ("commit-ref", "git show 31e47115:scripts/experiments/a.sh"),
    ("commit-ref-dot-slash", "git show 31e47115:./scripts/experiments/a.sh"), ("short-hex-is-not-ref", "git show 31e471:scripts/experiments/a.sh"),
    ("hex-word-defaced", "defaced:scripts/experiments/a.sh"), ("hex-41-before-colon", "a" * 41 + ":scripts/experiments/a.sh"),
    ("sha256-before-colon", hashlib.sha256(b"x").hexdigest() + ":scripts/experiments/a.sh"), ("hex-glued-letter", "g31e47115:scripts/experiments/a.sh"),
    ("two-on-one-line", "cp scripts/experiments/a scripts/experiments/b"), ("old-and-new-both", f"mv {OLD} {NEW}"),
]


def sample_boundary_matrix(library):
    forward = [library.PathMove(1, OLD, NEW, "2026-10-23", "records/x.md")]
    backward = [library.PathMove(1, NEW, OLD, "2026-10-23", "records/x.md")]
    for name, text in CASES:
        changed = library.rewrite_moved_paths(text, forward)
        # 倒着换：把正着换的结果（或原文里已有的新前缀写法）倒回去，看边界是不是对称
        mirrored = text.replace(OLD, NEW)
        say("b1-forward", case=name, text=text[:70], result=changed[:90], replaced=changed != text)
        say("b1-backward-of-mirrored", case=name, text=mirrored[:70], result=library.rewrite_moved_paths(mirrored, backward)[:90],
            reverted=library.rewrite_moved_paths(mirrored, backward) != mirrored)


def sample_roundtrip(library):
    """B2：倒着换不是单射——正着换的结果、留着一处没换的混合写法，倒着换出同一份。"""
    forward = [library.PathMove(1, OLD, NEW, "2026-10-23", "records/x.md")]
    backward = [library.PathMove(1, NEW, OLD, "2026-10-23", "records/x.md")]
    pinned = "run scripts/experiments/a.sh\nrun scripts/experiments/b.sh\n"
    fully_moved = library.rewrite_moved_paths(pinned, forward)
    mixed = fully_moved.replace(NEW + "/b.sh", OLD + "/b.sh")
    digest = lambda text: hashlib.sha256(text.encode()).hexdigest()[:16]  # noqa: E731
    say("b2-backward-not-injective", pinned=digest(pinned), fully_moved_back=digest(library.rewrite_moved_paths(fully_moved, backward)),
        mixed_back=digest(library.rewrite_moved_paths(mixed, backward)), fully_moved_differs_from_mixed=fully_moved != mixed)


def sample_nested_prefix(library_path, work):
    """B3：文本一级认嵌在别的目录底下的同名段，路径一级只认仓根起的前缀。夹具目录 fixtures/g/scripts/experiments 改名成 …/experiment-apparatus。"""
    root = os.path.join(work, "b3-nested")
    make_repository(root, {"fixtures/g/scripts/experiments/replay.sh": "bash fixtures/g/scripts/experiments/e1.sh\n",
                           "docs/a.md": "见 fixtures/g/scripts/experiments/e1.sh\n", "scripts/experiments/top.sh": "echo\n", "records/2026-10-23-sample.md": "# 样本\n"})
    git(root, "mv", "fixtures/g/scripts/experiments", "fixtures/g/scripts/experiment-apparatus")
    write(root, "fixtures/g/scripts/experiment-apparatus/replay.sh", "bash fixtures/g/scripts/experiment-apparatus/e1.sh\n")
    write(root, "docs/a.md", "见 fixtures/g/scripts/experiment-apparatus/e1.sh\n")
    git(root, "mv", "scripts/experiments", "scripts/experiment-apparatus")
    write(root, ".claude/gate.d/path-moves.tsv", f"{OLD}\t{NEW}\t2026-10-23\trecords/2026-10-23-sample.md\n")
    kept = run_library(library_path, root, "moved-only-filter", "HEAD", data="fixtures/g/scripts/experiment-apparatus/replay.sh\ndocs/a.md\nscripts/experiment-apparatus/top.sh\n")[1].split()
    say("b3-nested-prefix", kept_as_changed=kept, meaning="docs/a.md 是文本一级换算后相同被剔；夹具目录里被改名的 replay.sh 路径一级认不出被留下")


def sample_chains(library, library_path, work):
    """B4：串起来的次序。登记 A→B 再 B→C：基准里 A 的文件落在 C；基准里的 B 文本也换到 C。"""
    root = os.path.join(work, "b4-chain")
    make_repository(root, {"scripts/a/x.sh": "bash scripts/a/x.sh\n", "scripts/b/y.sh": "bash scripts/b/y.sh\n", "records/2026-10-23-sample.md": "# 样本\n"})
    git(root, "mv", "scripts/a", "scripts/b2")
    git(root, "mv", "scripts/b", "scripts/c")
    git(root, "mv", "scripts/b2", "scripts/b")
    git(root, "mv", "scripts/b", "scripts/c/b")
    write(root, ".claude/gate.d/path-moves.tsv", "scripts/b\tscripts/c\t2026-10-20\trecords/2026-10-23-sample.md\nscripts/a\tscripts/b\t2026-10-23\trecords/2026-10-23-sample.md\n")
    moves, problems = library.path_moves(root)
    say("b4-chain-moves", moves=[(m.old, m.new) for m in moves], problems=problems)
    say("b4-paths-before-moves", path="scripts/b/x.sh", candidates=library.paths_before_moves("scripts/b/x.sh", moves),
        path_c="scripts/c/y.sh", candidates_c=library.paths_before_moves("scripts/c/y.sh", moves))
    say("b4-rewrite-order", text="scripts/a/x scripts/b/y", result=library.rewrite_moved_paths("scripts/a/x scripts/b/y", moves))


def sample_broken_registry(library, library_path, work):
    """B5：写坏的登记表（三列）各处怎么反应：baseline_text 交 PathMovesProblem（调用方红）、moved-only-filter 退 0 提示一行并原样交回、shell 一层 grep 当有内容行。"""
    root = os.path.join(work, "b5-broken")
    make_repository(root, {"a.txt": "scripts/old/x\n", ".claude/gate.d/path-moves.tsv": "scripts/old\tscripts/new\t2026-10-23\n"})
    text, problem = library.baseline_text("HEAD", "a.txt", cwd=root)
    code, out, err = run_library(library_path, root, "moved-only-filter", "HEAD", data="a.txt\n")
    import subprocess
    grep = subprocess.run(["bash", "-c", "grep -qv -e '^[[:space:]]*#' -e '^[[:space:]]*$' .claude/gate.d/path-moves.tsv"], cwd=root).returncode
    say("b5-broken-registry", baseline_text_problem=type(problem).__name__, filter_exit=code, filter_kept=out.split(), filter_stderr=err.strip()[:120], shell_sees_content_row=grep == 0)


def sample_half_moved(repo, library_path, work):
    """B6：同一份「只搬了一半」的内容，倒着换的钉值核对（reuse_library.pinned_sha256_now）认，正着换的豁免表核对（lib_long_run exempt-check）不认。"""
    import importlib.util
    import subprocess
    directory = os.path.join(repo, "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes")
    sys.path.insert(0, directory)
    specification = importlib.util.spec_from_file_location("reuse_library_under_test", os.path.join(directory, "reuse_library.py"))
    reuse_library = importlib.util.module_from_spec(specification)
    sys.modules["reuse_library_under_test"] = reuse_library
    specification.loader.exec_module(reuse_library)
    fixture = os.path.join(repo, ".claude/gate.d/fixtures/code-tooling.sh/long-run-header-exempt-moved-green/scripts/new-rig/live.sh")
    after_move = open(fixture, encoding="utf-8").read()
    before_move = after_move.replace("scripts/new-rig", "scripts/old-rig")
    first = after_move.index("scripts/new-rig")
    half = before_move[:first] + "scripts/new-rig" + before_move[first + len("scripts/old-rig"):]
    digest = lambda text: hashlib.sha256(text.encode()).hexdigest()  # noqa: E731
    pins = os.path.join(work, "b6-pins")
    write(pins, ".claude/gate.d/path-moves.tsv", "scripts/old-rig\tscripts/new-rig\t2026-10-23\trecords/2026-10-23-sample.md\n")
    verdicts = {}
    for label, text in (("fully-moved", after_move), ("half-moved", half), ("not-moved", before_move)):
        write(pins, "file.sh", text)
        verdicts[label] = reuse_library.pinned_sha256_now(os.path.join(pins, "file.sh"), digest(before_move), pins) is not None
    say("b6-pinned-sha256-now-accepts", occurrences_of_old_prefix=before_move.count("scripts/old-rig"), **verdicts)
    root = os.path.join(work, "b6-exempt")
    make_repository(root, {"scripts/old-rig/live.sh": before_move, "records/2026-10-23-sample.md": "# 样本\n",
                           ".claude/long-run-live-hosts-exempt": f"scripts/old-rig/live.sh  {digest(before_move)}  # 样本：文件头按现算写、不能改\n"})
    git(root, "mv", "scripts/old-rig", "scripts/new-rig")
    write(root, ".claude/gate.d/path-moves.tsv", "scripts/old-rig\tscripts/new-rig\t2026-10-23\trecords/2026-10-23-sample.md\n")
    for label, text in (("fully-moved", after_move), ("half-moved", half)):
        write(root, "scripts/new-rig/live.sh", text)
        write(root, ".claude/long-run-live-hosts-exempt", f"scripts/new-rig/live.sh  {digest(text)}  # 样本：文件头按现算写、不能改\n")
        done = subprocess.run([sys.executable, "-I", os.path.join(repo, ".claude/hooks/lib_long_run.py"), "exempt-check", root, "--base", "HEAD"], cwd=root, capture_output=True, text=True)
        say("b6-exempt-check", variant=label, exit=done.returncode, stderr_tail=done.stderr.strip().split("\n")[1][-160:] if done.returncode else "")


def sample_retained_old_line(library, work):
    """B7：改名提交落地以后基准里已经是新路径，登记表那一行还在：基准里故意留旧路径的行（讲改名本身的）被正着换成新路径，与此刻没动的同一行对不上。"""
    root = os.path.join(work, "b7-retained")
    line = f"- 记改名：{OLD} 改名为 {NEW}\n"
    make_repository(root, {"hist.md": line, ".claude/gate.d/path-moves.tsv": f"{OLD}\t{NEW}\t2026-10-23\trecords/2026-10-23-sample.md\n",
                           "records/2026-10-23-sample.md": "# 样本\n"})
    text, problem = library.baseline_text("HEAD", "hist.md", cwd=root)
    current = open(os.path.join(root, "hist.md"), encoding="utf-8").read()
    say("b7-retained-old-line", baseline_after_rewrite=text, current_unchanged=current, equal=text == current, problem=problem)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    repo, work = os.path.abspath(arguments.repo), os.path.abspath(arguments.work)
    library, library_path = load_library(repo)
    os.makedirs(work, exist_ok=True)
    sample_boundary_matrix(library)
    sample_roundtrip(library)
    sample_nested_prefix(library_path, work)
    sample_chains(library, library_path, work)
    sample_broken_registry(library, library_path, work)
    sample_half_moved(repo, library_path, work)
    sample_retained_old_line(library, work)


if __name__ == "__main__":
    main()
