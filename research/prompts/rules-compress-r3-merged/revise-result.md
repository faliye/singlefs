# 压缩 2f 修稿第三版结果

交到本目录：`third/`（整棵第三版合稿树，含 `contradictions.md`、`references.md`）、`dispositions-r2.md`（逐条去向表）、`diff-from-r2.patch`（`diff -ruN base third` 的原样输出，`base/` 是冻结的第二版树的拷贝，只少第二版的四个交付物 `merge-result.md`、`revise-result.md`、`dispositions.md`、`diff-from-r1.patch`，它们不是树的内容；交回前重跑，`patch -p1` 打到 `base/` 的拷贝上与 `third/` 逐文件相同，量的）。仓里一个文件没改，没跑项目代码，没交任务；只跑了只读的 python 文本脚本（`tools/`），脚本里改树都是同目录临时文件加 `os.replace`。

## ① 十份报告的条数与四个数

表里一行是一条（两条腿或两题说同一件事并成一行，每条腿都算一次）；四个数由 `tools/dispo_render.py` 数出，与 `dispositions-r2.md` 的表逐行对得上。

共 69 条（表里的行），成立 54、不成立 7、部分 8。

| 题·腿 | 总数 | 成立 | 不成立 | 部分 |
|---|---|---|---|---|
| 入口·攻 | 10 | 10 | 0 | 0 |
| 入口·辩 | 8 | 5 | 1 | 2 |
| 验证·攻 | 14 | 14 | 0 | 0 |
| 验证·辩 | 8 | 6 | 1 | 1 |
| 常驻·攻 | 7 | 7 | 0 | 0 |
| 常驻·辩 | 6 | 4 | 2 | 0 |
| 定义·攻 | 10 | 9 | 0 | 1 |
| 定义·辩 | 7 | 4 | 1 | 2 |
| 矛盾·攻 | 13 | 11 | 1 | 1 |
| 矛盾·辩 | 11 | 7 | 2 | 2 |

按报告累计（一行列几条腿算几次）94、成立 77、不成立 8、部分 9。辩方腿对第一轮判决逐条复核判「站得住」、第二版修补「够得着」的各条已落在第二版，不单列行；表里的辩方行只收它们指出的残留、新问题、顺带看到的与核查缺口。

## ② 用户定案第 23 到 26 条落在哪（`third/` 的 文件:行）

- 23：代码轮与治理文件轮的收法新写一节 `dot-claude/rules/implementation-workflow.md:25`（第一轮停法照三方规则、现查坐实就改再攻一轮至多三轮 :29 到 :30、现查推翻的不改不开轮 :31、第三轮又打中的改了收写进交用户表不开第四轮 :32、开了第二轮的在第二轮派辩方腿 :30）；第 2 步那一格 `:10` 指过去，「改 agent 定义…同规矩」「治理文件同规矩」两处 `:19`、`:21` 指过去。三方规则「多轮」一节 `dot-claude/rules/three-way-inference.md:43`：「设计推论轮」写出定义（设计判断、取舍、推翻或确立决策的那一类）并逐段写明管哪类轮：写了「设计推论轮」的两段（`:45` 开不开下一轮、`:47` 第三轮之后停）只管它，没写的段（两边互调、岔路代价数、用户叫停、核查与判决、判决起笔）管三类轮；开了第二轮派辩方腿的句子 `:45`；入口派发表「要推论」那一行 `dot-claude/main-agent.md:90` 分开指两处。
- 24：句子只住 `dot-claude/rules/verification.md:38`（带 `--full-gate` 或 `--whole-package-confirmed` 的命令，钩子 `run-scope-guard.sh` 弹的那一次就是确认、主 agent 不另问；不带这两个标记、钩子不弹的，主 agent 自己弹窗问一次；钩子弹哪几种照它文件头「放宽检查的提交标识」那一段）。指过去或写同一个意思的：`dot-claude/main-agent.md:26`、`dot-claude/agent-common.md:59`、`dot-claude/rules/session-wrapup.md:37`、`dot-claude/skills/crash-test/SKILL.md:12`、`dot-claude/skills/gate/SKILL.md:10`、`dot-claude/skills/gate/staged-gate.md:17`（带 `--full-gate` 那一条）与 `:18`（跑全量工作区的 `gate.sh` 不带标记、钩子不弹，主 agent 先自己弹窗）、`dot-claude/skills/crash-test/commit-stages.md:21`、`dot-claude/skills/merge-verify/SKILL.md:22`（退 3 的整包确认）。
- 25：三处原话各留：`dot-claude/main-agent.md:9`（原件那一句的意思，没有「只有」，第二版新加的「别的文件的简单任务…自己直接做」删掉）、`CLAUDE-merged.md:21`（「修它们本身就是这一轮的出口时，直接改，不先问」）、`dot-claude/agent-common.md:11`（「自己直接做，不派」）。
- 26：`dot-claude/rules/three-way-inference.md:72`（一条腿只抽一次样那一节：一轮的正推或辩方与攻方各是一次独立抽样，两条腿都没打中记没打中、这一题停；只有一条腿覆盖到的范围要拿它的「没打中」当依据时再派同立场腿）与 `:45`（开不开下一轮：两条腿都没打中就停）两处写同一件事，不相抵；代码轮与治理文件轮的停法在 `implementation-workflow.md:29` 指到三方规则那一段。

其余照主 agent 处置：「规则怎么读」改回原件原话 `dot-claude/agent-common.md:17`；上游指针 45 处括号写法改回 `文件`「节」（节名逐字取上游今天的标题行，全部核得到），`code-discipline-by-review.md` 8 处「上游「节」」收成同一写法，「射程与例外」那一处留 `writing-discipline-by-review.md` 里「文件名加项目一侧判据句」的写法；全树没有 文件:行 形的脚本指针；调查员验证命令写在改的那棵树上（`dot-claude/agents/investigator.md:28`、`:33`，`dot-claude/agents/implementation-writer.md:31`、`:46`，`dot-claude/agent-common.md:109`）；写段第 6 条补调查员例外（`dot-claude/main-agent.md:51`）；循环逐个发照钩子文件头 ⑥（`dot-claude/agents/tooling-writer.md:33`、`dot-claude/rules/command-safety-by-review.md:9`、`dot-claude/agent-common.md:67`）；技能重读入口补到「派出去之后」「交回怎么读」（`dot-claude/main-agent.md:5`、`:74`、`:78`）。参考件对着门禁文件头改对的句子（钉全、broad-mutation、formal-verification 七格、领卡、挂特性的文件跑到 0 条、qemu 与 lkmm、判器调用点、声明模块、测试线程）在去向表「验证」行与总表验证-10 到验证-19。

## ③ 没采纳的（或只采一半的）

- 辩·入口：`CLAUDE.md` 的读一次清单压缩上下文之后同样会掉（原件 `CLAUDE.md` 就这样写、不是压缩带来的，主 agent 的处置只要求补两节入口）；核查方式的缺口 1 到 4（主 agent 核查的做法，不是树里的句子）。
- 辩·入口 第 7 条登记位：引用改法清单里工具员写范围的行号改成 33 到 45；登记位（里程碑收口表、欠账表、决策正文）树里不写，交主 agent。
- 辩·验证：第四节核查缺口（同上）；不采纳的几行理由站得住（没有要改的句子）。
- 辩·常驻：crash-test 技能 `SKILL.md:35`「带什么前缀」读成范围声明说得通，原件同形；不采纳的几行与核查缺口（数由脚本数，核查归主 agent）。
- 攻·定义 材料不全（上一版交付物的毛病；这一版补丁整棵树出）。
- 辩·定义：第四节第 95 行（五段钩子头逐句对了被点名的四段，别的段仍只有指针级核对）、第 96 行（脚本一侧）、第五节第 1、2 点（核查做法）。
- 攻·矛盾：共用-10 条里钩子文件头仍写 rsync 副本填仓根（脚本一侧，登在 `references.md`「四」）；辩·矛盾：`quote-rust-items.py` 钩子名单（用户定案第 10 条排在脚本那一批）、验证-5 比服务拒绝名单宽（原件同样）、第四节核查缺口。
- 判「改得对」「没动做法」的点名条与抽查条：没有要改的句子。

## ④ 拿不准、要主 agent 或用户定的

1. 用户定案第 20 条「循环逐个发…一律拒」的射程：钩子对 Python 放行循环里每一轮 `os.killpg` 停自己用新会话起的那一组（它的 Python 判法那段注释写明），shell 循环里一律拒（判法 `process_target_findings` 里循环那一支不看是不是自己起的组）。规则、共用约束、工具员第 5 步照钩子写成「shell 循环里一律拒、Python 里放行」；用户原话是否连 Python 那一种也拒（那钩子要改）原话判不了。
2. `.claude/hooks/run-scope-guard.sh` 文件头写主 agent 的命令带 `--full-gate`、`--whole-package-confirmed` 的 `task-server/task submit` 弹窗（没跑，读文件头）；不带这两个标记的命令（`gate.sh --staged` 的 `gate-staged.sh` 不带 `--full-gate` 交、跑全量工作区的 `gate.sh`）钩子不弹，第三版写成「主 agent 自己弹窗问一次」。服务端对提交之外带 `--full-gate` 的命令收不收，没核（`full_gate_refusal`）。
3. `.claude/gate.d/crash-point-replay.sh` 文件头对挂特性的文件跑到 0 条写得不一致：格名表一行写「不为 0」，feature-gated-tests 格那一段写 0 条与 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、不判红。参考件照后一段改；门禁真实行为没跑。
4. 工具员第 5 步仍点名 `proc.py stop <pid>` 作为许的停法（原件写法），而共用约束「停任务与停进程」写 `proc.py` 只给主 agent：脚本在服务里起时它读不到会话身份。要不要从工具员第 5 步删掉，没定。
5. 「规则怎么读」改回「一律整份读」之后，`clerk` 定义「规则只读三样」靠「与定义冲突时以定义为准」解。
6. 用户定案第 9 条的欠账号要主 agent 登后补进 `references.md` 与参考件；`contradictions.md` 依据栏里脚本的行号是写手当时的读数，换上以文件头段落为准。
7. 挪进技能的细则出了工具员写范围与四道只认 `main-agent.md` 的检查（`references.md`「四」），在脚本那一批换上之前，改这三份技能的活写闸会拒；登记位不在入口列的几处（`references.md` 里写了，欠账表里没有）。
8. 总表依据栏的行号写手没重新核；这一版脚本只回查原话与合稿落点，不回查依据栏。

## ⑤ 字数（`len()` 数字符，`tools/wordcount.py` 量的，`wordcounts.txt`）

| | 仓里原件 | 第二版 | 第三版 |
|---|---|---|---|
| 常驻（`CLAUDE.md`、`main-agent.md` 与 `@` 的项目规则） | 123271（21 份） | 72491（16 份） | 73874（16 份），比第二版多 1383 |
| 按需读（别的规则、技能、定义、共用约束、参考件） | 180250（34 份） | 189705（35 份） | 191988（35 份），比第二版多 2283 |

多出来的是放回的判据与原话（规则怎么读、四处半句、版本戳、整包弹窗半句、参考件对着文件头改的句子、代码轮与治理文件轮一节、调查员验证命令、弹窗闸与写闸的文件头句）；删掉的是重复的判据（拒算三类、needs-attention、`--check-facts`、写者锁、变异清单写全两遍）。

## ⑥ 自查输出（都在本目录）

- `selfcheck-pointers.txt`：全树「`文件`「节」」指针 406 处，文件找不到 0、节找不到 0、标题重复 0（标题逐字 350、标题开头且唯一 21、加粗条目名 32、只在正文 3）。
- `selfcheck-snippets.txt`：第二轮报告点名的原件原话 25 个字面片段，在原件与 `third/` 里都找得到（第三版新写的句子原件栏标 -1）；去向表每一行「改在」的片段 `tools/dispo_render.py` 逐条回查，problems=0。
- `selfcheck-frontmatter.txt`：24 份带 frontmatter 的文件与第二版逐字相同。
- `contradictions.md` 表头的回查结果：`rows=118 users=26 refs_ok=455 problems=0`（`tools/contra_build.py`，只读）。
- 不写位置指代与自称、没有 `⇒`、没有代号：对 `diff-from-r2.patch` 里的加行逐行 grep 过（只剩「上下文」这类普通词）。
- `edit-log.txt`：每次替换的文件与旧串开头；`tools/`：核对脚本。
