<!-- knowledge-sync -->
# e152-relay-and-on-request 阶段同步

触发文件：.claude/gate.d/47-research-script-selftests.sh、.claude/gate.d/65-relay-timing.sh、.claude/gate.d/69-evidence-in-repo.sh、.claude/gate.d/73-research-gate-lint.sh、.claude/gate.d/stage-owners.tsv、research/scripts/relay-timing-lint.py、research/scripts/e152-tables.py

范围：E152（按里程碑对比六家文件系统的文件性能） 装置改为先读完再转打的那一批（2026-09-17 另一个会话写、留在工作区）连同 2026-09-19 第五次正式跑的产物收尾提交；门禁 73 号在 relay-timing-lint.py 上的一处拒绝补出路；门禁 69 号加「按要求才跑的实验」（research/on-request-experiments.tsv，登记 E152，用户 2026-09-19 定）。

## 搜索

- `grep -rn 'on-request\|按要求才跑' .claude CLAUDE.md research/scripts` → 69 号、清单文件、E152 实验页三处，都是这一批写的
- `grep -rn '69-evidence\|69 号' .claude/agents .claude/rules .claude/main-agent.md .claude/agent-common.md` → 定义与规则里只在 stage-owners.tsv 登记归属，没有复述判据，不用跟改
- `bash .claude/gate.d/73-research-gate-lint.sh` → 51 个脚本、108 条拒绝都带出路

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:5 | 按里程碑对比的结果写在 `research/perf-by-milestone.md`。只出观测； | 补了：只在用户说跑时才跑，登记在 research/on-request-experiments.tsv，69 号改查这一页写没写没有正式跑 |
| .claude/gate.d/69-evidence-in-repo.sh:12 | 判据一「实验跑了没留存」 | 改了：按要求才跑的实验改查实验页的一句，不要求新产物；红绿样本各加一格 |
