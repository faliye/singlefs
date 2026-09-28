# 里程碑二收尾：代码轮第一轮判决（2026-09-27）

<!-- doc-lint:not-numbers Z1 Z2 Z3 Z4 Z5 Z6 Z7 Z8 Z1Q-A Z1Q-B Z3-A -->

正文 `research/prompts/_m2-closeout-code-r1-body.md`，背景 `_m2-closeout-code-r1-background.md`，被判的 diff `_m2-closeout-code-r1-diff.md`（`346f5e6` → 冻结提交 `67f447de9761f826711565032d8ce16fbf44b902`，ref `refs/sop/m2-closeout-code-r1-snapshot`；src 43 份、+14780/−3047 行，三个实验装置 bin 不进）。三条腿：攻方 `m2-closeout-code-r1-opus-output.md`（Z1、Z3、Z5、Z8，模型目录 `m2-closeout-code-r1-opus-model/`）、正推 `m2-closeout-code-r1-sonnet-output.md`（Z2、Z4、Z6、Z7）、本地攻方 `m2-closeout-code-r1-local-attack-output-s1.md` / `-s2.md`（算术）。核查员 `m2-closeout-code-r1-verifier-output.md`（sha256 71c2f1c7…75e5）：攻方引文 9 处、产物逐字 10 行全 ✓，模型目录 `rerun.sh` 四条命令（三条目标测试、随机走两个宽度）复跑与报告、原始日志逐字段一致；正推引文 19 处 18 ✓，1 ✗ 是它自己披露的行号口径切换（快照 464–465 行对应主工作区 507 行，内容相同）；本地攻方样本 12 项、事实表 7 项、A4b 来源 3 项全 ✓，翻译核对表 16 条 14 ✓、2 ✗（`admission.rs` 行区间该写 476–482；核对表第 2 条「定稿」栏断言的英文措辞没出现在发出去的提示里）。

开工快照 `m2-closeout-code-r1-snapshot/`：正文 `sha256sum -c` OK；kb 10 份里 6 份（invariants、checks-owed、D13、D18、D23、D28）在腿交齐之后被第二批书记员改了，开工原样从 `refs/sop/worktree-backup-2026-09-27-0920` 倒推进 `kb-at-start/`（10 份与 `kb-sha256.txt` 逐个相同，主 agent 现核），核查员照它核。正文第二节列的 41 份 src 之外，材料员按范围现查多出两份新文件（`mounted_session.rs`、`layer0_progress.rs`，第四节点名），腿开跑前已补进正文与背景。

## 一、各格判定

| 格 | 判定 | 依据 |
|---|---|---|
| Z1 Z1Q-B 回退时新根不引用的隔离单元 | **打中，和条款说反话——两条已定条款在这一格打架**：D23（journal 的角色与格式） 已定项 14「释放」一格字面（`kb-at-start/.claude/kb/decisions/23-journal的角色与格式.md:379`「cur 那一版的账里仍分配、而 new 不引用的落点，释放代 = new 的 txg」）要释放它，D19（块指针的结构与宽度预算） 已定项 5 硬规则 1（`19-…:112`）要两份都留在已分配；代码照后者。**用户 2026-09-27 定「照 D19：隔离单元不释放」**：代码不动，D23 那一格补「隔离单元除外」（第三批 kb 规格） | 攻方 Z1Q-B（`rollback_to_a_version_not_referencing_a_quarantined_unit_leaves_it_allocated`，复跑逐字一致）；用户定案 |
| Z1 Z1Q-A 回退目标引用一个隔离单元 | **兑现了条款字面，条款的理由罩不住这一格**：目标引用的隔离单元（盘 1 那一份已知坏）不进复活集、不逐盘验，回退照样做成；条款给的理由「分配器不会发出去」在这一格不成立。**立欠账**（第三批 kb 规格），等 Z3-A 乙与 C554 乙落地一并看 | 攻方 Z1Q-A |
| Z1 随机走 | **没打中**：顺序写多单元文件、建 inode、正常卸载、抬 F、回退交错，4 GiB 与 384 槽两个宽度、24000 步、回退做成 4140 次，checker 红 0、冷恢复读回错 0 | 攻方；核查员复跑逐字段一致 |
| Z2 准入（5 格） | **兑现了条款**：D28 已定项 1 逐设备合取、已定项 3 切换预留、D2 已定项 13 例外 / C565、D16 已定项 1「准入」；已定项 4 ckpt_cost 兑现的是用户定案（kb 原文当时还是「Σ 树高」，第二批 kb 已改）。**这一格的对象是快照里 A4b 的式子**；A4c 之后用户改定 K1、A4d 在改，归第二轮 | 正推 Z2-1…Z2-5 |
| Z3 Z3-A 挂着之后的入口认不出换上去的空盘 | **打中，替没写的条款做了选择**：正常卸载、回退到现行那一版、会话发布在盘 1 换成全零空盘时照样做成，空盘收到 28 次写，之后真盘再挂可写被拒、checker 红 4 条；只有可写挂载那一处有 D18（块里携带什么信息） 已定项 11 的逐盘「不可见」核（快照 `mount.rs:2709` 附近），挂着之后的入口共用的 `caller_inputs_agreeing_with_the_disk`（快照 `mount.rs:2957`）对一份自证槽都没有的盘报不出不一致；kb 里挂着之后的入口没有条款。别的池的盘换进来被择系统配置那一步拒；抬 F 被读根环那一判顺手拒。**用户 2026-09-27 定「乙：每个入口都逐盘核，含会话发布」** | 攻方 Z3-A（4 条用例，复跑逐字一致）；用户定案 |
| Z3 其余 | 没打中（只读了代码）：树表重复、`checked_add`、环装不下 3 条记录、单元区起点；A2a 同一段同一块盘两次系统配置写没专门量 | 攻方 |
| Z4 checker 新判定（5 格） | **兑现了条款**：I-1.11 五步判定、I-8.6 计数器 0 判违例、I-5.1 按版判与共享子树合并、I-7.4 被抛弃根那一半读不出槽时报不适用、I-7.7 两支拆开判 | 正推 Z4-1…Z4-5 |
| Z5 按设备记屏障、原地覆写第三态 | **没打中**：第三态只罩长于 512 字节的写、撕裂点取中点，都是 D13（验证路线） 已定项 4 第 71 行的字面（条款本身被攻过零轮）；799 步多挂载流上 40312 个撕裂镜像逐字节换撕裂点，能自证的 0 个，换撕裂点不改恢复结局；FUA 前同一块盘上还有没冲的普通写 0 次 | 攻方 Z5 |
| Z6 崩溃注入与对拍 | **兑现了条款**：D13 已定项 7 三截（`CrashStateStage`、`WritableMountPhase` 三变体与 kb 逐字对应）、`model.rs` 双向比。第三截前缀交法与第二截没接记录核对器是已知（B3c-2 已改、不在快照里，归第二轮） | 正推 Z6-1、Z6-2 |
| Z7 头宽三方各算 | **兑现了条款**：三份各写各的算式、只共享标量；交叉断言扫 key 宽 0..=255，钉 6 个 `format-const` | 正推 Z7-1；本地攻方 k = 8 → 131、k = 24 → 163 |
| Z8 合并点 | **没打中**（只读了代码）：四条路径里没有「写了再拒」；唯一的次序怪点是 Z3-A 那两批检查没合成一套 | 攻方 |
| 本地攻方算术 | 事实表 7 项与仓里钉的数一致（第一条流 150994980 / 54，第二条流 14960689284 / 390，一条记录点名项 67）；两份样本主 agent 没给答案表，逐格比数没做，两份样本的方向一致性没判 | 核查员第三节 |

## 二、改法（被攻过零轮，第二轮攻）

| 改法 | 做什么 | 修哪一格 |
|---|---|---|
| Z3-A 乙 | `caller_inputs_agreeing_with_the_disk` 里每块交进来的盘要有至少一份本池 fsid 的自证系统配置槽，没有就拒（与可写挂载逐盘核第一支同一判）；会话每次发布之前也读两槽核一遍。代价（推的，没量）：每次发布每块盘多读两槽。实现等 C554 乙交回（同在 `mount.rs`）再派 | Z3-A 三形 |
| D23 已定项 14 补例外 | 「释放」一格补「隔离单元（D19 已定项 5 硬规则 1）除外」；代码不动 | Z1Q-B |
| D18 已定项 11 扩射程 | 逐盘「不可见」核扩到挂着之后每个收盘表的入口（正常卸载、管理员回退、会话发布、抬 F） | Z3-A 的条款空白 |
| Z1Q-A 立欠账 | 回退目标引用的隔离单元不进逐盘验、条款理由罩不住 | Z1Q-A |
| 本地腿流程 | 翻译核对表里行区间与「定稿」栏要照实际发出的提示写（核查员两处 ✗），进第十步素材 | — |

## 三、第二轮

只攻：Z3-A 乙的改后代码；第一轮没罩到的——A2a 同一段同盘两次系统配置写、journal 环转圈后的撕裂、续跑与分片（只读了代码、没造输入）；第一轮之后打进主工作区、不在快照里的那几批（A4c、A4d 的 K1 与 C545 准入先拒、C554 乙、B3c-2、B3a-3c、B3a-3d、55 号那两处步数、lint 清红），以及 A3。第二轮的快照重取。第三轮之后停。

## 四、按路径点名被判的文件（门禁 56 号）

被判的 src 43 份：`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker/src/lib.rs`、`crates/singlefs-checker/src/position_addressed.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-core/src/admission.rs`、`crates/singlefs-core/src/allocation_record_tree.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/code_two_tree.rs`、`crates/singlefs-core/src/extent_tree.rs`、`crates/singlefs-core/src/instance_table.rs`、`crates/singlefs-core/src/journal.rs`、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-core/src/make_filesystem.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/mounted_read.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/rollback_witness.rs`（删）、`crates/singlefs-core/src/root_record.rs`、`crates/singlefs-core/src/system_configuration.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/unit.rs`、`crates/singlefs-core/src/write_request_split.rs`、`crates/singlefs-format/src/lib.rs`、`crates/singlefs-harness/src/bin/first_transaction_on_device.rs`、`crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs`、`crates/singlefs-harness/src/crash.rs`、`crates/singlefs-harness/src/crash_injection.rs`、`crates/singlefs-harness/src/device_log.rs`、`crates/singlefs-harness/src/fault_injection.rs`、`crates/singlefs-harness/src/first_transaction_regions.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/layer0_progress.rs`、`crates/singlefs-harness/src/lib.rs`、`crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/src/on_device_modes.rs`、`crates/singlefs-harness/src/read_tally.rs`、`crates/singlefs-harness/src/segments.rs`。

按路径点名、不判的实验装置（对不对由各自实验页的变异表与单测判）：`crates/singlefs-harness/src/bin/e142_first_transaction_write_dump.rs`、`crates/singlefs-harness/src/bin/e142_first_transaction_write_dump_one_device.rs`、`crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`、`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 | 改了 | 2026-09-27 改了：「释放」一格补隔离单元例外，用户定照 D19，随第三批 kb 写回 |
| D18（块里携带什么信息） 已定项 11 | 改了 | 2026-09-27 改了：逐盘「不可见」核扩到挂着之后每个收盘表的入口，用户定乙，随第三批 kb 写回 |
| D19（块指针的结构与宽度预算） 已定项 5 | 不受影响 | 2026-09-27 不受影响：用户定照它，硬规则 1 原句不动 |
| D13（验证路线） 已定项 4 | 不受影响 | 2026-09-27 不受影响：Z5 兑现字面 |
| D13（验证路线） 已定项 5 | 不受影响 | 2026-09-27 不受影响：Z7 兑现字面 |
| D13（验证路线） 已定项 7 | 不受影响 | 2026-09-27 不受影响：Z6 兑现字面 |
| D28（挂载期承诺量） 已定项 1 | 不受影响 | 2026-09-27 不受影响：Z2 兑现 |
| D28（挂载期承诺量） 已定项 3 | 不受影响 | 2026-09-27 不受影响：Z2 兑现 |
| D28（挂载期承诺量） 已定项 4 | 不受影响 | 2026-09-27 不受影响：式子改 K1 出自 A4c 与用户定案，不出自这一轮 |
| D16（发布语义） 已定项 1 | 不受影响 | 2026-09-27 不受影响：Z2-5 兑现 |
| D2（RAID 条带策略） 已定项 13 | 不受影响 | 2026-09-27 不受影响：Z2-4 兑现 |
