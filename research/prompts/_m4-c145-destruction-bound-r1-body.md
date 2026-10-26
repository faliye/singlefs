# m4-c145-destruction-bound-r1：未完成销毁意图的区间上界在第一版怎么判（2026-10-26）

## 一、题面

`.claude/kb/checks-owed.md` C145（deadlist 在盘上没有落点）里的第二条判定：「未完成意图的区间里没有纪元 ≥ 上界的条目」。用户 2026-10-26 定这一条开推论轮（同一次弹窗另定：第一条并进 I-3.6，第三条加运行期断言并把等价性写成测试留档）。

这一条卡在：第一版的销毁意图不存上界 hi（`.claude/kb/decisions/05-快照-空间记账机制.md` 已定项 12 正文「hi 每批从派生链现取……不存续做点」），下一侧是活头时 hi = 写意图那次发布的 txg，盘上读不出来，判别子观测不到。池级 checker 那一段注释（`crates/singlefs-checker/src/walk.rs` 里 `entry_is_pending_release` 上面那三行）论证「下一侧是头时按 ∞ 判与按写意图那次的 txg 判，盘上读得到的条目上结论相同」。

这一轮要判的是：那段等价论证成不成立；成立的话这条判定在第一版的形态是什么（写成哪条可判的规则、挂哪条不变量）；不成立的话，要往盘上加什么（意图里存 hi、或别的可观测量）才判得了，代价与格式影响各是什么。

被判的条款：D5 已定项 12（定案「取「丙-区间（带上界）」」、「哪一份」原文、销毁原文「hi 在下一侧是快照时取它的 txg，是头时取写意图那个开放 checkpoint 的号……一批里每个动作施加前先核它要删的条目还在」）；`crates/singlefs-checker/invariants/05-空间记账.md` 的 I-3.5、I-3.6；d5-item12 三轮判决（`research/prompts/d5-item12-r3-main-verification.md`）。

## 二、实现今天的样子（主 agent 的观测，2026-10-26）

背景事实来自调查员报告 `~/.local/share/singlefs/work/m4-close-debt-specs/result.md`「## C145」一节，主 agent 现查过：

- checker：`crates/singlefs-checker/src/walk.rs` 的 `judge_the_deadlist_against_the_head_and_the_live_snapshots` 判 I-3.5 ③ ④ 与逆向、I-3.6；侧的划分 `deadlist_sides`、`DeadlistSide::owns_epoch`；待放集 `entry_is_pending_release`。
- core：建快照、覆盖写分流、销毁按批放块在 `crates/singlefs-core/src/transaction.rs` 的 `deadlist_changes_of_this_publish` 一带；放掉的条目取自上一版的 deadlist（`deadlist_entries_the_destruction_of_a_snapshot_releases`）。
- 销毁意图的编码在 `crates/singlefs-core/src/logged_operations.rs`；它存不存 hi 以代码为准，腿现查。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### B1　等价论证成不成立
格里要问：在第一版能写出的全部历史上（建快照、快照下覆盖写、写销毁意图、分批放块、中途崩溃与续做、回退、再建快照），下一侧是活头时，「按 ∞ 判」与「按写意图那次的 txg 判」在盘上读得到的条目上是否总给同一结论；找得到反例就给出那段历史与两种判法各判什么。

### B2　成立时的形态
格里要问：这条判定在第一版写成哪条可判的规则（例：以 ∞ 为上界判、或判「意图区间里纪元 ≥ 某个盘上可读量的条目不存在」）；挂 I-3.6 还是别的；改坏它时哪个必红世界会红。

### B3　不成立时要加什么
格里要问：要判这一条，盘上最少要多一个什么量（意图里存 hi、或别的）；D15 冻结层与 layout 的影响；有没有不改格式的替代判法。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | B1–B3 | 从 D5 已定项 12 原文、d5-item12 三轮判决与今天的实现推 B1 的结论与依据，给 B2 或 B3 的推荐形态；每条结论写文件:行号 |
| 云端攻方（Opus） | B1–B3 | 在冻结副本的拷贝上造 B1 的反例历史（两种判法结论不同、或盘上读得到的条目里有一条纪元 ≥ 真实 hi 而 ∞ 判法放过）；攻推荐形态的判别力。每个打中给可复现的命令与原样输出 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号；冻结副本路径在派发提示里给。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；拷冻结副本，不在冻结副本里改；每棵树各用一个编译目录，跑完删。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo；线程上限照派发提示。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
