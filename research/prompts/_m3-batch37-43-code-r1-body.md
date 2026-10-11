# 删除线第 37 批 X、第 38 批 Y、第 39 批 Z、第 41–43 批合入的代码：代码轮第一轮正文（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 X1 X2 X3 X4 X5 Y1 Y2 Y3 Y4 Y5 Y6 Z1 Z2 A1 A2 A3 A4 B1 C1 H1 H2 H3 H4 F1 F2 R P V -->

## 一、题面

删除线第 37 批 X、第 38 批 Y、第 39 批 Z、第 41、42、43 批于 2026-10-08 先后合进主工作区（`records/2026-10-01-删除与并发事务.md` 同日那几条）。

- X 照判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第三节第 1 条，修 W、V 留下的几处。
- Y、Z 照同一份判决的「四形都修」（用户 2026-10-08 定）改 core 与 GPU 恢复内核。
- 第 41 批收 Y、Z 留下的四件（records 第 1264、1266 行）；第 42 批修 Z 那条造流就红的对拍用例；第 43 批照用户 2026-10-08 定的「乙：core 另起错误成员」把 C_MAPPING 那一拒另起成员、列进 D23 单子。
- 条款在 `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14（2026-10-08 删除线第 44 批已写回 `C_NEWER_ROOTS`、`C_MAPPING`、`F1_AFTER_F2`、`F2_NEWEST` 与 C_MAPPING 那一拒在单子上）、`.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1 第九项。

这是形式化线拍最终冻结树之前改 core 的最后一轮。这一轮判两件事：代码做的是不是判决定的改法；哪一格会错。

改动全文在 `research/prompts/_m3-batch37-43-code-r1-diff.md`。冻结副本在 `~/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/`，取的是第 42、43 批合入之后那一版。被判文件的 sha256 在 `research/prompts/m3-batch37-43-code-r1-snapshot/crates-sha256.txt`。

实现员报告（结论句只当线索）：
- `~/.local/share/singlefs/work/m3-batch37/x/report.md`
- `~/.local/share/singlefs/work/m3-batch38/y/report.md`
- `~/.local/share/singlefs/work/m3-batch39/z/report.md`
- `~/.local/share/singlefs/work/m3-batch41/a/report.md`
- `~/.local/share/singlefs/work/m3-batch42/a/report.md`
- `~/.local/share/singlefs/work/m3-batch43/a/report.md`

| # | 改动 | 主要落点（冻结副本） | 定的改法 |
|---|---|---|---|
| X1 | `crates/mutations.tsv` 第 3075、3116、3117 行重取锚点 | `crates/mutations.tsv` | 判决第二节 K1 W7、V7 行 |
| X2 | 双故障慢档用例改名，期望改成抬 F 之前池级 checker 一条都不红 | `crates/singlefs-harness/tests/random_histories.rs` | 判决第三节第 1 条 |
| X3 | F1 的拒归进 D23 单子：`RefusalReasonOnTheD23List` 加成员，判据在崩溃镜像上复算；崩溃注入第三截补两臂、一格计数 | `crates/singlefs-harness/src/mount_transparency.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs` | records 第 1234 行（主 agent 定：在镜像上复算、不加 `RefusalConditionNotMet` 成员） |
| X4 | `read_at` 注释 | `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` | 根环线建议 |
| X5 | `allocator.rs` 里那句 `protected.extend` 的用例 | 新建的 harness 用例 | 判决第二节 K1 / K2 W1 行 |
| Y1 | `C_NEWER_ROOTS` | `crates/singlefs-core/src/mount.rs` | 判决 K2 W1 / W2 H1 行 |
| Y2 | `C_MAPPING`（收窄）：较新可读根映射算得出就护住，算不出不拒 | `mount.rs` | 同上；records 第 1266 行 ① |
| Y3 | `F1_AFTER_F2` | `mount.rs` | 判决 K2 W3 H2 行 |
| Y4 | `F2_NEWEST` | `crates/singlefs-core/src/recovery.rs` | 判决 K1 / K2 W4 H4 行 |
| Y5 | 第 3163 行换取样点 | `crates/mutations.tsv`、harness 用例 | 判决 K1 W7、V7 行 |
| Y6 | `reuse_after_raising_the_floor.rs` 同形用例改名、改期望 | `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` | 判决第三节第 1 条 |
| Z1 | GPU 恢复内核改 `F2_NEWEST`，实例表沿链读，超过 512 行或 256 片记判不了；对拍加 4 条流 | `crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl`、对拍文件 | 判决第三节第 2 条 |
| Z2 | 建模差两处对齐；恢复内核钉值改成 328934 | `crash_judge_recovery.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` | 同上 |
| A1 | c_见证 = 0 时不退，CPU、GPU 两边都加守卫 | `recovery.rs`、`crash_judge_recovery.wgsl` | D23 已定项 14；records 第 1264 行 |
| A2 | 恢复内核钉值 328934 → 328970 | `crash_judge_dispatch.rs` | 守卫多出的展开表达式 |
| A3 | F2 函数改名 | `recovery.rs`、`mount.rs`、`crates/singlefs-core/src/mounted_read.rs`、`mount_transparency.rs`、wgsl | records 第 1266 行 ③ |
| A4 | F1_AFTER_F2「重读之后故障撤掉」那一形的用例 | 新建的 harness 用例 | records 第 1266 行 ④ |
| B1 | Z 那条「所选根的实例表抛弃 V」对拍流改造法：R 的根记录与 R 那次发布记录里的实例表指针改指基镜像空槽上带 (V⁻ 实例, V⁻ txg) 那一行的表 | 对拍文件 | 主 agent 上卡跑时它红在造流那一步（records 同日那一条） |
| C1 | C_MAPPING 那一拒另起成员 `StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords`，与 C393 共用映射现算那一套；D23 单子加同名成员与镜像复算一臂（三样都成立判合法：被点名的是水位之上读得出记录的发布或较新有效根、读得出的记录不齐、顺树表指针找到的账根两份都验不过；映射现算那一半不复算，只判必要条件）；`crash_injection.rs`、`history.rs`、`model_comparison.rs` 补臂 | `mount.rs`、`mount_transparency.rs`、`crash_injection.rs` | 用户 2026-10-08「乙：core 另起错误成员」；判据主 agent 定、零轮 |

**已知、腿不必再报**：
- W1 键不带跨度那一形：Y 读码推出造不出，记欠账 C750（键不带跨度）。
- R 在 M1 那一刻整个看不见那一形：不修，C747。
- V2 折叠表漏判：C748。
- F1 拒因计数没有钉：C749。
- C1 的镜像复算只判必要条件：映射现算那一半不复算；账根以下的节点读不出时实现照拒、复算判不合法（多报红一侧）。
- 第 42 批那条流上 R 当挂载根的状态 CPU 多红 I-3.8（实现员读码推的），GPU 要同判，归主 agent 上卡核。

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- X 合入验证：「✓ 测试全绿（181 个测试二进制）」「✓ fmt 过」「✓ clippy 过」（日志 `~/.local/share/singlefs/work/merge-and-verify/2026-10-08-1376721/`）。
- Y 合入验证：「✓ 测试全绿（183 个测试二进制）」，fmt、clippy 都过（`…/2026-10-08-1689647/`）。
- Z 合入验证：同样全绿（`…/2026-10-08-1800503/`），整表静态检查 `CHECKED 3286 318 9`。
- 第 41 批合入验证：「✓ 测试全绿（184 个测试二进制）」，fmt、clippy 都过（`…/2026-10-08-2166401/`），static-check `CHECKED 3302 319 9`。
- 第 42、43 批合入验证：（合入后填）。
- GPU 对拍：主工作区加第 41、42 批那一版带 `--nocapture` 跑的那一趟（填）。Z 合后、第 41 批合后两趟都红在第 42 批修的那条造流上，没带 `--nocapture`，「本次未跑」条数不作数。

方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（X1–X5、Y1–Y6、Z1–Z2、A1–A4、B1、C1）
- Y1–Y4 与 A1：是不是逐支照判决第二节 K2 四行定的改法。
  - `C_NEWER_ROOTS` 认哪几条根：比所选那一版新、按实例表有效、读得出。
  - `C_MAPPING` 什么时候现算、什么时候拒。
  - `F1_AFTER_F2` 用退到的那一版当水位。
  - `F2_NEWEST` 的区间、实例表过滤、读不出时只认恰好相等、见证记录读不出不退、c_见证 = 0 不退、c_见证 落在所选那次发布自己的记录上不退。
  - 恢复、只读挂载、可写挂载三处是不是同一份判法。
- Z1、Z2、A1、A3：GPU 恢复内核的 F2 是不是与 CPU 逐支同判；实例表沿链读的上限与判不了记号回落 CPU 是不是对。
- X3、C1：D23 单子上的拒因与 `mount_transparency.rs` 的归类，逐成员对一遍；C1 的镜像复算是不是照那三样判、有没有调实现判拒的函数。
- B1：新造法是不是只让 R 认的那份实例表多那一行，V⁻、V 不受影响。
- 变异行：新行与改过锚点的行，点名的测试与替换文对不对。

### K2　哪一格会错
在冻结副本的拷贝上攻，只跑 harness 档（checker 档与 GPU 不跑；GPU 一侧读 wgsl、用 CPU 判法算期望）。用户决定的那几步放开扫，只固定前缀与故障。
- Y1、Y2：较新可读根的账读错一版，或映射现算漏了 / 多了，导致隔离了不该隔离的、或漏了该隔离的。
- Y2：「映射算不出不拒」这一支放过一形本该拒的，可写挂载之后单元被写掉。
- Y3：退到的那一版再判 F1 时，水位或点名单元取错。
- Y4、A1：`F2_NEWEST` 退过已确认的那一版，或该退不退；实例表过滤放错。
- Z1、A1：GPU 与 CPU 判得不同的一形。
- C1：C_MAPPING 那一拒与 C393 那一拒报错报串了；镜像复算把一次合法的拒判成不合法、或反过来（读故障下的取法照 X 那一拒）。
- 不重复第 35、36 批那一轮攻过的四形（H1–H4）本身，攻它们的改法。

### K3　被判的文件名单
`research/prompts/m3-batch37-43-code-r1-snapshot/crates-sha256.txt` 里的每份一格。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从判决定的改法推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批，改法算零轮。
- 误红、漏判、漏红的口径同 m3-batch22-code-r1 正文第五节。

交付：
- 报告写进 `research/prompts/m3-batch37-43-code-r1-<腿名>-output.md`。分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 模型与脚本放 `research/prompts/m3-batch37-43-code-r1-<腿名>-model/`。
  - `rerun.sh` 交回前在空目录里从头跑通一次。
  - 仓路径从环境变量 `SINGLEFS_REPO` 取；包装脚本另给一个变量时，在文件头写明变量名。
  - 编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`，上限显式给：编译 6G、跑 4G、harness 大测试二进制 8G，不用 `auto`。
  - 同一时刻只起一条 cargo，用自己的编译目录。
  - `compare` 比之前先排序、去掉耗时字段，自己不依赖 `SINGLEFS_REPO`，用法写在文件头第一行。
- 草稿放 `~/.local/share/singlefs/work/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
