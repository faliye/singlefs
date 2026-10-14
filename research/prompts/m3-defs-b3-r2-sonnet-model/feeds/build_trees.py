# 把改前、改后两份冻结副本各拷成一棵可喂钩子的树：<工作目录>/tree-before、<工作目录>/tree-after（不拷 multi-host.env、research/ 大目录、target）。
# 用法：python3 -I build_trees.py <改前冻结副本> <改后冻结副本> <工作目录>
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as c
before, after, work = sys.argv[1:4]
os.makedirs(work, exist_ok=True)
c.build_tree(before, os.path.join(work, "tree-before"))
c.build_tree(after, os.path.join(work, "tree-after"))
print(f"name=build_trees before={before} after={after}")
