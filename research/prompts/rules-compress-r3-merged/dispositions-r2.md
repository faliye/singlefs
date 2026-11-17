# 十份腿报告逐条去向（压缩 2f，第三版）

写法：「题·腿」里 攻 = opus 攻方腿，辩 = sonnet 辩方腿；题名是入口、验证、常驻、定义、矛盾（对应 `rules-compress-<题>-r2-*-output.md`）。两条腿（或两题）说同一件事的并成一行，「题·腿」一栏列出各条腿。「成立」指对着仓里的原件与脚本、门禁、钩子文件头现核之后这一条站得住；「不成立」是现核之后不改（理由写在最后一栏）；「部分」是只有一半要改在树里，或只涉及脚本一侧与主 agent 的核查做法。最后一栏的 文件:行 都是 `third/` 里的（`dot-claude/` 即仓里的 `.claude/`，`CLAUDE-merged.md` 即 `CLAUDE.md`），由脚本 `tools/dispo_render.py` 按片段回查，找不到就不出表。腿提的改法只当线索，句子都照原件或脚本文件头的原话补回。

辩方腿对第一轮判决逐条复核判「站得住」、第二版修补「够得着」且没改错的各条，不单列行：它们已落在第二版里，第三版没有再动（第二版去向表里同一些行没有重出）。

## 条数（脚本数出来的，与下面的表逐行对得上）

共 69 条，成立 54 条，不成立 7 条，部分 8 条（脚本 `tools/dispo_render.py` 数表里的行：69 行，四个数相加 69）。

| 题·腿 | 对应报告 | 总数 | 成立 | 不成立 | 部分 |
|---|---|---|---|---|---|
| 入口·攻 | rules-compress-entry-r2-opus-output.md | 10 | 10 | 0 | 0 |
| 入口·辩 | rules-compress-entry-r2-sonnet-output.md | 8 | 5 | 1 | 2 |
| 验证·攻 | rules-compress-verification-r2-opus-output.md | 14 | 14 | 0 | 0 |
| 验证·辩 | rules-compress-verification-r2-sonnet-output.md | 8 | 6 | 1 | 1 |
| 常驻·攻 | rules-compress-residents-r2-opus-output.md | 7 | 7 | 0 | 0 |
| 常驻·辩 | rules-compress-residents-r2-sonnet-output.md | 6 | 4 | 2 | 0 |
| 定义·攻 | rules-compress-definitions-r2-opus-output.md | 10 | 9 | 0 | 1 |
| 定义·辩 | rules-compress-definitions-r2-sonnet-output.md | 7 | 4 | 1 | 2 |
| 矛盾·攻 | rules-compress-contradictions-r2-opus-output.md | 13 | 11 | 1 | 1 |
| 矛盾·辩 | rules-compress-contradictions-r2-sonnet-output.md | 11 | 7 | 2 | 2 |
| 各行按报告累计（一行列几条腿算几次） | | 94 | 77 | 8 | 9 |

## 去向表

| 题·腿 | 编号 | 一句话 | 成立 | 改在 third/ 的 文件:行 / 不改的理由 |
|---|---|---|---|---|
| 入口·攻、入口·辩、矛盾·辩 | 入口攻 一-1；入口辩 第 3 条；矛盾辩 第二节第 2 条 | 「主 agent 可以自己写的只有两类」没按判决去掉，收窄成限定在 crates/ 与 kb，与共用约束「简单任务自己直接做」相抵 | 成立 | `dot-claude/main-agent.md:9`：照用户定案第 25 条：入口留原件那一句的原意、没有「只有」，第二版新加的「别的文件的简单任务…自己直接做」删掉；`CLAUDE-merged.md`「修它们本身就是这一轮的出口时，直接改，不先问」与共用约束「简单任务不派发，自己直接改」各留原话 |
| 入口·攻 | 攻 一-2 | 压缩上下文之后的重读只罩验证段与派发两件事，「派出去之后」「交回怎么读」两节入口只写「先读」 | 成立 | `dot-claude/main-agent.md:5`；`dot-claude/main-agent.md:74`；`dot-claude/main-agent.md:78`：照主 agent 的处置：入口第 5 行与两节入口都写明被叫醒、读交回之前，正文不在上下文里就整份重读；技能 frontmatter 按「与第二版相同」不动 |
| 入口·攻、入口·辩 | 攻 二-1；辩 第 4 条残留 | 「整批重交」定义里「第 1 步自己红了、退 2 的」读得出一条指令，撞第 1 步退 1、退 2 两支 | 成立 | `dot-claude/skills/merge-verify/SKILL.md:14`：点名是哪一种退 2（脚本文件头 `merge-and-verify.sh` 的退出码表里退 2 有几种意思，只有 `--fingerprint-of` 那一种出路是整批重交） |
| 入口·攻 | 攻 二-2 | 调查员「查到就修」例外写进写段第 2 条后，写段第 6 条「主工作区里没有打了而没验过的补丁」不再成立 | 成立 | `dot-claude/main-agent.md:51`：第 6 条加调查员例外；同一句话在别处的出现处见「定义」题同类行 |
| 入口·攻 | 攻 二-3 | 拒算第 1 条「主 agent 也不往哪一棵树里手写」读得出连删行也不许，撞技能「已不拒算」行主 agent 直接删，且与同节开头重复 | 成立 | `dot-claude/main-agent.md:68`：删掉第 1 条里的重复半句，「不另往哪一棵树里手写这一行」只留同节开头一处 |
| 入口·攻、入口·辩 | 攻 二-4（五处）；辩 表末行与「顺带看到」 | 同一条判据住入口与技能两处：needs-attention 不是结局、control check 回读、--check-facts、拒算罩不到的三类、调查员例外（技能那一处括号把例外插在「各在副本里改」前面，读得出相抵） | 成立 | `dot-claude/main-agent.md:74`；`dot-claude/skills/dispatch/SKILL.md:41`；`dot-claude/skills/dispatch/SKILL.md:39`；`dot-claude/skills/dispatch/SKILL.md:75`；`dot-claude/main-agent.md:68`；`dot-claude/skills/dispatch/SKILL.md:14`：每条判据收成一处：前四条判据留入口（常驻、只提交不派子 agent 的会话也读得到），技能那一处改写成指过去、只留入口没有的细节（退出码以文件头为准、缺一样退 78）；拒算那一条把脚本射程「只取验证包不带特性那一趟」与包名并进入口，技能末段删；调查员例外判据留写段第 2 条，技能拆成两句 |
| 入口·攻 | 攻 观察-1 | 派发表新指到参考件，`CLAUDE-merged.md` 读参考件的时机只列两份规则 | 成立 | `CLAUDE-merged.md:66`：时机一句补上主 agent 派发表 |
| 入口·攻、矛盾·攻、矛盾·辩 | 入口攻 观察-2；矛盾攻 核不动 1、常驻-8；矛盾辩 第二节第 3 条 | 「提交之外」那一条是一次弹窗还是两次，修稿一方自己挂着没定 | 成立 | `dot-claude/rules/verification.md:35`：照用户定案第 24 条：带 `--full-gate` 或 `--whole-package-confirmed` 的命令，钩子弹的那一次就是确认、主 agent 不另问；不带这两个标记、钩子不弹的，主 agent 自己弹窗问一次；句子只住 verification.md 一处（落点逐处见下面「用户定案 24」行） |
| 入口·攻 | 攻 三-1 | 版本戳红了「先读再跑」丢了，只剩不常驻的上游 README | 成立 | `CLAUDE-merged.md:42`：照原件 `CLAUDE.md` 原话放回 |
| 入口·攻 | 攻 三-2 | 「本项目只接这一份」的先行词落到 `-en` 上，读得出只接 `-en` | 成立 | `CLAUDE-merged.md:40`：照原件 `CLAUDE.md` 原话放回，不另改成 `-zh` |
| 入口·辩 | 辩 第 3 条与表第 3 行 | 入口第 9 行新增的半句是新判据（把原件封闭写法与共用约束的冲突按共用约束一边定了） | 成立 | `dot-claude/main-agent.md:9`：并入第一行，同一处改 |
| 入口·辩 | 辩 不采纳 22 行 | 跑整包的确认办法：原件入口写「带它的提交弹窗要用户确认，对话不自己决定」，第二版入口只留三遍确认，弹窗半句只在共用约束 | 成立 | `dot-claude/main-agent.md:20`：把这半句放回入口；与用户定案 24 的钩子弹窗一致 |
| 入口·辩 | 辩 第 7 条登记位 | 细则出了工具员写范围与四道检查的射程：引用改法清单写的「:33 到 :36」偏窄（实际 33 到 45 行），登记位不在入口规定的几处 | 部分 | `references.md:168`：行号改到 33 到 45；登记位（里程碑收口表、欠账表、决策正文）是主 agent 换上时要登的，树里不写，交主 agent（result.md ④） |
| 入口·辩 | 辩 第 2 条「同一形状没处理」 | `CLAUDE.md` 的读一次清单压缩上下文之后同样会掉 | 不成立 | 原件 `CLAUDE.md:34`、`:68`、`:70` 就这样写，第二版没动，不是压缩带来的；用户定案与主 agent 的处置都只要求把技能重读的入口补到「派出去之后」「交回怎么读」两节，不扩到别的读一次清单 |
| 入口·辩 | 辩 核查方式的缺口 1 到 4 | 第一轮判决：没自己跑 cite-check、现查只盖了部分判决条、两个总数没人复核、不采纳各行没记读过哪几行 | 部分 | 说的是主 agent 核查的做法，不是树里的句子；cite-check 已由主 agent 补跑，判决各条现查归压缩 2g（主 agent），树不改 |
| 验证·攻 | 攻 一-1；辩 第二节末行（提交之外）、第四节末条 | 第 18 条只落了提交时那一趟，提交之外是先弹窗再被钩子弹一次；定案 24 在第二版之后 | 成立 | `dot-claude/rules/verification.md:38`：照用户定案第 24 条改：句子只住 verification.md「提交之外」那一条，别处（main-agent、agent-common、session-wrapup、crash-test 技能、gate 技能、staged-gate、commit-stages、merge-verify 退 3 一支）指过去或写同一个意思；钩子弹哪几种命令照 `.claude/hooks/run-scope-guard.sh` 文件头「放宽检查的提交标识」那一段 |
| 验证·攻 | 攻 一-2 | 「停」与「再抽一次」的先后，去向表与总表把依据挂在用户定案第 17 条名下，第 17 条没说这件事 | 成立 | `dot-claude/rules/three-way-inference.md:45`：用户定案第 26 条正好定了这件事：两条腿都没打中记没打中、这一题停；只有一条腿覆盖到的范围再派同立场腿；总表里这一行的依据改成用户定 26，不再挂第 17 条 |
| 验证·攻、验证·辩 | 攻 一-3；辩 第二节「开轮四句」 | 「开了第二、三轮的，其中至少一轮派辩方」读得出两种（只开到第二轮的管不管） | 成立 | `dot-claude/rules/three-way-inference.md:45`：写成「开了第二轮的，第二轮与第三轮里至少一轮派辩方，只开到第二轮的在第二轮派」；代码轮与治理文件轮同样（用户定 23：开了第二轮的在第二轮派辩方腿） |
| 验证·攻 | 攻 一-4 | 放回常驻的判器调用点一句写「两样都不是的判红」，判法文件头是五样有一样成立就合规 | 成立 | `dot-claude/rules/verification.md:25`：常驻一句改成指文件头「一处调用点算合规，有一样成立就行」那一段并写明不止两样；参考件同一行改成列文件头五样 |
| 验证·攻、矛盾·辩 | 验证攻 二-1（连同 二-2）；矛盾辩 第二节第 4 条 | 「设计推论轮」没有定义，规则、定义、治理文件的轮照哪一套开轮读出两种；「多轮这一套」管整节还是只管开轮那一段 | 成立 | `dot-claude/rules/three-way-inference.md:43`；`dot-claude/rules/implementation-workflow.md:10`：照用户定案第 23 条：写出设计推论轮的定义（设计判断、取舍、推翻或确立决策的那一类）；三方规则「多轮」一节逐段写明管哪类轮（写了「设计推论轮」的两段只管那一类，别的段管三类轮）；治理文件轮与代码轮照 implementation-workflow.md 新写的一节收 |
| 验证·攻 | 攻 二-3 | 「任何结论」与「别的结论」同在一条里，停下那一轮的结论算不算被支撑读出两种 | 成立 | `dot-claude/rules/three-way-inference.md:72`：照用户定案第 26 条重写：两条腿都没打中就记没打中、这一题停；只有一条腿覆盖到的范围要拿它的「没打中」当依据时再派同立场腿；「开不开下一轮」与「一条腿只抽一次样」两处写同一件事、不相抵 |
| 验证·攻 | 攻 二-4 | 代码轮新走法留了两个空：现查推翻了怎么收，第三轮又打中怎么收 | 成立 | `dot-claude/rules/implementation-workflow.md:31`；`dot-claude/rules/implementation-workflow.md:32`：照用户定案第 23 条补：现查推翻的不改、不为它开轮；第三轮又打中的改了收、写进判决交用户表、不开第四轮 |
| 验证·攻 | 攻 二-5 | 「第一轮一致没打中就停」又住在 three-way-inference.md 与 implementation-workflow.md 两处 | 成立 | `dot-claude/rules/implementation-workflow.md:29`：implementation-workflow.md 只指过去，判据住 three-way-inference.md「开不开下一轮」一处 |
| 验证·攻 | 攻 三-1 | 参考件说 formal-verification「钉全」是一项，门禁文件头是 toolchain、proof（整格）、table 各一项，钉全由 E165 第二段驱动判 | 成立 | `dot-claude/rules/verification-reference.md:138`：照文件头改，同一表里「E165 的红例都钉进钉表」半句改成钉表成形，钉全归驱动；总表登照脚本改 |
| 验证·攻 | 攻 三-2 | broad-mutation「不指定范围那一趟自己先起驱动」与文件头「只在服务分片起的那一趟起驱动」不合 | 成立 | `dot-claude/rules/verification-reference.md:149`：照文件头「判什么」那一段改；总表登照脚本改 |
| 验证·攻 | 攻 三-3 | 「formal-verification 各格」少了第七格 same-origin-counter | 成立 | `dot-claude/rules/verification-reference.md:170`：补一条，节首写七格；总表登照脚本改 |
| 验证·攻 | 攻 三-4 | 「领卡被拒的那一格判红、那一趟不起」对 feature-gated-tests 那一处不成立 | 成立 | `dot-claude/rules/verification-reference.md:21`：照文件头「显卡与清场」三处各写各的；总表登照脚本改 |
| 验证·攻 | 攻 三-5 | crash-point-replay：挂特性的测试文件跑到 0 条参考件写在「判什么」里，文件头报本次未跑、不判红 | 成立 | `dot-claude/rules/verification-reference.md:146`：照文件头「feature-gated-tests 格」那一段改；该文件头格名表一行写「不为 0」、后面一段写「报本次未跑、不判红」，两处写法不同，这里照后一段，列进 result.md ④ |
| 验证·攻 | 攻 三-6 | qemu-device-streams 与 lkmm 两行转述比文件头窄（两个对照必须红；每份 litmus 两条判定） | 成立 | `dot-claude/rules/verification-reference.md:158`；`dot-claude/rules/verification-reference.md:42`：两行照文件头改；总表登照脚本改 |
| 验证·辩 | 辩 第二节「声明模块」 | 声明模块那一条放回的话不是原话：常驻窄、参考件宽，同一条判据两种射程，总表登成「改回原判据」 | 成立 | `dot-claude/rules/verification.md:23`；`dot-claude/rules/verification-reference.md:151`：按用户的处置统一成脚本 `crash-case-check.py` 文件头 modules 一行的射程（逐字用文件头那一句）；总表标「照脚本改」 |
| 验证·辩 | 辩 第二节「测试线程」 | 测试线程放回常驻的话比原件宽（各趟），参考件仍是原件列法，总表没登 | 成立 | `dot-claude/rules/verification.md:27`；`dot-claude/rules/verification-reference.md:81`：`crash-point-replay.sh` 文件头写带 gpu 与不带 gpu 的趟都不带 `--test-threads`，「各趟」是对的；参考件改成同一列法，总表登照脚本改 |
| 验证·辩 | 辩 第二节「多机可用」 | 测试自己的配置文件那半句被改写成「不在…之内」，不是原话 | 成立 | `dot-claude/rules/verification.md:67`：原话放回 |
| 验证·辩 | 辩 第二节「变异清单」 | implementation-workflow.md 第 9 行「写进存进仓里的变异清单」两个动词叠在一起 | 成立 | `dot-claude/rules/implementation-workflow.md:9`：改成「写进仓里的变异清单」 |
| 验证·辩 | 辩 第三节第 53 行 | 四道门禁（formal-verification、broad-mutation、qemu-device-streams、lkmm）参考件逐句没人核 | 成立 | `dot-claude/rules/verification-reference.md:165`：这一版逐句对了四道文件头（三-1 到三-6 与这四道各行），能对出的都改了；对完的句子列在 result.md |
| 验证·辩 | 辩 第四节第 1、2、3、5 条 | 门禁与脚本文件头没有 sha256 钉住，行号会漂；cite-check 的 5 处没判；现查只盖了判决 1、2、5；腿的「没打中」清单没逐条交代 | 部分 | 前一条的处置是规则引脚本、门禁、钩子一律引到文件头的哪一段、不写死行号（全树现核，没有 文件:行 形的指针）；后几条是主 agent 核查的做法，不是树里的句子，由压缩 2g 的判决逐条现读 |
| 验证·辩 | 辩 第三节第 51、52、121 行；第二节「上游指针」「三方常驻补指向」「入口一行两跳」 | 不采纳的几行理由站得住；上游指针改括号是用户定案；三方常驻指到技能那一节够得着；入口那一行经「多轮」再指到代码轮两跳读得通 | 不成立 | 辩方腿自己判理由站得住、不是要改的句子；上游指针一条按主 agent 的处置另改回「文件名加节」写法，见「定义」题同类行 |
| 常驻·攻、常驻·辩 | 攻 一-1；辩 第三节判决第 7 条与留着的第 1、2、3 小条 | 43 处上游「节」指针全改成括号写法，只有「射程与例外」一处该改，改完出了门禁 governance-refs ③ 的射程；另有 8 处停在「上游「节」」、一句里半括号半「」的混写 | 成立 | `dot-claude/agent-common.md:20`；`dot-claude/rules/code-discipline-by-review.md:59`；`dot-claude/rules/show-me-test-by-review.md:12`；`dot-claude/rules/writing-discipline-by-review.md:56`：照主 agent 的处置：括号写法 45 处全部改回反引号文件名加「节」（节名逐字取上游今天的标题行，全部核得到），code-discipline-by-review.md 的 8 处「上游「节」」收成同一写法并删掉文件头的「里叫「上游」」括注；上游今天确实指不到的只有「射程与例外」一处，保持第二版的写法（`communication-discipline.md` 后接项目一侧的判据句）；`skills/gate/staged-gate.md` 那一处是原件里就有的描述性括号，不动 |
| 常驻·攻、常驻·辩、矛盾·攻 | 常驻攻 一-2；常驻辩 第二节第 2 行；矛盾攻 格二 D 第 1 件（合稿-5） | 变异清单只补了实验装置一张表，脚本、钩子、门禁的自证没有落点；「实验装置」不加限定读成含入库装置 | 成立 | `dot-claude/rules/show-me-test-by-review.md:7`；`dot-claude/rules/show-me-test-by-review.md:7`：补一句：没有变异表收的测试用弄坏开关证红（照测试脚本规则「测试脚本自己也要被测试证明有效」里原有那一条），commit message 照样写明；括号里「实验装置」写成 `scripts/experiment-apparatus/` 下装置、入库装置的变异行进 `crates/mutations.tsv`（实验员定义第 3 条的写法） |
| 常驻·攻 | 攻 二-1 | 「新测试先证明会红」的判据在 implementation-workflow.md 与 show-me-test-by-review.md 各写全一遍 | 成立 | `dot-claude/rules/implementation-workflow.md:9`：implementation-workflow.md 那一格只留「两样都要」与指针 |
| 常驻·攻、常驻·辩、矛盾·攻 | 常驻攻 二-2；常驻辩 第六节旁记；矛盾攻 用户定-18 | 用户定案第 18 条在门禁技能、staged-gate、commit-stages 三处没落（仍写「（或用户要求时）跑」）；与定案 24 同一件事 | 成立 | `dot-claude/skills/gate/SKILL.md:10`；`dot-claude/skills/gate/staged-gate.md:17`；`dot-claude/skills/crash-test/commit-stages.md:21`：按用户定案 24 一并改：三处各指到 verification.md「提交之外」那一条，不另写一遍；staged-gate 里跑全量工作区那一条不带 `--full-gate`、钩子不弹，写明主 agent 先自己弹窗问一次 |
| 常驻·攻 | 攻 二-3 | 测试脚本规则补回上游 show-me-test 时写成两节，读得出「只受这两节管」 | 成立 | `dot-claude/rules/test-script-discipline.md:4`：照原件管整份上游文件写成「整份」，两节只当括注 |
| 常驻·攻 | 攻 三-1 | staged-gate.md 的残句「（用户要求时 `=user-request`）」（已退役的前缀，原件带来） | 成立 | `dot-claude/skills/gate/staged-gate.md:17`：删掉这个括号 |
| 常驻·攻 | 攻 用户定案第 20、21 条的落点备注 | 命令行里停进程「照旧一次点名一个」与 tooling-writer 第 5 步「自证里只对自己起的进程号发信号」两处读得窄；总表常驻-11 末列「是否放回交用户定」没改成引第 21 条 | 成立 | `dot-claude/agents/tooling-writer.md:33`：归「定义」题同类行一并改（工具员第 5 步照钩子文件头 ⑥ 写清不许的几种与许的那一种）；总表常驻-11 照定案 21 改 |
| 常驻·辩 | 辩 第四节 `:73` | 总表 9 行「只改措辞」只核了几行，常驻-12、常驻-14 其实动了做法（定期回来复检→不轮询；不归的报给用户→主 agent 自己做） | 成立 | `contradictions.md:9`；`contradictions.md:9`：规则文字不动（向共用约束与主 agent 入口靠拢）；总表两行标签改成「改了判据」，并把「只改措辞」的别的几行逐行回读一遍（见 contradictions.md 表头） |
| 常驻·辩 | 辩 第三节第二行（commit-stages、技能 SKILL.md:35 的「前缀」） | crash-test 技能 `SKILL.md:35`「谁在什么时候跑、带什么前缀」同形写法没动 | 不成立 | 那里的「前缀」读成写在 `task submit` 前面的范围声明也说得通，辩方自己判不算新问题；原件同形 |
| 常驻·辩 | 辩 第四节 `:72` `:74` `:75` `:63`、`:117`、`:119`、第五节 | 不采纳的几行（射程多了「自证」、门禁是下限只搜一次、节标题变动 21 个、检出工具可终止放回、欠账号没登、light/heavy 两档）理由站得住；核查缺口（现查只盖了判决 1、2、3、7、数对不上） | 不成立 | 辩方复核理由站得住，没有要改的句子；核查与去向表条数的毛病归主 agent 的判决，这一版的去向表数由脚本数、与行对得上 |
| 定义·攻、定义·辩、矛盾·攻 | 定义攻 A；定义辩 3.3-4、判决 5/6 的修补；矛盾攻 用户定-19 | 调查员「查到就修」改在主工作区，验证命令的 --cwd 与 <根> 却填第 2 步修之前建的副本；实现员第 1 步改成「你的副本」把借用的一路带偏；共用约束命名检查那句括号后接 <快照>，读得出调查员也写 <快照> | 成立 | `dot-claude/agents/investigator.md:28`；`dot-claude/agents/investigator.md:33`；`dot-claude/agents/implementation-writer.md:31`；`dot-claude/agent-common.md:109`：照主 agent 的处置：验证命令写在它改的那棵树上（主工作区仓根）；`prove-red.sh` 文件头把 `--copy` 指主工作区当拒绝项，所以证红的 `--cwd` 与 `--copy` 改完之后从主工作区新建一份带改动的副本；交回闸 `handback-guard.sh` 文件头 ⑤ 认主工作区的根与 `<快照>`（只在改动全落在副本里时），调查员不写 <快照> |
| 定义·攻、定义·辩、矛盾·攻 | 定义攻 B；定义辩 3.2；矛盾攻 合稿-10 与格二 D 第 2 件 | 「规则怎么读」第一条被改成两句：原件「一律整份读」变成「点名的做那一步之前读那一节」，「读那一节，读全」读得出两种；去向表记成用户定的、总表记主 agent 定 | 成立 | `dot-claude/agent-common.md:17`：照主 agent 的处置改回原件原话；总表对应行改标「改回原判据」，不再写「用户定」 |
| 定义·攻、定义·辩、矛盾·攻 | 定义攻 C；定义辩 3.3-1、3.3-2、3.3-3；矛盾攻 用户定-20 与核不动 2 | 用户定案第 20 条在工具员第 5 步落得不全：没写循环逐个发、先挑再发；自证与脚本两句各排掉对方的写法；proc.py 与「只给主 agent」打架；规则没写后台符号 & | 成立 | `dot-claude/agents/tooling-writer.md:33`；`dot-claude/rules/command-safety-by-review.md:8`：工具员第 5 步照钩子文件头 ⑥ 写清不许的几种（别的进程组、一次几个目标、shell 循环逐个发、按名字或 cgroup 或 /proc 挑再发）与许的那一种（点名一个自己起的进程号；对自己用新会话起的那一个子进程的组发，Python 里 `os.killpg` 在循环里每一轮停自己起的那一组也放行）；规则与共用约束同改；`proc.py stop <pid>` 保留原件写法（钩子放行点名一个，直接起它的限制在共用约束「停任务与停进程」） |
| 定义·攻 | 攻 D | 派发闸那一项「写者锁持有者还在跑」写了两遍，一遍带「写者锁已判」放行行、一遍不带 | 成立 | `dot-claude/agent-common.md:104`：删掉后一处 |
| 定义·攻 | 攻 E | 实现员第 3 步仍写「在草稿目录建副本」：一律交补丁之后手里已有改动所在的副本，另建一份没有这次改动 | 成立 | `dot-claude/agents/implementation-writer.md:46`：分两支：实现员在自己的副本里证红，调查员改在主工作区、改完另建带改动的副本 |
| 定义·攻 | 攻 F | 弹窗闸：合稿把出处写成「路径、命令」（钩子只认反引号里的）、断言词写成开放的「这类」（钩子只认九个）、「选项里」比 description 宽 | 成立 | `dot-claude/agent-common.md:104`；`dot-claude/agent-common.md:104`：照 `ask-user-claim-guard.sh` 文件头「判法」那一段改 |
| 定义·攻 | 攻 G | 定时闸只留「除 remind.sh 之外」，项目代码闸把只给主 agent 的工具名单删成代码指针，子 agent 读不出 remind.sh 自己起不了、出路只剩「经 submit 交」 | 成立 | `dot-claude/agent-common.md:102`；`dot-claude/agent-common.md:95`：放回原件的工具名与子 agent 一支的出路；定时闸写明主 agent 与子 agent 一样拒 |
| 定义·攻 | 攻 H | 写闸一项没列钩子第七道里的批次代号；共用约束只把字母代号算批次代号，钩子另拒汉字数字加天干那一类 | 成立 | `dot-claude/agent-common.md:96`；`dot-claude/agent-common.md:39`：照 `write-guard.sh` 文件头「七、会话号与批次代号」改；「第 N 批」纯批号照留 |
| 定义·攻 | 攻 I | 交回正文闸的「书记员」在合稿树里没有所指（钩子里是 kb-scribe） | 成立 | `dot-claude/agent-common.md:103`：改成写回员（`kb-scribe`）与执行员 |
| 定义·攻 | 攻 材料不全 | diff-from-r1.patch 不是第二版相对第一版的全部改动：用户定案第 22 条那一处不在补丁里 | 部分 | 说的是上一版交付物，被判文件本身落对了；这一版的 diff-from-r2.patch 用 `diff -ruN` 对冻结树整棵算，交回前重跑并核「打到第二版上得到第三版」 |
| 定义·辩 | 辩 第二节判决 3、4 | 旧数据复跑那句只剩两跳（触发句换了）；丢了「写成交接的副本不列」 | 不成立 | `dot-claude/agent-common.md:24`；`dot-claude/agent-common.md:100`：辩方复核两条第二版都放回了、与原件触发条件一致；没有要改的句子 |
| 定义·辩 | 辩 第五节第 3 点抽样 | 9 行抽样里发现四处删减：三方攻方「由用户决定的动作」括注里的小节指针、三方辩方「对不上就拒交回」、实现员补丁里「合入后验证认这几张登记表」、执行员停机条款「写进报告，主 agent 跑」 | 成立 | `dot-claude/agents/three-way-attack.md:30`；`dot-claude/agents/three-way-defense.md:33`；`dot-claude/agents/implementation-writer.md:70`；`dot-claude/agents/experiment-runner.md:42`：四处都按原件原话放回（原件一句话里带一条做法或一个指针，辩方判说明性，放回无损） |
| 定义·辩 | 辩 第四节第 95、96、91、98、99、119 行 | 不采纳的行：项目代码闸等四段没核完；quote-rust-items 钩子库还没加；四条 --item 命令整条删成指针；说明句 3 到 5；指针表 E 类；执行员「卡住」去向表没跟着改 | 部分 | 95 行：这一版把攻方点名的弹窗闸、定时闸、项目代码闸、写闸、交回正文闸五段逐句对了文件头（攻 F 到 I），其余没有被点名的段仍只有指针级核对；96 行是脚本一侧的活（主 agent 的批次）；91、98、99 理由站得住；119 行树里已落、记录里的行已由新去向表替换 |
| 定义·辩 | 辩 第五节第 1、2 点 | 判决没自己跑 cite-check.py（它是直接跑的文本工具、不该拿服务排队当理由）；倒推树的替代理由没写全 | 部分 | 主 agent 核查的做法，不是树里的句子；cite-check 已由主 agent 补跑（调度记录第二轮那一段），树不改 |
| 矛盾·攻、矛盾·辩 | 攻 用户定-20、常驻-10；辩 第二节第 3 条备注、第三节常驻-10 | 总表标签：用户定-20 的工具员第 5 步与共用约束 ⑥ 放行行是新判据，不是「改回原判据」；常驻-10 标「只改措辞」而规则比原件多要求 `--wait`、比钩子认的写法严 | 成立 | `contradictions.md:9`；`contradictions.md:9`；`dot-claude/rules/command-safety-by-review.md:8`：总表重出时标签照实更正：用户定-20 拆成两行（规则与共用约束样本进程那一条「改回原判据」，工具员第 5 步与 ⑥ 放行行「改了判据（用户定 20，与钩子文件头 ⑥ 一致）」）；常驻-10 改标「照脚本改」，规则写成钩子认的 `setsid [--wait] <命令> &`（不带 `-f`），同一层紧跟 `名字=$!` |
| 矛盾·攻、矛盾·辩 | 攻 格二 A；辩 第二节第 5 条 | 用户定案第 21、22 条没进总表；第 22 条那一处（共用约束「或卡住」）不在 diff-from-r1.patch 里；常驻-11 说明栏还写「是否放回交用户定」 | 成立 | `contradictions.md:160`；`contradictions.md:161`：「用户定」一节收编号 1 到 26，撤回的标撤回；常驻-11 说明引用户定 21；补丁这一版由 `diff -ruN` 对冻结树整棵出，不再有补丁与树对不上的一处 |
| 矛盾·攻、矛盾·辩 | 攻 格二 B；辩 第二节第 6 条（a）（b）（d） | 总表 9 处落点行号随修补偏了一到四行，6 行把原件栏与合稿栏写反或指到第二版里已不在的原文；表头写「核过、problems=0」对第二版不成立 | 成立 | `contradictions.md:5`：总表整张重出，两处原话（仓里原件 文件:行）与合稿落点（third/ 的 文件:行）由脚本逐字回查，查不到的改到查得到；表头写这一次回查的命令与结果 |
| 矛盾·攻、矛盾·辩 | 攻 格二 C；辩 第二节第 6 条（c） | 总表 4 到 23 处〔〕里写的是转述或用「…」拼接，不是原文（合稿-1、合稿-6、合稿-9、合稿-10、用户定-2、用户定-4、主 agent 处置-6） | 成立 | `contradictions.md:5`：引号里只放逐字原文：脚本逐条查片段在所引行里，查不到的在原件或合稿那一行里重选一段逐字片段 |
| 矛盾·攻 | 攻 格二 D 第 3 件 | 常驻-12 并进来的半句「任务卡住由服务判」紧接「不拦命令、不停进程与子 agent」，括号只点 hook 与看门狗，读成举例会把服务也罩进来，而服务判出卡住是停的 | 成立 | `dot-claude/rules/command-safety-by-review.md:17`：在那半句后补一句：服务判出卡住是停它的，不在「检出」之列，判法指到 `task_server_lib.py` 文件头「错误类型」表的 `stalled` 一行 |
| 矛盾·攻、矛盾·辩 | 攻 格二 B 末段；辩 第二节第 5 条共用-10 | 钩子文件头 handback-guard.sh 仍写「共用约束写明 rsync 拷的副本 <根> 填仓根」，共用约束里没有这一句（原件与第二版都没有） | 部分 | 脚本一侧的文字（归脚本那一批）；规则一侧不写这一句；总表照旧交主 agent |
| 矛盾·辩 | 辩 第三节共用-6 备注、第二节常驻-8 残留二 | `quote-rust-items.py` 钩子名单里 0 命中而材料员定义写着直接跑；验证-5 的「经服务交的命令里不许写」比服务拒绝名单宽（原件同样） | 不成立 | 前一条是用户定案第 10 条排在脚本那一批的活，规则文字不改；后一条原件 `.claude/rules/implementation-workflow.md` 就这样写，不是压缩带来的 |
| 矛盾·辩 | 辩 第四节第 1 到 4 点 | 判决里没有主 agent 自己跑的 cite-check 与快照倒推两条命令；81 条没有一条经主 agent 现读；两条腿共用同一个前提（总表登记的引文对）；判决第 6 条没写是哪几条 | 部分 | 主 agent 核查做法，不是树里的句子；这一版总表由脚本回查并交主 agent，主 agent 判决时逐条现读（压缩 2g） |
| 矛盾·攻、矛盾·辩 | 攻 各格判定一览（入口-4、合稿-1、合稿-4 到 12、用户定-13 到 18 等）；辩 第一节、第三节 24 条抽查 | 点名的 24 条与抽查的 24 条逐条判「改得对」或「没动做法」 | 不成立 | 两条腿判改得对，没有要改的句子；其中二义与残留已按上面各行落实 |
