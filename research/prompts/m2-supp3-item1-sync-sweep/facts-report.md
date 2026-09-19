# m2-supp3-item1 阶段同步：写事实表（回扫员，第 7、8 步）

范围：基准 `58a5ebb`，结束 `a023682`（临时提交对象，`refs/sync-tmp/m2-supp3-item1`）。全程按提交读（`git show <rev>:路径` 或用 `git archive` 出的干净副本），不读工作区文件（工作区混着别的会话未提交的改动）。

## 产出文件

- 变更清单：`/tmp/claude-1000/m2-supp3-item1-sync/changes.md`
- 事实表：`/tmp/claude-1000/m2-supp3-item1-sync/facts.tsv`（16 行事实：F1–F5、F7–F12、F14–F17、F19，跳过 F6、F13、F18——F6 并入 F2/F7，F13 因引用的 H1 标题不带措辞类关键词而撤掉、H1 本身不是纯措辞变更不需要该行，F18 并入 F17）
- 候选表：`/tmp/claude-1000/m2-supp3-item1-sync/candidates.tsv`（21 行候选，19 个不同的行）

## 命令与末行输出

`--changes`：
```
  ✓ 变更清单 /tmp/claude-1000/m2-supp3-item1-sync/changes.md：1 条变更记录
```

`--check-facts`（改过两版，末次通过）：
```
  ✓ 事实表罩全了：1 条变更记录都有出处，16 行事实的检索词都在基准那一版的现状句里命中（新立事项除外、它们在基准零命中）
```

`--facts`：
```
  ✓ 候选表 /tmp/claude-1000/m2-supp3-item1-sync/candidates.tsv：21 行（19 个不同的行）
```

## 说明

- 这一阶段只有 1 条 kb 变更记录（H1，`.claude/kb/milestone/02-second-txn.md` 的「### 2026-09-19」），因为改动集中在这一份文件与 `.claude/rules/implementation-workflow.md`、`records/2026-09-16-subagent拆分提案.md` 三份现状载体里；`git diff --name-status 58a5ebb a023682` 核过，没有第四份现状载体被改。
- 事实表 16 行覆盖派发提示列的 10 件做成的事：F1=第 1 件收口、F2/F3=层 0 并行化与门禁 54 号线程判据、F4/F5=implementation-workflow.md 新两段、F7=门禁 33 号（连带 stage-owners.tsv 登记）、F8/F9=收口表新增第 43/44 行、F10=收口表第②行补坐实、F11=增补 3 第 2 件加验收、F12=生成器调入口的两条前提、F14=bash-command-detector.sh 的 heredoc 修法、F15/F16=看门狗三处修法、F17=续做与临时索引两次真活首次使用、F19=`crates/mutations.tsv` 追加第 129–145 行。
- `MAXIMUM_CANDIDATE_LINES_PER_FACT` 现读脚本是 500（不是派发提示写的 150，脚本已改，本轮按脚本实跑行为为准），F2 用 `越跑越慢` 单一概念词命中基准/结束各 2 行，没有用 `&&`；其余用 `&&` 的（无，本表已都不需要）。
- 本表未处理 `H1` 之外可能存在的、`--changes` 因「纯追加（无删改行）」而没有列出片段的改动（脚本 `changed_segments` 只报 replace/delete 操作产生的片段，纯 append 不出现）；这类内容我改用 `git show` 逐行读原文核实（如收口表第 43、44 行、②行补充、设想实现第 2 件验收、预想细节前提、mutations.tsv 新增行），均已计入事实表，标「新立：」。

## 没做什么

- 没做逐行判定（第 9–11 步），归分组的回扫员另做。
- 没有改任何被搜到的文件，没有改 `stale=`。
- 没有核实 `mutations.tsv` 实际内容（第 129–145 行的原文与替换文逐条对照），只按 kb 文字与 `git diff --stat` 的行数变化（128→145，新增 17 行）取信；深入核对属于逐行判定或实现验收范畴。
