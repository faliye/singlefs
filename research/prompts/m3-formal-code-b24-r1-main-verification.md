# 第 24 批代码轮第一轮判决（2026-10-05）

<!-- doc-lint:not-numbers K1 K2 K3 F3 H2 A1 B1 A2 B3 C2 M7a M7b M7c M7d M7e M7f M7g M7h M7i -->

被判的文件：`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs`、`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs`、`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`、`crates/singlefs-core/src/root_record.rs`，`crates/mutations.tsv` 24-1 追加的 7 行。

这组用例（崩溃注入第二、三截的接线、发布臂与 F3）经第 18、20、21、23、24 批五次代码轮，这是最后一轮（`.claude/rules/three-way-inference.md`「第三轮之后停」）：这一轮新冒出的零轮形态写进交用户表或记账，不再开轮。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b24-r1-body.md`、`_m3-formal-code-b24-r1-diff.md`、`_m3-formal-code-b24-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-formal-code-b24-r1-snapshot/`（crates 288 份，冻结副本 `/tmp/claude-1000/m3-formal-code-b24-r1/tree/`；kb 2 份） | 冻结副本对 crates 快照全过，kb 对快照全过 |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b24-r1-sonnet-output.md`（sha256 cac2d63d…），模型 `…-sonnet-model/` | 引文 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b24-r1-opus-output.md`（sha256 4719d54e…），模型 `…-opus-model/`（30 个文件） | 主 agent 照它的 `tools/rerun.sh` 在会话草稿 `b24-opus-rerun/` 复跑，见第四节 |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root <冻结副本> --root <主仓> --background <背景材料>`：「✓ 核了 82 处引文，对上 82 处，没判 0 处（逐处列在上面）」 |

两条腿第一次派出时都撞了用量限额、报告没落盘；重派的两条腿读了前一条的交接摘要（会话草稿 `handover/`），已落盘的模型核过之后接着用，报告与 `out/` 下的数都是重派那一条跑的。

主仓在快照之后有 8 份被别的会话改过（10-04）：`crates/mutations.tsv`、`crates/singlefs-checker-tier/tests/` 下三份固定脚本流用例、`crates/singlefs-core/src/write_request_split.rs`、`crates/singlefs-harness/tests/` 下三份。腿读的是冻结副本；两份报告只引到其中的 `crates/mutations.tsv`，引的是 24-1 那七行，冻结副本第 2436–2442 行在主仓里逐行原样都在（`grep -F -x` 7 处命中）。

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| K1 F3-d 照判决做了没有 | 一致：A、B、C 取样与 b23 攻方 `f3-fix.txt` 逐项同，六行原文在冻结副本各命中一次，剩余种子数 1、1、4096、4095、4095、13254 逐格同 | 同（数与正文第二节、用例注释逐个同） | 站住 |
| K1 六行变异各红在哪 | 部分：点名那条都红；按位拼接、或代异或两行另红 C | 同；C 的注释自己写了这两个数 | 站住：变异行点名一条，点名那条红就够 |
| K1 取样覆不覆盖真历史 | 冲突：段号到 100 / 222（384 槽、16 / 48 步），4 GiB 档段长到 30；判决自己定的就是 0..=63 与 28，不是 24-1 改错 | 打中：战役大档 3942 个第一次崩溃态里 2018 个段号大于 63、最大 210（候选段 235）；段长 28 够、但 B 只变末 12 个位置；C 的两个历史种子只差最低位 | **坐实**（复跑 `segments.txt`、`real-*.txt` 逐字节同）。两腿方向一致、取样不同（正推 harness 探针、攻方链 checker 档库的探针），数各自对得上自己的产物 |
| K1 第七种有害改坏 | 有，四类：段号取低 6 位、只折前 28 / 29 个写、历史种子取低位、只折段末 12 / 16 个写；前三类加强取样转红，段末窗口那类仍绿 | 打中：七行三条全绿而有损（段号低 6 位稠密撞 1890、按位拼接落第 6 位起 1258、只折末 12 个写 310、跳过段内第 12..16 个写 120、只折末 16 个写 13、历史种子取最低位 / 截成 u8 损在跨历史）；另两行（只折前 28、段号截 u8）今天量不出损 | **坐实**（复跑 `f3.txt` 逐字节同，`summarize_f3.py` 汇总与报告 K1 那段逐行同）。两腿点名的改坏一一对得上（正推「段末 12 / 16 窗口」= 攻方 M7a、M7b） |
| K1 改法 F3-e（A2、B3、C2） | — | 攻方提：A2 段号 0..=63 再加 2^k、2^k + 1（k = 6..=20）共 384930 个种子两两不同；B3 长 32 的段扣下 1 或 2 个写的 528 个子集；C2 历史种子 0、1、2^63。入库那一种三条全绿，登记七行与攻方九行每行至少一条红 | **零轮**：只在攻方模型上量过（复跑汇总里每一行都对得上：M7a–M7i 在 A2 / B3 / C2 至少一列红，入库那一种三列全绿）。照最后一轮的规矩不再开轮：它只加宽测试取样、不改实现，进第 26 批实现员，标「被攻过零轮」，并给它自己的检查记一笔账（第三节） |
| K2 B1-c 钉没钉住「建 0 个」 | 一致：旧用例绿的 244 格（漂移网格）里前提成立 ⇔ 变异红，0 分歧；两边调的是同一个 `publish_new_inodes(…, 0, …)` | 没打中：宽扫 20898 份盘面，带 B1-c 用例绿的 606 份两行变异都红；B1-c 让旧用例转红的 654 份全是旧用例本来就放过「建 0 个」的 | 站住（复跑 `b1-slots.txt`、`b1-wide.txt`、`b1-summary.txt` 同）。前提与变异臂调同一个函数：前提直接问「那一支建 0 个在这份盘面上做得成吗」，正是变异能不能分开的条件，不需要两条独立路 |
| K2 四处注释 | 一致：模块文档三句、H2 常数注释（3k − 1 从 k ≥ 2、S 取 5–15、今天 S = 8）、`root_record.rs` 卸载记号、B1 文件第 41 行都对得上 | 没打中：S = 5、8、15 上 k = 2..=40 都是 3k − 1、k = 1 是 3，S = 4、16 编不过；C670 模块文档「换一份盘面可以为真」量到判 `Illegal` | 站住（复跑 `h2-txg*.txt`、`h2-compile-s4/s16.txt`、`c670.txt` 同） |

## 三、去向

1. 24-1 与主 agent 合入后改的那句注释站住，不回改。
2. 第 26 批实现员：F3-e 三条用例（A2、B3、C2，原型 `research/prompts/m3-formal-code-b24-r1-opus-model/probe/opus-b24-probe/src/main.rs` 的 `f3_proposed`），变异照搬攻方 `tools/opus-mutations.tsv` 的 M7a、M7d、M7f 三行（各点名 B3、A2、C2），可以替掉 B 或与 B 并存；标「被攻过零轮」。
3. 记账：F3-e 被攻过零轮，它自己的取样（段号 2^k、段长 32、历史种子 2^63）对更长的历史、更多步数、别的种子基罩不罩得住没人攻过；写回时立一笔欠账。
4. 提交时：23-1、24-1 与第 26 批的 checker 档用例、变异整表。

## 四、复跑核对

主 agent 在会话草稿 `b24-opus-rerun/` 照 `WORK=<草稿> bash research/prompts/m3-formal-code-b24-r1-opus-model/tools/rerun.sh all` 复跑，harness 那一段第一次内存排队退 252（别的会话 32G 的 checker 档在跑），把 `RUN_WITH_MEMORY_CAP_WAIT_SECONDS` 放到 1800 秒补跑 `harness`、`h2s-setup`、`h2s 5`、`h2s 15`、`h2s-compile 4`、`h2s-compile 16`，退 0；模型目录 `out/` 下 16 份与复跑产物逐个 `cmp` 相同；`summarize_f3.py` 对复跑产物的汇总与报告 K1 那段同；`h2-txg*.txt` 三份 `holds=false` 各只 1 行（k = 1）；`c670.txt` 的 `c670-other-disk` 行同。

核不动：checker 档用例与变异没跑（24-1 的 7 行在入库装置上的红绿归提交时）；攻方探针链 checker 档的库跑真历史与挂载，没起 checker 档包的测试目标与装置，数照「副本上的数」引，不进 kb；两腿「没打中」各格只抽一次。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | F3-e（A2、B3、C2）与三行变异 | 攻方 | 只在攻方模型上量过；主 agent 复跑同 | 第 26 批实现员，标零轮；记一笔欠账；不用用户定 |

不涉及要用户定的岔路。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-10-05 不受影响：判别力缺口在用例的取样，不在条款 |
