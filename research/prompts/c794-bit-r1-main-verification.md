# c794-bit-r1 判决：C794 盘上隔离位没定的六处条款，第一轮（2026-10-15）

被判的对象：实现员副本补丁（`research/prompts/_c794-bit-r1-diff.md`，不合入）碰到的六处条款 Q1–Q5、Q8；涉及 `crates/singlefs-core/src/allocator.rs`、`recovery.rs`、`mount.rs`（回退那一段）、`crates/singlefs-checker/src/walk.rs`、GPU 判器三份 `.wgsl` 与 `crates/singlefs-checker-tier/src/crash_judge_gpu.rs`、`crates/singlefs-harness/src/model.rs`。这一轮不改主工作区的 `crates/`。

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 -->

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 开工快照 | 冻结副本里 `sha256sum -c research/prompts/c794-bit-r1-snapshot/crates-sha256.txt`；主工作区 `kb-sha256.txt` | 都对上 |
| 攻方 `research/prompts/c794-bit-r1-opus-output.md`（sha256 7416ab1f…）、正推 `research/prompts/c794-bit-r1-sonnet-output.md`（sha256 be78807a…，235 行） | `cite-check.py <两份> --root <仓根> --background research/prompts/_c794-bit-r1-background.md` | 「✓ 核了 60 处引文、7 处裸行号，对上 60 处，没判 7 处」 |
| 两腿模型 | 拷进草稿目录，各经 `run-with-memory-cap.sh 24G` 跑 `tools/rerun.sh` | 正推「name=rerun_compare compared=16 differing=0」；攻方「name=rerun_done compared=31 differing=0」（日志在会话草稿 `c794-bit-r1-rerun-logs/`） |
| 正文一处指路 | 攻方报：Q1 格写「D22 的失败处置」，D22 文件里 grep「失败处置」0 次 | 主 agent 正文写错了出处；对应条款在 `.claude/kb/decisions/23-journal的角色与格式.md` 第 406 行。两腿都按条款内容判，不影响判定 |

## 二、逐格判

| 格 | 正推 | 攻方 | 判 |
|---|---|---|---|
| Q1 两位同置 | 取「三处都拒 / 都判红」：池级 checker 当已释放、不判红那一半站不住（同一份镜像 core 拒、checker 全绿，量的）；崩溃状态 3011 个里 0 次走到这一组合 | 打中（量过）：写者缺陷写出两位同置后池级 checker 全绿、冷启动走读拒成 I-1.1；I-1.1 条文（`crates/singlefs-checker/invariants/01-块自描述.md` 第 6 行）里没有标志位组合这一样，拒因归错了判据 | 一致：三处读者对这一组合要同判。定：core 读者与 GPU 恢复内核照拒，池级 checker 判红；不变量清单给这一组合单写一句（分配记录那一类），core 的拒因与 checker 的违例都点名它，不挂在 I-1.1 下 |
| Q2 回退清不清位 | 取清：不清时「曾隔离、被回退重新引用、之后写者漏释放」checker 全绿（4 格矩阵量的）；清位在逐盘验之后、随同一次发布落盘 | 实现员读法没打中；不清那一读法打中（量过，同形）；入库用例已钉住清位 | 一致，照实现员的读法定：清 |
| Q3 checker 判「隔离位在有根引用的记录上」 | 两个都站得住；若加只能取「最新根走读」口径（「任一走过的版本」口径在合法刚隔离状态上红 939/949） | 「任一走过的版本」口径打中（误判合法状态）；「最新根引用」口径是攻方自提收严（零轮）：合法镜像都不红、当场抓得到回退不清位 | 「任一走过的版本」口径出局。不加与「最新根口径加」都站得住，按跑前条款第三行写岔路单交用户（`research/prompts/c794-bit-r1-forks.md`） |
| Q4 回退判隔离单元看不看位 | 两个都站得住、写者正确时逐例同判；写者漏释放或位丢了时看位放行一个有一份坏拷贝的回退，推断读法拒（推的偏向不改）；C572 判决 A2 的变异行在看位读法下变成等价变异 | 改成看位打中（量过）：不带位的漏释放单元有一份坏，回退不验就做成，之后 I-2.1、I-4.8、I-7.4 红；推断读法在任何写之前拒 | 一致：看位读法出局（两腿都量到它放过坏拷贝），照实现员的读法定：按推断，不改 |
| Q5 按记录还是按单元 | 取按记录：对称镜像与崩溃状态上两读法同判（0 个差异），写者造出的不对称镜像上按单元全绿而 core 拒；I-3.1 / I-3.11「按单元」那一整句要换 | 按单元打中（量过）：写者只在一块盘上置位时按单元豁免全绿，按记录判 I-3.11 红 | 一致，照实现员的读法定：按记录；写回时换掉 I-3.1、I-3.11 里「按单元」那一句（主 agent 2026-09-25 定的读法随用户这次定案一起撤） |
| Q8 理想模型观测隔离位 | 两个都站得住：加 5 行能判「活单元带位」，随机历史与故障注入今天撞不上，检出 0 次 | 量到盲点：变异「新分配的记录带隔离位」两档随机历史全绿，随机历史里隔离 0 次；加观测能抓「新记录带位」是推的 | 加与不加都站得住，写岔路单交用户；两腿都量到的事实「随机历史与故障注入里一次隔离都没有」单列：今天没有一种随机测试走得到隔离，这本身是测试面的缺口，不论 Q8 怎么选都记一笔欠账 |

单次抽样：两腿各抽一次；「没打中」那几格不单独当「没打中」用，Q2、Q4、Q5 的定案靠两腿各自量到对面读法判错。崩溃状态（回退那次发布 1048588 态、B 那次发布与回退后覆盖写各 16777228 态）两腿都没枚举，Q2、Q3、Q5 在崩溃状态上的几句是推的；四个读法在回退那次发布上的全量枚举（攻方报告「要主 agent 跑的命令」，合计 4194352 态）是 checker 档重活，提交之外跑要先问用户。

## 三、处置

- 定：Q1（三处同判、单写一句不变量）、Q2（清）、Q4（推断）、Q5（按记录）；主 agent 定的 Q6（删 GPU 内核里没人读的表与阶段）、Q7（坏盘输入最大跨度改 0x3FFF）。
- 交用户：Q3、Q8（岔路单 `research/prompts/c794-bit-r1-forks.md`）。
- 实现跟着改：`crates/singlefs-core/src/recovery.rs` 冻结副本第 4747 行对称拒文案补上隔离位（正推报）；Q1 那一句不变量；`crates/mutations.tsv` 照实现员报告失效的 8 行与新加的 11 行。
- 新欠账：随机历史与故障注入里一次隔离都没有（两腿量的），隔离那几条路只有手写用例罩着。

## 四、补：回退那次发布的崩溃状态全量枚举（2026-10-15）

用户弹窗定「现在就跑」之后，主 agent 照攻方报告「要主 agent 跑的命令」跑（驱动 `~/.local/share/singlefs/work/c794-bit-r1-enum/run.sh`，经 `detached-run.sh`，24G、31 线程，每个读法一个新判定库；攻方原型自带的「不超过 10^6 个状态」上限在拷贝里放到 2_000_000，否则 1048588 态拒跑）。判器走 CPU（`CRASH_POINT_REPLAY_JUDGE route=cpu why_not_the_gpu=gpu_not_enabled_in_the_configuration`：拷贝里没有多机配置）。四行原样：

```
ENUM flow=rollback-publish label=baseline states=1048588 oracle_violations=0 failed_states=0 file_read=1048588 checker_violated={}
ENUM flow=rollback-publish label=per-unit states=1048588 oracle_violations=0 failed_states=0 file_read=1048588 checker_violated={}
ENUM flow=rollback-publish label=q3-newest states=1048588 oracle_violations=0 failed_states=0 file_read=1048588 checker_violated={}
ENUM flow=rollback-publish label=keeps-bit-q3-newest states=1048588 oracle_violations=0 failed_states=0 file_read=1048588 checker_violated={"I-3.11": 9}
```

保位那一读法的两条发现都在回退那次发布的根已落之后（`segment=3 publish=after_the_last_root first_state=1048579`、`segment=all_persisted publish=every_write_persisted first_state=1048587`，`red_states=9`）。与攻方跑前推的预期逐条相同：清位读法（Q2 定案）、按单元豁免、Q3 按最新根口径在回退那次发布的全部崩溃状态上都不红；回退不清位时 Q3 那一口径在回退落盘之后当场抓到。第二节「Q2、Q3、Q5 在崩溃状态上的几句是推的」那一句，在回退那次发布上由这一趟坐实；B 那次发布（16777228 态）仍没跑。四份输出在会话草稿 `c794-bit-r1-enum-results/`。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D3（空间分配） 已定项 7 | 备料 | 2026-10-15 不受影响：这一轮不改决策文件；隔离位借跨度段次高位那一句随用户定案与这一轮判下的 Q1、Q2、Q4、Q5 一起写回 |
| D19（块指针的结构与宽度预算） 已定项 5 | 备料 | 2026-10-15 不受影响：同上，硬规则 1「记录留在已分配」的盘上表示改成带隔离位，随这一批写回 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-10-15 不受影响：回退判隔离单元照旧按推断（Q4），清位随回退那次发布落盘（Q2），「在任何写之前拒」那一格不变 |
