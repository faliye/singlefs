# 严查第三轮（确认轮）·甲组报告（规则与入口）

范围：`git diff 73ba4a4 d6a3bcb`（HEAD d6a3bcb）在 CLAUDE.md、.claude/main-agent.md、.claude/agent-common.md、.claude/rules/*.md、.claude/skills/*/SKILL.md 上的改动与这些文件今天的全文；重点核 governance-defs-r3 写回的三处（`git diff bfc447e d6a3bcb` 只动了 agent-common.md:46、main-agent.md:54、mutation-sampling.md:82）。r1、r2 报告与同步记录、defs-r2 / defs-r3 判决已处置的不重报；defs-r3 第四节两项不重报。行号全部 `grep -n` 现取。

## 发现

| # | 文件:行 | 类别 | 严重程度 | 原句 | 证据 | 建议改法 |
|---|---|---|---|---|---|---|
| A1 | .claude/rules/mutation-sampling.md:74–75（对 :82） | ② 同一文件两处说法相反（defs-r3 D1/D2 写回 :82 后引入） | 中 | :74–75「跑起来才发现的编不过由门禁 59 号单独报成「无效」，与「没红」分成两栏、各带各的出路：无效要改的是那一行替换文，没红才是去补一条会红的用例。」 | :82 现在写「无效」有三种来源：「替换文编不过，或测试进程在跑到点名的测试之前就被杀（尾巴带 `process didn't exit successfully` 与信号），或点名的名字认不出（名字里带 `$` 这类字符，59 号按字面去找）」。后两种改替换文修不好：名字认不出要改的是表里第六列（`awk -F'\t' '$NF ~ /\$/' crates/mutations.tsv` 现查 11 行，第 202、297、298、684–689、693、694 行，全是 `…$`），进程被杀要看被杀的原因。59 号自己在 `.claude/gate.d/59-crates-mutation-replay.sh:348` 对三种来源一律打「替换文写进源码之后编不过」、:413 出路一律「改这一行替换文，不是去补用例」（门禁脚本归丙组，这里只点它与规则同错）。mutation-triage.md:29 让分诊员「没红与无效的逐条按 `mutation-sampling.md` 分类」，读到的就是 :74–75 这一句 | :74–75 改成「无效要按 :82 列的三种来源分：替换文编不过的改那一行替换文；点名的名字认不出的改表里的测试名；进程被杀的先看是谁杀的」，或指到 :82；59 号的 :348 与 :413 交丙组同步 |
| A2 | .claude/main-agent.md:54（「要推论」一行，defs-r3 D5 写回）对 .claude/agents/three-way-verifier.md:21、.claude/rules/implementation-workflow.md:24 | ② 三处对「设计轮有没有开工快照」说法不一 | 低 | main-agent:54「照 `.claude/rules/three-way-inference.md`「核查员按轮派」派 `three-way-verifier`（有腿交了模型、产物或复跑命令就派，输入里给派腿时主 agent 记下的 `date -u` 与开工快照；不派的在判决里写明为什么）」 | 这一行罩设计轮与代码轮两种，读作「每轮都给快照」；three-way-verifier.md:21「没给快照的轮（设计轮）对主树核」把设计轮当作没有快照；implementation-workflow.md:24 标题「代码轮派腿之前记一份开工快照」只要求代码轮记；three-way-inference.md 全文 `grep -n 'date -u\|快照' .claude/rules/three-way-inference.md` 零命中，设计轮什么时候记、罩哪些文件只能去代码轮那一节类推。照 main-agent 做不会错（多给一份快照核查员也用得上），但主 agent 读 implementation-workflow 会以为设计轮可以不记 | 三处统一成一种说法：要么 implementation-workflow.md:24 那一节改成「派三方的腿之前」（不限代码轮）、verifier:21 的括注「（设计轮）」删掉；要么 main-agent:54 写成「代码轮给开工快照，每轮都给 `date -u`」 |
| A3 | .claude/agent-common.md:57 | ① 与钩子行为不符（改前就有，r1/r2 没报：`grep -n '按模式找进程' research/prompts/governance-review-r*` 零命中） | 低 | 「run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。」 | `.claude/hooks/bash-command-detector.sh:4–5`「按模式找进程（`pgrep -f`、`pkill -f`、`killall`）不在这里判：上游 SOP 的 `.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh` 在执行前拒绝」；合成输入（mutation-triage、run_in_background=true、`pgrep -f cargo`）现跑：pattern-process-guard.sh rc=2「✗ 这条命令里有按模式找进程的写法…执行前拒绝」，bash-command-detector.sh rc=0、检出记录 0 行；agent-watch.py:1575「按模式找进程由上游钩子在执行前拒绝，不写检出记录」。同一行末尾又写「不用 `pgrep -f` / `pkill -f`」，照做不出错，但「只记检出」这半句是错的 | 删掉「与按模式找进程」，或改成「按模式找进程（`pgrep -f`、`pkill -f`、`killall`）由上游钩子 `pattern-process-guard.sh` 在执行前拒绝，前台、后台都拒」 |
| A4 | .claude/agent-common.md:46 | ① 退出码说窄了 | 低 | 「251（scope 起不来）与 252（内存不够排不上）是那条命令一行都没跑，照实报、不绕开包装去裸跑」 | `research/scripts/run-with-memory-cap.sh:20–21`「251 slice 的总上限设不上、systemd-run 起不来（…）、账与锁的目录建不了，或查不到 scope 的结局：那条命令一行都没跑（或结局判不了）」。「查不到 scope 的结局」那一支命令已经跑了，结果判不了；照共用约束报「一行都没跑」是错的事实 | 改成「251（scope 起不来，或查不到 scope 的结局）是那条命令没跑或结局判不了」 |
| A5 | .claude/rules/format-evolution.md:70 | ③ 列举像是全集而漏了几类，照做会漏搜（r2 A4 的处置之后仍缺） | 低 | 「27 号从此在 kb 正文与 `research/`、`crates/` 下的 `.rs` 里替你盯着；别处它都不扫：门禁脚本（`.sh`、`.py`）、`research/results/` 下的产物、规则与 agent 定义、`records/`、`research/prompts/`、kb 的历史节与变更史，这几类照第 1 步手工搜。」 | `.claude/gate.d/27-format-constants.sh:56` 只扫 `.claude/kb/**/*.md`、:83 只扫 `research/**/*.rs` 与 `crates/**/*.rs`。「这几类」没列到的还有：`research/` 下的 `.md`（`research/perf-by-milestone.md` 这类，不在 prompts 与 results）、`briefs/`、`CLAUDE.md`、`.claude/main-agent.md` / `agent-common.md`（算不算「agent 定义」不明）、`.tsv`（`crates/mutations.tsv`、`research/mutations/*.tsv`、`.claude/gate.d/*.tsv`）、`Cargo.toml` 与 `litmus/`。读成「只手搜这几类」就漏 | 改成「27 号只扫这两处；别处（全仓其余文件，含上面举的几类）改完照第 1 步手工搜」，列举降为「例如」 |
| A6 | .claude/agent-common.md:34、:35、:37 | ③ 分桶没罩住 prior-art | 低 | :35「tools 只有 Read / Bash 的：新建文件一律排他…」；:37「tools 只有 Read / Bash 的，改已有文件只用定点替换…」 | `grep -n '^tools' .claude/agents/*.md`：prior-art.md:4「tools: Read, Bash, WebFetch, WebSearch」，既不是「有 Write / Edit」也不是「只有 Read / Bash」，按字面三条都不管它（kb-scribe 的「只有 Edit」r1 A11 已处置，这一桶是另一个） | 「只有 Read / Bash 的」改成「没有 Write / Edit 的」 |

什么现象会推翻各条：
- A1：59 号的判档里「无效」其实只认编译期信号（:344 的正则改窄了），或 mutation-sampling.md:74–75 在别处另有一句把三种来源的出路分开写了。
- A2：three-way-inference.md 或 three-way-verifier.md 另有一处写明设计轮也记快照（`grep -rn '快照' .claude/rules .claude/agents` 有这样一句）。
- A3：pattern-process-guard.sh 在 run_in_background 为 true 时放行、只记检出（合成输入 rc 为 0）。
- A4：run-with-memory-cap.sh 在「查不到 scope 的结局」时退的不是 251。
- A5：27 号另有一段 glob 扫 `research/**/*.md`、`briefs/`、`*.tsv`（`grep -n glob .claude/gate.d/27-format-constants.sh` 只有 :56、:83）。
- A6：prior-art 定义里另写了它的写法，且与共用约束「写」一节不冲突时不需要归桶。

## 核过没问题的

| 载体 | 核了什么 | 怎么核的 |
|---|---|---|
| mutation-sampling.md:82（defs-r3 D1 写回） | 判档次序与 59 号一致：包装五种结局先判，再 `FAILED` 且退出码非 0 记抓到、点名那一行出现记没红、没出现而有 `^error(\[E\d+\])?: ` 或 `could not compile` 记无效、都没有记「没跑到」算没红；「计数：」行八栏、无 💥 ⚠️；另外三栏不为 0 整道红 | 读 59-crates-mutation-replay.sh:311–349、:395–437 |
| mutation-sampling.md:82 里 mutate.sh 那半句 | 七种符号与各自判不判失败；收尾「计数：」只有内存撞顶与超时 | mutate.sh:473–538、:647 |
| agent-common.md:46（defs-r3 D7 写回） | 包装原样传出 0–249、2 是用法错；`replay.sh` 构建失败退 2 | run-with-memory-cap.sh:16–26；replay.sh:619 |
| agent-common.md:46、implementation-workflow.md:48–56、crash-test / gate SKILL | 谁能跑哪几道：crash-verifier 带前缀跑 55、57、59 放行，跑 54 拒；gate-triage 带前缀跑 gate.sh、gate-staged.sh、87 放行；主 agent 不带前缀跑 54 --full 拒；implementation-writer 跑 74 放行；主 agent与 kb-scribe 跑 stage-selftest.sh、number-name-sync.sh 放行 | 合成 PreToolUse JSON 喂 heavy-test-guard.sh，rc 0 / 2 与文字一致 |
| agent-common.md:55 并行写法 | `{ 甲; echo $? > a.rc; } & { 乙; …; } & wait; cat …` 在 run_in_background 与前台都不拒、不记检出；两个 `&` 后逐个 `wait "$pid"` 记一条检出；一个 `&` 后 `wait "$p1"` 不记 | 合成输入喂 bash-command-detector.sh，检出记录现数行数 |
| agent-common.md:57「拒绝七种」 | 与 bash-command-detector.sh 文件头 ①–⑦ 对得上（⑦ 射程是仓里或 /tmp/claude-1000/ 下） | 读 :30–135 |
| main-agent.md:59（暂存行） | 三行命令、`layer0_tree_ready`、退出码取经内存包装那一条 | 54-layer0-replay.sh:227–234 |
| main-agent.md:61（kb-scribe 行） | relabel-item.py 确实改写 `crates/**/*.rs`（与 `.claude/rules/*.md`），33 号数 `crates/mutations.tsv` 锚点 | lib-item-ref-status.py:71–79；33-mutation-tables.sh:11、:76–88；kb-scribe.md:30、:39 已为 rules 放行 |
| main-agent.md:30 | `watch.sh --processes`、`--ack 进程:<pid>`、`ask-every-minutes=60` 都在 | watch.sh:7、:10；watch.conf:23 |
| three-way-inference.md:145 新三级标题 | main-agent.md 引的「交岔路时写岔路单」仍命中；`-s2` 命名没有门禁或脚本按别的形态解析 | grep；`grep -rn -- '-s2' .claude/agents .claude/gate.d research/scripts` 只有 replay.sh:464 无关注释 |
| format-evolution.md:26 | 20 号 :214 认阿拉伯与单字汉字数字加「项 / 条」，75 号 :226 认同一形态且括注可省 | 读源码 |
| CLAUDE.md:108 | `grep -c '^\| I-.*已实现' .claude/kb/invariants.md` = 46；invariants.md 开头条数说明在；02-second-txn.md 步 0 下有「现状」段（:66） | 现跑 |
| CLAUDE.md:22 | 「本地阶段判别力」「编号与简称」是 gate.sh:265、:285 的阶段名，单跑命令形态对 | 读 gate.sh |
| crash-test SKILL | 54、74 带 gate-covers；55 不声明；57 是 LKMM | `grep -n gate-covers .claude/gate.d/*.sh` |
| 轻门禁现跑（GATE_BASE=73ba4a4） | 10、20、50、58、62、63、68、72、90 退 0；rules-lint（按 gate.sh 项目本地接法，29 份）退 0；doc-lint 全仓「检查 488，跳过 0」退 0 | 现跑 |

## 没做什么

- 没跑任何重型测试（54、55、57、59、87、整轮门禁、check.sh、全量 cargo test）；没编译小 crate（defs-r3 已在草稿里实测过信号杀的判档，A1 不依赖新的量）。
- 门禁 12 号现跑退 1，红在 `research/prompts/governance-review-r1-report-B.md:61`（本分支 4717077 新加）与 `research/prompts/m2-lastflag-implementer-report.md:130` 的角标 `′`：不在甲组射程里，没判该怎么处置，只报出来。
- 门禁 47 号现跑退 1：`check-segment-registry.py`、`replace-once.py`、`replace-batch.py`（「以 root 跑」那一格）、`sweep-term.py` 自证没过，多半是这个容器以 root 跑与别的会话的产物，不在甲组射程里，没判。
- `number-name-sync.sh .` 退 1，红在 `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:2783、:2850` 与 `records/2026-09-24-里程碑二收尾调度.md:62、:140`，与 r2 甲组报告所记相同，不在射程里。
- 没核 `.claude/agents/`、`.claude/hooks/`、`.claude/gate.d/` 自己的改动（乙组、丙组），只在甲组文件指到它们时读了对应几行；A1 里 59 号 :348、:413 两处打印要丙组同步。
