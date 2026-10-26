"""shapecount.py <corpus.json>: 数语料（mkcorpus.py 的输出）里本报告各反例形状的条数（正则，粗数）。"""
import json, re, sys
c = json.load(open(sys.argv[1]))
pats = {
    "花括号空份（引号外，后缀位置也算）": r"(?<![\$'\"\\])\{(,[^{}\s,]*|[^{}\s,]*,)\}",
    "同一行双引号包着的 $(…) 之后接 shell 的 heredoc": r'"[^"\n]*\$\([^)\n]*\)[^"\n]*"[^\n]*\b(bash|sh)\b[^\n]*<<',
    "cat 源 > 目的 之后又有 2>": r"\bcat\s+[^|<>\n]+>\s*\S+\s+\d?>\s*\S+",
    "env 后面以 = 开头的词": r"\benv\s+(-\S+\s+)*=",
    "bash -c --": r"\bbash\s+(-\S*\s+)*-c\s+--\b",
    "bash -s --": r"\bbash\s+-s\s+--\b",
    "ln -s ../ 相对源": r"\bln\s+-s\w*\s+\.\./",
    "heredoc 喂 shell（任意）": r"\b(bash|sh)\b[^\n]*<<",
    "引号外带逗号的花括号": r"(?<![\$'\"\\])\{[^{}\s]*,[^{}\s]*\}",
}
print(len(c), "条")
for name, pattern in pats.items():
    print(sum(1 for x in c if re.search(pattern, x)), name)
