# sweep-acceptance-2026-09-18-v3-judge-6：逐行判（D1, E1, F7b）

阶段：sweep-acceptance-v3-judge-6。改动范围：基准提交 b1c8cef~1，结束提交 00c9d4f
（本报告的上下文一律来自 `git show 00c9d4f:路径`，不读工作区）。
候选表：`research/prompts/sweep-acceptance-2026-09-18-v2-candidates.tsv`；
事实表：同目录 `sweep-acceptance-2026-09-18-v2-facts.tsv`。分到的组：D1、E1、F7b。

判据照 `.claude/agents/sweep.md` 第 9 步：把候选行里描述现状的分句换成事实表里的「新事实」，
原句还是不是真话；带日期的现状句（即使标了日期）仍按现状句判，只有说「那一天发生了什么」的才算事件句。

三个分组对应的事实（事实表原文）：

- **D1**：旧事实「inode 记录偏移 88 改动计数」；新事实「四的 inode 记录偏移 88 改动计数改写 3（= 发布它的 checkpoint_txg）」——
  即 E142（第一个事务的干跑）第十一次跑已把该字段从字面 1 改成 checkpoint_txg=3，并与 `crates/` 实装比对 21/21 全等。
- **E1**：旧事实是 CLAUDE.md 旧句「层 0 崩溃点重放（门禁 54 号）的负载还只有第一个事务，覆盖写、释放、多次挂载、回退都没进来，
  checker 也只判第一版那部分不变量」；新事实「新开 layout/02-second-txn.md 登记第二个事务写出的五种盘上形态；
  发布 B 真设备两盘合计 21 次写调用（块层记 26 次）、344 576 字节、4 屏障、1 FUA」。
- **F7b**：旧事实「C314（回退可以复用被抛弃的根引用的单元） 记着『检查仍欠：崩溃点重放 harness（多次挂载的录制流）』——
  复用被抛弃的根引用的单元这条路径还没有实做」；新事实「步 5（抬回退下界 F 之后复用第一个数据单元的落点）落地：
  `raise_rollback_floor`、`reclaim_released_up_to`；层 0 固定脚本到 E（54 段、2104413 个状态）」。

三组候选行数：D1 132 行、E1 26 行、F7b 37 行，共 195 行，逐行判过，不抽样。

## 逐行判定

### D1（`.claude/kb/checks-owed.md`「检索词」等：inode 记录偏移 88 改动计数）

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| D1 | .claude/kb/checks-owed.md:52 | 不相干 | C42 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/checks-owed.md:87 | 不相干 | C77 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/checks-owed.md:88 | 不相干 | C78 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/checks-owed.md:95 | 不相干 | C85 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/checks-owed.md:98 | 不相干 | C88 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/checks-owed.md:151 | 不相干 | C149 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/checks-owed.md:200 | 不相干 | C199 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/checks-owed.md:259 | 不相干 | C278 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/checks-owed.md:263 | 不相干 | C282 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/checks-owed.md:268 | 不相干 | C287 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/checks-owed.md:292 | 不相干 | C314 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/checks-owed.md:301 | 不相干 | C330 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/checks-owed.md:302 | 不相干 | C331 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/checks-owed.md:306 | 不相干 | C335 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions.md:46 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions.md:219 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/03-空间分配.md:180 | 不相干 | C113 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/03-空间分配.md:187 | 不相干 | D22（单元原子性怎么合成） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/05-快照-空间记账机制.md:268 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/05-快照-空间记账机制.md:445 | 不相干 | D22（单元原子性怎么合成） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/05-快照-空间记账机制.md:481 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/08-核心索引结构.md:372 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/08-核心索引结构.md:398 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/08-核心索引结构.md:403 | 不相干 | E98（inode 记录与 inode 树的几何） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/08-核心索引结构.md:479 | 不相干 | D6（快照实现模型） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/09-加密.md:30 | 不相干 | D22（单元原子性怎么合成） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/13-验证路线.md:235 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/13-验证路线.md:422 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/16-发布语义.md:113 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/16-发布语义.md:209 | 不相干 | E150（回退复用被抛弃的根引用的单元） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/16-发布语义.md:211 | 不相干 | C314 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/16-发布语义.md:215 | 不相干 | E148（提交固定点按两棵记录树重算） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/16-发布语义.md:248 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/16-发布语义.md:251 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/16-发布语义.md:273 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/16-发布语义.md:313 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/16-发布语义.md:372 | 不相干 | C282 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/18-块里携带什么信息.md:648 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/18-块里携带什么信息.md:820 | 不相干 | E85（单元头的字段表） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/18-块里携带什么信息.md:834 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/18-块里携带什么信息.md:876 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/18-块里携带什么信息.md:882 | 不相干 | D5（快照 / 空间记账机制） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:196 | 不相干 | D19（块指针的结构与宽度预算） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/21-权威态与派生态的分界.md:419 | 不相干 | D22（单元原子性怎么合成） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/21-权威态与派生态的分界.md:422 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:141 | 不相干 | D23（journal 的角色与格式） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:143 | 不相干 | D5（快照 / 空间记账机制） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:488 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:489 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:500 | 不相干 | E142（第一个事务的干跑） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:505 | 不相干 | D9（加密） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:521 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:546 | 不相干 | I-7.1 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:48 | 不相干 | D5（快照 / 空间记账机制） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:83 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:264 | 不相干 | D22（单元原子性怎么合成） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:509 | 不相干 | E32（上一条时间线的残留） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:600 | 不相干 | I-8.3 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:677 | 不相干 | E32（上一条时间线的残留） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:691 | 不相干 | C113 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:799 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:807 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:924 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:1137 | 不相干 | D5（快照 / 空间记账机制） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:1209 | 不相干 | C113 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:1211 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:1218 | 不相干 | D22（单元原子性怎么合成） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:1222 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:1239 | 不相干 | D22（单元原子性怎么合成） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:1240 | 不相干 | E36（槽位映射那一维） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments.md:160 | 事件句不改 | 这是 experiments.md 里 E142（第一个事务的干跑）的跑次汇总，逐字记的是『第十一次跑把改动计数字段从字面 1 改成 checkpoint_txg=3……与 crates/ 实装比对 21/21 全等』——说的正是新事实本身那次发生的事，换成新事实原句仍然为真，不改。 |
| D1 | .claude/kb/experiments/104-扫描重建的现行版本判定.md:11 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/142-第一个事务的干跑.md:63 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/142-第一个事务的干跑.md:84 | 不相干 | E7 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/142-第一个事务的干跑.md:166 | 不相干 | D23（journal 的角色与格式） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/23-journal几何.md:77 | 不相干 | D23（journal 的角色与格式） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:93 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:8 | 不相干 | I-8.3 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/61-反向链hash算法的均匀性.md:37 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/78-重放的起点.md:38 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/79-根记录的容量.md:11 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/88-冒名单元与孤儿单元的判别.md:10 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/91-已定格式下的环账与准入账.md:19 | 不相干 | D23（journal 的角色与格式） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/91-已定格式下的环账与准入账.md:21 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:27 | 不相干 | D22（单元原子性怎么合成） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:131 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/98-inode记录与inode树的几何.md:18 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/98-inode记录与inode树的几何.md:32 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/98-inode记录与inode树的几何.md:115 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/98-inode记录与inode树的几何.md:117 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:14 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:21 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:22 | 不相干 | D22（单元原子性怎么合成） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:25 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:27 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:48 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:56 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:76 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/invariants.md:25 | 不相干 | I-1.2 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/invariants.md:57 | 不相干 | I-7.8 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/invariants.md:131 | 不相干 | I-3.9 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/layout/01-first-txn.md:61 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/layout/01-first-txn.md:64 | 不相干 | C32 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/layout/01-first-txn.md:75 | 不相干 | C32 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/layout/01-first-txn.md:175 | 不相干 | D18（块里携带什么信息） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/layout/01-first-txn.md:177 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/layout/01-first-txn.md:257 | 不相干 | D8（核心索引结构） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/layout/01-first-txn.md:285 | 不相干 | E142（第一个事务的干跑） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/layout/01-first-txn.md:290 | 不相干 | D5（快照 / 空间记账机制） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/layout/01-first-txn.md:331 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/layout/01-first-txn.md:356 | 不相干 | D22（单元原子性怎么合成） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/milestone/01-first-txn.md:58 | 不相干 | D22（单元原子性怎么合成） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/milestone/01-first-txn.md:86 | 不相干 | D8（核心索引结构） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/milestone/01-first-txn.md:163 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/milestone/01-first-txn.md:170 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/milestone/01-first-txn.md:177 | 不相干 | D23（journal 的角色与格式） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/milestone/01-first-txn.md:191 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/milestone/01-first-txn.md:198 | 不相干 | D22（单元原子性怎么合成） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/milestone/02-second-txn.md:100 | 不相干 | D19（块指针的结构与宽度预算） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/milestone/02-second-txn.md:113 | 要改 | milestone/02-second-txn.md「会碰到的决策点」逐字仍写『代码与 E142（第一个事务的干跑） 都按 1，改它要重跑 E142（第一个事务的干跑），2026-09-16 核出、没改』——按新事实这句已经不是真话（E142 第十一次跑已把该字段改成 checkpoint_txg=3，并与 crates/ 比对 21/21 全等）。改成：「D8（核心索引结构） 已定项 6（改动计数 = 最后一次改动所在发布的 checkpoint_txg，这次写 4；第一个事务写 3——已按 E142（第一个事务的干跑）第十一次跑改成 checkpoint_txg=3，并与 crates/ 实装比对 21/21 全等，2026-09-18 起字段与字段表定义一致）；……」 |
| D1 | .claude/kb/milestone/02-second-txn.md:149 | 不相干 | D18（块里携带什么信息） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/milestone/02-second-txn.md:156 | 不相干 | I-3.8 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/milestone/02-second-txn.md:166 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/milestone/02-second-txn.md:169 | 不相干 | D23（journal 的角色与格式） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/verification-build.md:66 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | .claude/kb/verification-build.md:67 | 不相干 | D22（单元原子性怎么合成） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | records/2026-08-29-组合对攻轮.md:52 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | records/2026-09-06-树ID水位臂比较重做.md:15 | 不相干 | 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | records/2026-09-13-总审核.md:13 | 不相干 | C314 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | records/2026-09-13-总审核.md:454 | 不相干 | I-7.8 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | records/2026-09-13-未定项总清单与待用户定案.md:29 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |
| D1 | records/2026-09-13-未定项总清单与待用户定案.md:64 | 不相干 | D16（发布语义） 这一行讲的是别的字段或别的机制里用到 checkpoint_txg 的地方，不是 inode 记录偏移 88 这个『改动计数』字段本身取值是否已从字面 1 改成 checkpoint_txg=3；换成新事实，原句真假不受影响。 |

### E1（CLAUDE.md：层 0 崩溃点重放负载）

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| E1 | .claude/kb/checks-owed.md:33 | 要改 | checks-owed.md C22 的『前置』列仍写『崩溃点重放 harness（四样 2026-09-14 已有……层 0 的负载只有第一个事务，没有释放与重用）』——按新事实这半句已不是真话（层 0 第二条流已覆盖发布 B 的覆盖写与释放，I-4.8 已实现）。改成：「事务层、分配器、根环、崩溃点重放 harness（四样均已具备：`crates/singlefs-core`、门禁 54 号；层 0 第二条流已覆盖发布 B 的覆盖写、释放与重用）」。 |
| E1 | .claude/kb/checks-owed.md:354 | 不相干 | C380 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/checks-owed.md:403 | 不相干 | C374 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/decisions/23-journal的角色与格式.md:691 | 不相干 | C113 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/invariants.md:12 | 不相干 | I-3.8 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/invariants.md:53 | 不相干 | I-7.4 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/invariants.md:110 | 不相干 | I-2.1 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/invariants.md:123 | 不相干 | I-3.1 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/invariants.md:131 | 不相干 | I-3.9 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/invariants.md:157 | 不相干 | I-4.8 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/invariants.md:173 | 不相干 | I-5.4 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/invariants.md:275 | 不相干 | I-9.14 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/milestone/02-second-txn.md:14 | 事件句不改 | milestone/02-second-txn.md 开篇『题面』表明写『按当日的判断写……它留下的、仓里各处已经写明「要第二个事务」的东西，就是这个里程碑的题面』，这一行是『项目 CLAUDE.md 一句话版本』这个来源在 2026-09-16 建档时的逐字引用——记的是建档那一刻 CLAUDE.md 说了什么，不是现在 CLAUDE.md 说什么，换成新事实不影响这句『引用属实』的真假。 |
| E1 | .claude/kb/milestone/02-second-txn.md:74 | 不相干 | C22 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/milestone/02-second-txn.md:141 | 不相干 | 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/milestone/02-second-txn.md:166 | 不相干 | 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/milestone/02-second-txn.md:195 | 不相干 | 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/milestone/02-second-txn.md:358 | 不相干 | I-3.9 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/verification-build.md:139 | 不相干 | I-3.8 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/verification-build.md:155 | 要改 | verification-build.md 逐字写『2026-09-14 现状：……其余七条要的覆盖写、释放、多次挂载、回退还不在层 0 的负载里。』这是写在正文里（不在历史版本节）的现状句，按新事实已不成立。改成：「2026-09-18 现状：层 0 第二条流（固定脚本到 E）已覆盖覆盖写、释放、多次挂载、回退——发布 B 真设备两盘合计 21 次写调用（块层记 26 次）、344 576 字节、4 屏障、1 FUA，五种盘上形态见 [layout/02-second-txn.md](layout/02-second-txn.md)。」 |
| E1 | .claude/kb/verification-build.md:157 | 不相干 | 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/verification-build.md:163 | 不相干 | C314 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/verification-build.md:248 | 不相干 | E77 这一行讲的是已实现、已落地或另一件事的现状/设计，不是在断言『层 0 崩溃点重放的负载还只有第一个事务、缺覆盖写/释放/多次挂载/回退』——换成新事实，原句真假不受影响（多数本身已与新事实一致，或压根不是这句话）。 |
| E1 | .claude/kb/verification-build.md:249 | 事件句不改 | verification-build.md 这一行是三方对抗轮『本地（找反例）』腿的结论记录：『释放路径的 bug 在第一个事务里不触发……⇒ 层 0 负载要有释放』，说的是那一轮论证当时发现了什么、得出什么要求，是那次发生的事，不是现在层 0 有没有释放的现状断言，换成新事实仍然为真。 |
| E1 | CLAUDE.md:97 | 不相干 | 这一行就是 CLAUDE.md 在 00c9d4f 这一版的现有正文：『层 0 崩溃点重放（门禁 54 号）的负载是两条流：第一个事务，以及固定脚本到 E……checker 判 29 条不变量』——它已经是新事实本身，不是旧事实『还只有第一个事务』那句，谈不上要不要换。 |
| E1 | records/2026-09-03-验证三件套落地调研.md:60 | 事件句不改 | records/2026-09-03-验证三件套落地调研.md 是按日期归档的调研记录（2026-09-03），逐字『本地腿与正推腿各自指出释放路径在第一个事务里不触发』——记的是那次调研发现了什么，是那一次发生的事，records/ 按日期不回改。 |

### F7b（C314：崩溃点重放 harness 多次挂载录制流）

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F7b | .claude/kb/checks-owed.md:200 | 事件句不改 | checks-owed.md C199 这一行是 2026-09-08 至 2026-09-14 三方论证与用户复核的历史叙述（『2026-09-13 第一轮……2026-09-13 第二轮……2026-09-14 用户复核确认』），文中提到 C314 只是引它当当时讨论的相关条目、当时建议『先还 C314 再取暖机』——记的是那几轮论证当时发生了什么，不是在断言 C314 今天还没还清，换成新事实仍然为真。 |
| F7b | .claude/kb/checks-owed.md:262 | 要改 | checks-owed.md C281 第四列逐字『决策侧已还……检查仍欠：事务层、管理员回退实现、崩溃点重放 harness』——按新事实这半句已不是真话：管理员回退（`mount_rollback`）与事务层实现均已落地（milestone/02-second-txn.md「现状（2026-09-17 落地）」），崩溃点重放 harness 层 0 固定脚本到 E（54 段、2104413 个状态，含步 5 `raise_rollback_floor`、`reclaim_released_up_to`）已覆盖这条路径。改成：「决策侧已还（……）；检查已还：事务层与管理员回退实现（`mount_rollback`）已落地，崩溃点重放 harness（层 0 固定脚本到 E，54 段、2104413 个状态）已覆盖」。 |
| F7b | .claude/kb/checks-owed.md:292 | 要改 | checks-owed.md C314 自己的登记行第四列逐字『条款已写……检查仍欠：崩溃点重放 harness（多次挂载的录制流）；查账集合 2026-09-16 用户定案取窄读法』——按新事实这半句已不是真话（层 0 固定脚本到 E，步 5 `raise_rollback_floor`、`reclaim_released_up_to` 落地）。改成：「条款已写（……）；检查已还：崩溃点重放 harness（多次挂载的录制流）——层 0 固定脚本到 E（54 段、2104413 个状态），步 5 `raise_rollback_floor`、`reclaim_released_up_to` 落地；查账集合 2026-09-16 用户定案取窄读法（只隔离被抛弃根独占的槽，D23 已定项 14）」。 |
| F7b | .claude/kb/checks-owed.md:296 | 不相干 | C318 讲的是『影子账隔离的单元没进准入不等式』这条独立的记账口径缺口，不是崩溃点重放 harness 有没有实做，换成新事实原句真假不受影响。 |
| F7b | .claude/kb/checks-owed.md:328 | 不相干 | C356 讲的是第九项统计量之差算出负值、没有钳位的算术缺口，不是崩溃点重放 harness 有没有实做，换成新事实原句真假不受影响。 |
| F7b | .claude/kb/decisions/16-发布语义.md:189 | 不相干 | decisions/16-发布语义.md 已定项 8 的正文『前置 C314 先还。状态：已定。』是这条决策自身的前置条件说明（决策已拍板，但要求先还清 C314 才实现），不是在断言 C314『今天』还没还清，这句作为决策记录本身仍然成立。 |
| F7b | .claude/kb/decisions/16-发布语义.md:205 | 不相干 | 该小标题只是复述『暖机取甲′，先还 C314』这个决策条件，不是在断言 C314 现在的完成状态，换成新事实原句仍然成立。 |
| F7b | .claude/kb/decisions/16-发布语义.md:207 | 要改 | decisions/16-发布语义.md 已定项 8 正文逐字『**检查**那一半仍欠（崩溃点重放 harness 的多次挂载录制流），C314 因此仍在未还表。』——按新事实已不是真话。改成：「**检查**那一半已还（崩溃点重放 harness 的多次挂载录制流——层 0 固定脚本到 E，步 5 `raise_rollback_floor`、`reclaim_released_up_to` 落地），C314（回退可以复用被抛弃的根引用的单元） 因此已从未还表移出。」 |
| F7b | .claude/kb/decisions/16-发布语义.md:209 | 要改 | decisions/16-发布语义.md 已定项 8 的『前置的进展』注逐字『……崩溃点重放 harness 那条检查仍欠；暖机在一块盘的模型里把……』——按新事实已不是真话。改成：「……崩溃点重放 harness 那条检查已还（层 0 固定脚本到 E，步 5 落地）；暖机在一块盘的模型里把『落到被抛弃的根上』要的故障数从 1 抬到 4（按根槽数说）。」 |
| F7b | .claude/kb/decisions/16-发布语义.md:211 | 要改 | decisions/16-发布语义.md 已定项 8『代价』一段逐字『C314 还清之后落到 layout/01-first-txn.md 与 E142，在那之前字节表暂按 1』——按新事实 C314 已还清，字节表也已按 3 落地（E142 第十一次跑、layout/01-first-txn.md）。改成：「C314（回退可以复用被抛弃的根引用的单元） 已还清，改动已落到 [layout/01-first-txn.md] 与 E142（第一个事务的干跑）第十一次跑，字节表按 3；……」 |
| F7b | .claude/kb/decisions/16-发布语义.md:224 | 事件句不改 | 这一段『> 建议：先还 C314，再取甲′……』是当时给用户的决策建议、说明为什么要先修 C314 再选臂，是那次论证给出的理由记录，不是在断言 C314 现在还没修，换成新事实仍然为真。 |
| F7b | .claude/kb/decisions/16-发布语义.md:291 | 要改 | decisions/16-发布语义.md:291 逐字『前置 C314（回退可以复用被抛弃的根引用的单元） 还清后生效。』——按新事实 C314 已还清，暖机已生效（同文档 209 行已写『暖机生效』）。改成：「……前置 C314（回退可以复用被抛弃的根引用的单元） 已还清，暖机已生效。」 |
| F7b | .claude/kb/decisions/23-journal的角色与格式.md:1209 | 不相干 | decisions/23-journal的角色与格式.md 已定项 14 的回退例外正文讲的是回退机制本身的定义（影子账、窄读法等），是已定案且已描述为生效的机制，不是在断言崩溃点重放 harness 还没做，换成新事实原句仍然成立。 |
| F7b | .claude/kb/decisions/23-journal的角色与格式.md:1245 | 不相干 | 这一行是指出『那次发布之前都不生效』这句要靠影子账才成立的技术说明，并链到 checks-owed.md 的 C314，本身不断言 harness 状态，换成新事实原句仍然成立。 |
| F7b | .claude/kb/decisions/28-挂载期承诺量.md:30 | 不相干 | decisions/28-挂载期承诺量.md 讲的是第九项『被抛弃根独占量』这个准入公式怎么算、由谁维护，是已实现的记账口径说明，不是崩溃点重放 harness 有没有做，换成新事实原句仍然成立。 |
| F7b | .claude/kb/experiments.md:168 | 事件句不改 | experiments.md 里 E150 这一行是『已跑（2026-09-13 两次……）』的实验汇总，记的是那次计数模型实验测了什么、得出什么结论，是那次发生的事，换成新事实仍然为真。 |
| F7b | .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:1 | 事件句不改 | experiments/150-*.md 的标题『已跑（2026-09-13 两次……）』记的是那次实验的发生与产物，是事件记录，换成新事实仍然为真。 |
| F7b | .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:4 | 事件句不改 | experiments/150-*.md「问题」一节描述那次实验要回答的问题与背景，是实验设计记录，不是崩溃点重放 harness 现状的断言，换成新事实仍然为真。 |
| F7b | .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:8 | 事件句不改 | experiments/150-*.md「结论（2026-09-13 跑完）」记的是那次计数模型实验测出的数字与两种修法的代价，是那次发生的事，换成新事实仍然为真。 |
| F7b | .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:74 | 事件句不改 | experiments/150-*.md 这一行是实验记录表里『要回答的一问』的答案汇总，属于那次实验产物的一部分，是事件记录，换成新事实仍然为真。 |
| F7b | .claude/kb/invariants.md:53 | 不相干 | invariants.md I-7.4 的状态列已写『已实现（2026-09-17……层 0 每个崩溃状态都判）』，与新事实一致，不是在断言 harness 缺失；这一行讲的是不变量定义与判别力本身，换成新事实原句仍然成立。 |
| F7b | .claude/kb/milestone/01-first-txn.md:184 | 要人看 | milestone/01-first-txn.md「会碰到的决策点」写『暖机取甲′、前置 C314；还清后第一个事务之前补两次空发布，还清前字节表按不补』——这是里程碑一（已收口于 2026-09-14）当时未决时写的条件分支，按新事实 C314 已还清、字节表也已按补两次空发布落地，但分不清这份已收口里程碑的『决策点』清单是要保持只读的历史存档，还是要随 C314 还清同步更新，需要人判。 |
| F7b | .claude/kb/milestone/01-first-txn.md:231 | 不相干 | milestone/01-first-txn.md 这一行讲的是里程碑一自己范围内『释放/重用/残留三条要等发布 B 与 C，回退那两格要等回退进里程碑』——这是给里程碑一定范围用的正确表述（里程碑一本来就不含回退），不因 C314 后来还清而改变，换成新事实原句仍然成立。 |
| F7b | .claude/kb/milestone/02-second-txn.md:9 | 事件句不改 | milestone/02-second-txn.md 开篇『题面』表逐字写『按当日的判断写……它留下的、仓里各处已经写明「要第二个事务」的东西，就是这个里程碑的题面』，这一行是『层 0 的固定负载』这条来源在 2026-09-16 建档时的逐字引用（当时随 C314 定案加『回退到 A、D』），记的是建档那一刻的判断，换成新事实不影响这句『引用属实』的真假。 |
| F7b | .claude/kb/milestone/02-second-txn.md:74 | 不相干 | milestone/02-second-txn.md 这一行讲的是只供测试的开关设计（强制抬 F、复用窗口置 0、关掉影子账等），是仍在用的测试基础设施说明，不是在断言崩溃点重放 harness 缺失，换成新事实原句仍然成立。 |
| F7b | .claude/kb/milestone/02-second-txn.md:166 | 不相干 | milestone/02-second-txn.md「现状（2026-09-17 落地）」逐段描述 `mount_rollback` 的实现细节，是已落地代码的准确现状描述，与新事实一致，不需要改。 |
| F7b | .claude/kb/milestone/02-second-txn.md:186 | 不相干 | milestone/02-second-txn.md 这一行是『必红』验收要求的规格描述（关掉影子账两格必须红、打开必须 0 违例），是设计规格，不因是否已实现而改变，换成新事实原句仍然成立。 |
| F7b | .claude/kb/milestone/02-second-txn.md:191 | 不相干 | milestone/02-second-txn.md 这一行只是列出步 4 / 步 5 会碰到的决策点名单（C314、C281 等条目名），不断言它们的完成状态，换成新事实原句仍然成立。 |
| F7b | .claude/kb/milestone/02-second-txn.md:332 | 事件句不改 | milestone/02-second-txn.md 这一行逐字写『C281 / C314 的「前置」列过时（写着「检查仍欠：崩溃点重放 harness」，而全量枚举与定向自证今天都有）』——这本身就是 2026-09-17 那次逐句核对（`research/prompts/m2-closeout-debts-check.md`）发现的问题记录，是那次核对当时发生的事，而且正好印证了本组第 2、3 行『要改』的判断；换成新事实，这句『发现了过时』仍然为真，不改。 |
| F7b | .claude/kb/verification-build.md:74 | 不相干 | verification-build.md 这一行讲的是准入公式『可用 = 容量 − 已分配 − defer 待释放 − 被抛弃根独占量』怎么核对，是已实现的记账口径说明，不是崩溃点重放 harness 状态，换成新事实原句仍然成立。 |
| F7b | .claude/kb/verification-build.md:153 | 不相干 | verification-build.md 这一行是『放开对被抛弃的根的保护』这条必红用例的规格描述（两格必须红、影子账打开必须 0 违例），是设计规格，不因是否已实现而改变，换成新事实原句仍然成立。 |
| F7b | .claude/kb/verification-build.md:163 | 要改 | verification-build.md:163 逐字『层 0 的固定脚本因此是 mkfs、A、B、C、回退到 A、D（2026-09-13 随 C314 定案登记）』——按新事实层 0 固定脚本已扩展到 E（步 5 `raise_rollback_floor`、`reclaim_released_up_to` 落地，54 段、2104413 个状态），只写到 D 已经不完整。改成：「层 0 的固定脚本因此是 mkfs、A、B、C、回退到 A、D、E（2026-09-13 随 C314 定案登记，步 5 `raise_rollback_floor`/`reclaim_released_up_to` 落地后扩展到 E：54 段、2104413 个状态）。」 |
| F7b | records/2026-09-13-总审核.md:13 | 事件句不改 | records/2026-09-13-总审核.md 是按日期归档的记录（2026-09-13），逐字『C314 那一行逐字列过该改哪几节，改的人做了零与八、漏了六与七』——记的是那次总审核发现了什么，是那一次发生的事，records/ 按日期不回改。 |
| F7b | records/2026-09-13-总审核.md:165 | 事件句不改 | records/2026-09-13-总审核.md 这一行是那次总审核表格里对 D16 已定项 8 定案句的评语，记的是那次审核发现的问题，是那一次发生的事，records/ 按日期不回改。 |
| F7b | records/2026-09-13-总审核.md:205 | 事件句不改 | records/2026-09-13-总审核.md 这一行是那次总审核列出的『挡路的欠账 14 条』清单，记的是那一天的未还表快照，是那一次发生的事，records/ 按日期不回改。 |
| F7b | records/2026-09-13-未定项总清单与待用户定案.md:29 | 事件句不改 | records/2026-09-13-未定项总清单与待用户定案.md 是按日期归档的记录，这一行记的是那天关于 D16 已定项 8 暖机方案候选与建议的清单，是那一次发生的事，records/ 按日期不回改。 |
| F7b | records/2026-09-13-未定项总清单与待用户定案.md:64 | 事件句不改 | records/2026-09-13-未定项总清单与待用户定案.md 这一行记的是那天甲′方案与『C314 还清前字节表按 checkpoint_txg 1』的约定，是那一次发生的事，records/ 按日期不回改。 |

## 反向核（第 10 步）：这三件做成的事，各自的载体里有没有记着

- D1「E142 第十一次跑把改动计数字段从字面 1 改成 checkpoint_txg=3，与 crates/ 比对 21/21 全等」：
  已记在 `.claude/kb/experiments.md:160`（E142 汇总行），本组 D1-71 判事件句不改，已算记过，不补。
- E1「层 0 第二条流已覆盖发布 B 的覆盖写/释放，两盘合计 21 次写调用（块层 26 次）、344 576 字节、4 屏障、1 FUA」：
  已记在 CLAUDE.md:97（本组 E1-25）、`.claude/kb/experiments.md:170`、`.claude/kb/experiments-history.md:34`（`git grep -c "344 576" 00c9d4f -- .claude/kb` 命中 3 处），不补。
- F7b「步 5 `raise_rollback_floor`、`reclaim_released_up_to` 落地；层 0 固定脚本到 E（54 段、2104413 个状态）」：
  已记在 `.claude/kb/milestone/02-second-txn.md`（`git grep -c raise_rollback_floor 00c9d4f -- .claude/kb` 命中该文件 4 处、
  `git grep -c reclaim_released_up_to` 命中 1 处、`git grep -c 2104413 00c9d4f -- .claude/kb` 命中 4 个文件共 9 处、`git grep -c "54 段"` 命中该文件 4 处），不补。

三件都已在各自载体里记着，没有要补的（本组没有 M 行）。未搜 `research/prompts/`、`briefs/`。

## 判定汇总

- D1（132 行）：要改 1（D1-120，milestone/02-second-txn.md:113）；事件句不改 1（D1-71）；不相干 130。
- E1（26 行）：要改 2（E1-1 checks-owed.md:33；E1-20 verification-build.md:155）；事件句不改 3（E1-13、E1-24、E1-26）；不相干 21。
- F7b（37 行）：要改 7（F7b-2、F7b-3、F7b-8、F7b-9、F7b-10、F7b-12、F7b-32）；要人看 1（F7b-22）；
  事件句不改 14（F7b-1、11、16、17、18、19、20、24、29、33、34、35、36、37）；不相干 15。

## 没做什么

- 没改任何文件，`checks-owed.md`、`decisions/16-发布语义.md`、`verification-build.md`、`milestone/02-second-txn.md` 等
  里判定「要改」的那几处，改后的句子已写在上面的表格里，交主 agent 或 `kb-scribe` 落地。
- F7b-22（milestone/01-first-txn.md:184）判「要人看」：分不清已收口里程碑的「会碰到的决策点」清单该保持只读历史存档
  还是随 C314 还清同步更新，未替它下结论。
- 没有跑 `research/scripts/stale-candidates.py --changes`/`--facts` 等写事实表的命令，本轮只分到逐行判一段。
- 没搜索 `research/prompts/`（冻结证据）与 `briefs/`（按日期不回改），按定义第 10 步不搜。
- 不在本组的 A/B/C/D2/D13/D14/F1–F6/F7/F8–F12/G1–G4/H1–H4（候选表里其余行）不判，交给别的组。

## 第 11 步 `--check-report` 输出末行

```
$ python3 research/scripts/stale-candidates.py --check-report research/prompts/sweep-acceptance-2026-09-18-v2-candidates.tsv research/prompts/sweep-acceptance-2026-09-18-v3-judge-6.md --groups D1,E1,F7b
  ✓ 报告判全了：195 行候选都有逐行判定
```
退出码：0
