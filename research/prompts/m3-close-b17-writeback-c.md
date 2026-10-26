# 批 17 阶段同步写回（fix-c）：实验页与布局表

日期：2026-10-26。四条都落地了：第 1 条工作区已是新句、没改；第 2、3、4 条照回扫报告与派发提示改了三语。`kb-spec-check.py` 绿，`replace-batch.py` 回读一致。

## 规格

- `/home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-fix-c/spec.json`：`96c3e8ec32ed4e44facada9b0bf19715f79e696878a1cc93f1abd583e4843b7b`
- `/home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-fix-c/spec.md`（内容相同）：`0502c2cb696185bfaf05b2f2ee952cbec6742c3b40fe69c723571c042bd4ffd1`
- 喂 replace-batch 的 `replace-batch-spec.json`：`71bfa7ee68787fcc8dc2b6839276c773553b7d1b284d3e7c871f877932bf121e`
- 补一处空行的 `fix-blank-spec.json`：`9f3556ad98d84959be2a858445e632fab4c13daf1b4a8447a7d034f281a2c15b`（下文「另一批」）
- 63 条。旧串是照工作区现文生成的整行（生成脚本 `gen.py`，每条子串在那一行里先核了恰好 1 次）。
- `kb-spec-check.py` 末行原样：`  ✓ 规格 63 条（1 份）：旧串各恰好 1 次，编号引用、指代、依据配对、依据没有别的会话还没提交的改动、变更史形态都过（查了 63 条）`（rc=0；第一次跑红 9 处：新串里光写了 `A2`（登记位里 A2 是「系统配置是明文」）、变更史里光写了 `E184`，改了写法再跑转绿）。

## 逐条

1. **E142 第 22、310 行（三语）：工作区已是新句，没改。** 中文第 22 行现文「这是装置按取号写一轮量的，两态闭式 16777240，补第三态后全量 16777260；取号写两轮之后 `crates/` 里枚举的是 61 次操作、全量 16777268，本页装置没按两轮重跑」，第 310 行现文「本页装置量到的新池新建文件流是 12 单元、两态闭式 16777240、补第三态全量 16777260（取号写一轮）；今天 `crates/` 里的流取号写两轮，61 次操作、全量 16777268」；英日两份第 22、310 行同义（「two rounds … 16777268」「2 周 … 16777268」）。测试现钉：`crash_enumeration_new_pool_file_creation_stream.rs:774 const FULL_STATES_WITH_TWO_STATES_PER_WRITE: u64 = 16_777_243;`、`:778 const FULL_STATES: u64 = 16_777_268;`。E142 三份 sha256 开工与收尾相同。
2. **E184（三语）**，每种语言改了 4 处：
   - 第 158 行（「第 2 次重跑」一节候选甲、乙那一条）：「追不追由主 agent 定」改成「主 agent 定不追（乙已成）」；英「the main agent decided not to append it (B is done).」、日「メイン agent は追記しないと決めた（乙で済んでいる）。」
   - 第 246 行（岔路表第一行「还差什么」）：「候选甲追不追原登记第十二节由主 agent 定」改成「候选甲（往原登记第十二节追加）不追，乙已成」；英日照 s8 第 141、142 行的改后句。同一格里「谁改缺主 agent 定」没动（派发没点）。
   - 路径与结论登记第 5 行（中 577、英日 586）：`:762` 改成工作区现行的 `:774`。
   - 第 6 行（中 578、英日 587）：工作区已经部分是新句（A1 的 16842785 与旧段形的 16842782 都写了），我补上了另一半：前 12 段得新池新建文件流的两态闭式 16777243，并指到「做了什么」第 6 条（那一条写着「前 12 段得 43 写、16777243」）。没用 `A2` 这个记号，因为它在 kb 里登记成了别的东西。
3. **布局 01 第八节说明（中 408、英 407、日 409）**：「取号、暖机、普通发布三行与整条流那一句同产物第 36–39 行逐字相同」改成 s7 第 73 行的改后句：暖机、普通发布两行同产物第 37、38 行逐字相同；取号那一行与整条流那一句是取号写两轮之后的形状，由 `new_pool_file_creation_publish.rs` 的 `recorded_paths_match_the_registered_segment_sequences` 钉住（现查第 327、378、414 行：`(6, "2+2", 7)`、`(61, "2+2+2+1+2+2+1+2+24+2+1+2", 16_777_243)`）；产物第 36、39 行仍是取号写一轮的 `2`、16777240。末句「状态数是闭式算出来的」改成 16777243 是两态闭式、三态全量 16777268 在门禁 checker-tier-crash-point-replay 里枚举。英日照 s7 第 67、70 行的改后句，E142 补上了译名，「rig」写成术语表的「apparatus」。现查：`replay.sh:1627` 登记的仍是 r25 那份产物；`git show HEAD:` 读出它第 36 行是 `segments=2 closed_form=4`、第 39 行是 `closed_form=16777240`，第 37、38 行与表里暖机、普通发布两行一致。
4. **`deleted_inodes` 根的 key 区间（照主 agent 定案）**：
   - 布局 03 第 13 行（三语，日文那一份也在第 13 行，不是 s4 写的第 14 行）的状态格：「key 区间是格式级空白，条款没写这一格；…；key 区间归 C478（…）」改成「key 区间已定：叶层码 2 节点取首末两条条目的 key（D18（块里携带什么信息） 已定项 2）；代码 key 区间写队列首末两个号（一条 key 时 [N, N]），与条款同值；池级 checker 核不核这个根的区间见欠账表」。没写欠账号：等主 agent 收段时补。
   - E172「它答不了的」第 6 条（中 164、英 163、日 164）：「对错归 C478（…） 与 C713（…）」改成：层级的条款是 D8（核心索引结构） 已定项 16 第 3 条（td）与 D18（块里携带什么信息） 已定项 24 第 1 条（tt）；key 区间的条款是 D18（块里携带什么信息） 已定项 2（td 是叶层，取首末两条条目的 key；tt 取子树覆盖区间）；tt 的区间归池级 checker 判；池级 checker 核不核 td 的区间见欠账表。层级的两处条款取自欠账表已还清行 C713（`checks-owed.md:904`）。
5. **回看与变更史**：E184 三语「影响的决策」4 行、E172 三语 8 行，回看格前面都加上「2026-10-26 不受影响：改的是过时的现状句与指路，结论与产物不变；此前 」（英日照译）。`experiments-history` 三语的 E172、E184 两节各加了一个 `### 2026-10-26` 块，块里一个 `- ` 项，带依据与改前（`git show :<路径>`，基于 `841b3beab`）。E142 没改正文，所以它的回看和变更史都没动。

## 另一批：补一处空行

第一批插入变更史块后，新块和下一个日期标题之间多了一个空行（三份变更史各 2 处，共 6 处）。我用 `fix-blank-spec.json` 把每处的三个换行改成两个；它不加任何文字，所以没再跑 kb-spec-check。输出：`✓ 6 处都命中得对（3 个文件），--dry-run 没写`，实写 `✓ 6 处替换，写了 3 个文件（改名换上新 inode），回读一致`（rc=0）。

## replace-batch 原样输出

- `--dry-run`：`✓ 63 处都命中得对（15 个文件），--dry-run 没写`（rc=0）
- 实写：`✓ 63 处替换，写了 15 个文件（改名换上新 inode），回读一致`（rc=0）
- 回读另核了一遍：63 条的新串在目标文件里各恰好命中 1 次，0 条不对。
- 写范围闸预检：15 个路径全部退 0。
- 变更史前后各跑了一次 `git diff`：新增的 `+` 行只有这 6 个块。

## 要主 agent 判的点

1. E172 第 313 行（「H2 两行同值……对错归 C478（…），层级归 C713（…），这里不判」）和第 356 行（「第 7 行只记值……归 C478（…） 与 C713（…）」）还在指已还清的两笔账，派发只点了第 162 行，所以我没动。第 313 行还写着 td 的子树覆盖读法是 [0, 2⁶⁴ − 1]，和 D18 已定项 2「叶层取首末两条条目的 key」对不上。要判这两行算事件句还是现状句。
2. E172 正文现在引了 D18（块里携带什么信息） 已定项 2，但「影响的决策」表里没有这一行（D18 有已定项 24、18、11 三行）。加不加、关系写什么，不归我判。
3. 布局 01 第 413 行取号那一行的出处格还写「产物 `name=segments path=instance_acquisition`」，而那份 r25 产物里取号是 `segments=2`，和表里的 `2+2` 对不上。派发没点这一格，我没改。
4. 布局 03 与 E172 里「见欠账表」的号，等另一位写回员登号回来后补。

## 找不到的目标

0 条。

## 改过的文件与 sha256（开工时 → 收尾时）

| 文件 | 开工时 | 收尾时 |
|---|---|---|
| `.claude/kb/experiments/172-dry-run-of-bytes-written-by-deletion.en.md` | `d6a278c1823a2c9a…` | `142b319893a761b5…` |
| `.claude/kb/experiments/172-删除写出的字节的干跑.md` | `8337c3a641d34a77…` | `4c30691aaf5717df…` |
| `.claude/kb/experiments/172-削除で書き出されるバイトのドライラン.ja.md` | `ded7773059b81b9b…` | `a0a757d24e380357…` |
| `.claude/kb/experiments/184-cost-counts-for-the-regular-and-large-workload-tiers.en.md` | `80a3c154f23d967e…` | `260f7088dad83c64…` |
| `.claude/kb/experiments/184-常规层与大负载层的代价计数.md` | `74234d17fc638259…` | `3d2d408683ae3c8f…` |
| `.claude/kb/experiments/184-通常層と大規模負荷層のコスト計数.ja.md` | `669929bc25fa74af…` | `8c90f9f07068ea60…` |
| `.claude/kb/layout/01-first-txn.en.md` | `663304bb708bda0a…` | `9eedcf679302b9ed…` |
| `.claude/kb/layout/01-first-txn.ja.md` | `d64b1a9ee5475204…` | `62a30cff60a05c3d…` |
| `.claude/kb/layout/01-first-txn.md` | `f9bec67da3a0d880…` | `63a1dbe4c64a597a…` |
| `.claude/kb/layout/03-third-txn.en.md` | `dd8e9a72c468020e…` | `ff320b0dafd00fe2…` |
| `.claude/kb/layout/03-third-txn.ja.md` | `3c1afa2f1b73752f…` | `54ec533c160d04e5…` |
| `.claude/kb/layout/03-third-txn.md` | `54a0db36f089f2ac…` | `a726a5900cd5d6a2…` |
| `.claude/kb/experiments-history.en.md` | `012996073ad1abd2…` | `1937d29b846d2a0d…` |
| `.claude/kb/experiments-history.ja.md` | `16790aa9a3751ab2…` | `77542e263f927911…` |
| `.claude/kb/experiments-history.md` | `c3b13ea600d38e3f…` | `d5d16c7be3903166…` |

全长哈希在 `sha-start.txt`、`sha-end.txt`（同一目录）。E142 三份、`decisions.md`、`decisions-history.md` 开工与收尾相同。

## 推翻条件

- `crash_enumeration_new_pool_file_creation_stream.rs` 第 774 行不再是 `FULL_STATES_WITH_TWO_STATES_PER_WRITE`，E184 第 5 行的 `:774` 就错了。
- `replay.sh` 改登记了取号两轮之后重出的 E142 产物，布局 01 那段「产物第 36、39 行仍是取号写一轮」就要改。
- 调度记录里主 agent 定的不是「候选甲不追」，第 2 条不成立。

## 没做什么

- 没跑 `doc-decisions.sh --write`（派发写明是并行写回），没跑门禁与三语六格，没提交。
- 没判定案对不对，没加 D18 已定项 2 那一行关系，没动 E172 第 313、356 行和布局 01 第 413 行。
- 没登欠账号（归另一位写回员）。
- 没建仓副本或编译目录；草稿目录里只有规格、生成脚本与几份文本输出。
