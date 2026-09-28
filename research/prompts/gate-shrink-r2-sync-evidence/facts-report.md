# 阶段同步·写事实表：门禁优化第二轮

## 基准与范围

- 基准提交：`HEAD`（`e5253e8a2afd26d3352a6f02fe015d67858eab89`）。
- 结束：工作区（这一批未提交）。
- 命令：`python3 research/scripts/stale-candidates.py --changes --base HEAD --out /tmp/claude-1000/gate-sync-facts/changes.md`
  → `✓ 变更清单 /tmp/claude-1000/gate-sync-facts/changes.md：981 条变更记录`

## 阻塞：工作区被别的会话的改动污染，981 条变更记录里没有一条属于这一批

`--changes` 是对 `HEAD..工作区` 整棵树算的，工作区里同时有别的会话未提交的改动。逐条数了来源：

```
grep -E '^\| H[0-9]+ \|' /tmp/claude-1000/gate-sync-facts/changes.md | awk -F'|' '{print $3}' | sed 's/^ *//;s/ *$//' | sort | uniq -c | sort -rn
    560 .claude/kb/decisions-history.md
    403 .claude/kb/experiments-history.md
      8 .claude/kb/checks-owed.md
      3 .claude/kb/experiments/142-新池新建文件的干跑.md
      2 .claude/kb/layout/01-first-txn.md
      2 .claude/kb/experiments/162-崩溃放量判定块存储选型.md
      1 .claude/kb/milestone/03-third-txn.md
      1 .claude/kb/milestone/02-second-txn.md
      1 .claude/kb/invariants.md
```

现查坐实这 981 条一条都不属于「门禁优化第二轮」：

1. `decisions-history.md`、`experiments-history.md`（963 条，占 98%）是另一个并发会话的「变更史条目改按决策/实验分组」搬迁（工作区里这两份文件自己的「## 历史版本」段落写着
   「2026-09-28：条目改按决策分组：`.claude/kb/decisions-history/2026-09.md`……」「2026-09-28（其二）：补抄第一次搬迁漏掉的份数……」）。
   `git show HEAD:.claude/kb/decisions-history.md | wc -l` = 383 行，工作区 `wc -l .claude/kb/decisions-history.md` = 173769 行——这不是这一批改的。
2. 其余 18 条（`checks-owed.md` 8 条、`experiments/142` 3 条、`experiments/162` 2 条、`layout/01-first-txn.md` 2 条、`milestone/02`/`03`/`invariants.md` 各 1 条）逐条现查标题，都是别的实验/决策日常更新（E142 第十九次跑、E162 路径搬迁、layout 段序列手算、milestone 更新），与门禁重构无关。
3. 反向核实这一批自己的 kb 写回（`gate-t3-scribe`、`gate-t3c-scribe`）没有产生任何新的「## 历史版本」条目：两份报告都记录了 `decisions-history.md`、`experiments-history.md` 写前写后 sha256 相同（未变），且报告「没做什么」一节明写「没碰 `checks-owed.md`、`milestone/02-second-txn.md`……决策 05、20、21、23、`invariants.md`」。

结论：`--check-facts` 要求 981 条 H 编号「每条都要出现在某一行事实的出处列里」，但这 981 条没有一条是「门禁优化第二轮」做的事，逐条去读、去判它们说的是什么，是在替另一个会话做它自己的阶段同步——越出了「只改自己的部分」的范围（`agent-common.md`「禁止自行扩大任务范围」）。这不是我这批事实表能罩全的，需要主 agent 决定：等那个搬迁会话自己提交、以它的提交做新的 `--base` 再对这一批跑一次，或者另行指派谁去把那 981 条的事实表补齐。

## 这一批（门禁优化第二轮）自己的事实表

按「做成的事」1–6 逐条现查了旧值/新值，6 条事实、检索词都在基准版至少命中一次、结束版都不超过 500 行（F1 命中 329 行，已抽查）：

事实表：`/tmp/claude-1000/gate-sync-facts/facts.tsv`（表头：编号/旧事实/新事实/检索词/出处，制表符分隔，6 行）

| 编号 | 旧事实 → 新事实（摘要） | 检索词 | 出处 |
|---|---|---|---|
| F1 | 门禁 23 道、散布「门禁 NN 号」称呼 → 收成 15 道按五类命名去编号（`54-layer0-replay` 除外），README/kb/skills 改称全名；`checks-owed`/决策05·20·21·23/`invariants`/`milestone/02`/E158 几份 kb 还没改，规格留 `/tmp` | `门禁 ?[0-9]{2} ?号` | 记录第二轮 4/4a 行、旧编号对照表；gate-t1、gate-t3-scribe、gate-t3c-scribe 报告 |
| F2（新立） | 门禁阶段各自实现参数解析，没有统一的 `--list`/`--check`/`.gate-cells`/按格汇总结构 → 共用库 `.claude/gate.d/lib/stage-cells.sh` 提供这套结构，`gate-structure-check.py` 静态判 | `gate-structure-check\.py\|stage-cells\.sh` | 记录第二轮 4b/4c 行；gate-structure-lib 报告 |
| F3 | `agent-common.md`「门禁」教各 agent 按 `stage-owners.tsv`（三列）自己跑登记给自己的阶段；`check-staged.sh` 存在；`gate-reuse-check` 在 Stop/SubagentStop 注册 → 门禁阶段统一归 `gate-triage`，其余 agent 交回前不跑；`check-staged.sh` 已删；`heavy-test-guard.sh` 新增「提交时才跑的检查」执行前拒绝；两处注册已删；`stage-owners.tsv` 改按格四列 | `check-staged\.sh\|gate-reuse-check\|stage-owners\.tsv` | 记录第二轮 4d 行 |
| F4（新立） | `checker-tier-crates-mutation-replay`（原 59 号）跨机分片不看本地开关 → 只在本地配置 `ENABLE_ACROSS_MACHINES=1` 时才跨机 | `ENABLE_ACROSS_MACHINES` | 记录第二轮 4 行「修的硬伤」 |
| F5 | `evidence-in-repo` 判据二在链接 worktree 里可能不生效；`fixture-claims` 只认部分带 expect 样本；`term-renames` 掩码正则没有左边界 → 三处都修好 | `evidence-in-repo\|fixture-claims\|term-renames` | 记录第二轮 4 行「修的硬伤」 |
| F6 | `governance-refs` 只认编号称呼 → 认全名与格名两种，判「用编号称呼门禁」为旧写法 | `governance-refs` | 记录第二轮 4 行；gate-t1 报告 |

未单独立事实的：`peer-host-lib.sh` 改成按进程号逐个停对端进程——现查 `git grep -c peer-host-lib HEAD -- '*.md'` 为 0，这个脚本的行为从未被任何 kb/规则/README 文字描述过，没有现状载体会「说旧的」，不出候选，故不立行。

## 两条命令的输出末行

```
$ python3 research/scripts/stale-candidates.py --check-facts /tmp/claude-1000/gate-sync-facts/facts.tsv --base HEAD
……
  ✗ H40 没有任何一行事实罩着：.claude/kb/decisions-history.md 2026-09-01
  ✗ 事实表没罩全：981 条变更记录没罩、0 行正则坏了、0 行检索词在基准零命中、0 行在结束零命中、0 行检索词太宽、0 行多余的 &&、0 行冒充新立、0 行冒充只改措辞、0 行宽词没抽查
     → 怎么办：……
退出码 6
```

除「981 条变更记录没罩」（见上「阻塞」一节，全部是别的会话的改动）外，其余七项检查（正则坏、基准零命中、结束零命中、太宽、多余 &&、冒充新立、冒充只改措辞、宽词没抽查）全部为 0——这一批自己的 6 条事实本身写法过关。

```
$ python3 research/scripts/stale-candidates.py --facts /tmp/claude-1000/gate-sync-facts/facts.tsv --base HEAD --out /tmp/claude-1000/gate-sync-facts/candidates.tsv
  ✓ 候选表 /tmp/claude-1000/gate-sync-facts/candidates.tsv：388 行（382 个不同的行）
```

候选表按事实分组：F1 329 行、F3 25 行、F5 30 行、F6 4 行；F2、F4 是新立事实，按定义不出候选（0 行）。

## 产出

- `/tmp/claude-1000/gate-sync-facts/changes.md`（981 条变更记录 + 现状载体差异片段，原始材料，未裁剪）
- `/tmp/claude-1000/gate-sync-facts/facts.tsv`（这一批自己的 6 条事实）
- `/tmp/claude-1000/gate-sync-facts/candidates.tsv`（388 行候选，382 个不同行，供下一段逐行判）

## 没做什么

- 没有为另一个并发会话的 981 条变更记录（`decisions-history.md`/`experiments-history.md` 搬迁与其余 18 条不相关改动）写事实、没有替它们跑逐行判——不属于「门禁优化第二轮」这一批，越权替别的会话做阶段同步违反「只改自己的部分」。
- 没有让 `--check-facts` 退出码归零：只要工作区里那 981 条还没被主 agent 处理（等那个会话提交、换基准，或另派人补），这一批的事实表结构上就没法让它归零，这不是我这份事实表本身的缺陷（各项质量检查全部 0）。
- 没有对 `peer-host-lib.sh` 立事实行（现查全仓 `.md` 里 0 处提及，没有现状载体会说旧的）。
- 没有改任何被搜到的文件、没有改 `stale=`。
- 没有跑门禁、样本、lint、重型测试。
