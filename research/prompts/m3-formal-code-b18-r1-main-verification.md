# 第 18 批代码轮第一轮判决（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 P1 P2 P3 V242u U6 F1 F2 F3 F4 F5 -->

被判的文件：`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/tests/model_answers_unmount_with_the_floor_at_the_current_version.rs`，`crates/mutations.tsv` 18-1 追加的 6 行与 18-2 删一行、加三行。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b18-r1-body.md`、`_m3-formal-code-b18-r1-diff.md`、`_m3-formal-code-b18-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-formal-code-b18-r1-snapshot/`（crates 275 份，冻结副本 `/tmp/claude-1000/m3-formal-code-b18-r1/tree/`） | crates 无输出；kb 三份对不上：`.claude/kb/checks-owed.md`、`.claude/kb/decisions/16-发布语义.md`、`.claude/kb/decisions/23-journal的角色与格式.md`——开工之后根环那条线第 9 批写回甲、乙（`/tmp/claude-1000/m3-r1/batch9-writeback-a/report.md` 第三节、`…-b/spec.md`）、删除与并发那条线第三份写回（`/tmp/claude-1000/m3-writeback3/scribe-{a,b}/`）与这条线 19-3 改了它们；正推腿报它引的 D16「树表 0 条那一版上正常卸载」那一句由第 76 行挪到第 77 行，引的是今天的行号；两腿引的 kb 行不落在那几处改动的语义里（改的是 E2 句、抬 F 上限行、C333 / C342 / C597 / C398、D23 别的分项） |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b18-r1-sonnet-output.md`（sha256 7b2989f2…） | 引文 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b18-r1-opus-output.md`（sha256 269ec1b2…），模型 `…-opus-model/` | 主 agent 照它的 `tools/rerun.sh` 跑 quick、rand、sweep 三段（工作目录在会话草稿 `b18-rerun-{quick,rand,sweep}/`，三段 RC 都 0；第一次把脚本拷进草稿跑，脚本按自身位置推仓根而退 1，改在原位跑——脚本只写工作目录、不写模型目录）：quick 45 份里 43 份去掉耗时逐份同、缺的 2 份（`count.out`、`t3v242u-none.trace`）脚本本就不产出；rand 13 份里 12 份同、缺 `rand-32768-full.out`（腿手跑的）；sweep 与它 `outputs/sweep/` 共有的 36 份逐份同（脚本产出的 vA / vB 名字与腿手跑的分批不同） |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root <冻结副本> --root . --background <背景材料>`：「✓ 核了 72 处引文，对上 72 处，没判 0 处」 |

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| K1 P1 第三截接判官 | 五样输入、与 I-8.4 先后一致；只判 C670 不拿 U6 其余三条定红绿是范围冲突（已记欠） | 第三截合法态上 0 误红（判官没被调到：基线上再挂一次都没被拒）；没崩过的那一次被拒时漏的态第二截已红；V242u 在用例 3 上种子 0 红（第三截不合法 2 次），42 个种子里 35 个红，二次崩溃种子只随（历史种子、第一次崩溃段号）变，独立抽样 14 次 | 站住；V242u 那一行靠抽样，提交时不红照「抽样没抽中」处理（判决 `research/prompts/m3-formal-code-b16-r1-main-verification.md` 之后记的口径），攻方改法 F3（二次崩溃种子混进第一次崩溃那个子集，量过 35 → 41）零轮、会改已有复现行，交下一批先评估 |
| K2 P2 接线用例 | — | **打中（复跑坐实）**：第三截调用点三种接错三条用例全绿、都让 V242u 由红转绿，6 行变异没一行落在调用点上（用例 2 绕过调用点）；第二截所选那一版交 None 时用例 1 照绿（不合法改由 U6「复算为假」给出）；下标取错今天观测不到（随机 60 段注入过读故障的态 0 个） | 打中：改法 F1（用例 1 断言不合法条件里有 C670 那一条、没有「复算为假」，量过）、F2（第三截一条经调用点的靶向二次崩溃用例，推的）、F5（调用点算录制流下标抽成函数单独钉，推的）进下一批实现员 |
| K3 P3 卸载那一格 | 与 D16（发布语义） 已定项 1、实现逐字段一致；删除批 GA 没让「树表 0 条不写」不成立；「F 不动」「会话关」两条断言没有只对它们红的变异行 | — | 站住；两条断言各补一行变异进下一批 |
| 越出攻击面 | — | 随机历史基线 6 段在第二截报新发现，全落在「删除之后的建 inode」（`CreateInode`），实现报 `WriteToAnInodeThatIsNotInTheTree { inode: 1 }` | 复跑坐实（rand 那 6 行逐行同），属删除与并发那条线，已转它（复现命令与样本）；这条线不判 |

## 三、去向

1. 18-1、18-2 的改动站住，不回改。
2. 下一批实现员：F1、F2、F5；18-2 两条断言各补一行变异；F3 先由实现员评估改动已有复现行的范围、交主 agent 定再做；连同第 17 批判决与 19-1 交回留下的 `crates/singlefs-checker/src/image.rs` 第 195 行注释、`crash_judge_checker_common.wgsl` 第 160–161 行没人用的常量。
3. 提交时：18-1 的三条 checker 档用例、6 行变异整表；V242u 那一行照抽样口径判。

## 四、复跑核对

见第一节表。核不动：checker 档没跑；F2、F5 推的。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | F3 换二次崩溃的抽样种子 | 攻方 | 副本上量过 | 下一批先评估对已有复现行的影响，再决定要不要做；不用用户定 |

不涉及要用户定的岔路。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-04 不受影响：P3 与补进定案的那一句逐字段一致 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-10-04 不受影响：只拿见证判据当第三截判官的判据 |
