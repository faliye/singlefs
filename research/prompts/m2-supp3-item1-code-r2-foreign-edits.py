python3 - <<'EOF'
import importlib.util
spec=importlib.util.spec_from_file_location('ro','research/scripts/replace-once.py');ro=importlib.util.module_from_spec(spec);spec.loader.exec_module(ro)
rc,m=ro.replace_once('.claude/kb/experiments.md','树表第二层门槛每头一棵 livelist **44** 头、共享树 **66** 头，第三层 5985 / 8977 头；','树表第二层门槛每头一棵 livelist **27** 头、共享树 **39** 头，第三层 2187 / 3279 头（2026-09-16 按树表条目 200、每层 81 棵重跑）；'); print(rc); assert rc==0,m
M='.claude/kb/milestone/02-second-txn.md'
E=[
('代码与 E142（第一个事务的干跑） 都按 1，改它要重跑 E142（第一个事务的干跑），2026-09-16 核出、没改）','代码与 E142（第一个事务的干跑） 都按 1，改它要重跑 E142（第一个事务的干跑），2026-09-16 核出；2026-09-18 已改成 3 并重跑，E142（第一个事务的干跑） 第十一次跑）'),
('今天没有条款说 checker 怎么认（','「非空」的认法 2026-09-17 已写进 D16（发布语义） 已定项 1 表格后段（'),
('；checker 怎么从盘上认「非空持久有效根」；','；checker 怎么从盘上认「非空持久有效根」（2026-09-17 已定，D16（发布语义） 已定项 1 表格后段）；'),
('- E152（按里程碑对比六家文件系统的文件性能） 的 singlefs 臂今天量的是「所有树第一次落盘的那一个事务」；第二次写要量 B 的写数与字节数，与六家「刚格式化之后的第二次写」对。','- E152（按里程碑对比六家文件系统的文件性能） 的 singlefs 臂量「所有树第一次落盘的那一个事务」与第二次写（发布 B）的写数与字节数，与六家「刚格式化之后的第二次写」对（第三、四次跑，见步 7 现状）。'),
('按条目宽 200 重跑的结果在工作区、决策正文没跟着改；','按条目宽 200 重跑的结果已提交；D6（快照实现模型） 正文与实验索引那一行 2026-09-18 已改成 39 头 / 27 头（知识腐烂回扫）；'),
]
for o,n in E:
    rc,m=ro.replace_once(M,o,n); print(rc); assert rc==0,(m,o[:40])
EOF