#!/usr/bin/env python3
"""从仓里现在的 71 / 72 / 56 号各造几份「改回去」的副本，写进给定目录；判别力证明拿它们跑样本。

    python3 make-reverted.py <仓根> <出口目录>

71-item5-reverted.sh：④ 改回在整行上判（不剥「」）。
71-item6-reverted.sh：⑤ 五种词法一条都不找；71-item6-drop<N>.sh：只去掉第 N 种。
72-item8-reverted.sh、56-item8-reverted.sh：判决改回「改动范围里任何一份 *-main-verification.md」（被改过的旧判决也算）。
每处替换先断言恰好命中一次。
"""
import os, sys

root, out = sys.argv[1:3]
os.makedirs(out, exist_ok=True)

def read(name):
    return open(os.path.join(root, ".claude/gate.d", name), encoding="utf-8").read()

def write(name, text):
    open(os.path.join(out, name), "w", encoding="utf-8").write(text)

def replace_once(text, old, new):
    assert text.count(old) == 1, old
    return text.replace(old, new)

gate71 = read("71-agent-def-flow-only.sh")
write("71-item5-reverted.sh", replace_once(gate71,
    """            for match in HALF_SENTENCE.finditer(unquoted_line):
                start = max(0, match.start() - 12)
                explanatory_half_sentences.append(f"{location}（「{match.group(1)}」）：…{unquoted_line[start:match.end() + 24].strip()}…")""",
    """            for match in HALF_SENTENCE.finditer(line):
                start = max(0, match.start() - 12)
                explanatory_half_sentences.append(f"{location}（「{match.group(1)}」）：…{line[start:match.end() + 24].strip()}…")"""))
start = gate71.index("LEXICAL_EXPLANATIONS = [")
end = gate71.index("]\n", start) + 2
write("71-item6-reverted.sh", gate71[:start] + "LEXICAL_EXPLANATIONS = []\n" + gate71[end:])
entries = [line for line in gate71[start:end].split("\n") if line.startswith("    (")]
assert len(entries) == 5
for number, entry in enumerate(entries, 1):
    write(f"71-item6-drop{number}.sh", replace_once(gate71, entry + "\n", ""))

gate72 = read("72-agent-def-adversarial-review.sh")
write("72-item8-reverted.sh", replace_once(gate72,
    """verdicts="$(grep -E '^research/prompts/.*-main-verification\\.md$' <<<"$added" || true)\"""",
    """verdicts="$(grep -E '^research/prompts/.*-main-verification\\.md$' <<<"$changed" || true)\""""))
gate56 = read("56-crates-adversarial-review.sh")
write("56-item8-reverted.sh", replace_once(gate56,
    """verdicts="$(grep -E '^research/prompts/.*-main-verification\\.md$' <<<"$added" || true)\"""",
    """verdicts="$(grep -E '^research/prompts/.*-main-verification\\.md$' <<<"$changed" || true)\""""))
print(f"写了 {len(os.listdir(out))} 份副本到 {out}")
