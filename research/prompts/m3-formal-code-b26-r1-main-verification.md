# 第 26 批代码轮第一轮判决（2026-10-05）

<!-- doc-lint:not-numbers J1 J2 J3 C1 C2 C3 C4 G1 G2 G3 G4 G5 G6 B4 C670 MUT -->

被判的文件（第 26 批三份补丁动到的全部 `crates/`）：
- 26-3：`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker/src/image.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-harness/src/mount_transparency.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs`、`crates/singlefs-harness/tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs`；
- 26-1r：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-harness/src/fault_injection.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/src/mount_transparency.rs`、`crates/singlefs-harness/src/reference_filesystem_translation.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`、`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs`，以及它的十份测试文件（改动全文 `research/prompts/_m3-formal-code-b26-r1-diff.md` 26-1r 一节逐份列）；
- 26-2：`crates/singlefs-checker-tier/src/crash_identity.rs`、四份 checker 档测试、`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs`；
- `crates/mutations.tsv` 三份追加、替换 5 行、删 RR8。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b26-r1-body.md`、`_m3-formal-code-b26-r1-diff.md`、`_m3-formal-code-b26-r1-background.md`（第一次生成 1.28 MB 那一份挪在会话草稿 `b26-materials-v1/`，欠账表只取表头与五行重做） | — |
| 冻结副本与快照 | `/tmp/claude-1000/m3-formal-code-b26-r1/tree/`（第 26 批合入后、根环线 10c 之前），清单同目录 297 行；kb 快照 `research/prompts/m3-formal-code-b26-r1-snapshot/kb-sha256.txt` | 写判决前核过 |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b26-r1-sonnet-output.md`（sha256 26e77ccd…），模型 `…-sonnet-model/` | 主 agent 照 `tools/rerun.sh all` 复跑 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b26-r1-opus-output.md`（sha256 2624cdb9…），模型 `…-opus-model/` | 主 agent 照 `tools/rerun.sh` 全段复跑 |
| 引文 | 两份腿报告 | 两腿各自 cite-check 绿（86 / 86、38 / 38） |

## 二、逐格判决

### J1　26-3

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 七处改动做的是不是判决说的 | 一致 | — | 站住 |
| 合法历史上误红（I-9.16 ③ ④、I-2.4 根槽、I-5.4 新格、I-5.5、挂载透明新行） | 没见误红（7 种底盘面整挂载、341 个二次崩溃前缀态） | 没打中：14 段写死步序全量层 0 共 354 278 态、第一次崩溃后挂载 139 513 次、二次崩溃 7 531 673 个，0 违例 | **两次独立抽样都没打中** |
| I-9.16 ④ 放过缺种类的树表（C1） | 打中：去掉 deadlist 那一条，checker `violated=[]`、I-9.16 `Holds`，core 可写挂载拒「树表里没有那棵树」 | 没量这一格 | **坐实**（复跑 `PROBE_I916 deadlist_entry_6 violated=[]`、`PROBE_CORE writable_mount_outcome=Err(Recovery(UnitMalformed { what: "树表里没有那棵树" }))`）。分辨的事实：core 重建一版时十一种各要一条（冻结副本 `crates/singlefs-core/src/recovery.rs:2539-2561` 逐种 `tree_of`）。C655 / C687 判决第一节那句「core 在树表非空时要的有根的树是种类 1、2、3、4」只读了同一函数前面取四条根指针的几行，前提读窄了；这里更正：**④ 改成树表非空时十一种各恰好一条**（`walk.rs` 的 `TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`） |
| I-5.4 挂法（C2） | 打中：分配记录树两份清零时 I-5.4 报 `Holds`（旧代码报不适用），评估一次就压过不适用 | 没打中（读码看层 0 计数不受影响） | **坐实**（复跑新代码那一半 `zeroed=Holds`；旧代码那一半读码：`walk.rs:4905-4908` 没有一棵树走到叶时整条不适用，没复跑）。改法：带树一版分配记录树根全零那一判只在至少一棵分配记录树走到叶时计进 I-5.4 的评估；一棵都没走到时照旧报部分没判 |
| 运行配置「R2 = 内置默认」（J1-c） | 一致 | 打中：期望取自被测 core 的私有常量，副本上 T_time 改 4 挂载透明 45 条全绿；同一改坏由系统配置字节钉兜住 | **坐实**（复跑 `mutations.txt` 同）。改法 G1：期望字节在 harness 里照条款写死（`.claude/kb/decisions/16-发布语义.md` 与 `.claude/kb/layout/01-first-txn.md` 的可调值行） |
| 字段表外的两段（J1-d） | 没提 | 推的：根槽 [457, 槽宽) 与系统配置槽 [489, 4096) 没人判，条款只说余量留给将来 | **条款缺口，不在这一批改**：第一版恒 0 还是不判要先定条款，立一笔欠账 |
| journal_tail、journal 实例代号两行 | 一致（推的）；「R1 ≤ R2」与「挂载做完取等」两半没有红用例与变异 | 没打中 | 站住；补两半各一条会红的用例与变异 |
| 字节区间字段表、层 0 钉值 | 一致 | 没打中；26-2 #8 的 69 条在合入后冻结副本上逐条复现 | 站住（复跑 `quick-4gib-cmp.txt` 同） |

### J2　26-1r

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 注入读故障的态上「新拒因复算为假不算不合法」（C3 / J2-a） | 冲突：盘面健康也判合法，被判的挂载其实没有读故障 | 打中：崩溃注入交的标记是「历史里有没有读故障步」，被判的那次挂载是不带故障层的内存盘，瞬时读错不会发生；zeros 那组比重 29% 的段被认成注入过（今天遮住 0 次） | **两腿同判，坐实**（复跑 `frac.txt` 同）。这条放宽是主 agent 在 26-1 交回时收下的，撤回：改法 G3 第二种——判官加一个入参「被拒的这次挂载自己在读故障下跑没有」，崩溃注入恒交否，harness 那份造瞬时读错的用例交是 |
| 新拒因少一条 C670 同形的判法（J2-b） | 没提 | 打中：坏实现 MUT-B1 下无故障崩溃态上 90 次新拒因全判合法，checker 0 违例 | **坐实**（复跑 `mutB1-…txt` 同）。不是 26-1r 引入，拒因分开之后才露成单独一格。改法 G4：没注入读故障的崩溃态上新拒因复算为真，报「正确实现摆不出这份盘面」；零轮，实现员要在随机历史与崩溃注入快档上核它不误红 |
| N-配置 去掉那一支 | 一致，收严 | 没打中 | 站住 |
| 二次崩溃镜像在注入态下照判（J2-d） | 没找到误红 | 没打中误红（1 848 852 个），但与 D13 射程句「带故障的状态不在射程」字面不一致 | 改法 G5：二次崩溃那一截照第一截按标记跳过，与射程句一致 |
| 新步与新比重 | 一致，六组指纹在 HEAD 上复现 | 没打中 | 站住 |
| F3-e 取样（C4 / J2-f） | 罩住点名三行；另有漏形 | 打中：只折前或末 32 个写、历史种子只留两位或对 3 取余，入库七条全绿 | **坐实**（复跑 `f3.txt`、`seglen.txt` 同）。这组用例 b24 已是最后一轮；G6 的 C3、B4 两条取样只加宽测试、攻方模型上量过各抓一类，加进来，不再开轮；剩下的漏形记进 b24 那笔欠账 |
| 替换的 5 行与删的 RR8 | 一致 | 没打中，逐行改坏都红 | 站住（复跑 `mutations.txt` 同；正推 38 行逐条打全红） |

### J3　26-2

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| #1、#2–4、#6、#7、#8、C686 | 一致 | 没打中 | 站住；根环线在「10b + 第 26 批」那棵树上点名重跑 #1–#8 全绿（记录第十八节），#7 的 9 坐实 |
| #9 进度文件钉值 | 「待重录」今天必红 | 同 | 补钉：根环线取到的 `391f6f68a754b6ca4b231d01aaa89193b6974f8751ca0a4bec25e1e2b77bac12`；根环线 10c、10d 合入后再跑一次，变了以新值为准 |

## 三、去向

1. **第 28 批实现员**（补丁，在今天主工作区上做——已含根环线 10c、10d 与删除线第 18 批）：I-9.16 ④ 改十一种（坏镜像：缺 deadlist、缺 logged_ops 各一份，只红 I-9.16）；I-5.4 挂法（C2）；G1；journal_tail 两半的用例与变异；G3；G4；G5；G6 的 C3、B4；#9 补钉；27-2b 留的「新用例罩不到四个调用点」那一格不在这一批。
2. **第 26-5（GPU）**：在删除线 P2 之后的树上做 I-9.16 ③ ④（④ 按十一种）、根槽与带树一版分配记录树根，连同根环线报的「GPU 读码 2 节点不判指针头部 extent 偏移」。
3. **写回**（并进写回批）：I-9.16 ④ 照十一种写；I-5.4 那一句照 C2 的改法写；C664、C696 还清条件对照本判决；新立欠账两笔：根槽与系统配置槽尾部字节的条款、F3 剩下的漏形（并进 b24 那笔）。
4. **下一轮代码轮**（第 28 批合入后）：攻 G3、G4、G5 与 ④ 十一种。

## 四、复跑核对

攻方：`REPO=$PWD FROZEN=/tmp/claude-1000/m3-formal-code-b26-r1/tree WORK=<会话草稿>/b26-opus-rerun bash research/prompts/m3-formal-code-b26-r1-opus-model/tools/rerun.sh`（全段），退 0；脚本自己的 `compare` 段 20 份里 19 份 `SAME`，`r1-file-zeros.txt` 一份 `DIFF`，主 agent 逐行看：差在 `elapsed_seconds`、`elapsed_secs` 两行与腿那一趟多打的 `time` 输出四行，内容同。

正推：`REPO=$PWD FROZEN=… WORK=<会话草稿>/b26-sonnet-rerun bash research/prompts/m3-formal-code-b26-r1-sonnet-model/tools/rerun.sh all`，退 0：#9 的 58 行 SHA-256 `7e64a883…` 同；I-5.4 新代码 `zeroed=Holds`、I-9.16 缺种类 `Holds` 而 core 拒，同报告；38 行 harness 档变异逐条红。

核不动：checker 档用例、变异整表、崩溃注入战役与 GPU（两腿都没跑）；C2 旧代码那一半读码。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | I-9.16 ④ 十一种 | 正推量到缺口，主 agent 读码定范围 | 缺口量过、改法没实现 | 第 28 批 |
| 2 | G3 判官入参「被拒的挂载自己在不在读故障下」 | 攻方 | 射程量过、改法推的 | 第 28 批 |
| 3 | G4 新拒因的 C670 同形判法 | 攻方 | 坏实现下数过、判法推的 | 第 28 批 |
| 4 | G5 二次崩溃按标记跳过 | 攻方 | 推的 | 第 28 批 |
| 5 | 槽尾字节第一版恒 0 还是不判 | 攻方 | 推的 | 欠账，要先定条款 |

不涉及要用户定的岔路；第 5 条以后要定条款时再交。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 1 | 支撑 | 2026-10-05 不受影响：G5 让代码照射程句「带故障的状态不在射程」办，条款不改 |
| D8（核心索引结构） 已定项 8 | 不影响 | 2026-10-05 不受影响：树表字段表不变，I-9.16 ④ 照 core 的读法收严 |
| D22（单元原子性怎么合成） 已定项 7 | 备料 | 2026-10-05 不受影响：根槽余量的条款缺口记欠账，没改字段表 |
