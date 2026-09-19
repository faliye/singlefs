# 上游 SOP 五段的英、日译文填写报告（2026-09-19）

原文：`research/prompts/sop-upstream-2026-09-19-zh-sections.md`。只填内容：没暂存、没提交，没跑 bump.sh，没动 VERSION / MANIFEST / SOURCE-MANIFEST / 溯源哈希 / CHANGELOG。
改法：`research/scripts/replace-once.py` 定点替换，旧串是锚点那一句，新串是锚点加上译文；十次都报「命中 1 次，已替换并回读确认」。别的会话已经填好的段（command-safety、session-wrapup，以及 evidence / test 里已有的改动）没有碰。

## 插在哪（插完之后的行号，指新段第一行）

| 段 | en | ja | 锚点（插在它后面） |
|---|---|---|---|
| kb：带日期的现状快照 | `rules/kb-discipline.md:205`（三级标题，在第 8 节末尾） | `rules/kb-discipline.md:188` | 第 8 节最后一句「由 doc-lint 强制」的译文 |
| evidence：回扫要按旧说法搜 | `rules/evidence-discipline.md:373` | `rules/evidence-discipline.md:316` | 「判据：把这句话里的旧值换成新值……」的译文，在「撤回的理由失效……」小节之前 |
| test：阳性对照的答案不能让被测方读到 | `rules/test-discipline.md:81` | `rules/test-discipline.md:78` | 「改了就是新实验，重跑」的译文，在「失败条款……」小节之前 |
| show-me-test：射程只到 `.claude/gate.d/` | `rules/show-me-test.md:176` | `rules/show-me-test.md:164` | 「本地阶段第一次被扫就是 7 条……」的译文 |
| show-me-test：推论第 5 条 | `rules/show-me-test.md:221` | `rules/show-me-test.md:214` | 推论第 4 条末句「这一轮跑过且通过才换下」的译文 |

## doc-lint 末行（仓根下 `bash scripts/doc-lint.sh`）

- en：`✓ 文档铁律检查通过（检查 29，跳过 1；DOC_LINT_VERBOSE=1 看全部）`；同时照常报「本语言（en）没有词表，A / D / I 三项未实现」
- ja：`✓ 文档铁律检查通过（检查 29，跳过 1；DOC_LINT_VERBOSE=1 看全部）`；同时照常报「本语言（ja）没有词表，A / D / I 三项未实现」

## 用词对齐（取自目标仓已有写法）

| zh | en | ja | 出处 |
|---|---|---|---|
| 回扫 | sweep | 洗い出す / 洗い直す | en/ja evidence-discipline「撤回……要回扫」一节 |
| 回扫员 | sweep agent | 洗い出し担当 | 目标仓里没有现成译法，新起；与「sweep」「洗い出す」同词根 |
| 子 agent | subagent | サブエージェント | command-safety、session-wrapup |
| 阳性对照 | positive control | 陽性対照 | test-discipline |
| 欠账表 | checks-owed table | 未返済の検査の表 | show-me-test「326 条欠检查」一段 |
| 样本目录 | fixture directory | 標本ディレクトリ | show-me-test「判别力样本」= discrimination fixture / 判別力標本 |
| ## 历史版本 | "## Revision history" | 「## 改訂履歴」 | kb-discipline 第 8 节 |
| 实例 | instance | 実例 | ja evidence-discipline:215 |
| 实测（日期，singlefs） | Measured (date, singlefs) | 実測（日期、singlefs） | 各文件已有实测段 |

## 逐段核对表

| 段 | 仓 | 文件:行 | 与原文差在哪 |
|---|---|---|---|
| kb | en | kb-discipline.md:205–217 | 例句「2026-09-14 现状：」译成 "Status on 2026-09-14:"，「现状（日期）：」译成 "Status (date):"。「检查一上就整片红」译成 "a check would turn whole areas red at once"，"at once" 对应「一上就」。其余限定词（两次抽样、日期删掉再读、三个现状例、三个事件例、「暂」）都在 |
| kb | ja | kb-discipline.md:188–200 | 同上：「整片红」译成「一面が一度に赤くなる」，「一度に」对应「一上就」。「第一个事务」译成「最初のトランザクション」 |
| evidence | en | evidence-discipline.md:373–381 | 「逐句搜」译成 "search, one by one"。「只抽了样」译成 "only sampled"。无增删 |
| evidence | ja | evidence-discipline.md:316–324 | 「四个回扫员」译成「四人の洗い出し担当」，后文「两个 / 一个」随之写成「二人 / 一人」。「还没有第二个实例」译成「二つ目の実例はまだない」。无增删 |
| test | en | test-discipline.md:81–88 | 「16 位哈希」译成 "16-character hash"：中文「位」不分 bit 与字符，按 singlefs `research/scripts/stale-candidates.py:116`（`hexdigest()[:16]`，16 个十六进制字符）取字符义；这是译者按实现消歧，原文没写单位。「验收回扫员」译成 "acceptance-testing the sweep agents"。「载体类型」译成 "carrier type" |
| test | ja | test-discipline.md:78–85 | 「16 位」译成「16 桁」（与中文同样不点明 bit / 字符，但「桁」在日文里读作位数字符）。「载体类型」译成「媒体の種類」。无增删 |
| show-me-test 段 1 | en | show-me-test.md:176–179 | 「单跑整仓 gate-lint」译成 "running gate-lint on its own over the whole repository"。「样本目录」译成 "fixture directory"。无增删 |
| show-me-test 段 1 | ja | show-me-test.md:164–167 | 原文一句「……交给 gate-lint；调用时设……」在日文里断成两句（「渡す。呼ぶときは……」），内容不变 |
| show-me-test 段 2 | en | show-me-test.md:221–225 | 「整组放行」译成 "wave the whole group through"；「全判要改这种」译成 "a report judging everything 'needs a change'"，把「这种」落成 "report"（与上一句「套话报告」同类），属补主语，不加限定 |
| show-me-test 段 2 | ja | show-me-test.md:214–218 | 同上，「这种」落成「報告」 |

## 没做到的

- 没有第二个人或模型复核译文；上表是我自己逐句对照原文核的。
- 「回扫员」在两个目标仓都没有现成译法，sweep agent / 洗い出し担当 是我起的，发版会话若另有定名需统一。
- en / ja 仓没有文风词表，doc-lint 的 A / D / I 三项在这两个仓里未实现，通过不代表这三项验过。
