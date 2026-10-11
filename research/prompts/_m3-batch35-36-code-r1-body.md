# 删除批第 35 批 W（core 修补）与第 36 批 V（GPU）合入的代码：代码轮第一轮正文（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 W1 W2 W3 W4 W5 W6 W7 V1 V2 V3 V4 V5 V6 V7 F1 F2 A B C D E S1 R2 -->

## 一、题面

删除批第 35 批 W 与第 36 批 V 2026-10-08 先后合进主工作区（`records/2026-10-01-删除与并发事务.md` 同日那几条）。W 照 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第三节第 1 条与根环线 `research/prompts/m3-batch10y-code-r1-main-verification.md` K4-1 修 core；V 照同一份判决第三节第 2 条与 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第三节第 1 条改 GPU 判器与 CPU 走读。条款已写回 `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 与 `.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1 第九项。这是形式化线拍最终冻结树之前改 core 的最后一轮。这一轮判两件事：代码做的是不是判决定的改法；哪一格会错。

改动全文在 `research/prompts/_m3-batch35-36-code-r1-diff.md`。冻结副本 `~/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/`（V 合后那一版；之前已合根环线 10z 第一块，它的三份文件不在这一轮被判之列）。被判文件的 sha256 在 `research/prompts/m3-batch35-36-code-r1-snapshot/crates-sha256.txt`（20 份）。实现员报告（结论句只当线索）：`~/.local/share/singlefs/work/m3-batch35/w/report.md`、`~/.local/share/singlefs/work/m3-batch36/v/report.md`。

| # | 改动 | 主要落点（冻结副本） | 定的改法 |
|---|---|---|---|
| W1 K4-1 | C 那一圈去掉回退候选根的豁免，取窄读：只豁免现行账里仍分配着的槽（主 agent 认，免得第九项与「已分配」重复算；这次挂载里释放、回收时靠 `crates/singlefs-core/src/allocator.rs` 里 `protected.extend(placements_named_by_unapplied_journal_records())` 那一份补隔离） | `crates/singlefs-core/src/mount.rs` | T 判决第二节「T1 C 的候选豁免」 |
| W2 C_ACCOUNT | 水位之上每次没施加的发布，用任一条读得出的记录的新根段拼一条根、按影子账同一份读法读它的账，账里的落点并进 C 的集合 | `mount.rs` | 「T1 C 的盲区」 |
| W3 F1_ANY | F1 的「点名单元验不过」改成某个点名单元两份副本都验不过；重放那一份照旧要两份都验过，两份共用读一份副本的小函数 | `crates/singlefs-core/src/recovery.rs`、`mount.rs` | T4 行 |
| W4 F2_EXACT | 只退到 c_见证 那条记录所属那一版本身；那条记录读不出、c_见证 = 0、那条根读不出都不退 | `recovery.rs` | T5 行 |
| W5 | `read_recovery_fields` 照同一份 F2 判法算期望 | `crates/singlefs-harness/src/mount_transparency.rs` | T5 行 |
| W6 | 排演函数改交回 `Result`、调用方先判记录再排演；两句旧注释；一条测试名去日期 | `mount.rs`、`allocator.rs` | T8 行 |
| W7 变异 | 追加 15 行、替换 4 行、删第 33 批 T 的两行（F2 乙「两条都读不出就退」「施加过记录……也退」，`F2_EXACT` 之后不再红） | `crates/mutations.tsv` | — |
| V1 GPU 恢复内核补 F2 | 照 CPU 的 `F2_EXACT`：c_见证 从两个系统配置槽现算，退到根环里 (txg, 实例) 与 c_见证 那条记录恰好相等、读得出的那条根；走读一个调用点走两遍 | `crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl` | T 判决第三节第 2 条 |
| V2 H-S1-fold | CPU 走读全部版本走完之后再按最终的折叠表把每一版的落点折成单元判 I-3.5 | `crates/singlefs-checker/src/walk.rs` | S 判决第三节第 1 条 |
| V3 H-S2-tail | GPU `read_at` 读空段最后一槽的 32 KiB 时拼出整段 CRC（版本表第 8 字，`JUDGE_TABLES_VERSION` 5→6）；CRC 对得上而调用方还要读字节的记判不了、回落 CPU；「16 KiB 节点加下一槽」一条流钉两边同判 | `crates/singlefs-checker-tier/src/crash_judge_common.wgsl`、`crash_judge_checker.wgsl`、`crash_judge_tables.rs` | 同上 |
| V4 预算 | 三个内核共用的 `read_unit_via_locations` 两次读并成一个调用点（派发没点名，实现员为预算加的；根环线 2026-10-08 逐条看过认等价）；钉值 恢复 318224、扫描 352195、走读 346138 | `crash_judge_common.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` | 预算 400000 |
| V5 | 一形坏镜像（两份都读不出、第 0 条落点早被走过）；删完那条流加「CPU 无违例」、C744 ① 那条流加「两种版本同在至少一个」；对拍文件 5 处借卡改经 `lease_card_or_report_not_run` | `crates/singlefs-harness/tests/checker_known_bad_images.rs`、对拍文件 | S 判决；根环线托付 |
| V6 ①b 两形 | 意图刚发完、第一批写到中间，各挂两次续做，两份盘逐字节相同 | `crates/singlefs-harness/tests/unlink_crash_image_remounted_twice_continues_to_identical_disks.rs` | D8（核心索引结构） 已定项 16 第 7 条 ①b |
| V7 变异 | 追加 16 行 | `crates/mutations.tsv` | — |

**已知、腿不必再报**：`crates/mutations.tsv` 第 3116、3117 行（第 33 批 S 的两行 GPU 变异）原文被 V 改没了，进下一批重取锚点；慢档用例 `crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_is_red_only_on_i_7_4_before_any_floor_raise_in_the_known_double_fault_form` 期望该改成不红（C 护住了）；F1 的拒在 `mount_transparency.rs` 归类（W 停下的第 6 条）进下一批；`read_at` 注释点名「拼读切片后 16 KiB 没有字节」（根环线建议）进下一批。

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- W 合入验证「✓ 测试全绿（177 个测试二进制）」「✓ fmt 过」「✓ clippy 过」（日志 `~/.local/share/singlefs/work/merge-and-verify/2026-10-08-861601/`）；V 合入验证「✓ 测试全绿（178 个测试二进制）」「✓ fmt 过」「✓ clippy 过」（日志 `…/2026-10-08-932164/`），code-source-discipline 红在第 3116、3117 行（已知，见第一节）。
- 主 agent 在 V 合后的主工作区上清场上卡跑对拍文件（release、`verdict-store,gpu`，`SINGLEFS_HEAVY_TESTS=user-request`）：「test result: ok. 159 passed; 0 failed; 2 ignored」，`CRASH_JUDGE_GPU_MISMATCH` 0 行、本次未跑 0 条（日志 `~/.local/share/singlefs/work/m3-batch36/gpu-agree-after-v.log`）。T 合入后二分出来的那 52 条（`~/.local/share/singlefs/work/bisect-gpu-red/`）都在里面、都绿。
- harness 慢档：T 合后那一趟见 `records/2026-10-01-删除与并发事务.md` 2026-10-08 那条（一条期望要改、`random_histories` 撞 8G 没跑完）；W、V 之后没重跑。
- 冻结副本取在 V 合后，20 份 `sha256sum -c` 全 OK。

方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（W1–W7、V1–V7）
- W1–W4：照 T 判决定的改法；W1 窄读之下「R 自己新分配的单元不在所选那一版的账里、照样隔离」是不是对每一种盘面都成立。
- W3：F1 与重放各用哪一份判法，重放的语义有没有变。
- V1：GPU 的 F2 与 CPU 的 `version_witnessed_by_the_system_configuration_to_fall_back_to` 是不是逐支同判（c_见证 怎么算、= 0、记录读不出、根读不出、`JournalPolicy::Ignore` 不退）。
- V2：改了取单元的时刻之后，「走过」、诞生代号、父单元那几把键是不是照旧；I-3.5 按条款字面判的落点集合对不对。
- V3、V4：拼读的 CRC 与 CPU 读 32 KiB 的结果在哪些取值上等价；并成一个调用点之后次序、早退、`no_slice`、判不了记号与改前逐条相同。
- V5–V7：照点名的。

### K2　哪一格会错
在冻结副本的拷贝上攻，只跑 harness 档（checker 档与 GPU 不跑；GPU 一侧读 wgsl、用 CPU 判法算期望）；用户决定的那几步放开扫，只固定前缀与故障：
- W1：一形窄读放错——现行账里仍分配、被豁免的槽在这次挂载里释放又过回收门槛、那条根的根槽还没写掉时被发出去（根环线指的那一形）。
- W2：C_ACCOUNT 读账读出被抛弃根之外的东西、或拼出的根读错一版，隔离了不该隔离的、或漏了该隔离的。
- W3 / W4：F1_ANY 放过一形单元其实读不出的；F2_EXACT 该退不退（落在用户接受的那一形之外的读不出）或仍退过已确认的那一版。
- V1–V4：GPU 与 CPU 判得不同的一形（含 F2 那一支、拼读那一支、并成一个调用点之后的读）。
- V2：改了取单元的时刻之后一形本该红的 I-3.5 漏判。

### K3　被判的文件名单
`research/prompts/m3-batch35-36-code-r1-snapshot/crates-sha256.txt` 里 20 份每份一格。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从判决定的改法推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮。
- 误红、漏判、漏红的口径同 m3-batch22-code-r1 正文第五节。

交付：
- 腿的报告写进 `research/prompts/m3-batch35-36-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 模型与脚本放 `research/prompts/m3-batch35-36-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取，包装脚本另给一个变量时在文件头写明变量名；编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G；harness 大测试二进制 8G），不用 `auto`；同一时刻只起一条 cargo；`compare` 比之前排序、去掉耗时字段、自己不依赖 `SINGLEFS_REPO`，用法写在文件头第一行；用自己的编译目录。草稿放 `~/.local/share/singlefs/work/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
