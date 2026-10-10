#!/usr/bin/env python3
"""从攻方腿的 make_device.py 生成辩方腿的 make_device.py：模块换成 w3r2_module.rs，自证测试多断言 C4g、C2i、C2ig 红。
用法：mk_make_device.py <攻方腿的 make_device.py> <输出 make_device.py>"""
import sys
src, out = sys.argv[1], sys.argv[2]
s = open(src, encoding="utf-8").read()
old = '''    assert!(!verdicts.c1 && verdicts.c2a && !verdicts.c2b && verdicts.c2c && verdicts.c2d && verdicts.c3 && verdicts.c4, "{verdicts:?}");
'''
assert s.count(old) == 1
s = s.replace(old, old + '''    assert!(verdicts.c4g && verdicts.c2i && verdicts.c2ig, "{verdicts:?}");
''')
old2 = 'module = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "w3_module.rs"), encoding="utf-8").read()'
assert s.count(old2) == 1
s = s.replace(old2, 'module = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "w3r2_module.rs"), encoding="utf-8").read()')
head = '"""从 E175 装置源码原样（device-source.rs）生成攻方腿的原型装置 w3_attack_candidates.rs。'
assert s.count(head) == 1
s = s.replace(head, '"""m3-w3-r2 辩方腿：从攻方腿的 make_device.py 改来，模块换成 w3r2_module.rs（加 C1b、C2i、C4g 与持久镜像计数）。\n原说明：从 E175 装置源码原样（device-source.rs）生成攻方腿的原型装置 w3_attack_candidates.rs。', 1)
open(out, "w", encoding="utf-8").write(s)
