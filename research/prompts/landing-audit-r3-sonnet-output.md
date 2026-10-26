# landing-audit-r3 辩方腿（Sonnet）报告：K1 未定项清单的完整性

日期：2026-10-23。立场：辩方，复核前两轮判决与这次审计记下的延后清单，替「没有去向」的一方也找站得住的理由。格：`_landing-audit-r3-body.md` 节「三、格」的 K1 三条。方案按 `crates/` 与 `scripts/` 今天的实现谈；这一批没改 `crates/`。开工快照 `sha256sum -c research/prompts/landing-audit-r3-snapshot/judged-sha256.txt`：74 份全对上（调度记录与九十节都在其中，下面引的行号就是快照时的行号）。

## 复跑

读仓里的文件，不编译、不起重活。取证脚本 `tools/evidence.sh`（小节号 C1 到 C16，下面各表的依据栏引它），模型目录 `research/prompts/landing-audit-r3-sonnet-model/`。工作目录仓根：

```
nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/landing-audit-r3-sonnet-model/tools/rerun.sh --repo "$PWD" --frozen "$PWD" --work <新的空目录>
```

这一轮没有冻结副本，`--frozen` 也给仓根。末行 `name=evidence_diff same=yes` 表示输出与 `out/evidence-output.txt` 逐行相同（我交回前跑过，退 0）；仓变了就列 `evidence.diff`。

| 文件 | sha256 |
|---|---|
| tools/rerun.sh（模板只改 rerun_steps） | e9470ced8d6d27f6beb1486dffcd13a664f5e1d312216c483ecb24754f9b5de2 |
| tools/evidence.sh | 4c58bbfe450f57459dae10828cd366fda2f346296af04d73738e3fed33aa9018 |
| out/evidence-output.txt | b8ca5d15970195ecea16f1be86453846b3679b60d32e98f4739f0daf4b41ebe5 |

## 判定一览

| K1 的条 | 结果 |
|---|---|
| 第一条：每一处「没做」「没查」「留给下一批」「交主 agent」的去向 | 归并成 66 件事：有去向 40 件；去向只有一半 5 件；没有去向 21 件（重 6 件、轻 15 件）。21 件里辩方判：够不着 9、同样打中别的臂 5、站得住 7 |
| 第二条：九十节 ① 到 ⑲ 的形态 | 能直接派 5 行（④ ⑩ ⑭ ⑰ ⑱）；差一两样 8 行（① ③ ⑥ ⑦ ⑧ ⑫ ⑬ ⑲）；要先定形态 2 行（② ⑮）；派不了 3 行（⑤ ⑨ ⑯，⑯ 的判据红不起来）；已过时 1 行（⑪）。说同一件事该并的 1 对（⑪ 与 ⑱），另有 3 组应同批派一个人。⑰ 的出处在调度记录里找不到 |
| 第三条：第二轮判决交主 agent 自己改的 10 处 | 都改了；对不上判决的话的 1 处（K3-1 限定没落到调度记录:40 的 1.1%）；话对得上、形态有毛病的 3 处（⑥ ⑪ ⑯） |

最该补的 6 件（重项）：N01、N06、N08、N09、N10、N15，见表 C。
## 二、K1 第一条：每一处「没做」「没查」「留给下一批」「交主 agent」的去向

判定词用定义里的三个：站得住（这一条有去向、去向够用）／够不着（判决或记录没罩到这一条）／同样打中别的臂（让邻近一条登了记的理由，放到这一条上一样成立）。「去向」栏开头的【有】【半】【无】是我自己的计数标记。条目按事归并，同一件事在几份报告里都被点到的，出处栏并列。

### 表 A：有去向（40 条）

| 编号 | 出处 | 没做的是什么 | 去向 | 辩方判定 | 推翻条件 |
|---|---|---|---|---|---|
| Y01 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md:269`、`research/prompts/landing-audit-r2-sonnet-output.md:105` | 第八处同一写法（decision-links 的待回填清单）没并进库 | 【有】`records/2026-09-16-subagent拆分提案.md:1482`（⑪）；第七批已改：`records/2026-10-22-子agent结果落地审计.md:194`、`.claude/gate.d/doc-experiments.sh:1084`（C3 现取）。⑪ 那一行没跟着标「已做」，见 K1-2 | 站得住 | `doc-experiments.sh:1084` 不再调 `baseline_rows` |
| Y02 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md:272`、`research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md:496` | 库自证没登 `selftest-registry.tsv`，没挪进 `lib/` | 【有】拆分提案:1483（⑫） | 站得住 | ⑫ 行被删而库仍在 gate.d 顶层 |
| Y03 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md:246-261`、`research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md:453-476`、`research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md:482-486` | 样本档、弄坏开关复跑、真仓单跑，以及标「推的」的几条，都没在阶段里跑（样本要和暂存区相同，这一批没暂存） | 【有】调度记录:173、调度记录:194（两处都写「归提交前整轮」）；`research/prompts/_landing-audit-r3-body.md:31` 同一件事；整轮门禁在 `.claude/singlefs-ai-sop/scripts/gate.sh:421` 起 `stage-selftest.sh`。弱点：命令清单在归档目录里，下一次提交会被 `archive-past-rounds` 删，之后只在 git 里 | 站得住（弱） | 整轮门禁不再含样本档这一步 |
| Y04 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md:270` | 三语 kb 仍写已删的滞后表 | 【有】调度记录:174，主 agent 已三语各改一处 | 站得住 | `grep -rln replay-comparison-units-lag` 在 `.claude/kb` 下再有命中 |
| Y05 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md:271`、`research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md:495`、`research/prompts/landing-audit-r2-sonnet-output.md:242` | `.claude/batch-scope` 没登这一批的文件 | 【有】`research/prompts/landing-audit-r2-main-verification.md:62`（归提交前，暂存时登） | 站得住 | 提交前整轮 batch-scope 格红而判决那句已被忘 |
| Y06 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md:273` | 执行员定义里「登记」那一步、写范围表那一行归别的任务 | 【有】调度记录:172（任务 17）、调度记录:177（任务 18） | 站得住 | 两个任务的交回里没有这两处 |
| Y07 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md:238`、`research/prompts/landing-audit-r2-sonnet-output.md:250` | `AGENT_WATCH_BREAK=finished` 前台跑不完 | 【有】调度记录:179（后台跑，退 1，失败行列了） | 站得住 | 179 行的读数被证伪 |
| Y08 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md:219` | 新库自证进 `SELFTEST_RUNNERS` | 【有】调度记录:177；`.claude/gate.d/code-tooling.sh:419` 现取在 | 站得住 | 419 行被删 |
| Y09 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md:242` | 第 42 行的注、`--summary` 存档归主 agent | 【有】调度记录:170 | 站得住 | 存档文件不在 |
| Y10 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-definitions.md:53` | K3-2 要立的检查：三方腿提示的「报告：」一行形态 | 【有】第五批派发闸三方形态判，`research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md:93` | 站得住 | 派发闸自证里这几格被删 |
| Y11 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-definitions.md:58-60` | K3-7、K3-8、K3-9 要立的检查 | 【有】报告自己写了「无机读判法，不立」；`research/prompts/landing-audit-r1-main-verification.md:59-61` 采的是文字 | 站得住 | 有人给出能机读判的写法 |
| Y12 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-definitions.md:77-79`、`research/prompts/subagent-landing-audit-2026-10-22/r1fix-definitions.md:83`、`research/prompts/subagent-landing-audit-2026-10-22/r2fix-definitions.md:40` | 改过的九份定义没走定义三方，第七批两处被攻过零轮 | 【有】`research/prompts/landing-audit-r2-main-verification.md:9-17` 九份都在被判清单；调度记录:193；`research/prompts/_landing-audit-r3-body.md:19-23` | 站得住 | 第三轮腿报告漏判其中一份 |
| Y13 | `research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md:492`、`research/prompts/landing-audit-r2-opus-output.md:100` | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md` 那张「七处」表要补一句四处认法变了 | 【有】补进了库文件头，`research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md:76`；归档报告是冻结证据，不回改 | 站得住 | 库文件头那段被删 |
| Y14 | `research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md:493`、`research/prompts/landing-audit-r2-opus-output.md:258` | `checker-independence-and-sync.sh:670` 与 `doc-decisions.sh` 的 history-loss 取基准没查用途 | 【有】拆分提案:1489（⑱） | 站得住 | ⑱ 行被删 |
| Y15 | `research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md:483` | decision-links 新样本在 `EXPERIMENT_PAGES_AND_PRODUCTS_BREAK` 各条下没逐条跑 | 【有】`.claude/gate.d/stage-break-switch-replay-exempt.tsv:5`（doc-experiments 整道不复跑，`research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md:171` 同说） | 站得住 | 豁免表那行被删 |
| Y16 | `research/prompts/subagent-landing-audit-2026-10-22/r2fix-hooks.md:279` | usage、dispatch 的主会话那部分按修改时刻整份收 | 【有】拆分提案:1490（⑲）末句 | 站得住 | ⑲ 行被删 |
| Y17 | `research/prompts/subagent-landing-audit-2026-10-22/r2fix-hooks.md:281`、`research/prompts/subagent-landing-audit-2026-10-22/r2fix-hooks.md:265` | 派发闸弄坏开关 19 条点名对不上、不在复跑表 | 【有】拆分提案:1490（⑲） | 站得住 | 同上 |
| Y18 | `research/prompts/subagent-landing-audit-2026-10-22/r2fix-hooks.md:262-264` | 要主 agent 照新切法重出结局账存档 | 【有】调度记录:178（sha256 已更新） | 站得住 | 存档 sha256 对不上 |
| Y19 | `research/prompts/subagent-landing-audit-2026-10-22/r2fix-hooks.md:282` | 交回闸 `three-way-verifier` 一支、⑰ 的文件名判法照旧 | 【有】`research/prompts/landing-audit-r2-main-verification.md:51`（不采，⑰ 宽着）。类型表本身的问题另见 N11 | 站得住 | 出现一次 `x-report.md` 被工具层拒 |
| Y20 | `research/prompts/landing-audit-r1-main-verification.md:41`、`research/prompts/landing-audit-r1-main-verification.md:90` | K2-1 那五处交用户 | 【有】用户 2026-10-22 定七处一起并（r1 判决:90），第五批做了 | 站得住 | 七处里有一处没改调库 |
| Y21 | `research/prompts/landing-audit-r1-main-verification.md:45`、`research/prompts/landing-audit-r1-opus-output.md:142` | 写范围一侧的格（登记位里每个目录在写回员写范围里有一行） | 【有】拆分提案:1479（⑧）；判据缺子目录一条见 N06 | 站得住 | ⑧ 行被删 |
| Y22 | `research/prompts/landing-audit-r1-main-verification.md:47`、`research/prompts/landing-audit-r1-opus-output.md:157` | 「实验号住在哪」两份目录表不一致 | 【有】拆分提案:1480（⑨） | 站得住 | ⑨ 行被删 |
| Y23 | `research/prompts/landing-audit-r1-main-verification.md:84` | 九十节要加的五条 | 【有】拆分提案:1477-1481（⑥ 到 ⑩），五条都在 | 站得住 | 五行少于五 |
| Y24 | `research/prompts/landing-audit-r1-sonnet-output.md:46-49` | 命名检查「说跑了、没真跑」没有检查 | 【有】拆分提案:1477（⑥）；形态缺项见 K1-2 | 站得住 | ⑥ 行被删 |
| Y25 | `research/prompts/landing-audit-r1-sonnet-output.md:82-85` | 写回员「找不到的目标」只有文字 | 【有】拆分提案:1478（⑦） | 站得住 | ⑦ 行被删 |
| Y26 | `research/prompts/landing-audit-r1-main-verification.md:94-96`、`research/prompts/landing-audit-r2-main-verification.md:115-117` | 两份判决「五、没核动的」各三条 | 【有】判决自己列成限度；K4-4 的限度进调度记录:170；第二轮三条交第三轮 K3 格（`research/prompts/_landing-audit-r3-body.md:51-54`） | 站得住 | 第三轮 K3 腿没碰这几条 |
| Y27 | `research/prompts/landing-audit-r2-main-verification.md:62` | 辩方遗留三件：样本档没跑、batch-scope、九十节标题 | 【有】前两件归提交前（同一行），标题不改（冻结证据按原标题引） | 站得住 | 九十节标题被别处引成新标题 |
| Y28 | `research/prompts/landing-audit-r2-main-verification.md:107`、`research/prompts/landing-audit-r2-main-verification.md:39` | 第三轮被判清单补 `implementation-writer.md` | 【有】`research/prompts/_landing-audit-r3-body.md:20` | 站得住 | 被判清单里没有这份 |
| Y29 | `research/prompts/landing-audit-r1-sonnet-output.md:119-124` | 比对登记只读手填、没有定义要求登记、写范围的字没补、「已立」措辞 | 【有】拆分提案:1486（⑮）；`.claude/agents/experiment-runner.md:44`、`.claude/agents/experiment-runner.md:52`；`.claude/kb/milestone/03-third-txn.md:185` | 站得住 | 这三处文字被改回 |
| Y30 | `research/prompts/landing-audit-r1-opus-output.md:357-360` | gate-overlap 判定那一半与新格真仓读数，攻方腿不跑 | 【有】调度记录:162、调度记录:130 | 站得住 | 两处读数被证伪 |
| Y31 | `research/prompts/landing-audit-r2-opus-output.md:285` | 归档目录下一次提交会被整个删，引用只在 git | 【有】`scripts/kb/archive-past-rounds.py:49` 的规矩 + 共用约束「找不到历史实验的数据」一条（`.claude/agent-common.md:29`） | 站得住 | 删除后引用既不在 git 也不在仓 |
| Y32 | `research/prompts/subagent-landing-audit-2026-10-22/tw-gate-scripts.md:59-64` | r5 比对对象表第 15 至 19 行在单元表里没有单元 | 【有】`.claude/kb/checks-owed.md:560`（C879） | 站得住 | C879 行被删 |
| Y33 | `research/prompts/subagent-landing-audit-2026-10-22/tw-gate-scripts.md:66-70` | 复跑表 E165 三行期望的是原产物 | 【有】`.claude/kb/checks-owed.md:561`（C880） | 站得住 | C880 行被删 |
| Y34 | 调度记录:93 | 盲判另归出一类：派发提示没套用已有规则 | 【有】拆分提案:1476（⑤） | 站得住 | ⑤ 行被删 |
| Y35 | 调度记录:123 | 看门狗检出：`cp` 目标含命令替换，写范围算不出 | 【有】拆分提案:1488（⑰）同一类 | 站得住 | ⑰ 行被删 |
| Y36 | 调度记录:176 | 看门狗检出：常驻内存阈值为 0、开关轮输出被当长跑判红 | 【有】拆分提案:1484（⑬）、拆分提案:1485（⑭） | 站得住 | 两行被删 |
| Y37 | 调度记录:158 | `gate-overlap.py --touched-by` 认不出两种换文件写法 | 【有】拆分提案:1481（⑩），交上游 | 站得住 | ⑩ 行被删 |
| Y38 | `research/prompts/landing-audit-r2-main-verification.md:51`、`research/prompts/landing-audit-r1-sonnet-output.md:96` | 交回闸超长拒绝说明没写 `result.md` | 【有】已做，`research/prompts/subagent-landing-audit-2026-10-22/r2fix-hooks.md:228-240` | 站得住 | 说明里没有 `result.md` |
| Y39 | 调度记录:127 | 定义那一路「要立的检查」（执行员自查、设计员核可跑性、延后项、回扫的 crates 行） | 【有】拆分提案:1472-1475（① 到 ④） | 站得住 | 四行少于四 |
| Y40 | `research/prompts/subagent-landing-audit-2026-10-22/tw-definitions.md:76`、`research/prompts/subagent-landing-audit-2026-10-22/tw-hooks.md:104` | `apply-writer-patch.py` 认 `report.md` | 【有】调度记录:128 主 agent 改了四处，自证绿 | 站得住 | `report.md` 补丁被它收下 |

### 表 B：去向只有一半（5 条）

去向在，但缺一块：去向行本身没写这一块，或只在一份将被归档的文件里。

| 编号 | 出处 | 有的去向 | 缺的 | 辩方判定 | 推翻条件 |
|---|---|---|---|---|---|
| H01 | `research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md:491` | 【半】调度记录:194 写了「并进 ⑱ 那一批」 | ⑱ 行（拆分提案:1489）只写取基准的另两处，没有这一项；`.claude/gate.d/decision-evidence-lag.tsv:9` 仍写「基准那一版没有这张表、或不在 git 仓里时不比」，没补「git 取不到判红」。照 ⑱ 派工具员的人读不到它 | 够不着 | ⑱ 行或头注补上这一句 |
| H02 | `research/prompts/subagent-landing-audit-2026-10-22/r2fix-hooks.md:278` | 【半】拆分提案:1490（⑲）的现象栏写了「跨行的引文没处理」 | ⑲ 的「要拦什么」栏只写 19 条点名、复跑表一行、主会话改筛，没有跨行引文的检查或出路 | 够不着 | ⑲ 的第二栏补一句，或判决写明不处理 |
| H03 | `research/prompts/landing-audit-r1-sonnet-output.md:120`、`research/prompts/subagent-landing-audit-2026-10-22/tw-gate-scripts.md:186-190` | 【半】`.claude/gate.d/doc-experiments.sh:2406` 的格头注自陈四样「管不到」，并附退役判据 | 其中「留存产物名指到的文件在不在」、单元表里没有登记的单元、复跑表里没有登记的驱动行三样，没有欠账行也没有九十节行；⑮ 只罩第一样（散文没有固定形态） | 够不着 | 三样各有行，或判决写明靠头注即可 |
| H04 | 调度记录:144、调度记录:181 | 【半】两处都写「提交前整轮再判」「不是这一批的文件」（144 行原话：「是存量，不在这一批修，提交前整轮再判」） | 门禁 doc-text 两格存量红、code-tooling 两格存量红（research-script-selftests 的三份自证、change-range-single-source），没有登成可勾的行（不在 C 表、不在九十节）；按 `.claude/agent-common.md:98` 的「红了先看…不在的不修，照写」这样处理是合规的 | 站得住（弱） | 整轮门禁红在这几格而没人认 |
| H05 | 调度记录:157、`research/prompts/landing-audit-r1-opus-output.md:351` | 【半】主 agent 照用户授权自己跑了 `gate-overlap.py`，输出存 `landing-audit-r1-snapshot/gate-overlap-list.txt` | 重型测试闸把只读的 `--list` 也当提交时的检查拒，闸没改；每一轮攻方腿的合并与重合一格都撞这一下 | 站得住（弱）；若第三轮之后不再开轮，影响止于此 | 下一次开三方又被拒而没人管 |

### 表 C：没有去向（21 条）

依据栏的 C1 到 C16 是 `tools/evidence.sh` 的小节号（脚本与输出在模型目录，sha256 见开头）。分量：重＝丢了会让一条已确认的缺口或判决话失效；轻＝文字或设计取舍，丢了影响小。

| 编号 | 出处 | 没做的是什么 | 依据 | 分量 | 辩方判定 | 推翻条件 |
|---|---|---|---|---|---|---|
| N01 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md:219` | 「要它进开关复跑的」，往 `.claude/gate.d/break-switch-replay-files.tsv` 加 `subagent_transcript.py` 一行（三个调用方共用的交回定义，两条弄坏开关只有它自己的自证在守） | C7：复跑表里命中 0；C1：九十节 0、调度记录里只有自证登记（调度记录:177）一处，没有复跑表 | 重 | 同样打中别的臂：⑲ 登 `runner-dispatch-guard.sh` 不在复跑表是同一个理由 | 复跑表加了该行，或判决写明不登 |
| N02 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md:239`、`research/prompts/landing-audit-r2-sonnet-output.md:249` | 看门狗的「起点」没改成调库，同一口径写两份 | C1：九十节「起点」0、调度记录 0 | 轻 | 站得住：`research/prompts/landing-audit-r2-sonnet-output.md:249` 判小偏离，两处同义 | 两处起点口径出现差异 |
| N03 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md:241`、`research/prompts/landing-audit-r2-sonnet-output.md:251` | `dispatch-prompt.py` 文件头三条弄坏开关（`long-run-lines-dropped`、`long-run-types-narrowed`、`live-words-narrowed`）点名对不上失败行 | C1：这三个名字在九十节 0、调度记录 0；⑲ 只管派发闸的 88 条 | 轻 | 同样打中别的臂：⑲ 对派发闸同一病已登 | 三条点名改对，或写明不改 |
| N04 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-definitions.md:54` | K3-3 要立的检查：共用约束括号点名的每个类型，其定义里「交回前不跑门禁阶段」那一句必须带「命名检查除外」 | 九十节没有这一行（读全 ① 到 ⑲ 一遍）；`research/prompts/subagent-landing-audit-2026-10-22/r1fix-definitions.md:58-60` 对别的几条写了「无机读判法，不立」，这一条是 grep 就能判的，却没有这样声明 | 轻 | 够不着：判决只采了文字 | 登一行，或判决写明靠自觉 |
| N05 | `research/prompts/subagent-landing-audit-2026-10-22/r1fix-definitions.md:57` | K3-6 要立的检查：写范围表某行出处写「定义的写范围由…补」的，对应定义的「写范围」一节必须点名那条路径 | 九十节 ⑧ 判的是 kb 目录登记位，不是这件事；`research/prompts/landing-audit-r1-sonnet-output.md:122` 说表有行、定义没字，门禁格只判到类型名一级，这一处靠人工补好 | 轻 | 够不着 | 登一行，或判决写明 |
| N06 | `research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md:494`、`research/prompts/landing-audit-r2-opus-output.md:279`、`research/prompts/landing-audit-r2-sonnet-output.md:134` | 候选扫描认登记目录下各层子目录，写回员写范围的 `*` 不跨目录，⑧ 要立的格得先定「子目录算不算覆盖」 | C1：九十节「子目录」0；拆分提案:1479 判据只写「有一条覆盖它的模式」；`research/prompts/landing-audit-r2-main-verification.md:90` 只采了 `./` 与绝对路径的规整 | 重 | 够不着：今天 invariants 目录下全在一层（r2 辩方查过 10 份），但 ⑧ 的判据没定 | ⑧ 补一句，或目录下出现子目录时有格红 |
| N07 | `research/prompts/landing-audit-r1-main-verification.md:43`、`research/prompts/landing-audit-r1-opus-output.md:108` | `scripts/gate/admission.py` 里复跑表的读法（第五份）不在这一批 | C13：`scripts/gate/admission.py:5438` 的 `REPLAY_TABLE_START` 仍在；C1：九十节 0、调度记录 0 | 轻 | 同样打中别的臂：K2-1 的理由（同一件事各抄一份），用户已定七处一起并 | 登一行，或 `admission.py` 改调库 |
| N08 | `research/prompts/landing-audit-r1-sonnet-output.md:95`、`research/prompts/landing-audit-r2-sonnet-output.md:263` | 共用约束写 SUMMARY、FINDINGS、ANALYSIS 开头也被工具层拒，两条腿都没核；派发闸不判这三个前缀 | C6：`.claude/agent-common.md:108` 仍写「以 REPORT、SUMMARY、FINDINGS、ANALYSIS 开头（不分大小写）」；`.claude/hooks/runner-dispatch-guard.sh` 里 `findings` 只出现在注释、拒绝说明和样本里（110、1311、2561 行），没有按 summary、findings、analysis 前缀判文件名的写法 | 重 | 够不着：`research/prompts/landing-audit-r2-main-verification.md:51` 只处理了「名字里带 report」那半句（`research/prompts/subagent-landing-audit-2026-10-22/r2fix-definitions.md:23-25` 实做是整半句删） | 工具层规则文本到手并与这句核过，或派发闸加这三个前缀 |
| N09 | `research/prompts/landing-audit-r2-main-verification.md:54`、`research/prompts/landing-audit-r2-sonnet-output.md:168` | 第二轮判决对 K3-4 写「不改：定义写的就是」后面接主 agent 建的工作树，这句话在定义里没有；`.claude/agent-common.md:97` 写的是 git worktree 建的副本，读者会以为子 agent 可以自己建 | C9：`.claude/agent-common.md`、`.claude/agents`、`.claude/main-agent.md` 里命中 0；`.claude/agent-common.md:53` 把 `worktree` 列在子 agent 不做的 git 写操作里 | 重 | 够不着：结论（不改）可由「不做」一节立住，但判决引的前提不在文本里；要么在 97 行补一句由主 agent 建，要么改判决的话 | 97 行补上这半句，或判决改引实际存在的句子 |
| N10 | `research/prompts/landing-audit-r1-sonnet-output.md:107-110` | 结局账只有按需命令：没有谁在什么时候跑它、没有落盘的账；「返工几次」只有字样标记 | C8：`agent-cost.py outcome` 在 `main-agent.md`、`agent-common.md`、`.claude/hooks` 里的调用点 0；调度记录:59 原缺口要的是「常设的账」 | 重 | 够不着：`research/prompts/landing-audit-r1-main-verification.md:67` 只采了 `--until` 与存档，「常设」的调用点没人接 | 某个流程写了到点跑 outcome，或判决写明按需即可 |
| N11 | `research/prompts/subagent-landing-audit-2026-10-22/r2fix-hooks.md:280`、`research/prompts/landing-audit-r2-sonnet-output.md:148` | 三方类型认法在第四处：`agent-cost.py` 按 `three-way` 开头认，`three-way-materials` 交回不点名 `-output.md` 会落进新桶；`handback-guard.sh` 的类型集多一个没有定义的 `three-way-verifier` | C5：`scripts/agents/agent-cost.py:1017`、`scripts/agents/agent-cost.py:1083` 两处前缀判；`.claude/hooks/handback-guard.sh:73` 的集合带 `three-way-verifier`；⑯（`records/2026-09-16-subagent拆分提案.md:1487`）只写三份类型名列表 | 轻 | 同样打中别的臂：⑯ 登记三份各抄一份的理由放到第四处、放到那个多出来的名字上一样成立 | ⑯ 的现象栏补第四处与多出的名字，或 `agent-cost.py` 改读 frontmatter |
| N12 | `research/prompts/subagent-landing-audit-2026-10-22/r2fix-hooks.md:279` | `--until` 仍只 `outcome` 认 | 报告原话「--until 仍只 outcome 认」；调度记录:138 的 2268 个数即因只有 `--since` 而不能原样复跑，调度记录:178 已注 | 轻 | 站得住：`--until` 是为存档那一次加的 | `usage` 之类的数被拿去和 `outcome` 并排比较 |
| N13 | `research/prompts/subagent-landing-audit-2026-10-22/r2fix-hooks.md:277` | `dispatch-prompt.py` 文件头 ⑰ 那一句只写四种标签，没写行首与句首才认 | 报告自陈没改；C1 没有任何去向行 | 轻 | 站得住：文字，不影响自证（报告同说） | 有人照文件头写出被派发闸拒的提示 |
| N14 | 调度记录:121-122 | 看门狗检出：重型测试闸在 `s2/setup.sh`、`run-sample.sh` 不存在时没判全，只记不处置 | C14 两行原话「不存在时重型测试闸没判全」；C1：九十节无此事 | 轻 | 同样打中别的臂：同批的另两条检出（⑬ ⑭ ⑰）都登了行，这两条没登 | 登一行，或判决写明不处理 |
| N15 | 调度记录:56 | 流程缺口「修红」（抽 15 个修红里 7 个是「前一个实现员或合入带进来、到 checker 档才露的红」，`research/prompts/subagent-landing-audit-2026-10-22/rework.md:178` 折算 26.49 百万，比已登记的命名检查（4.75）、占号（7.37）都大（`research/prompts/subagent-landing-audit-2026-10-22/rework.md:129-130`）） | 九条缺口的判定表（调度记录:98-108）没有这一行，九十节 0（C1「修红」0）；K7 过时有写不改的理由（调度记录:102），这一条没有 | 重 | 够不着：可能是有意留给既定制度（重型测试只在提交时跑），但没写下 | 判定表或九十节补一行写明不改及理由 |
| N16 | 调度记录:57 | 流程缺口「用量撞顶、客户端关闭截断子 agent」（截断 38 个，白做约 2 到 4 百万） | 同上；C1 用量撞顶九十节 0 | 轻 | 够不着，理由同 N15 | 同上 |
| N17 | `research/prompts/subagent-landing-audit-2026-10-22/rework.md:158-190` | 审计自己记下的五条缺口：K2（入库装置草稿不编不跑）、K5（测量管子不自证）、K10（登记判据当场报了问题仍先跑整格）、K12（变异锚点没跟改）、K13（连带红的已有用例）；K14、K16 现行文字有近似句，没逐句核 | C8：K12、K5、K2、K10、K13 在对应定义里的痕迹全 0 | 轻 | 够不着：调度记录:98-108 没把它们列进判定也没写不改 | 判定表补行，或判决写明次数太少不立 |
| N18 | `research/prompts/subagent-landing-audit-2026-10-22/tw-outcome-ledger.md:136` | 落地率「偏高 1 到 4 个点」是旧口径，新口径多算了 Bash 写入，没用新口径再抽样 | 调度记录:36 仍写旧结论，调度记录:138 只说新数取代了机械层读数 | 轻 | 站得住：旧口径写在抽查结论栏，带口径 | 有人拿 36 行当新口径的结论 |
| N19 | `research/prompts/landing-audit-r2-sonnet-output.md:120` | `replay_rows` 把新格原来的宽读法收紧成 `E<数字>` 与 exact 或 timing，`replay.sh` 将来加别的形状的行新格会漏看 | 辩方自判「行为收紧，不是等价替换」，判决没有条目 | 轻 | 站得住：今天 `replay.sh` 上无差 | `replay.sh` 加了不属这个形状的行而没有格报 |
| N20 | `research/prompts/landing-audit-r1-opus-output.md:318` | `heavy-test-guard` 自证读数今天 1395，调度记录:135 写 1386，差 9 种，攻方推是别的会话加的格，没逐格对 | 调度记录:135 的 1386 是当时读数，无别的去向 | 轻 | 站得住：记录写的是当时读数 | 差的 9 种里有这一批改的 |
| N21 | `research/prompts/landing-audit-r1-sonnet-output.md:141` | 共用约束写「输出不重定向到 `.log`」，比两日志闸管的范围宽 | 判决无条目，作者自判可留 | 轻 | 站得住 | 有人因这句被误拦 |

## 三、K1 第二条：九十节 ① 到 ⑲ 的形态够不够直接派

尺子：一行要写到别人拿去就能派——判据（什么算红）、样本（造什么输入看它红）、出路（谁来改、改在哪、取不到时怎么办）。行在 `records/2026-09-16-subagent拆分提案.md:1472-1490`。

先数：C2 取出 19 行；写了红样本的只有 ⑩ ⑬ ⑭ ⑰ ⑱ 五行（第二栏里同时有自证与应当，或有样本二字）；去向栏写下一批工具改动的 17 行，没有一行写文件清单或谁先谁后（⑤ 写「下一轮清单」，⑩ 写交上游）。

| 行 | 判据 | 样本 | 出路 | 能不能直接派 | 缺的 | 辩方判定 |
|---|---|---|---|---|---|---|
| ① | 报告里「登记落地清单」一节在，每行文件:行与用例名「在仓副本里找得到」（拆分提案:1472） | 无 | 「交回闸或执行员交回的报告检查」二选一没定 | 差一样 | 「仓副本」指哪一份没写：交回闸挂在交回上，而共用约束要子 agent 交回前删自己建的副本，不删的要在报告里逐个写全路径（`.claude/agent-common.md:86`）；入库装置那一支清单格写「待主 agent 证红」（`.claude/agents/experiment-runner.md:40`），判据没放行这种格 | 够不着 |
| ② | 第十四节预计挂钟与第六节态数乘单态耗时对得上，「差一个数量级就红」（拆分提案:1473） | 无 | 加在 preregistration-parallelism 格 | 先定形态 | 态数与单态耗时在设计员定义里是自由文字（`.claude/agents/experiment-designer.md:33`：量过的写命令与读数，估的写式子和依据），没有字段可解析；一个数量级没有出处（同 `research/prompts/landing-audit-r2-main-verification.md:78` 的门槛教训） | 够不着 |
| ③ | `records/` 新增的带「这类词」的行，同行有 C 号、节号或文件:行且指得到（拆分提案:1474） | 无 | 加在 doc-process-records | 差一样 | 词不封闭；C16：调度记录里命中三个词的行 10 行、同行带指针的 3 行（粗算），照字面推过去是大面积误红；没有绿样本 | 站得住（方向对，缺词表与绿样本） |
| ④ | 目标在 `crates/`、判「要改」「要补」的行不在「交实现员」一节就退非 0（拆分提案:1475） | 无 | 加在已有的 `--check-report`（`scripts/kb/stale-candidates.py:22`） | 能 | 样本要重造（`research/prompts/subagent-landing-audit-2026-10-22/tw-definitions.md:62-63` 当时造的在草稿，不入库） | 站得住 |
| ⑤ | 无：自述「还没想清做成哪种检查」（拆分提案:1476） | 无 | 先列常设要求清单 | 不能 | 常设要求清单与各自出处 | 站得住（它自己就写了不能派） |
| ⑥ | 交回闸核会话记录里跑没跑过两条命名检查，「没跑就拒交回」（拆分提案:1477） | 无 | 加在 `handback-guard.sh`（已在读会话记录） | 差一样 | 没写适用条件：只有改了 `crates/`、`scripts/` 下 `.rs` 的才要跑；`.claude/agent-common.md:97` 写明「在 rsync 拷的副本里改的，不在副本上跑」，这一支按定义该写进「要主 agent 跑的命令」，照字面也会被拒 | 够不着 |
| ⑦ | 规格点名的每个目标文件在主工作区存在，或报告「找不到的目标」一节有它（拆分提案:1478） | 无 | 交回闸对写回员的报告检查 | 差一点 | 报告在工作根、不进仓（`research/prompts/subagent-landing-audit-2026-10-22/tw-definitions.md:70`），交回闸怎么拿到规格路径没写 | 站得住 |
| ⑧ | 登记的每个目录在写回员那几行里有「一条覆盖它的模式」（拆分提案:1479） | 无 | code-tooling 加一格 | 差一样 | 「覆盖」没定义：扫描认子目录、写范围的 `*` 不跨目录（N06） | 够不着 |
| ⑨ | 无判据：写成一份目录登记、两边都读，是重构（拆分提案:1480） | 无 | — | 不能 | 不是会红的检查；辩方实测今天占号脚本没漏过任何入库装置的号（`research/prompts/landing-audit-r2-sonnet-output.md:224`），触发条件没写 | 站得住（没立成检查是对的，行里应写「等流程变了再立」） |
| ⑩ | 上游识别 `hook-swap.py` 与 `.new` 加 `mv` 两种写法（拆分提案:1481） | 有：造只用 `mv` 换钩子的会话记录，应当判到那个钩子 | 交上游发版会话 | 能（交上游） | — | 站得住 |
| ⑪ | 已做：决策挂账表与 decision-links 都改调库（C3 两行）；剩一句验证句，之后…对这几道门禁报不出与库整段相同的段（`records/2026-09-16-subagent拆分提案.md:1482`） | 有（两份新样本，`research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md:11`） | — | 过时 | 行没标已做，去向栏仍写下一批工具改动；剩的那句写的路径 `scripts/gate-overlap.py` 不存在（C4），实际在 `.claude/singlefs-ai-sop/scripts/gate-overlap.py` | 够不着 |
| ⑫ | 二选一：库挪进 `lib/`，或登记表认 `lib-` 起头的库（拆分提案:1483） | 无 | 没定选哪个 | 差一样 | 选择；挪库要全仓改引用 | 站得住 |
| ⑬ | 看门狗阈值用不到可分辨下限的峰值（拆分提案:1484） | 有 | 绝对下限「例如 0.5 GiB」或退回 `--process-rss-gibibytes` | 差一点 | 阈值是例子，没出处，选哪个没定 | 站得住 |
| ⑭ | 自证进程底下写的文件不判「长跑判红」（拆分提案:1485） | 有 | 明确 | 能 | — | 站得住 |
| ⑮ | 比对对象表定固定形态，格解析它，与登记表加头注点名不登记的单元逐行对（拆分提案:1486） | 无 | 设计员定义要改、格要改 | 先定形态 | 固定形态本身是设计任务；E165 已有 r1 到 r5 五份、五种写法（调度记录:92 原话「五份五种写法」），旧登记怎么办（豁免、只管新写的、还是回改）没写 | 够不着 |
| ⑯ | 三份定义的 frontmatter 加键，三处改读它；「门禁查重报不出三份相同的类型名列表」（拆分提案:1487） | 无 | 明确 | 不能（判据红不起来） | 三份类型名各占一行（C5），而 `gate-overlap.py` 整段相同的门槛是 8 行（C4：`CLONE_MINIMUM_LINES = 8`），这条判据永远是绿；第四处 `agent-cost.py` 按 `three-way` 开头认和多出来的 `three-way-verifier` 没列（N11） | 同样打中别的臂：⑪ 同写了靠 gate-overlap 验证 |
| ⑰ | `mktemp -d` 模板前缀落在自己认领的草稿目录里就按草稿目录内判，不报（拆分提案:1488） | 有，红绿两向都写了 | 明确 | 能 | 出处写的是「第七批，一个工具员 20 分钟里报了五次以上」（`records/2026-09-16-subagent拆分提案.md:1488`），C15：调度记录里没有这句，第七批一节也没有 mktemp；只有调度记录:123 是同类 | 站得住（行是好的，出处断了） |
| ⑱ | 逐处核用途，是「与基准比」的改调库、取不到判红，各加「基准取不到」样本（拆分提案:1489） | 有 | 明确（先调查后改） | 能 | 加 H01 那一句 | 站得住 |
| ⑲ | 先把 19 条点名对齐，再在复跑表加派发闸一行，主会话改按记录时刻筛（拆分提案:1490） | 无 | 复跑表一行 | 差一样 | 捆了三件事（点名与复跑表／主会话筛／跨行引文），第三件没有检查栏（H02）；19 条名单报告只列了名字和「族」（`research/prompts/subagent-landing-audit-2026-10-22/r2fix-hooks.md:97`），逐条名单得登进复跑表之后由门禁现出 | 够不着 |

### 该并的

按改同一份文件、判同一类缺陷来看，真正说的是同一件事的只有一对，另有三组应当同批派给一个人（不是并成一行）：

| 组 | 理由 | 建议 |
|---|---|---|
| ⑪ 与 ⑱ | 同一种写法（取基准那一版、取不到当成没有）的第 N 处；⑪ 已做，剩的验证句路径还是错的 | ⑪ 标已做，剩的验证句并进 ⑱，一行写完 |
| ⑬ 与 ⑭ | 同一份脚本 `scripts/agents/agent-watch.py`（113 行、131 行各一条规则），同一类误报，同一个人 | 同批派一个人 |
| ① 与 ⑦（兄弟 ⑥） | 都是 `handback-guard.sh` 对某一类型的专项检查，机制相同（读报告里某一节或会话记录里某条命令，对着仓或规格查）；分三个人派会撞同一个文件 | 同批派一个人，三行判据各自独立 |
| ② 与 ⑮ | 都先要把设计员定义里的自由文字改成固定形态，都改 `doc-experiments.sh` 与 `experiment-designer.md` | 同批派一个人，先定形态再写格 |

没有别的两行说同一件事：⑧ 与 ⑨ 都是两份目录表对不上，但对象不同；⑫ 与 ⑲ 都和复跑登记有关但是不同的表。

## 四、K1 第三条：第二轮判决「三、判决」交主 agent 自己改的几处，改了没有、对不对得上

判决原话在 `research/prompts/landing-audit-r2-main-verification.md:103`（调度记录六处限定、九十节 ⑥ 的形态、⑪ 补第九处）和 `research/prompts/landing-audit-r2-main-verification.md:105`（九十节加的两行）。下面逐处对着判决「二、逐格判」里各条的话和调度记录、九十节现在的字。

| 判决的话（出处） | 现在的字（落点） | 对得上吗 | 辩方判定 |
|---|---|---|---|
| K3-1：「真浪费约 1% 到 2%」加限定，下沿 ≤ 30% 时成立、50% 时 3.4%；`agent-cost.py` 文件头写门槛没出处；「丢失的不多」不翻（`research/prompts/landing-audit-r2-main-verification.md:78`） | 调度记录:138 写「70%、30%、10 行三道门槛没有出处」「下沿 30% 抬到 50% 时是 3.4%」；`scripts/agents/agent-cost.py:38` 头注写「没有出处」并指到判决；调度记录:42「丢失的不多」没动 | 对得上一大半。判决点名的那句「真浪费约 1% 到 2%」在调度记录里一字不存在（`grep` 命中 0）；最近的是调度记录:40 的「真正白花约占全部花费的 1.1%」，那一行没带门槛的限定，限定只落在调度记录:138 的 2.0% 上。两个数是同一批桶的两种读法（抽查外推 与 机械读数），后者带了限定、前者没有 | 站得住（头注、138 行都在）；够不着一处：40 行的 1.1% |
| K3-2：三方腿分两桶，调度记录那一行加限定（`research/prompts/landing-audit-r2-main-verification.md:79`） | 调度记录:138 写「按类型名记有效，不核报告在不在仓、同轮有没有判决」；新桶在调度记录:178、`scripts/agents/agent-cost.py:1020` | 对得上 | 站得住 |
| K3-3：「往后复跑得出同一组数」改成「当天复跑逐行相同；往后点到与工作树会变」（`research/prompts/landing-audit-r2-main-verification.md:80`） | 调度记录:178 写「当天复跑与存档逐行相同」，括号里写「往后「点到」没有上界、落地比对读此刻的工作树，数会变」 | 对得上 | 站得住 |
| K3-4：「返工约 14%」改成关键词计数、误报漏报都有，不当损失的上界（`research/prompts/landing-audit-r2-main-verification.md:81`） | 调度记录:42 写「返工字样那个数是关键词计数，误报漏报都有，不当损失的上界读」；调度记录:138 括号里同说 | 对得上。残留：调度记录:42 同一行前面仍写「花费大头的损失在返工」，后接 13.8%，读者先读到结论、后读到限定 | 站得住（残留是措辞） |
| K3-5：K4、K3 两行与载体搬家一行加样本来源的限定（`research/prompts/landing-audit-r2-main-verification.md:82`） | 调度记录:100 写「8 次全在 E171、E188 两条派发最多的链上，证的是归类一致、不证普遍」；调度记录:101 同；调度记录:106 写「9 行出自同一次搬家，频率上是一次事件」 | 对得上 | 站得住 |
| K3-6：「与改前门禁逐行相同」那一句加限定；`--root .` 与快照根同义写在「一、核查」（`research/prompts/landing-audit-r2-main-verification.md:83`） | 调度记录:173 写「这种对照只走到「基准取得到、表在」一种结局与今天表里有的行形」；`research/prompts/landing-audit-r2-main-verification.md:32` 写了快照全对上 | 对得上 | 站得住 |
| 九十节 ⑥ 的形态改成核会话记录（`research/prompts/landing-audit-r2-main-verification.md:57`、`:103`） | 拆分提案:1477 写交回闸核「自己的会话记录里跑没跑过那两条命名检查命令」 | 话对得上；但新形态缺适用条件（K1-2 ⑥ 一行：改 `.rs` 才要跑、rsync 副本一支按定义不跑） | 站得住；形态够不着 |
| ⑪ 补「第九处」`doc-decisions.sh` 的 `base_ledger_text`（`research/prompts/landing-audit-r2-main-verification.md:103`） | 拆分提案:1482 写了第九处与 `base_ledger_text`，并把它并进第七批那一份 | 对得上；后来第七批做完了（C3），行没标已做 | 站得住；行是过时的（K1-2 ⑪ 一行） |
| 九十节加两行：正推 10 的盲区、K3-1 的三方类型表（`research/prompts/landing-audit-r2-main-verification.md:105`） | 拆分提案:1486（⑮）、拆分提案:1487（⑯） | 两行都在，话对得上；⑯ 的判据红不起来、第四处没列（K1-2 ⑯ 一行、N11） | 站得住；⑯ 形态够不着 |

判决「三、判决」第二、第三条要主 agent 自己改的共 10 处（六处限定、⑥、⑪、两行），都动了手，没有漏做的。对不上判决的话的只有一处：K3-1 的限定没落到调度记录:40 的 1.1%（判决点名的那句原文不存在，限定落在了 138 行）。话对得上、形态有毛病的三处：⑥ 缺适用条件、⑪ 已过时没标、⑯ 判据红不起来，详见上一节。

## 五、替「没有去向」的一方说话

21 件没有去向的，各有站得住的理由，也各有够不着的地方：

- 站得住的 7 件（N02、N12、N13、N18、N19、N20、N21）：都是文字、设计取舍或带口径的旧读数，作者当时自判小或可留，漏了也不会让一条已确认的缺口失效。这一类放着不补，我认为合理。
- 同样打中别的臂的 5 件（N01、N03、N07、N11、N14）：让邻近一行登了记的理由放到它们身上原样成立——⑲ 因派发闸开关不在复跑表、点名对不上而登，N01、N03 是同一件事换了个脚本；⑯ 因三份类型表各抄一份而登，N11 是第四份；K2-1 因同一段读法各抄一份而七处一起并，N07 是第八份；⑬ ⑭ ⑰ 因看门狗检出而登，N14 是同批的另两条检出。这一类要么一起登，要么说明为什么单这几条不登。
- 够不着的 9 件（N04、N05、N06、N08、N09、N10、N15、N16、N17）：判决或记录没罩到。其中 N06、N08、N09、N10、N15 分量是重：N06 让 ⑧ 的判据没定；N08 是共用约束里一句没核过的断言，派发闸也没拦；N09 是判决引的前提在文本里没有；N10 是原缺口要的「常设」没人接；N15 是审计自己量出的、比已登记的命名检查与占号都大的缺口，没写不改的理由。

表 B 的 5 件是「去向在、缺一块」，补一句话就闭合：H01 补 ⑱ 一句，H02 补 ⑲ 一句，H03 三样管不到各登行或判决写明靠头注，H04、H05 站得住（弱）。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果。
- 没读第三轮攻方腿的报告与模型，没写 K2、K3（已定项、报出的数）的任何结论；表里碰到的数（调度记录:138 一类）只当出处用，没复算。
- 没编译、没跑门禁阶段与 `gate.sh`、没跑重型测试；取证脚本只有 grep、awk 与一小段 python。
- 第三批四份交回报告（`research/prompts/subagent-landing-audit-2026-10-22/tw-definitions.md`、`research/prompts/subagent-landing-audit-2026-10-22/tw-hooks.md`、`research/prompts/subagent-landing-audit-2026-10-22/tw-gate-scripts.md`、`research/prompts/subagent-landing-audit-2026-10-22/tw-outcome-ledger.md`）和 `research/prompts/subagent-landing-audit-2026-10-22/rework.md` 不在题面清单里：Y32 到 Y40、H03、N17、N18 来自对它们的「没做、没查、下一批、交主 agent」字样检索加读相关段，没有整份核。
- N08 没核 Claude Code 工具层对 summary、findings、analysis 起头的 `.md` 的真实行为（本地没有那份规则文本）；只核了共用约束这句话在、派发闸不判。
- N17 里 K14、K16 的现行文字有近似句，没逐句核它们是否已关住缺口。
- 表 C 的分量、K1-2 的能不能直接派是我的读法，没造红绿输入跑过检查本身；⑥ 的适用条件、⑯ 的红不起来是读代码与读原文推的（⑯ 的依据是 `CLONE_MINIMUM_LINES = 8` 与三处类型名各占一行，没有真的跑 gate-overlap，它在提交时才跑）。
- ⑰ 说的 `mktemp -d <草稿目录>/名.XXXXXX` 这种带路径的模板，`.claude/hooks/bash-command-detector.sh` 头注只写了不带路径的 `-t` 模板与 `d=$(mktemp -d)` 两种已处理；今天 ⑰ 是否仍成立，我没造命令验。
- 没碰仓里被判的任何文件；写了的只有本报告、模型目录 `research/prompts/landing-audit-r3-sonnet-model/` 与草稿目录。没有仓副本与编译目录。
- 要主 agent 跑的命令：无。

## 引文核对（交回前跑）

命令：`python3 scripts/agents/cite-check.py research/prompts/landing-audit-r3-sonnet-output.md --root . --background research/prompts/_landing-audit-r3-background.md`，退 0，末行：

```
  ✓ 核了 4 处引文、155 处裸行号，对上 4 处，没判 155 处（其中裸行号没有引文可比 155 处；逐处列在上面）
```

cite-check 只认引文紧挨着位置的两种写法，所以只数到 4 处。其余当原文引的加引号句子，我写进报告之前都在被引文件的那一行用 grep -F 对过（带省略号的两处按省略号的规矩对）；报告里另有几处加引号的是术语或标签（例如「去向在、缺一块」「交主 agent」「五、没核动的」），不是引文。
