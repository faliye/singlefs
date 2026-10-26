"""返工词表里 \bfix 只因 fixture / fixed-* 命中的派发（结局账每个子 agent 一行的 jsonl，outcome_dump.py 出的）。用法：python3 rework_fixture.py <closed-agents.jsonl>"""
import json, re, sys
agents = [json.loads(line) for line in open(sys.argv[1])]
total = sum(agent["cost"] for agent in agents)
words = re.compile(r"修(?!订)|重跑|重派|再派|重做|返工|续做|续派|接续|接手|接着做|（续）|补写|补做|改名|改号|改钉值|第\s*[2-9]\d*\s*次|redispatch|re-?run|renumber|retry|\bfix", re.I)
only = [agent for agent in agents if agent["rework"] and all(re.match(r"fix(ture|ed)", agent["description"][match.start():match.start() + 7], re.I) for match in words.finditer(agent["description"]))]
for agent in only:
    print(f"name=fixture_only type={agent['type']} description={agent['description'][:70]}")
print(f"name=fixture_only_total count={len(only)} cost_share={sum(agent['cost'] for agent in only) / total:.2%}")
