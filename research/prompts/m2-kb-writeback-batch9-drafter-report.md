# kb 第九批写回规格 —— 报告（kb-spec-drafter）

## 规格文件

- `/tmp/claude-1000/kb-batch9-drafter/spec.json`
- `/tmp/claude-1000/kb-batch9-drafter/spec.md`（同内容，markdown 写法）

```
$ sha256sum /tmp/claude-1000/kb-batch9-drafter/spec.json /tmp/claude-1000/kb-batch9-drafter/spec.md
62a588f8db95a6d81ceb398d9ff769fca2254408382cf926f42482e813e02b95  /tmp/claude-1000/kb-batch9-drafter/spec.json
09ac1500f20af31892c3b82cb587d44beddee4470203a3bb834eb772f65cd55d  /tmp/claude-1000/kb-batch9-drafter/spec.md
```

## kb-spec-check.py 原样末行（两份规格各跑一次，都绿）

```
$ python3 research/scripts/kb-spec-check.py /tmp/claude-1000/kb-batch9-drafter/spec.json
  ✓ 规格 3 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 3 条）

$ python3 research/scripts/kb-spec-check.py /tmp/claude-1000/kb-batch9-drafter/spec.md
  ✓ 规格 3 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 3 条）
```

## 逐条：旧串恰好命中 1 次

条 1（`.claude/kb/decisions/23-journal的角色与格式.md`，D23（journal 的角色与格式） 已定项 14 射程，追加零单元发布不冻结那一句）：

```
$ grep -cF -- "- **切换重新读盘择根带出的三件**：……（C458（实例切换取内存里的根，不重新读盘））。" ".claude/kb/decisions/23-journal的角色与格式.md"
1
```

条 2（`.claude/kb/invariants.md` 第 64 行，I-7.13（系统配置池级字段在读者收的范围里） 判据 / 状态列）：

```
$ grep -cF -- "$(cat 该行原文)" ".claude/kb/invariants.md"
1
```

条 3（`.claude/kb/decisions-history/2026-09.md`，新条目其二十二）：旧串是多行锚点（`## 历史版本` 加紧跟的其二十一标题整行），grep -cF 不认多行，改用 Python 字符串精确计数核过，命中 1 次；`kb-spec-check.py` 本身用同一种精确字符串计数复核过，两次都绿（见上）。

## 三条规格分别做什么

1. **D23（journal 的角色与格式） 已定项 14 射程**：在已有的最后一条射程 bullet（切换重新读盘择根三件）之后，追加一条：发布末尾系统配置轮换之后那道屏障报错时，带单元 / 带记录的发布与轮换报错同一处置（冻结、原样重发）；零单元发布（`crates/singlefs-core/src/transaction.rs` 的 `publish_without_units`，末尾走 `persist_the_root_then_rotate_the_system_configuration` 那一路）这一支不冻结、整个挂载返回错误（失败经 `PublishError::BlockDevice`），是替条款没写的一支做的选择；钉用例随代码三方第三轮那一批。函数名与错误成员现查确认：`transaction.rs:1193` `persist_the_root_then_rotate_the_system_configuration`，`transaction.rs:1073` 注释「零单元发布……一失败就整个挂载返回错误」，`transaction.rs:1237` `return Err(PublishError::BlockDevice(cause))`。依据：判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节 Y1 那一格；`m2-impl-c577-barrier-implementer-report.md` 第 106–111 行第 2 条。这一条只改射程，没碰「**依据**」段、没增删实验号，门禁 75 号双向检查不适用，规格里没带实验页改动。

2. **`invariants.md` I-7.13（系统配置池级字段在读者收的范围里）**：判据列末尾插入一句「与 core 收同一张表」，列出八项字段（格式版本、加密类型、槽距、根槽宽、环长、单元区起始槽号偏移 417、journal 环起点偏移 325、根环起点偏移 371），指回 D22（单元原子性怎么合成） 已定项 16 第 1 条（那一条 2026-09-27 已经写了 core 那一半，现查过 `.claude/kb/decisions/22-单元原子性怎么合成.md:332`）；状态列末尾追加一句写明今天 checker 只判前五项、core 只判后三项，两边补齐随代码三方第三轮那一批，指回判决 Y4-a 那一格。

3. **决策变更史 `2026-09.md`**：在「## 历史版本」下插入新条目「### 2026-09-27（其二十二）」，覆盖上面两条的改前 / 改后与依据（判决 Y1、Y4-a 两格；C577（系统配置没见证到的最新根，乙罩不到） 实现员报告第 106–111 行第 2 条）。当月已有条目最大编号是「其二十一」（`grep -oP '（其[一二三四五六七八九十]+）' 2026-09.md` 全文范围内 09-27 那一天最大是 21，已现查），取「其二十二」。

## 全 kb 搜索「零单元」「整池拒绝挂载一致」（写规格之前先搜，命中而不在这 2 条里的逐处列出，不自扩）

`整池拒绝挂载一致`：全 kb 只 1 处命中，`.claude/kb/invariants.md:64`，就是条 2 本身，没有别处。

`零单元`：命中 10 处，全部不在这 2 条规格里，原样列出交主 agent（这一批没动它们）：

- `.claude/kb/decisions/16-发布语义.md:218`（已定项 9 定案：树表 0 条 ⇒ 零单元）
- `.claude/kb/decisions/16-发布语义.md:225`（同项依据：E142（第一个事务的干跑））
- `.claude/kb/experiments/142-第一个事务的干跑.md:288`（影响的决策表，D16（发布语义） 已定项 9 一行）
- `.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:217`（装置引用表里「零单元那几版」）
- `.claude/kb/milestone/02-second-txn.md:143`（现状段落，零单元写行与暖机）
- `.claude/kb/milestone/02-second-txn.md:458`（崩溃注入判别力，「零单元发布在记录与根之间少一道屏障」）
- `.claude/kb/layout/01-first-txn.md:408`（字节表行，零单元写行、零单元暖机）
- `.claude/kb/decisions-history/2026-09.md:916`（历史条目改前段）
- `.claude/kb/decisions-history/2026-09.md:1809`（历史条目②句，「预分配的零单元」）
- `.claude/kb/decisions-history/2026-09.md:10078`（历史条目③句，「预分配的零单元」）

## 要主 agent 判的点

无：三条判决对应的定案句与 kb 现文没有互相矛盾的读法，规格改法与 kb-spec-check.py 一次跑绿，没有改不动的分句。

## 没做什么（固定会有的）

- 没写 kb 正文，只写了草稿目录的规格与本报告；没跑 kb 门禁阶段（21、49、75 号等），那是书记员写完之后的活；没判 D18 已定项 11、D22 已定项 16 现值是否已经写对（本批不动它们，只现查引用点确认依据成立）；没扩大到「零单元」「整池拒绝挂载一致」命中的别处，也没动 I-7.14 立号、C580 还清、D18 已定项 11 / D22 已定项 16 的「实现随第三轮那一批」改现值——这些按主 agent 指示归下一批。
