#!/usr/bin/env python3
"""全仓术语改名：把一个概念的旧名换成新名，源码、文档、冻结提示与留存产物一起换。

用法：
  sweep-term.py --check                 # 全仓还剩多少处旧名（豁免的不算），剩了就退出码 1
  sweep-term.py --apply                 # 换成新名
  sweep-term.py --selftest

为什么源码与产物要一起换：产物里的串来自源码里的串，只换一边，`replay.sh` 的 exact 判据下次复跑必然「对不上」；
两边一起换，复跑出来的字节与换后的产物仍然逐字节一致 —— 这一条要靠复跑证明，不是声称。

豁免写在项目根的 `.claude/term-rename-exempt`，一行一条、`#` 后写为什么；指向不存在的路径判红
（不起作用的豁免项会让人以为那批文件已经被绕开了，判据与 `.claude/doc-lint-exclude` 同）。

--apply 的写回不在原文件上就地写：每份文件同目录排他新建临时文件、fsync、照原权限位与属主、改名换上
（`lib_atomic_replace.py`）。扫全仓会扫到 .sh 与 .py，正在按偏移边跑边读它的进程读的仍是旧 inode 的旧内容。
一份被拒（多个硬链接、当前用户只读、建不了临时文件、写或改名失败）就跳过它、接着换别的，最后逐份列出。

退出码：--check 0 搜不出旧名、1 还有旧名；--apply 0 换完、3 有文件没换上（那几份原文件没动、临时文件已删）；
2 登记表或豁免表有问题。
"""
import os
import sys

import re

from lib_atomic_replace import ReplaceRefused, problems_with_replacement_by_rename, replace_file_contents_by_rename

SKIP_DIRS = {".git", "target", "node_modules"}
EXEMPT_FILE = ".claude/term-rename-exempt"
RENAMES_PATH = ".claude/kb/term-renames.md"
TABLE_MARK = "<!-- term-renames:table -->"


def load_renames(root):
    """从 .claude/kb/term-renames.md 的登记表读「旧 → 新」，一行一对。
    加一条术语改名 = 往那张表加一行，这个脚本与门禁自动管到它，不用再改代码。"""
    path = os.path.join(root, RENAMES_PATH)
    if not os.path.exists(path):
        print("  ✗ 找不到术语对照表 %s" % RENAMES_PATH)
        print("     → 怎么办：建一份，表上方写 %s，四列：旧、新、匹配（整串 / 词边界）、是什么。" % TABLE_MARK)
        sys.exit(2)
    text = open(path, encoding="utf-8").read()
    if TABLE_MARK not in text:
        print("  ✗ %s 里没有 %s 标记，找不到哪张表是登记表" % (RENAMES_PATH, TABLE_MARK))
        print("     → 怎么办：在登记表上方单起一行写这个标记。")
        sys.exit(2)
    pairs = []
    for line in text[text.index(TABLE_MARK):].split("\n"):
        if not line.startswith("|"):
            continue
        cells = [c.strip().strip("`") for c in line.strip("|").split("|")]
        if len(cells) < 3 or cells[0] in ("旧", "") or set(cells[0]) <= set("-: "):
            continue
        old, new, how = cells[0], cells[1], cells[2]
        if how not in ("整串", "词边界"):
            print("  ✗ %s：「%s」那一行的「匹配」写的是「%s」，只认「整串」或「词边界」" % (RENAMES_PATH, old, how))
            print("     → 怎么办：整串就写整串；缩写这类怕打中别的词的写词边界。")
            sys.exit(2)
        pairs.append((old, new, how))
    if not pairs:
        print("  ✗ %s 的登记表一行数据都没有" % RENAMES_PATH)
        print("     → 怎么办：每改一个全仓术语加一行，门禁才管得到它。")
        sys.exit(2)
    return pairs


def compile_rules(pairs):
    """替换规则按表的顺序；检测正则**独立合成、比替换表宽**——替换漏掉哪种形态，检测也要报得出来
    （自检与被检共用同一条判据，判据错了两边一起错，见 checks-owed C428）。"""
    replacements, detectors = [], []
    for old, new, how in pairs:
        if how == "词边界":
            if old.endswith("_"):
                pattern = r"\b" + re.escape(old)
            elif old.startswith("_"):
                pattern = re.escape(old) + r"\b"
            else:
                pattern = r"\b" + re.escape(old) + r"\b"
        else:
            pattern = re.escape(old)
        replacements.append((re.compile(pattern), new))
        detectors.append(pattern)
    return replacements, re.compile("|".join(detectors))


def load_exempt(root):
    path = os.path.join(root, EXEMPT_FILE)
    if not os.path.exists(path):
        print("  ✗ 找不到豁免登记表 %s" % EXEMPT_FILE)
        print("     → 怎么办：建一张，一行一条 <相对路径或目录>  # 为什么；没有豁免就留一张只有注释的空表。")
        sys.exit(2)
    entries, missing = [], []
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        relative = line.split("#")[0].strip()
        entries.append(relative)
        if not os.path.exists(os.path.join(root, relative)):
            missing.append(relative)
    # 对照表与这个脚本自己都写得出旧名（表里那一列就是旧名，脚本要从表里读），
    # 不豁免就会把自己吃掉 ⇒ 强制要求，不靠写表的人记得。
    myself = os.path.relpath(os.path.abspath(__file__), root)
    # 脚本在 root 外面（门禁拿真脚本扫判别力样本就是这样）时它根本不会被扫到，不用要求豁免
    required_paths = [RENAMES_PATH] + ([myself] if not myself.startswith("..") else [])
    for required in required_paths:
        if not any(required == e or required.startswith(e.rstrip("/") + "/") for e in entries):
            print("  ✗ %s 没登记进 %s" % (required, EXEMPT_FILE))
            print("     → 怎么办：加一行豁免它。对照表那一列就是旧名，这个脚本要从表里读旧名，")
            print("               不豁免的话第一次 --apply 就把它们自己换掉，之后再也换不动东西。")
            sys.exit(2)
    if missing:
        print("  ✗ 豁免登记表里有 %d 条指不到文件：" % len(missing))
        for relative in missing:                               # gate-lint:detail
            print("      %s" % relative)
        print("     → 怎么办：指不到的豁免项删掉或改成现存路径；不起作用的豁免会让人以为那批文件已经被绕开了。")
        sys.exit(2)
    return entries


def exempt(relative, entries):
    for entry in entries:
        if relative == entry or relative.startswith(entry.rstrip("/") + "/"):
            return True
    return False


def walk(root, entries):
    for base, dirs, names in os.walk(root):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for name in names:
            relative = os.path.relpath(os.path.join(base, name), root)
            if not exempt(relative, entries):
                yield relative


def sweep(root, apply_changes, defer=()):
    """defer 是**分批落地**用的，不是豁免：别的会话正在读那批文件时先放着，那一批照样要换，换完 --check 才绿。
    豁免只有 .claude/term-rename-exempt 一处，别把 defer 当第二处豁免用。"""
    entries = load_exempt(root)
    entries = list(entries) + list(defer)
    pairs = load_renames(root)
    replacements, detect = compile_rules(pairs)
    files, occurrences, scanned = 0, 0, 0
    worst = []
    refused = []
    for relative in walk(root, entries):
        try:
            text = open(os.path.join(root, relative), encoding="utf-8").read()
        except (UnicodeDecodeError, OSError):
            continue
        scanned += 1
        count = len(detect.findall(text))
        if not count:
            continue
        if apply_changes:
            for pattern, new in replacements:
                text = pattern.sub(new, text)
            try:
                notes = replace_file_contents_by_rename(os.path.join(root, relative), text)
            except ReplaceRefused as refusal:
                refused.append((relative, count, str(refusal)))
                continue
            for note in notes:
                print("  ! %s" % note)
        files += 1
        occurrences += count
        worst.append((count, relative))
    if apply_changes:
        print("  ✓ 换了 %d 份文件、%d 处（登记的改名 %d 条、豁免 %d 条）" % (files, occurrences, len(pairs), len(entries)))
        if refused:
            for relative, count, message in refused:                # gate-lint:detail
                print("      %s（%d 处）：%s" % (relative, count, message))
            print("  ✗ 有 %d 份文件没换上，那几份原文件没动、临时文件已删（逐份的原因与下一步在上面）" % len(refused))
            print("     → 怎么办：按每份后面那行「→ 怎么办」处理（多半是有多个硬链接或当前用户只读），再跑一次 --apply；")
            print("               已经换好的那些搜不出旧名，不会重复换。")
            return 3
        print("     → 下一步：cargo build 与 cargo test --workspace，再 bash research/scripts/replay.sh 全量复跑，")
        print("               产物必须仍然逐字节一致 —— 源码与产物一起换，这一条才成立，要跑出来看，不能声称。")
        return 0
    if occurrences:
        for count, relative in sorted(worst, reverse=True)[:20]:   # gate-lint:detail
            print("      %-72s %d 处" % (relative, count))
        print("  ✗ 全仓还有 %d 份文件、%d 处旧名（登记的改名 %d 条、豁免 %d 条不算）" % (files, occurrences, len(pairs), len(entries)))
        print("     → 怎么办：跑 python3 research/scripts/sweep-term.py --apply 换掉；")
        print("               确实该留旧名的，登记进 .claude/term-rename-exempt 并写明为什么。")
        return 1
    print("  ✓ 全仓搜不出登记过的旧名了：查了 %d 份文本文件、%d 条改名登记" % (scanned, len(pairs)))
    print("     没扫的 %d 处（.claude/term-rename-exempt 登记，各有理由）：%s" % (len(entries), "、".join(entries)))
    return 0


def selftest():
    import tempfile
    with tempfile.TemporaryDirectory() as work:
        os.makedirs(os.path.join(work, ".claude"))
        os.makedirs(os.path.join(work, "keep"))
        open(os.path.join(work, "keep", "quoted.md"), "w", encoding="utf-8").write("btrfs 的 superblock\n")
        # 每一种变体各一行：漏掉哪一种，下面「换完判绿」那一步就红
        open(os.path.join(work, "code.rs"), "w", encoding="utf-8").write(
            "const SUPERBLOCK_BYTES: u64 = 481;\n"
            "const SUPER_BLOCK_BYTES: u64 = 481;\n"
            "struct SuperBlock;\nstruct Superblock;\nstruct Super_Block;\n"
            "let superBlock = 1;\nlet super_block = 2;\nlet superblock = 3;\n"
            "// super-block 与 超级块 两种写法\n"
            "let sb_mac = 4;\nlet tail_sb = 5;\nlet sb = 6;\n")
        open(os.path.join(work, ".claude", "term-rename-exempt"), "w", encoding="utf-8").write("keep/  # 别家术语\n.claude/kb/term-renames.md  # 表自己写着旧名\n%s  # 脚本自己写着旧名\n"
            % os.path.relpath(os.path.abspath(__file__), work))
        os.makedirs(os.path.join(work, ".claude", "kb"))
        open(os.path.join(work, ".claude", "kb", "term-renames.md"), "w", encoding="utf-8").write(
            "<!-- term-renames:table -->\n"
            "| 旧 | 新 | 匹配 | 是什么 |\n|---|---|---|---|\n"
            "| 超级块 | 系统配置 | 整串 | 中文 |\n"
            "| SUPER_BLOCK | SYSTEM_CONFIGURATION | 整串 | 常量带下划线 |\n"
            "| SUPERBLOCK | SYSTEM_CONFIGURATION | 整串 | 常量 |\n"
            "| Super_Block | SystemConfiguration | 整串 | 大驼峰带下划线 |\n"
            "| SuperBlock | SystemConfiguration | 整串 | 大驼峰 |\n"
            "| Superblock | SystemConfiguration | 整串 | 大驼峰 b 小写 |\n"
            "| superBlock | systemConfiguration | 整串 | 小驼峰 |\n"
            "| super_block | system_configuration | 整串 | 蛇形带下划线 |\n"
            "| super-block | system-configuration | 整串 | 连字符 |\n"
            "| superblock | system_configuration | 整串 | 蛇形 |\n"
            "| sb_ | system_configuration_ | 词边界 | 缩写前缀 |\n"
            "| _sb | _system_configuration | 词边界 | 缩写后缀 |\n"
            "| sb | system_configuration | 词边界 | 裸缩写 |\n")
        if sweep(work, False) != 1:
            print("  ✗ 自检失败：带旧名的小仓 --check 没判红")
            print("     → 怎么办：看 walk() 与 MAP 的匹配。")
            return 1
        sweep(work, True)
        if sweep(work, False) != 0:
            print("  ✗ 自检失败：--apply 之后 --check 仍判红")
            print("     → 怎么办：看 MAP 的替换顺序，大小写形态是不是漏了一种。")
            return 1
        if "superblock" not in open(os.path.join(work, "keep", "quoted.md"), encoding="utf-8").read():
            print("  ✗ 自检失败：豁免目录里的旧名被换掉了")
            print("     → 怎么办：看 exempt() 的目录前缀判法。")
            return 1
        open(os.path.join(work, ".claude", "term-rename-exempt"), "w", encoding="utf-8").write("nowhere/  # 指不到\n")
        code = None
        try:
            sweep(work, False)
        except SystemExit as stop:
            code = stop.code
        if code != 2:
            print("  ✗ 自检失败：豁免登记表指不到文件时没判红")
            print("     → 怎么办：看 load_exempt() 的存在性断言。")
            return 1
    # 写回的方式：inode 要换、在读的进程读旧内容、权限位与符号链接保住、硬链接与只读拒绝、失败不留临时文件（判据在 lib_atomic_replace.py）
    running_script = "#!/usr/bin/env bash\n" + "echo 前面这几行正在被 bash 边跑边读\n" * 20 + "run_step 超级块\nexit $?\n"
    problems = problems_with_replacement_by_rename(
        apply_to_this_file_only, running_script, running_script.replace("超级块", "系统配置", 1))
    if problems:
        print("  ✗ 自检失败：--apply 写回的方式不对")
        for problem in problems:
            print("     " + problem)  # gate-lint:detail
        print("     → 怎么办：按方括号里那一格查 lib_atomic_replace.py 的 replace_file_contents_by_rename，"
              "以及 sweep() 里 --apply 那一段是不是还有就地写。")
        return 1
    print("  ✓ 自检：带旧名判红、换完判绿、豁免目录不被换、豁免指不到文件判红；--apply 换上的是新 inode、权限位不变、"
          "改之前打开文件的读者接着读到旧内容、fsync 在改名之前、经符号链接改的是它指向的文件、"
          "有两个硬链接或当前用户只读都拒绝、fsync 或改名失败都不动原文件也不留临时文件")
    return 0


def apply_to_this_file_only(path):
    """自证用：把 path 所在的目录当仓根、登记一张只有「超级块 → 系统配置」的改名表，目录里别的条目全登记豁免，
    让 --apply 只换 path 这一份（经符号链接那一格就一定走链接那一个名字）。返回退出码。"""
    root = os.path.dirname(path)
    target_name = os.path.basename(path)
    other_names = sorted(name for name in os.listdir(root) if name not in (target_name, ".claude"))
    os.makedirs(os.path.join(root, ".claude", "kb"), exist_ok=True)
    with open(os.path.join(root, RENAMES_PATH), "w", encoding="utf-8") as handle:
        handle.write("%s\n| 旧 | 新 | 匹配 | 是什么 |\n|---|---|---|---|\n| 超级块 | 系统配置 | 整串 | 中文 |\n" % TABLE_MARK)
    with open(os.path.join(root, EXEMPT_FILE), "w", encoding="utf-8") as handle:
        handle.write("%s  # 表自己写着旧名\n" % RENAMES_PATH)
        for name in other_names:
            handle.write("%s  # 自证只换 %s 这一份\n" % (name, target_name))
    try:
        return sweep(root, True)
    except SystemExit as stop:
        return stop.code


def main(argv):
    if "--selftest" in argv:
        return selftest()
    positional = [a for a in argv if not a.startswith("--")]
    root = os.path.abspath(positional[0]) if positional else os.path.abspath(
        os.path.join(os.path.dirname(__file__), "..", ".."))
    defer = [argv[i + 1] for i, a in enumerate(argv) if a == "--defer" and i + 1 < len(argv)]
    positional = [a for a in positional if a not in defer]
    if defer:
        print("  ⚠️ 这一批先放着（分批落地，不是豁免）：%s" % "、".join(defer))
    return sweep(root, "--apply" in argv, defer)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
