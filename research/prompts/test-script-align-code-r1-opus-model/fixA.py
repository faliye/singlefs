# 原型改法 A（只在草稿副本上）：版本表与被判的根槽写下标写进每个点的计划哈希（进块键）
import sys
path = sys.argv[1]
s = open(path, encoding="utf-8").read()
old = '        plan_text.push_str(&format!("states={}\\n", ordinals.end - ordinals.start));\n'
assert s.count(old) == 1
new = ('        // 原型改法 A：版本表与被判的根槽写进块键\n'
       '        plan_text.push_str(&format!("versions={:?}\\njudged_root_index={}\\n", flow.versions, flow.judged_root_index));\n' + old)
open(path + ".tmp", "w", encoding="utf-8").write(s.replace(old, new))
