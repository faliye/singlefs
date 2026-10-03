# 根环实现第 8 批：代码轮第二轮正文（2026-10-03）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 E2 B2 RR1 RR2 RR3 RR4 RR5 TF N3 S4 -->

## 一、题面

第 8 批两份实现补丁与一份改钉合进了 `crates/`（`records/2026-09-30-里程碑三开工第一轮.md` 第 8 批）。这一轮判：第一轮打中的那几处修补本身做得对不对，这一批新做的池级 checker 簇是不是条款说的，哪一格会错。第一轮判决在 `research/prompts/m3-root-ring-code-r1-main-verification.md`。

改动全文在 `research/prompts/_m3-root-ring-code-r2-diff.md`（三份，各带实现员报告的路径），冻结副本 `/tmp/claude-1000/m3-root-ring-code-r2/tree/`：

| # | 改动 | 主要落点 | 压着的条款或定案 |
|---|---|---|---|
| A | E2：进程内记根之前，挂载时读不出的槽成了窗口里最小的那个就先重读那一槽，读得出且按实例表有效照读出的 txg 算，读得出而自证不过或被抛弃当没有根，仍读不出才按根环表记的最大 txg 算；B2：抬 F 的上限、「非空」「前一条」都只在一圈窗口里的有效根上取；RR4：抬 F 那一遍的回收门槛并进挂载时读不出的槽 | `crates/singlefs-core/src/allocator.rs`、`transaction.rs` 两处记根、`mount.rs` 抬 F | D16（发布语义） 已定项 1（2026-10-03 写回的窗口六处、E2、B2 三句）；第一轮判决第三节表二 |
| B | 回退候选集加一圈窗口：`RollbackCandidateExclusion` 加成员 `OutsideTheOneRevolutionWindow`，模型一侧加拒因与映射 | `mount.rs`、`crates/singlefs-harness/src/model.rs`、`model_comparison.rs` | D16（发布语义） 已定项 1 窗口六处里的回退候选集 |
| C | RR2 卸载用例与 M-F 变异；RR5 变异行改名；影子账两读者改回不判指针头部；`mounted_read.rs` 两种拒法分成员（条目宽窄于字段表仍报 `RecordMalformed`，子指针头部一段不是第一版取值报 `PointerInTheRecordCannotBeDereferenced`）；七类走读读者判 MAC / nonce / 算法 / 压缩 | `mounted_read.rs`、`pointer.rs`、`extent_tree.rs`、`allocation_record_tree.rs`、`records.rs`、`instance_table.rs` 等 | D19（块指针的结构与宽度预算） 已定项 3；D9（加密） 已定项 14；第一轮判决 RR2、RR5 |
| D | I-3.5 出现规则（TF、β、gap、gap-row、N3）：新判法 `judge_the_appearance_rule_of_adjacent_candidate_versions`、洞判法 `hole_in_the_candidate_set_between`；走读两处修正（谱系树、墓碑树容器的落点记在根节点下面；树表共享时照样并进映射根）；I-9.25 回到 kb 字面 | `crates/singlefs-checker/src/walk.rs`、`image.rs` | `.claude/kb/invariants.md` 的 I-3.5、I-9.25 两行；`research/prompts/m3-birth-floor-r2-main-verification.md`、`m3-birth-floor-r3-main-verification.md` |
| E | I-8.10 扩到七段（CPU 与 GPU 两侧）、RR1 卸载记号其余位在 CPU 与 GPU checker 都判、RR3、GPU 内核旧格式常量 | `walk.rs`、`crates/singlefs-checker/src/lib.rs`、`crates/singlefs-checker-tier/src/crash_judge_*.wgsl` 与 `crash_judge_gpu.rs` | `.claude/kb/invariants.md` 的 I-8.10 一行（用户 2026-10-03 定扩到七段）；第一轮判决 RR1、RR3 |
| F | 改钉：I-3.5 进 `writable_mount_of_a_formatted_pool.rs` mkfs 之后与无文件两张不适用表；`crates/mutations.tsv` 第 477、1767、1916–1918 行原文改到今天的源码；8-2 新用例文件第 110 行通配臂改穷举 | `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs`、`crates/mutations.tsv` | 改钉报告 `/tmp/claude-1000/m3-r1/batch8-repin/report.md` |

## 二、实现今天的样子（主 agent 的观测，2026-10-03）

- 冻结副本取自 2026-10-03 主工作区：第 8 批两份补丁与改钉都打上之后；形式化接入那条线第 16 批（`model.rs`、`model_comparison.rs`、`mount_transparency.rs`、`crash_injection.rs`）也已合入。被判文件的 sha256 在 `research/prompts/m3-root-ring-code-r2-snapshot/crates-sha256.txt`，冻结副本逐份对得上。
- 合入后验证（`/tmp/claude-1000/merge-and-verify/batch8/`）：harness 档红 1 条（I-3.5 登记表），fmt、clippy 过；改钉之后那个测试二进制 15 过 0 红，`code-source-discipline.sh --check mutation-tables` 绿（「crates/mutations.tsv 2261 条的原文各命中源码一次」）。checker 档一样没跑。
- 已知没做、不在这一轮被判范围的（腿报出来不算打中，判决里照原去向写）：池级 checker 的 I-7.9 「之前的根」还取全环、没跟上 B2（8-1 报告第五节第 1 点、8-3 报告第一、三、六条，进第 9 批）；GPU 判器不判 I-3.5、靠临时表 `POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET` 编过（用户定改回要求全覆盖，进第 9 批）；GPU 恢复内核拼新根时实例表、分配记录树根照抄所选根、卸载记号恒假（8-2 报告第六节 S7，进第 9 批）；harness 判法 `mount_transparency.rs` 仍按取甲（进第 9 批）；I-3.5 的 ③ ④（deadlist）；`root_record.rs` 那一类读者没判 MAC 等；e171、e165 三处「311 字节头」注释；清扫器；I-7.8 认领规则；洞上 I-3.1 与取号跳过两件条款（第 9 批另开三方）。
- 8-2 自报四处条款没写、自取读法（报告第六节 S4）：① I-3.5 只判出现规则、③ ④ 没实现；② R⁻ 那一侧几条并成一组时 gap-row 取「组里任一实例接得上就算」；③ 「两者之间每个实例的行都是 (j, 0, 0)」读成每个中间实例都要有一行、T 与 W 都是 0，缺行算接不上；④ 「整对报不适用」取那一对不判、不让整条变成不适用（只有一个单元都没判到时整条不适用）。
- 8-1 自报两处条款没写、自取读法（报告第五节）：E2 判「按实例表有效」，挂载时用最新那条根的表，写行那次发布一定下来就换成写行那一张；算法类型与压缩那几段照 D9（加密） 已定项 14 判成该指针所在的结构损坏。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　E2、B2、RR4 落地（A）
三句写回条款（D16 已定项 1）与代码是不是逐字对得上：E2 重读那一刻读到的四种结局（读得出有效、读得出自证不过、读得出被抛弃、仍读不出）各自记成什么，有没有一段合法历史让记下的 txg 高于真住在那一槽的根（不安全方向）；E2 选的那张实例表在写行之前、之后各是哪一张，换表那一刻窗口最小的槽判得对吗；B2 的上限只取窗口里的根之后，F 能不能越过窗口里一条读不出的有效根（8-1 报告第五节第 1 点那一形：实现上限 22、F 越过读不出的 X）——这条在条款下是对的还是错的；RR4 用例只抬到 6，「抬到 B2 上限」那一形有没有别的用例或变异钉住。

### K2　回退候选集的窗口（B）
`OutsideTheOneRevolutionWindow` 判的锚 N 与 D16 已定项 1 其余五处是不是同一个；排除次序（`RollbackCandidateExclusion` 成员次序）里它排在哪、排错会不会让一个本该报别的拒因的候选报成窗口外；模型一侧的拒因与映射在模型对拍里有没有一条历史让两边判不同。

### K3　I-3.5 出现规则（D）
新判法与 I-3.5 一行原文、m3-birth-floor 第二、三轮判决定下的 TF、β、gap、gap-row、N3 逐条对得上吗；8-2 自取的四处读法各自在哪段合法历史上误红、在哪段坏镜像上漏红；洞判法 `hole_in_the_candidate_set_between` 的三态在有洞的环上（8-3 报告 `/tmp/claude-1000/m3-r1/batch8-investigate/report.md` 第十节那张「窗口里每个 txg 住着什么」的表）判得对吗；走读两处修正（容器落点记在根节点下、树表共享时并进映射根）有没有让别的不变量的读数变了（I-3.1、I-3.10、I-5.4）。

### K4　I-8.10 七段、RR1、RR3 与 GPU 两侧（E）
七段各比的是记录新根段与同代根的哪一段，CPU 与 GPU 两侧逐段同口径吗；RR1 卸载记号其余位在 CPU、GPU 判法与 core 读者三处是不是同一个口径；GPU 内核换了格式常量之后，有没有一处偏移或宽度还按旧头 311 / 新根段 188 / 点名项 67 算（`.claude/rules/format-evolution.md`「改一个格式常量」第 1 步那几类派生形态）。

### K5　读者与拒法（C）
七类走读读者判 MAC / nonce / 算法 / 压缩的处置与 D19 已定项 3、D9 已定项 14 原文逐字相同吗；`mounted_read.rs` 两种拒法的分界（条目宽 vs 子指针头部）有没有一种输入落到错的成员；影子账两读者改回不判指针头部之后，有没有一段合法历史让影子账收下一个 core 别的读者拒的节点。

### K6　改钉（F）
I-3.5 进两张不适用表的那两步，checker 判不适用的依据是不是 8-2 写的不适用条件，有没有把本该成立的一步登成不适用；5 行变异的新原文各自改坏的是不是那一行名字说的行为。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3、K4、K5、K6 | 逐条从条款或定案原文推到冻结副本里的行，再推到钉它的测试与变异行；第一轮站住的不重推，推这一批改了的 |
| 云端攻方（Opus） | K1、K2、K3、K4，重点 K1 与 K3 | 在冻结副本的拷贝上造反例，只跑 harness 档；不许重复第一轮攻过的角度（第一轮判决第二节各格），攻第一轮修补本身、8-1 与 8-2 自取的读法、这一批没被看过的地方 |

## 五、跑前写死的判据

- 两腿都没打中 ⇒ 站住（一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记）。
- 打中 ⇒ 主 agent 现查坐实，改法进第 9 批；第三轮只攻前两轮站住的形态。
- 第二节列为已知没做的几样，腿报出来不算打中，判决里照原去向写。
- 腿提的新改法算零轮。

交付：腿的报告写 `research/prompts/m3-root-ring-code-r2-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-root-ring-code-r2-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
