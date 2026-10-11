# 第 37 批代码轮第二轮正文（2026-10-08）

<!-- doc-lint:not-numbers B1 B2 B3 B4 C1 C2 C3 C4 -->

## 一、题面

第一轮判决 `research/prompts/m3-formal-code-b37-r1-main-verification.md` 采了攻方打中的 B1-甲、乙、丙与 B2、B3、B4 几处，主 agent 当场改（判决第三节）。这一轮攻第一轮的修补本身，改动全文 `research/prompts/_m3-formal-code-b37-r2-diff.md`（相对第一轮冻结副本）：

- `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 的内联单测 `every_criterion_has_a_positive_control_and_every_control_is_red`：改成正式跑用的五档（r1、r2、两份表都取的 r2、r3、r5）各自断言「跑出对照的判据集合 = 写死的登记集合」且读数全红，五档并起来罩 `Criterion::ALL`。
- `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs` 三处注释（行号与 25）。
- `crates/singlefs-checker/src/image.rs` 第 212 行补两种不适用。
- `crates/mutations.tsv` 本批四行（第 3383、3407 行整行换，第 3496、3497 行追加）。

条款同第一轮（E165（恢复重放有界穷举与变异杀伤界）的阳性对照条款，I-4.1（重放后树可遍历））：`research/prompts/e165-preregistration.md` 第 5.4 节与 V2、`research/prompts/e165-r2-prereg.md` 第 5.5 节（另看第 5.4 节 J7 判在哪几档）、`research/prompts/e165-r3-prereg.md` 第 5.6 节（拆行）、`research/prompts/e165-r5-prereg.md` 第 5.3、5.4 节；正式跑的档取 `scripts/experiments/replay.sh` 起这个装置的驱动。

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- 冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b37-r2/tree/`（取自改后的主工作区，清单同目录 `crates-sha256.txt`）。树里别的线的改动不判。代码引文读这棵树，写 `crates/…` 相对路径与这棵树里的行号。
- 主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 跑过：改后单测 `1 passed`；nff 17 passed；clippy、fmt 过；四行变异 `prove-red.sh` 结果与「改前的并集单测在 `ALimit => !run_profile.judges_j8()` 下仍过」的现跑，见第一轮判决第四节。

方案按 `crates/` 今天的实现来谈。

## 三、格

### C1　写死的五份集合对不对（辩方、攻方都判）
每档的集合与三份登记逐行对：每条判据在哪几档判（登记原句），写死的集合与 `main` 判定行里真判的（按 `judges_*`、`compares_the_two_record_tables`、族）是不是逐档相同；r1 只取合法、对抗两族，r2–r5 取三族，与正式跑各档起的族对不对得上；有没有一档正式跑判了、集合里没写（或写了、正式跑不判）。

### C2　第一轮判决够不够得着（辩方）
B1-甲 的打中是否成立（改前单测确实放得过、正式跑确实判了没对照）；第一轮「采攻方」的取舍有没有同样打中别的写法；把第一档 r3 直接建表与 r3 两份表都取的那一档拿掉，会不会让哪一条判据失去唯一一份读数。

### C3　新单测还能放过什么（攻方）
「只加不删」一类（某档多跑一条对照）新单测判不判；五档之外正式跑还有没有别的档（`--table scan`、反盘次序、r4、标记奇偶另一种）该进来；读数「全红」这条断言在每档有没有被空集放过；挂钟有没有明显变长。

### C4　四行变异与注释（攻方、辩方）
四行锚点各恰好一次、点名的测试筛得到、在改后单测下红在断言上、在改前单测下（换掉的两行）的行为；第 3497 行只有 nff 那条杀得掉的说法成不成立；三处注释与代码对不对得上。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | C1、C2、C4 | 复核第一轮判决：它采的打中够不够得着、改法是不是同样碍着别的、替被改掉的写法（并集判、r3 直接建表那一档）辩护 |
| 云端攻方（Opus） | C1、C3、C4，重点 C1、C3 | 攻第一轮的修补本身：找改后单测放过的坏法，找写死集合与登记、正式跑对不上的一格；不重复第一轮攻过的「并集顶替」那一角 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 照那一判；一次抽样的「没打中」只算这一次。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第三轮，换攻击面。
- 腿提的新判法、新用例算零轮；条款没写、要用户定的，交用户。
- 重型测试不跑：被判的是 checker 档装置二进制的内联测试，腿一样都不跑，读码判，要跑的写成命令交主 agent；harness 档测试可以跑。

交付：腿的报告写 `research/prompts/m3-formal-code-b37-r2-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b37-r2-<腿名>-model/`（有就放，带一份 `tools/rerun.sh`）；草稿放 `~/.local/share/singlefs/work/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
