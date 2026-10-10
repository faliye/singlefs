# 第 35 批代码轮第一轮判决（2026-10-06）

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 P7 Q1 Q3 Q4 FL B4 ALT1 R1 R2 R3 -->

被判的文件（第 35 批三份补丁，逐份见改动全文 `research/prompts/_m3-formal-code-b35-r1-diff.md`）：
- 35-1：`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`；
- 35-2：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl`、`crates/singlefs-checker-tier/tests/gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs`；
- 35-3：`crates/singlefs-harness/src/model.rs`、`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs`、`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs`；
- 三份的追加与替换：`crates/mutations.tsv`。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b35-r1-body.md`、`_m3-formal-code-b35-r1-diff.md`、`_m3-formal-code-b35-r1-background.md`（第一版 1.45 MB 挪到会话草稿 `b35-materials-v1/`，裁成 81 KB） | — |
| 冻结副本与快照 | `/tmp/claude-1000/m3-formal-code-b35-r1/tree/`（第 35 批合入后），清单同目录 316 行；kb 快照 `research/prompts/m3-formal-code-b35-r1-snapshot/kb-sha256.txt` | 腿交回后核：冻结副本 316 行全 OK；kb 快照里 `.claude/kb/checks-owed.md` 对不上（别的会话之后改过；两腿引的 C735、C738 两行原文仍在，cite-check 在今天的文件上全对上），其余全 OK |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b35-r1-sonnet-output.md`（sha256 a3a2ed92…），模型 `…-sonnet-model/` | 主 agent 照 `tools/rerun.sh` 复跑（第四节） |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b35-r1-opus-output.md`（sha256 3cc26999…），模型 `…-opus-model/` | 主 agent 照 `tools/rerun.sh` 复跑（第四节） |
| 引文 | 两份腿报告 | 主 agent 合跑 cite-check（`--root` 冻结副本与仓）：71 / 71 |

腿跑着的时候删除线第 28 批 K 合进主工作区，动了 `walk.rs`、`crash_judge_checker.wgsl`、对拍用例、坏镜像用例、`mutations.tsv`（合前原样 `/tmp/claude-1000/m3-batch28/before/`）；两腿读的是冻结副本，判决按冻结副本核。

## 二、逐格判决

### P1　FL

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 取的新根段与 core 施加的同一份 | 一致：core 施加 B 的 27 个子集上两边逐子集同向；FL 关时 54 个子集分开 | 没打中：随机历史 26 269 张、层 0 53 352 态，字节不同 0 处、core 施加而 checker 不认 0 处 | **站住**（两腿立场不同、各抽一次，不记成「没打中」的结论） |
| 「末条没落就不认」改了哪些状态 | 条款没写 / 冲突：I-3.10、I-5.4 两行没写「带末条标志」；core 不施加 B 的 37 个子集里 checker 仍认 21 个 | **打中、不分辨臂**：末条落了而前面某条没落（断号）的发布，core 不施加，新旧 checker 都认（新 1 122 张、旧 2 251 张），样本里没多出红；35-1 说明「末条还没落的那次发布 core 不施加，这里也不认」只对一半成立 | **坐实**：主 agent 现查 `crates/singlefs-checker/src/walk.rs:5569`「末条还没落的那次发布」与同一段 5573 行「断号即止、点名单元验证、发布边界不复刻」并存；core 的施加单位是一次发布、首条起连续走到末条（`crates/singlefs-core/src/recovery.rs:3517`「才整体施加；前缀停在一次发布中间（末条没到、断号」）。两腿同一形。checker 认的是 core 不施加的一版，与 I-5.4 / I-3.10 条文「下一次挂载先施加的那一版」字面不符；改法 R1（取 I-4.1 那份重放 `replay_the_next_mount_would_do`（`walk.rs:8324`）施加的第一次发布的末条）进第 36 批，被攻过零轮 |
| 退路 | 一致（合法）；首条写旧时红 I-3.1、I-3.5，同 C738 | 走得到（6 MiB 与 1 MiB journal 环、根环 S = 8，环转一圈同一段盖首末两条的槽，15 / 64 子集），取的那一份与 core 相同 | 站住 |
| GPU 两处 | 一致（读码，推的） | 没打中（读码，推的） | 站住（推的）；R1 若改 CPU 认法，GPU 扫描内核同改 |
| 越格：I-3.1 误红 | — | journal 环转过由记录施加那一版的记录之后 I-3.1 在合法历史上误红，新旧同红 | 不归这一轮：C697（已分配统计在短环加大文件上误红） 那一形，已有用户定的 RCW |

### P2　Q1 甲

两腿同判站住：由记录施加出来的那一版全零与带自己单元的候选根同红 {I-2.1、I-3.1、I-4.8、I-7.4}；下一次挂载那一版全零只红 I-4.1、core 两种挂载都拒；I-4.1 是唯一射程写「重放之后那一版」的一条（`.claude/kb/invariants.md` I-4.1 那一行）；合法镜像上全零的根与记录 0 条（攻方 79 621 张）。攻方另记一处措辞：红的是「重放施加出来的那一版」，重放两次发布而第一次全零时不红、core 也照常挂上——两边一致，判定不改；35-1 说明照 R3 改措辞，进第 36 批。四处「部分没判 / 射程里判不了」的处置两腿都判自洽；kb 里 I-9.28 两句与 I-5.5 那句的旧说法已在第 34 批判决第三节第 3 条的写回清单里。

### P3　Q3 甲（GPU 走读内核）

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 全零分支红位 | 一致（读码，推的）：六个阶段共用一个调用点 | 没打中（读码，推的） | 站住（推的） |
| 四条新对拍流判不判得出名字说的那一形 | CPU 一侧在 harness 里同搭法重做，红集合与注释、断言对得上 | 没打中：四条流对应的 CPU 变异在流上都有判定不同的状态 | 站住（GPU 推的） |
| 「拿掉根槽写」两条的构造 | — | **打中**：构造出的盘面与合法历史逐字节不同——实例 1 那一行的 W 合法 4、构造 0（录制时挂载读到 B 的根、什么都没施加），根槽没落而系统配置已轮换；测试注释「挂载写出的行与之后的发布不变」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:7754`）不成立；末态判定不受影响 | **坐实**（攻方 crafted 段逐扇区比，复跑见第四节）。改法 R2（合法造法录流：崩在 B 的根槽写之前、从崩溃镜像起新进程录挂载与覆盖写；第四条在挂载之后改记录）进第 36 批，被攻过零轮 |

攻方另报：实例表行的 W 与恢复施加的最大事务号对不对得上，今天没有不变量判。主 agent 现查：池级 checker 只在 I-1.2 的「已发布」谓词里读 W（`crates/singlefs-checker/src/walk.rs` 里 `transaction <= row.applied_transaction_high_water_mark` 那一处），不判它的值。记欠账（第三节第 2 条）。

### P4　Q4 甲与两处说明

两腿同判站住：新理由只在「认出来且一版都没判到」时走，不压过违例、部分没判、成立；坏镜像（extent 树那一条根指针写零）是这一形；合法历史上 0 次（攻方 79 621 张）。I-9.28、I-7.4 两处说明判定没动。攻方读码另记：认出来的那一版的数据单元全落在池里没有的盘上时也走到这一句（说「没有数据单元」，实际有），只有坏镜像造得出，推的——第 36 批把理由措辞收窄到它判的事实。正推附带：extent 树根写零后整套 checker 只有 I-5.5、I-9.10 变、0 红，改动之前就有，不归这一轮。

### P5　35-2 GPU 恢复内核择根

正推站住：160 种单字节翻转上 core 与 WGSL 谓词仿写 160 / 160 相同（136 拒、24 收），四条用例的位置都在 core 拒的 136 里，`parse_root` 两个调用点与 `parse_record` 已各判五段，旧函数名 0 处残留；GPU 一侧是仿写，推的。攻方没攻。

### P6　35-3

正推：B4 第 34 批点名的三个窗口已关，取样点之间的空档窗口（5–28、30–98、100–118、120–133）仍存活——有限取样的固有限度，不再追（`.claude/rules/mutation-sampling.md` 第三类的「补一个敏感的取样点」已对点名的窗口做到）；I-5.4 与 I-3.10 逐状态同集在 60 个状态上 0 分歧（近似枚举，层 0 真数核不动）；829 行注释对。攻方没攻。R1 之后两者仍须经同一个函数认这一版。

### P7　变异行

正推站住：21 行整行与改动全文逐字相同、锚点各命中 1 次；harness 档 13 行全红在断言、点名测试失败、还原核 sha 0 不对；955、965 的 ALT1 判的还是原来那条判；GPU 8 行只做了静态核。攻方没攻。

## 三、去向

1. **第 36 批实现员**（零轮）：
   - R1：`walk.rs` 认「下一次挂载先施加的那一版」改取 `replay_the_next_mount_would_do` 施加的第一次发布（末条那一份新根段），I-3.10 与 I-5.4 仍同一个函数；GPU 扫描内核 `record_index_of_the_version_the_next_mount_applies_first` 同改（GPU 没有那份重放的，读码定怎么对齐，对不齐就停下交主 agent）；断号那一形（攻方 `seed=188022785` 那一行）进坏镜像或判定用例，配变异。
   - R2：对拍用例「拿掉根槽写」的两条流改成合法造法录流，注释照改；编过不跑。
   - R3：35-1 两处说明的措辞（FL 那句补断号那一半；I-4.1 那句写「重放施加出来的那一版」）；Q4 新理由收窄到它判的事实。
2. **欠账**：实例表行的 W 与恢复施加的最大事务号对不对得上没有不变量判，记进 C740（实例表行的 W 与恢复施加的最大事务号没有不变量判）。
3. **写回批**：第 34 批判决第三节第 3 条那几项，加 I-3.10、I-5.4 两行写明认的是「下一次挂载会施加的那一版」（R1 合入后照实现写）。
4. **越格**：I-3.1 那一形归 C697（已分配统计在短环加大文件上误红），已在根环线手上。

## 四、复跑核对

正推：模型 `tools/rerun.sh setup build tests probes fl-off rows b4` 在会话草稿 `b35-sonnet-rerun/` 跑，退 0，编译日志三个被测包各有 `Compiling`。25 份 `t-*.log` 与 `b4-windows2.log` 去掉耗时、线程号、「has been running」行与筛掉的条数、按行排序之后逐行相同；13 行变异的 `exit`、`kind`、`named_test_failed`、`compiled_pkg`、`restore` 与首个 panic 逐行相同（线程号除外）。腿另跑的 `t-base-row0.log` 与 `baseline-rows.out` 不在 `rerun.sh` 里；那 13 条点名测试随坏镜像用例整份二进制（`t-kbi.log`）复跑全过。

攻方：模型 `tools/rerun.sh <草稿目录>` 不给段名全跑，在会话草稿 `b35-opus-rerun/` 九段各 `rc=0`。`B35R`、`B35L0T`、`B35X`、`B35F`、`B35E`、`B35S`、`B35D`、`B35R-FIRST` 行去掉 `seconds=` 之后排序比：复跑 1 024 行、腿留存 1 024 行，差 0 行（含 P1-b 那一行 `seed=188022785 … checker=Some((2, 6)) core=Some(((2, 5), None, None))` 与 P3-c 的 `B35X B-clean …` 三行）。

核不动：GPU 对拍（35-1 四条、35-2 四条）与 GPU 变异行、checker 档那条新池新建文件用例的 I-5.4 计数、层 0 快档，归提交时。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | R1（断号的发布 checker 不再认） | 攻方提，两腿同一形 | 推的，没实现 | 第 36 批 |
| 2 | R2（对拍流合法造法） | 攻方 | 攻方副本量过 CPU 一侧末态判定相同 | 第 36 批 |
| 3 | R3（说明措辞） | 攻方 | 文字 | 第 36 批 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 15 | 备料 | 2026-10-06 不受影响：FL 与 R1 都是让 checker 认 core 按已定项 15 真施加的那一版，条款不变 |
| D19（块指针的结构与宽度预算） 已定项 3 | 不影响 | 2026-10-06 不受影响：35-2 把头部判法铺到 GPU 恢复内核择根，字段表不变 |
