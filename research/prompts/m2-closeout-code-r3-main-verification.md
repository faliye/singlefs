# 里程碑二收尾：代码轮第三轮判决（2026-09-28）

<!-- doc-lint:not-numbers X1 X2 X3 X4 X5 X6 Y1 Y2 Y4 Y5 Z1Q-A Z1Q-B Z3-A P1 Q4 Q7 -->

正文 `research/prompts/_m2-closeout-code-r3-body.md`，材料 `_m2-closeout-code-r3-{checklist,appendix,diff,background}.md`。快照 `refs/sop/m2-closeout-code-r3-snapshot`（提交 `0cc2a238a57d297f511f9e08cedbbc36c6d56d67`），基准 `refs/sop/m2-closeout-code-r3-base`（`9a9cfb6c`），清单在 `research/prompts/m2-closeout-code-r3-snapshot/`。这份判决由主 agent（会话 singlefs-1e，接手 singlefs-99）写：原定的核查员被停（调度记录 `records/2026-09-24-里程碑二收尾调度.md` 第 320 行，用户「判决和核查是主agent 下的」），核查照 `.claude/rules/three-way-inference.md`「核查与判决由主 agent 做」五步由主 agent 自己做。

**三条腿与每一份样本**：
- 云端攻方（Opus，X1、X5 ①②③、X6）：`research/prompts/m2-closeout-code-r3-opus-output.md`，模型目录 `research/prompts/m2-closeout-code-r3-opus-model/`。第 3 批按用户要求停下（报告第 80–81 行），停下时那一条流没跑完、不计数。
- 云端辩方（Sonnet，X4）：`research/prompts/m2-closeout-code-r3-sonnet-output.md`。
- 本地攻方（X3 与 X2 的算术）：提示 `research/prompts/m2-closeout-code-r3-local-attack.md`，译文核对表 `research/prompts/m2-closeout-code-r3-local-attack-translation-audit.md`。三份样本主 agent 都读了：
  - `research/prompts/m2-closeout-code-r3-local-attack-output-s1.md`：干净，采。八组算式主 agent 逐个复算，全对。
  - `research/prompts/m2-closeout-code-r3-local-attack-output-s2.md`：干净，采。八组结论与 s1 逐格相同；只有第一组第一段的种类两份写法不同（s1 写 system-configuration，s2 写 acquiring-instance-number），两态、三态项都是 3 与 8，不影响数。
  - `research/prompts/m2-closeout-code-r3-local-attack-output-void1.md`：损坏闸留下的作废副本，参考。八组数与 s1 逐格相同，损坏在第 64 行「Wholewhole-line」，没有打中。

**主 agent 的核查**（五步）：
1. `python3 research/scripts/cite-check.py` 核两条云端腿的报告，`--root` 取快照（`git archive` 解出 `crates/`、`litmus/`，再解 HEAD 的 `.claude/kb/`），`--background` 取背景材料：「核了 16 处引文，对上 16 处，没判 0 处」。脚本认不出的写法（「快照 `文件` 第 N 行」）没有逐处人工核；判决引到的那几处按下面第 2 步在产物里核。
2. 产物：攻方报告「复跑命令与文件哈希」表 44 份文件 `sha256sum -c` 全对。判定一览引的几行在日志里逐字找到：`logs/x5-analysis-sweep.txt` 的 `RUNS 17516`、`logs/x5-analysis-enum.txt` 的 `ENUM_STATES_TOTAL 721617`、`logs/x6-snapshot.log` 第 2100 行 `STALE accepted=312 refused=956 flagged=58`、`logs/mutTree.log` 第 40、69 行两份用例 22、23 条全过、`logs/mut736-old-target.log` 第 7 行 `0 passed … 22 filtered out`。
3. 复跑没做：`rerun.sh` 约 75 分钟，要在快照副本上跑崩溃枚举（重型），与本机在跑的提交前全量抢机器。判决只引日志里的原样行，不引复跑数。
4. 本地腿译文核对表 22 条：它点名的快照行 15 处逐处对上（数组、常量、注释原句），引实现员报告的两处（合入后验证一报告第 239–242 行）引文里的省略号只省了行号清单，没丢限定词；表里自己记的两处译文少了限定词（第 2 条「只有」、第 3 条单元写的括注）与一处多出来的说明句（第 5 条），都不影响要算的数。
5. 开工快照：`crates/`、`litmus/` 186 份 `sha256sum -c` 全对；kb 7 份里 6 份今天改过（`decisions/13`、`16`、`22`、`23`、`invariants.md`、`checks-owed.md`），6 份的 HEAD 版本逐字等于开工时的 sha256，腿引的 kb 行对着 HEAD 版本核。`dispatch-time.txt` 被别的会话在清钟点写法时改成只剩日期，原值是一个带秒的时刻（照用户 2026-09-28 定不写时刻，这里不抄）；腿开工时刻不影响任何一格的判定。

## 一、各格判定

| 格 | 判定 | 依据 |
|---|---|---|
| X1-a 写行之后掉电、写行那条根的槽再坏 | **打中，兑现了条款字面而条款的理由罩不住**：每次可写挂载都拒、只读照常，只做过 mkfs 的池同形；与 D16（发布语义） 已定项 7 那一句的理由说反话，条款没给出路。**用户 2026-09-28 定维持今天的判法**（原话「X1-a 选A 不是已经实现了吗」，调度记录第 315 行）：就是 C579（判据在单故障合法状态上也拒可写） 的定案，不改代码，另两个改法不做 | 攻方 X1-a（报告第 200 行起那一节，`logs/x1-readonly.log`）；用户定案 |
| X1-b 已知槽集含挂载后写过的槽 | **兑现了条款** | 攻方 X1-b（第 240 行起） |
| X1-c 「自证不过」那一半 | **替没写的条款做了选择**：四处走到；管理员回退算候选与生效 F 照「根槽这一次读坏」那一行处置，是实现对 D16 已定项 1 字面（只写抬 F 上限）的扩写；另三处读根环不分类，单故障下无害（推的，加上 X5 ① 读故障 0 红）。扩写随 kb 第十批写进 D16 已定项 1，被攻过这一轮；读不出仍当没有根的两处另立 C584（C580 余下的两处读不出仍当没有根），同批写回 | 攻方 X1-c（第 249 行起）；kb 第十批 K2 条目 6、K1 条目 3 |
| X1-d 回退扩写拒错或放错 | **没打中（一次抽样，不支撑结论）** | 攻方 X1-d（第 263 行起） |
| X1-e 两条改形用例、水位与 F_生效 取 max 挪到哪 | **兑现了条款，另有两处没罩住**：两条改形对；inode 号水位的新用例在副本上对变异表那一行判红；树 ID 水位取内存里那一半改形之后没有用例罩着（副本上把 `.fold(current.root.tree_identifier_watermark, u64::max)` 改成 `.fold(0, u64::max)`，两份用例全过）；变异表那一行在快照里点名的是旧测试名，今天的树已换靶。**用户 2026-09-28 定「不做这两个 交给 里程碑3 去做」**（调度记录第 319 行），立 C586（回退取树 ID 水位的那一半没有用例），已写进欠账表 | 攻方 X1-e（第 270–292 行，`logs/mutTree.log`、`logs/mut736-*.log`）；用户定案 |
| X2、X3 钉值与层 0 文件静态改的算术（本地攻方） | **两份干净样本八组全部对上**：第一条流两态 16,777,240、三态 16,777,260（s1 第 1 组）；第二条流 1,358,954,604 与 1,358,954,634、差 30（第 2 组）；C577 之后第二条流减 151,388,145 得 50,724,921（第 3 组）；取号屏障那一段 262,147 与 262,152（第 4 组）；σ 2^16 = 65,536（第 5 组）；分片两趟打印 7 + 51 = 58 行、进度文件 47 行（第 6 组）；差 1 的两对 12 + 1、16 + 1（第 7 组）；变异 7 + 9 = 16 行、证红 3（第 8 组）。s2 同样八组逐格相同 | `research/prompts/m2-closeout-code-r3-local-attack-output-s1.md`、`research/prompts/m2-closeout-code-r3-local-attack-output-s2.md`；作废副本 `research/prompts/m2-closeout-code-r3-local-attack-output-void1.md` 只参考 |
| X4 前两轮判决的复核（辩方） | **前两轮九条判决都站得住**：第一轮 Z1Q-A、Z1Q-B、Z3-A，第二轮 Y5-3a、Y5-3b，两轮「已知」段没有挡住真打中；C577 加屏障之后 Z3-A 乙三形没变（静态调用图，没重跑）；第二轮 Y1「没打中」不是只读了代码（原样日志在） | 辩方「八、汇总表」（第 172–183 行） |
| X5 ① 第二轮 Y1 换历史再抽一次 | **没打中（第二次抽样）**：种子 1–10、五个入口，单故障扫 17516 段 0 段红（30 段是已知的取号回卷写）；入口返回即掉电与注入之后掉电，111 条流 721617 个崩溃状态 0 红。与第二轮 Y1 合起来，两次抽样都没打中，记「没打中」 | 攻方 X5 ①（第 296 行起，`logs/x5-analysis-sweep.txt`、`logs/x5-analysis-enum.txt`） |
| X5 ② 轮换之后屏障 litmus | **兑现了条款**（D16 已定项 7 最后三步与「屏障完成才返回」）；litmus 注释里块层那一半的用例路径过期，并进 C586 | 攻方 X5 ②（第 412–427 行） |
| X5 ③ 零单元发布屏障报错 | **兑现了条款**（D23（journal 的角色与格式） 已定项 14 替没写的条款做的选择，今天由用例钉住）；之后那一步接 X1-a | 攻方 X5 ③（第 428 行起，`logs/repo-x5-3.log`） |
| X6-a 落后支放进发布途中取下的旧快照 | **打中，和条款说反话**：发布 P 途中（单元已写、根槽没写）取下的盘换上来，会话覆盖写或建 inode 照常做成，池级 checker 红 1 条 I-3.1（已分配统计对得上）；1268 格里 58 格；D18（块里携带什么信息） 已定项 11 字面「不落后」，实现判的是「落后且缺现行那一版的单元」。**用户 2026-09-28 选改法 C、放到里程碑三**（原话「我选C 但是放到里程碑三去」，调度记录第 316 行）：落后且（缺现行那一版的单元，或现行那一版的根写在这块盘上而那一槽里不是它）才拒，四个入口同一判；立 C585（会话发布放进发布途中取下的旧快照），已写进欠账表；改法 C 被攻过零轮 | 攻方 X6-a（第 85 行起，`logs/x6-snapshot.log` 第 2100 行）；用户定案 |
| X6-b 只剩一槽自证、回退之后建对象写几次 | **没打中（一次抽样）**：1728 串单故障历史不误拒 | 攻方 X6-b（第 143 行起） |
| X6-c 四入口核两样、拒在任何写之前 | **兑现了条款**：956 格拒、0 格拒前写 | 攻方 X6-c（第 157 行起） |
| X6-d core 与 checker 收同一张表 | **兑现了条款**：21 格两边收拒一致；371 = 0 那一格两边报的错误成员不同，结局相同 | 攻方 X6-d（第 165 行起） |
| X6-e I-7.14 | **兑现了条款**：字段互换只红 I-7.14；盘体对调红 I-7.14 与 I-7.1 | 攻方 X6-e（第 186 行起） |

第三轮之后停，不开第四轮。

## 二、交用户的（零轮与定案）

| 项 | 状态 |
|---|---|
| 改法 C（C585） | 用户已定、放里程碑三；被攻过零轮，实现时走代码三方 |
| D16 已定项 1「根槽这一次读坏」扩到挂着时的管理员回退 | 被攻过这一轮（X1-c、X1-d），kb 第十批写回 |
| 回退取槽的修补：`crates/singlefs-harness/src/history.rs` 回退的 `Err` 分支改调 `root_ring_slot_still_bad_after_one_reread_of_rollback_error`，`crates/singlefs-harness/src/model_comparison.rs` 加这个函数与它的单测 | **零轮**：第五步 harness 故障注入大档撞出，改在第三轮快照之后，没被攻过（调度记录第 322 行）；它是 harness 的判定辅助，不改 core |
| 快照之后别的 src 改动 | `mount.rs`、`recovery.rs`、`transaction.rs`、`checker/src/image.rs`、`e158_root_choice_repair.rs`、harness 的 `model.rs`、`fault_injection.rs` 相对快照只改了注释（删钟点写法、I-7.14 立号那一句），`git diff --cached refs/sop/m2-closeout-code-r3-snapshot` 现查，不含代码行 |

## 三、按路径点名被判的文件（门禁 56 号）

这次提交暂存区里改过的 src 15 份（`git diff --cached --name-only` 现查）：`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker/src/lib.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`、`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs`、`crates/singlefs-core/src/make_filesystem.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/system_configuration.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-harness/src/fault_injection.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/src/model.rs`。前 13 份（除 `fault_injection.rs`、`model.rs`）的代码改动在第三轮快照里、被这一轮判过；`history.rs`、`model_comparison.rs` 快照之后那一处见第二节「零轮」；`fault_injection.rs`、`model.rs` 快照之后只改注释。`e158_root_choice_repair.rs`、`new_pool_file_creation_on_device.rs` 是实验与真设备装置，对不对由各自的变异表与单测判。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D18（块里携带什么信息） 已定项 11 | 改了 | 2026-09-28 改了：四个入口核两样与落后支的判法照实现写，射程补 X6-a 那一形与用户定的改法 C（kb 第十批 K2 条目 1、2） |
| D16（发布语义） 已定项 1 | 改了 | 2026-09-28 改了：「根槽这一次读坏」那一行扩到挂着时的管理员回退（X1-c，kb 第十批 K2 条目 6） |
| D23（journal 的角色与格式） 已定项 14 | 改了 | 2026-09-28 改了：零单元发布屏障报错那一支有用例钉住（X5 ③）、C579 用户定案写进射程（kb 第十批 K2 条目 4、5） |
| D22（单元原子性怎么合成） 已定项 16 | 改了 | 2026-09-28 改了：core 读 325 / 371 已实现、与 checker 收同一张表（X6-d，kb 第十批 K2 条目 3） |
| D16（发布语义） 已定项 7 | 不受影响 | 2026-09-28 不受影响：X5 ① 两次抽样都没打中、X5 ② 兑现；X1-a 与它的理由说反话那一格用户定维持今天的判法 |
| D13（验证路线） 已定项 4 | 不受影响 | 2026-09-28 不受影响：X2、X3 两份干净样本的闭式都等于用例钉的数 |
