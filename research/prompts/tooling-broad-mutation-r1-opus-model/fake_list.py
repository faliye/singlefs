#!/usr/bin/env python3
"""攻方腿自己的假清单（调法 <包>）：把 PROBE_WORK/mutants.json 里这个包的项打出来。"""
import json, os, sys
print(json.dumps([x for x in json.load(open(os.path.join(os.environ["PROBE_WORK"], "mutants.json"))) if x["package"] == sys.argv[1]]))
