# 阶段同步 sweep-acceptance-v2 · 逐行判（组 D1、E1、F7b）

基准提交 b1c8cef~1，结束提交 00c9d4f（读上下文一律 `git show 00c9d4f:路径`）。
候选表：`research/prompts/sweep-acceptance-2026-09-18-v2-candidates.tsv`；事实表：`research/prompts/sweep-acceptance-2026-09-18-v2-facts.tsv`。
分到的组：D1（132 行）、E1（26 行）、F7b（37 行），每一行都判过，不整组放行。

三条事实（摘自事实表）：
- D1：旧「inode 记录偏移 88 改动计数」→ 新「第一个事务的 inode 记录偏移 88 改动计数改写 3（= 发布它的 checkpoint_txg）」（出处 H37）。
- E1：旧「CLAUDE.md 逐字写着层 0 崩溃点重放（门禁 54 号）的负载还只有第一个事务……」→ 新「新开 layout/02-second-txn.md 登记第二个事务写出的五种盘上形态；发布 B 真设备两盘合计 21 次写调用……」（出处 H39）。
- F7b：旧「C314 记着『检查仍欠：崩溃点重放 harness（多次挂载的录制流）』——复用被抛弃的根引用的单元这条路径还没有实做」→ 新「步 5（抬回退下界 F 之后复用第一个数据单元的落点）落地：`raise_rollback_floor`、`reclaim_released_up_to`；层 0 固定脚本到 E（54 段、2104413 个状态）」（出处 H43–H50）。

判据：把这一行里说到的那件事换成「新事实」，原句还是不是真话；事件句（带日期、描述那一次发生的事）不改；提到检索词但说的不是这件事实的判「不相干」；技术上无法用现有证据确证是否已满足的判「要人看」并写明卡在哪一句。

## 关键现查（供以下判定引用）

- `git show 00c9d4f:.claude/kb/layout/01-first-txn.md` 第 257 行：`| inode 记录（偏移 88） | 改动计数 | 8 | 3（= 发布它的 checkpoint_txg） | D8（核心索引结构） 已定项 6 | 已定 |` —— 字节表已经写着新值 3。
- 同文件第 423 行（## 历史版本 · 2026-09-18）：「四的 inode 记录偏移 88 改动计数曾经写 1，现在写 3……2026-09-18 用户定案改代码写 3，E142 第十一次跑重跑坐实」。
- `git show 00c9d4f:CLAUDE.md` 第 97 行已经是新说法（两条流、29 条不变量），E1 的旧事实已不在 CLAUDE.md 里。
- `git show 00c9d4f:.claude/kb/milestone/02-second-txn.md` 第 64 行（步 5 现状）：固定脚本做到发布 E，含「重开回退到 (1, 3)、D」——是一次多次挂载（重开）+ 回退的录制流。
- `git show 00c9d4f:.claude/kb/checks-owed.md` 第 292 行（C314）与第 262 行（C281）在 00c9d4f 仍写着「检查仍欠：崩溃点重放 harness（多次挂载的录制流）/ 事务层、管理员回退实现、崩溃点重放 harness」，未随层 0 到 E 这件事改写——层 0 的「多次挂载录制流」基础设施已经有了，但 C314/C281 各自要求的「判别力自证」（关掉影子账两格必须由绿转红等）是否已挂上这条流，现有证据确证不了，故判「要人看」而非直接判「要改」。

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| D1 | .claude/kb/checks-owed.md:52 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/checks-owed.md:87 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/checks-owed.md:88 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/checks-owed.md:95 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/checks-owed.md:98 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/checks-owed.md:151 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/checks-owed.md:200 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/checks-owed.md:259 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/checks-owed.md:263 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/checks-owed.md:268 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/checks-owed.md:292 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/checks-owed.md:301 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/checks-owed.md:302 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/checks-owed.md:306 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions.md:46 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions.md:219 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/03-空间分配.md:180 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/03-空间分配.md:187 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/05-快照-空间记账机制.md:268 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/05-快照-空间记账机制.md:445 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/05-快照-空间记账机制.md:481 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/08-核心索引结构.md:372 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/08-核心索引结构.md:398 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/08-核心索引结构.md:403 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/08-核心索引结构.md:479 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/09-加密.md:30 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/13-验证路线.md:235 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/13-验证路线.md:422 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/16-发布语义.md:113 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/16-发布语义.md:209 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/16-发布语义.md:211 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/16-发布语义.md:215 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/16-发布语义.md:248 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/16-发布语义.md:251 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/16-发布语义.md:273 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/16-发布语义.md:313 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/16-发布语义.md:372 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/18-块里携带什么信息.md:648 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/18-块里携带什么信息.md:820 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/18-块里携带什么信息.md:834 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/18-块里携带什么信息.md:876 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/18-块里携带什么信息.md:882 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:196 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/21-权威态与派生态的分界.md:419 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/21-权威态与派生态的分界.md:422 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:141 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:143 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:488 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:489 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:500 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:505 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:521 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/22-单元原子性怎么合成.md:546 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:48 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:83 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:264 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:509 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:600 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:677 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:691 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:799 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:807 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:924 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:1137 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:1209 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:1211 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:1218 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:1222 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:1239 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/decisions/23-journal的角色与格式.md:1240 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments.md:160 | 事件句不改 | 事件句：记 2026-09-18 第十一次跑把改动计数字段从 1 改成 checkpoint_txg=3 这件事，带日期，仍是真话 |
| D1 | .claude/kb/experiments/104-扫描重建的现行版本判定.md:11 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/142-第一个事务的干跑.md:63 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/142-第一个事务的干跑.md:84 | 不相干 | 不相干：这是 root_record 的 checkpoint_txg 产物摘录，不是 inode 记录偏移 88 改动计数 |
| D1 | .claude/kb/experiments/142-第一个事务的干跑.md:166 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/23-journal几何.md:77 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:93 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:8 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/61-反向链hash算法的均匀性.md:37 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/78-重放的起点.md:38 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/79-根记录的容量.md:11 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/88-冒名单元与孤儿单元的判别.md:10 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/91-已定格式下的环账与准入账.md:19 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/91-已定格式下的环账与准入账.md:21 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:27 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/97-记账与分配记录的条目编码.md:131 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/98-inode记录与inode树的几何.md:18 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/98-inode记录与inode树的几何.md:32 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/98-inode记录与inode树的几何.md:115 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/98-inode记录与inode树的几何.md:117 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:14 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:21 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:22 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:25 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:27 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:48 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:56 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/experiments/99-writebuffer条目与seq的去重.md:76 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/invariants.md:25 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/invariants.md:57 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/invariants.md:131 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/layout/01-first-txn.md:61 | 不相干 | 不相干：已写着新值 3，不是旧值 1，这行不受这条事实的新旧变化影响 |
| D1 | .claude/kb/layout/01-first-txn.md:64 | 不相干 | 不相干：已写着新值 3，不是旧值 1，这行不受这条事实的新旧变化影响 |
| D1 | .claude/kb/layout/01-first-txn.md:75 | 不相干 | 不相干：已写着新值 3，不是旧值 1，这行不受这条事实的新旧变化影响 |
| D1 | .claude/kb/layout/01-first-txn.md:175 | 不相干 | 不相干：已写着新值 3，不是旧值 1，这行不受这条事实的新旧变化影响 |
| D1 | .claude/kb/layout/01-first-txn.md:177 | 不相干 | 不相干：已写着新值 3，不是旧值 1，这行不受这条事实的新旧变化影响 |
| D1 | .claude/kb/layout/01-first-txn.md:257 | 不相干 | 不相干：已写着新值 3，不是旧值 1，这行不受这条事实的新旧变化影响 |
| D1 | .claude/kb/layout/01-first-txn.md:285 | 不相干 | 不相干：已写着新值 3，不是旧值 1，这行不受这条事实的新旧变化影响（此处 checkpoint_txg=3 是分配/记账记录的代字段，同一取值不同用途） |
| D1 | .claude/kb/layout/01-first-txn.md:290 | 不相干 | 不相干：已写着新值 3，不是旧值 1，这行不受这条事实的新旧变化影响（记账 key 的代字段，同一取值不同用途） |
| D1 | .claude/kb/layout/01-first-txn.md:331 | 不相干 | 不相干：已写着新值 3，不是旧值 1，这行不受这条事实的新旧变化影响 |
| D1 | .claude/kb/layout/01-first-txn.md:356 | 不相干 | 不相干：已写着新值 3，不是旧值 1，这行不受这条事实的新旧变化影响 |
| D1 | .claude/kb/milestone/01-first-txn.md:58 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/milestone/01-first-txn.md:86 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/milestone/01-first-txn.md:163 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/milestone/01-first-txn.md:170 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/milestone/01-first-txn.md:177 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/milestone/01-first-txn.md:191 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/milestone/01-first-txn.md:198 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/milestone/02-second-txn.md:100 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/milestone/02-second-txn.md:113 | 要改 | **会碰到的决策点**：D4（校验和位置） 已定项 7 / 已定项 5；D8（核心索引结构） 已定项 6（改动计数 = 最后一次改动所在发布的 checkpoint_txg，这次写 4；第一个事务写的是 3——2026-09-16 核出「改动计数 1」与字段表定义不符，2026-09-18 用户定案改代码写 3，E142（第一个事务的干跑） 第十一次跑重跑坐实）；段序列登记表八还没有 B 那一行……（后文不变） |
| D1 | .claude/kb/milestone/02-second-txn.md:149 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/milestone/02-second-txn.md:156 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/milestone/02-second-txn.md:166 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/milestone/02-second-txn.md:169 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/verification-build.md:66 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | .claude/kb/verification-build.md:67 | 不相干 | 不相干：泛指 checkpoint_txg 概念或其他字段，不是 inode 记录偏移 88 改动计数的具体取值 |
| D1 | records/2026-08-29-组合对攻轮.md:52 | 事件句不改 | 事件句：描述 2026-08-29 那次组合对攻轮核实到的三条闸不触发，带日期，仍是真话 |
| D1 | records/2026-09-06-树ID水位臂比较重做.md:15 | 事件句不改 | 事件句：2026-09-06 树 ID 水位臂比较重做记录里的一句引文，描述那次比较，仍是真话 |
| D1 | records/2026-09-13-总审核.md:13 | 事件句不改 | 事件句：2026-09-13 总审核发现「暖机落字节的连带回扫只做了一半」，带日期记那次审核结果，仍是真话 |
| D1 | records/2026-09-13-总审核.md:454 | 不相干 | 不相干：讲的是 I-7.8 树 ID 水位读法的问题，不是 inode 记录偏移 88 改动计数 |
| D1 | records/2026-09-13-未定项总清单与待用户定案.md:29 | 事件句不改 | 事件句：2026-09-13 未定项清单记那一天 D16 已定项 8 待选的三条臂与代价，带日期，仍是真话 |
| D1 | records/2026-09-13-未定项总清单与待用户定案.md:64 | 事件句不改 | 事件句：2026-09-13 未定项清单记那一天 D16 已定项 8 的选择与前置状态，带日期，仍是真话 |
| E1 | .claude/kb/checks-owed.md:33 | 要改 | 事务层、分配器、根环、崩溃点重放 harness（四样 2026-09-14 已有：`crates/singlefs-core`、门禁 54 号；层 0 的负载是两条流：第一个事务，以及固定脚本到 E，含释放与重用；C22 的故障注入自证检查仍欠） |
| E1 | .claude/kb/checks-owed.md:354 | 不相干 | 不相干：讲的是 C380 环上有洞时释放代判不出唯一值这件事，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/checks-owed.md:403 | 不相干 | 不相干：讲的是 C374 释放代与树表诞生 txg 的检查已还这件事，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/decisions/23-journal的角色与格式.md:691 | 不相干 | 不相干：讲的是管理员回退的重放下界定义，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/invariants.md:12 | 不相干 | 不相干：讲的是池级 checker 判 29 条这件事，已是当前正确计数，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/invariants.md:53 | 不相干 | 不相干：I-7.4 本身的定义，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/invariants.md:110 | 不相干 | 不相干：I-2.1 本身的定义与实现状态，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/invariants.md:123 | 不相干 | 不相干：I-3.1 本身的定义与读法，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/invariants.md:131 | 不相干 | 不相干：I-3.9 本身的定义，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/invariants.md:157 | 不相干 | 不相干：I-4.8 本身的定义与实现状态，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/invariants.md:173 | 不相干 | 不相干：I-5.4 本身的定义与实现状态，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/invariants.md:275 | 不相干 | 不相干：I-9.14 本身的定义与实现状态，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/milestone/02-second-txn.md:14 | 要人看 | 要人看：这行是里程碑 2026-09-16 建档时引用的 CLAUDE.md 原文（题面依据表），CLAUDE.md 现已改写成两条流的负载描述；该文档声明「按当日判断写、不要求持续正确」，这一行该跟着改还是保留作立项证据，需要人确认 |
| E1 | .claude/kb/milestone/02-second-txn.md:74 | 不相干 | 不相干：讲的是只供测试的开关清单，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/milestone/02-second-txn.md:141 | 不相干 | 不相干：讲的是 mount_writable 当前实现细节，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/milestone/02-second-txn.md:166 | 不相干 | 不相干：讲的是 mount_rollback 当前实现细节，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/milestone/02-second-txn.md:195 | 不相干 | 不相干：讲的是抬回退下界 F 当前实现细节，不是「层 0 负载覆盖范围」这句断言 |
| E1 | .claude/kb/milestone/02-second-txn.md:358 | 不相干 | 不相干：讲的是崩溃点重放跑得慢这件性能欠账，已确认两条流都存在，不受这条事实影响 |
| E1 | .claude/kb/verification-build.md:139 | 不相干 | 不相干：讲的是崩溃一致性三件套已接上，不是「层 0 负载覆盖范围」这句断言本身 |
| E1 | .claude/kb/verification-build.md:155 | 事件句不改 | 事件句：明确标注「2026-09-14 现状」，描述那一天哪几条必红用例做成了，带日期，仍是真话 |
| E1 | .claude/kb/verification-build.md:157 | 事件句不改 | 事件句：明确标注「2026-09-14 的负载还只有它」，描述那一天的状态，带日期，仍是真话 |
| E1 | .claude/kb/verification-build.md:163 | 事件句不改 | 事件句：描述 2026-09-13 随 C314 定案登记的层 0 固定脚本计划（到 D 为止），带日期记那次登记，仍是真话 |
| E1 | .claude/kb/verification-build.md:248 | 事件句不改 | 事件句：记三方论证反推腿/攻击腿那一轮的判决内容，带轮次归属，描述那一次发生的事 |
| E1 | .claude/kb/verification-build.md:249 | 事件句不改 | 事件句：记本地腿找反例那一轮的判决内容，描述那一次发生的事 |
| E1 | CLAUDE.md:97 | 不相干 | 不相干：这正是已经改好的新说法（两条流、29 条不变量），已与新事实一致，不是旧值残留 |
| E1 | records/2026-09-03-验证三件套落地调研.md:60 | 事件句不改 | 事件句：2026-09-03 验证三件套落地调研记录里指出释放路径在第一个事务里不触发，带日期，仍是真话 |
| F7b | .claude/kb/checks-owed.md:200 | 不相干 | 不相干：C199 词条里提到 C314 只是背景依赖引用，不是「检查仍欠：崩溃点重放 harness（多次挂载录制流）」这句本身 |
| F7b | .claude/kb/checks-owed.md:262 | 要人看 | 要人看：C281 也写「检查仍欠：事务层、管理员回退实现、崩溃点重放 harness」，而 mount_rollback 已落地、层 0 已跑到 E（含回退到 D）；这条具体检查（I-7.2 必须绿、判别力自证）是否已随之满足，需要人核实 |
| F7b | .claude/kb/checks-owed.md:292 | 要人看 | 要人看：C314 记的「检查仍欠：崩溃点重放 harness（多次挂载的录制流）」——层 0 固定脚本已到 E（含重开回退），多次挂载录制流的基础设施已有；但 C314 判别力自证要求的两格（关掉影子账必须由绿转红）是否已挂上这条流未经证实，需要人核实是否该把这一行改判 |
| F7b | .claude/kb/checks-owed.md:296 | 不相干 | 不相干：C318 讲的是影子账隔离的单元没进准入不等式，是另一笔账，不是「检查仍欠：崩溃点重放 harness」这句本身 |
| F7b | .claude/kb/checks-owed.md:328 | 不相干 | 不相干：C356 讲的是第九项净释放为负没有钳位，是另一笔账，不是「检查仍欠：崩溃点重放 harness」这句本身 |
| F7b | .claude/kb/decisions/16-发布语义.md:189 | 不相干 | 不相干：这是已定项 8 的定案摘要句（暖机取甲′、前置 C314），是定案文本本身，不是检查状态的断言 |
| F7b | .claude/kb/decisions/16-发布语义.md:205 | 事件句不改 | 事件句：已定项 8 的小标题，记 2026-09-13 定案与两轮三方论证这件事，带日期，仍是真话 |
| F7b | .claude/kb/decisions/16-发布语义.md:207 | 要人看 | 要人看：这行说「检查那一半仍欠（崩溃点重放 harness 的多次挂载录制流），C314 因此仍在未还表」；层 0 已跑到 E 含多次挂载与回退，这句是否仍成立需要人核实（暖机已生效那半没有争议，检查那半有争议） |
| F7b | .claude/kb/decisions/16-发布语义.md:209 | 要人看 | 要人看：这行说「崩溃点重放 harness 那条检查仍欠」，与层 0 已跑到 E（含回退）的事实是否矛盾需要人核实 |
| F7b | .claude/kb/decisions/16-发布语义.md:211 | 要改 | **代价**：改已定项 7 的 fsync 返回条件（新实例确认前多一个条件）；第一个事务的 checkpoint_txg 从 1 变 3，多两条空记录与两次根写——已落到 [layout/01-first-txn.md](../layout/01-first-txn.md) 与 E142（第一个事务的干跑）（2026-09-13 暖机生效、2026-09-18 E142 第十一次跑坐实，字节表不再按 1 写）；D28（挂载期承诺量） 已定项 3 的切换预留……（后文不变） |
| F7b | .claude/kb/decisions/16-发布语义.md:224 | 事件句不改 | 事件句：记「先还 C314 再取甲′」这条建议是怎么来的（C314 不分辨臂但要先修），描述那次论证推理，仍是真话 |
| F7b | .claude/kb/decisions/16-发布语义.md:291 | 要改 | ⚠️ **fsync 返回条件多一句（2026-09-13，随已定项 8 定）**：新实例在本实例写成的根覆盖两块盘之前，fsync 不返回、不向管理员确认回退——做法是连推空发布至多 R = 3 次；暖机已生效（2026-09-13 起，字节表已含两次空发布、checkpoint_txg=3），不再等 C314 还清 |
| F7b | .claude/kb/decisions/23-journal的角色与格式.md:1209 | 不相干 | 不相干：这是管理员回退的显式例外定义正文，不是「检查仍欠」这句断言 |
| F7b | .claude/kb/decisions/23-journal的角色与格式.md:1245 | 不相干 | 不相干：这是回退例外「之前不生效」这句为何要靠影子账成立的原理说明，只把 C314 当出处指过去，不是检查状态断言 |
| F7b | .claude/kb/decisions/28-挂载期承诺量.md:30 | 不相干 | 不相干：这是准入不等式第九项「被抛弃根独占量」的定义，把 C314/C318 当背景引用，不是检查状态断言 |
| F7b | .claude/kb/experiments.md:168 | 事件句不改 | 事件句：记 E150 已跑（2026-09-13 两次）这件事，带日期，仍是真话 |
| F7b | .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:1 | 事件句不改 | 事件句：实验标题记「已跑（2026-09-13 两次）」，带日期，仍是真话 |
| F7b | .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:4 | 事件句不改 | 事件句：记这个实验当时要回答的问题（C314 两格能不能在计数模型里可达），描述那次实验的题面，仍是真话 |
| F7b | .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:8 | 事件句不改 | 事件句：明确标注「结论（2026-09-13 跑完）」，带日期记那次模型结论，仍是真话 |
| F7b | .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:74 | 事件句不改 | 事件句：实验问答表里记那次模型的问答，描述那次实验，仍是真话 |
| F7b | .claude/kb/invariants.md:53 | 不相干 | 不相干：I-7.4 本身的定义，不是「检查仍欠：崩溃点重放 harness」这句断言 |
| F7b | .claude/kb/milestone/01-first-txn.md:184 | 要改 | **会碰到的决策点**：D16（发布语义） 已定项 8（暖机取甲′、前置 C314（回退可以复用被抛弃的根引用的单元）；暖机已落地，第一个事务的字节表已含两次空发布、checkpoint_txg=3，不再等 C314 还清）；C94（登记的格式常量与后来的定案对不上）（记录头登记值取哪个数）； |
| F7b | .claude/kb/milestone/01-first-txn.md:231 | 事件句不改 | 事件句：记里程碑「第一个事务」步 7 出口时，回退那两格（C314）要等回退进里程碑二这件事，描述当时的依赖判断，仍是真话（回退后来确实进了里程碑二） |
| F7b | .claude/kb/milestone/02-second-txn.md:9 | 事件句不改 | 事件句：里程碑 2026-09-16 建档时「题面」表里的一行，记「2026-09-13 随 C314 定案再加回退到 A、D」这一步计划，带日期，仍是真话（后来又延伸到 E，不否定这行曾经的计划） |
| F7b | .claude/kb/milestone/02-second-txn.md:74 | 不相干 | 不相干：讲的是只供测试的开关清单（含关掉影子账），是现状描述，不是「检查仍欠」这句断言 |
| F7b | .claude/kb/milestone/02-second-txn.md:166 | 不相干 | 不相干：讲的是 mount_rollback 当前实现细节，不是「检查仍欠」这句断言 |
| F7b | .claude/kb/milestone/02-second-txn.md:186 | 不相干 | 不相干：这是必红用例的规格要求（关掉影子账两格必须红），是规格不是完成状态断言 |
| F7b | .claude/kb/milestone/02-second-txn.md:191 | 不相干 | 不相干：这是会碰到的决策点清单，把 C314 当列表项之一列出，不是「检查仍欠」这句断言 |
| F7b | .claude/kb/milestone/02-second-txn.md:332 | 事件句不改 | 事件句：明确标注「2026-09-17 逐句核过」，记那次核账发现七条债一条都没销，带日期，仍是真话 |
| F7b | .claude/kb/verification-build.md:74 | 不相干 | 不相干：讲的是准入可用量与分配器发得出槽数的对拍方法，不是「检查仍欠」这句断言 |
| F7b | .claude/kb/verification-build.md:153 | 不相干 | 不相干：这是必红用例表里的规格要求一行，是规格不是完成状态断言 |
| F7b | .claude/kb/verification-build.md:163 | 事件句不改 | 事件句：描述 2026-09-13 随 C314 定案登记的层 0 固定脚本计划（到 D 为止），带日期记那次登记，仍是真话 |
| F7b | records/2026-09-13-总审核.md:13 | 事件句不改 | 事件句：2026-09-13 总审核发现「暖机落字节的连带回扫只做了一半」，带日期记那次审核结果，仍是真话 |
| F7b | records/2026-09-13-总审核.md:165 | 事件句不改 | 事件句：2026-09-13 总审核表里记 D16 已定项 8 定案句与 C314 仍在欠账表的矛盾，带日期，仍是真话 |
| F7b | records/2026-09-13-总审核.md:205 | 事件句不改 | 事件句：2026-09-13 总审核记那 14 条挡路欠账（含 C314）今天都在未还表，带日期，仍是真话 |
| F7b | records/2026-09-13-未定项总清单与待用户定案.md:29 | 事件句不改 | 事件句：2026-09-13 未定项清单记那一天 D16 已定项 8 待选的三条臂与代价，带日期，仍是真话 |
| F7b | records/2026-09-13-未定项总清单与待用户定案.md:64 | 事件句不改 | 事件句：2026-09-13 未定项清单记那一天 D16 已定项 8 的选择与前置状态，带日期，仍是真话 |

## 交回前核查

```
$ python3 research/scripts/stale-candidates.py --check-report research/prompts/sweep-acceptance-2026-09-18-v2-candidates.tsv research/prompts/sweep-acceptance-2026-09-18-v2-judge-6.md --groups D1,E1
  ✓ 报告判全了：158 行候选都有逐行判定
EXIT: 0

$ python3 research/scripts/stale-candidates.py --check-report research/prompts/sweep-acceptance-2026-09-18-v2-candidates.tsv research/prompts/sweep-acceptance-2026-09-18-v2-judge-6.md
（--groups 不认带字母后缀的 F7b，故不带 --groups 跑一次，只看 D1/E1/F7b 的缺漏——例子里全是别组 A3/A4，不是我的组）
  ✗ 1597 行候选没有逐行判定，例：A3 .claude/kb/checks-owed.md:83；A3 .claude/kb/decisions/03-空间分配.md:68；A3 .claude/kb/decisions/18-块里携带什么信息.md:547；A3 .claude/kb/decisions/21-权威态与派生态的分界.md:436；A3 .claude/kb/decisions/21-权威态与派生态的分界.md:448；A4 .claude/kb/checks-owed.md:297；A4 .claude/kb/experiments/142-第一个事务的干跑.md:181；A4 .claude/kb/layout/01-first-txn.md:417；A4 .claude/kb/milestone/01-first-txn.md:19；A4 .claude/kb/milestone/01-first-txn.md:99
  ✗ 报告没判全：共 1792 行候选
     → 怎么办：在「## 逐行判定」表里给每一行候选一行 | 组 | 载体 | 判定 | 改后的句子或理由 |，判定以 要改 / 要补 / 事件句不改 / 不相干 / 要人看 开头，最后一格写改后的句子或理由
EXIT: 3

算术核对：总候选 1792 行 − 缺漏 1597 行 = 195 行 = D1(132) + E1(26) + F7b(37)，与本报告已判的行数一致；缺漏样例（A3、A4……）全是别的组，D1/E1/F7b 三组没有一行落在缺漏里。

## 反向检查（第 10 步，仅限分给我的三条事实）

- D1（inode 记录偏移 88 改动计数改写 3）：已记在 `.claude/kb/layout/01-first-txn.md` 第 257 行（字节表）与第 423 行（## 历史版本 2026-09-18）、`.claude/kb/experiments.md` 第 160 行（E142 摘要），三处都现查过，不缺登记。
- E1（新开 layout/02-second-txn.md，发布 B 真设备 21 次写调用等）：`git cat-file -e 00c9d4f:.claude/kb/layout/02-second-txn.md` 存在，且第 19 行原样含「21 次写调用」「344 576 字节」「4 次屏障」「1 次 FUA」，不缺登记。
- F7b（步 5 落地：raise_rollback_floor、reclaim_released_up_to；层 0 到 E）：已记在 `.claude/kb/milestone/02-second-txn.md` 第 64 / 195 行（步 5 现状），不缺登记。
- 三条事实都已在该记它们的载体里找到，本轮「## 逐行判定」不需要补 M1/M2 一类的「要补」行。未搜 `research/prompts/` 与 `briefs/`。

## 没做什么

- 只判了分给我的 D1、E1、F7b 三组共 195 行，其余组（A、B、C、G、H 等 1597 行）不归我判，交别的回扫员。
- 4 处标「要人看」（checks-owed.md:262 C281、checks-owed.md:292 C314、decisions/16-发布语义.md:207、decisions/16-发布语义.md:209）与 1 处标「要人看」（milestone/02-second-txn.md:14）：C314/C281 的「判别力自证」（关掉影子账两格必须由绿转红）是否已经挂到层 0 到 E 这条新流上，光靠现有文本核不出来，需要跑一次针对性的崩溃点重放或让写代码的人确认；milestone 文档那一行的「该不该跟着 CLAUDE.md 现状改」也没有再深挖里程碑文档的既往惯例，留给人判。
- 未编译 Rust、未跑门禁、未跑 gate.sh，按定义不必做。
- 未验证判定为「不相干/事件句不改」的每一处历史文件是否还有更早的第三方事实矛盾（如 decisions/16-发布语义.md:211/291 与 milestone/01-first-txn.md:184 之间的暖机是否已生效存在内部前后不一致，本报告已按现查证据判「要改」，但没有回头去改其余提及同一暖机模型、未落在 D1/E1/F7b 候选行里的地方——那些不在我的候选范围内）。
