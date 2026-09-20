#!/usr/bin/env python3
"""回扫一条规则时核对判据有没有被误删：原文的每个片段，今天还在不在正文里。

规则正文只写怎么做，实测与论证从正文删掉（.claude/singlefs-ai-sop/rules/rules-discipline.md）。
删的时候最容易出的错是顺手带走一句谁也没注意到的判据——删了不会有任何东西报警。
这个脚本拿 git 里的那一版当基准，逐片段查它今天还在不在。

  python3 research/scripts/rules-sweep-audit.py .claude/rules/x.md [--base HEAD]
  python3 research/scripts/rules-sweep-audit.py --selftest

退出码：0 = 每个片段都找得到；1 = 有片段找不到，逐条列出；2 = 用法或环境不对。
找不到不等于错：删掉日期、把「（2026-09-17 用户定）」整个去掉，都会让那一句对不上。
它报的是「这一句要人看一眼」：是判据就放回正文，是历史或论证才算删对了。
"""
import re
import subprocess
import sys
from pathlib import Path

DECORATION = re.compile(r"[\s*`>_~|—·…]|⚠️|⚠")
SPLIT = re.compile(r"[。；\n]")
MINIMUM_FRAGMENT_LENGTH = 8


def normalize(text):
    return DECORATION.sub("", text)


def fragments(text):
    for raw in SPLIT.split(text):
        piece = normalize(raw)
        if len(piece) >= MINIMUM_FRAGMENT_LENGTH:
            yield raw.strip(), piece


def audit(original_text, current_texts):
    haystack = normalize("\n".join(current_texts))
    return [raw for raw, piece in fragments(original_text) if piece not in haystack]


def selftest():
    original = "第一句要留下的判据写在这里。第二句是实测经过，删掉。第三句也被删了但它其实是判据。"
    kept = "第一句要留下的判据写在这里。"
    missing = audit(original, [kept])
    assert len(missing) == 2 and any("第三句" in m for m in missing), missing
    assert audit(original, [original]) == [], "原样不动时不该报任何片段"
    assert audit(original, [kept.replace("判据", "**判据**")]) != [], "装饰之外的字改了要报"
    print("  ✓ rules-sweep-audit 自检通过（删一句会红、原样不动不红、改字会红）")
    return 0


def main(argv):
    if "--selftest" in argv:
        return selftest()
    if not argv:
        print("  ✗ 没给要核对的规则文件")
        print("     → 怎么办：python3 research/scripts/rules-rationale-audit.py .claude/rules/x.md")
        return 2
    base = "HEAD"
    if "--base" in argv:
        index = argv.index("--base")
        base = argv[index + 1]
        argv = argv[:index] + argv[index + 2:]
    failed = False
    for name in argv:
        rule_path = Path(name)
        completed = subprocess.run(["git", "show", f"{base}:{rule_path}"],
                                   capture_output=True, text=True)
        if completed.returncode != 0:
            print(f"  ✗ {rule_path} 在 {base} 里取不到：{completed.stderr.strip()}")
            print(f"     → 怎么办：确认路径没写错，或者用 --base <另一个提交> 指到回扫之前那一版。")
            failed = True
            continue
        if not rule_path.is_file():
            print(f"  ✗ {rule_path} 不在")
            print(f"     → 怎么办：确认路径没写错；这一份要是整个删掉了，就别拿它跑这个核对。")
            failed = True
            continue
        current = [rule_path.read_text(encoding="utf-8")]
        missing = audit(completed.stdout, current)
        if missing:
            failed = True
            print(f"  ✗ {rule_path}：{len(missing)} 个片段在正文里找不到了")
            for raw in missing:
                print(f"     {raw[:100]}")
            print( "     → 怎么办：逐条看——是判据就放回正文；是实测、论证或定案记述，删掉就对了；")
            print( "               只是把日期或定案人删掉而让句子对不上的，看一眼确认没丢别的字就算过。")
        else:
            print(f"  ✓ {rule_path}：{len(list(fragments(completed.stdout)))} 个片段都还在正文里")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
