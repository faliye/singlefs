# landing-audit-r6 辩方腿报告（K1：未定项清单的完整性，2026-10-23）

正文：`research/prompts/_landing-audit-r6-body.md` 节「三、格」K1。立场：替「清单收全了、每行都能直接派」找站得住的理由，找不到的逐条列出。不写 K2、K3。

## 复跑

模型目录：`research/prompts/landing-audit-r6-sonnet-model/`。整条复跑命令（工作目录随意，`--work` 给新的空目录）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/landing-audit-r6-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新的草稿目录> --model /home/fy5090/code/singlefs/research/prompts/landing-audit-r6-sonnet-model
```

- 包装下的那一趟：我起过一次，内存包装排队（`slice 已占 40.1 GiB`，本机上主 agent 的 checker 档崩溃点重放占着），等了 120 秒，我自己停掉排队的进程（`proc.py stop`），包装退 251（「带内存上限的 scope 起不来：排队那一步退 143」），命令一行没跑。这一次的输出不算结果。
- 之后没绕开：改成直接 `bash research/prompts/landing-audit-r6-sonnet-model/tools/rerun.sh …`（不经包装；两步只起 grep、cmp 和 python3，不起编译出来的代码），跑两遍（`rerun-a`、`rerun-b`），第二遍两份输出与模型目录 `out/` 逐字节相同（`name=destinations … 逐字节相同`、`name=overlap_targets … 逐字节相同`，退 0）。要包装下复跑的，写进末尾「要主 agent 跑的命令」。
- 欠账表 `.claude/kb/checks-owed.md` 是现读的，别的会话改它时 `destinations.out` 最后一列会变，别的列不变。

sha256（`sha256sum`，相对模型目录）：

| 文件 | sha256 |
|---|---|
| inputs/dispatch-record.md（开工时的调度记录整份拷贝） | bd98c0f8c49ded78597a5945484745aa063376d5fd46c536227c6cd93035e112 |
| inputs/proposal-section-90.txt（拆分提案第 1466 到 1505 行，九十节 ①–㉝） | c95dce39c0b5714237750dd4ee6c261b01104241ce964cd8645fed7369c340c7 |
| probes/destinations.sh | 2b640f5dc9aa6945b7d5b1fa167321565b3efd525969eff2792e36e1aa9e9b52 |
| probes/overlap_targets.py | 484b951906d638a68a6a1b77c47ade4fea1cf88058ad84f043b8b327c72003f5 |
| tools/rerun.sh | d9981eda4b38dd99ee51bc6d1698565e7e189a4da50db2f956c486cfd2b40761 |
| out/destinations.out | ca0f56b2ee0be83b86af4b4bda9417760b885623418bd1640db0596524bc9223 |
| out/overlap_targets.out | 727d5e5cae9a2a228e2f4b45a34ab2ca81f62478ed321760387e415a27f7a6a3 |

## 读法

- 「去向」只认三处加两处：九十节某一行（`records/2026-09-16-subagent拆分提案.md` 第 1466 到 1505 行）、调度记录某一行（`records/2026-10-22-子agent结果落地审计.md`）、判决某一条（`research/prompts/landing-audit-r4-main-verification.md`、`landing-audit-r5-main-verification.md`）；再加欠账表与 `.claude/gate.d/` 下的待办表。字符串出现在原出处自己那一行里的不算去向。
- 去向格第一个字是判：「有」找到了；「半」找到一部分或去向列已过期；「无」没有去向。全文数条用这三个字。
- 「无」的依据主要是 `out/destinations.out`：每一项取几个只会出现在去向里的关键字符串，数它们在九十节、调度记录、两份判决、r6 正文、欠账表里各出现几次（行号 N01 起）。0 命中只说明字符串没出现，不说明没有人用别的写法提过；推翻条件一栏写了怎样的行会推翻。
- 文件名不带目录的：`b9-*.md`、`b10-*.md`、`b11-*.md` 在 `research/prompts/subagent-landing-audit-2026-10-22/`；`landing-audit-r4-*`、`landing-audit-r5-*` 在 `research/prompts/`；`records/` 下两份写全名。
- 被判改动里混有别的会话的改动（正文节「二」）：本报告点到 `bash-command-detector.sh` 的 ⑫、`heavy-test-guard.sh` 文件头的地方都标了是别人的。

## 表一　两份判决与四份腿报告里的「没核」「没做」「留给下一轮」

PRO = `records/2026-09-16-subagent拆分提案.md`（九十节），REC = `records/2026-10-22-子agent结果落地审计.md`，r4v、r5v = 两份判决。

| 编号 | 出处 | 去向 | 依据 | 推翻条件 |
|---|---|---|---|---|
| 1.01 | `landing-audit-r4-main-verification.md:77` 三道样本档读数没重跑 | 有：判决自己写「不引」；整道样本档归提交前（REC:264）。code-tooling 那一道后来主 agent 跑了 97 个用例（REC:282），doc-decisions、doc-experiments 两道没人记过重跑 | `grep -n 'doc-decisions 53' landing-audit-r4-main-verification.md` 命中第 77 行；REC:282 查到「97 个用例」 | 提交时 stage-selftest 对这两道判错而没人事先知道 |
| 1.02 | `landing-audit-r4-main-verification.md:78` 攻方没扫真会话记录与真提示语料 | 有：`landing-audit-r5-opus-output.md:292` 全数 3343 份，读数在 `landing-audit-r5-opus-output.md:299` 起的表里 | 同一个数在 r5v:23 复跑核过（「抽了 3343 份」） | 复跑数与腿报告不同（r5v:23 说只差腿自己那一次交回） |
| 1.03 | ⑭ 只读过代码：`landing-audit-r4-main-verification.md:61`、`landing-audit-r5-main-verification.md:35`、`:78`、`landing-audit-r5-sonnet-output.md:129`、`landing-audit-r4-sonnet-output.md:423`、`landing-audit-r4-opus-output.md:302` | 无：r5 判决写「第三轮点名量」，r6 正文 K1 到 K3 一处都没点名 ⑭，九十节 ⑭ 一行的去向列只写「照报，没罩」，调度记录没有它的后续 | `out/destinations.out` N01：九十节 1（⑭ 那一行自己）、r6 正文 0、欠账表 0；`grep -c '⑭' _landing-audit-r6-body.md` 为 0 | r6 正文、攻方提示或哪份提示里点名了看门狗 ⑭ 的自证要量 |
| 1.04 | `landing-audit-r5-main-verification.md:77` 辩方模型整条没复跑 | 有：r5v:25 已写辩方独有的数不引成依据 | 同一判决第 25 行 | 以后有引用辩方独有数的地方 |
| 1.05 | `landing-audit-r5-main-verification.md:79` 攻方第 284 行「与改前不逐条相同的另两处」没核（单独的 `\r` 与坏字节判定在守卫判定之前） | 无 | `landing-audit-r5-opus-output.md:284` 是原话；N17 两个关键字在九十节、调度记录、两份判决、r6 正文里都是 0 | 九十节或调度记录里有一行讲这两处 |
| 1.06 | 「记没打中」：`landing-audit-r4-main-verification.md:52`（4 处）、`landing-audit-r5-main-verification.md:53`（3 处） | 半：r4 的 4 处在 r5 里换角度复核过（`landing-audit-r5-sonnet-output.md:99` 到 `:131`）；r5 的 3 处没有后续，第三轮只攻已定项，按三轮规则这就是最后一次 | r5v:53 原话「记没打中；单条腿一次」；r6 正文节「三」没有它 | 这三处（`landing-audit-r5-opus-output.md:357` 到 `:364`）里有一处在今天的实现上打得中 |
| 1.07 | ⑯ 前一半：`landing-audit-r5-main-verification.md:33` 把第一轮判决「这一轮派发坐实」改成靠先例 | 半：r5 判决表里改了；r4v:36、r4v:66 原文没动；PRO:1487 的去向列仍写「加键之后还没真派过一次三方腿，landing-audit-r4 开轮时顺带核」，这件事 r4、r5 已经了结 | 拆分提案第 1487 行 `grep -c '开轮时顺带核'` 为 1；r4v:66 仍含「这一轮派发坐实」 | 主 agent 认为判决原文不改、九十节去向列也不用改 |
| 1.08 | ⑯ 后一半：`landing-audit-r5-main-verification.md:34`「全部定义类型」措辞更正 | 半：r5 表里更正；r4v:36 原文仍是旧措辞 | `sed -n 36p landing-audit-r4-main-verification.md` 含「全部定义类型」 | 同 1.07 |
| 1.09 | `landing-audit-r5-main-verification.md:44`、`:66` ⑳ 收严的代价 402 份 | 有：PRO:1491 去向列写明 | PRO 第 1491 行含「402 份」（命令数 1） | — |
| 1.10 | `landing-audit-r5-main-verification.md:67` ㉜ 六处 | 半：做了（b11-guards.md、b11-watch-wording.md、b11-worktree-reports.md），PRO:1503 去向列仍只写「第十一批」，没写做了 | `sed -n 1503p` 去向格只有「第十一批」；对照 REC:300 到 REC:303 | 主 agent 把「第十一批」当作已做的写法 |
| 1.11 | `landing-audit-r5-main-verification.md:68` 轻三条与 ㉙ 里 ⑬ 删掉 | 有：PRO:1504（㉝）、PRO:1500（㉙ 括注里写了 ⑬ 判没有对象，删掉） | 两行现查 | — |
| 1.12 | `landing-audit-r5-main-verification.md:52` ㉛ 严格解码不采 | 半：判决没问题，但 PRO:1502 去向列仍写「下一批工具改动」，㉛ 已在第十批做完 | `grep -n 'replay-output-strict-decode' .claude/gate.d/code-tooling.sh` 命中第 149 行；`ls .claude/gate.d/fixtures/code-tooling.sh/` 有 `break-switch-replay-undecodable-red`；`grep -n 'refuse_break_outside_selftest' scripts/agents/subagent_transcript.py` 命中；REC:281 | 有人指出 ㉛ 还有一半没做（唯一候选是 ㉝ 的第三句，见 2.14） |
| 1.13 | `landing-audit-r4-main-verification.md:67` 八条修补 | 有：PRO:1499（㉘）写了做完与进 r5 | 现查 | — |
| 1.14 | `landing-audit-r4-main-verification.md:68` 派发前撞上的两处 | 有：PRO:1500（㉙），仍开着，两半今天都没做（见 2.15） | `grep -c '禁读' .claude/hooks/runner-dispatch-guard.sh` 的命中全是别的用途；`scripts/kb/replace-once.py`、`insert-row.py`、`replace-batch.py` 里 `grep -c '会话名'` 为 0 | 钩子或脚本里出现判禁读清单自己报告路径、判会话名的代码 |
| 1.15 | `landing-audit-r4-main-verification.md:62` merge-and-verify 闭包，连带 `b9-defs-verify.md:400` 带 `--copy`、`--peer` 报本次未跑 | 半：闭包那一半判决采为观测；`--copy`、`--peer` 那一半只在 `landing-audit-r4-sonnet-output.md:369`（设计如此）与 `landing-audit-r4-opus-output.md:283` 里提过，判决没写；脚本文件头第 108 行写明了这两种本次未跑 | `grep -n -e '--peer' scripts/gate/merge-and-verify.sh` 命中第 108 行 | 有人要求带 `--peer` 也跑这一步 |
| 1.16 | `landing-audit-r5-main-verification.md:69` 方法上的一条（收严先拿真记录量） | 有：PRO:1503 形态列 | PRO 第 1503 行含「先拿真会话记录量改前改后」 | — |
| 1.17 | `landing-audit-r4-sonnet-output.md:423` ⑳ 的 27 份「交付：报告」真提示没验 | 有：`b9-dispatch-guard.md:152` 起整节在 3317 份真提示上比了改前改后 | 该节标题行；PRO:1491 现象列也引了 27 份 | 27 份里有一份在今天的派发闸下判错 |
| 1.18 | 账表三方腿桶改读登记后桶变了多少没量：`landing-audit-r4-sonnet-output.md:391`、`b9-leg-types.md:333`、REC:243 | 无 | N02：九十节 0、r4 判决 0、r5 判决 0、r6 正文 0 | `agent-cost.py outcome` 在两版上跑一次，桶差已写进某份记录 |
| 1.19 | ㉑ 实测没覆盖的几种：混合大小写、前缀后紧跟字母、`.MD`、`.markdown`、别的目录、Edit 建文件、Bash 写（`b9-prefix-probe.md:38`、`:39`、`b9-leg-types.md:334`、`landing-audit-r4-sonnet-output.md:414`、REC:231） | 无；而且 PRO:1492 写成「不分大小写都被 Write 拒」，实测只做了全大写与全小写 | N03：混合大小写 在九十节 0、判决 0、r6 正文 0，`summaryx` 全 0；`b9-prefix-probe.md:34` 推翻条件一句自己写了「本轮没测混写和别的目录」 | 有人测过 `Summary.md` 并记下 |
| 1.20 | `landing-audit-r5-sonnet-output.md:234` 60 份旧拒新放的提示当时的真实后果没核 | 半：价（402 份）记进 PRO:1491；后果没法核（当时没有今天的派发闸），腿自己说了 | 同一行原话 | — |
| 1.21 | `landing-audit-r5-sonnet-output.md:236` 同一会话里热加载 frontmatter 没验 | 有：靠先例，r5v:33 | r5v:33 | 新开一个会话派三方腿时 `leg:` 键读不到 |
| 1.22 | `landing-audit-r5-opus-output.md:381`、`:382` 没跑 break-switch-replay 真仓读数、没重跑 `subagent_transcript.py` 三处调用方自证 | 有：REC:282（四格）、REC:281（`subagent_transcript.py --selftest` 查了 13 项）、REC:305 | 三行现查 | 三格里有一格后来红了 |
| 1.23 | `landing-audit-r5-opus-output.md:315` 起 K3-2a（三处草稿目录认法） | 有：PRO:1501（㉚）；看门狗那处并进 PRO:1503 | 两行现查；b11-watch-wording.md 已做看门狗那处 | — |
| 1.24 | `landing-audit-r5-opus-output.md:328` 起 K3-2b（开关值漏进环境改判续做闸） | 有，而且登记了两处：PRO:1504（㉝）与 `.claude/gate.d/break-switch-outside-selftest-pending.tsv:6`（continuation-guard.sh）加 `:7`（handback-guard.sh） | `grep -n 'continuation-guard' .claude/gate.d/break-switch-outside-selftest-pending.tsv` 命中第 6 行 | — |

## 表二　第九批交回报告（b9-*.md）里的「没做」「没核」「留给」

| 编号 | 出处 | 去向 | 依据 | 推翻条件 |
|---|---|---|---|---|
| 2.01 | 各份交回的头一条：没跑重型测试、门禁阶段、各类 lint、定义三方，没提交（每份「没做什么」都有，14 份） | 有：`.claude/agent-common.md:98` 写明扫仓的门禁阶段与整轮门禁归主 agent；REC:264 的收拢表「提交前」一行 | `grep -n '扫仓的门禁阶段（本地与共享）' .claude/agent-common.md` 命中第 98 行 | 提交前整轮门禁在这一批的文件上红出一格没人事先知道 |
| 2.02 | `b9-defs-verify.md:395` 合入后验证新加的 checker 档快档那一步在真仓上没真跑过，要等下一趟带前缀合入 | 无：调度记录没有一行写这一步第一次真跑要看什么（红了怎么记、本次未跑怎么记）；`landing-audit-r4-main-verification.md:62` 只说闭包判法本仓不落空 | `awk 'NR>=248' records/2026-10-22-子agent结果落地审计.md \| grep -c '真跑'` 输出 2，两处都是命名检查「真跑完」（第 279 行与第 287 行），不是合入后验证的真跑 | 调度记录或 `.claude/main-agent.md:52` 写了第一次真跑的验收 |
| 2.03 | `b9-defs-verify.md:397` `.claude/hooks/selftest-registry.tsv` 里 `scripts/gate/merge-and-verify.sh` 一行的理由没提新格在样本根上真起那一道 | 无 | `sed -n 110p .claude/hooks/selftest-registry.tsv` 理由仍只写假 cargo 与 `--peer`；N11 九十节命中 1 处是 ⑫ 一行讲 `lib-lag-baseline.py` 的，不是这一件 | 有人说档位没变所以理由不用补（那也是去向，要写下来） |
| 2.04 | `b9-defs-verify.md:383` 快档那一步编 checker 档包与别的几步共用 `--memory`，建议照 `.claude/skills/crash-test/commit-stages.md` 给的上限（推的，没量过） | 无 | N12：九十节 0、记录 0、判决 0；`scripts/gate/merge-and-verify.sh:661` 现查用的是 `$MEMORY` | 第一次真跑量出峰值并记下 |
| 2.05 | `b9-defs-verify.md:399` 到 `:403` 留给定义三方的几点：第 3 条没写例外、`--copy` `--peer` 本次未跑、闭包外读文件、重跑命令不是逐条 `--exact`、`records/` 下 `.txt` | 有：第一点 PRO:1499（㉘）；`--copy` `--peer` 见 1.15；闭包外 `landing-audit-r4-main-verification.md:62`；重跑命令 PRO:1503（㉜）；`.txt` 见 `landing-audit-r5-sonnet-output.md:103` | 各行现查 | `.txt` 一点以提交时门禁 doc-text 的结果为准（`landing-audit-r5-sonnet-output.md:107`） |
| 2.06 | `b9-defs-verify.md:404` `WORKSPACE_LOCK_BREAK=trust-dead-holder` 的文件头点名本来就不全 | 无 | N13：`trust-dead-holder` 九十节 0、记录 0、判决 0、正文 0 | 九十节已有一行把这一类统一收（建议见 4.13） |
| 2.07 | `b9-defs-verify.md:396` `write-guard.sh` 自证红一格 | 有：REC:246 主 agent 照规格换上，自证 229 种退 0 | REC:246 原话 | — |
| 2.08 | `b9-dispatch-guard.md:263` 三处推的口子没改 | 有：PRO:1498（㉗） | 现查 | — |
| 2.09 | `b9-dispatch-guard.md:264` 跨行的「」引文没处理（引号按行配对） | 无：PRO:1490（⑲）现象格里有「跨行」二字，是 `agent-cost.py` 报告标签那一件的描述，去向格只写主会话按时刻筛；`r2fix-hooks.md:278` 是同一条的原始出处 | N05：九十节 1（现象格）、记录 0、判决 0、正文 0 | 有一行去向格点名了派发闸的跨行引号 |
| 2.10 | `b9-dispatch-guard.md:240` 与 `b9-leg-types.md:336`：`scripts/agents/dispatch-prompt.py` 文件头第 38、39 行还写派发闸 ⑰ 对三方腿「报告行点名的文件不是 …或没有报告行就拒」，现在判法是整份提示里一处这一形态都没有才拒 | 无 | `sed -n 38,39p scripts/agents/dispatch-prompt.py` 仍是旧句；N04 两个关键字全 0 | 有一行去向格列了这句文字过期 |
| 2.11 | `b9-dispatch-guard.md:266` `disable-check` 点名只列了 299 格里的 70 格 | 有：PRO:1498（㉗）；同类「点名不全」另有 2.06、2.33、3.13 三处，只有这一条被登记 | 现查 | — |
| 2.12 | `b9-dispatch-guard.md:267` 没回原会话记录逐份核那 3 份是不是转述 | 半：PRO:1498（㉗）把它写成推的、没量过误放的后果 | N06 之外无命中；PRO 第 1498 行含「3 份真提示只靠这一认法放行」 | 回原会话核出 3 份里有放错的 |
| 2.13 | `b9-gates-a.md:224` 到 `:227` 没核动的几项（别的样本、真仓上四格、新格对 gate-structure 等三格） | 有：REC:264 提交前一行；REC:212 提交前整轮；没有逐项清单 | 两行现查 | 提交时这几格里有一格红 |
| 2.14 | `b9-gates-a.md:233` `lib-lag-baseline.py` 文件头「调它的九处」没补两处 | 半：PRO:1497（㉖）；今天已补 | `grep -n 'layout-checker-sync' .claude/gate.d/lib-lag-baseline.py` 命中第 33 行；PRO 第 1497 行仍把它列为待做 | — |
| 2.15 | `b9-gates-a.md:234` `heavy-test-guard.sh` 文件头自证一段没补顶层 `lib-` 库 | 半：PRO:1497（㉖）；今天文件头第 159 行已写（这份钩子别的会话也在改，补的是谁没查） | `grep -n 'GATE_TOP_LIBRARY_PREFIX' .claude/hooks/heavy-test-guard.sh` 命中第 159 行 | — |
| 2.16 | `b9-gates-a.md:235` 门禁 doc-registries 另四格（audit-contradictions、declared-counts、invariant-anchors、invariant-prerequisites）读不出 kb 目录登记位时仍当空的 | 有：PRO:1497（㉖），今天仍开着 | `awk '/kb_candidate_roots > /{print NR}' .claude/gate.d/doc-registries.sh` 只有第 3484、3714 行（invariant-count-elsewhere 与 inline-sums 两格）；另四格走 `done < <(kb_candidate_roots)` 进程替换，失败吞掉 | 另四格在登记位坏掉时红 |
| 2.17 | `b9-gates-a.md:236` `.claude/gate.d/lib-forbidden-notations.py` 本身不实现 `--selftest`，带它起会被判表外自证 | 半：PRO:1497（㉖）把它写成「带 `--selftest` 而没登记」，要「照自证正文判档登记」——这份库没有自证正文可判 | `grep -c 'argv\|__main__\|def main' .claude/gate.d/lib-forbidden-notations.py` 为 0；对它起 `--selftest` 被 `heavy-test-guard.sh` 拒「没在登记表里」 | 库里有自证入口（那就该登记） |
| 2.18 | `b9-gates-a.md:237` `.claude/batch-scope` 没登记这一批文件 | 有：REC:212「提交前整轮（… `.claude/batch-scope` 登记）」 | 现查 | — |
| 2.19 | `b9-gates-b.md:269` `scripts/kb/claim-experiment.sh` 没加进 `.claude/gate.d/break-switch-replay-files.tsv` | 无 | `grep -c 'claim-experiment' .claude/gate.d/break-switch-replay-files.tsv` 为 0；N15 `claim-experiment` 九十节 1 处是 ⑨ 一行描述，不是复跑表 | 有一行写了复跑表不收它的理由 |
| 2.20 | `b9-gates-b.md:272` `CLAIM_SKIP_USED_CHECK`、`CLAIM_SKIP_WORKTREES` 不是 `_BREAK` 名字，平常跑设了不拒 | 无：REC:133 提到这两个开关的自证红，不是这一件 | N15：`CLAIM_SKIP` 九十节 0、判决 0、正文 0 | 门禁 break-switch-outside-selftest 的待办表里有它 |
| 2.21 | `b9-gates-b.md:270` 不登记的比对对象点名行没立只缩不涨 | 有：PRO:1497（㉖）；今天这批点名行从 5 行涨到 8 行（见表四），正是这一条要拦的 | 表四命令 | — |
| 2.22 | `b9-gates-b.md:271` `.claude/gate.d/stage-owners.tsv` 里 replay-comparison-units 一行不含设计员，设计员写了固定形态的表后到执行员登记前这一格红 | 无 | `grep -n 'replay-comparison-units' .claude/gate.d/stage-owners.tsv` 第 107 行归属是 experiment-runner,implementation-writer；N16 九十节 1 处是 ② 一行 | 设计员定义里写了「报告里写明」就是处置（交回里说定义已写，没人判过） |
| 2.23 | `b9-gates-b.md:273`、`:274` ㉔ 二元键已是四元键、目录登记位置 | 有：PRO:1495（㉔）；`landing-audit-r5-sonnet-output.md:117` 判位置站得住 | 现查 | — |
| 2.24 | `b9-gates-b.md` 里工具员在 Bash 用 python 写了一份仓内样本文件（钩子当时没拦） | 有：共用约束 ⑧（`.claude/agent-common.md:81`），REC:234 记了 | 现查 | tooling-writer 在 Bash 里 python 写仓内文件今天仍放行 |
| 2.25 | `b9-handback-guard.md:217` 到 `:220`：交回闸 ⑤ 不认 Bash 里 mv、cp、重定向写的 `.rs`；命令替换的根；改完被删的 worktree 副本 | 半：前两件 PRO:1496（㉕）；第三件只写在 `.claude/hooks/handback-guard.sh:44` 文件头，没有处置 | `sed -n 44p .claude/hooks/handback-guard.sh` | 被删的 worktree 副本里改 `.rs` 的交回在真记录里出现 |
| 2.26 | `b9-handback-guard.md:221`、`:222`：检出钩子 ⑰ 的 `-p`、`--tmpdir`；共用约束补两样 | 有：PRO:1496（㉕）、REC:246（共用约束已补） | 现查 | — |
| 2.27 | `b9-handback-guard.md:223` `.claude/hooks/lib_writer_lock.py` 文件头「两处用这一份」；它的弄坏开关对检出钩子哪几格有影响没逐条核 | 半：文件头 PRO:1496（㉕）有；弄坏开关没逐条核无 | `sed -n 3p .claude/hooks/lib_writer_lock.py` 仍是「两处用这一份」；N18 | 开关核过并记下 |
| 2.28 | `b9-leg-types.md:332`、`landing-audit-r4-sonnet-output.md:423` 加 `leg:` 键后没真派一次 | 半：见 1.07（去向列过期） | 同 1.07 | — |
| 2.29 | `b9-leg-types.md:335` ㉑ 新拒 investigator 的 `findings.md` 要不要放宽 | 有：PRO:1492 写「主 agent 判不放宽」 | 现查 | 真记录里这类类型被误拒的份数涨 |
| 2.30 | `b9-prefix-probe.md:38`、`:39`、`b9-leg-types.md:334` | 见 1.19（无） | — | — |
| 2.31 | `b9-watch-cost.md:303` 到 `:309`：`scripts/gate/hook-swap.py` 只收 `.claude/hooks/` 一层，`scripts/agents/` 下改动靠手工「基线到草稿的差定点换」 | 无：本批任务 29（`agent-watch.py`、`agent-cost.py`）与第十一批任务 36（`agent-watch.py`）都走了手工协议 | `grep -nF '只收 <仓根>/.claude/hooks/ 这一层里已有的普通文件' scripts/gate/hook-swap.py` 命中第 13 行；九十节 hook-swap 0 处 | 有行写了「scripts 下改动走 replace-batch 即可」 |
| 2.32 | `b9-watch-cost.md:310` 没碰 `scripts/agents/watch.sh` 键表以外部分，自证不核新键能转过去 | 无 | 同上 N 系列无对应关键字 | `watch.sh --selftest` 加了这一格 |
| 2.33 | `b9-watch-cost.md:313` 开关表里原有几条点名不全（`memoryscan` 等） | 无 | N13：`memoryscan` 全 0 | 同 2.06 |
| 2.34 | `b9-watch-cost.md:311` `selftest-registry.tsv` 里 agent-watch.py 一行理由过时、`:314` ㉒ 定义一侧 | 有：REC:240 并进任务 30，`b9-leg-types.md:244` 起「5. ③ 登记表一行理由」做了；㉒ 定义一侧 REC:246 | 现查 | — |

## 表三　第十、十一批交回报告（b10-*.md、b11-*.md）与调度记录第十、十一批里的「没做」「没核」「留给」

| 编号 | 出处 | 去向 | 依据 | 推翻条件 |
|---|---|---|---|---|
| 3.01 | `b10-defs-sync.md:148` `.claude/hooks/heavy-test-guard.sh` 主 agent 被拒时的「照抄」一行没有例外 | 有：REC:278 主 agent 自己加了 `merge_verify_hint`；r5v:8 把「两处拒绝说明」列进被判过的文件 | REC:278 原话「进 landing-audit-r5」；r5v:8 | r5 之后有人改回 |
| 3.02 | 验证段第 1 步命令本身没带前缀、正文也没写只重跑红格的命令怎么跑：`b10-defs-sync.md:98`、`:151`，`b11-watch-wording.md:171`，REC:278 末句「留给 r5 判」 | 无：r5 判决一次没提 | N10：「验证段第 1 步」r5 判决 0 处、九十节 0 处、r6 正文 0 处（r4 判决 1 处是列被判文件）；`sed -n 52p .claude/main-agent.md` 的命令代码段是 `bash scripts/gate/merge-and-verify.sh [--skip-apply] [--memory 上限] [--threads N] <补丁目录…>`，第 7 行与第 96 行的同一条命令带了 `SINGLEFS_HEAVY_TESTS=user-request` 前缀，第 52 行的没带（三行各 `grep -c` 得 1、1、0） | 第 52 行的写法被判定为「前缀写在同一步正文里就够」并写进判决 |
| 3.03 | `b10-defs-sync.md:158` 到 `:161` 改了三份定义类文件，门禁 agent-def-adversarial-review 要一轮三方点名 | 有：r4、r5 判决的「这一轮被判过的文件」一节；第十一批又改了这三份，r6 正文节「一」表已列。r6 判决要再写一遍这一节 | 现查 r6 正文 | r6 判决漏列其中一份 |
| 3.04 | `b10-hooks.md:331` 到 `:333` 两处门禁登记（`SELFTEST_RUNNERS`、`break-switch-replay-files.tsv`）工具员没权限写 | 有：REC:279 主 agent 补 | 现查 | — |
| 3.05 | `b10-hooks.md:345` 到 `:348` 另三种「草稿目录：」认法 | 有：PRO:1501（㉚）；看门狗那处第十一批做完（`b11-watch-wording.md:170` 写明另两处留下一批） | 现查 | — |
| 3.06 | `b10-hooks.md:349` 到 `:353` 交回闸 ⑤ 认不出的写法：`bash -c` 只收一层、`( cd X && 两条 )` 子 shell 不收、写在 `$(…)` 里的不收、`set -e` 下前面失败后面不跑（都是严的一边，会误拒真跑完的） | 半：只写在 `.claude/hooks/handback-guard.sh:45` 文件头当限度；没有一行说改还是不改 | `sed -n 45p .claude/hooks/handback-guard.sh` 命中；N07 `bash -c`、`set -e` 九十节 0、记录 0、判决 0；欠账表命中是别的事 | 真交回里出现这几种形状被误拒 |
| 3.07 | `b10-hooks.md:354` ⑯ 收严后行首带列表记号、粗体的「草稿目录：」不再建写者锁（推的，没量过） | 半：PRO:1501（㉚）形态格还写「⑯ 收严的影响拿主会话记录里全部派发提示量一遍」；这件事 r5 攻方已量（76 份少建锁，列表 66 份、粗体 11 份，全在 2026-10-14 之前，`landing-audit-r5-opus-output.md:310`、`:311`），九十节没跟着改 | 两处现查 | r5 攻方的数复不出（那是正文 K3 的活） |
| 3.08 | `b10-hooks.md:355`、`:356`、`:357` a9717 类不再拦；交回闸一侧没扫真交回写法；复跑只跑了 30 条 | 有：PRO:1491（⑳）；`landing-audit-r5-opus-output.md:299` 起 719 次真交回；REC:282、REC:305 | 现查 | — |
| 3.09 | `b10-replay-decode.md:191` 到 `:202` 三条命令交主 agent；`run-build-directory.sh` 坏字节归别的会话 | 有：REC:282、REC:284、REC:305 | 现查 | — |
| 3.10 | `b11-guards.md:293` 管道写法（`\| tail` 一类）不认；`printf … $?`、`X=$?` 再 `echo $X`、被截成 persisted-output 的 `rc=` 行都按读不到算 | 半：`\| tail` 一类 REC:301 与 r6 正文节「三」K2 末句；后三种（语料里 0、0、1 条）没有去向 | `b11-guards.md:293` 原话「重抽语料核过」：前两种 0 条，persisted-output 1 条 | 后三种形状在真交回里被误拒 |
| 3.11 | `b11-guards.md:294` 命名检查写成 `PIPESTATUS` 的一份（`a8519aab2a61cba49`）不认 | 无 | N07：`PIPESTATUS` 全 0 | 有一行去向提到它 |
| 3.12 | `b11-guards.md:295` 交回闸 ② 没改成「仓外副本按点名的绝对路径核引文」（要主 agent 定） | 有：REC:302 判 `m4-predev` 那 9 次「照拒对」 | 现查 | 有人定三方腿可以把报告写在仓外副本 |
| 3.13 | `b11-guards.md:296`、`b11-worktree-reports.md:157`：派发闸 `report-name-unchecked`、`report-line-position-anchored` 两条开关原本没把会红的格列全（多出 13 条、18 条失败行），交回闸 `no-cite` 点名没补新格、`limit-hint-unnamed` 点名截短 | 无：PRO:1498（㉗）只管 `disable-check` 与 `dispatch-prompt.py` 的开关；这四条是同一类「点名不全」，没有去向 | N06：`report-name-unchecked` 九十节 0（记录 1 处是第三批任务 12 的出处）、`no-cite` 全 0 | 门禁 break-switch-replay 格查的是三样：点名的项没在失败行里、退 0、超时（`b11-worktree-reports.md:148`），不查「失败行都被点名」；若主 agent 认定这就是该格的合同，这一类统一判不收 |
| 3.14 | `b11-watch-wording.md:169` `lib_prompt_scratch_line.py` 文件头调用方表没列看门狗 | 有：REC:300 主 agent 补了 | 现查 | — |
| 3.15 | `b11-watch-wording.md:170` ㉚ 另两处 | 有：PRO:1501 | 现查 | — |
| 3.16 | `b11-watch-wording.md:173` 工作根下留着一份 `agent-watch-selftest-scratch-ts3nxvwv`（67 字节的 `progress.md`，2026-10-22） | 无 | `ls ~/.local/share/singlefs/work \| grep agent-watch-selftest` 今天仍在；N20 `ts3nxvwv` 全 0；`scripts/agents/agent-watch.py:5317` 自证用 `finally` 删，被杀时不走 | 目录被谁认领删了 |
| 3.17 | `b11-worktree-reports.md:150` 提交前 gate-overlap 会不会把 `registered_trees` 与 `scripts/hosts/clean-session-scratch.py` 的 `worktrees_of` 报成同段 | 有：提交前整轮（REC:264）；没有逐项 | 现查 | 提交时 gate-overlap 报了 |
| 3.18 | `b11-worktree-reports.md:155` 目录改名一并改了冻结的真提示 | 有：REC:303、REC:305 | 现查 | — |
| 3.19 | `b11-worktree-reports.md:156` 别的 worktree 里相对路径写的报告照旧按项目根解（语料 0 次） | 无：PRO:1504（㉝）的「相对路径」指的是 `lib_prompt_scratch_line.py` 解草稿目录，不是这一件 | 拆分提案第 1466 到 1505 行内 `grep -n '相对路径'` 只命中第 1504 行 | 语料里出现一份这样写的交回 |
| 3.20 | `b11-worktree-reports.md:159` `m4-predev` 有自己的会话记录目录，没放进语料 | 无（低） | 该行原话 | 那边的会话也派三方腿并把报告写在仓外 |
| 3.21 | REC:301 `\| tail` 接在命名检查后面、输出看得出跑完了的那一类没收，「记进 landing-audit-r6 的攻击面」 | 有：r6 正文节「三」K2 末句 | `grep -n 'tail' research/prompts/_landing-audit-r6-body.md` | — |
| 3.22 | REC:305「剩下的只有三方打中才知道的」 | 半：第十、十一批各自结尾都没有收拢表，也没有每批收尾的结局账 | 见表五 5.01、5.02 | — |

## 表四　九十节 ⑩、㉕ 到 ㉝ 每一行：形态够不够直接派、该不该并、是不是已经做了

判的第一个字：「已做」去向列该改成做完；「要改」形态要先改写才能派；「能派」判据、样本、出路都有；「并」与别的行说同一件事。判据、样本、出路三样是我读各行形态列得的，没有的写「无」。

| 编号 | 行 | 判 | 依据 | 推翻条件 |
|---|---|---|---|---|
| 4.01 | ⑩（PRO:1481）上游 `gate-overlap.py --touched-by` 认不出换钩子的写法 | 要改：判据有（造只用 `mv` 换钩子的会话记录，应当判到钩子）；样本无文件名；出路「上游填内容，交发版会话」没点名文件；而且前提一半不成立 | 本仓的上游拷贝 `.claude/singlefs-ai-sop/scripts/gate-overlap.py` 与上游 zh 仓里 `scripts/gate-overlap.py` 的 sha256 前 20 位都是 `e0802021d3972c680385`；第 574 行 `elif command_name in ('cp', 'mv', 'install', 'ln')` 把 `mv` 的目的路径记进写入目标。探针 `out/overlap_targets.out`：`cp … a.sh.new && mv a.sh.new a.sh` 认出 `a.sh` 与 `a.sh.new`，单独 `mv` 认出 `a.sh`，`python3 scripts/gate/hook-swap.py … --hook .claude/hooks/a.sh` 认出 `[]`；第 509 行文件说明本来就写「解释器里写的文件都认不出，那一半由门禁阶段兜底」。REC:219 写的「先看上游三语仓有没有在发版」没写进这一行，也没人记过结果 | 用 `--touched-by` 整条（含 git 基准）对一份只 `mv` 换钩子的会话记录跑，判不到那个钩子；我只核了 `bash_write_targets` 这一层 |
| 4.02 | ㉕（PRO:1496）交回闸 ⑤ 的 Bash 写 `.rs`、根的命令替换；检出钩子 ⑫ 的 `-p` `--tmpdir`；`lib_writer_lock.py` 文件头 | 要改：判据泛（「各带红格与弄坏开关」），样本无；三件落在三份文件上，其中检出钩子是别的会话刚收严的（PRO:1496 自己写了从只记检出变成被拒） | `sed -n 1583p .claude/hooks/bash-command-detector.sh` 说 `-p`、`--tmpdir` 交 None（别的会话的 ⑫ 收严那一版）；PRO:1517 起的九十三节记的就是那次收严 | 别的会话说检出钩子这部分它自己收 |
| 4.03 | ㉖（PRO:1497）五件 | 要改，缩成两件：(a) 另四格读不出登记位判红并加样本——能派（判据、样本、出路都有，今天仍开着，见 2.16）；(e) 不登记的比对对象点名行只缩不涨——能派，键「（登记出处，#号）」可从 `.claude/gate.d/replay-comparison-units.tsv` 第 18 到 25 行直接读。其余三件：lib-lag-baseline 文件头与 heavy-test-guard 文件头已做（2.14、2.15）；lib-forbidden-notations 没有对象（2.17） | 见表二；`grep -c '^# 不登记的比对对象' .claude/gate.d/replay-comparison-units.tsv` 为 8，开工时调度记录写的是五行 | 另四格有红样本 |
| 4.04 | ㉗（PRO:1498）派发闸口子 | 要改：一行混了五件。三方腿报告路径三种认法已做（`.claude/hooks/lib_three_way_legs.py:15` 文件头点名 ㉗，`is_leg_report_path` 在第 94 行）；「提示里没有草稿目录行时 ⑰ 只判裸文件名」就是 r5 判决接受的代价（见 4.13 并）；剩两件能派：三方形态只认带报告标签或落在本腿轮名下的路径、名里带 report 照判；`disable-check` 与 `dispatch-prompt.py` 开关点名并进 4.12 | r5v:44 与 PRO:1491 去向格对读（后者写「2026-10-21 以前不照骨架写的 402 份真提示同此」） | 派发闸 ⑰ 的实现里还有一种没走共用函数的认法；我只读了库文件头与两份判决，没读 ⑰ 的实现 |
| 4.05 | ㉘（PRO:1499） | 已做：第十批做完，r5 攻过；去向格「进 landing-audit-r5」已了结（r5v:67） | 现查 | — |
| 4.06 | ㉙（PRO:1500）两半 | 要改：两半没有共同点，该拆。(a) 派发闸禁读清单里不许有本腿报告路径——能派（判据有；样本取 r4 攻方提示头一版，REC:251 记了；出路派发闸一处），而且与 ㉗ 的「三方形态不把禁读清单里的路径当本腿报告」改同一处，并；(b) 会话名判法挪进三个 kb 写入脚本还是放进 doc-text 按 diff 判——「或」没选，派不出去，先定 | `grep -c '会话名'` 在 `scripts/kb/replace-once.py`、`insert-row.py`、`replace-batch.py` 里为 0；`.claude/hooks/runner-dispatch-guard.sh` 里禁读清单只在必填输入名与自证样本里出现（`grep -n '禁读'` 命中第 147、182、337、1700、2137、2197、2198、2465、2467、2469 行），没有判禁读清单内容的代码 | 选定的一边已经有人做了 |
| 4.07 | ㉚（PRO:1501） | 已做一半、能派一半：看门狗 `progress_file_of` 第十一批做完（`b11-watch-wording.md`）；「⑯ 收严的影响量一遍」r5 攻方量过（`landing-audit-r5-opus-output.md:310`）；剩 `scripts/agents/dispatch-prompt.py:228` 的 `DRAFT_LINE` 与 `scripts/agents/agent-cost.py:763` 两处改调共用库，判据有（`landing-audit-r5-opus-output.md:317` 量出 2026-10-22 起骨架 `--check` 与新库不同 6 份），样本有，出路有 | `sed -n 228p scripts/agents/dispatch-prompt.py`、`sed -n 763p scripts/agents/agent-cost.py` 现查 | — |
| 4.08 | ㉛（PRO:1502） | 已做：第十批任务 34；去向格仍写「下一批工具改动」 | 见 1.12 | — |
| 4.09 | ㉜（PRO:1503） | 已做：第十一批任务 35 到 37；去向格只写「第十一批」 | 见 1.10 | — |
| 4.10 | ㉝（PRO:1504）四件 | 要改，拆：(a) 围栏照 CommonMark——能派，样本是 `landing-audit-r5-opus-output.md:129` 那一节；(b) 相对路径按仓根解——判据不明（r5 攻方自己写推的、没量过），先量；(c) 续做闸守卫——并：`.claude/gate.d/break-switch-outside-selftest-pending.tsv:6` 已登记同一件（欠账表 C622 那一半），九十节这一句多余；(d) 进程安全扫描只带部分目录——能派，样本是 REC:283 的 `scripts/agents` 单目录跑 | `grep -n 'continuation-guard' .claude/gate.d/break-switch-outside-selftest-pending.tsv` 命中第 6 行；`.claude/process-safety-pending` 现有 4 行（`wc -l`） | (d) 的待改清单逻辑已有人改（我没读到这样的代码） |
| 4.11 | 并：㉚ 剩下两处与 ㉝(a)(b) | 并：都围着 `.claude/hooks/lib_prompt_scratch_line.py`，先修库（围栏、相对路径）再让 `dispatch-prompt.py`、`agent-cost.py` 改调，一个工具员一批改完，自证只落一次 | 库文件头调用方表（`b11-watch-wording.md:169` 说明已列看门狗） | — |
| 4.12 | 并：「弄坏开关点名不全」一类 | 并：㉗ 的 `disable-check` 70/299 与 `dispatch-prompt.py` 3/18、2.06（`trust-dead-holder`）、2.33（`memoryscan`）、3.13（四条）是同一类，散在五处，只有两处有行。要么并成一行并写判据，要么判不收（格只查三样，见 3.13） | 各出处行 | 主 agent 定这一类不修 |
| 4.13 | 并：㉗ 第三件与 ⑳ 去向格的 402 份代价；㉗(i) 与 ㉙(a) | 并：前一对说同一件事，后一对改同一处（派发闸 ⑰ 的禁读清单那一步），各并成一行 | PRO:1491、r5v:44、PRO:1498、PRO:1500 对读 | — |
| 4.14 | 已做仍写待办的去向格（汇总） | 已做：⑯（PRO:1487 「还没真派过…开轮时顺带核」）、㉒（:1493 「.txt 门禁认不认没核，进 r4」）、㉓（:1494 「进 r4」）、㉖ 两件、㉗ 第一件、㉚ 看门狗与量、㉛、㉜；表头 PRO:1466 写「四条与一条没想清」，该节现有 33 行 | `awk 'NR>=1466 && NR<=1505 && /^\| [①-㉝] \|/' records/2026-09-16-subagent拆分提案.md \| wc -l` | 九十节某行去向格逐字写着做完 |

## 表五　调度记录收尾项、C879 与 `replay-comparison-units.tsv` 文件头

| 编号 | 出处 | 去向 | 依据 | 推翻条件 |
|---|---|---|---|---|
| 5.01 | `.claude/main-agent.md:27` 要求每批交回后做一张收拢表，逐条判本轮关、下轮开、不做 | 无：第十批、第十一批各自的小节末尾都没有收拢表；第九批收尾有（REC:256）。开着的线——九十节 ⑩、㉕、㉖、㉗、㉙、㉚、㉝，提交前整轮，C879——没有一处在第十一批之后合着列过 | `awk 'NR>=267' records/2026-10-22-子agent结果落地审计.md \| grep -c '收拢表'` 输出 0；全文 `grep -n '收拢表'` 只有第 256 行 | r6 判决后补写收拢表（这一份是否算，要等判决） |
| 5.02 | 同一条要求每批收尾跑闭窗口结局账、落进 `records/` | 无：只有第九批的 `records/landing-audit-b9-outcome-2026-10-23.txt` | `ls records \| grep -c outcome` 输出 1；`awk 'NR>=267' records/2026-10-22-子agent结果落地审计.md \| grep -c 'agent-cost.py outcome'` 输出 0 | 第十、十一批的账补跑并落盘 |
| 5.03 | K1 第 3 条：`.claude/kb/checks-owed.md:560` 的 C879 与 `.claude/gate.d/replay-comparison-units.tsv` 文件头 | （总述，不计数）方向一致，数对不上 | 见 5.03a 到 5.03d | — |
| 5.03a | 条数：C879 三语（`.claude/kb/checks-owed.md:560`、`.claude/kb/checks-owed.en.md:560`、`.claude/kb/checks-owed.ja.md:560`）都写 r5 登记比对对象表第 15 至 19 行共五条；`.claude/gate.d/replay-comparison-units.tsv` 第 18 到 25 行是八行「# 不登记的比对对象」：r5 的 #15 到 #19 五行加 r7 的 #1 到 #3 三行 | 无：r7 三行不在 C879 里，也没有别的欠账号接 | `grep -c '^# 不登记的比对对象' .claude/gate.d/replay-comparison-units.tsv` 输出 8；`git show :.claude/gate.d/replay-comparison-units.tsv \| grep -c '^# 不登记的比对对象'`（暂存区那一版，批九的）输出 5；`grep -n 'e165-r7-prereg' .claude/gate.d/replay-comparison-units.tsv` 命中第 23、24、25 行 | C879 补写 r7 三条，或另开一个 C 号 |
| 5.03b | 指向：tsv 第 17 行写「E165 r5 #15 到 #19 这五条……就是 `.claude/kb/checks-owed.md` 的 C879」，只覆盖五行；r7 三行（第 23 到 25 行）下面没有这样的句子。`research/prompts/e165-r7-prereg.md:378` 自己写「C879 那一形」，所以 r7 的设计者知道这是同一件，登记时没写 | 无 | `sed -n 17p .claude/gate.d/replay-comparison-units.tsv` 与 `sed -n 378p research/prompts/e165-r7-prereg.md` | 第 17 行改成「八条」并列出 r7 三条 |
| 5.03c | 出路：C879 的出路是把这几条登记进 tsv，tsv 第 17 行说有了单元就删掉这几行、改登记成正式行 | 有：两处互相不矛盾（这一点对得上） | 两行对读 | — |
| 5.03d | 「那五行指回 C879」（C879 正文）：指回的是第 17 行那一句，第 18 到 22 行的五行本身各自只写理由，不含 C879 | 半 | `sed -n 18,22p .claude/gate.d/replay-comparison-units.tsv \| grep -c C879` 输出 0 | 把 C879 号写进每一行的理由 |
| 5.04 | 九十节 ㉖(e) 要立的「只缩不涨」与 5.03a 的八行 | 半：该立；但立的顺序要看：基准那一版（暂存区）只有五行，r7 三行不在基准里，一旦格上线、先于 r7 提交，三行都判红 | 5.03a 的两条命令 | 先提交 r7 三行进基准再立格 |
| 5.05 | C880（`.claude/kb/checks-owed.md:561`，与 C879 同一批挂账）说 `scripts/experiment-apparatus/replay.sh` 里 driver_e165、driver_e165_light、driver_e165_adversarial 三行期望的是 2026-10-01 原产物 | 有：今天仍成立，r7 换的是另外三个驱动（`research/prompts/e165-r7-prereg.md:234`），不影响 C880 | `sed -n 1667,1669p scripts/experiment-apparatus/replay.sh` 三行期望产物都是 `2026-10-01` 开头 | 那三行期望产物被改 |

## 一览（数是 `awk` 按每行去向格头一个字数出来的，不是手数）

命令：`awk -F'|' '/^\| [1-5]\.[0-9]+[a-d]? \|/ { line=$0; gsub(/\\\|/,"@@",line); split(line,f,"|"); … }'` 取每行第四格（去向格）头一个字。

| 表 | 行数 | 有 | 半 | 无 | 别的 |
|---|---|---|---|---|---|
| 表一 判决与四份腿报告 | 24 | 13 | 7 | 4 | — |
| 表二 第九批交回报告 | 34 | 14 | 7 | 12 | 1 行「见 1.19」 |
| 表三 第十、十一批交回与调度记录 | 22 | 12 | 4 | 6 | — |
| 表五 收尾项、C879 | 9 | 2 | 2 | 4 | 1 行总述 |
| 合计 | 89 | 41 | 20 | 26 | 2 |
| 表四 九十节 ⑩、㉕ 到 ㉝ | 14 | 判：已做 5、并 3、要改 6 | | | 能直接派的 0 行整行，缩小后能派的见下 |

「无」的 26 行归成 17 件（逐件列出，括号里是行号）：

1. 看门狗 ⑭ 的自证从没在真进程上量过，r5 判决写「第三轮点名量」，r6 正文没点名（1.03）。
2. r5 判决第 79 行点名没核的攻方两处推的差异：单独的 `\r` 与坏字节判定先于守卫判定（1.05）。
3. 账表三方腿桶改读登记后桶变了多少，两版没比（1.18）。
4. ㉑ 实测没覆盖的几种形状，而九十节 ㉑ 一行写成「不分大小写」（1.19）。
5. 合入后验证新加的那一步：第一次真跑要看什么没写（2.02）；`.claude/hooks/selftest-registry.tsv` 里 merge-and-verify.sh 一行的理由没补（2.03）；那一步的内存上限与别的几步共用 `--memory`，建议值推的（2.04）。三行同一个对象，该并一行。
6. 「弄坏开关点名不全」：`trust-dead-holder`（2.06）、`memoryscan`（2.33）、派发闸与交回闸四条（3.13）；九十节只有 ㉗ 管了两处（见 4.12）。
7. 派发闸跨行「」引文没处理（2.09）。
8. `scripts/agents/dispatch-prompt.py` 文件头第 38、39 行过期（2.10）。
9. `scripts/kb/claim-experiment.sh` 不在复跑表（2.19）。
10. `CLAIM_SKIP_USED_CHECK`、`CLAIM_SKIP_WORKTREES` 不是 `_BREAK` 名字，门禁认不出（2.20）。
11. `.claude/gate.d/stage-owners.tsv` 里 replay-comparison-units 一行不含设计员（2.22）。
12. `scripts/gate/hook-swap.py` 不收 `scripts/` 下的文件，三次改动靠手工协议（2.31）；`watch.sh` 键表新键自证不核（2.32）。
13. 验证段第 1 步命令代码段不带前缀、正文没写只重跑红格的命令怎么跑（3.02）。
14. 命名检查写成 `PIPESTATUS` 的不认（3.11）。
15. 工作根下遗留的 `agent-watch-selftest-scratch-ts3nxvwv`（3.16）。
16. 别的 worktree 里相对路径写的报告按项目根解（3.19）；`m4-predev` 自己的会话记录目录没进语料（3.20）。
17. 第十、十一批没有收拢表、没有结局账（5.01、5.02）；C879 与 tsv 的条数对不上、r7 三行没有指向（5.03a、5.03b）。

「半」的 20 行里有一半是同一个毛病：做了，去向列没改（1.07、1.08、1.10、1.12、2.14、2.15，以及 4.14 汇总的八行去向格）。`records/2026-09-16-subagent拆分提案.md` 第 1466 行节标题「四条与一条没想清」，节里现有 33 行。

## 对九十节的建议改法（只交，不替主 agent 判）

- 关：㉘、㉛、㉜ 写做完；⑯、㉒、㉓ 去向格改成「r4、r5 已了结」；㉖ 两件、㉗ 第一件与第三件、㉚ 看门狗与量写做完。
- 拆成能直接派的：㉕ 拆交回闸一件、检出钩子一件（先问写 ⑫ 的会话）、文件头并进「文本补全」；㉖ 缩成「另四格」与「不登记行只缩不涨」两件；㉙ 拆两半；㉝ 拆四件、(c) 删（待办表已登记）。
- 并：㉚ 剩下两处与 ㉝(a)(b)（同一份库）；㉗(i) 与 ㉙(a)（同一处）；弄坏开关点名不全一类并一行（或判不收）；文本补全一行（`lib_writer_lock.py` 文件头、`dispatch-prompt.py` 文件头、`selftest-registry.tsv` 理由）；`merge-and-verify.sh` 新一步三件并一行。
- 改写：⑩ 先重写前提（`mv` 已认，剩 `hook-swap.py` 一种，与上游自己写明的限度同一件），点名上游文件，再谈交发版会话。
- 新补一行：第十、十一批收拢表与结局账；C879 补 r7 三条（或另开号）；验证段第 1 步命令加前缀。

## 没做什么

- 没判 K2、K3（攻方的格），没替主 agent 撤销或维持判决，只交复核结果。
- 十四份交回报告没有逐字读完：读了每份的「结论」「没做什么」「射程外」「要主 agent 跑的命令」「顺手看到」「过程里要主 agent 知道的」「交主 agent 定」几节，加上按「没量、没核、没测、没查、留给、交主 agent、要你定、要主 agent 定、不知道、没验、没试、没看」十二个词的全文 `grep -n`。正文里不含这些词的延后写法可能漏。推翻条件：有人在这些报告里找到一句不含这十二个词、也不在上述几节里的延后写法，而它没有去向。
- 四份腿报告读了各自的「没做什么」「这条腿自己的限度」「没打中」几节与总表，没读全文。
- 没起门禁阶段，没起 `gate-overlap.py --touched-by` 整条（只核了它的 `bash_write_targets` 一层，见 4.01）；没读派发闸 ⑰ 的实现（4.04）、交回闸 ⑤ 的内部（3.06）；4.10(d) 是否已做只看了自证片段，没有依据说做了，也没有依据说没做。
- 没复算第十一批的数（3、46、693、22、41、37、9）和两份判决的数，那是攻方 K3 的格；本报告引的数只有表里写明命令的那些。
- 内存包装下的复跑没成（退 251，见开头）；直接跑的两遍输出逐字节相同。
- 没核别的会话改的部分（`bash-command-detector.sh` 的 ⑫、`heavy-test-guard.sh` 文件头、`scripts/hosts/` 下的脚本）；点到的地方都标了是别人的。
- 没写任何模型之外的仓内文件；没动九十节、调度记录、C879（建议只写在上面）。

## 要主 agent 跑的命令

1. 内存包装下复跑（工作目录仓根，两份输出应与 `research/prompts/landing-audit-r6-sonnet-model/out/` 逐字节相同，欠账表那一列例外）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/landing-audit-r6-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新的草稿目录> --model /home/fy5090/code/singlefs/research/prompts/landing-audit-r6-sonnet-model
```

   看末两行 `name=destinations … 逐字节相同`、`name=overlap_targets … 逐字节相同`。本机此刻放不下这一条时它会排队（我撞上过），可以等，或改走第二台。
2. 4.01 的端到端核：造一份只用 `mv` 换钩子的会话记录，对它跑 `python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --touched-by <会话记录> .`，看是否判到那个钩子。这一条归「提交时才跑的检查」，子 agent 不跑，要主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request`。
3. 5.03a 的复核：`grep -c '^# 不登记的比对对象' .claude/gate.d/replay-comparison-units.tsv`（8）与 `.claude/kb/checks-owed.md` 第 560 行对读。

## 引文核对

草稿目录 `~/.local/share/singlefs/work/landing-audit-r6-sonnet/` 里自己建的 `rerun-a`、`rerun-b` 是输出文本，没有仓副本、编译目录或库拷贝；交回前删掉两个输出目录，模型目录里的 `out/` 留着。
