#!/usr/bin/env python3
"""上一轮及更早的实验记录归档进版本库：删掉工作区里的那批，保留本轮的。

规则见 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」之下那三段：
冻结只在本轮内有效；这一轮提交之后，下一次提交把上一次留下的那批删掉；本次的照旧留着，本次的验证照旧做得了；
要查删掉的那些，去版本库的历史里找。

用法：
  archive-past-rounds.py --check          还留着上一轮的记录就判红（门禁用）
  archive-past-rounds.py --apply          删掉；同时把指向被删文件的引用改成不带路径的说法
  archive-past-rounds.py --selftest

判据（机械，不靠人记）：
  删 —— `research/results/` 与 `research/prompts/` 下，**在 HEAD 里有、且工作区没改过**的文件。
        没改过 = 它是上一次提交固化下来的，属于上一轮及更早。
  留 —— 工作区里新加或改过的（本轮在产生的）、三方判决 `*-main-verification.md`（kb 的依据指着它）、
        `abandoned-rounds.tsv`（登记表，门禁 66 号的输入）。
  ⚠️ 删文件会让别处指向它的链接指空，所以 --apply 同时把那些引用**只留文件名、去掉路径**，
     不留一个指空的路径——门禁 23 号判的就是这个。「找不到就去 git 历史里看」写在 `.claude/agent-common.md`，不逐处重复。
"""
import os
import re
import subprocess
import sys

DIRS = ("research/results", "research/prompts")
KEEP = re.compile(r"-main-verification\.md$|abandoned-rounds\.tsv$")
LINK = re.compile(r"research/(?:results|prompts)/[^\s`)（）\]\n，。、；]+")
SCAN = (".claude/kb", "records", ".claude/rules", "briefs")


def git(*args, root="."):
    return subprocess.run(["git", "-c", "core.quotepath=false", *args], cwd=root,
                          capture_output=True, text=True, check=True).stdout


def past_round_files(root="."):
    tracked = [p for p in git("ls-files", *DIRS, root=root).splitlines() if p]
    dirty = set(p for p in git("diff", "--name-only", "HEAD", "--", *DIRS, root=root).splitlines() if p)
    return sorted(p for p in tracked if p not in dirty and not KEEP.search(p))


def text_files(root="."):
    for base in SCAN:
        for directory, _, names in os.walk(os.path.join(root, base)):
            for name in names:
                if name.endswith((".md", ".tsv")):
                    yield os.path.relpath(os.path.join(directory, name), root)


def rewrite_links(root, doomed):
    """把指向被删文件的引用改成不带路径的说法，别留指空的路径。"""
    doomed = set(doomed)
    changed = 0
    touched = 0
    for relative in text_files(root):
        path = os.path.join(root, relative)
        try:
            text = open(path, encoding="utf-8").read()
        except (UnicodeDecodeError, OSError):
            continue
        hits = 0

        def fix(match):
            nonlocal hits
            target = match.group(0)
            if target not in doomed:
                return target
            hits += 1
            # 只留文件名，不留路径、也不逐处写「已归档」——那句话在 .claude/agent-common.md 里写一处就够
            return os.path.basename(target)

        new = LINK.sub(fix, text)
        if hits:
            # 反引号里的路径换掉之后那对反引号就多余了，留着不碍事；括注重复的收一下
            open(path, "w", encoding="utf-8").write(new)
            changed += hits
            touched += 1
    return changed, touched


def run(root, apply_changes):
    doomed = past_round_files(root)
    if not doomed:
        # 数磁盘上真有的，不数 git 索引：删过还没提交时索引里仍跟踪着那些文件，
        # 拿索引数报「现有多少份」会报出一个磁盘上不成立的数。
        kept = sum(len(names) for base in DIRS for _, _, names in os.walk(os.path.join(root, base)))
        print("  ✓ 没有上一轮留下的实验记录（%s 下磁盘上现有 %d 份，都是本轮的或按规则保留的）" % ("、".join(DIRS), kept))
        print("     保留的两类：三方判决 *-main-verification.md（kb 的依据指着它）、abandoned-rounds.tsv（门禁 66 号的输入）")
        return 0
    if not apply_changes:
        by_dir = {}
        for p in doomed:
            by_dir[p.split("/")[1]] = by_dir.get(p.split("/")[1], 0) + 1
        for name, count in sorted(by_dir.items(), key=lambda item: -item[1]):
            print("      research/%s/  %d 份" % (name, count))       # gate-lint:detail
        print("  ✗ 还留着上一轮及更早的实验记录 %d 份（本轮的与判决不算）" % len(doomed))
        print("     → 怎么办：跑 python3 research/scripts/archive-past-rounds.py --apply 删掉，")
        print("               它同时把别处指向这些文件的引用改成只留文件名、不留路径；")
        print("               要查删掉的内容去 git 历史里找，有疑问就重新验证、不翻旧证据。")
        return 1
    changed, touched = rewrite_links(root, doomed)
    for relative in doomed:
        os.remove(os.path.join(root, relative))
    print("  ✓ 删了 %d 份上一轮及更早的实验记录；%d 份文件里 %d 处引用去掉了路径、只留文件名" % (len(doomed), touched, changed))
    print("     → 下一步：跑门禁 23 号（文档指向）确认没有指空的链接，再跑一次本脚本 --check 判绿。")
    return 0


def selftest():
    import tempfile
    with tempfile.TemporaryDirectory() as work:
        os.makedirs(os.path.join(work, "research/results"))
        os.makedirs(os.path.join(work, "research/prompts"))
        os.makedirs(os.path.join(work, ".claude/kb"))
        git("init", "-q", root=work)
        subprocess.run(["git", "config", "user.email", "t@t"], cwd=work, check=True)
        subprocess.run(["git", "config", "user.name", "t"], cwd=work, check=True)
        open(os.path.join(work, "research/results/e1-old.out"), "w").write("old\n")
        open(os.path.join(work, "research/prompts/r1-main-verification.md"), "w").write("判决\n")
        open(os.path.join(work, ".claude/kb/page.md"), "w").write(
            "产物 `research/results/e1-old.out`，判决 `research/prompts/r1-main-verification.md`\n")
        git("add", "-A", root=work)
        subprocess.run(["git", "commit", "-qm", "round one"], cwd=work, check=True)
        open(os.path.join(work, "research/results/e2-new.out"), "w").write("new\n")
        if run(work, False) != 1:
            print("  ✗ 自检失败：有上一轮记录时 --check 没判红")
            print("     → 怎么办：看 past_round_files() 的 ls-files 与 diff 组合。")
            return 1
        run(work, True)
        if os.path.exists(os.path.join(work, "research/results/e1-old.out")):
            print("  ✗ 自检失败：上一轮的产物没删掉")
            print("     → 怎么办：看 run() 的 os.remove 那一段。")
            return 1
        if not os.path.exists(os.path.join(work, "research/prompts/r1-main-verification.md")):
            print("  ✗ 自检失败：判决被删了，它该留")
            print("     → 怎么办：看 KEEP 那条正则。")
            return 1
        if not os.path.exists(os.path.join(work, "research/results/e2-new.out")):
            print("  ✗ 自检失败：本轮新产生的被删了")
            print("     → 怎么办：未跟踪的文件不在 ls-files 里，看 past_round_files() 为什么把它算进去了。")
            return 1
        page = open(os.path.join(work, ".claude/kb/page.md"), encoding="utf-8").read()
        if "research/results/e1-old.out" in page:
            print("  ✗ 自检失败：指向被删文件的引用还留着指空的路径")
            print("     → 怎么办：看 rewrite_links() 的 LINK 正则与 doomed 集合。")
            return 1
        if "research/prompts/r1-main-verification.md" not in page:
            print("  ✗ 自检失败：指向判决的引用被误改了，判决没删就不该改它的链接")
            print("     → 怎么办：rewrite_links() 只改 doomed 里的路径，看它为什么碰了别的。")
            return 1
        if run(work, False) != 0:
            print("  ✗ 自检失败：删完 --check 仍判红")
            print("     → 怎么办：看 past_round_files() 删除之后还认出了什么。")
            return 1
    print("  ✓ 自检：有旧记录判红、删完判绿、判决与本轮新产物不被删、指向被删文件的引用改成不带路径的说法")
    return 0


def main(argv):
    if "--selftest" in argv:
        return selftest()
    positional = [a for a in argv if not a.startswith("--")]
    root = os.path.abspath(positional[0]) if positional else os.path.abspath(
        os.path.join(os.path.dirname(__file__), "..", ".."))
    return run(root, "--apply" in argv)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
