"""matrix.py <families.json> <orig-out.json> <new-out.json>: 每条一行 tag|hook|agent|改前rc|改后rc|命令sha8，供与 out/matrix.tsv 逐行比。"""
import json, sys, hashlib
o = json.load(open(sys.argv[2])); n = json.load(open(sys.argv[3]))
for a, b in zip(o, n):
    assert a["cmd"] == b["cmd"]
    print("|".join([b["tag"], b["hook"], b["agent"], str(a["rc"]), str(b["rc"]), hashlib.sha256(b["cmd"].encode()).hexdigest()[:8]]))
