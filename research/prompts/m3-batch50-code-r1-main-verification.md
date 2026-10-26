# m3-batch50-code-r1 判决：删除线 W3 + 第 50 批（M3、叶死亡退役树）代码轮第一轮（2026-10-09）

被判的改动：主工作区合入的 W3 + 第 50 批补丁（`~/.local/share/singlefs/work/m3-batch50/m3b/patch/`），`crates/` 下 70 个文件、变异表追加 71 行、整行替换 45 行、删 2 行。按路径点名改过的源码：`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker/src/lib.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/bad_disk_input.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs`、`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`、`crates/singlefs-core/src/admission.rs`、`crates/singlefs-core/src/container_tree.rs`、`crates/singlefs-core/src/inode_tree.rs`、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-core/src/lineage_tree.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/records.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/retirement_tree.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/unit.rs`、`crates/singlefs-core/src/unlink.rs`、`crates/singlefs-core/src/write_accounting.rs`、`crates/singlefs-format/src/lib.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/src/mount_transparency.rs`、`crates/singlefs-harness/src/scenario.rs`；GPU 判器 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crash_judge_checker_common.wgsl`、`crash_judge_checker_scan.wgsl`、`crash_judge_common.wgsl`。合后验证打出的红由第 52 批补丁（`~/.local/share/singlefs/work/m3-batch52/fix-reds/patch/`，改 `crates/singlefs-core/src/recovery.rs`、`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` 与六份 checker 档测试、一份 harness 测试）关，那一批随本判决的改法一起合、一起进下一轮。

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 开工快照 | `sha256sum -c research/prompts/m3-batch50-code-r1-snapshot/crates-sha256.txt`（主工作区与冻结副本各一次）、`kb-sha256.txt` | 71 份 crates 与 11 份 kb 全对，没有别的会话改过 |
| 正推腿 `research/prompts/m3-batch50-code-r1-sonnet-output.md`（464 行，sha256 4fb94fa89c1e…）、攻方 `research/prompts/m3-batch50-code-r1-opus-output.md`（233 行，sha256 6091663686ca…） | `cite-check.py` 两份合判，`--root` 冻结副本与仓根，`--background` 背景材料 | 「✓ 核了 163 处引文，对上 163 处，没判 0 处」 |
| 攻方第二次抽样 `research/prompts/m3-batch50-code-r1-opus-s2-output.md`（309 行，sha256 a41d71fe7086…） | 同上 | 「✓ 核了 19 处引文，对上 19 处，没判 0 处」 |
| 攻方模型 | 拷到 `~/.local/share/singlefs/work/m3-batch50-code-r1-verify/`，`rerun.sh … all`，rc=0 | compare 九份「一致」（churn-totals、closed-early、edge-mut、edge-orig、m3-core-lib、m4-absorb、m4-core-lib、m5-random、m5-rollback-all） |
| 第二次抽样模型 | 拷到 `~/.local/share/singlefs/work/m3-batch50-code-r1-verify-s2/`，逐阶段从头复跑 | 见第六节 |
| 主 agent 现查 | 读冻结副本 | `crates/singlefs-core/src/inode_tree.rs` 第 462–468 行先 `right_pair_fits` 后 `left_pair_fits`；`crates/singlefs-checker/src/walk.rs` 第 202–212 行登记退役记录宽 34、第 3523 行按登记宽比；`crates/singlefs-harness/tests/checker_known_bad_images.rs` 没有改退役容器记录宽的坏镜像；`.claude/kb/invariants.md` 第 32 行 I-1.7 写「类型 5（谱系）是 16」、没有类型 3；`scripts/experiment-apparatus/e176_crates_oracle/oracle.rs` 第 124、125、183 行读已删的 `containers_dropped`（独立 Cargo，`scripts/experiment-apparatus/replay.sh` 第 324、325 行只登记 E176 主装置） |
| 合后 checker 档（主 agent 在副本 `merge-copy/repo` 上跑，不是腿的数） | `checker-quick.log`、`feature-gated-nff.log`；打第 52 批之后 `checker-quick-b52.log`、`feature-gated-b52.log` | 打第 52 批前 CPU 8 红、GPU 6 红；打之后 CPU 61 个目标 658 过、1 红（根环线那份 nff 装置钉值，归它的第二十六块），GPU 20 个目标 491 过、0 红、NOT_RUN 0 |

## 二、逐格判（三份样本合看）

| 格 | 正推 | 攻方 s1 | 攻方 s2 | 判 |
|---|---|---|---|---|
| K1 Opt2 滤法与五种形状 | 一致 | 没打中（形状）；**打中判别力**：环那一项在走得到的状态上不大于 cur，去掉它（M5）回退用例 25 条与随机历史全绿（主 agent 复跑 `m5-rollback-all.out`、`m5-random.out` 一致） | 没打中（1 008 段定向扫、70 次抛弃） | 行为三份都没打中，**坐实一处判别力缺口**：Opt2 那一项没有会红的测试。它是用户定案里的一项，不删，补构造态单测（第三节 F3） |
| K2 台阶行、谱系、恢复要求、ckpt_cost | 一致 | 没打中 | 没打中（65 548 个崩溃状态上 I-3.13、I-9.30 0 红；C769 给了一张推的坏镜像） | 两次攻方都没打中，过 |
| K3-1 登记 | 一致；三处注释残留「十一条」 | 没打中 | 打中：`crash_points_tree_identifier_watermark_orphans.rs` 还断言 65548 | s2 那一处与主 agent 合后跑出的红同一处，第 52 批已改成 65549、副本复跑转绿；注释三处并进第 53 批 |
| K3-2、K3-3 死亡写侧、回退、恢复 | 一致 | 没打中（181 次退役逐次比对） | 没打中（138 856 次删除类发布、退役完备判据 0 红） | 过 |
| K3-4 容器树一次追加多条 | 一致 | **打中**：退役容器（与谱系容器）不满就关，池级 checker 一条不红；墓碑同形红 I-9.21（主 agent 复跑 `closed-early.out` 一致） | 没打中（它没造这一形） | 坏镜像漏报坐实（不分辨批次，谱系树早就同样没有）。补判法（F2） |
| K4-1、K4-2 M3、R3 | 一致（与 E176 M3 臂 40 种子 × 9000 步逐步相同） | 没打中 | 没打中 | 过 |
| K4-3 N1 | I-9.12 仍拦改路由的错 | 拦不住区间里别的值，写侧有用例钉 | 同，记射程 | 用户定 X4 时已知的代价，不算打中 |
| K4-4 模型与实现 | 冲突：`model.rs` 没有 M3，E176 对拍装置编不过 | **打中**：「先右」改「先左」（M4）core 240 条与发布路径 4 条全绿（复跑 `m4-core-lib.out`、`m4-absorb.out` 一致） | **打中**：三片各 117 条删中间一条，两种写法退役不同的叶（467 对 234），checker 都不红 | 三份一致坐实：M3 的方向没有会红的检查，对拍模型没有独立参照。补（F1） |
| K5-1 取哪一版、W | 一致 | 没打中（新写最多 19 槽，W 36–39） | 没打中（60 141 次删除类发布每次至多一条退役记录） | 过。正推另报「不满足相邻两片之和 ≥ 234 的树一次可退 116 条」：那棵树在 M3 下走不到（两次攻方都量到至多一条），而没有不变量判这个前提，W 的上界压在它上面，记欠账（第四节） |
| K5-2 贴边建满后 unlink 不超卖 | 规则没说；它的读法下 384 槽小盘没造出 | 没打中（读法「每次删除都发成 ∧ 删完稳态可用 ≥ 0」，四个起点 104 段，最少 18.9 槽；去掉退役各项 M2 照绿） | 没打中（它自起名为「乙（加撇号）」的读法：放过的写写得下、删除与随后的释放不被落点拒、物理空闲不少于 ckpt ÷ 副本 + 挂载期；四条臂扫到边；实现员候选甲在两条臂上都是负的、分不出退役项） | 两次攻方用两种读法都没造出超卖，去掉退役项也都不红：会红的用例在这块盘面上写不出。取实现员候选丙：不做这条用例，退役预付由 `admission.rs` 单测钉值与变异行罩着。「删除类发布之后的可用怎么读」条款没写，记欠账（第四节） |
| K5-3 常驻 3 槽与实占 | — | 开第一片退役容器后稳态可用永久少 1 槽（墓碑 3 槽），被 defer 盖住 | 同，量过两格的差，判为保守双扣 | 与 D3 已定项 2 现行式子的写法一致（墓碑树同形），不是超卖，不改 |
| K6 checker 与 GPU | 一致；I-1.7 对退役宽 34 没有坏镜像，invariants 第 32 行陈旧 | 没打中（GPU 没跑） | 没打中（GPU 只读码） | GPU 那一半由主 agent 在副本上跑：打第 52 批后 491 过 0 红。I-1.7 退役宽补坏镜像、kb 那一句改（F4、F5） |
| K7 判别力 | 98 行非 checker 档变异全红；两处缺口（先右后左、I-1.7 类型 3 不判宽） | M4、M5 照绿 | 三条照绿：先左后右、I-9.32 去「容器出生代 < 死亡 txg」、去「死亡 txg ≤ 这一版」 | 坐实五处缺口，各补一条会红的用例并留变异（F1、F3、F4） |

顺带（s2 第 275–278 行）：在内存稀疏盘上用会话造「崩溃恢复抛弃最新根」，抛弃那一刻池级 checker 红 I-3.1，不删任何 inode 的对照也红，与删除线无关，主 agent 没核；转根环线（抛弃根与隔离归它），不进本轮。

## 三、采纳的改法（第 53 批，与第 52 批补丁一起合）

| 编号 | 改法 | 验收 | 来源 |
|---|---|---|---|
| F1 | M3 的独立参照：harness 档里按 D8 已定项 6 与 E176 M3 臂另写一份叶形状参照（不调 `inode_tree.rs`），在随机历史上与发布路径逐次比每片的记录数、身份与退役集合；core 单测加 [117, 117, 117] 删中间一条那一格，钉「先右」退役的是哪一片 | M4（先左）在新用例上红；改回不红 | 正推 K4.4、K7.2，攻方 s1 F1、s2 K4-4 |
| F2 | 退役树与谱系树补 I-9.21 / I-9.22 同形的判法：产品镜像上非最右容器装满（退役 959、谱系 1359），测试镜像上不超旋钮值，容器号沿叶序严格递增；CPU 与 GPU 同判（用户 2026-09-29「配了 GPU 就一定要 GPU 判」） | 攻方 `opus_probe_container_closed_early` 那一形红，只红新判法；合法镜像不红 | 攻方 s1 F2 |
| F3 | Opt2 环那一项：构造态单测直接调回退取水位那一段，交一份 cur 内存水位低于环里一条没被抛弃的有效根的输入，断言取环里那条；再交一条被抛弃的根，断言不取 | M5 红 | 攻方 s1 F3 第一种 |
| F4 | 坏镜像：退役容器记录宽 ≠ 34 只红 I-1.7；I-9.32 两条（容器出生代 ≥ 死亡 txg、死亡 txg > 这一版）各只红 I-9.32 | 正推 C-retwidth、s2 两条 I-9.32 变异红 | 正推 K6.3、K7.2，s2 K7 |
| F5 | 措辞：`transaction.rs` 第 3172、8639 行与 `recovery.rs` 第 2564 行注释的「十一条」；`crates/mutations.tsv`「实审 A3d：重建读记账树不判节点头…」那一行名字（第 52 批之后红在条目宽）；`crates/singlefs-checker-tier/src/bad_disk_input.rs` 第 181 行文档注释后半句 | 门禁 mutation-tables 格绿 | 正推 K3.1、第 52 批实现员报告 |

kb 写回（随第 53 批一份规格）：`.claude/kb/invariants.md` 第 32 行 I-1.7 改成谱系 24、加退役 34；F2 的新不变量两行与条数；第四节的欠账。`invariants.md` 与 `checks-owed.md` 在根环线 m3-batch10z-xy-code-r1 的快照里，写前把原串新串发它。

## 四、零轮与欠账

- F1–F4 的改法都是腿提的或主 agent 推的，被攻过零轮；进第 53 批的代码轮。
- 记欠账（取号现查最大号）：
  - 「删除类发布之后的可用怎么读」条款没写：贴边 unlink 不超卖的判据定不下，两次攻方抽样用两种读法都没造出超卖、去掉退役项都不红；
  - W 的「删除类发布至多一片叶身份消失」压在「相邻两片之和 ≥ 234」这个 M3 前提上，没有不变量判它（正推 K3.2 手造的树一次可退 116 条，那棵树在 M3 下走不到）；
  - E176 对拍装置 `e176_crates_oracle` 编不过（读已删的 `containers_dropped`），不在复跑登记里，修它归实验执行员。
- C769（③ 同 key 恒成立）：s2 给了一张推的坏镜像（台阶代段写晚一格加同格重发，② 漏报），并进 C769 正文。

## 五、交用户表

本轮没有要用户定的岔路。K5-2 取候选丙（不做那条用例）是主 agent 按两次攻方抽样定的，汇报时写明。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D3（空间分配） 已定项 2 | 不影响 | 2026-10-09 不受影响：K5-3 量到的常驻双扣与现行式子的写法一致，式子不改 |
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-10-09 不受影响：W 加 6 + H_退役 的上界两次抽样没打穿；前提没有判据记欠账，不改正文 |
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-09 不受影响：M3 判定照实现；缺的是对拍参照（F1），不改条款 |
| D23（journal的角色与格式） 已定项 14 | 不影响 | 2026-10-09 不受影响：Opt2 滤法三份样本都没打中，只补判别力（F3） |

## 六、复跑核对（第二次抽样模型）

拷到 `~/.local/share/singlefs/work/m3-batch50-code-r1-verify-s2/`，照报告第 7–16 行逐阶段从头跑（prepare 到 crash，再 compare），全部阶段退 0；compare 22 份全「同」（abandoned-no-delete-control、abandoned-suffix-scan、crash-plans-over-budget、crash-rollback-after-absorb、first-file-watermark、k5-edge 四份、k7-direction 两份、k7-mutations、main-badimages、random-leaves 九份）。第二节里引的 s2 的数都出自这几份。

## 七、用户定案（2026-10-09）

| 格 | 定案 |
|---|---|
| K5-2 | 「【贴边建满后 unlink 不超卖】 这个用例要做，做一个warning。内容是没测出来。」：推翻第二节取的候选丙。用例照两次攻方抽样的读法做（每次删除类发布都发成、删完稳态可用 ≥ 0），实质断言照判；它对退役预付项没有判别力，跑完打一行 warning 写明没测出来，不让用例红。归第 55 批 |
