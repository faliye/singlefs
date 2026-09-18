#!/usr/bin/env python3
"""按搬迁登记表（research/scripts/path-moves.tsv）把全仓指向旧路径的地方改成新路径，或者只查还剩几处。

用法：
    rewrite-moved-paths.py --check [仓库根]    # 只查：全仓还有几处指向登记过的旧路径，有就退出码 1、逐处列出
    rewrite-moved-paths.py --apply [仓库根] [--skip 文件 …]   # 改写：Markdown 链接按文件自己的位置重算相对路径，其余写法按前缀改，改完回读；
                                                          # --skip 跳过别的会话正在写的文件（它自己改），跳过的照样在 --check 里报
    rewrite-moved-paths.py --selftest          # 自证：造一个带旧路径的小仓，--check 必须判红、--apply 之后必须判绿

为什么要有它：`.claude/rules/path-moves.md`——文件与目录的路径不在冻结范围内。2026-09-17 把两份字节布局表搬进
`.claude/kb/layout/` 时只改了现行文件，`research/prompts/` 下 200 多份材料与 E142 的留存产物仍指旧路径，
而 E142 的装置源码里那条字符串已经改了，门禁 87 号的逐字复跑因此对不上。

范围：仓库里全部文本文件（`.git`、各处 `target/`、登记表自己除外），`research/prompts/`、`research/results/`、`records/`、
历史版本节都在内。说搬迁这件事本身的那一行不改、不报：同一行里旧、新两条路径都出现，或带着「改前」「搬到」「搬进」「搬迁」「改名为」这类记变更的词（变更史里「改前：链接指向旧路径」照旧是真话）。

退出码：0 通过 / 写成；1 --check 发现旧路径；2 参数或登记表问题；3 改写之后回读仍有旧路径。
"""
import os
import pathlib
import posixpath
import re
import sys
import tempfile

REGISTRY = "research/scripts/path-moves.tsv"
TEXT_SUFFIXES = (".md", ".sh", ".py", ".rs", ".tsv", ".toml", ".json", ".txt", ".out", ".log", ".diff", ".patch", ".yaml", ".yml")
LINK = re.compile(r"\]\(([^)\s]+)\)")


def load_moves(root: pathlib.Path):
    path = root / REGISTRY
    if not path.is_file():
        raise SystemExit(f"  ✗ 找不到搬迁登记表 {path}\n     → 怎么办：登记表住在 {REGISTRY}，每行「旧路径<TAB>新路径<TAB>日期<TAB>依据」。")
    moves = []
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 4 or not fields[0].startswith(".claude/") and not fields[0].startswith("research/") and not fields[0].startswith("crates/") and not fields[0].startswith("records/"):
            print(f"  ✗ {REGISTRY}:{number} 不是四列、或旧路径不是从仓库根写的：{line}")
            print("     → 怎么办：四列用制表符分隔，旧路径与新路径都从仓库根写（例 .claude/kb/x.md）。")
            raise SystemExit(2)
        moves.append((fields[0], fields[1]))
    return moves


def forms(old: str, new: str):
    """一条搬迁在正文里的几种写法，长的先换，免得短的先把长的吃掉一半。"""
    pairs = [(old, new)]
    for prefix in (".claude/kb/", ".claude/"):
        if old.startswith(prefix) and new.startswith(prefix):
            pairs.append((old[len(prefix):], new[len(prefix):]))
    if old.startswith(".claude/kb/"):
        pairs.append(("kb/" + old[len(".claude/kb/"):], "kb/" + new[len(".claude/kb/"):]))
    old_base = posixpath.basename(old)
    new_from_kb = new[len(".claude/kb/"):] if new.startswith(".claude/kb/") else new
    pairs.append((old_base, new_from_kb))
    if old_base.endswith(".md"):
        pairs.append((old_base[:-3], new_from_kb[:-3]))
    seen = set()
    ordered = []
    for pair in sorted(pairs, key=lambda item: -len(item[0])):
        if pair[0] in seen or is_tail_of_new_path(pair[0], new):
            continue
        seen.add(pair[0])
        ordered.append(pair)
    return ordered


def is_tail_of_new_path(old_form: str, new: str) -> bool:
    """旧写法恰好是新路径（或去掉 .md 的新路径）按路径段切出来的末尾一截：文件名没变的搬迁里，光写文件名分不出指旧文件还是新文件，
    拿它改写会把已经是新路径的文字再换一遍（2026-09-18 实测：`.claude/agent-common.md` 搬进 `.claude/agents/` 时改出
    `.claude/agents/agents/.claude/agents/agent-common.md`），拿它检查会把指新文件的地方永远判成旧路径。这种写法两边都不用。"""
    if ("/" + new).endswith("/" + old_form):
        return True
    return new.endswith(".md") and ("/" + new[:-3]).endswith("/" + old_form)


MOVE_WORDS = ("改前", "搬到", "搬进", "搬迁", "改名为", "改名成")


def statement_line(line: str, old: str, new: str) -> bool:
    """说搬迁这件事本身的那一行，不改：同一行里旧、新路径都在（任一写法），或者带着「改前」「搬到」这类记变更的词。"""
    old_hit = any(o in line for o, _ in forms(old, new))
    if not old_hit:
        return False
    new_base = posixpath.basename(new)
    # 文件名没变的搬迁，「文件名 + 上一级目录名都在这一行」分不出这一行说的是搬迁还是只是同时提到了那个目录（2026-09-18 实测：
    # 「`.claude/agents/*.md`、`.claude/agent-common.md`」被当成说搬迁的一行放过），只认整条新路径。
    same_file_name = posixpath.basename(old) == new_base
    both_paths = new in line or (not same_file_name and new_base in line and posixpath.basename(posixpath.dirname(new)) in line)
    return both_paths or any(word in line for word in MOVE_WORDS)


def rewrite_text(text: str, relative_file: str, moves):
    here = posixpath.dirname(relative_file)
    changed = 0
    out_lines = []
    for line in text.split("\n"):
        skip = {index for index, (old, new) in enumerate(moves) if statement_line(line, old, new)}

        def fix_link(match):
            nonlocal changed
            target = match.group(1)
            if re.match(r"^[a-z]+:", target) or target.startswith("#") or target.startswith("/"):
                return match.group(0)
            path, sep, anchor = target.partition("#")
            resolved = posixpath.normpath(posixpath.join(here, path)) if path else ""
            for index, (old, new) in enumerate(moves):
                if index in skip:
                    continue
                if resolved == old:
                    changed += 1
                    return "](" + posixpath.relpath(new, here or ".") + (sep + anchor if sep else "") + ")"
            return match.group(0)

        line = LINK.sub(fix_link, line)
        # 全部写法合成一个正则一趟换完：逐种写法分开换时，前一种换出来的新路径会被后一种当旧写法再换一遍。
        replacement_by_old_form = {}
        for index, (old, new) in enumerate(moves):
            if index in skip:
                continue
            for old_form, new_form in forms(old, new):
                replacement_by_old_form.setdefault(old_form, new_form)
        if replacement_by_old_form:
            alternation = "|".join(re.escape(old_form) for old_form in sorted(replacement_by_old_form, key=len, reverse=True))
            pattern = re.compile(r"(?<![A-Za-z0-9_.-])(" + alternation + r")(?![A-Za-z0-9_-])")
            line, count = pattern.subn(lambda found: replacement_by_old_form[found.group(1)], line)
            changed += count
        out_lines.append(line)
    return "\n".join(out_lines), changed


def remaining(text: str, moves):
    hits = []
    for number, line in enumerate(text.split("\n"), 1):
        for old, new in moves:
            if statement_line(line, old, new):
                continue
            for old_form, _ in forms(old, new):
                if re.search(r"(?<![A-Za-z0-9_.-])" + re.escape(old_form) + r"(?![A-Za-z0-9_-])", line):
                    hits.append((number, old_form))
                    break
    return hits


def text_files(root: pathlib.Path, skip=()):
    for directory, subdirectories, files in os.walk(root):
        relative_directory = pathlib.Path(directory).relative_to(root).as_posix()
        subdirectories[:] = [s for s in subdirectories if s not in (".git", "target", "node_modules", "__pycache__")]
        for name in files:
            relative = name if relative_directory == "." else f"{relative_directory}/{name}"
            if relative == REGISTRY or relative == "research/scripts/rewrite-moved-paths.py" or relative in skip:
                continue
            if name.endswith(TEXT_SUFFIXES):
                yield relative


def run(root: pathlib.Path, apply: bool, skip=()) -> int:
    moves = load_moves(root)
    scanned = 0
    findings = []
    rewritten = 0
    for relative in text_files(root, skip if apply else ()):
        path = root / relative
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        scanned += 1
        if apply:
            new_text, count = rewrite_text(text, relative, moves)
            if count:
                path.write_text(new_text, encoding="utf-8")
                rewritten += 1
                if remaining(path.read_text(encoding="utf-8"), moves):
                    print(f"  ✗ {relative} 改写之后回读仍有旧路径")
                    print("     → 怎么办：这一处的写法不在 forms() 列的几种里，手工改或给 forms() 补一种写法。")
                    return 3
        else:
            for number, old_form in remaining(text, moves):
                findings.append(f"{relative}:{number} 还指着旧路径 {old_form}")
    if apply:
        print(f"  ✓ 按 {len(moves)} 条搬迁登记改写了 {rewritten} 个文件（扫了 {scanned} 个文本文件），回读都没有旧路径")
        return 0
    if findings:
        for finding in findings:
            print(f"      {finding}")   # gate-lint:detail
        print(f"  ✗ 全仓还有 {len(findings)} 处指向登记过的旧路径（扫了 {scanned} 个文本文件、{len(moves)} 条搬迁登记）")
        print("     → 怎么办：跑 python3 research/scripts/rewrite-moved-paths.py --apply 改成新路径；路径不在冻结范围内，research/prompts/ 与 research/results/ 也改（.claude/rules/path-moves.md）。")
        return 1
    print(f"  ✓ 全仓没有指向登记过的旧路径的地方（扫了 {scanned} 个文本文件、{len(moves)} 条搬迁登记）")
    return 0


def selftest() -> int:
    with tempfile.TemporaryDirectory() as work:
        root = pathlib.Path(work)
        (root / "research/scripts").mkdir(parents=True)
        (root / REGISTRY).write_text(".claude/kb/old-name.md\t.claude/kb/folder/01-new.md\t2026-09-17\t自证\n", encoding="utf-8")
        (root / ".claude/kb/milestone").mkdir(parents=True)
        (root / "research/prompts").mkdir(parents=True)
        (root / ".claude/kb/milestone/a.md").write_text("见 [old-name.md](../old-name.md)「八」与 `.claude/kb/old-name.md`\n", encoding="utf-8")
        (root / "research/prompts/b.md").write_text("冻结材料里也写了 old-name八那张表与 old-name.md 八\n从 `.claude/kb/old-name.md` 搬到 `.claude/kb/folder/01-new.md`\n", encoding="utf-8")
        if run(root, apply=False) != 1:
            print("  ✗ 自证失败：带旧路径的小仓 --check 没判红")
            print("     → 怎么办：remaining() 的匹配坏了，看 forms() 与边界正则。")
            return 1
        if run(root, apply=True) != 0 or run(root, apply=False) != 0:
            print("  ✗ 自证失败：--apply 之后 --check 仍不绿")
            print("     → 怎么办：rewrite_text() 没改全，看链接重算与 forms()。")
            return 1
        link_line = (root / ".claude/kb/milestone/a.md").read_text(encoding="utf-8")
        statement = (root / "research/prompts/b.md").read_text(encoding="utf-8").split("\n")[1]
        frozen_line = (root / "research/prompts/b.md").read_text(encoding="utf-8").split("\n")[0]
        if "folder/01-new八" not in frozen_line:
            print(f"  ✗ 自证失败：旧名后面紧跟汉字的写法没改到（边界把汉字当成了词的一部分）：{frozen_line!r}")
            print("     → 怎么办：边界正则只认 ASCII 的字母、数字、下划线、连字符。")
            return 1
        if "](../folder/01-new.md)" not in link_line or ".claude/kb/old-name.md" not in statement:
            print(f"  ✗ 自证失败：链接没按文件位置重算、或说搬迁本身的那一行被改了：{link_line!r} / {statement!r}")
            print("     → 怎么办：看 fix_link 的相对路径与 statement_line 的判法。")
            return 1
    same_name_failure = selftest_same_file_name_move()
    if same_name_failure:
        return same_name_failure
    print("  ✓ 自证：带旧路径的小仓 --check 判红；--apply 之后判绿，链接按文件位置重算，说搬迁本身的那一行保留旧路径；文件名不变的搬迁只换整条路径、不重复换")
    return 0


def selftest_same_file_name_move() -> int:
    """文件名不变、只换目录的搬迁：整条旧路径换成新路径且只换一次，光写文件名的地方不动，--apply 之后 --check 判绿。"""
    with tempfile.TemporaryDirectory() as work:
        root = pathlib.Path(work)
        (root / "research/scripts").mkdir(parents=True)
        (root / REGISTRY).write_text(".claude/shared.md\t.claude/agents/shared.md\t2026-09-18\t自证\n", encoding="utf-8")
        (root / ".claude/agents").mkdir(parents=True)
        (root / ".claude/rules").mkdir(parents=True)
        (root / ".claude/rules/c.md").write_text("开工先读 `.claude/shared.md` 与 [共用](../shared.md)，光写 shared.md 也行\n管 `.claude/agents/*.md`、`.claude/shared.md`\n", encoding="utf-8")
        (root / ".claude/gate.sh").write_text('pattern = r"^\\.claude/shared\\.md$"  # .claude/shared.md\n', encoding="utf-8")
        if run(root, apply=False) != 1:
            print("  ✗ 自证失败：文件名不变的搬迁，带整条旧路径的小仓 --check 没判红")
            print("     → 怎么办：forms() 把整条旧路径也当成新路径的末尾一截丢掉了，看 is_tail_of_new_path。")
            return 1
        if run(root, apply=True) != 0 or run(root, apply=False) != 0:
            print("  ✗ 自证失败：文件名不变的搬迁 --apply 之后 --check 仍不绿")
            print("     → 怎么办：光写文件名的写法还在 forms() 里，改写或检查把指新文件的地方当成旧路径。")
            return 1
        rule_line = (root / ".claude/rules/c.md").read_text(encoding="utf-8")
        gate_line = (root / ".claude/gate.sh").read_text(encoding="utf-8")
        expected = "开工先读 `.claude/agents/shared.md` 与 [共用](../agents/shared.md)，光写 shared.md 也行\n管 `.claude/agents/*.md`、`.claude/agents/shared.md`\n"
        if rule_line != expected or "agents/agents" in gate_line or ".claude/agents/.claude" in gate_line or "# .claude/agents/shared.md" not in gate_line:
            print(f"  ✗ 自证失败：文件名不变的搬迁改出了重复换的路径，或光写文件名的地方被改了：{rule_line!r} / {gate_line!r}")
            print("     → 怎么办：rewrite_text() 要一趟换完全部写法，forms() 要丢掉新路径末尾那一截。")
            return 1
    return 0


def main(argv) -> int:
    if len(argv) >= 2 and argv[1] == "--selftest":
        return selftest()
    if len(argv) < 2 or argv[1] not in ("--check", "--apply"):
        print("  ✗ 用法：rewrite-moved-paths.py --check|--apply [仓库根] 或 --selftest")
        print("     → 怎么办：先 --check 看还剩几处，再 --apply 改。")
        return 2
    rest = argv[2:]
    skip = []
    while "--skip" in rest:
        at = rest.index("--skip")
        skip.append(rest[at + 1])
        del rest[at:at + 2]
    root = pathlib.Path(rest[0]) if rest else pathlib.Path(__file__).resolve().parents[2]
    return run(root.resolve(), apply=argv[1] == "--apply", skip=tuple(skip))


if __name__ == "__main__":
    sys.exit(main(sys.argv))
