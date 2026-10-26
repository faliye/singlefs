#!/usr/bin/env python3
"""H2：主工作区只读现查。拿冻结树里被判的共用库算两样（真登记那一行的实据、改动范围里只差搬迁的），再用一条不共用它代码的路子各算一遍：
git cat-file --batch 取基准那一份、bytes.replace 整串换（不判前后边界、不跳提交引用），与工作区那一份逐字节比；两边的集合逐条对。
git 一律带 GIT_OPTIONAL_LOCKS=0（不顺手刷新索引、不写主仓的 .git）。
用法：python3 probe_main_repo.py --tree <冻结树> --repo <仓根> [--base <基准>]"""
import argparse
import importlib.util
import os
import subprocess
import sys

sys.dont_write_bytecode = True
OLD, NEW = "scripts/experiments", "scripts/experiment-apparatus"
ENV = dict(os.environ, GIT_OPTIONAL_LOCKS="0", LC_ALL="C.UTF-8")
for variable in ("LAG_BASELINE_BREAK", "GATE_BASE"):
    ENV.pop(variable, None)
    os.environ.pop(variable, None)
os.environ["GIT_OPTIONAL_LOCKS"] = "0"


def git(repo, *arguments, data=None):
    return subprocess.run(["git", "-c", "core.quotepath=false", *arguments], cwd=repo, env=ENV, capture_output=True, input=data, check=True).stdout


def blobs(repo, specs):
    output = git(repo, "cat-file", "--batch", data="".join(spec + "\n" for spec in specs).encode("utf-8"))
    found, offset = {}, 0
    for spec in specs:
        end = output.index(b"\n", offset)
        header = output[offset:end].split(b" ")
        if header[-1] == b"missing":
            found[spec], offset = None, end + 1
            continue
        size = int(header[2])
        found[spec], offset = output[end + 1:end + 1 + size], end + 1 + size + 1
    return found


def read(repo, path):
    full = os.path.join(repo, path)
    if os.path.islink(full) or not os.path.isfile(full):
        return None
    with open(full, "rb") as handle:
        return handle.read()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--tree", required=True)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--base")
    arguments = parser.parse_args()
    repo = os.path.abspath(arguments.repo)
    base = arguments.base or git(repo, "merge-base", "HEAD", "@{upstream}").decode().strip()
    spec = importlib.util.spec_from_file_location("lag_baseline_probe", os.path.join(arguments.tree, ".claude/gate.d/lib-lag-baseline.py"))
    lib = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(lib)
    print(f"name=probe.base base={base[:9]}")
    count, problems, note = lib.path_move_report(repo, base)
    print(f"name=probe.lib_evidence rows={count} problems={[what for what, _ in problems]!r} note={note!r}")
    subjects = [line for line in git(repo, "ls-tree", "-r", "--name-only", base, "--", OLD).decode().split("\n") if line]
    contents = blobs(repo, [f"{base}:{path}" for path in subjects])
    present = same = 0
    for path in subjects:
        now = read(repo, NEW + path[len(OLD):])
        present += now is not None
        same += now is not None and now == contents[f"{base}:{path}"].replace(OLD.encode(), NEW.encode())
    print(f"name=probe.independent_evidence subjects={len(subjects)} present={present} same_after_plain_replace={same}")
    listed = set(git(repo, "diff", "--name-only", base, "--").decode().split("\n")) | set(git(repo, "diff", "--name-only", "--cached").decode().split("\n"))
    listed |= set(git(repo, "ls-files", "--others", "--exclude-standard").decode().split("\n"))
    listed = sorted(path for path in listed if path)
    moved, problem = lib.moved_only_paths(base, listed, cwd=repo)
    tree = {line.split("\t", 1)[1] for line in git(repo, "ls-tree", "-r", base).decode().split("\n") if line}
    sources = {}
    for path in listed:
        before = path if path in tree else (OLD + path[len(NEW):] if path.startswith(NEW + "/") and OLD + path[len(NEW):] in tree else None)
        if before is not None:
            sources[path] = before
    contents = blobs(repo, sorted({f"{base}:{source}" for source in sources.values()}))
    independent = set()
    for path, source in sources.items():
        now = read(repo, path)
        if now is None and path.startswith(OLD + "/"):
            now = read(repo, NEW + path[len(OLD):])
        before = contents[f"{base}:{source}"]
        if now is not None and before is not None and (now == before or now == before.replace(OLD.encode(), NEW.encode())):
            independent.add(path)
    print(f"name=probe.change_range listed={len(listed)} lib_moved_only={len(moved)} independent_moved_only={len(independent)} problem={problem!r}")
    print(f"name=probe.only_lib count={len(moved - independent)} first={sorted(moved - independent)[:8]!r}")
    print(f"name=probe.only_independent count={len(independent - moved)} first={sorted(independent - moved)[:8]!r}")
    governed = sorted(path for path in moved if lib.governed_class(path))
    print(f"name=probe.lib_moved_only_governed count={len(governed)} paths={governed!r}")
    # 真登记那一行当对照：几种实现（被判的、补了最终落点的、去掉占用核对的、整个实据不核的）在它上面各判一次
    original = lib.move_evidence

    def without_occupied(root, base_commit, moves):
        rows, found, text = original(root, base_commit, moves)
        return rows, [item for item in found if "本来就在的文件" not in item[0]], text

    def unchecked(root, base_commit, moves):
        return 0, [], "实据不核"

    for label, function in (("judged", original), ("no_occupied_check", without_occupied), ("evidence_unchecked", unchecked)):
        lib.move_evidence = function
        count, problems, note = lib.path_move_report(repo, base)
        print(f"name=probe.real_row_control.{label} problems={len(problems)} note={note[:60]!r}")
    lib.move_evidence = original


if __name__ == "__main__":
    main()
