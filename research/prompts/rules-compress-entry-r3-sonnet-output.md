# 规则压缩复核第三轮（entry）正推腿报告

日期：2026-11-16。轮名 rules-compress-entry-r3，第三轮（最后一轮）。本腿立场：正推，逐处核 `research/prompts/rules-compress-r3-merged/diff-from-r2.patch` 落在这一组（CLAUDE-merged.md、dot-claude/main-agent.md、dot-claude/skills/merge-verify/SKILL.md、dot-claude/skills/dispatch/SKILL.md）的每一处改动。

## 说明

- 合稿树前缀：`research/prompts/rules-compress-r3-merged/`（下文引用都写全路径）。改前指第二版（补丁里 `-` 行，补丁行号标在表里），改后指第三版（合稿树里的行号，已现查）。
- 开工核了 sha256：仓里 `CLAUDE.md`、`.claude/main-agent.md` 与 `research/prompts/rules-compress-entry-r3-snapshot/judged-sha256.txt` 一致；合稿树四个文件与 `merged-tree-sha256.txt` 一致（`sha256sum` 现跑，四份都对上）。
- 技能 `merge-verify`、`dispatch` 在仓里没有原件（是从 `.claude/main-agent.md` 拆出来的），它们的依据是 `.claude/main-agent.md` 原件那一段与脚本文件头。
- 「成立 / 不成立 / 推不出」的读法：成立 = 这一格问的那件事上没找到问题；不成立 = 找到了问题；推不出 = 材料不够。
- 没往服务交任何任务，没跑项目代码；只用了读文件、`grep`、`sha256sum`、`cite-check.py` 与一个只读的引文核对脚本（见「没做什么」）。末行：`✓ 核了 2 处引文、118 处裸行号，对上 1 处，没判 119 处`（`cite-check.py` 退 0；报告的引文多用〔〕写，它不认，那几处由上述脚本核）。

## 三格结论

| 格 | 回 | 一句话 |
|---|---|---|
| 一 第二轮修补对不对 | 成立（带两处多出来的半句） | 这一组补丁 19 处改动里，放回原话的与原件逐字相同，指针指到的落点都在；用户定案第 24、25 条落对；有两处改后比依据多出半句，都不改变做法，见「多出来或少掉的半句」 |
| 二 修补引出的新问题 | 不成立（找到 4 处小的，没有改变做法的） | 同一条判据没有又住两处；指针都指得到且唯一；有 1 处指针的落点没有它说的东西、1 处读得出两种意思、2 处只是列举不全 |
| 三 能不能换上 | 补丁范围内成立；整组逐句对原件推不出 | 补丁范围内没找到总表与用户定案没登的做法不同；整组每一句对原件是攻方腿的活，本腿只对补丁里动到的句子 |

## 这一组能不能换上（一句话）

能换上。没有「换上之前必须改」的；下面「换上之后可以再改」的 5 条都是依据标签、落点用词、触发清单与列举不全，不改变照做的结果。

## 逐处核对表：CLAUDE-merged.md

| 编号 | 改前（补丁行） | 改后（合稿树行） | 依据 | 对不对 |
|---|---|---|---|---|
| C1 上游仓一行 | 转述：「多语言版本**对外以 `-en` 为准**，本项目只接这一份」（`research/prompts/rules-compress-r3-merged/diff-from-r2.patch:8`） | `research/prompts/rules-compress-r3-merged/CLAUDE-merged.md:40`〔同一份规范有多语言版本，**对外以 `-en` 为准**；本项目只接其中一份，不需要知道别的〕 | 原件 `CLAUDE.md:41`〔同一份规范有多语言版本，**对外以 `-en` 为准**；本项目只接其中一份，不需要知道别的〕 | 对：整格与原件逐字相同，「这一份」没有先行词的二义（第二版写成「这一份」，先行词落到 `-en` 上）消了 |
| C2 版本戳一行 | 转述：只有「对不上就红」，没有后半句（补丁行 11） | `research/prompts/rules-compress-r3-merged/CLAUDE-merged.md:42`〔对不上就红——那是在提醒「规矩变过了，先读再跑」〕 | 原件 `CLAUDE.md:43`〔对不上就红——那是在提醒「规矩变过了，先读再跑」〕 | 对：放回原话半句；只有前一个句号换成分号，不影响读法 |
| C3 参考件那一句 | 转述：到 `verification.md` 与 `implementation-workflow.md` 点名它的那一步之前读（补丁行 20） | `research/prompts/rules-compress-r3-merged/CLAUDE-merged.md:66`〔到 `.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md` 与 `.claude/main-agent.md` 派发表点名它的那一步之前读点名的那一节〕 | 原件 `CLAUDE.md` 没有参考件这一句（参考件是压缩新建的）；依据是合稿树 `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:96`〔各道判什么照 `.claude/rules/verification-reference.md`「门禁管哪一半」〕 | 对：派发表那一行确实点名了参考件，两个节名在参考件里各只出现一次（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:140` 是「门禁管哪一半」，`:14` 是「门禁 crash-point-replay 怎么跑」）；列举不全见 N3 |

## 逐处核对表：dot-claude/main-agent.md

| 编号 | 改前（补丁行） | 改后（合稿树行） | 依据 | 对不对 |
|---|---|---|---|---|
| M1 技能重读的触发 | 转述：「每次派发之前读」dispatch 技能，「压缩过上下文之后再做这两件事之前同样重读」（补丁行 515） | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:5`〔每次派发之前、被叫醒后处置之前、读交回之前读 `.claude/skills/dispatch/SKILL.md`〕 | 原件没有技能；依据是技能自己的三节（一轮怎么开、怎么收，派出去之后，交回怎么读），标题行在 dispatch 技能的第 10、22、43 行与 `main-agent.md:74`、`:78` 两处指针 | 对：三处触发与三节一一对上；与 main-agent.md 第 74 行起看门狗那一项的差异见 N4 |
| M2 职责一段 | 转述：「`crates/` 与 kb 里，主 agent 可以自己写的只有两类…；别的文件的简单任务照共用约束「派发」一节自己直接做」（补丁行 520） | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:9`〔探索性的 `crates/` 改动、定案后主 agent 自己写得清的小处 kb 改动可以自己写（派发表「改 `crates/`：探索性的」与「一批阶段任务结束」两行）〕 | 原件 `.claude/main-agent.md:7`〔探索性的 `crates/` 改动、定案后主 agent 自己写得清的小处 kb 改动可以自己写〕；用户定案第 25 条 `records/2026-11-16-三方豁免取消与存量补轮.md:213`〔25. 主 agent 自己可以写什么：「三处原话各留，不合并」，不写「只有」。〕 | 对：这一句与原件那一段逐字相同，「只有两类」与「别的文件的简单任务…」两处第二版新加的都删了（`grep -rnF` 合稿树 0 命中）；另两处原话在 `CLAUDE-merged.md:21`〔直接改，不先问〕与 `dot-claude/agent-common.md:11`〔自己直接做〕，后者见旁注 S1 |
| M3 整包弹窗半句 | 转述：「三遍确认另写进调度记录」，没有弹窗那半句（补丁行 529） | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:20`〔带 `--whole-package-confirmed` 的提交弹窗要用户确认，对话不自己决定〕 | 原件 `.claude/main-agent.md:19`〔带它的提交弹窗要用户确认，对话不自己决定〕；钩子 `.claude/hooks/run-scope-guard.sh:7`〔要用户在界面上确认（下面「放宽检查的提交标识」）〕 | 对：放回原话，与钩子文件头一致；原件里「累计满两小时、`task confirm`、四小时硬上限」那几句住在共用约束 `dot-claude/agent-common.md:65`（`grep` 命中 `over-four-hours` 与 `task confirm`），入口不丢 |
| M4 一轮第 3 条 | 转述：「提交之外要跑不指定范围的，照…「提交之外」那一条先弹窗」（补丁行 536） | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:26`〔确认一次（钩子弹过的不另问）〕 | 用户定案第 24 条 `records/2026-11-16-三方豁免取消与存量补轮.md:212`〔带 `--full-gate` 或 `--whole-package-confirmed` 的命令由钩子弹的那一次就算确认，主 agent 不另问第二次；命令不带这两个标记、钩子不弹的，主 agent 自己弹窗问一次。〕；落点 `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:38` 两半都写了 | 对：改后只带前半句，后半句（不带标记的主 agent 自己问一次）由落点那一条写全，不丢；原件 `.claude/main-agent.md:26`〔先弹窗问用户，同意了才跑〕的「同意了才跑」在落点 `verification.md:38`〔要用户确认一次、同意了才跑〕里；与第 24 条的两半一一对上，没有第 24 条没说的半句 |
| M5 写段第 6 条 | 转述：只有「直接回那个会话「已收尾」」（补丁行 545） | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:51`〔调查员「查到就修」那一支在主工作区改了、验证命令还没跑完的，先回在改，命令跑完再回「已收尾」〕 | 原件 `.claude/main-agent.md:51`〔主工作区里没有打了而没验过的补丁，直接回那个会话「已收尾」〕；用户定案第 19 条 `records/2026-11-16-三方豁免取消与存量补轮.md:171`〔19. 调查员「查到就修」那一支：「照旧在主工作区改」，规则里写明它是「一律交补丁」的例外。〕 | 前半句与原件逐字相同。新加的半句原件没有、第 19 条也没说（第 19 条只定了「在主工作区改」）；总表登了（`research/prompts/rules-compress-r3-merged/contradictions.md:128` 入口-15，标「改了判据」），但依据栏写「用户定案第 19 条」不对：它是修稿一方对第二轮腿打中的处置。做法上说得通（原件那条的前提「主工作区里没有打了而没验过的补丁」在这一支上不成立）。归「多出来的半句」H1 |
| M6 拒算第 1 条 | 转述：「实现员不往拒算表里写，主 agent 也不往哪一棵树里手写」「罩不到…这三类审 diff 时…」，同一判据在入口与技能末段各写一遍（补丁行 554、1218） | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:68`〔实现员不往拒算表里登记，收的行只走拒算补丁目录〕，同一行后接〔`refused-test-registry.py` 的清单只取验证包不带特性那一趟的测试函数，罩不到挂特性的测试文件、产品 GPU 包的测试包 `singlefs-background-gpu-tests` 与别的包里的新测试〕 | 原件 `.claude/main-agent.md:68`〔实现员不登记，也不替它登记〕与 `.claude/main-agent.md:72`〔这份清单只取验证包不带特性那一趟的测试函数〕；脚本文件头 `scripts/gate/refused-test-registry.py:22`〔覆盖：只有验证包不带特性那一趟的测试函数；挂特性的测试文件（feature-gated-tests 格）、产品 GPU 包（product-gpu-real-card-tests 格）与别的包里的 #[test] 的拒算这里不取。〕 | 对：三类与脚本文件头逐类对上；判据只住入口一处，技能末段删掉了（V3）；「不登记」「不替它登记」的两半由「收的行只走拒算补丁目录」加上同节开头〔不另往哪一棵树里手写这一行〕（`dot-claude/main-agent.md:66`）合起来覆盖。「第 3c 步」那一指针没写文件名，见 N5 |
| M7 派出去之后的末句 | 转述：「细则…：先读 `.claude/skills/dispatch/SKILL.md`「派出去之后」」（补丁行 561） | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:74`〔做这一步（起看门狗、被叫醒后处置）之前先读〕 | 同 M1：技能「派出去之后」节 `research/prompts/rules-compress-r3-merged/dot-claude/skills/dispatch/SKILL.md:22` | 对：触发写清了（起看门狗、被叫醒后处置）；原件没有技能，这是压缩新引的指针 |
| M8 交回怎么读的末句 | 转述：「先读 dispatch 技能「交回怎么读」」（补丁行 566） | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:78`〔读交回之前先读〕 | 同 M1：技能「交回怎么读」节 `research/prompts/rules-compress-r3-merged/dot-claude/skills/dispatch/SKILL.md:43` | 对 |
| M9 派发表「要推论」行 | 转述：「开不开第二、三轮照同一份「多轮：…」」（补丁行 575） | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:90`〔设计推论轮开不开下一轮照同一份「多轮：一次打穿不算数，三轮里多数打穿才算」；代码轮与治理文件轮照 `.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」〕 | 用户定案第 23 条 `records/2026-11-16-三方豁免取消与存量补轮.md:211`〔23. 治理文件（规则、定义、门禁、脚本）的三方轮「照代码轮收」：打中的主 agent 现查坐实就改，改完再攻一轮，没新打中就收，最多三轮。〕；落点两节都在（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:25` 是「代码轮与治理文件轮怎么收」，`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:41` 是「多轮：一次打穿不算数，三轮里多数打穿才算」） | 对：这一行只指路，两类轮各指一处；第 23 条的内容在落点（`implementation-workflow.md:25` 起）。原件 `.claude/main-agent.md:116` 的「第一轮结论不一致无法收敛，开第二轮…第三轮为最终轮」已由总表与用户定案第 2 条登记 |

## 逐处核对表：dot-claude/skills/dispatch/SKILL.md

| 编号 | 改前（补丁行） | 改后（合稿树行） | 依据 | 对不对 |
|---|---|---|---|---|
| D1 第 4 条的实现员与调查员 | 转述：「实现员一律交补丁，只派一个也这样（调查员「查到就修」那一支除外，在主工作区改；各在副本里改，交…）」，「各」没有先行词（补丁行 1140） | `research/prompts/rules-compress-r3-merged/dot-claude/skills/dispatch/SKILL.md:14`〔（实现员各在副本里改，交 `crates.patch` 与 `mutations-append.tsv` 等，主 agent 用 `scripts/gate/apply-writer-patch.py` 打；调查员「查到就修」那一支是例外，判据在 `.claude/main-agent.md`「写与验分两段」写段第 2 条）〕 | 原件 `.claude/main-agent.md:27`〔（各在副本里改，交 `crates.patch` 与 `mutations-append.tsv` 等，主 agent 用 `scripts/gate/apply-writer-patch.py` 打）〕；用户定案第 1 条 `records/2026-11-16-三方豁免取消与存量补轮.md:118`〔1. 实现员「一律交补丁」：只派一个也在副本里改、交补丁。〕与第 19 条（见 M5）；落点 `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:47`〔调查员「查到就修」那一支是例外，照旧在主工作区改〕 | 做法对：先行词补上了，第 1 条与第 19 条都落对。「判据在…写段第 2 条」的「判据」用词有偏差：写段第 2 条只写例外存在并指到调查员定义，什么算「查到就修」写在 `.claude/agents/investigator.md`，见 N1 |
| D2 坏卡与服务健康 | 转述：「处置后用 `python3 task-server/control check` 回读，退 0 才算服务健康（缺一样退 78，出路写在输出里）」（补丁行 1149） | `research/prompts/rules-compress-r3-merged/dot-claude/skills/dispatch/SKILL.md:39`〔处置后的回读照 `.claude/main-agent.md`「派出去之后」；`python3 task-server/control check` 缺一样退 78，出路写在输出里〕 | 原件 `.claude/main-agent.md:88`〔处置后用 `python3 task-server/control check` 回读，退 0 才算服务健康（缺一样退 78，出路写在输出里）〕；脚本文件头 `task-server/control:16`〔都过退 0；缺一样退 78、出路写全〕 | 对：「退 0 才算健康」那半句住入口 `dot-claude/main-agent.md:74`，技能只留退 78，一处写全一处指过去；退 78 与文件头一致 |
| D3 needs-attention | 转述：技能里写全「退出…时任务还在、不是结局：读提醒，处置完再起 `task wait`」，与入口重复（补丁行 1152） | `research/prompts/rules-compress-r3-merged/dot-claude/skills/dispatch/SKILL.md:41`〔`task wait` 以 `needs-attention` 退出时怎么办照 `.claude/main-agent.md`「派出去之后」；退出码以 `task-server/task_server_lib.py` 文件头「错误类型」表为准〕 | 原件 `.claude/main-agent.md:94`〔读 `task wait` 打出的那一行提醒，处置完再起 `task-server/task wait <号>`〕；`task-server/task_server_lib.py:2046`〔任务还没结局时 wait 报提醒退这个码〕 | 对：入口 `dot-claude/main-agent.md:74` 写着〔`task wait` 以 `needs-attention` 退出时任务还在、不是结局〕，落点在；技能里不再写「不是结局」，由入口一处写全 |
| D4 事实表验收 | 转述：「交回的事实表过了 `scripts/kb/stale-candidates.py --check-facts` 才切候选表」（补丁行 1161） | `research/prompts/rules-compress-r3-merged/dot-claude/skills/dispatch/SKILL.md:75`〔事实表的验收照 `.claude/main-agent.md` 派发表这一行〕 | 原件 `.claude/main-agent.md:121`〔交回的事实表过了 `scripts/kb/stale-candidates.py --check-facts`〕；落点 `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:95`〔交回的事实表过了 `scripts/kb/stale-candidates.py --check-facts` 才往下〕 | 对：落点写了同一判据，「这一行」指前文点名的「一批阶段任务结束」那一行，唯一；脚本 `scripts/kb/stale-candidates.py:14` 有 `--check-facts` |

## 逐处核对表：dot-claude/skills/merge-verify/SKILL.md

| 编号 | 改前（补丁行） | 改后（合稿树行） | 依据 | 对不对 |
|---|---|---|---|---|
| V1 整批重交的退 2 | 转述：「第 1 步自己红了、退 2 的，第 2 步还没做过，只重交第 1 步」，「退 2」有几种意思（补丁行 1199） | `research/prompts/rules-compress-r3-merged/dot-claude/skills/merge-verify/SKILL.md:14`〔第 1 步只重跑红的那一条退 2、说那一趟开跑以来主工作区变了的，第 2 步还没做过，只重交第 1 步〕 | 原件 `.claude/main-agent.md:57`〔那一条退 2、说那一趟开跑以来主工作区变了的，照它的出路整批重交〕；脚本文件头 `scripts/gate/merge-and-verify.sh:13`〔开跑先照那一趟记的指纹核主工作区自那一趟开跑以来没变，变了退 2、一个补丁都不打〕与 `:209`〔--fingerprint-of 那一趟开跑以来主工作区在它的起点上变了〕 | 对：点名了是带 `--fingerprint-of` 那一种退 2，与文件头一致；打补丁核不过那种退 2（同节退 2 一支）的出路另写在段树那一句，不混。「第 2 步还没做过，只重交第 1 步」原件没有这半句，是 `整批重交` 定义里自己推的，见 H2 |
| V2 退 3 的整包确认 | 转述：「照「整包归提交时的整轮门禁；提交之外整包跑先弹窗问用户」弹窗」（补丁行 1208） | `research/prompts/rules-compress-r3-merged/dot-claude/skills/merge-verify/SKILL.md:22`〔提交之外要整包跑照 `.claude/rules/verification.md`「一个门禁：两种跑法」「提交之外」那一条确认一次（带 `--whole-package-confirmed` 的由钩子弹，主 agent 不另问）〕 | 原件 `.claude/main-agent.md:57`〔照「整包归提交时的整轮门禁；提交之外要整包跑先弹窗问用户」弹窗〕；用户定案第 24 条（引文见 M4）；钩子 `.claude/hooks/run-scope-guard.sh:33`〔--whole-package-confirmed 让整包测试过提交检查；对话不许自己用，要用户在界面上确认〕 | 对：后面「用户同意的…带 `--whole-package-confirmed`」「用户不同意的不合」保留，同一趟整包命令本来就带标识，钩子弹的那一次就是确认；与第 24 条的前半句一一对上 |
| V3 删末段 | 转述：技能末尾一段重写「清单只取验证包不带特性那一趟…三类审 diff 时照第 3c 步」（补丁行 1218） | 删除；判据只留在 `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:68` | 原件 `.claude/main-agent.md:72`〔这份清单只取验证包不带特性那一趟的测试函数〕；脚本 `scripts/gate/refused-test-registry.py:22` | 对：判据不再住两处；技能第 3 条〔没有细则〕仍指入口，不受影响 |

## 改后比依据多出来或少掉的半句

| 号 | 位置 | 多出来 / 少掉 | 半句 | 去向 |
|---|---|---|---|---|
| H1 | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:51` | 多出来 | 〔调查员「查到就修」那一支在主工作区改了、验证命令还没跑完的，先回在改，命令跑完再回「已收尾」〕 | 原件 `.claude/main-agent.md:51` 没有；用户定案第 19 条只说「照旧在主工作区改」，没说怎么回别的会话；总表 `research/prompts/rules-compress-r3-merged/contradictions.md:128`（入口-15）登了「改了判据」，但依据栏写成用户定案第 19 条。实际是修稿一方对第二轮腿打中的处置，该改标「主 agent 处置」（总表表头 `contradictions.md:5` 的标签更正一段就是这么改别的行的）。「改了判据」是真话：这一支让原件那句「主工作区里没有打了而没验过的补丁」的前提不再恒成立 |
| H2 | `research/prompts/rules-compress-r3-merged/dot-claude/skills/merge-verify/SKILL.md:14` | 多出来（补丁没动它，第二版就有） | 〔第 2 步还没做过，只重交第 1 步〕 | 原件 `.claude/main-agent.md:57` 只写了「照它的出路整批重交」与（已做过第 2 步时）「第 2 步在新留下的快照上重验」；这半句是自然推论，不改变做法，不挡换上 |
| H3 | `research/prompts/rules-compress-r3-merged/dot-claude/skills/dispatch/SKILL.md:14` | 多出来 | 〔判据在 `.claude/main-agent.md`「写与验分两段」写段第 2 条〕里的「判据」 | 写段第 2 条（`research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:47`）只写例外存在；什么算「查到就修」写在 `research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:23`〔写「查到就修」的：给压着的条款〕。见 N1 |
| H4 | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:26` | 少掉（落点有） | 原件 `.claude/main-agent.md:26`〔同意了才跑〕 | 住在 `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:38`〔要用户确认一次、同意了才跑〕，入口指过去 |
| H5 | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:20` | 少掉（落点有） | 原件 `.claude/main-agent.md:19` 里的确认文件、两小时提醒、`task confirm`、四小时硬上限 | 住在 `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:65`（`grep` 命中 `over-four-hours`、`task confirm`） |
| H6 | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:68` | 少掉（改写） | 原件 `.claude/main-agent.md:68`〔实现员不登记，也不替它登记〕的「也不替它登记」 | 由〔收的行只走拒算补丁目录〕加 `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:66`〔不另往哪一棵树里手写这一行〕覆盖，等价；总表 `contradictions.md:129`（入口-16）标「只改措辞」，站得住 |
| H7 | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:9` | 少掉（落点有） | 原件 `.claude/main-agent.md:7` 句尾「自己写的 `crates/` 改动并进同一批的代码轮三方」 | 住在 `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:105`〔`crates/` 改动（产品代码与测试代码都算）并进同一批的代码轮三方〕 |

## 格二：修补引出的新问题

同一条判据是不是又住了两处：核了本补丁动到的七处判据，都只住一处，落点指过去——整包弹窗（`dot-claude/rules/verification.md:38`）、拒算清单射程（`dot-claude/main-agent.md:68`，技能末段已删）、`needs-attention`（`dot-claude/main-agent.md:74`）、`control check` 回读（同一行）、`--check-facts`（`dot-claude/main-agent.md:95`）、退 2 的整批重交（`dot-claude/skills/merge-verify/SKILL.md:14`）、调查员例外（`dot-claude/main-agent.md:47`）。没有又住两处的。

指针指不指得到且唯一：本补丁新写的指针十四处（合稿树行号：`CLAUDE-merged.md:66`；`main-agent.md:5`、`:74`、`:78`、`:90`、`:96`；`dispatch/SKILL.md:14`、`:39`、`:41`、`:75`；`merge-verify/SKILL.md:22`、`:14`）逐个 `grep` 过节名与行名，都指得到、各只有一处，标题重复 0。找到的问题如下，都小：

| 号 | 位置 | 问题 | 做法会不会因此不同 |
|---|---|---|---|
| N1 | `research/prompts/rules-compress-r3-merged/dot-claude/skills/dispatch/SKILL.md:14`〔判据在 `.claude/main-agent.md`「写与验分两段」写段第 2 条〕 | 落点是例外的句子，不含「判据」；读的人到了那一句还要再跳 `investigator.md` | 不会：落点写明例外存在，再指到调查员定义 |
| N2 | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:51`〔先回在改〕 | 「回在改」读得出两种：回一句「在改」；或先去把改的做完再回。后半句也没写别的会话收到「在改」之后做什么、等多久 | 小：两种读法下，等的都是验证命令跑完再回「已收尾」 |
| N3 | `research/prompts/rules-compress-r3-merged/CLAUDE-merged.md:66` 的三处列举 | 列举不全：技能 `research/prompts/rules-compress-r3-merged/dot-claude/skills/crash-test/SKILL.md:18`〔各格判什么在 `.claude/rules/verification-reference.md`「门禁管哪一半」〕与 `research/prompts/rules-compress-r3-merged/dot-claude/skills/crash-test/commit-stages.md:24`〔照 `.claude/rules/verification-reference.md`「门禁 crash-point-replay 怎么跑」与门禁文件头〕也点名它，工具员定义与 `fs-design.md` 也点名 | 不会：那几处自己写了怎么读，主 agent 跟着技能走就读到 |
| N4 | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:5` 与 `:74` | 重读技能的触发清单两处写法不同：第 5 行三件事不含「起看门狗」，第 74 行含；压缩上下文之后第 5 行那一句不会让人在起看门狗之前重读 | 小：起看门狗总在一次派发之后，派发前已读；压缩上下文之后才会掉 |
| N5 | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:68`〔这三类审 diff 时照实现员定义第 3c 步看有没有读活树、有没有列进报告〕 | 原件 `.claude/main-agent.md:72` 写的是 `.claude/agents/implementation-writer.md` 第 3c 步，这里只写「实现员定义」，没有路径；唯一（`research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:49` 是 3c） | 不会 |

## 格三：补丁范围内与原件比做法

补丁动到的 19 处里，改后与原件相比做法有不同的有三处，都已登记：M4 与 V2（用户定案第 24 条，总表入口-1 `contradictions.md:15`）、M5（总表入口-15，标签见 H1）。其余十六处与原件或脚本文件头一致，或只是把同一判据收成一处。没找到总表与用户定案都没登的做法不同。

整组每一句对原件（尤其带「不」「只」「一律」「除外」「先」「才」的）是攻方腿的活；本腿只抽了入口一轮第 1 到 15 条里的几条（`40 万上下文`、监控一行、`不是自己的任务，交给它的会话`、`用完就删`、`收拢要定期做`、`SendMessage 发给那个会话`）确认合稿树里在（`grep` 都有命中，行 31 到 38），没逐句比，所以整组「推不出」。

## 旁注：不在这一组、但读的时候撞到的

- S1 用户定案第 25 条说三处原话各留；`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:11`〔自己直接做〕不是原件 `.claude/agent-common.md:8`〔简单任务不派发，自己直接改〕的原话，修稿报告也把它写成〔自己直接做，不派〕。归共用约束那一组，不影响这一组。
- S2 用户定案第 27 条（`records/2026-11-16-三方豁免取消与存量补轮.md:246`）要删「那一轮只攻上一轮的修补」；合稿树 `research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:30` 还留着。第 27 到 29 条是第三版冻结之后定的（本轮正文只列 1 到 26），归规则那一组，主 agent 换上时要带上。

## 要改的（逐条标时机）

换上之前必须改：无。

换上之后可以再改（都在这一组里）：

1. `research/prompts/rules-compress-r3-merged/dot-claude/skills/dispatch/SKILL.md:14` 把「判据在」改成「例外写在」（N1）。
2. `research/prompts/rules-compress-r3-merged/contradictions.md:128` 入口-15 依据栏改记「主 agent 处置」；`research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:51` 写明「回在改」是回一句什么、对方等什么（H1、N2）。
3. `research/prompts/rules-compress-r3-merged/CLAUDE-merged.md:66` 去掉三处列举，或补上 crash-test 技能（N3）。
4. `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:5` 的触发清单补上「起看门狗」，或 `:74` 改成与它一致（N4）。
5. `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:68` 的「实现员定义」写回 `.claude/agents/implementation-writer.md`（N5）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 一 第二轮修补对不对 | 成立 | 19 处里放回的原话与原件逐字相同（C1、C2、M2、M3），与文件头一致（M6、V1、D2、D3），用户定案第 24、25 条落对；两处多出来的半句（H1、H3）不改变做法 |
| 二 修补引出的新问题 | 不成立 | 没有又住两处；指针都指得到且唯一；找到 N1 到 N5 五处小问题，都不改变照做的结果 |
| 三 能不能换上 | 补丁范围内成立，整组推不出 | 补丁里动到的句子没有总表与用户定案没登的做法不同；整组逐句对原件本腿没做 |
| 这一组换上 | 能换上 | 换上之前必须改的没有；换上之后可以再改的五条见上 |

## 要主 agent 跑的命令

无。本腿没有需要经任务服务跑的命令，也没有写模型、没有起样本进程、没有建仓副本。

## 没做什么

- 不判别的腿的格（整组对原件逐句的格三）没替攻方腿做；没替主 agent 采纳或出判决；没往统一任务服务交任务，判决要引的数没有，也就没有要复跑的。
- 没读禁读清单里的文件（攻方腿这一轮的报告与模型目录、草稿目录，别的几题第三轮各腿的产出）。
- 草稿目录 `~/.local/share/singlefs/work/rules-compress-entry-r3-sonnet` 里临时放过一个只读核对脚本（逐条核报告里〔〕引文在被引行里逐字在不在：60 处全对，另一处是缩写路径 `CLAUDE-merged.md:21`，用 `grep -nF` 核过），已删；没有副本与编译目录。
