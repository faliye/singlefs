#!/usr/bin/env python3
"""模型 M2（gate-shrink-r1 云端攻方）：B5「47 号 research-script-selftests 只认自己实现了 --selftest 入口的脚本」。
「谁声称有 --selftest」的认法换掉之后，哪些真有自证的脚本会从被扫集合里掉出去（掉出去就没人要求它有阶段在跑）。
三种认法：
  R0  今天的：文件全文里出现 --selftest 这串字（47 号 research-script-selftests 格覆盖段的 claimed）；
  R1  按入口写法列举：在本仓现有写法里挑最常见的两种（python 的 == 比较、shell 的 case 分支）；
  R2  按入口写法列举：本仓现有全部六种写法的并集（== 比较、in argv、argparse、case 分支、[ ] / [[ ]] 比较、startswith / argv[1] ==）；
  R3  不列写法：只认 .sh / .py，先去掉注释行与出路文字（与 47 号覆盖段认「有阶段在跑」时去掉的是同一套：# 注释行、
      echo / printf / howto / bad / die / say / print( 后面带引号的参数），剩下的代码里还出现 --selftest 才算。
两批输入：research/scripts/ 今天的全部文件；再加五份临时造的新脚本（各用一种本仓还没有的写法实现真的自证入口）
与两份只在注释、出路文字里提到别人 --selftest 的脚本。
用法：python3 m2_selftest_claim_recognizers.py <仓根>
"""
import glob, os, re, sys, tempfile

OUTPUT_TEXT = re.compile(r'''(?:\b(?:echo|printf|howto|bad|die|say)\b(?:\s+-\w+)*|\bprint\()(?:\s*f?(?:"(?:[^"\\\n]|\\.)*"|'(?:[^'\\\n]|\\.)*'))+''')
R1_FORMS = [re.compile(r'''==\s*\[?\s*["']--selftest["']'''), re.compile(r'''^\s*"?--selftest"?\s*\)''', re.M)]
R2_FORMS = R1_FORMS + [
    re.compile(r'''["']--selftest["']\s+in\s+'''),
    re.compile(r'''add_argument\(\s*["']--selftest'''),
    re.compile(r'''\|\s*--selftest\s*\)|--selftest\s*\|'''),
    re.compile(r'''\[\[?\s*"?\$\{?1[^\]]*\]?\s*==?\s*"?--selftest"?'''),
    re.compile(r'''startswith\(\s*["']--selftest|argv\[1\]\s*==\s*["']--selftest'''),
]


def claims_r0(path, text):
    return "--selftest" in text


def claims_r1(path, text):
    return any(form.search(text) for form in R1_FORMS)


def claims_r2(path, text):
    return any(form.search(text) for form in R2_FORMS)


def claims_r3(path, text):
    if not path.endswith((".sh", ".py")):
        return False
    code = "\n".join(line for line in text.splitlines() if not line.lstrip().startswith("#"))
    code = OUTPUT_TEXT.sub(" ", code)
    return "--selftest" in code


RECOGNIZERS = [("R0", claims_r0), ("R1", claims_r1), ("R2", claims_r2), ("R3", claims_r3)]

SYNTHETIC = {
    # 真有自证入口，写法在本仓 research/scripts/ 里还没出现过
    "new-getopts.sh": "#!/usr/bin/env bash\nwhile [[ $# -gt 0 ]]; do\n  case \"$1\" in\n    --selftest|-s) run_selftest; exit $? ;;\n  esac\n  shift\ndone\n",
    "new-match.py": "import sys\nmatch sys.argv[1:]:\n    case [\"--selftest\"]:\n        sys.exit(selftest())\n",
    "new-set-membership.py": "import sys\nif {\"--selftest\"} & set(sys.argv):\n    sys.exit(selftest())\n",
    "new-flag-constant.py": "import sys\nSELFTEST_FLAG = \"--selftest\"\nif SELFTEST_FLAG in sys.argv[1:]:\n    sys.exit(selftest())\n",
    "new-if-test.sh": "#!/usr/bin/env bash\nif test \"${1:-}\" = --selftest; then run_selftest; exit $?; fi\n",
    # 只在注释与出路文字里提到别人的 --selftest，不该算
    "mention-in-comment.sh": "#!/usr/bin/env bash\n# research/scripts/other.sh --selftest 里对应那一格判错\necho ok\n",
    "mention-in-howto.sh": "#!/usr/bin/env bash\necho \"  → 怎么办：单跑 bash research/scripts/other.sh --selftest 看它报什么\"\n",
}
SHOULD_CLAIM = {"new-getopts.sh", "new-match.py", "new-set-membership.py", "new-flag-constant.py", "new-if-test.sh"}


def main():
    root = sys.argv[1]
    real = {}
    for path in sorted(glob.glob(os.path.join(root, "research/scripts/*"))):
        if os.path.isfile(path):
            real[os.path.relpath(path, root)] = open(path, encoding="utf-8", errors="replace").read()
    print("== research/scripts/ 今天的文件：各认法认下的份数，与 R0 的差")
    base = {path for path, text in real.items() if claims_r0(path, text)}
    for name, recognizer in RECOGNIZERS:
        claimed = {path for path, text in real.items() if recognizer(path, text)}
        print(f"{name}\tclaimed={len(claimed)}\tR0 认而它不认={sorted(base - claimed)}\t它认而 R0 不认={sorted(claimed - base)}")
    print("== 临时造的七份：认下 = 1")
    print("文件\t该认\t" + "\t".join(name for name, _ in RECOGNIZERS))
    for file_name, text in SYNTHETIC.items():
        path = "research/scripts/" + file_name
        verdicts = "\t".join("1" if recognizer(path, text) else "0" for _, recognizer in RECOGNIZERS)
        print(f"{file_name}\t{'1' if file_name in SHOULD_CLAIM else '0'}\t{verdicts}")


main()
