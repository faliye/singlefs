<!-- knowledge-sync -->
# governance-rot-r3 阶段同步

触发文件：CLAUDE.md、.claude/agent-common.md、.claude/agents/crash-verifier.md、.claude/agents/experiment-designer.md、.claude/agents/experiment-runner.md、.claude/agents/gate-triage.md、.claude/agents/implementation-writer.md、.claude/agents/kb-scribe.md、.claude/agents/mutation-triage.md、.claude/agents/sweep.md、.claude/agents/three-way-attack.md、.claude/agents/three-way-local-attack.md、.claude/agents/three-way-local-defense.md、.claude/agents/three-way-materials.md、.claude/agents/three-way-verifier.md、.claude/gate.d/54-layer0-replay.sh、.claude/rules/format-evolution.md、.claude/rules/fs-design.md、.claude/rules/implementation-workflow.md、.claude/rules/mutation-sampling.md、.claude/rules/path-moves.md、.claude/rules/three-way-inference.md、research/scripts/ask-local.sh、research/scripts/ask-local-selftest.sh、research/scripts/stale-candidates.py；另改了 .claude/main-agent.md 与三个 .claude/skills/*/SKILL.md（不在触发登记里，一并记）

这一阶段做成的事：五个只读核对员逐份核 29 份治理文档（报告 A–E，共 97 条），主 agent 逐条现查后处置：纯描述修正直接改；十二项要定的由用户在三个弹窗里定（层 0 全量由主 agent 建 worktree 并跑、定义问题出口内直改出口外弹窗、历史类文件换名照实际做法、57 号文档照实写、ask-local.sh 没验过退 6、本地腿派哪条由主 agent 定并写理由、云端腿再抽由主 agent 再派、本地腿用 run_in_background、实现员在主工作区改、15 与 74 号按轻阶段、草稿目录改成按 uid 取（另起第 4 轮）、pre-commit 本机没装）。

## 搜索

- `python3 .claude/gate.d/lib-governance-refs.py` → 改后「扫 29 份治理文档：门禁号 112 处、路径 305 处、小节 93 处」，0 处红
- 编号简称比对（登记位取 decisions/、experiments/ 标题行与 checks-owed.md、invariants.md 表行，扫 5 份规则、CLAUDE.md、main-agent、agent-common、16 份定义、3 个 skill）→ 改前裸编号 14 处、简称写错 2 处；改后只剩 fs-design.md 一处 D14 的嵌套括号，是比对脚本认不了嵌套括号，不是真问题
- `grep -rn "三类，判据不同"` 除 mutation-sampling.md 自己之外 → 0 处，改标题不留悬空指向
- `grep -n "e<号>_<简称>\|e<号>-<简称>"` 治理文档 → 改后 0 处
- 上游共享 skill `.claude/singlefs-ai-sop/skills/gate/SKILL.md` 的阶段表比 gate.sh 少 8 个阶段（E-18）：属上游，只能在上游填，交做发版的会话；本仓的 gate 薄壳已写明以 gate.sh 的 run_stage 为准
- `replace-batch.py --help` 直接抛 FileNotFoundError、没有用法说明：记一笔，不在这一轮改

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| CLAUDE.md:6 | 该一行代码（错字） | 改了：「改」；A-18 |
| CLAUDE.md:16 | 改了本文件要新开会话才对派出去的 agent 生效 | 改了：写明只对临时派的 agent 成立，定义开了 omitClaudeMd 不读本文件；records 指向补劝阻句；A-7、A-9 |
| CLAUDE.md:22 | 用着发现问题就直接改…跑相关门禁（47、62、63 与 doc-lint…），不先问 | 改了：用户定「出口内直改，出口外弹窗」；补 72、68 号与两道共享阶段的单跑命令；A-4、A-5、A-6 |
| CLAUDE.md:108 | 固定脚本（…各一次）；第一版 23 条加 …（18 条名单） | 改了：次数指到 02-second-txn.md 步 0；不变量名单指到 invariants.md，不再手抄（漏了 5 条）；A-1、A-2、A-3 |
| CLAUDE.md:49 | 填之前先看上游三语仓 `git status` | 不改：A-8 说本机只有 zh 仓，那是云端容器里只克隆了一份；i18n-sync.sh 默认在兄弟目录找各语言仓，用户本机成立 |
| .claude/main-agent.md:14 | 禁止在subagent中跑重型测试（…） | 改了：写明 crash-verifier、gate-triage 的例外；A-10 |
| .claude/main-agent.md:24 | 回到那张表再判一次 | 改了：点名第 2 步的三处与第 8 步收拢表；A-13 |
| .claude/main-agent.md:54 | 有腿交了模型或产物就派 three-way-verifier | 改了：补复跑命令、不派写理由；本地腿派哪条由主 agent 定并写理由；云端腿再抽由主 agent 再派；A-15、E-1、E-3、E-4（用户定） |
| .claude/main-agent.md:59 | 派 crash-verifier …跑 54 号 --full；54、55、57、59 照复用判定 | 改了：用户定层 0 全量由主 agent 建 worktree 并跑、经内存包装；57 号没有复用；A-11、B-6、B-7（用户定） |
| .claude/main-agent.md:60 | 撤回一个数、改格式常量、新立一条判据 | 改了：与 sweep 定义四种活同词；A-16 |
| .claude/main-agent.md:30 | 看门狗告警逐条列举（漏五类） | 改了：指到 agent-watch.py 文件头与 watch.conf；A-12 |
| .claude/agent-common.md:28 | evidence-discipline.md「所有旧数据都只是参考」 | 改了：指到原条所在；B-1 |
| .claude/agent-common.md:46 | 重型测试（手抄清单）…只跑自己动到的测试二进制 | 改了：清单指到 implementation-workflow.md；补 run-with-memory-cap.sh；15、74 号按轻阶段；「只跑」改成「重型测试里只跑」；B-2、B-3、B-4（用户定）、B-26 |
| .claude/agent-common.md:55 | `&` 只在同一条命令随后用不带参数的 `wait` 等齐时用 | 改了：与 command-safety.md「并行不许把失败吃掉」一致；B-20 |
| .claude/agent-common.md:34 | 草稿目录里用什么写都行 | 改了：补 ⑦ 同一 inode 改脚本的例外；只有 Edit 的新建照排他写；B-21、B-23 |
| .claude/agent-common.md:57 | 写进草稿目录里的进度记录 | 改了：与 main-agent.md 同名；A-17 |
| .claude/agents/crash-verifier.md:25 | 54 号默认不带 --full，派发提示点名要层 0 全量时才带 | 改了：层 0 全量归主 agent；E142 补简称；B-7（用户定） |
| .claude/agents/gate-triage.md:26 | 54、55、57、59 照各自的复用判定走…其余阶段单跑时不带前缀 | 改了：57 号现跑；整轮之后不再单跑；B-6、B-22 |
| .claude/agents/gate-triage.md:32 | update-ref refs/singlefs/gate-ok | 改了：ref 名与 --staged 不写；B-5 |
| .claude/agents/implementation-writer.md:28 | 追加进 mutations-append.tsv；check.sh 那一套 lint；门禁阶段都不跑；交回前 git apply --check | 改了：表名、lint 命令写全不跑 check.sh、跑登记给自己的阶段、删 apply --check（用户定主工作区改）；B-9、B-10、B-11、B-25 |
| .claude/agents/kb-scribe.md:32 | （无） | 补了：写入后钩子中途交回 ✗ 的处置；B-19 |
| .claude/agents/kb-scribe.md:39 | relabel-item.py 会改写的文件（漏 crates） | 改了：补 crates 并列给主 agent；标题行计数改成状态与索引页计数；预检检出指到草稿；B-17、B-18、B-24 |
| .claude/agents/mutation-triage.md:17 | bin 名以 [[bin]] 的 name 为准；59 号编不过的列在「没跑到」；三个数原样贴汇总行 | 改了：没登记的 bin 名；输入补 59 号输出路径；探针在副本上换；59 号栏目与数标记；B-13、B-14、B-15、B-16、E-15 |
| .claude/agents/sweep.md:22 | 例 F1–F6（U+2013） | 改了：stale-candidates.py 只认 ASCII 连字符；C-6 |
| .claude/agents/three-way-attack.md:37 | 要改代码试的，把仓拷到草稿目录…只在副本上改 | 改了：明写可以编译、经内存包装；小节名去掉已删的括注（上一轮）；C-12 |
| .claude/agents/three-way-local-attack.md:27 | 前台跑 ask-local.sh…；判红重跑沿用同一个号；oov-check.py <样本>；写范围没有 runlog | 改了：run_in_background（用户定）、`>\|` 重用号、退出码 6、oov-check 带提示文件与截断说明、写范围补 runlog；C-2、C-3、C-4、C-5 |
| .claude/agents/three-way-local-defense.md:11 | 读攻方的「做什么」「写范围」两节 | 改了：补「输入」一节；A-14 |
| .claude/agents/three-way-verifier.md:21 | 快照路径（crates/ 与 .claude/kb/ 至少这两样）；到快照里取那一行 | 改了：快照是 sha256 清单，按 sha256sum -c 与倒推副本核；C-1 |
| .claude/agents/three-way-materials.md:24 | 47 号红时要停下的脚本名单 | 改了：补 quote-rust-items.py；C-15 |
| .claude/agents/experiment-runner.md:28 | 写变异表 research/mutations/e<号>_<简称>.tsv…；变异三个数 | 改了：入库装置变异行进 crates/mutations.tsv、三个数待 59 号；文件名用英文名；decision-links-pending 空时跳过；报超时与内存撞顶；E152 补简称；C-8、C-9、C-13、C-14 |
| .claude/agents/experiment-designer.md:28 | --exclude-dir=experiments …（排不掉索引文件）；实验简称 | 改了：补排除文件与排不干净时的处置；登记里定英文名；C-9、C-10 |
| .claude/gate.d/54-layer0-replay.sh:232 | 出路里 --full 那行不带前缀 | 改了：带前缀并经内存包装，照抄不再被 heavy-test-guard 拒；B-8 |
| .claude/rules/implementation-workflow.md:48 | 重型测试清单（与钩子不全一致）；pre-commit hook 照旧跑整轮门禁；54、55、57、59 复用 | 改了：清单与钩子「按类」对齐并指过去、15/74 轻阶段、删 pre-commit（用户：本机没装）、57 号照跑、层 0 全量主 agent 跑、腿名、E152 简称；B-3、B-4、D-5、D-6、D-11、D-18、D-19 |
| .claude/rules/fs-design.md:137 | C8（范围判定）与 C19（腰线渗透）；D5 索引表第 8 行；「不许指向核心层的对象」；见第 1 条；incompat 示例；裸 D 编号；预留空位 | 改了：简称、D5 已定项 4 第 8、9 行、D21 出处、指到四样成本第 1 条、示例加示意说明、补 D 简称、补位图宽度与登记位；D-1、D-2、D-10、D-11、D-15、D-16、D-17 |
| .claude/rules/format-evolution.md:26 | 要按门禁 20 号写「—— 半定（一项未定）」；门禁管哪一半只点 75 号；27 号从此替你盯着 | 改了：括注写法与两道门禁的实际判法；补 99 号；27 号射程；D-7、D-8、D-9 |
| .claude/rules/path-moves.md:37 | 历史类文件保留旧名；那次 17 个 / 那次 C232；本规则的登记表；搬迁没有门禁兜；doc-lint 判全仓引用 | 改了：用户定规则照实际改；删经过括注；登记表指到 term-renames.md；写明哪些有阶段兜；编号简称的检查射程；D-3、D-4、D-12、D-13、D-14 |
| .claude/rules/three-way-inference.md:145 | （粗体开头的正文，不是标题） | 改了：提成小节标题，定义与 main-agent 的指向按 grep -n ^# 找得到；C-11 |
| .claude/rules/three-way-inference.md:21 | 派这一轮缺的那一侧；云端 A / 云端 B；反推腿；对齐三步；只有辩方复核的轮可以不派 | 改了：用户定主 agent 定并写理由；腿名与定义名一致；三步各归谁；核查员条件；E-1、E-2、E-3 |
| .claude/rules/three-way-inference.md:163 | 云端腿同样适用（谁再抽没写）；一个检测器只查得了一类；本地腿要前台跑；看到「没做」就是没验 | 改了：用户定主 agent 再派；两个检测器的分工；run_in_background（用户定）；没跑成与没做都退 6；E-4、E-6、E-7、E-8 |
| .claude/rules/three-way-inference.md:173 | 草稿与临时文件放腿自己的目录（/tmp/claude-1000/） | 不改：用户定改成按 uid 取，另起第 4 轮连同写范围表与钩子一起改；E-5、B-12 |
| .claude/rules/mutation-sampling.md:11 | ## 三类，判据不同（全文编到第八类）；后面的条目一条都不跑；引号里的句子不在 test-discipline 原文；三个数怎么数 | 改了：标题与八类说明、锚点在派活之前逐条核、退出码 5 不算数、引文改转述并指出处、三数怎么数与其余几栏；E-10、E-11、E-12、E-13、E-14 |
| .claude/skills/crash-test/SKILL.md:10 | （薄壳只指共享正文） | 补了：四样各落哪道门禁、重型谁跑；E-16 |
| .claude/skills/gate/SKILL.md:10 | （薄壳只指共享正文） | 补了：gate.sh / check.sh 在本项目是重型、快速反馈单跑轻阶段、共享阶段表不全以 gate.sh 为准；E-17、E-18 |
| .claude/skills/decide/SKILL.md:10 | （薄壳只指共享正文） | 补了：本项目一条决策一个文件、格式以 format-evolution.md 为准；E-19 |
| research/scripts/ask-local.sh:104 | 检测器没跑成 / 找不到时只打 stderr、退出码 0 | 改了：用户定改成退 6、不打正文、留作废副本；文件头补实词自复读；E-6、E-7 |
| research/scripts/ask-local-selftest.sh:61 | （无这两个用例；依赖本机 ~/code/ai-center 的 key） | 补了：自带假 key、检测器崩了与找不到两个用例；把被测脚本换成 HEAD 那一版，新增 6 条断言全红 |
| research/scripts/stale-candidates.py:23 | 命中不许超过 150 行 | 改了：文件头与代码常量、sweep 定义一致（三处同出 852e4b3，代码按 500 判）；C-7 |
| .claude/kb/checks-owed.md:195 | C234 题面引了规则已删的「被判那一项所在文件」 | 不改：kb 欠账题面归 kb-scribe，留给下一轮；E-9 |
| .claude/kb/vm-harness.md:161 | 谁在拦：门禁 65 号（relay-timing-lint.py） | 不改：kb 正文，留给下一轮 kb-scribe（第 1 轮同步记录已登记） |

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 核对员报告 A 组 | research/prompts/governance-rot-r2-audit-A.md | 61 |
| 核对员报告 B 组 | research/prompts/governance-rot-r2-audit-B.md | 57 |
| 核对员报告 C 组 | research/prompts/governance-rot-r2-audit-C.md | 46 |
| 核对员报告 D 组 | research/prompts/governance-rot-r2-audit-D.md | 82 |
| 核对员报告 E 组 | research/prompts/governance-rot-r2-audit-E.md | 67 |
