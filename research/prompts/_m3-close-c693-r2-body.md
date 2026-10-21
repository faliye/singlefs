# m3-close-c693-r2：K 代撤回之后的四块重推（推论轮第二轮）正文（2026-10-20）

<!-- doc-lint:not-numbers V1 V2 V3 V4 -->

## 一、题面

欠账 C693（K 代撤回之后要重推的论证）。第一轮判决 `research/prompts/m3-close-c693-r1-main-verification.md`；两腿报告 `research/prompts/m3-close-c693-r1-sonnet-output.md`、`research/prompts/m3-close-c693-r1-opus-output.md`，攻方模型 `research/prompts/m3-close-c693-r1-opus-model/`（三份纯算术脚本，主 agent 复跑逐字节相同）。第一轮正文 `research/prompts/_m3-close-c693-r1-body.md`，问题单 `research/prompts/c693-forks.md`。

第一轮之后新到的两份证据：

| 证据 | 回答 | 出处 |
|---|---|---|
| E186（不认识的树·记账维重推）第二次跑 | U5 改判据之后 Q1 的胜者形态 | 重跑登记 `research/prompts/e186-preregistration.md` 第 659–983 行；产物 `research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz` 第 186276–186278 行：两个 `r_tree_dim` 分组各 3200 点、`legal_pooled` 6400 点都是 `form=unique_carry consequence=carry_stands`；实验页 `.claude/kb/experiments/186-不认识的树-记账维重推.md`「第二次跑」一节。**不是盲测**：判据在第一次跑的产物之后改，攻方第一轮已在第一次跑的产物上按同一判据重算出同一方向 |
| 调查：树表条目带读者不认识的 flags 位 | 第一轮 W2 实现出入的后果 | 报告 `~/.local/share/singlefs/work/m3-close-c693-w2-unknown-flags-investigation/report.md`（sha256 e6d20d8f6d34dc7c19ed6ccb5964ee162d0a98712c470705b20459ebc22a6062），调查用测试在同目录 `repo/crates/singlefs-harness/tests/tree_table_entry_with_an_unknown_flags_bit_in_the_newest_version.rs`（11 条，2 条 `minimal_` 最小复现，主 agent 看过原样输出两条都 ok） |

调查的读数（量过，副本）：系统配置见证了最新一版 R 时，恢复、只读挂载、可写挂载三处都拒（`UnitMalformed { what: "树表条目" }` 一族）；没见证 R 时（崩在 R 的根槽之后、轮换之前；或 R 已确认而 R 那次轮换写的系统配置槽两块盘都读回全零），三处都经 F2 退到上一版并做成，可写挂载在上一版上取号，挂载返回 Ok、不报错，R 写下的内容读不到。同一字节真损坏（校验和不重算）时没见证两种盘面三处结局逐项相同：三处的退回分支接住任何走读失败、不看失败种类（`crates/singlefs-core/src/recovery.rs:7116`、`crates/singlefs-core/src/mounted_read.rs:655`、`crates/singlefs-core/src/mount.rs:8016`）。今天的写者每次发布在根槽 FUA 之后无条件轮换系统配置槽（`crates/singlefs-core/src/transaction.rs:1533`、`:1537`）。

## 二、实现今天的样子

同第一轮正文第二节，`crates/` 没变。第一轮两腿已列的实现事实照引第一轮报告：记账树每次发布按 key 集合之差整棵重规划（`crates/singlefs-core/src/code_two_tree.rs:655` 起删上一版全部非台阶 key、插这一版的）；`crates/` 里没有 write buffer 前端；记账树叶容量 477、内部 150（`crates/singlefs-core/src/transaction.rs:2561`）；未知 flags 位让整张树表读不下去（`crates/singlefs-core/src/records.rs:753`、`crates/singlefs-core/src/recovery.rs:2804`）。

方案按 `crates/` 今天的实现来谈。

## 三、格

### V1　D8 已定项 8 ㊀
E186 第二次跑判 `unique_carry`、㊀ 不翻面。这一格攻：
- 这个判决撑不撑得住（不是盲测、判据跑后改；第二次跑只改了 U5 与后果表两处，其余照第一次）；第一轮攻方 W1-丁「第 7 位按节点数分不开」作为代价信息怎么写进 ㊀ 的射程。
- 第一轮正推提的两处措辞改动（正推报告第 1.5 节）：㊀「只换代号」对台阶行不成立、射程「不搬的代价」在一代口径下说得过头；改后条文逐句写出、攻它们。
- 第一轮攻方 W1-戊「按统计量的一条规则」（零轮）：㊀ 站住之后还要不要。

### V2　㊁ 与实现的出入
㊁ 第一轮站住。调查坐实：今天未知 flags 位与单元损坏走同一条路，没见证最新一版时三处静默退到上一版。这一格判：
- 改法写成对 `crates/` 的改动：不认识的 flags 位做成 ㊁ 的「该条目 EIO、位置还在」（对象级），不进整表拒、不进 F2 退回；或另有改法。每个改法写明它在调查的三种盘面上各落哪一版。
- 与根环这一簇已定的 F2 改法（`research/prompts/m3-close-rootring-r3-main-verification.md`：C755 并集判退、C751 共用实例表指针才收）有没有交集；「退回分支不看失败种类」要不要分种类。
- 落点：随哪一步实现（第一轮正推说随 deadlist 合并那一步，攻方说欠的是实现）；有没有要立的欠账。

### V3　D5 已定项 4 的依据句、「树高」与「代价曲线」几句
取舍第一轮站住。以第一轮攻方 `out/w3.out` 那几行为准，把 D5:145、D5:146 两句依据与「树高恒为 2」（D5 第 42、159、169 行，以第一轮正推报告第 3.4 节点的行为准）、「段宽不构成代价 / 代价曲线是平的」逐句写出改后条文；攻它们与产物对不对得上、有没有在一个取样点上说全称。

### V4　点删与墓碑
第一轮判 D5 已定项 3 第 3 行与 C344 不销账。这一格判：
- 缩成什么题：并进 D8 已定项 13（同一次发布同 key 中间态折叠，C383）还是单列；检查换成什么形（第一轮攻方 `out/w4.out`：朴素取最大折叠 20 次发布行数 16 → 35，I-3.13 语法判、条数判、I-9.30 ① 都 0 红）。
- 删一个可写头时它的台阶行怎么离开记账树：条款空白，写出候选与各自的检查；与 `crates/singlefs-core/src/transaction.rs:3682` 的 TODO 是不是同一件事。
- 两条各落哪个里程碑作业点（`.claude/kb/milestone/` 下，照欠账表现有的落点写法）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | V1–V4 | 把四格的改后条文与改法逐句写出：条款改哪几行、改成什么，`crates/` 改哪一处；第一轮已列的不重列 |
| 云端攻方（Opus） | V1–V4 | 攻第一轮的修补本身、产生结论的方法、从未被看过的地方；不重复第一轮攻过的角度（读法 B1/B2、U5 空真、按节点数、`w_gens` 观测不到、宽读 E71 过界、树高只在一点量过、前端逼出墓碑） |
