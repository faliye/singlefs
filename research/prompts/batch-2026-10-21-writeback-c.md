# 写回员 c 报告（batch-2026-10-21 第二轮 批次 2，kb-scribe）

## 一、结论

- 41 行清单：写了 35 行（三语同改；变更史另加 6 项，第 40 行并在 D26 那一项里），剔出 2 行（第 3、4 行，layout/01 的 M1、M2），不改 4 行（第 14 行 feature-bits 位 2；第 7、8 行 F34 的 decisions.md 两行与第 15 行 M3 只列证据）。第三节逐行列。
- 写了 25 个文件（24 个 kb 三语文件 + decisions-history.md），63 处替换，`replace-batch.py` 一次落盘并回读一致。
- 没跑 `doc-decisions.sh --write`（并行写回，主 agent 全部交回后跑一次）。没用 `relabel-item.py`（没有翻状态的规格）。
- 变更史 `decisions-history.md` 只有中文一份（仓里没有 `.en.md`、`.ja.md`），派发提示里「英日两份变更史」无对象。

## 二、规格与规格检查

- 规格：`/home/fy5090/.local/share/singlefs/work/batch-2026-10-21-writeback-c/spec.json`（63 条，带依据）、`spec.md`（同内容）、`batch.json`（replace-batch 用，不带依据）、`spec-zh.json`（25 条中文）、`spec-translations.json`（38 条译文）。sha256 在同目录 `spec-sha256.txt`：
  - spec.json 2eff63b3e6aad6a7632e946d2a59f6992fe8e9ecd4130256a9b96f84551af886
  - spec.md 43740d85e341e382dc73cd3a275198501f875db797c70d0b83842079a369278d
  - batch.json 8a0f6e8fe29894a397b750e9e24d732aa7dd5cca2352e7a5f9c5a7799c0f2753
- 写之前 `python3 scripts/kb/kb-spec-check.py spec-zh.json`，原样末行（退出码 0）：
  `  ✓ 规格 25 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史形态都过（查了 25 条）`
- 整份 `spec.json`（63 条）写之前的结果：旧串全部恰好命中 1 次；唯一的红是 10 条译文新串里的编号引用（退出码 1），原样末两行：
  `  ✗ 规格 63 条里有 10 处会让门禁红（逐处列在上面）`
  `  → 怎么办：按每处改规格原文再跑一次这个脚本；改不动的（要判断的）交主 agent，不派书记员`
  这 10 处全是同一类：`kb-spec-check.py` 的编号引用检查只认全角括号写的中文简称（`following_name` 只认「（」），而译文照 `.claude/kb/translation-glossary.md` 文件头要求写成半角括号加译名（`D26 (Background Compaction and Placement Reclamation)`），所以译文里新出现的 D26、C820 一律判「没带简称」。逐条：第 14、15 条（layout/01 en、ja 的 C820）、第 41、42 条（D5 en、ja 的 D26）、第 47、48 条（D8 en、ja 的 D26）、第 53、54、56、57 条（freeze-layer-membership en、ja 的 D26）。改不动（改成全角中文简称会违反术语表与 trilingual-glossary 格），按「改不动的交主 agent」列在这里。写之后再跑，旧串已不在，会全红，不作结果。
- 写之前每个路径都过了写范围闸（`write-guard.sh`，25 个路径逐个退出码 0，检出记录 `precheck.jsonl` 在草稿目录）。

## 三、逐行（41 行）

| 行 | 组 | 载体 | 处理 |
|---|---|---|---|
| 1 | F14 | layout/01:415 | 写了（zh、en、ja）：取号两轮——`[取号第一轮…] 屏障 [取号第二轮…] 屏障 [实例表单元…]`、孤立看 `2+2+18+2+1+2`、固定脚本流这几段 `2+2+2+18+2+1+2+16`、「取号每轮之后那道池屏障由取号自己发」；段号「第 15–21 段」删掉没换新号（见第五节） |
| 2 | F16 | D15:92 | 写了（三语）：「整池拒绝挂载（2026-10-21 定，D22 已定项 9）」 |
| 3 | M1 | layout/01:410 | 剔出（见第四节） |
| 4 | M2 | layout/01:114 | 剔出（见第四节） |
| 5 | F32 | D18:134 | 写了（三语）：`crates/singlefs-checker/src/rebuild.rs` 的扫描重建是只读的第一版（不写回盘） |
| 6 | F33 | D26:184 | 写了（三语），与第 19、21、23 行（F40）同一句合并改 |
| 7 | F34 | decisions.md:175 | 不改，列证据（第六节） |
| 8 | F34 | decisions.md:297 | 不改，列证据（第六节） |
| 9 | F34 | D5:31 | 写了（三语）：已定项 21 进度 未实现 → 已实现；decisions.md 的生成块与计数由主 agent 跑 `--write` |
| 10 | F34 | D5:456 | 写了（三语）：「全部根的 txg 列表（有：树表里不带销毁中记号的码 14 条目集合）。I-3.7 的 checker 已实现。」；另见第五节第 1 点（依据行同一事实多改了一句） |
| 11–13 | F38 | D26:28（zh、en、ja） | 写了：`empty_segments` 已是记录口径，C372 的代码那一半已做 |
| 14 | M1 | feature-bits.md:19 | 没补（见第四节第 3 点） |
| 15 | M3 | D26:13 | 进度一律不翻，列证据（第六节） |
| 16–18 | F39 | layout/01:310（zh、en、ja） | 写了：第一版 `N₀` = 1（算式等计数模型验过再定，欠 C820）；「新池新建文件恒过」一格没动（第五节第 3 点） |
| 19–24 | F40 | D26:184、:296（zh、en、ja） | 写了：射程里 discard 的产品路径调用者（`discard_the_segments_reusable_as_a_whole` 在用户改动的发布与整理显式入口之后调用）；未定项 9 取法表「产品路径的调用者」一格 |
| 25–27 | F41 | D5:232（en、ja、zh） | 写了：logged_ops 写 0（整理意图条目的字段表已定，见 D26 已定项 2；C364 仍欠 I-4.4 的判法） |
| 28–33 | F41 | D8:110、:265（en、ja、zh） | 写了：:110「字段表已定…回退取哪一版、态别仍留」；:265 同 D5:232 |
| 34–39 | F41 | freeze-layer-membership:33、:106（en、ja、zh） | 写了：:33 字段表已定、key 10 = 种类 2 + 签发 txg 8；:106「字段表已定，回退取哪一版、态别留到整理落地那天」 |
| 40 | M2 | decisions-history.md:14539 | 写了：D26 同日块（已有 `### 2026-10-21`）加一项，三处改动合在一项里，含「产品路径的调用者」一格 |
| 41 | F46 | D10:19（zh、en、ja） | 写了：E10 只跑了第一段 21 片里的 4 片、读数只当线索、剩 17 片停下 |

## 四、剔出的行与没补的行

依据代码只在暂存区、不在 HEAD（841b3beab）的，不写。`git -c core.quotepath=false status --short -- crates/` 现取的未提交清单是 24 条（全是暂存的 M 或 A，没有未暂存的），其中含 `make_filesystem.rs`、`mkfs_bytes_judged_by_the_checker.rs`、`transaction.rs`、`mounted_session.rs` 等。

1. 第 3 行（M1，layout/01:410 mkfs 种根改成 `2+12+1+1+1+4`、27 次操作、闭式 4117）：剔出。
   - `git show HEAD:crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs | grep -n "14+1+1+1+4\|2+12+1+1+1+4"` 只命中第 280 行 `"14+1+1+1+4",`；`git diff --cached -- 同一文件` 才有 `-"14+1+1+1+4"` / `+"2+12+1+1+1+4"`。
   - 同一行里 E142 说明那句（`14+1+1+1+4`、「第二十四次及之前它少每盘 [0, 根环基址) 那两步清零」）没动，跟着那一批一起交那个会话。
2. 第 4 行（M2，layout/01:114 fsid 对外入口随机生成）：剔出。
   - `git grep -n "make_filesystem_drawing_the_filesystem_identifier" HEAD -- crates | wc -l` → 0；`git grep --cached -n …` 命中 `crates/singlefs-core/src/make_filesystem.rs:8`、`:72`（只在暂存区）。
   - 原始输出在 `facts/mkfs-check.txt`。
3. 第 14 行（M1，feature-bits.md 登记 incompat 位 2）：没补，交主 agent。
   - 位号唯一登记位是 D15 已定项 4 那张表（`.claude/kb/decisions/15-格式冻结政策.md:82` 写的是 `| incompat | 2..255 | 未分配；每开一条布局线追加一位 |`），表里没有位 2。
   - 代码在 HEAD：`git show HEAD:crates/singlefs-core/src/system_configuration.rs | grep -n INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT` → 第 35 行 `pub const INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT: u8 = 0x04;`（引入提交 0b926bd3f，注释写「位号的登记归阶段同步」）。
   - 要先在 D15 已定项 4 的表里加位 2 一行（这是改 D15 的定案，要主 agent 或用户判），再给 `feature-bits.md` 三语补记账行与第 19、21 行。

## 五、要主 agent 判的点

1. D5 已定项 21 的「依据」那一行（`无实验：…而那条 checker 未实现`）判决没列，我顺手改成「那条 checker 已实现」，三语都改了。原因：射程那句改成已实现之后，同一节依据行还写未实现，两句互相矛盾。不认的话把规格里 `来证，而那条 checker 未实现。` 那三条（spec.json 里依据写了「写回员加」的）改回。
2. D5 已定项 21 的「欠」行（`C144（快照谱系的区间标号没有载体）——「全部根的 txg 列表」那一格的账就记在它名下`）没动：列表现在有了（码 14 条目集合），C144 还成不成立由主 agent 判。
3. layout/01 准入行（zh :310）：`N₀` 改成 1 之后「新池新建文件恒过」一格没改，判决要求主 agent 核一眼（新池每块盘上至少有一个全空聚簇段才恒过；我没量）。
4. layout/01 里与取号两轮有关的段号，我只改了写行那一行，删掉了「第 15–21 段」没写新号。按 `覆盖写固定脚本流的段序列` 那串 80 个数现算（`re.search(r"覆盖写固定脚本流的段序列 `([0-9+]+)`", 文本)` 切开）：写行那一组 `2,2,2,18,2,1,2,16` 是第 16–23 段；值 16 在第 23、27、60、64、73、77 段，值 18 在第 19 段。
   - 暖机行（zh 里「覆盖写固定脚本流第 20–21、24–25 段各 `2+16`」）按此应是第 22–23、26–27 段；抬 F 行（「第 56–65 段」）应是第 58–67 段。这两行不在 41 行里，没改。
5. D26 未定项 9 取法表下面「还开着两问」（zh :298）：写着「在产品路径上发 discard 时答」，产品路径已经发 discard，两问是否已答、照不照旧由主 agent 判，没动。
6. D8 已定项 5（:110）与 freeze-layer-membership 的 :106：我只改了「字段表」半句；回退取哪一版、态别是否也已定（判决要核 D26 与 D23），我没核，仍写留到整理落地。freeze-layer-membership :33 那一行「判不动」的推导（key 编码既已定，判据哪一句卡在哪）要不要重推，判决点名主 agent 看，没动。
7. D26 :184 旧文里「命中 11 份文件」的清单：现查 `git grep -l "整理" HEAD -- 'crates/*.rs' | wc -l` → 65，我没重列，删掉了数和清单，只留仍成立的几条（水位常量、`mount_transparency` 两份、`reclaim_released_up_to`、`discard_at`）。删掉的有 `system_configuration.rs` 字段表注释、`recovery.rs` 一句注释（HEAD 里 `recovery.rs:4762` 已是 logged_ops 字段表检查，不再只是注释）、`crates/singlefs-format/src/lib.rs` 四个常量文档；旧文在 `git show HEAD:.claude/kb/decisions/26-后台整理与放置回收.md`。
8. 变更史 6 项的「依据」指到 `research/prompts/batch-2026-10-21-sync.md`（按派发提示写），`ls` 现查此刻这个文件不存在（主 agent 写）。变更史里还引了 `~/.local/share/singlefs/work/batch-2026-10-21-sync-judge-<段号>/report.md`（仓外路径，也是派发提示的写法），门禁若不收要主 agent 改。
9. 译文自拟的 C820 译名（术语表没有）：en `Compaction floor N₀ formula lacks a counting model`、ja `整理下限 N₀ の算式に計数モデルがない`。
10. E10 实验页与 `.claude/kb/experiments.md` 的状态仍写「未跑」，与 D10 现在写的「只跑了 4 片」不一致；那两处是 F46 的另外两行，不在我这份。
11. C364 的简称没动（用户定改题、全仓同步，不在这份）：新句里仍写旧简称「整理意图记录没有字段表」，改题时这几处（D5 :232、D8 :110 与 :265、freeze-layer-membership :33）会被全仓同步命中。
12. 需要主 agent 跑 `doc-decisions.sh --write --check decision-items-sync`（并行写回）：预期 `.claude/kb/decisions.md` D5 行进度列由「已实现 12 项 / 未实现 8 项 / 无对象 1 项」变成「已实现 13 项 / 未实现 7 项 / 无对象 1 项」，分项清单第 21 项（现 `—— 已定 · 未实现`，:177）变成 `—— 已定 · 已实现`；D26 行不变（「已实现 0 项 / 未实现 11 项 / 无对象 2 项」）。`decisions.en.md`、`decisions.ja.md` 对应行我没碰，跑完后核一下。

## 六、分项进度：这一份不翻，只列证据（HEAD = 841b3beab）

原始输出在 `facts/progress-evidence.txt`。

- D5 已定项 19（decisions.md:175，现 未实现）：
  - `git grep -n "D5（快照 / 空间记账机制） 已定项 19" HEAD -- 'crates/*.rs'` 命中 `records.rs:592`、`transaction.rs:5673`、`:5858`、`:5895`、`:5910`、`:7705` 与 `crates/singlefs-harness/tests/snapshot_and_compaction_over_a_file_other_than_the_first.rs:4`。
  - 判定函数是 `transaction.rs:5944` 的 `snapshot_lifetime_verdict`，它的结果枚举 `SnapshotLifetimeVerdict` 上 `:5895` 的注释写「第一版不带「n > 1」那一臂：稀疏旁表恒 0 条，判决 S9」；销毁快照 `publish_snapshot_destruction_intent` 在 `transaction.rs:8963`。
  - 稀疏旁表只有注册、没有写者：`git grep -n "sparse_side_table" HEAD -- 'crates/singlefs-core/src/*.rs'` 命中 `recovery.rs:3211` 与 `transaction.rs` 的树标识符字段（:3019、:3072、:3081、:3107、:13475）。05 正文该项「先问旁表」那一半实现的是「恒 0 条」的版本，是否记已实现由主 agent 判。
- D16 已定项 14（decisions.md:297，现 未实现）：
  - `git grep -n "D16（发布语义） 已定项 14" HEAD -- 'crates/*.rs'` 零命中；`git grep -n -E "快照发布点|同窗" HEAD -- 'crates/*.rs'` 零命中。
  - `UserChange::CreateSnapshot`（`mounted_session.rs:1737`）走 `publish_create_snapshot`（`transaction.rs:8885`，注释「独立发布，不与别的用户改动合在一起」）；同窗两个快照用 `(txg, 快照 id)` 打破并列这一半没找到源码点名。
- D26 已定项 1、2、3、5、6（D26:13、14、15、17、18，现全是 未实现；D26 行进度列「已实现 0 项 / 未实现 11 项 / 无对象 2 项」）：
  - `git grep -h -o -E "D26（后台整理与放置回收） 已定项 [0-9]+" HEAD -- 'crates/*.rs' | sort | uniq -c` → 项 1 共 20 处、项 2 共 4 处、项 3 共 2 处、项 5 共 6 处、项 6 共 1 处。
  - 项 1：`allocator.rs:842`（`empty_segments` 记录口径）、`:902`、`:936`（R 排他的 `lowest_empty_segment`）、`compaction_plan.rs:319`（R 排不排他）、`:421`（签发前判）、`checker/src/image.rs:287`（I-5.6）。
  - 项 2：`logged_operations.rs:41`（整理意图条目）、`transaction.rs:5430`、`:9147`、`crates/singlefs-harness/tests/compaction_intent_is_logged_and_resumed_after_a_crash.rs:1`。
  - 项 3：`admission.rs:849`、`transaction.rs:4818`（合取 7 `全空聚簇段数 ≥ N₀`）；`N₀` 取值 HEAD 里 `compaction_plan.rs:354` 写「第一版取 1」。
  - 项 5：`region_scan.rs:71`、`transaction.rs:9109`、`:10900`（搬迁不改诞生代号、分配代口径）、harness 的 `history.rs:5826`、`model.rs:250`、`model.rs:1696`。
  - 项 6：`compaction_plan.rs:486`（整理的运行时观测，起跑次数、批数、停机原因、用掉的根槽数）。
  - 另：`transaction.rs:9125` 的 `publish_relocation_batch`、`mounted_session.rs:1938` 的 `run_compaction`、`compaction_plan.rs:820` 的 `compaction_starts_after_a_user_change`；批大小条款没定处产品路径在签任何意图之前停下（`compaction_plan.rs:299` 注释与 `:416` 的 `BatchSizeNotDecidedByTheClauses`），项 1 的触发与停机哪些真做了由主 agent 判。

## 七、写之前现查的事实（命令与结果，HEAD = 841b3beab）

| 行 | 命令 | 结果 |
|---|---|---|
| 1 | `git show HEAD:crates/singlefs-core/src/transaction.rs \| grep -n 'fn write_the_witnessed_acquisition'`，再看函数体 | `:1225`；体内 `for _round in ACQUISITION_WRITE_ROUNDS`（:1235）、`pool.perform_commit_step(CommitStep::Barrier)`（:1254），取号两轮、每轮后一道屏障 |
| 2 | `grep -n '已定项 9' .claude/kb/decisions/22-单元原子性怎么合成.md`（读已定项 9 正文） | 正文「读者遇到系统配置的格式版本不是 1，整池拒绝挂载…用户 2026-10-21 定「条文改成整池拒」」 |
| 5 | `git ls-tree HEAD crates/singlefs-checker/src/rebuild.rs`；`git grep -n "扫描重建" HEAD -- 'crates/*.rs' \| wc -l` | blob a3f4a35f…在 HEAD；234 行（判决写 235 是工作区数）；文件头注释「扫描重建（只读，不写回盘）」 |
| 6、19–24 | `git grep -n` 上面第六节的函数名；`git grep -n "discard_at(" HEAD -- crates` | `discard_the_segments_reusable_as_a_whole`：定义 `mounted_session.rs:2405`，调用 `:1920`（用户改动的发布后）、`:1994`（`run_compaction` 后），函数体里 `device.discard_at(` 在 `:2464`（判决写的 1921、1995、2465 是工作区行号，HEAD 差 1） |
| 11–13 | `git show HEAD:crates/singlefs-core/src/allocator.rs \| grep -n 'fn empty_segments'` | `:846`，注释 :841–844 写「段内 64 槽都没有未释放分配记录的段数…记录口径」 |
| 9、10 | `git grep -n "fn publish_create_snapshot" HEAD -- crates`；`git show HEAD:crates/singlefs-checker/invariants/05-空间记账.md \| grep -n "I-3.7"`；`git show HEAD:crates/singlefs-checker/src/walk.rs \| grep -n "fn judge_snapshots_and_deadlists_of_the_versions"` | `transaction.rs:8885`；I-3.7 状态格「已实现（2026-10-15，…`IMPLEMENTED_INVARIANTS` 下标 94…」；`walk.rs:15872` |
| 16–18 | `grep -n 'N₀' .claude/kb/decisions/03-空间分配.md`；`git grep -n "N₀" HEAD -- crates/singlefs-core/src/*.rs` | D3 :148「`N₀` 第一版取 1」；`compaction_plan.rs:354`「第一版取 1」；欠 C820（`checks-owed.md:535`） |
| 25–39 | 读 D26 已定项 2 正文；`git ls-tree HEAD crates/singlefs-core/src/logged_operations.rs` | 字段表 key 10 = 种类 2 + 签发 txg 8、value 34、条目 44，四个 format-const 标记；`logged_operations.rs` 在 HEAD |
| 41 | `git ls-files research/results/e10-aging-compaction-2026-10-13 scripts/experiment-apparatus/e10_aging_compaction.rs`；读 `records/2026-10-13-里程碑四预开发.md:122` | 4 个 .out（main-s4、disk_1gib 三档）与装置在库；记录写「停在 21 片里跑了 4 片…读数（4 片，只当线索）」 |

## 八、`replace-batch.py`、改过的文件、前后哈希

`python3 scripts/kb/replace-batch.py --dry-run batch.json` 原样末行：`✓ 63 处都命中得对（25 个文件），--dry-run 没写`（退出码 0）。
`python3 scripts/kb/replace-batch.py batch.json` 原样输出（退出码 0，存在 `replace-batch-output.txt`）：
`✓ 63 处替换，写了 25 个文件（改名换上新 inode），回读一致`

变更史（`git diff -- .claude/kb/decisions-history.md`，写前写后各一次，写前的 diff 在 `history-diff-before.txt`）：写之前该文件相对暂存区没有未暂存的改动（`history-diff-before.txt` 为空；暂存区相对 HEAD 有别的会话的 1 行改动，没碰）；写之后未暂存的新增行全是这一轮的 6 项（D5、D8、D10、D15、D18 各新开 `### 2026-10-21` 块；D26 并进已有的 `### 2026-10-21` 块，加在块顶）。每项形态：`- 阶段同步 batch-2026-10-21：…` 加缩进两格的 `**依据**` 与 `**改前**` 两行。

第 2、3 步的 `--write`：没跑（并行写回，主 agent 全部交回后跑一次）；`relabel-item.py`：没用（没有翻状态的规格；D5 已定项 21 的进度标记是直接改那一行的规范标记）。

改过的文件与哈希（前 = 开工时，后 = 收尾时；完整哈希在 `sha256-before.txt`、`sha256-after.txt`，下面取前 12 位）：

| 文件 | 前 | 后 |
|---|---|---|
| `.claude/kb/decisions/05-snapshot-space-accounting-mechanism.en.md` | a0e9a5bd0cab | 4a2d8a0a3c01 |
| `.claude/kb/decisions/05-スナップショット-空間会計メカニズム.ja.md` | 1c25d17dc2ed | db6715b15f14 |
| `.claude/kb/decisions/05-快照-空间记账机制.md` | dc680474fbe5 | 0e611a0f8bbd |
| `.claude/kb/decisions/08-core-index-structure.en.md` | 71f690695def | f77bca16a391 |
| `.claude/kb/decisions/08-コアインデックス構造.ja.md` | 32ea4f30e1a5 | d8e7a8288a37 |
| `.claude/kb/decisions/08-核心索引结构.md` | 67c7451d751b | f29dea8aac93 |
| `.claude/kb/decisions/10-true-answer-to-random-writes-fragmentation.en.md` | 074ae1a2131e | 851a5642ca45 |
| `.claude/kb/decisions/10-ランダム書き込み-断片化の真の回答.ja.md` | eb083af3451d | c741a90cb6cc |
| `.claude/kb/decisions/10-随机写-碎片化的真答案.md` | eb65ae6ac912 | 2e96410a3670 |
| `.claude/kb/decisions/15-format-freeze-policy.en.md` | 756328f4cf10 | 1d3decf8439e |
| `.claude/kb/decisions/15-フォーマット凍結ポリシー.ja.md` | acb16f12dd93 | 261cfd6c14f6 |
| `.claude/kb/decisions/15-格式冻结政策.md` | d4408990f2e9 | 1ddc2b2435ff |
| `.claude/kb/decisions/18-information-carried-in-a-block.en.md` | 1e3202a42847 | c5e55e600fc5 |
| `.claude/kb/decisions/18-ブロックに含める情報.ja.md` | 0e50ebe1eaec | 03cad3b4ac16 |
| `.claude/kb/decisions/18-块里携带什么信息.md` | 4f5ad0038b91 | ff5c63e1db36 |
| `.claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md` | cfbc024888ff | 2356fe5c59a7 |
| `.claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md` | 5c90f6411283 | a37fb0b9184d |
| `.claude/kb/decisions/26-后台整理与放置回收.md` | d921c3a50273 | 8f446cfb5bfb |
| `.claude/kb/decisions-history.md` | f3c573715b90 | 873e66d2dc1d |
| `.claude/kb/decisions.md` | 9abb9da05230 | 9abb9da05230 |
| `.claude/kb/freeze-layer-membership.en.md` | de8a0c91a0e4 | 4f79e081927d |
| `.claude/kb/freeze-layer-membership.ja.md` | 41a4f163cfc8 | 202a4cc21e79 |
| `.claude/kb/freeze-layer-membership.md` | 931db7032a59 | 7e51edc179bf |
| `.claude/kb/layout/01-first-txn.en.md` | cf80e6452cf6 | 96e213cfc8ee |
| `.claude/kb/layout/01-first-txn.ja.md` | 2925d1b2b17b | 9a6186d51791 |
| `.claude/kb/layout/01-first-txn.md` | 2beaca3fe0f4 | 8b0c33ba9db0 |

`.claude/kb/decisions.md` 开工时与收尾时哈希相同（9abb9da05230…），没改。

## 九、没做什么

- 没判定案对不对，判决里的判断以判决为准；没写规格外的句子（例外：第五节第 1 点的 D5 依据行，已单列）；没修不是这一轮的红；没提交；没跑门禁阶段、没跑 `--write`。
- 没改 03、13、19、22、23 号决策、欠账表、不变量清单；没改 `.claude/kb/decisions.md`、`feature-bits.md`、`milestone/`、`experiments*`；没改 C364 简称。
- 没跑译文四格（trilingual-files、structure、changed-together、glossary）的门禁；自己用术语表对这 8 组（24 个文件）跑了一次新增违反的检查：中文列的串进了中文文件、英语/日语列的串缺在译文里的新增处数为 0。
- 没改 layout/01 其余段号（第五节第 4 点）、没核「新池新建文件恒过」（第五节第 3 点）。

推翻本报告结论的现象：`git show HEAD:<路径>` 里读不到第七节任一行写的函数或行；或 `git status --short -- crates/` 显示其中某个文件在 HEAD 之后又提交了与我引的函数不同的版本。
