# 形式化验证第 12、13 批合入的代码：代码轮第三轮判决（2026-10-03）

<!-- doc-lint:not-numbers K1 K2 K3 K4 L1 L2 L3 L4 L5 L6 H1 U1 U2 U5 Z1 Z2 Z3 Z4 Z5 T1 TP SN0 SN1 SB RN RNSB KF KG KJ1 KJ1T TV4 S5 Q1 Q3 O1 O2 O3 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 / 改动全文 | `research/prompts/_m3-formal-code-r3-body.md`、`_m3-formal-code-r3-background.md`、`_m3-formal-code-r3-diff.md`（14 448 行） | — |
| 开工快照 | `research/prompts/m3-formal-code-r3-snapshot/crates-sha256.txt`（冻结副本 `/tmp/claude-1000/m3-formal-code-r3/tree/`，244 份）、`kb-sha256.txt` | 两腿开工与收工各 `sha256sum -c --quiet` 无输出 |
| 云端辩方（Sonnet） | `research/prompts/m3-formal-code-r3-sonnet-output.md`（169 行，sha256 86cf2c10…） | 只读码，没编译 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-r3-opus-output.md`（253 行，sha256 7ca5496e…），模型 `…-opus-model/`（原型 r3probe、18 份输出） | 主 agent 照报告「复跑」一节在会话草稿 `code-r3-rerun/` 从头跑 `tools/rerun.sh`，结果见第四节 |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root <冻结副本> --root . --background <背景材料>`：「✓ 核了 148 处引文，对上 148 处，没判 0 处」 |

## 二、逐格判决

| 格 | 辩方 | 攻方 | 判决 |
|---|---|---|---|
| K1 RefFS 接调用点 | 站得住：H1 a–f 与 U2 各有落点与变异行；归属每种步只有一种读法 | 没打中：五个取样点 8 113 个候选段 0 段跨步；调用点原样跑 534 态与 1 602 个二次崩溃态全「两者都绿」；弃权 0 | **站住**（一次抽样，跑前判据）。层 0 枚举域上一段带内容发布就是 2^24 态，攻方没跑，不当覆盖 |
| K1 附：辩方 Z3 翻译层把没返回的步当成功返回 | 零轮：`reference_filesystem_translation.rs:293–296` `NeverReturned` 与 `ReturnedOk` 同映 `Succeeded`，第 138 行注释写「按在途算」；今天被一律弃权遮住 | — | **坐实（主 agent 现查冻结副本那几行），零轮**：第 14 批把「注入点没撞上的历史照判」落地时一并改成在途结局，不然那一改就把它露出来 |
| K2 二次崩溃 F 的界 | 站得住，错只往漏判偏 | 没攻到底；读码推按内容认界有两种形状：盘上已生效 F 高于这一串抬到的 F 时界找不到、`mount_transparency.rs:298` panic；更早的系统配置写恰带同一个 F 时界认早（漏判） | **推的，零轮**：没造出来；改法是界按挂载交回的抬 F 那几串发布在写流里的位置取（第二轮攻方原提的取法），找不到时不 panic、整段记不判并计数。进第 14 批 |
| K3 checker 逐档与择根 | 站得住：与实现同序同判；条款 D22 已定项 16 那一行先写核偏移、后写越界整池拒，只在 incompat 上写了不核偏移，「先判值」的字面要补（Z5） | 没打中：`walk.rs` 新 fold 与 `choose_root` 同序同择法 | **站住**；Z5 写回补一句 |
| K3 附：辩方 Z1 GPU 判器择根平局 | 零轮：`crash_judge_checker.wgsl:2869` 注释「相等取后面的」、`:2874` `!root_key_less` 相等也换 | — | **坐实（主 agent 现查冻结副本那两行），零轮**：13-3 只改了 CPU 一臂，GPU 一臂照旧取后一个；改成严格更大才换，加一条 GPU 与 CPU 同判的用例（GPU 用例是 checker 档）。进第 14 批 |
| K4-a E165 r4 生成器的 T1 时机 | 站得住（读码：闸、轮换、取号、写行 txg 对上） | **打中（实现一侧量过）**：登记 T1 两句不等价——「这次挂载里第一次读」与括号「读阶段第一遍读不出」；实现第一次读系统配置槽在择系统配置那一步（第 3、4 次读），见证读在后；照前一句，SN0、SN1、SB、RNSB 都不走重读，生成器却记走第二遍 28 466 | **坐实（主 agent 复跑 `k4t1.log` 逐字同，第四节），不翻 Q1**：生成器写的是「坏到重读之前」那一种时机，实现在这种时机下确实走重读（`k4t1.log` 第 3、5、7、11 行 `OnTheOneReread`），历史是真的；错在登记与实验页把它说成「第一次读」。改法乙：登记补修订、实验页那一句改成「坏到重读之前」并注明「只坏第一次读」时系统配置槽那一读被择系统配置吃掉、见证不判真。「只坏第一次读」时择系统配置可能择到另一槽、之后的挂载走另一条路，这一种生成器没枚举（攻方改法丙），零轮，记新欠账、不开轮 |
| K4-b KG 两道保护、取号、轮换、写行 txg | 站得住 | 没打中（读码，取号与逐盘核两条是推的） | **站住** |
| K4-c E171 取号、暖机出错时 O1 一列静默不判 | 站得住（读码：关一条记 `off`、不适用记 `na`） | **打中（读码）**：`step_acquire_and_warm_up` 在取号或暖机交回 `Err` 时直接 false、不调 `judge`，O1-对拍留 `not_modelled`；之后的态 O1-崩溃判官记不适用；S5 只在基线判（`e171_three_oracle_collusion.rs:5049`）；一行这样的坏实现会进 Q1 分母、三列都不红 | **坐实（主 agent 现查冻结副本第 617–645 行与第 5049 行；登记第 41 行读法 5 末句「一次调用返回错误而模型说该成功……是 O1-对拍的红」、第 42 行读法 6「除这两种来源之外出现不适用，走停机 S5」）**：装置没照登记落地这两句，是装置缺陷，不改登记。改法甲（取号、暖机在不带故障的流上交回 `Err` 记 O1-对拍对不上）与丙（S5 在逐行那一趟同样判）都做；E171 整表在改完之前不跑 |
| 辩方 Z2 测试支撑 `history.rs:2628` 等同形 `max_by_key` | 零轮，低 | — | 记欠账，不进这一批 |
| 辩方 Z4 变异表缺 fsync 一臂 | 零轮，低 | — | 第 14 批顺手补一行 |

被点名的 `crates/*/src/*.rs`：`crates/singlefs-reference-filesystem/src/lib.rs`、`crates/singlefs-harness/src/reference_filesystem_translation.rs`、`crates/singlefs-harness/src/fault_injection.rs`、`crates/singlefs-harness/src/mount_transparency.rs`、`crates/singlefs-harness/src/lib.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-replay-prefix/src/lib.rs`、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`、`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs`、`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`。

## 三、改法（第三轮是最终轮，不开第四轮；这一轮的改法都是零轮，进第 14 批实现，交用户表见第五节）

1. 翻译层 Z3 与「注入点没撞上的历史 RefFS 照判」一起改（`reference_filesystem_translation.rs`、`fault_injection.rs`），用例与变异各一。
2. GPU 判器择根平局 Z1（`crash_judge_checker.wgsl`），GPU 与 CPU 同判用例一条（checker 档）、变异一行。
3. K2 的界按写流位置取，找不到不 panic（`mount_transparency.rs`、`crash_injection.rs`），用例与变异各一。
4. E171 装置 K4-c 甲与丙（`e171_three_oracle_collusion.rs`，归执行员）；整表在这之后跑。
5. E165 r4 K4-a：登记修订、实验页改句（归执行员）；新欠账「只坏第一次读时择系统配置择到另一槽」一族。
6. Z4 变异表 fsync 一臂；`crates/mutations.tsv` 第 1898、1903 行锚点（E165 r3 第二段）。
7. 写回：Z5（D22 已定项 16 补「逐档读到可用槽先判值、再核偏移」）；Z2 新欠账。

## 四、复跑核对

主 agent 在会话草稿 `code-r3-rerun/` 照攻方报告「复跑」一节从头跑 `tools/rerun.sh <冻结副本> <草稿>`（五件全跑）：18 份输出去掉包装行与耗时行后与模型目录 `outputs/` 同名文件逐字节相同（脚本打「same」18 行、没有 DIFF），含 `k4t1.log`（K4-a 实现一侧的读序）与 `inject-6.log`（攻击面之外那两条新发现）。K4-c 只读码坐实，E171 装置是 checker 档，攻方要的真装置变异由 E171 第三段执行员写进内联单测、提交时跑。

## 五、交用户表（零轮，不再开轮）

| 形态 | 来源 | 处置 |
|---|---|---|
| Z1 GPU 择根平局取前 | 辩方 | 第 14 批实现，改法与 CPU 一臂同形 |
| Z3 翻译层没返回的步按在途 | 辩方 | 第 14 批实现 |
| K2 界按写流位置取、找不到不 panic | 攻方推的 | 第 14 批实现 |
| K4-c E171 取号暖机出错记 O1 红、逐行趟判 S5 | 攻方 | 第 14 批执行员，照登记读法 5、6 落地 |
| K4-a 「只坏第一次读」时机一族 | 攻方改法丙 | 记欠账，不建 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 3 | 不影响 | 2026-10-03 不受影响：RefFS 调用点站住，Z3 是实现缺陷 |
| D13（验证路线） 已定项 6 | 不影响 | 2026-10-03 不受影响：E171 装置缺陷照登记修，结论还没出 |
| D22（单元原子性怎么合成） 已定项 16 | 不影响 | 2026-10-03 不受影响：先判值与实现一致，字面补一句（Z5） |
| D22（单元原子性怎么合成） 已定项 24 | 不影响 | 2026-10-03 不受影响：GPU 一臂照同一读法改 |
