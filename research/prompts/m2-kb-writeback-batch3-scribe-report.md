# kb 写回第三批：书记员报告

规格文件：`/tmp/claude-1000/kb-writeback-batch3/spec.json`（sha256 8179e3a5…，与主 agent 给的一致）、
`/tmp/claude-1000/kb-writeback-batch3/spec.md`（sha256 913d93c7…，与给的一致）。共 47 条，覆盖 6 个 kb 文件。

`python3 research/scripts/kb-spec-check.py spec.json` 末行（重跑确认）：
```
✓ 规格 47 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 47 条）
```

## 一、开工时的 sha256sum（6 个规格点名文件 + decisions.md/decisions-history.md）

```
8c51cf2f3fb158f7…  .claude/kb/decisions/23-journal的角色与格式.md
cf40fa6514fca971…  .claude/kb/decisions/18-块里携带什么信息.md
6588ff97f0c8ea27…  .claude/kb/checks-owed.md
dbf65f597bbfe21c…  .claude/kb/decisions-history/2026-09.md
80a1bfa8439e72e2…  .claude/kb/experiments/142-第一个事务的干跑.md
d4385a1dbaa3abba…  .claude/kb/experiments/157-并行线一两条条款的计数模型.md
0d9469f1d3d908ff…  .claude/kb/decisions.md
ddceceebf413372…  .claude/kb/decisions-history.md
```

## 二、写范围闸预检（每个规格文件路径逐个探）

6 个路径逐个 `AGENT_HOOK_DETECTIONS=… bash .claude/hooks/write-guard.sh` 探测，全部 `rc=0`（放行），无 `rc=2`：
`.claude/kb/checks-owed.md`、`.claude/kb/decisions-history/2026-09.md`、`.claude/kb/decisions/18-块里携带什么信息.md`、
`.claude/kb/decisions/23-journal的角色与格式.md`、`.claude/kb/experiments/142-第一个事务的干跑.md`、
`.claude/kb/experiments/157-并行线一两条条款的计数模型.md`。

## 三、replace-batch.py 原样输出

草稿目录批处理文件：`/tmp/claude-1000/kb-writeback-batch3/scribe/batch.json`（从 spec.json 抽取 file/old/new 三键，47 条，顺序不变）。

`--dry-run`：
```
✓ 47 处都命中得对（6 个文件），--dry-run 没写
```

实写：
```
✓ 47 处替换，写了 6 个文件（改名换上新 inode），回读一致
```

按文件计数（与 report.md 起草员报告一致）：D23 决策 1 条、D18 决策 1 条、checks-owed 1 条（新立 C572）、
decisions-history/2026-09.md 1 条（两个变更史条目一并插入）、E142 37 条（回看行首插一句）、E157 6 条（同）。

## 四、分项翻状态

本批规格不含分项翻状态（主 agent 已在派发提示里写明「这一批没有」），未跑 `relabel-item.py`，
也就没有它的「要人看」清单可交。条 1、条 2 改的是 D23 已定项 14 与 D18 已定项 11 的正文内容
（射程表内一格「释放」的例外、依据段追加一句），不是索引表状态/计数，因此按定义「新立分项、改索引行这类不翻状态的规格
写完正文自己跑一次 21 号」处理——见下节。

## 五、21 号（决策分项同步）

写完 kb 正文之后单独跑（未经 --write，因为本来就是绿）：
```
✓ 决策索引表状态列与正文同步（28 条决策）
✓ 决策分项清单与正文同步（350 个分项）
```
本批没有改变已定项数量或状态，21 号不需要 `--write`。

## 六、49 号（决策变更史快查）与 decisions-history.md 回读

先跑不带 `--write`：报「决策变更史的快查与原文有 1 处对不上：`.claude/kb/decisions-history.md` 的生成块与按原文重新生成的不一致」，
没有列出任何「快查·…还没写」的条目（`lib-history-brief.py` 的 `problems` 列表为空，只是生成块过期），
确认两条新条目（其五 D23 已定项14、其六 D18 已定项11）都带齐了「> 快查·改前」「> 快查·改后」两行，判定安全，执行 `--write`：
```
✓ 按原文重新生成了 .claude/kb/decisions-history.md：565 条条目，这次补了 0 条「（待补）」快查
```
0 条待补，证明这一轮没有把「（待补）」灌进不是这一轮的条目。

`git diff -- .claude/kb/decisions-history.md` 里新增的表行（`+|` 开头，去掉计数行）共 5 行，其中：
- **属于本批**（条 4）：`D18（块里携带什么信息） 已定项 11：逐盘「不可见」核扩到挂着之后收盘表的每个入口`（出现 1 次）；
  `D23（journal 的角色与格式） 已定项 14：管理员回退「释放」一格给隔离单元补例外，留在已分配`（出现 2 次，分别在 D19 与 D23 两节——
  该条目正文提到 D19，渲染器按提到的编号交叉列进多节，不是重复写入）。
- **不属于本批，主 agent 已准许带入**：`D13（验证路线） 已定项 7：射程改写记录核对器收两份镜像、第三截交整条历史流 + 挂载流 + 挂载后那次发布`；
  `D28（挂载期承诺量） 已定项 4：分配记录树那一项的式子从 K0 换成 K1（整棵树）`——这两条是别的批次已经写进
  `decisions-history/2026-09.md` 但 `decisions-history.md` 还没重新生成过的条目，`--write` 顺带把它们也渲染进来了。

## 七、门禁阶段（登记给 kb-scribe 的 41 个）

`awk … stage-owners.tsv | wc -l` = 41。逐个跑（原样汇总见 `research/prompts/kb-writeback-batch3-tmp-evidence/gate-results.txt`）：

36 绿（含 3 个退 77「本次无对象可判」：35-user-verdict-owed、36-invariant-count-cross-file、61-settled-same-file），
5 红：27-format-constants、52-segment-registry、69-evidence-in-repo、75-decision-experiment-links、78-owed-cited-tests。

逐一核对红的文件是否在这一批改动内，全部**不是这一轮的**，不修，照写：
- **27**：`research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`、`…e157_parallel_line_one_clauses.rs` 里
  两个 const 写成表达式——这两份是实验源码，不在 kb-scribe 写范围，也不在本批 47 条规格里。
- **52**：`.claude/kb/layout/01-first-txn.md` 段序列登记表与 `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`
  对不上——两份都不在本批规格里。
- **69**：`research/prompts/m2-kb-writeback-batch2-scribe-report.md` 引了 `/tmp/claude-1000/kb-writeback-batch2/…` 两处——
  是**上一批（第二批）**书记员报告的遗留，不是这一批。
- **75**：`.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md`「没回看」1 处、`research/prompts/m2-closeout-code-r1-main-verification.md:52-57`
  六行「行形状」——E156 与该判决文件都不在本批规格 47 条里。
- **78**：`checks-owed.md` 里 C283 引用的测试标识符仓里找不到——C283 那一行不在本批规格里
  （用 `git diff -- .claude/kb/checks-owed.md` 核过，本批只碰了 C571 之后与 C554 之间插入 C572 一行、以及一处「新立…」提要句，
  与 C283 所在行不相交）。

`doc-lint.sh` 末行：
```
✓ 文档铁律检查通过（检查 521，跳过 0；DOC_LINT_VERBOSE=1 看全部）
```

## 八、收尾 sha256sum

```
80782e1c5da5004c…  .claude/kb/decisions/23-journal的角色与格式.md
0c8f3fb88b14ded4…  .claude/kb/decisions/18-块里携带什么信息.md
72e5b66dcfd99677…  .claude/kb/checks-owed.md
7524ce0703b91c1e…  .claude/kb/decisions-history/2026-09.md
610db36d95b787ea…  .claude/kb/experiments/142-第一个事务的干跑.md
d597b23d42cd8992…  .claude/kb/experiments/157-并行线一两条条款的计数模型.md
0d9469f1d3d908ff…  .claude/kb/decisions.md（未变——本批没有直接改它）
10d1b2a3d013471…  .claude/kb/decisions-history.md（由 49 号 --write 重新生成）
```

## 九、没做什么

- 没跑 `relabel-item.py`（本批不含分项翻状态）。
- 没碰 `research/prompts/` 下起草员报告第四节列出的 8 处岔路单改动——按主 agent 说明那部分已由主 agent 自己改，不归书记员写范围。
- 没修 27、52、69、75、78 号报的红：全部核实不在本批 47 条规格所涉的文件/行范围内，判定为别的会话或早先批次遗留，照实报给主 agent，不自行处理。
- 没有编译、没有跑重型测试、没有提交。
- 没有判断任何决策或欠账的对错，只按规格逐条写回。
