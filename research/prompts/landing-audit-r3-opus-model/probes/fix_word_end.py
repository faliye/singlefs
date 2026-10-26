r"""全部子 agent 的派发描述：改前 \bfix 认、改后 \bfix(?:es|ed)?(?![\w-]) 不认、别的返工词也不中的，按命中的词分组。用法：fix_word_end.py <会话目录根>"""
import glob, json, os, re, collections, sys
before = re.compile(r"\bfix", re.I); after = re.compile(r"\bfix(?:es|ed)?(?![\w-])", re.I)
rest = re.compile(r"修(?!订)|重跑|重派|再派|重做|返工|续做|续派|接续|接手|接着做|（续）|补写|补做|改名|改号|改钉值|第\s*[2-9]\d*\s*次|redispatch|re-?run|renumber|retry", re.I)
lost = collections.Counter(); examples = collections.defaultdict(list); n = 0
for meta in glob.glob(os.path.join(sys.argv[1], "*", "subagents", "agent-*.meta.json")):
    d = re.sub(r"\s+", " ", json.load(open(meta)).get("description") or ""); n += 1
    if before.search(d) and not after.search(d) and not rest.search(d):
        word = re.search(r"\bfix[\w-]*", d, re.I).group(0).lower()
        lost[word] += 1; examples[word].append(d[:70])
print("描述", n, "只因尾词界不再算返工的", sum(lost.values()), dict(sorted(lost.items())))
for w, ex in sorted(examples.items()): print("  ", w, sorted(ex)[:4])
