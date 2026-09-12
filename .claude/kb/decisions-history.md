# 决策变更史

**这是 [decisions.md](decisions.md) 与 `decisions/` 下各决策正文的文末历史，拆出来单独成文。**
正文只写现状，历史一律在这里——规则见 `.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 8 条。
每条写三段：改前是什么、改后是什么、依据是什么。

⚠️ **变更史里的分项编号一律是「今天的编号」，不是当时的编号。**
2026-08-30 起，每条决策的分项只有一套编号、按状态分住「已定项」与「未定项」两节
（见 decisions-history.md 2026-08-30（其二十四）那一条）。那一次把全仓的分项引用同步改写了，
**包括变更史里早于那一天的条目**。所以一条 2026-08-26 的条目里写着「已定项 k」，
说的是**那条分项今天叫这个号、今天是已定**，不是「2026-08-26 当天它就已经定了」——
某一项当时是什么状态，看记它那一条自己的正文（「改前 / 改后 / 依据」三段）。
同一次改写也动了 `records/` 与 `research/prompts/`；`research/results/` 是实验产物，**一个字没动**。

## 条目住在哪

条目按日期所在的月份，原样住在各月一份的文件里。**新条目写进当月那一份**，放在它「## 历史版本」下的最上面：

| 月份 | 文件 |
|---|---|
| 2026-09 | [2026-09-decisions-history.md](2026-09-decisions-history.md) |
| 2026-08 | [2026-08-decisions-history.md](2026-08-decisions-history.md) |

- 换月时新建 `.claude/kb/<年-月>-decisions-history.md`，文件头照上面几份写，并在这张表顶上加一行。
- 别处写的「decisions-history.md 某日（其 N）」按日期到对应月份那一份去找：拆档时每条的日期与序号一个没动。
- 门禁阶段「决策变更史的条目住在它日期所在月的那一份」（`.claude/gate.d/48-history-month-file.sh`）判这两条：条目住错月份、或写回 decisions-history.md，都会红。

## 历史版本

- 2026-09-12：347 条条目按日期原样拆进 2026-08 与 2026-09 两份（此前全住在 decisions-history.md 一份里，7150 行）。
  这天之前写下的「decisions-history.md 第 N 行」这类引用，`research/prompts/` 与 `records/` 里原样保留，行号已经对不上，按那一条的日期与序号去找。
