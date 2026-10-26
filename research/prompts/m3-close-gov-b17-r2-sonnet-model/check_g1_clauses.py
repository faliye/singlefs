#!/usr/bin/env python3
"""H1：D13 已定项 17 定案里和两处复述有关的片段，逐片在 kb 那一行、implementation-workflow 复述行、verification 复述行里找。
用法：python3 -I check_g1_clauses.py <仓根>
kb 行 = 以「- 两种排除不进键，理由不同、分开记。」起头的那一行；两处复述行 = 含「算键时有两种排除」的 implementation-workflow 行与含「两种排除理由不同」的 verification 行。
每个片段打 kb / impl / verif 三栏 Y 或 -。（kb 的行号、规则的行号也打出来，按 grep -nF 现取。）"""
import os, sys

repo = os.path.abspath(sys.argv[1])


def find(path, needle):
    for number, line in enumerate(open(os.path.join(repo, path), encoding="utf-8").read().split("\n"), 1):
        if needle in line:
            return number, line
    raise SystemExit(f"{path} 里找不到「{needle}」")


kb_n, kb = find(".claude/kb/decisions/13-验证路线.md", "- 两种排除不进键，理由不同、分开记。")
impl_n, impl = find(".claude/rules/implementation-workflow.md", "算键时有两种排除")
ver_n, ver = find(".claude/rules/verification.md", "两种排除理由不同")
print(f"kb_line={kb_n} impl_line={impl_n} verif_line={ver_n}")
KB_FIRST = find(".claude/kb/decisions/13-验证路线.md", "- 两种排除不进键")[0]
# 定案第一条（键的组成）与第二条（一律不进键）也是这两处复述压着的
kb1_n, kb1 = find(".claude/kb/decisions/13-验证路线.md", "- 重型门禁、崩溃判定库与实验准入的每一份判定")
kb2_n, kb2 = find(".claude/kb/decisions/13-验证路线.md", "- 一律不进键、也不许当复用依据")
print(f"kb_bullet1_line={kb1_n} kb_bullet2_line={kb2_n}")
CLAUSES = [
    ("理由不同、分开记", "理由不同、分开记"),
    ("文件级对象 NOT_JUDGING_FILES 那两份", "`NOT_JUDGING_FILES` 那两份"),
    ("算键的库自己", "算键的库自己"),
    ("单元算法", "单元算法"),
    ("按角色排", "按角色排"),
    ("管怎么存、怎么算键，不管判对错", "管怎么存、怎么算键，不管判对错"),
    ("它们的单元不进任何一份判定的键", "它们的单元不进任何一份判定的键"),
    ("边级定义：只打提示、不改判定结局的子进程边", "只打提示、不改判定结局的子进程边"),
    ("按实测排", "按实测排"),
    ("不沿它收", "不沿它收"),
    ("现在只有（现状句）", "现在只有"),
    ("第二台那一路只出提示，按这条边排出键", "第二台那一路只出提示，按这条边排出键"),
    ("登记位 EXCLUDED_PROGRAM_EDGES", "EXCLUDED_PROGRAM_EDGES"),
    ("三格自证全句", "这条路上的任何异常不改包装退出码；判法读包装输出前去掉这条路打的行；切边前后少掉的单元只经这条边可达"),
    ("边级排除要配三格自证", "边级排除要配三格自证"),
    ("另有一格键不变而退出码变必红", "另有一格「键不变而退出码变」必红"),
    ("模块级语句：一起收进来", "一起收进来"),
    ("模块级语句：一起进闭包", "一起进闭包"),
    ("模块级四样清单", "import、赋值、类头本身（不连方法）、带装饰器或缺省值的 def 头"),
]
print("clause\tkb\timpl\tverif")
for label, needle in CLAUSES:
    print(f"{label}\t{'Y' if needle in kb else '-'}\t{'Y' if needle in impl else '-'}\t{'Y' if needle in ver else '-'}")
KEY = [
    ("键组成：从 Cargo.lock 解析出的闭包真引到的依赖版本", "闭包真引到的依赖版本"),
    ("键组成：含 cargo 解析后的生效构建环境与外部工具版本", "含 cargo 解析后的生效构建环境与外部工具版本"),
    ("不进键清单：种子串", "种子串"),
    ("不进键清单：路径（仓根也算）", "路径（仓根也算）"),
]
print("key-clause\tkb_bullet1_or_2\timpl\tverif")
for label, needle in KEY:
    print(f"{label}\t{'Y' if needle in kb1 or needle in kb2 else '-'}\t{'Y' if needle in impl else '-'}\t{'Y' if needle in ver else '-'}")
