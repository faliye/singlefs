# 第 27–29 批代码轮第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers L1 L2 L3 L4 L5 G1 G3 G4 G5 C3 B4 KX1 KX2 T13 MUT -->

## 一、题面

第 27、28、29 三批合开一轮（一批一轮的例外，依据在 `records/2026-10-01-里程碑三形式化接入.md` 第二十一节开头：三批改动互相牵连，G3 / G4 / G5 都在挂载透明与崩溃注入，26-5 与 29-2 都在 GPU 判器）。改动全文 `research/prompts/_m3-formal-code-b29-r1-diff.md`（六份补丁照实现员交回的原样、按合入次序，加主 agent 自己的两处）：

- 第 27 批（`records/2026-10-01-里程碑三形式化接入.md#27`）：照判决 `research/prompts/m3-formal-code-b25-r1-main-verification.md`「三、去向」改。27-2：25-1 的 T13 回退用例与 drop-retain / drop-extend 两行变异，O1 新用例到第 3 片（改名 `model_compares_inode_creations_into_the_third_inode_leaf_container.rs`）与 KX1 / KX2，135 边界 core 那条纯函数用例；E171 装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 照登记修订 5 改（`research/prompts/e171-r1-prereg.md` 第 870–1079 行：甲 十件逐件 `catch_unwind` 与 `piece_panics`、丙 对拍 k = 6、no_root_write 先于对拍）。27-2b：模型 135 边界用例（`crates/singlefs-harness/src/model.rs` 的 `mod tests`）；装置补 `unattributed_panic_states=`（登记末尾「主 agent 认定：27-2 装置交回」第 2 条）。
- 第 28 批（`#28`）：照判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md`「三、去向」第 1 条改。28-1（CPU）：池级 checker `crates/singlefs-checker/src/walk.rs` I-9.16 ④ 改十一种、I-5.4 违例照报而「成立」要至少一棵树走到叶；G1 运行配置期望字节取自条款而不取被测 core；journal_tail 两半；G3 `ReadFaultsUnderTheRefusedMount`（崩溃注入交否、harness 瞬时读错交是）；G5 注入态不判二次崩溃透明；F3 取样补 C3 / B4。26-5（GPU）：`crash_judge_checker.wgsl`、`crash_judge_checker_scan.wgsl`、`crash_judge_common.wgsl` 的 I-9.16 ③ ④（十一种）、根槽 428..457 全 0、带树一版分配记录树根全零挂 I-5.4、码 2 节点指针头 extent 偏移，对拍五条坏流。
- 第 29 批（`#29`）：29-1 G4：新拒因 `SystemConfigurationUnreadableAfterTheReadStage` 在第二、三截加 C670 同形判法（新成员 `SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`），`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 的「盘 0 两槽撕裂」改判不合法、另起注入故障口径下合法的一条；E171 装置 `RefusalConditionNotMet` 穷举补一臂。29-2：GPU 恢复读者照 core 每条指针头判 extent 偏移（根记录四条择根时跳过；树表条目根指针、inode 内部条目子指针读失败码 63 / 64；多层树与 extent 段内子指针、实例表链读不出），走读内核补三处，对拍改一条、加六条。
- 主 agent：#9 补钉（`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs` 打的行与进度文件两个 SHA-256，取自根环线在「第 28 批 + 10e」树上点名跑的两趟日志）；只开一个特性编不干净的修法（`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs` 两处 cfg，`crates/singlefs-checker-tier/src/crash_amplification.rs` 六个常量 import 改挂 `cfg(feature = "gpu")`、`BlockVerifier::Never` 加 `expect(dead_code)`）。

条款：上面两份判决与 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`；`research/prompts/e171-r1-prereg.md` 修订 5 与其后三节「主 agent 认定」；`.claude/kb/decisions/13-验证路线.md`「#### 已定项 1」挂载透明逐字段表；`.claude/kb/invariants.md` I-2.4、I-5.4、I-5.5、I-9.16 四行；`.claude/kb/checks-owed.md` C655、C664、C670、C687、C696 五行。

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 冻结副本 `/tmp/claude-1000/m3-formal-code-b29-r1/tree/`（主仓第 29 批与主 agent 两处改动合入之后的 `crates/`、`Cargo.toml`、`Cargo.lock`，sha256 清单在同目录 `crates-sha256.txt`）。这棵树里还有别的线同期合进来的改动（根环线 10c、10d、10e 与第 927 行那份，删除线第 18、19 批），它们不在这一轮被判；腿的反例落在它们的代码上时照写，主 agent 转给那条线。代码引文读冻结副本，写 `crates/…` 相对路径与那棵树里的行号。要编就拷到自己的草稿目录。
- 合入验证：27-2b `/tmp/claude-1000/merge-and-verify/2026-10-05-545460/`（harness 档 160 个测试二进制全绿），第 28 批 `…/2026-10-05-859756/`（160 个全绿），第 29 批 `…/2026-10-05-1290166/`（161 个全绿；随机历史 release 另跑一趟）；fmt、clippy 都过。只开一个特性那处：四种特性组合 `cargo clippy -p singlefs-checker-tier --all-targets -- -D warnings` 都退 0。checker 档一条没跑（重型，归提交时）；GPU 对拍、E171 装置内联用例与 checker 档的变异行只编过。
- 实现员证红：27-2、27-2b 点名的变异抓到；28-1 9 行全抓；26-5、29-2 的 GPU 变异没跑（要 checker 档）；29-1 harness 档 3 行加换点名的那一行全抓。
- 主 agent 已知、已记进下一批的差别（腿可以坐实或补形，不用重报为新发现）：GPU 恢复内核在 MAC / nonce、算法类型、压缩码、压后长度几段与 core 判法不同形；码 2 多层树内部子指针、extent 内部子指针、实例表链指针三处 extent 偏移改了而没有变异行（现有流里造不出多层树）；27-2b 新用例罩不到 `judge_the_confirmed_loss_shapes` 的四个调用点。

方案按 `crates/` 今天的实现来谈。

## 三、格

### L1　第 27 批：b25 判决的改法与 E171 修订 5 装置
代码做的是不是 b25 判决「三、去向」与修订 5 说的；T13 回退用例、drop-retain / drop-extend、KX1 / KX2、135 边界两条用例各罩不罩得住它要证的行为（换一个取样点还红不红）；装置的逐件 `catch_unwind`、`piece_panics`、`unattributed_panic_states=`、k = 6 对拍、no_root_write 先于对拍，有没有一种 panic 或一种态既不进逐件计数、也不进 unattributed（漏算），或被算两次；`judge_the_confirmed_loss_shapes` 那四个调用点改坏了哪个测试会红（重点）。

### L2　第 28 批 CPU：b26 判决第 1 条
I-9.16 ④ 十一种在合法历史上会不会误红（哪一版、哪种盘面上合法地少一种）；I-5.4「成立要至少一棵树走到叶」会不会在合法历史上把该成立的报成不适用、或在坏盘上放过；G1 字节取自条款对不对、全不全；journal_tail 两半；G3 判别子是不是判得出「这一拒是因为注入的读故障」；G5 不判二次崩溃透明会不会放过一个只在注入态上露的坏实现；C3 / B4 两形取样在更长历史上罩不罩得住。

### L3　第 29 批 G4
没注入读故障的崩溃态上，新拒因的拒一律判不合法：有没有一段合法历史（含系统配置轮换的撕裂写、mkfs 清零、回退）让某块盘两个系统配置槽在没注入任何读故障时都读不出，因而误红；第二、三截判法与 C670（`.claude/kb/checks-owed.md` C670 那一行）同不同形；那份手工两槽损坏用例翻成不合法、注入故障口径下那一条判合法，是不是都从条款推得出；E171 装置补的那一臂。

### L4　GPU：26-5 与 29-2
GPU 判器上 I-9.16 ③ ④、根槽 428..457、分配记录树根全零、各处指针头 extent 偏移，与 CPU（池级 checker、core 恢复）在同一盘面上判得是不是逐格相同；找一个 CPU 红而 GPU 绿（或反过来）的盘面；I-9.16 ③ 在 GPU 上「只违反 ③ 的表超额整态交 CPU」那条路是不是真交得回；对拍新加的六条与改的那一条，各自坏掉 GPU 哪一处会红。

### L5　主 agent 的两处
#9 两个钉值的出处站不站得住（取自哪棵树、是不是同一棵）；cfg 改动之后，提交时门禁编的那种特性组合（两个都开）下，被改的用例是不是照样编进来、照样跑。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | L1–L5 | 从判决与条款正推每处改动做的是不是条款说的，每格给判定、引文与推翻条件 |
| 云端攻方（Opus） | L1–L5，重点 L1 的四个调用点、L3、L4 | 找反例：一段合法历史让新判法误红，或一个坏实现让它放过；在拷出的副本上造、量 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 照那一判；一次抽样的「没打中」只算这一次。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第二轮，换攻击面。
- 腿提的新判法、新用例算零轮。
- 重型测试不跑：checker 档的测试、GPU 对拍与 `crates/mutations.tsv` 整表都不跑，涉及它们的结论写「推的」；harness 档测试可以跑。

交付：腿的报告写 `research/prompts/m3-formal-code-b29-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b29-r1-<腿名>-model/`，带一份 `tools/rerun.sh`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
