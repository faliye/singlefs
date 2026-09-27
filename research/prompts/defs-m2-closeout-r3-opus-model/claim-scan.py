#!/usr/bin/env python3
"""defs-m2-closeout-r3 云端攻方 K3 / G4：拿弹窗闸今天的判法（G4 之后）与 G4 之前那一版（F13，只把 FILE_AND_LINE 换回改前那一行）
各判两批句子，数谁放、谁拒。判法取自 .claude/hooks/ask-user-claim-guard.sh 里 heredoc 那段 python 原文（不改一字，只把 main 去掉）；
F13 那一版在内存里把 G4 改的那一行换回附录二 diff 的「-」行。只读，不写任何文件。
  甲批：仓里 git 跟踪的每个文件，各造三句带断言词的问句：「<路径>:1 写着这一格一定不变」「见 <路径>：12，这一格一定不变」「<文件名>:3 写着它一定为 0」。
  乙批：没有出处、只是「编号/编号：数」的断言句（G4 的自证只收了 / 后面带汉字的那几种）。
用法（仓根下）：python3 research/prompts/defs-m2-closeout-r3-opus-model/claim-scan.py"""
import os, subprocess, sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
HOOK = os.path.join(ROOT, ".claude/hooks/ask-user-claim-guard.sh")
G4_LINE = '    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/(?:{ASCII_PATH_CHARACTER}+|{FILE_NAME_CHARACTER}*\\.(?:{EXTENSION_ALTERNATION}))"\n'
F13_LINE = '    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/{FILE_NAME_CHARACTER}+"\n'

def judge_module(source):
    namespace = {"__name__": "claim_scan"}
    exec(compile(source, HOOK, "exec"), namespace)
    return namespace

def main():
    text = open(HOOK, encoding="utf-8").read()
    body = text.split("3<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0]
    body = body.replace("sys.exit(main())", "")
    if body.count(G4_LINE) != 1:
        print("✗ 今天的钩子里找不到 G4 那一行，判法变了，这个扫描作废"); return 1
    versions = {"G4": judge_module(body), "F13": judge_module(body.replace(G4_LINE, F13_LINE))}
    paths = subprocess.run(["git", "-C", ROOT, "-c", "core.quotepath=false", "ls-files"], capture_output=True, text=True, check=True).stdout.splitlines()
    batch_a = []
    for path in paths:
        batch_a += [f"{path}:1 写着这一格一定不变", f"见 {path}：12，这一格一定不变", f"{os.path.basename(path)}:3 写着它一定为 0"]
    batch_b = ["F1/F4：2 处一定都要改", "54/55：2 道一定要带前缀", "E142/E156：3 个字段一定是 0", "O3/O8：2 格一定该升成打中",
               "r1/r2:2 处一定都要回退", "H1/H2：2 格一定是打中", "G1/G9：两条一定都要改", "K1/K2：2 格一定都中"]
    forms = ("路径:1", "见 路径：12", "文件名:3")
    rejected_sets = {}
    for name, module in versions.items():
        rejected_a = [s for s in batch_a if module["unsupported_claims"](s)]
        rejected_sets[name] = set(rejected_a)
        by_form = [sum(1 for index, s in enumerate(batch_a) if index % 3 == form and s in rejected_sets[name]) for form in range(3)]
        print(f"{name}：甲批按句式分的拒数：" + "、".join(f"{forms[form]} {by_form[form]}" for form in range(3)))
        passed_b = [s for s in batch_b if not module["unsupported_claims"](s)]
        print(f"{name}：甲批 {len(batch_a)} 句（{len(paths)} 个跟踪文件 × 3），拒 {len(rejected_a)} 句")
        for sentence in sorted(set(rejected_a))[:6]:
            print(f"    拒：{sentence}")
        print(f"{name}：乙批 {len(batch_b)} 句没有出处的断言，放过 {len(passed_b)} 句")
        for sentence in passed_b:
            print(f"    放：{sentence}")
    print(f"甲批里 G4 拒而 F13 放的 {len(rejected_sets['G4'] - rejected_sets['F13'])} 句，F13 拒而 G4 放的 {len(rejected_sets['F13'] - rejected_sets['G4'])} 句")
    return 0

if __name__ == "__main__":
    sys.exit(main())
