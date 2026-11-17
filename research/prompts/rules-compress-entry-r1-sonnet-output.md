# rules-compress-entry-r1 正推腿报告

日期：2026-11-16。轮名 rules-compress-entry-r1，第一轮。立场：逐条核落点。

没有写模型，没有复跑命令。辅助脚本只读仓里文件、按整串找行号，放在草稿目录 `~/.local/share/singlefs/work/rules-compress-entry-r1-sonnet/`（`landing_rows.py` 的 sha256 是 c321b2009492ddad57b03c543d590030ec1d1bd7e88b4a2d01bc7076cd9334dc，`pointer_check.py` 是 4f376b360059d86bfa7f2323612c4df20bf672e05a718fe19f7a37e1fc982643）。

## 被判对象核对

原件两份与快照 `research/prompts/rules-compress-entry-r1-snapshot/judged-sha256.txt` 逐份相同；合稿树四份与 `merged-tree-sha256.txt` 逐份相同（命令 `sha256sum`，量的）：

```
87510a525cc97c082a47c119c95c5f7c76e2a59faa24cbc7a6a8bc42ddb048a7  CLAUDE.md
c310eb2f472bca9172c9e5bb5d0be21f0da5c3a752191ffb90014e41b5293bc9  .claude/main-agent.md
ca5c5a6f1d688650b966b0211f94128ddb5f0f7d0daa4a9406f10a66948e3f0d  research/prompts/rules-compress-r1-merged/CLAUDE-merged.md
ff0259970433b696c3ebeb31a2c434065d54e56a2e9efac2f8fd4de203ce5d0b  research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md
253db0060aa5fd8ee54aeee695698746bed13462d68c67db9284e59a668f6c85  research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md
ca378a40977130d069c8ae8e17b9575a6b4554540eedf0ad4050a6deaecdb6c1  research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md
```

干到一半收到主 agent 的消息：合稿树里 `.claude/` 改名 `dot-claude/`、`CLAUDE.md` 改名 `CLAUDE-merged.md`，内容没变。收到时在读完四份合稿、开始逐条核指针的那一步；之后重读了正文，全部引文照新路径写，上面四个 sha256 是改名之后量的。

## 问题一：逐条落点表

做法：两份原件从头到尾逐句过，每条规则取原话里的一段，在合稿树里找承接它的那一句；两边都按整串在文件里现找行号（`landing_rows.py`，六批共 337 条，六批输出都是 misses=0，量的）。落点不在这一组四份文件里的，「说明」列写落在哪一类文件。合稿树里的路径前缀都是 `research/prompts/rules-compress-r1-merged/`。

### 原件 CLAUDE.md（91 条）

| 号 | 原件 | 合稿树落点 | 说明 |
|---|---|---|---|
| C01 | 「一个从零设计的 COW 文件系统，Rust 实现。」（`./CLAUDE.md:3`） | 「一个从零设计的 COW 文件系统，Rust 实现。」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:3`） |  |
| C02 | 「（它们的病历和解法），不是移植目标」（`./CLAUDE.md:4`） | 「（它们的病历和解法），不是移植目标」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:4`） |  |
| C03 | 「目的不是改一行代码，而是避免再发」（`./CLAUDE.md:6`） | 「目的不是改一行代码，而是避免再发」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:5`） |  |
| C04 | 「没做完的转进里程碑三项五（五·7 到五·11）」（`./CLAUDE.md:8`） | 「没做完的转进里程碑三项五（五·7 到五·11）」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:7`） | 里程碑一段，四个里程碑文件路径都在 |
| C05 | 「`scripts/crash/crash-amplification.sh --list-items` 列分项」（`./CLAUDE.md:8`） | 「`scripts/crash/crash-amplification.sh --list-items` 列分项」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:7`） |  |
| C06 | 「代码在 `crates/` 下十一个 crate」（`./CLAUDE.md:8`） | 「代码在 `crates/` 下十一个 crate」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:7`） | 矛盾总表 入口-7 |
| C07 | 「`CLAUDE.md` 只放公共上下文：项目是什么、当前里程碑、规则、项目本地事实」（`./CLAUDE.md:12`） | 「`CLAUDE.md` 只放公共上下文：项目是什么、当前里程碑、规则、项目本地事实」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:11`） |  |
| C08 | 「@.claude/main-agent.md」（`./CLAUDE.md:14`） | 「@.claude/main-agent.md」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:13`） |  |
| C09 | 「续做（SendMessage）沿用第一次派发时的定义；新建的定义要等几秒才派得出去」（`./CLAUDE.md:16`） | 「续做（SendMessage）沿用第一次派发时的定义；新建的定义要等几秒才派得出去」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:15`） |  |
| C10 | 「要新开会话才对临时派的 agent（general-purpose 这类）生效」（`./CLAUDE.md:16`） | 「要新开会话，才对临时派的 agent（general-purpose 这类）生效」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:15`） |  |
| C11 | 「要它们知道就写进定义或 `.claude/agent-common.md`」（`./CLAUDE.md:16`） | 「要它们知道就写进定义或 `.claude/agent-common.md`」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:15`） |  |
| C12 | 「计划与实测在 `records/2026-09-16-subagent拆分提案.md`（除非要查来历，别读它）」（`./CLAUDE.md:16`） | 「两版封存，不再改（除非要查来历，别读它们）」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:21`） | 矛盾总表 入口-8 |
| C13 | 「开着的欠账在 `records/2026-10-29-subagent拆分提案.md`」（`./CLAUDE.md:16`） | 「欠账记在现行账本 `records/2026-10-29-subagent拆分提案.md`」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:21`） |  |
| C14 | 「私有 memory 只放用户个人的偏好（提交时间窗、回复语言这类）」（`./CLAUDE.md:20`） | 「私有 memory 只放用户个人的偏好（提交时间窗、回复语言这类）」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:19`） |  |
| C15 | 「往 `CLAUDE.md` 加内容之前先问它属于哪个 agent，属于就写进那个定义或共用约束」（`./CLAUDE.md:20`） | 「往 `CLAUDE.md` 加内容之前先问它属于哪个 agent，属于就写进那个定义或共用约束」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:19`） |  |
| C16 | 「不占 `.claude/kb/` 的 C / D / E 编号，kb 只管文件系统本身」（`./CLAUDE.md:22`） | 「不占 `.claude/kb/` 的 C / D / E 编号，kb 只管文件系统本身」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:21`） |  |
| C17 | 「修它们本身就是这一轮的出口时，直接改，不先问」（`./CLAUDE.md:22`） | 「修它们本身就是这一轮的出口时，直接改，不先问」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:21`） |  |
| C18 | 「记下或弹窗，不自行扩大范围」（`./CLAUDE.md:22`） | 「记下或弹窗」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:21`） | 落点改成点名那一条禁令 |
| C19 | 「改完记进那份计划」（`./CLAUDE.md:22`） | 「改完记进现行账本」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:21`） | 矛盾总表 入口-9 |
| C20 | 「门禁归提交时，由提交前主 agent 照 `.claude/skills/gate/staged-gate.md` 跑的 `scripts/gate/gate-staged.sh` 判」（`./CLAUDE.md:22`） | 「再照 `.claude/skills/gate/staged-gate.md` 跑 `scripts/gate/gate-staged.sh` 整轮」（`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:41`） | 合稿的 CLAUDE-merged.md 里没有这一句，落在常驻规则 |
| C21 | 「脚本自证与门禁样本登记在 `.claude/hooks/selftest-registry.tsv`，改了就由主 agent 照钩子拒绝说明里写的起法经任务服务交」（`./CLAUDE.md:22`） | 「脚本自证与门禁样本登记在 `.claude/hooks/selftest-registry.tsv`，改了就由主 agent 照钩子拒绝说明里写的起法经任务服务交」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:21`） |  |
| C22 | 「都按批走一轮三方、不登豁免」（`./CLAUDE.md:22`） | 「按批走一轮三方」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:21`） | 「不登豁免」在下一行的落点 |
| C23 | 「都按批走一轮三方、不登豁免」（`./CLAUDE.md:22`） | 「不等用户点名、不登豁免」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:27`） |  |
| C24 | 「其余几类还没有格判、由主 agent 写判决时逐份点名」（`./CLAUDE.md:22`） | 「这几类门禁还没判，由主 agent 写判决时逐份点名」（`research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:21`） |  |
| C25 | 「改了触发文件的写阶段同步记录（门禁 doc-process-records 的 knowledge-sync 格判）」（`./CLAUDE.md:22`） | 「写阶段同步记录（门禁 doc-process-records 的 knowledge-sync 格判）」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:21`） |  |
| C26 | 「上游四篇不 `@` 常驻」（`./CLAUDE.md:34`） | 「上游四篇不 `@` 常驻」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:33`） |  |
| C27 | 「<!-- doc-lint:read-once design-doc-discipline.md pushback-discipline.md code-discipline.md preflight-discipline.md -->」（`./CLAUDE.md:35`） | 「<!-- doc-lint:read-once design-doc-discipline.md pushback-discipline.md code-discipline.md preflight-discipline.md -->」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:34`） |  |
| C28 | 「**对外以 `-en` 为准**」（`./CLAUDE.md:41`） | 「**对外以 `-en` 为准**」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:40`） |  |
| C29 | 「**拷贝，不是符号链接**」（`./CLAUDE.md:42`） | 「**拷贝，不是符号链接**」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:41`） |  |
| C30 | 「rsync -a --exclude '.git' --exclude CLAUDE.md ../singlefs-ai-sop-zh/ .claude/singlefs-ai-sop/」（`./CLAUDE.md:42`） | 「rsync -a --exclude '.git' --exclude CLAUDE.md ../singlefs-ai-sop-zh/ .claude/singlefs-ai-sop/」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:41`） |  |
| C31 | 「diff -rq --exclude=.git --exclude=CLAUDE.md ../singlefs-ai-sop-zh .claude/singlefs-ai-sop」（`./CLAUDE.md:42`） | 「diff -rq --exclude=.git --exclude=CLAUDE.md ../singlefs-ai-sop-zh .claude/singlefs-ai-sop」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:41`） |  |
| C32 | 「不用 `cp -r` 同步」（`./CLAUDE.md:42`） | 「不用 `cp -r`」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:41`） |  |
| C33 | 「门禁第一阶段拿它跟副本的 `VERSION` 比，对不上就红」（`./CLAUDE.md:43`） | 「门禁第一阶段拿它跟副本的 `VERSION` 比，对不上就红」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:42`） |  |
| C34 | 「不跑 `bump.sh`、不动 `VERSION`、不改 MANIFEST / SOURCE-MANIFEST 与译文首行的溯源哈希、不写带版本号的 CHANGELOG 条目、不提交，这些归做发版的会话」（`./CLAUDE.md:44`） | 「不跑 `bump.sh`、不动 `VERSION`、不改 MANIFEST / SOURCE-MANIFEST 与译文首行的溯源哈希、不写带版本号的 CHANGELOG 条目、不提交，这些归做发版的会话」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:43`） |  |
| C35 | 「填之前先看上游三语仓 `git status`，有人在发版就等它提交完，填完告诉发版会话填了哪些文件哪几段」（`./CLAUDE.md:44`） | 「填之前先看上游三语仓 `git status`，有人在发版就等它提交完，填完告诉发版会话填了哪些文件哪几段」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:43`） |  |
| C36 | 「发版之后同步副本、跑 `bash .claude/singlefs-ai-sop/install.sh` 刷版本戳」（`./CLAUDE.md:44`） | 「发版之后同步副本、跑 `bash .claude/singlefs-ai-sop/install.sh` 刷版本戳」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:43`） |  |
| C37 | 「**不许在 `.claude/singlefs-ai-sop/` 里就地改**」（`./CLAUDE.md:44`） | 「**不许在 `.claude/singlefs-ai-sop/` 里就地改**」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:43`） |  |
| C38 | 「**作业在本仓，不在上游仓**」（`./CLAUDE.md:44`） | 「**作业在本仓，不在上游仓**」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:43`） |  |
| C39 | 「判变异没红、无效变异之前读 `.claude/rules/mutation-sampling.md`」（`./CLAUDE.md:68`） | 「判变异没红、无效变异之前读 `.claude/rules/mutation-sampling.md`」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:62`） |  |
| C40 | 「写测试用例、入口脚本或自证之前读 `.claude/rules/test-script-discipline.md`」（`./CLAUDE.md:68`） | 「写测试用例、入口脚本或自证之前读 `.claude/rules/test-script-discipline.md`」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:62`） |  |
| C41 | 「搬文件或目录、改全仓术语之前读 `.claude/rules/path-moves.md`」（`./CLAUDE.md:68`） | 「搬文件或目录、改全仓术语之前读 `.claude/rules/path-moves.md`」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:62`） |  |
| C42 | 「新加脚本、往 `scripts/` 或 `research/` 放文件之前读 `.claude/rules/directory-layout.md`」（`./CLAUDE.md:68`） | 「新加脚本、往 `scripts/` 或 `research/` 放文件之前读 `.claude/rules/directory-layout.md`」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:62`） |  |
| C43 | 「`.claude/rules/pushback-discipline-by-review.md` 开会话先整份读一次、之后按名字回查」（`./CLAUDE.md:70`） | 「`.claude/rules/machine-first.md` 开会话先整份读一次、之后按名字回查」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:64`） | 另三篇并进 machine-first.md，并得对不对归常驻那一题 |
| C44 | 「`.claude/rules/code-discipline-by-review.md`、`.claude/rules/test-discipline.md` 由派出去的 agent 的定义点名读，主 agent 自己写 `crates/`、测试或脚本之前先整份读一次」（`./CLAUDE.md:70`） | 「`.claude/rules/code-discipline-by-review.md`、`.claude/rules/test-discipline.md` 由派出去的 agent 的定义点名读，主 agent 自己写 `crates/`、测试或脚本之前先整份读一次」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:64`） |  |
| C45 | 「只有本工程需要的纪律（文件系统怎么设计、压在本机资源上的流程）放 `.claude/rules/`，不往上游推」（`./CLAUDE.md:73`） | 「只有本工程需要的纪律（文件系统怎么设计、压在本机资源上的流程）放 `.claude/rules/`，不往上游推」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:69`） |  |
| C46 | 「理由、实测、经过、定案日期一律不写」（`./CLAUDE.md:75`） | 「照 `.claude/singlefs-ai-sop/rules/rules-discipline.md` 只写怎么做」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:69`） | 细目留在上游那一份（第 14–16 行那张表），合稿只留指针 |
| C47 | 「**决策索引**：编号、简称、状态、进度、指向正文的链接」（`./CLAUDE.md:81`） | 「**决策索引**：编号、简称、状态、进度、指向正文的链接」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:75`） |  |
| C48 | 「每个决策一个文件（`NN-简称.md`），正文与论证」（`./CLAUDE.md:82`） | 「每个决策一个文件（`NN-简称.md`），正文与论证」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:76`） |  |
| C49 | 「决策变更史，一条决策一节」（`./CLAUDE.md:83`） | 「决策变更史，一条决策一节」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:77`） |  |
| C50 | 「**实验索引**：编号、简称、状态、指向正文的链接」（`./CLAUDE.md:84`） | 「**实验索引**：编号、简称、状态、指向正文的链接」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:78`） |  |
| C51 | 「每个实验一个文件（`NN-简称.md`），正文与口径」（`./CLAUDE.md:85`） | 「每个实验一个文件（`NN-简称.md`），正文与口径」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:79`） |  |
| C52 | 「全部实验的变更史，一个实验一节」（`./CLAUDE.md:86`） | 「实验变更史，一个实验一节」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:80`） |  |
| C53 | 「不变量清单（按类拆开），checker 是它的可执行形式」（`./CLAUDE.md:87`） | 「不变量清单（按类拆开），checker 是它的可执行形式」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:81`） |  |
| C54 | 「门禁 code-source-discipline 的 feature-bits 格判两处逐位一致」（`./CLAUDE.md:88`） | 「门禁 code-source-discipline 的 feature-bits 格判两处逐位一致」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:82`） |  |
| C55 | 「`--check` 数还剩几处，没有门禁复查」（`./CLAUDE.md:89`） | 「`--check` 数剩几处，没有门禁复查」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:83`） |  |
| C56 | 「依据点名的分项存在且已定」（`./CLAUDE.md:90`） | 「写在 `.claude/gate.d/doc-decisions.sh` 文件头格名表那一格」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:84`） | 三条的细目换成指针，指针准不准见「一 丢失」第 1 处 |
| C57 | 「本机工具的现状与用法」（`./CLAUDE.md:91`） | 「本机工具的现状与用法」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:85`） |  |
| C58 | 「kb 的导航表，本身不放事实」（`./CLAUDE.md:92`） | 「kb 的导航表，本身不放事实」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:86`） |  |
| C59 | 「登记在 `.claude/history-carriers`」（`./CLAUDE.md:92`） | 「登记在 `.claude/history-carriers`」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:86`） |  |
| C60 | 「他家方案调研，含来源与口径」（`./CLAUDE.md:93`） | 「他家方案调研，含来源与口径」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:87`） |  |
| C61 | 「避坑清单，每做设计决定回来对一遍」（`./CLAUDE.md:94`） | 「避坑清单，每做设计决定回来对一遍」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:88`） |  |
| C62 | 「欠的检查：知道要拦什么但还拦不了的，含前置」（`./CLAUDE.md:95`） | 「欠的检查：知道要拦什么但还拦不了的，含前置」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:89`） |  |
| C63 | 「不约束执行，要立成规则先补那道检查」（`./CLAUDE.md:96`） | 「不约束执行，要立成规则先补那道检查」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:90`） |  |
| C64 | 「存不存盘、挂着能不能切与代码落点」（`./CLAUDE.md:97`） | 「存不存盘、挂着能不能切与代码落点」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:91`） |  |
| C65 | 「指不到的就是格式级空白，第八节是根槽写路径的段序列登记表」（`./CLAUDE.md:98`） | 「指不到的就是格式级空白，第八节是根槽写路径的段序列登记表」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:92`） |  |
| C66 | 「怎么把实验送进虚机在真块设备上跑」（`./CLAUDE.md:99`） | 「怎么把实验送进虚机在真块设备上跑」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:93`） |  |
| C67 | 「三样验证手段（checker、事务层、崩溃点重放）怎么落地、被谁挡着」（`./CLAUDE.md:100`） | 「三样验证手段（checker、事务层、崩溃点重放）怎么落地、被谁挡着」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:94`） |  |
| C68 | 「每步设想、验收标准、写出的字节、会碰到的决策点；每步开工前回来改」（`./CLAUDE.md:101`） | 「每步设想、验收标准、写出的字节、会碰到的决策点；每步开工前回来改」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:95`） |  |
| C69 | 「规矩在 `.claude/rules/directory-layout.md`」（`./CLAUDE.md:102`） | 「规矩在 `.claude/rules/directory-layout.md`」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:96`） |  |
| C70 | 「挑机器、内存额度、线程、GPU 与卡都由服务定，提交时不写」（`./CLAUDE.md:103`） | 「线程固定是登记的每台默认重活线程数（`.claude/heavy-thread-default`）、不许覆盖，其余由服务定」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:58`） | 事实表那一行只剩用途，判据落在技能与常驻规则 |
| C71 | 「挑机器、内存额度、线程、GPU 与卡都由服务定，提交时不写」（`./CLAUDE.md:103`） | 「清场与分卡都由服务做」（`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:46`） |  |
| C72 | 「子 agent 不交任务，要跑的命令写进报告交回主 agent」（`./CLAUDE.md:103`） | 「子 agent 一律不交任务」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:19`） |  |
| C73 | 「用法、判法与拒绝清单都在这份」（`./CLAUDE.md:103`） | 「用法、判法与拒绝清单」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:97`） |  |
| C74 | 「把承重的外部文献重新固定到本机（URL + sha256 + 引用方）」（`./CLAUDE.md:104`） | 「把承重的外部文献重新固定到本机（URL + sha256 + 引用方）」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:98`） |  |
| C75 | 「命中 `--match` 的进，命中 `--foreign` 的拒绝」（`./CLAUDE.md:105`） | 「命中 `--match` 的进，命中 `--foreign` 的拒绝」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:99`） |  |
| C76 | 「几个会话共写一批文件时只许这样改，不许整份重写」（`./CLAUDE.md:106`） | 「几个会话共写一批文件时只许这样改，不许整份重写」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:100`） |  |
| C77 | 「读时记 sha256，写前再比一次，变了就拒绝」（`./CLAUDE.md:107`） | 「写前比对读时记的 sha256，变了就拒绝」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:101`） |  |
| C78 | 「表由 `scripts/experiment-apparatus/e152-tables.py` 生成」（`./CLAUDE.md:108`） | 「表由 `scripts/experiment-apparatus/e152-tables.py` 生成」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:102`） |  |
| C79 | 「改一份就三份一起改」（`./CLAUDE.md:110`） | 「改一份就三份一起改」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:104`） |  |
| C80 | 「门禁 doc-text 的 trilingual 六格判」（`./CLAUDE.md:110`） | 「门禁 doc-text 的 trilingual 六格判」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:104`） |  |
| C81 | 「新术语先加进表再用」（`./CLAUDE.md:111`） | 「新术语先加进表再用」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:105`） |  |
| C82 | 「**没有 oracle。**」（`./CLAUDE.md:115`） | 「**没有 oracle。**」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:109`） |  |
| C83 | 「准入判据是门禁」（`./CLAUDE.md:116`） | 「准入判据是门禁」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:110`） |  |
| C84 | 「先定决策，再写代码」（`./CLAUDE.md:120`） | 「先定决策，再写代码」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:114`） |  |
| C85 | 「从事务开始，不从功能开始；第一个可运行目标是」（`./CLAUDE.md:121`） | 「从事务开始，不从功能开始；第一个可运行目标是」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:115`） |  |
| C86 | 「**只构成模型层的崩溃一致性证据**」（`./CLAUDE.md:122`） | 「**只构成模型层的崩溃一致性证据**」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:116`） |  |
| C87 | 「真设备那一侧只有门禁 qemu-device-streams 的六个模式」（`./CLAUDE.md:122`） | 「真设备那一侧只有门禁 qemu-device-streams 的六个模式」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:116`） |  |
| C88 | 「`grep -c '^crash-case:'` 数」（`./CLAUDE.md:122`） | 「`grep -c '^crash-case:'` 数」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:116`） |  |
| C89 | 「数法：`bash scripts/crash/crash-amplification.sh --list-items` 的输出行数」（`./CLAUDE.md:122`） | 「两个数的数法登记在 `.claude/gate.d/counted-claims.tsv`」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:116`） | 两条数的命令在仓里那张登记表第 5、6 行，现查在 |
| C90 | 「不进池级 checker 清单」（`./CLAUDE.md:122`） | 「不进池级 checker 清单」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:116`） |  |
| C91 | 「哪几条按回退候选集判、候选集的下界怎么取」（`./CLAUDE.md:122`） | 「哪几条按回退候选集判、候选集的下界怎么取」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:116`） |  |

`@` 引用的增减（命令 `diff <(grep '^@' CLAUDE.md) <(grep '^@' research/prompts/rules-compress-r1-merged/CLAUDE-merged.md)`，原样输出）：

```
17,19d16
< @.claude/rules/service-stop-readback.md
< @.claude/rules/artifact-line-pointer.md
< @.claude/rules/exhaustive-interleaving.md
21d17
< @.claude/rules/verify-before-claiming.md
25d20
< @.claude/rules/communication-discipline-by-review.md
```

少掉的五份在合稿树 `dot-claude/rules/` 下也没有同名文件（`ls` 现查，22 份里没有这五个名字）；它们并进了哪一份、并得对不对归常驻规则那一题，这条腿没核。上游七条 `@` 两边逐行相同。read-once 登记行原件 3 行、合稿 4 行（`grep -c 'doc-lint:read-once'`，量的），多的一行登的是参考件 `verification-reference.md`。

### 原件 .claude/main-agent.md（246 条）

| 号 | 原件 | 合稿树落点 | 说明 |
|---|---|---|---|
| M01 | 「它只写怎么做；为什么这么定、实测的数在」（`.claude/main-agent.md:3`） | 「它只写怎么做；为什么这么定、实测的数在」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:3`） |  |
| M02 | 「主 agent 只调度和判断：和用户说话、写三方正文与判决、定案、git 暂存与提交」（`.claude/main-agent.md:7`） | 「主 agent 只调度和判断：和用户说话、写三方正文与判决、定案、git 暂存与提交」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:9`） |  |
| M03 | 「task-server/task submit -- env MERGE_AND_VERIFY_KEEP_COPY=1 bash scripts/gate/merge-and-verify.sh <补丁目录…>」（`.claude/main-agent.md:7`） | 「task-server/task submit -- env MERGE_AND_VERIFY_KEEP_COPY=1 bash scripts/gate/merge-and-verify.sh <补丁目录…>」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:17`） | 原件四处抄同一条（第 7、33、57、118 行），合稿只这一处 |
| M04 | 「只跑补丁改到的测试目标与变异行点名的测试目标——不跑整个包，也不带上依赖它们的包」（`.claude/main-agent.md:7`） | 「只跑补丁改到的测试目标与变异行点名的测试目标（不跑整个包，也不带上依赖它们的包）」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:57`） |  |
| M05 | 「定案后主 agent 自己写得清的小处 kb 改动可以自己写」（`.claude/main-agent.md:7`） | 「主 agent 可以自己写的只有两类」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:9`） | 多了「只有」，见「二 二义」第 2 处 |
| M06 | 「自己写的 `crates/` 改动并进同一批的代码轮三方」（`.claude/main-agent.md:7`） | 「主 agent 自己写的 `crates/` 改动算进同一批」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:33`） |  |
| M07 | 「缺一样它不开工；共用约束在 `.claude/agent-common.md`」（`.claude/main-agent.md:7`） | 「缺一样它不开工；共用约束在 `.claude/agent-common.md`」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:9`） |  |
| M08 | 「不派 general-purpose；难的显式传 `model: opus`」（`.claude/main-agent.md:7`） | 「不派内置通用类型，难的显式传 `model: opus`」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:103`） |  |
| M09 | 「要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由」（`.claude/main-agent.md:10`） | 「要加崩溃点、要多枚举一档，就加；挂钟长不是收窄它的理由」（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:54`） | 常驻只留指针，原话在共用约束 |
| M10 | 「这一条管枚举域，不管跑的时机」（`.claude/main-agent.md:10`） | 「这一条管枚举域，不管跑的时机」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:13`） |  |
| M11 | 「门禁红了必须修好或查清原因，红着不提交」（`.claude/main-agent.md:10`） | 「门禁红了必须修好或查清原因，红着不提交」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:13`） |  |
| M12 | 「达成出口即终止任务，扩大任务必须弹窗」（`.claude/main-agent.md:11`） | 「达成出口即终止，扩大任务必须弹窗」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:14`） |  |
| M13 | 「每条各有验收标准，出口是全部达成」（`.claude/main-agent.md:11`） | 「每条各有验收标准，出口是全部达成」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:14`） |  |
| M14 | 「禁止因为产物变化无限继续任务」（`.claude/main-agent.md:11`） | 「禁止因为产物变化无限继续任务」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:14`） |  |
| M15 | 「如果验证为红给其他任务发消息，严禁自行修改扩大任务范围」（`.claude/main-agent.md:11`） | 「不属于自己任务的验证红了，照」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:14`） |  |
| M16 | 「全部写进 prune-server 自己的那份 RocksDB」（`.claude/main-agent.md:12`） | 「全仓只有一份判定库：prune-server 自己的那份 RocksDB」（`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:50`） | 各族的名单不再抄在入口 |
| M17 | 「见到新的判定存放处（另一个目录、TSV、sqlite、另一份 RocksDB）一律打回」（`.claude/main-agent.md:12`） | 「见到新的判定存放处（另一个目录、TSV、sqlite、另一份 RocksDB；并回即删的中转库与自证临时服务的库不算）」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:15`） |  |
| M18 | 「经 prune-server 客户端写进那一份库」（`.claude/main-agent.md:12`） | 「经 prune-server 客户端写进那一份库」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:15`） |  |
| M19 | 「读会改变判定结局的本机独有状态、写死主机名与地址、只在一台才跑得起的，一律打回」（`.claude/main-agent.md:13`） | 「读会改变判定结局的本机独有状态、写死主机名与地址、只在一台跑得起，一律打回」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:15`） |  |
| M20 | 「当场修那个缺陷，不往统一任务服务里加只派一台的名单」（`.claude/main-agent.md:13`） | 「当场修那个缺陷，不往统一任务服务里加只派一台的名单」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:15`） |  |
| M21 | 「禁止立刻直接修改方向和结论，先检查代码中有没有bug」（`.claude/main-agent.md:14`） | 「禁止立刻直接修改方向和结论，先检查代码中有没有bug」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:16`） |  |
| M22 | 「diff 里有兼容旧版本的分支、旧布局号、迁移路径的，打回去删掉」（`.claude/main-agent.md:15`） | 「diff 里有兼容旧版本的分支、旧布局号、迁移路径的，打回去删掉」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:17`） |  |
| M23 | 「一段实现先把代码与测试基本写完，再统一合入、编译、跑测试与验证」（`.claude/main-agent.md:17`） | 「一段实现先把代码与测试基本写完，再统一合入、编译、跑测试与验证」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:18`） |  |
| M24 | 「不指定范围的只在提交时或用户要求时由主 agent 经服务交」（`.claude/main-agent.md:18`） | 「提交之外，用户要求的照跑；主 agent 自己判要跑的，先弹窗问用户，同意了才跑」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:26`） | 矛盾总表 入口-1（改了判据） |
| M25 | 「主 agent 随时经服务交」（`.claude/main-agent.md:18`） | 「主 agent 随时经任务服务交，点这一次改到的项、格与目标」（`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:33`） |  |
| M26 | 「三遍写进调度记录与那一条点名的确认文件」（`.claude/main-agent.md:19`） | 「三遍确认另写进调度记录」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:20`） |  |
| M27 | 「提交时带确认标识 `--whole-package-confirmed <确认文件>`」（`.claude/main-agent.md:19`） | 「那一项的命令在 submit 后面带 `--whole-package-confirmed <那份文件的绝对路径>`」（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:65`） |  |
| M28 | 「带了标识照样累计满两小时推提醒、服务不停」（`.claude/main-agent.md:19`） | 「累计满两小时服务不停、推提醒给提交它的会话」（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:65`） |  |
| M29 | 「task-server/task confirm <号> --why」（`.claude/main-agent.md:19`） | 「task-server/task confirm <号> --why」（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:65`） |  |
| M30 | 「没带 `--full-gate` 的累计满四小时服务停这一次尝试、记结局 `over-four-hours` 打回，出路是收小范围或分片重交」（`.claude/main-agent.md:19`） | 「没带 `--full-gate` 的累计满四小时服务停这一次尝试、记结局 `over-four-hours` 打回，出路是收小范围或分片重交」（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:65`） |  |
| M31 | 「子 agent 报告里跑整个包的那一项没写三遍的不交，新派同类 agent 改成点名」（`.claude/main-agent.md:19`） | 「新派同类 agent 改成点名」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:20`） |  |
| M32 | 「证红、`--selftest`、样本档、弄坏开关、命名检查、喂钩子、实验与模型复跑」（`.claude/main-agent.md:20`） | 「证红、`--selftest`、样本档、弄坏开关、命名检查、喂钩子、实验与模型复跑」（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:55`） |  |
| M33 | 「主 agent 照原样经服务交、后台起 `task-server/task wait <号>` 等叫醒，跑完新派一个同类 agent」（`.claude/main-agent.md:20`） | 「主 agent 照原样经服务交、后台起 `task-server/task wait <号>` 等叫醒，跑完新派一个同类 agent」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:45`） |  |
| M34 | 「要关哪几项、做到什么算完（形态照实验的岔路单）」（`.claude/main-agent.md:24`） | 「要关哪几项、做到什么算完（形态照实验的岔路单）」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:24`） |  |
| M35 | 「没有依赖、输入齐的同类任务各成一批、在同一条消息里同时派」（`.claude/main-agent.md:24`） | 「没有依赖、输入齐的同类任务各成一批、在同一条消息里同时派」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:24`） |  |
| M36 | 「调查各派各的调查员，实验各派各的执行员（分开实验）」（`.claude/main-agent.md:24`） | 「调查各派各的调查员，实验各派各的执行员（分开实验）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:12`） |  |
| M37 | 「类别按 agent 定义分（调查、调研、改 `crates/`、改门禁与定义与脚本、写回 kb、跑实验、回扫、分诊、三方腿各是一类）」（`.claude/main-agent.md:24`） | 「类别按 agent 定义分（调查、调研、改 `crates/`、改门禁与定义与脚本、写回 kb、跑实验、回扫、分诊、三方腿各是一类）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:12`） | 矛盾总表 入口第二遍-1 |
| M38 | 「改 `crates/` 与改门禁不合给一个 agent」（`.claude/main-agent.md:24`） | 「改 `crates/` 与改门禁不合给一个 agent」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:12`） |  |
| M39 | 「同时在跑的子 agent 最多 5 个，不分模型（派发闸 ⑦ 拒第 6 个），超出的排下一波」（`.claude/main-agent.md:24`） | 「同时在跑的子 agent 最多 5 个，不分模型（派发闸 ⑦ 拒第 6 个），超出的排下一波」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:24`） |  |
| M40 | 「有依赖的只串依赖那一环；散的活不排成一串，合成大块」（`.claude/main-agent.md:24`） | 「有依赖的只串依赖那一环；散的活不排成一串，合成大块」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:12`） |  |
| M41 | 「答不出就还没规划完」（`.claude/main-agent.md:24`） | 「答不出就还没规划完」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:12`） |  |
| M42 | 「写死这一轮的派发上限与截止批次」（`.claude/main-agent.md:24`） | 「写死这一轮的课题、出口、派发上限与截止批次」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:24`） |  |
| M43 | 「<调度记录>#<批号>」（`.claude/main-agent.md:24`） | 「<调度记录>#<批号>」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:60`） |  |
| M44 | 「批内冒出的发现只进下一轮的清单，不当场派」（`.claude/main-agent.md:24`） | 「批内冒出的发现只进下一轮的清单，不当场派」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:24`） |  |
| M45 | 「这个决策能不能留到下次开？它挡不挡着本轮的课题？」（`.claude/main-agent.md:25`） | 「这个决策能不能留到下次开？它挡不挡着本轮的课题？」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:25`） |  |
| M46 | 「同时写出登的那一行的编号或 文件:行」（`.claude/main-agent.md:25`） | 「同一行写出登的那一行的编号或 文件:行」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:25`） |  |
| M47 | 「登记在 `.claude/gate.d/deferral-registries.tsv`」（`.claude/main-agent.md:25`） | 「`.claude/gate.d/deferral-registries.tsv` 登记的流程文件」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:25`） |  |
| M48 | 「只写进调度记录不算延后，算没去向」（`.claude/main-agent.md:25`） | 「只写进调度记录不算延后，算没去向」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:25`） |  |
| M49 | 「判据不是花了多少 token、跑了多久」（`.claude/main-agent.md:25`） | 「判据不是花了多少 token、跑了多久」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:25`） |  |
| M50 | 「提交之外任务确实要跑，先弹窗问用户，同意了才跑」（`.claude/main-agent.md:26`） | 「先弹窗问用户，同意了才跑」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:26`） | 矛盾总表 入口-1 |
| M51 | 「按要动的 `crates/` 文件聚簇」（`.claude/main-agent.md:27`） | 「按要动的 `crates/` 文件聚簇」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:27`） |  |
| M52 | 「同时派几个时一律交补丁」（`.claude/main-agent.md:27`） | 「实现员一律交补丁，只派一个也这样」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:14`） | 用户定案第 1 条 |
| M53 | 「`crates/mutations.tsv` 只追加、不算撞文件」（`.claude/main-agent.md:27`） | 「`crates/mutations.tsv` 只追加、不算撞文件」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:14`） |  |
| M54 | 「与在跑的实现员撞文件的等它交回、进下一批，不给在跑的追加文件」（`.claude/main-agent.md:27`） | 「与在跑的实现员撞文件的等它交回、进下一批，不给在跑的追加文件」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:14`） |  |
| M55 | 「要改一轮还没判决的代码轮快照里的文件，等那一轮判决（派发闸 ⑥）」（`.claude/main-agent.md:27`） | 「要改一轮还没判决的代码轮快照里的文件，等那一轮判决（派发闸 ⑥）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:14`） |  |
| M56 | 「在 `research/prompts/abandoned-rounds.tsv` 登记一行」（`.claude/main-agent.md:27`） | 「在 `research/prompts/abandoned-rounds.tsv` 登一行」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:14`） | 指针从规则那一节改指技能 three-way 的同名节，那一节在（第 31 行） |
| M57 | 「一周合一批、一批走一轮三方，不为一条发现开一轮」（`.claude/main-agent.md:27`） | 「一周合一批、一批走一轮三方，不为一条发现开一轮」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:27`） |  |
| M58 | 「推出来、没量过的，句子里写明」（`.claude/main-agent.md:28`） | 「推出来、没量过的，句子里写明」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:28`） |  |
| M59 | 「说不出来的，那条该进记录、不该进本轮」（`.claude/main-agent.md:29`） | 「说不出来的那条该进记录、不该进本轮」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:29`） |  |
| M60 | 「调查、调研类的派发点名的是要回答的问题，不是已抓到的问题」（`.claude/main-agent.md:29`） | 「调查、调研类点名的是要回答的问题，不是已抓到的问题」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:29`） |  |
| M61 | 「延下来的哪些本轮开、哪些继续留、哪些不做，逐条写去向」（`.claude/main-agent.md:30`） | 「延下来的哪些本轮开、哪些继续留、哪些不做，逐条写去向」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:30`） | 矛盾总表 入口-6 |
| M62 | 「一条都不许无声消失」（`.claude/main-agent.md:31`） | 「一条都不许无声消失」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:31`） |  |
| M63 | 「task-server/task submit -- python3 scripts/agents/agent-cost.py usage --since <开工日期>」（`.claude/main-agent.md:31`） | 「task-server/task submit -- python3 scripts/agents/agent-cost.py usage --since <开工日期>」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:16`） |  |
| M64 | 「agent-cost.py outcome --since <开工日期> --until <这一天> --summary --out <仓根>/records/<轮>-outcome-<这一天>.txt」（`.claude/main-agent.md:31`） | 「agent-cost.py outcome --since <开工日期> --until <这一天> --summary --out <仓根>/records/<轮>-outcome-<这一天>.txt」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:16`） |  |
| M65 | 「`--out` 只排他新建，那个文件已在就拒，换一个名字，不覆盖」（`.claude/main-agent.md:31`） | 「`--out` 只排他新建，那个文件已在就换一个名字」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:16`） |  |
| M66 | 「只核这一批自己的改动还在不在，别人的改动不追查、不拦」（`.claude/main-agent.md:32`） | 「只核这一批自己的改动还在不在，别人的改动不追查、不拦」（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:52`） |  |
| M67 | 「表里现有的行补完一轮三方删一行，只缩不涨」（`.claude/main-agent.md:32`） | 「不再加行，现有的行补完一轮三方删一行」（`research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:10`） |  |
| M68 | 「送不到再弹窗让用户转达」（`.claude/main-agent.md:32`） | 「送不到再弹窗请用户转达」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:32`） |  |
| M69 | 「同时待写回的全部定案（几轮判决、几次用户定案）合成一批」（`.claude/main-agent.md:33`） | 「同时待写回的全部定案（几轮判决、几次用户定案）合成一批」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:33`） |  |
| M70 | 「条目多的按 kb 文件不相交切成几份、同时派几个写回员」（`.claude/main-agent.md:33`） | 「条目多的按 kb 文件不相交切给几个写回员同时写」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:98`） |  |
| M71 | 「task-server/task submit -- bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync」（`.claude/main-agent.md:33`） | 「task-server/task submit -- bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:83`） |  |
| M72 | 「半途交回、失败的先合已完成的部分，补派的归写阶段的下一批」（`.claude/main-agent.md:33`） | 「半途交回、失败的先合已完成的部分，补派的归写段的下一批」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:33`） |  |
| M73 | 「`--check feature-gated-tests --item feature-gated:<文件名>`」（`.claude/main-agent.md:33`） | 「--check feature-gated-tests --item feature-gated:<文件名>」（`scripts/gate/merge-and-verify.sh:223`） | 合稿树里只剩「照脚本文件头」，那一句在脚本文件头 |
| M74 | 「三样都没有时报本次未跑」（`.claude/main-agent.md:33`） | 「三样都没有就不起那一道」（`scripts/gate/merge-and-verify.sh:227`） | 同上，落点是脚本文件头 |
| M75 | 「只对合入时改过锚点或冲突的变异行 `prove-red.sh` 证红（实现员各自证过的不重证，整表归 crates-mutation-replay）」（`.claude/main-agent.md:33`） | 「只对合入时改过锚点或冲突的变异行跑 `scripts/mutation/prove-red.sh` 证红（实现员各自证过的不重证，整表归 crates-mutation-replay）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:47`） | 矛盾总表 入口第二遍-2 |
| M76 | 「子 agent 交回前、合入后验证都不跑门禁阶段」（`.claude/main-agent.md:33`） | 「子 agent 交回前不跑门禁阶段与各类 lint」（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:108`） |  |
| M77 | 「代码三方一轮攻这一批合入，快照取在合入之后，不按补丁开轮」（`.claude/main-agent.md:33`） | 「代码三方一轮攻这一批合入，快照取在合入之后，不按补丁开轮」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:33`） |  |
| M78 | 「写一批验一批、改一点验一点，同样不许」（`.claude/main-agent.md:33`） | 「写一批验一批、改一点验一点，同样不许」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:33`） |  |
| M79 | 「哪一步用完就在那一步删，不等一批收尾，不留给以后的趟或别的任务复用」（`.claude/main-agent.md:34`） | 「哪一步用完就在那一步删，不等一批收尾，不留给以后的趟或别的任务复用」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:34`） |  |
| M80 | 「删之前 `du -sh` 记大小，别的会话的不碰」（`.claude/main-agent.md:34`） | 「删之前 `du -sh` 记大小」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:18`） |  |
| M81 | 「照它的报告跑完那几条命令就删」（`.claude/main-agent.md:34`） | 「照它的报告跑完那几条命令就删」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:18`） |  |
| M82 | 「不往 `<仓根>/target` 编，跑完 `task-server/task clean <号>` 删，不自己 `rm`」（`.claude/main-agent.md:34`） | 「不往 `<仓根>/target` 编，跑完 `task-server/task clean <号>` 删，不自己 `rm`」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:18`） |  |
| M83 | 「SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1」（`.claude/main-agent.md:34`） | 「SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:18`） |  |
| M84 | 「`<工作根>/merge-and-verify/<一层>/repo`」（`.claude/main-agent.md:34`） | 「<工作根>/merge-and-verify/<一层>/repo」（`.claude/hooks/completion-cleanup-guard.sh:4`） | 技能写「判哪几处照它文件头」，那一句在钩子文件头 |
| M85 | 「删了大的之后提醒用户跑 `sudo fstrim -v /`（要密码，主 agent 不跑）」（`.claude/main-agent.md:34`） | 「删了大的之后提醒用户跑 `sudo fstrim -v /`（要密码，主 agent 不跑）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:18`） |  |
| M86 | 「会话过 40 万上下文就压缩上下文接着做」（`.claude/main-agent.md:35`） | 「会话过 40 万上下文就压缩上下文接着做」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:35`） |  |
| M87 | 「用户说了可以一直跑，就不停，接着开下一批」（`.claude/main-agent.md:35`） | 「用户说了可以一直跑，就不停，接着开下一批」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:35`） |  |
| M88 | 「不另写一份 grep；✗ 的逐条点名，红的当场记」（`.claude/main-agent.md:36`） | 「不另写一份 grep；✗ 的逐条点名，红的当场记」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:36`） |  |
| M89 | 「三栏分开报」（`.claude/main-agent.md:36`） | 「只进这一栏，不进 ✓，也不进 ✗」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:53`） | 括注删了，那一句在入口点名的那一节里 |
| M90 | 「python3 scripts/kb/append-record.py <记录> <文本文件>」（`.claude/main-agent.md:37`） | 「python3 scripts/kb/append-record.py <记录> <文本文件>」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:37`） |  |
| M91 | 「python3 scripts/agents/who-touched.py <路径或符号>… [--since-minutes N]」（`.claude/main-agent.md:38`） | 「python3 scripts/agents/who-touched.py <路径或符号>… [--since-minutes N]」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:20`） |  |
| M92 | 「子 agent 的改动归到父会话」（`.claude/main-agent.md:38`） | 「子 agent 的记录一律归到它的父会话」（`scripts/agents/who-touched.py:15`） | 技能写「读法照它文件头」，三栏的读法在脚本文件头 |
| M93 | 「Edit / Write 为 0 的会话没改过它」（`.claude/main-agent.md:38`） | 「Edit / Write 为 0 的会话没改过它」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:20`） |  |
| M94 | 「写明文件:行、原话、复现命令与改法建议」（`.claude/main-agent.md:38`） | 「写明文件:行、原话、复现命令与改法建议」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:20`） |  |
| M95 | 「请各自认领」（`.claude/main-agent.md:38`） | 「请各自认领」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:20`） |  |
| M96 | 「发了、谁认了、谁没认，都记进这一轮的调度记录」（`.claude/main-agent.md:38`） | 「发了、谁认了、谁没认，都记进这一轮的调度记录」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:38`） |  |
| M97 | 「`git apply --numstat` 上算撞文件」（`.claude/main-agent.md:46`） | 「`git apply --numstat` 上算撞文件」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:46`） |  |
| M98 | 「不拆成两批先后打，并进同一个实现员的同一份补丁一次改完」（`.claude/main-agent.md:46`） | 「不拆成两批，并进同一个实现员的同一份补丁一次改完」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:46`） |  |
| M99 | 「写段的补丁不打主工作区：实现员一律交补丁」（`.claude/main-agent.md:47`） | 「写段的补丁不打主工作区：实现员一律在草稿目录的副本里改、交补丁，只派一个也这样」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:47`） | 用户定案第 1 条 |
| M100 | 「python3 scripts/hosts/minimal-copy.py <仓根> <段树> --with-test-inputs --with-gate-tables」（`.claude/main-agent.md:47`） | 「python3 scripts/hosts/minimal-copy.py <仓根> <段树> --with-test-inputs --with-gate-tables」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:47`） |  |
| M101 | 「task-server/task submit -- python3 scripts/gate/apply-writer-patch.py <补丁目录> --root <段树>」（`.claude/main-agent.md:47`） | 「task-server/task submit -- python3 scripts/gate/apply-writer-patch.py <补丁目录> --root <段树>」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:47`） |  |
| M102 | 「不等合入后验证、不等代码轮」（`.claude/main-agent.md:47`） | 「不等合入后验证、不等代码轮」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:47`） |  |
| M103 | 「验证段第 1 步那一条交出去就删」（`.claude/main-agent.md:47`） | 「段树在验证段第 1 步那一条交出去时删」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:47`） |  |
| M104 | 「一段在写段里跨轮停下也删」（`.claude/main-agent.md:47`） | 「一段在写段里跨轮停下也删」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:47`） |  |
| M105 | 「删段树的同一步删这一行」（`.claude/main-agent.md:47`） | 「删段树的同一步删这一行」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:47`） |  |
| M106 | 「实现员只写代码、测试与变异行、自己跑 `cargo fmt`」（`.claude/main-agent.md:48`） | 「实现员只写代码、测试与变异行、自己跑 `cargo fmt`」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:48`） |  |
| M107 | 「把动到的测试目标、clippy、build、证红、命名检查与带 `--item` 的门禁与崩溃放量入口（点自己改到的项）的命令写进报告」（`.claude/main-agent.md:48`） | 「动到的测试目标、clippy / build、证红、命名检查、登记的崩溃枚举用例」（`research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:50`） | 入口写「清单照」那一行，清单在常驻规则的表里 |
| M108 | 「这是写的一部分，不算验证」（`.claude/main-agent.md:48`） | 「这几条命令算写的一部分，不算验证」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:48`） |  |
| M109 | 「留到验证段一起看，不为它单派」（`.claude/main-agent.md:49`） | 「留到验证段一起看，不为它单派」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:49`） |  |
| M110 | 「挡着这一段的，并进写段的下一批接着写」（`.claude/main-agent.md:50`） | 「挡着这一段的，并进写段的下一批接着写」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:50`） |  |
| M111 | 「主工作区里没有打了而没验过的补丁」（`.claude/main-agent.md:51`） | 「主工作区里没有打了而没验过的补丁」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:51`） |  |
| M112 | 「没有已知要写而没写的」（`.claude/main-agent.md:53`） | 「没有已知要写而没写的」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:53`） |  |
| M113 | 「没带 `env MERGE_AND_VERIFY_KEEP_COPY=1` 的那一趟不留快照、不打合进主工作区的命令」（`.claude/main-agent.md:57`） | 「没带 `env MERGE_AND_VERIFY_KEEP_COPY=1` 的那一趟不留快照、不打合进主工作区的命令」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:18`） |  |
| M114 | 「被下一趟替掉的那一趟留下的快照，下一趟交出去就删」（`.claude/main-agent.md:57`） | 「被下一趟替掉的那一趟的快照，下一趟交出去就删」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:19`） |  |
| M115 | 「退 3 用户不同意、这一批放弃、任务被停这三支同样删那一趟留下的快照」（`.claude/main-agent.md:57`） | 「退 3 而用户不同意、这一批放弃、任务被停，这三种情形同样删那一趟的快照」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:19`） | 矛盾总表 入口-2 |
| M116 | 「先 `task-server/task cancel <号>`，再 `du -sh` 记大小」（`.claude/main-agent.md:57`） | 「先 `task-server/task cancel <号>`，再 `du -sh` 记大小」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:19`） |  |
| M117 | 「绿了（末行 `TASK_SERVER_RESULT` 是 `exit=0`）不合，接着做第 2 步」（`.claude/main-agent.md:57`） | 「绿了不合、接第 2 步」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:57`） |  |
| M118 | 「带 `--whole-package-confirmed`，`--cwd` 写快照」（`.claude/main-agent.md:57`） | 「带 `--whole-package-confirmed`，`--cwd` 写快照」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:22`） |  |
| M119 | 「用户不同意的不合、这一批停在验证段等用户定」（`.claude/main-agent.md:57`） | 「用户不同意的不合，这一批停在验证段等用户定」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:22`） |  |
| M120 | 「打补丁核不过（脚本整批停、退 2）的，照它出路里段树那一句办」（`.claude/main-agent.md:57`） | 「退 2（打补丁核不过，脚本整批停）：照它出路里段树那一句办」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:24`） |  |
| M121 | 「不算绿、不进第 2 步，等服务回来照原命令重交」（`.claude/main-agent.md:57`） | 「不算绿、不进第 2 步，等服务回来照原命令重交」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:25`） |  |
| M122 | 「脚本先逐份核补丁已打在树上，核不过退 1」（`.claude/main-agent.md:57`） | 「脚本先逐份核补丁已打在树上，核不过退 1」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:26`） |  |
| M123 | 「别的不改写、不再插 submit」（`.claude/main-agent.md:57`） | 「别的不改写、不再插 submit」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:23`） |  |
| M124 | 「那一条退 2、说那一趟开跑以来主工作区变了的，照它的出路整批重交」（`.claude/main-agent.md:57`） | 「那一条退 2、说那一趟开跑以来主工作区变了的，照它的出路整批重交」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:23`） |  |
| M125 | 「那一条自己带 `--crash-expand-up-to <同一个 n>`」（`.claude/main-agent.md:57`） | 「那一条自己带同一个 n」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:23`） |  |
| M126 | 「开头都带范围声明 `SINGLEFS_RUN_SCOPE`，写法照 `.claude/hooks/run-scope-guard.sh` 文件头」（`.claude/main-agent.md:57`） | 「开头都带范围声明 `SINGLEFS_RUN_SCOPE`，写法照 `.claude/hooks/run-scope-guard.sh` 文件头」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:26`） |  |
| M127 | 「验证包 `singlefs-verification` 的目标带 `--release`」（`.claude/main-agent.md:118`） | 「验证包 singlefs-verification 的目标带 --release 跑」（`scripts/gate/merge-and-verify.sh:59`） | 合稿树四份里没有这半句，落点是脚本文件头 |
| M128 | 「`crash-case-check.py`（placement、modules、provenance、must-red）与 `refused-test-registry.py list` 两条」（`.claude/main-agent.md:58`） | 「`crash-case-check.py`（placement、modules、provenance、must-red）与 `refused-test-registry.py list` 两条」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:28`） |  |
| M129 | 「点那一段列出的这一批新写、改写的变异行，在全域上判」（`.claude/main-agent.md:58`） | 「点那一段列出的这一批新写、改写的变异行，在全域上判」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:30`） |  |
| M130 | 「补丁改了 `.rs` 的，命名检查两条」（`.claude/main-agent.md:58`） | 「补丁改了 `.rs` 的，命名检查两条」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:31`） |  |
| M131 | 「不缩域再判一次」（`.claude/main-agent.md:58`） | 「不缩域再判一次」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:32`） |  |
| M132 | 「这一趟要现判的崩溃状态超 200000 或估不出」（`.claude/main-agent.md:58`） | 「这一趟要现判的崩溃状态超 200000 或估不出」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:34`） |  |
| M133 | 「都不算绿，也不拿缩域顶替」（`.claude/main-agent.md:58`） | 「都不算绿，也不拿缩域顶替」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:34`） |  |
| M134 | 「带 `--full-gate` 服务当场拒，没有全域的跑法」（`.claude/main-agent.md:58`） | 「带 `--full-gate` 服务当场拒，没有全域的跑法」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:34`） |  |
| M135 | 「先拆再合（拆出的改动进写段下一批，这一批停在验证段）」（`.claude/main-agent.md:58`） | 「先拆再合（拆出的改动进写段下一批，这一批停在验证段）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:34`） |  |
| M136 | 「留给提交时不指定范围的那一趟判，写进调度记录，这一批照样合」（`.claude/main-agent.md:58`） | 「留给提交时不指定范围的那一趟判（写进调度记录，这一批照样合）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:34`） |  |
| M137 | 「副本从这棵快照建、修补以它为基线写，修补的补丁目录接在这一批最后」（`.claude/main-agent.md:58`） | 「副本从这棵快照建、修补以它为基线写，修补的补丁目录接在这一批最后」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:35`） |  |
| M138 | 「第 1 步连同它整批重交（不带 `--item` 与 `--fingerprint-of`，键相同的照复用）」（`.claude/main-agent.md:58`） | 「第 1 步连同修补的补丁目录整批重交（不带 `--item` 与 `--fingerprint-of`，键相同的照复用）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:14`） | 原件各处重复的这一套收成一条定义 |
| M139 | 「合进主工作区的只有第 1、2 步在同一棵快照上都绿了的那一批」（`.claude/main-agent.md:58`） | 「第 1、2 步在同一棵快照上都绿了才合进主工作区」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:59`） |  |
| M140 | 「前一条退 0 才做下一条，中间不夹别的活」（`.claude/main-agent.md:59`） | 「前一条退 0 才做下一条，中间不夹别的活」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:59`） |  |
| M141 | 「都退 0 才往下；退非 0 不合，第 1 步整批重交」（`.claude/main-agent.md:59`） | 「都退 0 才往下；退非 0 不合，整批重交」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:37`） |  |
| M142 | 「task-server/task submit -- python3 scripts/gate/apply-writer-patch.py <日志目录>/merge-patch」（`.claude/main-agent.md:59`） | 「task-server/task submit -- python3 scripts/gate/apply-writer-patch.py <日志目录>/merge-patch」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:38`） |  |
| M143 | 「它一次整份写完或一个字不改」（`.claude/main-agent.md:59`） | 「它一次整份写完或一个字不改」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:38`） |  |
| M144 | 「这两样等完重做核的那几条、再交同一条，不重验」（`.claude/main-agent.md:59`） | 「这两样等完重做核的那几条、再交同一条，不重验」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:38`） |  |
| M145 | 「不重做核的那几条、不再交同一条、不照拒绝说明手工并」（`.claude/main-agent.md:59`） | 「不重做核的那几条、不再交同一条、不照拒绝说明手工并」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:38`） |  |
| M146 | 「在 `<日志目录>/merge-patch/result.md` 末尾补一节再交同一条」（`.claude/main-agent.md:59`） | 「在 `<日志目录>/merge-patch/result.md` 末尾补一节再交同一条」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:38`） |  |
| M147 | 「task-server/task submit -- python3 scripts/gate/apply-writer-patch.py --pending-status」（`.claude/main-agent.md:59`） | 「task-server/task submit -- python3 scripts/gate/apply-writer-patch.py --pending-status」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:38`） |  |
| M148 | 「`--rollback-pending` 退干净（或 `--finish-pending` 换完）」（`.claude/main-agent.md:59`） | 「`--rollback-pending` 退干净（或 `--finish-pending` 换完）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:38`） |  |
| M149 | 「--read-outcome <日志目录>/merge-patch/apply-outcome.tsv`，退 0 才算合全」（`.claude/main-agent.md:59`） | 「--read-outcome <日志目录>/merge-patch/apply-outcome.tsv`，退 0 才算合全」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:39`） |  |
| M150 | 「核的那几条连着三趟退非 0」（`.claude/main-agent.md:59`） | 「核的那几条连着三趟退非 0」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:40`） |  |
| M151 | 「认不出、请不动的弹窗问用户」（`.claude/main-agent.md:59`） | 「认不出、请不动的弹窗问用户」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:40`） |  |
| M152 | 「退 0 的已合，接合完之后那几样；别的从第 1 步整批重交」（`.claude/main-agent.md:59`） | 「退 0 的已合，接合完之后那几样；别的从第 1 步整批重交」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:41`） |  |
| M153 | 「快照取在全部合入之后，攻整段的合入」（`.claude/main-agent.md:60`） | 「快照取在全部合入之后，攻整段的合入」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:60`） |  |
| M154 | 「合成一批派 `implementation-writer` 一次改全」（`.claude/main-agent.md:61`） | 「合成一批派 `implementation-writer` 一次改全」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:61`） |  |
| M155 | 「去掉 `--item` 与 `--fingerprint-of`、范围改写补丁份数」（`.claude/main-agent.md:61`） | 「去掉 `--item` 与 `--fingerprint-of`、范围改写补丁份数」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:44`） |  |
| M156 | 「修补的实现员副本从主工作区建，第 1 步那一条只给这一批修补的补丁目录，再走第 2 步」（`.claude/main-agent.md:61`） | 「修补的实现员副本从主工作区建，第 1 步那一条只给这一批修补的补丁目录，再走第 2 步」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:46`） |  |
| M157 | 「整段的整轮重跑留到提交前。不逐条改、逐条验」（`.claude/main-agent.md:61`） | 「整段的整轮重跑留到提交前。不逐条改、逐条验」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:61`） |  |
| M158 | 「实验装置与实验跑片接着自动开始，不等用户点头」（`.claude/main-agent.md:62`） | 「实验装置与实验跑片接着自动开始，不等用户点头」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:62`） |  |
| M159 | 「不接入的要二次确认」（`.claude/main-agent.md:66`） | 「不接入的要二次确认」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:66`） |  |
| M160 | 「只由主 agent 判过再登」（`.claude/main-agent.md:66`） | 「只由主 agent 判过再登」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:66`） |  |
| M161 | 「只含拒算表的补丁目录两样都照打」（`.claude/main-agent.md:66`） | 「都照打只含拒算表的补丁目录」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:52`） |  |
| M162 | 「能改成算得出单元键的（输入编进二进制、在测试里现造、不读运行时拼出的仓根路径）不收」（`.claude/main-agent.md:68`） | 「改得成算得出单元键的写法：输入编进二进制、在测试里现造、不读运行时拼出的仓根路径」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:54`） |  |
| M163 | 「实现员不登记，也不替它登记」（`.claude/main-agent.md:68`） | 「实现员不登记，主 agent 也不替它登记」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:68`） |  |
| M164 | 「测试函数全名<TAB>为什么非读活树不可」（`.claude/main-agent.md:68`） | 「测试函数全名<TAB>为什么非读活树不可」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:54`） |  |
| M165 | 「git diff --no-index --no-prefix --full-index a/.claude/gate.d/refused-tests.tsv b/.claude/gate.d/refused-tests.tsv > <补丁目录>/crates.patch」（`.claude/main-agent.md:68`） | 「git diff --no-index --no-prefix --full-index a/.claude/gate.d/refused-tests.tsv b/.claude/gate.d/refused-tests.tsv > <补丁目录>/crates.patch」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:54`） |  |
| M166 | 「替掉这一批里原来那一份」（`.claude/main-agent.md:68`） | 「替掉这一批里原来那一份」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:54`） |  |
| M167 | 「task-server/task submit --cwd <快照> -- python3 scripts/gate/refused-test-registry.py list <快照>」（`.claude/main-agent.md:69`） | 「task-server/task submit --cwd <快照> -- python3 scripts/gate/refused-test-registry.py list <快照>」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:55`） |  |
| M168 | 「这一批的第 2 步不等它」（`.claude/main-agent.md:69`） | 「这一批的第 2 步不等它」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:55`） |  |
| M169 | 「行不归这一批，主 agent 在主工作区直接删」（`.claude/main-agent.md:69`） | 「主 agent 在主工作区直接删」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:55`） |  |
| M170 | 「退 77（判定库服务不可用、清单不完整）不算判过，等服务回来重交」（`.claude/main-agent.md:70`） | 「退 77（判定库服务不可用、清单不完整）不算判过，等服务回来重交」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:70`） |  |
| M171 | 「审 diff 时照 `.claude/agents/implementation-writer.md` 第 3c 步看有没有读活树、有没有列进报告」（`.claude/main-agent.md:72`） | 「审 diff 时照 `.claude/agents/implementation-writer.md` 第 3c 步看有没有读活树、有没有列进报告」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:58`） | 矛盾总表 入口-11 |
| M172 | 「盯住，不能一直干等，也不强制结束」（`.claude/main-agent.md:76`） | 「盯住，不能一直干等，也不强制结束」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:74`） |  |
| M173 | 「主 agent 在它下一次回报之前处置或 `--ack`」（`.claude/main-agent.md:76`） | 「在看门狗下一次回报之前处置或 `--ack`」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:34`） |  |
| M174 | 「看门狗只报、不停子 agent 也不停进程」（`.claude/main-agent.md:76`） | 「看门狗只报，不停子 agent 也不停进程」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:74`） |  |
| M175 | 「告诉用户预计时长与哪几个读数算不正常」（`.claude/main-agent.md:78`） | 「告诉用户预计时长与哪几个读数算不正常」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:24`） |  |
| M176 | 「看门狗一个会话一只：会话里第一次派发之后」（`.claude/main-agent.md:80`） | 「看门狗一个会话一只，会话里第一次派发之后起」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:74`） |  |
| M177 | 「阈值在 `scripts/agents/watch.conf` 里配，不在命令行上手敲；调用里不加 `&`、`nohup`、`disown`」（`.claude/main-agent.md:80`） | 「阈值在 `scripts/agents/watch.conf` 里配，不在命令行上手敲；调用里不加 `&`、`nohup`、`disown`」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:26`） |  |
| M178 | 「它退出了（按告警退出、定时回报、全部交回）才再起一只，起之前先读它的输出」（`.claude/main-agent.md:80`） | 「它退出了（按告警退出、定时回报、全部交回）才再起一只，起之前先读它的输出」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:26`） |  |
| M179 | 「看门狗不读服务、不停任务」（`.claude/main-agent.md:80`） | 「不读服务、不停任务」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:26`） | 矛盾总表 入口-5 |
| M180 | 「一次工具调用超过 8 分钟、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、hook 检出本会话的问题命令、交回之后后台还在跑（被停的同样算）」（`.claude/main-agent.md:82`） | 「一次工具调用超过 8 分钟、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、hook 检出本会话的问题命令、交回之后后台还在跑（被停的同样算）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:29`） |  |
| M181 | 「根分区余量过低（工作根所在文件系统的可用量低于 `watch.conf` 的 `root-free-gib-alert`）」（`.claude/main-agent.md:82`） | 「根分区余量过低（工作根所在文件系统的可用量低于 `watch.conf` 的 `root-free-gib-alert`）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:29`） |  |
| M182 | 「没超时的等待循环、同一命令反复且输出不变、进度文件不涨（`scripts/agents/agent-watch.py` 的 `INFO_ALERT_NAMES`）」（`.claude/main-agent.md:82`） | 「没超时的等待循环、同一命令反复且输出不变、进度文件不涨（`scripts/agents/agent-watch.py` 的 `INFO_ALERT_NAMES`）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:30`） |  |
| M183 | 「卡住的（工具调用过长、无动静、结束本轮不会醒）才处置」（`.claude/main-agent.md:84`） | 「卡住的（工具调用过长、无动静、结束本轮不会醒）才处置」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:34`） |  |
| M184 | 「发消息让它改用写死的 pid」（`.claude/main-agent.md:84`） | 「发消息让它改用写死的 pid」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:35`） |  |
| M185 | 「python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>」（`.claude/main-agent.md:84`） | 「python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:35`） |  |
| M186 | 「核过没有进程在用（工作目录、命令行、`CARGO_TARGET_DIR` 指到它）再删」（`.claude/main-agent.md:84`） | 「核过没有进程在用（工作目录、命令行、`CARGO_TARGET_DIR` 指到它）由主 agent 删」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:35`） |  |
| M187 | 「以 `du` 为准、只删没有进程在用的，一时删不了、确认接着跑的用 `--ack 类:根分区余量过低`」（`.claude/main-agent.md:84`） | 「以 `du` 为准、只删没有进程在用的，一时删不了、确认接着跑的用 `--ack 类:根分区余量过低`」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:35`） |  |
| M188 | 「`task wait` 末行 `type=card-fault`、退 237」（`.claude/main-agent.md:86`） | 「`task wait` 末行 `type=card-fault`、退 237」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:39`） |  |
| M189 | 「请用户重置那张卡，不等、不重试」（`.claude/main-agent.md:86`） | 「请用户重置那张卡，不等、不重试」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:39`） |  |
| M190 | 「只报不删，主 agent 核过没人在用再删」（`.claude/main-agent.md:86`） | 「只报不删，主 agent 核过没人在用再删」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:37`） |  |
| M191 | 「不按无动静报」（`.claude/main-agent.md:86`） | 「不按无动静报」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:37`） |  |
| M192 | 「不发整点例行询问（`watch.conf` 的 `ask-every-minutes=0`）」（`.claude/main-agent.md:86`） | 「不发整点例行询问（`watch.conf` 的 `ask-every-minutes=0`）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:37`） |  |
| M193 | 「读它草稿目录的 `progress.md` 或它的会话记录，不发消息叫醒它」（`.claude/main-agent.md:86`） | 「读它草稿目录的 `progress.md` 或它的会话记录，不发消息叫醒它」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:37`） |  |
| M194 | 「处置后用 `python3 task-server/control check` 回读，退 0 才算服务健康（缺一样退 78，出路写在输出里）」（`.claude/main-agent.md:88`） | 「处置后用 `python3 task-server/control check` 回读，退 0 才算服务健康（缺一样退 78，出路写在输出里）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:39`） |  |
| M195 | 「写明改了哪一条、它手上哪一步要停掉或改做」（`.claude/main-agent.md:90`） | 「写明改了哪一条、它手上哪一步要停掉或改做」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:74`） |  |
| M196 | 「只是慢就再起一只看门狗接着盯，确认只是慢的带 `--ack` 再起」（`.claude/main-agent.md:94`） | 「只是慢就再起一只看门狗接着盯，确认只是慢的带 `--ack` 再起」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:37`） |  |
| M197 | 「`task wait` 以 `needs-attention` 退出」（`.claude/main-agent.md:94`） | 「`task wait` 以 `needs-attention` 退出」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:41`） |  |
| M198 | 「`cancel`、`clean` 只对自己交的号做」（`.claude/main-agent.md:94`） | 「`cancel`、`clean` 只对自己交的号做」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:41`） |  |
| M199 | 「agent 的结论句一律当线索：以仓里的产物为准，现查过再写进 kb」（`.claude/main-agent.md:98`） | 「agent 的结论句一律当线索：以仓里的产物为准，现查过再写进 kb」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:78`） |  |
| M200 | 「续做闸 `.claude/hooks/continuation-guard.sh` 拒绝」（`.claude/main-agent.md:100`） | 「续做闸 `.claude/hooks/continuation-guard.sh` 拒绝」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:78`） |  |
| M201 | 「python3 scripts/agents/agent-handover.py」（`.claude/main-agent.md:100`） | 「python3 scripts/agents/agent-handover.py」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:47`） |  |
| M202 | 「This agent has not reported yet: it is waiting on its own background work」（`.claude/main-agent.md:100`） | 「This agent has not reported yet: it is waiting on its own background work」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:51`） |  |
| M203 | 「看它最后在等什么，发消息让它接着做或交回，或者停掉」（`.claude/main-agent.md:100`） | 「看它最后在等什么，发消息让它接着做或交回，或者停掉」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:52`） |  |
| M204 | 「没删成的 cancel 退 5、照它的出路处置」（`.claude/main-agent.md:100`） | 「没删成的 cancel 退 5，照它的出路处置」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:54`） |  |
| M205 | 「有了结局的逐个 `task-server/task clean <号>`」（`.claude/main-agent.md:100`） | 「有了结局的逐个 `task-server/task clean <号>`」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:54`） |  |
| M206 | 「派发提示里不写线程、内存上限、台数与卡、分片与续跑」（`.claude/main-agent.md:104`） | 「不写线程、内存上限、台数与卡、分片与续跑」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:58`） |  |
| M207 | 「值是这一轮别的会话在改、这一件不碰的文件」（`.claude/main-agent.md:106`） | 「值是这一轮别的会话在改、这一件不碰的文件」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:60`） |  |
| M208 | 「点名的报告要在。缺一样派发闸拒」（`.claude/main-agent.md:106`） | 「缺一样派发闸拒」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:82`） |  |
| M209 | 「文件名里不带 report」（`.claude/main-agent.md:108`） | 「文件名里不带 report」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:62`） |  |
| M210 | 「事实、路径、行号、数与判据一个不少」（`.claude/main-agent.md:110`） | 「事实、路径、行号、数与判据一个不少」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:82`） |  |
| M211 | 「写之前先现读 X」（`.claude/main-agent.md:110`） | 「写之前先现读那一处」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:64`） |  |
| M212 | 「正文引到 30 份以上文件、清单要人挑的轮才派 `three-way-materials`」（`.claude/main-agent.md:116`） | 「正文引到 30 份以上文件、清单要人挑的轮才派 `three-way-materials`」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:90`） |  |
| M213 | 「清单默认全抄、索引表与历史节不抄」（`.claude/main-agent.md:116`） | 「清单默认全抄、索引表与历史节不抄」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:68`） |  |
| M214 | 「第一轮派正推，辩方只在已有一轮判决可复核时派」（`.claude/main-agent.md:116`） | 「已有一轮判决可复核时才派 `three-way-defense`」（`research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:9`） |  |
| M215 | 「第三轮为最终轮，论证前两轮的结论和判定过程」（`.claude/main-agent.md:116`） | 「第三轮为最终轮，论证前两轮的结论和判定过程」（`research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:43`） | 用户定案第 2 条 |
| M216 | 「不派核查员」（`.claude/main-agent.md:116`） | 「不派核查员」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:90`） |  |
| M217 | 「条款已定、验收标准写得清、改动有界」（`.claude/main-agent.md:117`） | 「条款已定、验收标准写得清、改动有界」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:91`） |  |
| M218 | 「主 agent 跑提交时的六道门禁：`crash-point-replay`、QEMU、herd7、crates 变异表、Verus 形式化验证、广谱变异提交门禁」（`.claude/main-agent.md:117`） | 「提交时照」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:107`） | 六道的名字在表里「暂存之后」那一行 |
| M219 | 「钉死的常量全过才往下」（`.claude/main-agent.md:118`） | 「钉死的常量全过才往下」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:92`） |  |
| M220 | 「e171_three_oracle_collusion_after_the_oracle_fixes stream-constants --streams 1,2,3」（`.claude/main-agent.md:118`） | 「e171_three_oracle_collusion_after_the_oracle_fixes stream-constants --streams 1,2,3」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:70`） |  |
| M221 | 「主 agent 自己写、直接和用户对话」（`.claude/main-agent.md:119`） | 「主 agent 自己写、直接和用户对话」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:93`） |  |
| M222 | 「`mutation-triage` 读输出分诊」（`.claude/main-agent.md:120`） | 「`mutation-triage` 读输出分诊」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:94`） |  |
| M223 | 「派发闸 ⑪」（`.claude/main-agent.md:120`） | 「派发闸 ⑪」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:72`） |  |
| M224 | 「只用过、没留改动的也写」（`.claude/main-agent.md:121`） | 「只用过、没留改动的也写」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:75`） |  |
| M225 | 「task-server/task submit -- python3 scripts/kb/batch-target.py」（`.claude/main-agent.md:121`） | 「task-server/task submit -- python3 scripts/kb/batch-target.py」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:75`） |  |
| M226 | 「交回的事实表过了 `scripts/kb/stale-candidates.py --check-facts`」（`.claude/main-agent.md:121`） | 「写完跑 `stale-candidates.py --check-facts 事实表 --base 基准 [--target 结束]`，退出码不是 0 就补完再跑」（`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:44`） | 入口与技能里没有这半句，只在回扫员定义里 |
| M227 | 「每一种判定各抽不少于 5 条（不足 5 条的全看）」（`.claude/main-agent.md:121`） | 「每一种判定各抽不少于 5 条（不足 5 条的全看）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:77`） |  |
| M228 | 「两次都干净才收」（`.claude/main-agent.md:121`） | 「两次都干净才收」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:77`） |  |
| M229 | 「不交写回员」（`.claude/main-agent.md:121`） | 「不交写回员」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:78`） |  |
| M230 | 「task-server/task submit -- python3 scripts/gate/stage-mine.py」（`.claude/main-agent.md:122`） | 「task-server/task submit -- python3 scripts/gate/stage-mine.py」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:96`） |  |
| M231 | 「带 `--full-gate` 的提交弹窗要用户确认，对话不自己决定」（`.claude/main-agent.md:122`） | 「这一条弹窗要用户确认、对话不自己决定」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:20`） |  |
| M232 | 「crates-mutation-replay 与 `crash-point-replay` 串行」（`.claude/main-agent.md:122`） | 「crates-mutation-replay 等 `crash-point-replay` 跑完再起」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:22`） |  |
| M233 | 「整轮全绿才前移 `refs/sop/staged-green`」（`.claude/main-agent.md:122`） | 「整轮全绿才前移 `refs/sop/staged-green`」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/staged-gate.md:17`） |  |
| M234 | 「有一条没结论或没红就判红、不许提交」（`.claude/main-agent.md:122`） | 「此刻源码上每一条变异都有单元键相同的结论、没有一条没红」（`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:149`） |  |
| M235 | 「整轮里不另外直接调这六道」（`.claude/main-agent.md:122`） | 「不另外直接调」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:96`） |  |
| M236 | 「规格检查已判：写回员自己起草并跑规格检查」（`.claude/main-agent.md:124`） | 「规格检查已判：写回员自己起草并跑规格检查」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:82`） |  |
| M237 | 「同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行」（`.claude/main-agent.md:124`） | 「同一份规格还要带上那个实验页 `### 影响的决策` 表里对应那几行的改动」（`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:25`） |  |
| M238 | 「判一个实验撑不撑一条分项归主 agent」（`.claude/main-agent.md:124`） | 「判一个实验撑不撑一条分项归主 agent」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:84`） |  |
| M239 | 「`crates/singlefs-checker/invariants/*.md` 与 `crates/singlefs-checker/invariants/**/*.md`」（`.claude/main-agent.md:124`） | 「`crates/singlefs-checker/invariants/*.md` 与 `crates/singlefs-checker/invariants/**/*.md`」（`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:42`） |  |
| M240 | 「红了或开关没红在点名那一格的新派工具员接着改」（`.claude/main-agent.md:125`） | 「红了或开关没红在点名那一格的新派工具员接着改」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:99`） |  |
| M241 | 「直到定位到文件与行（不先改方向和结论）」（`.claude/main-agent.md:126`） | 「直到定位到文件与行（不先改方向和结论）」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:100`） |  |
| M242 | 「主 agent 删掉登记里待删的问法（有的话）」（`.claude/main-agent.md:127`） | 「主 agent 删掉登记里待删的问法（有的话）」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:88`） |  |
| M243 | 「先派一个执行员把装置、单测与变异表写完，再一段派一个执行员并行交各段的跑法」（`.claude/main-agent.md:127`） | 「先派一个执行员把装置、单测与变异表写完，再一段派一个执行员并行交各段的跑法」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:89`） |  |
| M244 | 「一行开着的都不剩就停、交岔路表」（`.claude/main-agent.md:127`） | 「一行开着的都不剩就停、交岔路表」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:101`） |  |
| M245 | 「续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句」（`.claude/main-agent.md:127`） | 「续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:91`） |  |
| M246 | 「只是跑一条脚本的，主 agent 自己起」（`.claude/main-agent.md:129`） | 「只是跑一条脚本的，主 agent 自己起」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:103`） |  |

### 合稿树里找不到的句子（12 句，都是说明，不带做法）

找过的范围：合稿树 `CLAUDE-merged.md` 与 `dot-claude/` 整棵，逐句 `grep -rF -l` 数命中的文件数，12 句都是 0（量的）。每句的做法那一半另有落点（表里的号），丢的只是解释。

| 原件 | 原话 | 性质 | 做法那一半落在 |
|---|---|---|---|
| `./CLAUDE.md:12` | 出口、判阻塞、收拢、再判 | 介绍入口文件里有什么 | C07 |
| `./CLAUDE.md:16` | 同一会话里派的继承会话开始时那一份 | 为什么要新开会话 | C10 |
| `./CLAUDE.md:41` | 本项目只接其中一份，不需要知道别的 | 说明 | C28 |
| `./CLAUDE.md:43` | 那是在提醒 | 门禁红了是什么意思 | C33 |
| `./CLAUDE.md:120` | 未定项还开着就写下去的实现多半要返工 | 理由 | C84 |
| `./CLAUDE.md:122` | 含那两条流 | 括注 | C88 |
| `./CLAUDE.md:122` | 2026-10-11 的全量都 0 红 | 实测句，用户定案第 11 条点名删 | C86 |
| `.claude/main-agent.md:13` | 没有这种名单 | 指脚本文件头的括注 | M20 |
| `.claude/main-agent.md:15` | 格式锁定之后才另定 | 这条禁令到什么时候为止的括注 | M22 |
| `.claude/main-agent.md:47` | 带上门禁表与仓里的脚本，从它建的实现员副本才起得了 | 段树为什么带那几样 | M100 |
| `.claude/main-agent.md:57` | 汇总行之后没有第 2 步与合进主工作区那两段 | 退 75 那一趟脚本打什么 | M121 |
| `.claude/main-agent.md:59` | 没有合到一半 | 同一句的第二种说法 | M143 |

这 12 句之外，逐句过的时候没有遇到合稿树里找不到落点的判据、阈值、次序、路径、命令或退出码。

落点不在这一组四份文件里、而在脚本或钩子文件头的有 5 条（M73、M74、M84、M92、M127），合稿只写「照它文件头」；五处都在文件头里现找到了原话（表里的行号）。落在回扫员定义里的 1 条（M226）、写回员定义里的 2 条（M237、M239）、共用约束里的 8 条（M09、M27 到 M30、M32、M66、M76）、别的常驻规则与技能里的在表里逐条标了文件。

## 问题二：五格

这一节「成立」指这一格问的那件事站得住、没有打中；「不成立」指有打中，逐处列。

### 一 丢失：不成立（1 处，轻）

1. 冻结层归属登记表那一行的三条判据换成了指针，指针点名的地方没有其中两项。原件：「每层、每棵树、每个单元类都有行」（`./CLAUDE.md:90`）。合稿：「写在 `.claude/gate.d/doc-decisions.sh` 文件头格名表那一格」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:84`）。文件头格名表那一格今天写的是：「每层每个结构都有行」（`.claude/gate.d/doc-decisions.sh:22`），没有「每棵树」「每个单元类」；这两项在同一份脚本里那一格函数上方的注释：「每棵树在 key 编码层与索引节点内部布局层各至少一行」（`.claude/gate.d/doc-decisions.sh:1851`）。这是项目事实表里描述一道门禁的句子，不是主 agent 照做的步骤，所以记轻。改法二选一：指针改指那一格函数上方的注释，或者把原件那一句留着。
   - 什么现象会推翻它：`.claude/gate.d/doc-decisions.sh` 文件头格名表那一格里逐字有「每棵树」与「单元类」。现查 `grep -n '每棵树\|单元类' .claude/gate.d/doc-decisions.sh` 的命中从第 1851 行起，文件头没有。

其余 336 条都有落点（「问题一」两张表）。推翻条件：表里任何一行的落点句与原件句并排放，原件有一个判据、阈值、次序、路径、命令或退出码是落点句与它点名的文件头里都没有的。

### 二 二义：不成立（1 处）；另 2 处观察，不算打中

1. 提交之外用户要求跑不指定范围的那一支，合稿树里四处说法不齐，读得出「不再另弹窗」与「照样先弹窗」两种。
   - 入口改成了：「提交之外，用户要求的照跑；主 agent 自己判要跑的，先弹窗问用户，同意了才跑」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:26`）。这是矛盾总表「入口-1」登的那一处，标了改了判据。
   - 常驻规则那张表没跟着改：「提交之外要跑先弹窗问用户」（`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:32`）。共用约束同一句：「提交之外要跑先弹窗问用户」（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:59`）。
   - 技能里两句话并在一句里：「不指定范围的，主 agent 只在提交时或用户要求时交，提交之外要跑先弹窗」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:12`）。
   - 两种读法：读法甲，照入口第 3 条，用户开口要跑的那一趟直接交；读法乙，照 `verification.md` 那张表与共用约束，凡是提交之外都先弹窗、不分谁提的。两种读法做法不同（多不多一次弹窗）。原件两句本来就不齐：「不指定范围的只在提交时或用户要求时由主 agent 经服务交」（`.claude/main-agent.md:18`）与「提交之外任务确实要跑，先弹窗问用户，同意了才跑」（`.claude/main-agent.md:26`）；合稿只在入口一处收齐，另三处还是原样。
   - 改法：`verification.md` 那一格与共用约束那一句写成与入口第 3 条同一句，或只留指针。
   - 什么现象会推翻它：`verification.md` 那一格或共用约束那一句里写着「用户要求的照跑」之类的例外。`grep -n '提交之外'` 这四份的命中里没有。
2. 观察，不算打中：「主 agent 可以自己写的只有两类」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:9`）比原件多了「只有」。按字面读，主 agent 不许写别的；同一份文件的派发表又写着：「主 agent 写 `research/prompts/<阶段>-sync.md`」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:95`）与「主 agent 写岔路单」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:101`）。同一句的前半句列了「写三方正文与判决」，派发表逐行写了谁写，照表做不会做错，所以不算打中；要消掉这点歧义，把「只有两类」写成「本该派出去而主 agent 可以自己写的只有两类」。
3. 观察，不算打中：「整批重交」的定义写的是：「第 1 步连同修补的补丁目录整批重交」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:14`），而第 3 步核的那几条退非 0 的那一支没有修补：「都退 0 才往下；退非 0 不合，整批重交」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:37`）。没有修补时「连同修补的补丁目录」是空的，做法只有一种（原补丁目录原样重交、第 2 步在新快照上重验、回到第 3 步），与原件那一句一致，所以不算打中。

指针唯一性（`pointer_check.py`，四份被判合稿文件里「文件」后面紧跟「节」的指针共 62 处，量的）：标题逐字 48、标题开头且唯一 5、加粗条目名 7、只在正文里 2，找不到 0、不唯一 0。原样输出：

```
CLAUDE-merged.md:83	标题开头（唯一）	.claude/rules/path-moves.md「改一个全仓术语」	合稿树
CLAUDE-merged.md:116	只在正文里	crates/singlefs-checker/invariants/README.md「## 状态」	仓里
dot-claude/main-agent.md:15	标题开头（唯一）	.claude/rules/verification.md「判定库只有一份」	合稿树
dot-claude/main-agent.md:15	标题开头（唯一）	.claude/rules/verification.md「所有任务都要多机可用」	合稿树
dot-claude/main-agent.md:19	加粗条目名	.claude/agent-common.md「子 agent 不交任务」	合稿树
dot-claude/main-agent.md:20	加粗条目名	.claude/agent-common.md「日常开发的测试不跑整个包」	合稿树
dot-claude/main-agent.md:48	只在正文里	.claude/rules/implementation-workflow.md「实现员交回前」	合稿树
dot-claude/main-agent.md:99	标题开头（唯一）	.claude/agents/tooling-writer.md「输入」	合稿树
dot-claude/skills/merge-verify/SKILL.md:22	加粗条目名	.claude/agent-common.md「日常开发的测试不跑整个包」	合稿树
dot-claude/skills/merge-verify/SKILL.md:26	加粗条目名	.claude/agent-common.md「子 agent 不交任务」	合稿树
dot-claude/skills/merge-verify/SKILL.md:31	加粗条目名	.claude/agent-common.md「命名检查交主 agent 跑」	合稿树
dot-claude/skills/dispatch/SKILL.md:18	标题开头（唯一）	.claude/rules/verification.md「判定库只有一份」	合稿树
dot-claude/skills/dispatch/SKILL.md:41	加粗条目名	.claude/agent-common.md「长活交回主 agent」	合稿树
dot-claude/skills/dispatch/SKILL.md:51	加粗条目名	.claude/agent-common.md「子 agent 不交任务」	合稿树
合计 [('加粗条目名', 7), ('只在正文里', 2), ('标题开头（唯一）', 5), ('标题逐字', 48)]
```

「只在正文里」的两处：一处是表格里那一行的行名（「实现员交回前」），一处的落点文件原件就是这样引的，都唯一。写成「一节」加「那一条」两层的 6 处指针脚本不认，手核过：共用约束「不做」一节的「崩溃点测试准确率和正确率优先」「本机常有别的会话同仓干活」「执行前拒绝的写法」「门禁只有一个，跑法两种」，「写」一节的「里程碑 17 收尾之前不兼容任何旧东西」「进仓的文字只写本项目的东西」，在合稿的共用约束里各恰好一条（第 54、52、73、59、40、44 行，`grep -n` 现查）。派发表里写「细则在技能同名条目」的 6 行，在技能 dispatch「什么时候派哪个 agent」一节各有一条以行名起头的条目，6 对 6。

推翻条件：四份被判文件里有一处指针，点名的节在落点文件里找不到，或同名的有两处。

### 三 常驻够不够：成立

- 入口文件开头有两句总的入口：「会话里第一次派发之前先整份读 `.claude/skills/dispatch/SKILL.md`」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:5`），同一行另一句管验证段之前读 merge-verify。
- 逐条的入口 14 处（`grep -n -o` 现取，量的）：「一轮怎么开、怎么收」第 1、4、8、11、15 条各一处指 dispatch；验证段第 1、2、3、5 步与拒算第 1、2 条各一处指 merge-verify；「派出去之后」「交回怎么读」「派发提示怎么写」各一处指 dispatch。派发表另有一句总的入口（第 86 行）管写「细则在技能同名条目」的 6 行。
- 反过来对：两份技能里有细则的块，dispatch 是 5 条、3 节、派发表 6 行，merge-verify 是验证段第 1、2、3、5 步、拒算第 1、2 条与开头「整批重交」的定义；每一块在入口文件里都有上面的一处入口。技能里写「没有细则」的三处（验证段第 4、6 步、拒算第 3 条）入口那一句就是全文，两边并排核过。
- 不读技能照入口骨架做会不会做错：逐步看过，骨架句都只说到「做什么、绿了红了去哪」，命令、各支结局与出路都在入口句点名的那一节里，没有哪一步骨架句给了一半命令、又没叫人先读。

观察两条，不算打中：
- 「交回的事实表过了 `scripts/kb/stale-candidates.py --check-facts`」这半句（M226）在入口与技能里都没有了，只在回扫员定义里（回扫员自己跑、把输出末行交回）。主 agent 收事实表时看的是那一行输出，做法没变。
- 「崩溃点测试准确率和正确率优先」的正文与跑整包的确认办法，入口只留「照 `.claude/agent-common.md`」加节名与条名，落点是共用约束不是技能；指针写了文件与条名，那两条在共用约束里都在（M09、M27 到 M30）。

推翻条件：找得出一步，照入口文件那一句做会做错，而那一句、那一节的末尾与文件开头都没有叫人先读技能的话。

### 四 子 agent 读得到：成立（范围：指到这一组文件的指针）

- 合稿的 14 份定义与共用约束里，点到两份新技能 merge-verify、dispatch 的 0 份（`grep -l 'skills/merge-verify\|skills/dispatch'` 数的，量的）。
- 点到入口文件的只有共用约束 3 行与工具员定义 2 行。共用约束那 3 行（第 51、64、109 行）说的都是主 agent 接下来做什么（红了发给它的会话、合入后验证只由主 agent 跑、命名检查在验证段第 2 步由主 agent 交），不要子 agent 去读入口文件才做得了自己的事；工具员那 2 行是它的写范围与输入里列的路径。
- 实现员定义里说到「验证段第 2 步」的 3 行，写的是主 agent 在那一步交哪一条，那一条的整条命令就抄在定义里。

推翻条件：定义或共用约束里有一句要子 agent 自己照着做的话，做法只写在入口文件或这两份技能里。别的组的文件（规则、参考件、别的技能）被定义点没点名，这条腿没核。

### 五 悄悄改了做法：成立（没找到没登的）

逐条过落点表时，把做法与原件不同的落点逐个对矛盾总表与用户定案：

| 落点 | 改了什么 | 登在哪 |
|---|---|---|
| M24、M50 | 提交之外用户要求的不指定范围照跑 | 矛盾总表 入口-1 |
| M52、M99 | 只派一个实现员也在副本里改、交补丁 | 用户定案第 1 条（总表 用户定-1） |
| M214、M215 | 开不开第二、三轮收到规则一处 | 用户定案第 2 条（总表 用户定-2） |
| C19、M61、M56 | 指针改指现行账本、第 2 条、技能同名节 | 入口-9、入口-6；M56 只是换了落点文件 |
| 入口第 4 条治理文件的名单加了服务本体目录 | `prune-server/` 与客户端两处都写 | 合稿-2 |
| M115、M179、M171 | 措辞 | 入口-2、入口-5、入口-11 |
| M37、M75 | 第一遍稿丢的限定放回 | 入口第二遍-1、入口第二遍-2 |

没有找到做法变了而两处都没登的落点。一条观察：参考件登进 read-once（「参考件 `.claude/rules/verification-reference.md` 不 `@` 常驻」，`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:66`，下一行是登记行），而 doc-lint 那种登记的本意是开会话读一次、参考件是到那一步才读。这是新文件的登记办法，不是原件做法的改动；写手自己在合稿说明「拿不准的」里列了（「rules-manifest 格只认 `@` 与 read-once 两种登记」，`research/prompts/rules-compress-r1-merged/merge-result.md:163`），矛盾总表里没有这一条，提醒主 agent 判决时别漏。

用户定案十六条里落在这一组四份文件上的三条，都落对了：
- 第 1 条：「实现员一律在草稿目录的副本里改、交补丁，只派一个也这样」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:47`），技能里同一句（M52）。
- 第 2 条：入口写「开不开第二、三轮照同一份」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:90`），指到的那一段写全了：「第一轮两条腿结论一致、没打中，就停，不开第二轮」（`research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:43`）。
- 第 11 条：`2026-10-11` 在合稿树 `CLAUDE-merged.md` 与 `dot-claude/` 里 0 命中（量的）。

推翻条件：落点表里有一行，原件句与落点句照着做结果不同，而矛盾总表与调度记录十六条里都找不到它。

## 要主 agent 跑的命令

无。这条腿没有写模型，核的都是只读命令能核的；`scripts/agents/cite-check.py` 是可直接跑的文本工具，已跑。

## 核过的条数与没核到的范围

- 核过并确认有落点：337 条（原件 `CLAUDE.md` 91 条、`.claude/main-agent.md` 246 条），两边行号都是脚本按整串现找的；其中 1 条的落点指针不准（「一 丢失」第 1 处）。另 12 句说明在合稿树里没有，单列了。
- 条数是这条腿切的粒度：一句里几个判据的，多数各取一段，没有保证每个逗号分句都单列一行。没单列的分句是读两边原文时并排看的，没留下逐句的行号。
- 没核到：被并掉的五份常驻规则并进去之后内容丢没丢；合稿的共用约束、14 份定义、别的规则与技能自身（只在它们是这一组的落点时读了那一句）；用户定案第 3 到 10、12 条（落点不在这一组）与第 13 到 16 条（改脚本的活，没有文字落点）；脚本与钩子的文件头只核了表里点到的那几句在不在，没核它们与代码一不一致。
- 写手的对照表与 `merge-result.md` 没当证据用；矛盾总表只用来对「登没登」。禁读清单里的文件没读。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 一 丢失 | 不成立（1 处，轻） | 冻结层归属登记表那一行的指针指到门禁文件头格名表，那一格里没有「每棵树」「每个单元类」两项；其余 336 条有落点 |
| 二 二义 | 不成立（1 处） | 提交之外用户要求跑不指定范围的，入口写照跑，`verification.md` 那张表、共用约束与一份技能还写先弹窗；指针 62 处都唯一 |
| 三 常驻够不够 | 成立 | 技能里每一块细则在入口文件都有「先读」的入口，写「没有细则」的三处入口句就是全文 |
| 四 子 agent 读得到 | 成立 | 定义与共用约束没有把子 agent 指到两份新技能；指到入口文件的几行说的是主 agent 做什么 |
| 五 悄悄改了做法 | 成立 | 做法有变的落点都在矛盾总表或用户定案里；参考件登 read-once 那一处只在合稿说明里、总表没有 |

## 没做什么

- 没判别的腿的格，没替主 agent 采纳或出判决。
- 没往统一任务服务交任务；没有要复跑的数（报告里的数都是只读命令当场数的，标了「量的」）。
- 「没打中」的三格（三、四、五）只有这一条腿的一次样本，照规则不能单独撑结论。
- 没核到的范围见上一节。
