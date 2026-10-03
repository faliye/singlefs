#!/usr/bin/env python3
"""patch_e165.py 之后再打的一处：在各域循环之前把阳性对照等攒下的计数打出来清零，免得记到第一个域头上。
用法：patch_e165_precontrol.py <e165_replay_bounded_enumeration.rs>
"""
import pathlib, sys
p = pathlib.Path(sys.argv[1]); s = p.read_text()
old = "    let mut total_states: u64 = 0;\n    for domain in Domain::ALL {"
assert s.count(old) == 1
p.write_text(s.replace(old, "    s6_dump_and_reset(\"name=precontrol\");\n" + old))
