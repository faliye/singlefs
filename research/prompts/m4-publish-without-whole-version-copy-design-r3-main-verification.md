# m4-publish-without-whole-version-copy-design-r3：判决（2026-11-18）

正文 `research/prompts/_m4-publish-without-whole-version-copy-design-r3-body.md`，开工快照 `research/prompts/m4-publish-without-whole-version-copy-design-r3-snapshot/`。设计推论轮第三轮（最终轮）；前两轮判决 `research/prompts/m4-publish-without-whole-version-copy-design-r1-main-verification.md`、`research/prompts/m4-publish-without-whole-version-copy-design-r2-main-verification.md`，同一件事更早的一轮 `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/m4-publish-without-whole-version-copy-design-r3-sonnet-output.md` | 前两轮的甲只换三张表，`data_pointers`、各树形状、整版克隆仍整份拷，甲之后还剩 15 处随全版涨；不动形态两批里记录索引、记录一侧撤销、叶缓存簿记在甲之后白做，规划器与结算两半的骨架留；冻结重发三行不改；分步 0 到 7；交用户六条 |
| 攻方（Opus） | `research/prompts/m4-publish-without-whole-version-copy-design-r3-opus-output.md`，探针与脚本 `research/prompts/m4-publish-without-whole-version-copy-design-r3-opus-model/` | 已回收集是记录表的子集、有界、不是泄漏，第二轮判决「只涨不缩」不对；T5 的 0.861538 不在交错写段；块长换 16、256 不翻；整理发布装置走得到、改副本量；第二轮判决表里的数 58 处逐字对上；甲与不动形态两批三处并不住 |

核查：

- `task-server/task submit --cwd ~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-model-rerun -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m4-publish-without-whole-version-copy-design-r3 --out ~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-model-rerun/m4-publish-without-whole-version-copy-design-r3-tree --repository /home/fy5090/code/singlefs`，任务 19998 退 0。
- `python3 scripts/agents/cite-check.py research/prompts/m4-publish-without-whole-version-copy-design-r3-sonnet-output.md research/prompts/m4-publish-without-whole-version-copy-design-r3-opus-output.md --root ~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-model-rerun/m4-publish-without-whole-version-copy-design-r3-tree --background research/prompts/_m4-publish-without-whole-version-copy-design-r3-background.md`，末行「✓ 核了 51 处引文、67 处裸行号，对上 6 处，没判 112 处（其中裸行号没有引文可比 0 处；逐处列在上面）」。快照只记 kb，引 `crates/` 的行它都列成没判；判决只引主 agent 现读过的行与复跑出的读数。
- 主 agent 复跑（攻方的脚本拷到 `~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-model-rerun/model-r3` 再交）：
  - 按块长重算第二轮的探针产物，任务 19985 退 0：`name=block_size_flip … reading=T1_allocation_records block=16 copy_over_n_median_low=0.064896 copy_over_n_median_high=0.072056 threshold=0.25 flipped=no`、`block=64 copy_over_n_median=0.025178 … flipped=no`、`block=256 copy_over_n_median=0.032199 … flipped=no`；T4 三种块长同样 `flipped=no`。
  - 整理发布上的探针：在攻方建的副本上交装置（造完池接着跑 CPU 臂的整理），任务 19986 退 0，产物 12330 行；汇总任务 19991 退 0，末行「✓ 完整性闸过、量到了带搬迁的发布」，三个规模的 `name=compaction_run` 行都是 `outcome=ok … integrity=yes`。
- 主 agent 现读（主工作区）：`crates/singlefs-core/src/transaction.rs:1662`–`:1663`「let allocator_after_the_publish = std::mem::replace(allocator, allocator_before_this_publish);」、`:1705`「*allocator = allocator_after_the_publish;」、`:16233`「allocation_records.sort_by_key(AllocationRecord::sort_key);」。
- 核不动：攻方读已回收集的那几处加、删的路主 agent 没逐处读（只现读到 `crates/singlefs-core/src/allocator.rs:501` 的 `segments_reclaimed_not_yet_discarded` 是另一份按段的集合，攻方说的那一份没定位到）；「6573 次发布里 1849 次是缩的」是它在草稿目录跑自己的脚本读的，主 agent 没复跑；正推的 15 处、分步每步的改动量、内存估算都是推的；甲与不动形态两批并不住的三处是两腿读补丁推的，主 agent 没读补丁。

## 二、判决

更正第二轮判决两处：

1. 「已回收集只涨不缩」不对（攻方读码、主 agent 没复核，按线索采其方向）：它跟着分配记录表涨，记录表里已释放的记录要留到那个落点被复用为止；它不是另一个缺陷。第二轮记的「第三轮看它该不该清」到此关掉，去向并进甲的分步（分配器其余字段那一步）。
2. T5 的 `leftover_over_records_max=0.861538` 出在「放开垫片的槽」那一段，不在交错写段（攻方报交错写段最大 0.791809，主 agent 没复算）；翻面的结论不变。

整理发布上的读数（任务 19991 的汇总行；带搬迁的发布，每批搬的单元中位 4）：

| 规模 P | 带搬迁的发布数 | 单元 d 中位（全版中位） | 映射 d 中位（全版） | 分配记录 d 中位（全版） |
|---|---|---|---|---|
| 1024 | 130 | 19（1141） | 25（1127） | 80（16342） |
| 2048 | 124 | 20（2266） | 28（2241） | 84（32930） |
| 4096 | 132 | 20（4517） | 28（4470） | 86（66096） |

跑前写死的翻面线：C1（分配记录）`copy64_over_n_median=0.025306`、C2（映射）`0.072931`、C3（单元）`0.072393`，线都是 0.25，`flipped=no`；C4（d 随全版涨）三张表 `d_growth` 1.052632、1.120000、1.075000 对 `n_growth` 约 3.96 到 4.04，`flipped=no`。

三轮多数判：

- 表示取甲：`TransactionOutput` 里随池涨的表换成增量更新时与上一版共用没改的块的持久表，版本仍是值、读得到全版。三轮里乙（版本只带差量）、丙（差量加上一版引用）在没加强的形态上两轮打中，加强的形态零轮、没有模型；甲三轮没被打掉，量过两种发布：造池三段（任务 19957）与带搬迁的整理发布（任务 19991），一次发布改到的条数不随全版涨，块长 16、64、256 下都不翻（任务 19985）。射程：一个档（2288MiB）、规模到 4096；「一次改很多」的发布（删或截断大文件、整文件覆盖）没量，那一类 d 本身随文件大小涨，任何表示都躲不掉。
- 甲的范围比前两轮写的宽（正推，采）：只换单元、映射、分配记录三张表不够，版本里其余随池涨的字段与分配器的其余字段都要共享，建树与装配要改成只交改到的，才到「一次发布只与改到的有关」。逐处的清单在正推报告「甲之后还剩什么随全版涨」一节，主 agent 没逐处核，实现时照它逐处过。
- 冻结重发不用动（两腿一致，主 agent 现读 `transaction.rs:1662`–`:1663`、`:1705` 属实）：发布前那一份是克隆、发布后那一份是原地改的，落盘失败冻结的仍是后者；持久共享之后克隆变便宜，这三行与 D16（发布语义） 已定项 17 的字面不改。2026-11-06 那一轮 Q2.3 ③ 判做不到的是「换回滚日志」，甲不走那条路。
- 不动形态的两批与甲的关系（两腿一致，推的）：两批里的规划器（中央映射树只按改到的 key）与结算拆两半的骨架留得住；分配器的记录索引、记录一侧的撤销、叶缓存的簿记在甲之后要推倒重写；补丁新加的索引会让发布前那份克隆更重。两批的价值是在甲做完之前让 E202 的最小档造得出池，以及留下对拍用的参照写法。合入次序：两批先合、甲后做（正推推荐，零轮）。
- 那条恒开的整表断言：照第二轮的读法（位置不动、比的范围可以缩），够不够格算「全量遍历保留为参照」仍归两批的代码轮。
- 攻方「甲最可能按分配记录树的叶分块，这个量没量」：它在副本里补了按叶计数，整理发布上分配记录 `copy_by_leaf_median` 是 0.248129（P=1024）、0.147911（2048）、0.074602（4096），随规模降；P=1024 贴着 0.25 的线。按叶分块在小池上不便宜，记进分步那一步的验收。

第三轮之后不再开轮。

## 三、改法

- 甲（范围照上面第二条）：量过两种发布、一个档、三种块长；整文件覆盖与删大文件那一类没量。
- 分步 0 到 7（正推报告「分步终稿」）：零轮，实现时每步一轮代码三方。
- 删掉已回收集、改看记录是否已释放（攻方）：零轮。
- 角色到指针所在处的写死对照表、相等按内容比、另立单元的键：零轮（另立的键有第二轮 T3 与这一轮整理发布上的读数撑着次序那一半）。
- 两批先合、甲后做：零轮。

## 交用户的

1. 持久容器自己写还是引外部包。自己写：分配记录按叶分块约 150 到 250 行、单元与映射的有序持久表约 400 到 700 行（推的），不动依赖；引包：候选 imbl、im、rpds、immutable-chunkmap，本机离线库里没有这几个包，许可与缺口在第二轮辩方报告；也可以分配记录自己写、另两张引包。
2. 甲做不做、何时做。两批补丁在副本上量到 P=4096 带计时的建池 63.438 秒（改前 193.899 秒）、P=8192 是 185.673 秒（任务 19984，P=16384 那一半还在跑；都在本机负载很高时量的）。E202 登记的最小档是 36608 个活数据单元；两批够不够把它拉进四小时上限、较大的两档够不够，要等 19984 的另一半与机器闲时的实量。候选：两批合入后先实量，够用就把甲留到里程碑八；或现在就做甲。
3. 分步先换哪张表（分配记录先、单元与映射先、空闲图先），翻面观测是较大规模上分配记录条数与单元数之比（正推报告「交用户的清单」第 2 条）；这一条等第 2 条定了做甲再问。

## 这一轮被判过的文件

- .claude/kb/milestone/04-fourth-txn.md：判了「发布不再整版照抄」那一行的做法定为没改的与上一版共用块的持久表示、范围比那一行点的三处宽、与不动形态两批的先后；那一行与进入条件等用户对上面第 2 条定了之后写回。
- .claude/kb/decisions/16-发布语义.md：判了已定项 17 在持久共享的表示下字面不用改（落盘失败冻结的仍是发布之后那一份分配器）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 17 | 不影响 | 2026-11-18 不受影响：甲下发布前是克隆、发布后是原地改的那份，冻结重发那三行与条款字面不改 |
| D3（空间分配） 已定项 7 | 不影响 | 2026-11-18 不受影响：已释放的记录留到落点被复用是这一条定的，已回收集随它涨不是缺陷；只换内存里的表示，不碰盘上编码 |
