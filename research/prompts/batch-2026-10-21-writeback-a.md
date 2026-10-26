# 写回员 a 段报告：阶段同步 batch-2026-10-21 落在欠账表的行

只写了 `.claude/kb/checks-owed.md`、`.claude/kb/checks-owed.en.md`、`.claude/kb/checks-owed.ja.md`，三份一起改；决策正文、不变量清单一个字没碰，没做 git 写操作，没跑门禁阶段与 `--write`。

## 替换清单

- 路径：`/home/fy5090/.local/share/singlefs/work/batch-2026-10-21-writeback-a/replacements.tsv`，**121 行 = 做了 121 次替换**（`wc -l` 得 121；每行三个制表符分隔的字段，插行那 10 行是「文件、空旧串、新行、锚点」四字段，`awk -F'\t'` 数得 111 行 3 字段、10 行 4 字段）。
- 构成：`replace-once.py` 定点替换 101 次（中文 27、英文 37、日文 37，其中 24 次是八笔「待合入后验证」去向格补句）、`replace-once.py` 删行 10 次（旧串 = 整行加换行，新串空：英、日各把 C576、C296、C587、C668、C827 五行从开着的表里删掉）、`insert-row.py` 插行 10 次（同样五行，译成已还清表的四列行，插在中文已还清表同一前驱行之后：C576 在 C568 后，C296 在 C576 后，C587 在 C296 后，C668 在 C587 后，C827 在 C689 后）。
- 旁路：`apply.py`（草稿目录）逐次调 `scripts/kb/replace-once.py` 与 `scripts/kb/insert-row.py`，成功才追加一行；没用 Edit，没整份重写。
- `replace-once.py` / `insert-row.py` 全部成功退 0（`apply.out` 末行 `fails 0`）。

## 规格与规格检查

- 规格：`.../spec.json`（sha256 `74c8e48c51f8205f5feed14f5e62f5c4515fc5f4edb825364f14a0fe44c85b16`）与同内容 `spec.md`（sha256 `72c59f79870e233200ab41919367c081d5c30dd4e8b856f10dd2ed319b0d66d9`）。115 条；连续插入的几行在规格里并成一条（锚点 = 前驱整行，新串 = 前驱整行加新行）。
- **写之前**跑 `python3 scripts/kb/kb-spec-check.py spec.json` 的原样末行：`  ✓ 规格 115 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史形态都过（查了 115 条）`，退出码 0。（中间红过一次：C751 行号替换的旧串在中文、日文页各命中多次，补了上下文后绿。）
- 写完之后再跑同一条会红，因为旧串已经不在文件里（`旧串在文件里出现 0 次`），这是预期的，不代表规格错。
- 写范围闸预检：三个路径各喂一次 `write-guard.sh`，退出码都是 0（`precheck.jsonl` 在草稿目录）。
- 英、日页里新出现的已登记编号（C383、C289、C615、C768、D23、D26、E156 等）一律照规格检查的要求写成全角括号加中文登记简称；术语表写的是半角括号加译名，两者冲突时我取了能让规格检查绿的写法。原有的半角写法没动，新旧提及数量对得上的地方保持半角。

## 改了哪几处（行号是写完之后的；三份同一行号，除非另写）

| 行（编号） | 改前一句 | 改后一句 |
|---|---|---|
| 313 C445 | 「这个缺口已经在三条账里各自出现过一次：C344……」 | 改「曾在」，C344 后加「2026-10-20 已并进 C383，原文在 git 历史」 |
| 132 C159 | 「记账轴那一条随 C693 ③ 重推」 | 「已随 C693 ③ 定案：维持搬运（用户 2026-10-20 定）」 |
| 402 C618 | `FIRST_INODE_NUMBER` 51 行（2026-10-19 现数；英、日写 31 行无日期） | 61 行（2026-10-21 现数，命令接 `\| wc -l`） |
| 314 C446 | 在用 115 条、池级 checker 判 92 条、其余 23 条、「多数没写明它被什么挡着」、I-8.4 写了实现在途 | 在用 126、判 103、其余 23 条里 I-8.4 已实现在崩溃注入的二次崩溃上、22 条「未实现」且都带「前置：」（见下「要人看」第 1 条） |
| 115 C136（英、日） | 115 条在用（两处） | 126（两处） |
| 321、360、380、383、432、458、476、502 | C463、C555、C593、C596、C663、C707、C728、C754 前置格末尾的去向句 | 其后补一句「销账条件见里程碑四出口 9」，正文一字未动 |
| 499 C751 | 「D23 第 396 行」；「仍欠：条文随 kb 写回批写进 D23 已定项 14 的 F2 段」 | 第 398 行；「条文已写进 D23 已定项 14 的 F2 段（……用户 2026-10-20 定案）；仍欠：实现批钉……」 |
| 67、102、220、221、241（中文页）/ 英、日同行号 | 英、日页去向句停在「归统计量标签」「归挂载内实例切换」「归性能」旧写法 | 照中文原文翻成具体落点：10-tenth 三·1、14-fourteenth 一·1、06-sixth 一·10、04-fourth 二·2、09-ninth 二·4（英、日共 18 行：C71、C117、C286、C287、C323、C376、C459、C460、C486）。顺手把英、日页里被译坏的 `records/2026-09-28-milestone三第十项清单.md` 改回真路径 `records/2026-09-28-里程碑三第十项清单.md` |
| 99 C113 | 「仍欠：`crates/` 里没有扫描重建的读侧（`grep` 零命中）……不在第 50 批里」 | 读侧第一版已在池级 checker（`rebuild.rs`，只读、三张派生表）；还欠条款没定的格、分配记录树等的重建、写回 |
| 536（英、日 540→536） C821 | 题下「内核不置位……对拍不比……看不见」，前置格「—」 | 要拦什么格改成「曾」，前置格写「已做：内核 `scan_rebuild_main`、对拍用例、24 行变异都已在提交 0b926bd3f；重型复跑归提交时，绿了随里程碑四出口 9 一起销账」；留在开着的表，没挪 |
| 118 C141 | 「换成产出谓词……终止时的产出谓词取值相同」 | 「换成本地谓词：停机是删意图那一刻 `R` 按记录全空……全局差值只决定签不签下一条」，检查形态相应改 |
| 191 C243 | 前置格「`git grep scrub\|migrat` 零命中（2026-10-11 现查）……实现侧自证没有被测对象」；出处格「等整理进第一版」 | 整理已进 core 代码（`publish_relocation_batch`、`run_compaction`），自证有了被测对象，必红用例表那一行还没做；出处格改「整理已进 core 代码、这条用例还没做」 |
| 267 C364 | 「仍欠实现（`logged_operations.rs` 在第 2 批补丁里）与 I-4.4 的判法」 | 实现已入库，I-4.4 字段白名单判法已实现；门禁登记位逐字段比对是否已有，没核 |
| 272 C373 | 「`crates/` 里还没有整理代码」 | 整理代码与整理崩溃流已入库，这条断言按名字搜不到，是否已有待读流文件核 |
| 271 C372 | 「仍欠代码 `empty_segments` 从占用口径改成记录口径、checker 的 C413 那一半」 | `empty_segments` 已改记录口径，I-3.16 已实现；销账前核「怎么拦」格三处给同一个判定 |
| 689（英、日 683） C41 | 「（2026-10-15 实现，待提交落地销账）」 | 「C61 已还清」 |
| 365 C566 | 依据只写到第 5、6 次重跑 | 补第 7 次重跑（2026-10-18，洞数扫到环满一圈再跨圈，Q1a 峰值读法甲、乙都判仍在涨，没量到上界） |
| 364 C564 | 已登记 14 条崩溃枚举用例 | 35 条 |
| 203 C257 | 「仍按三级回退解析，第一级是行内最近的那个编号」 | 六级逐级回退（括号里列全），没有紧挨着的指名时先落到行内最近的编号 |
| 389 C602 | 积压那一批已由提交 31128273 归档 | 其后补「2026-10-21 又积压 471 份……`--apply --keep-registered` 归档，22 份文件里 90 处引用只留文件名」 |
| 英、日 C576、C296、C587、C668、C827 | 还在开着的六列表里（行 374、228、377、439、546 一带） | 删掉，在已还清表照中文四列行翻译重插（853–856、861 一带），顺序与中文一致 |

## 剔出的行（依据只在工作区或暂存区、不在 HEAD）

行清单第 19–21 行（F17：C656，`checks-owed.md:427`、`checks-owed.en.md:430`、`checks-owed.ja.md:430`）三行一字未改。判法与原样输出（`facts.out`）：
- `git show HEAD:crates/singlefs-core/src/make_filesystem.rs | grep -c make_filesystem_drawing_the_filesystem_identifier` 得 `0`（暂存区的 diff 才加这个入口与作废屏障）。
- `git show HEAD:crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs | grep -n '2+12\|14+1+1+1+4'` 只得 `280:        "14+1+1+1+4",`，HEAD 里没有 `2+12+1+1+1+4`。
- `git ls-tree HEAD --name-only crates/singlefs-checker-tier/tests/ | grep -c crash_enumeration_make_filesystem_over_a_used_pool_stream` 得 `0`。
- 同样的道理，行清单第 67–69 行（F41）不是剔出而是被第 62 行（F37）的改法顶掉，见下。

## 没改或只改了一半的行（原因）

1. **第 15–18、23、25、27 行（C555、C663、C754 及同批八笔）**：用户已定写进里程碑四出口 9，正文不改，只补了去向句（见上表）。「待合入后验证」是否成立、C663 去向格里还留着「先开一轮三方定」与「条款已定」并存，都交主 agent。
2. **第 67–69 行（F41：C364）与第 62–64 行（F37：C364）改法打架**：F41 要写成「仍欠 I-4.4 的判法（字段白名单）与会红的检查」，F37 要写成「实现已做、I-4.4 白名单判法已实现」。现查 HEAD 的 `crates/singlefs-checker/invariants/06-事务与崩溃一致性.md` 里 I-4.4 状态格是「已实现（2026-10-15 起判 logged_ops 里的条目……白名单……必红世界在 `checker_compaction_intent_and_relocation_must_red_worlds.rs`）」，所以按 F37 写，F41 没采。C364 的题（简称）没改（规则 5：改题归后一步）；该行「现象」格里「四样的宽度与编码全仓一处都没有」同属过时现状句，按规则 5 只改「字段表未定 / 没有写者」那类句子，这一句没动，留给改题那一步。
3. **「要人看」不改项（规则 7）**：
   - C88（`checks-owed.md:80`，三语同行）：「择新平局按它高者赢」与 D16、D22 已定项 7 的落点公式核（新条文「平局只剩第 0 代那几份同字节的根」）是否矛盾，没改（第 2 段报告「候选没罩到」第 2 条）。
4. **同类行但没改的**（旁发现里的）：
   - C116（`checks-owed.md:101`）：「证据在没提交的改动里（`crash_enumeration_unlink_merging_inode_leaf_containers_stream.rs`……），这一批提交之后销账」。现查 `git ls-tree HEAD --name-only crates/singlefs-checker-tier/tests/ | grep unlink_merging_inode_leaf_containers` 有这个文件，「没提交」已过时；但是销账还是改写没有给定句子，也属判断，没改。
   - C768（`checks-owed.md:514`）：「`crates/` 里没有消费者」与 `rebuild.rs` 模块文档「退役记录……按退役树认领」两说并存，要读代码定哪边，没改。
   - C754（`checks-owed.md:502`）正文里 `D23 第 392 行`（现为 394 行）、C755（`checks-owed.en.md:507` 的 `line 396`）、C757（三语 `第 396 行`）同样是三语页眉多出两行带来的行号偏移。C754 属规则 4 的八笔不动正文；C755、C757 不在行清单里，没扩范围。C751 行里那一处按行清单第 26 行顺带改了。
   - C566、C602 的英、日页前置格比中文落后（中文已写「去向：2026-10-19 挪出里程碑三，归里程碑九 / 里程碑四八·6」，英、日还是旧去向句）：不在行清单里，没改。
5. **C821 题（简称）没改**：「GPU 判器没有扫描重建内核」已不真，但简称是登记位，改它要全仓同步，与 C364 同理，交后一步。里程碑四出口 9 的文字只点名八笔，没有 C821；用户让 C821「绿了随出口 9 一起销账」，需要出口 9 那一行补上 C821，或主 agent 另定销账位置。

## 要主 agent 判的点

1. **C446 整笔可能已还清**：现查状态列以「未实现」开头的行 22 行，22 行都带「前置：」（`facts.out`，第 2 段判决说 20 条，是只数了「未实现，前置：」的写法，漏了「未实现（条款已写、检查仍欠），前置：」两行 I-1.5、I-2.2）；HEAD 的 `doc-registries.sh` 里已有 `invariant-prerequisites` 格（`git show HEAD:.claude/gate.d/doc-registries.sh | grep -c invariant-prerequisites` 得 19，弄坏开关名里直接写着「C446 判别力自证」）。我只改了数与「多数没写前置」那半句，没挪进已还清表。
2. **里程碑四出口 9 那一行只在工作区**：`git show HEAD:.claude/kb/milestone/04-fourth-txn.md | grep -c '^| 9 | 接手里程碑三'` 得 0，`git show :...` 也是 0，工作区得 1（`04-fourth-txn.md` 是 `MM`，别的写回员在改）。我补的八笔去向句与 C821 句都指向它，别的写回员没落地的话这些指针悬空。
3. **C821 留在开着的表**：按规则 3 没移；复跑（GPU 对拍、变异整表）我没跑，是重型。
4. 英、日页缺 C860、C861、C862 三行（中文页有）：已有欠账，不是这一批造成的，没补；结构四格里「表格行数」会红在这里。

## 写之前现查的事实（命令与原样输出在 `facts.out`；口径：HEAD 加这一轮已暂存的门禁与脚本改动）

- `git grep -n FIRST_INODE_NUMBER HEAD -- crates/singlefs-core/src | wc -l` → `61`。
- HEAD 的 `invariants/README.md` 第 36 行 `现共 126 条在用`；HEAD 的 `image.rs` 第 149 行 `IMPLEMENTED_INVARIANTS: [&str; 103]`；状态列首词计数 已实现 103、未实现 22、部分实现 1、不适用 6；22 行「未实现」全带「前置：」。
- `git show :.claude/gate.d/stage-inputs.tsv | grep -c '^crash-case:'` → `35`（暂存区；HEAD 是 34，这一轮暂存的门禁改动加了一条）。
- `git show :.claude/gate.d/lib-item-ref-status.py | grep -n 归属顺序` → 第 39 行，六级。
- HEAD：`crash_judge_conclusions.rs:438` `LEFT_UNSET_BY_THE_GPU_KERNELS: &[usize] = &[]`；`crates/mutations.tsv` 以 C821 开头 24 行；`crash_judge_checker_scan_rebuild.wgsl` 与对拍测试都在 HEAD。
- HEAD：`allocator.rs:846-847` `empty_segments` 返回 `full_segments_without_an_unreleased_record`；`git grep -n -i compaction HEAD -- 'crates/*/src/**' | wc -l` → `1415`；`publish_relocation_batch`（transaction.rs:9125）、`run_compaction`（mounted_session.rs:1938）、`logged_operations.rs`、`logged_operations_root_entries` 调用（transaction.rs:12338）、`walk.rs` 的 `judge_compaction_intents_of_the_versions`、`judge_fragmentation_and_empty_segment_statistics`、`crates/singlefs-checker/src/rebuild.rs`（提交 0b926bd3f）都在。
- D23 `实例表指针相同` 在 398 行（F2 段）、458 行（依据）。
- 里程碑 02、03 页的落点行：`10-tenth-txn.md` 三·1、`14-fourteenth-txn.md` 一·1、`04-fourth-txn.md` 二·2、`06-sixth-txn.md` 一·10、`09-ninth-txn.md` 二·4，HEAD 里各恰好 1 行。
- `records/2026-10-21-doc门禁违规修复.md` 第 99 行记着 471 份与 22 份文件 90 处引用；`grep -c 归档` 在 `commit-stages.md`、`staged-gate.md` 各 0。

## 回读

- 写完之后：1）每次替换的新串都在文件里、旧串不在；2）10 行插入的新行各出现 1 次；3）开着的表与已还清表的编号集合，中文与英、日现在只差 C860、C861、C862（英、日本来就缺）；4）已还清表的行序三语一致（`md5sum` 相同）；5）那 5 行在英、日已还清表里是 4 列，开着的表里 6 列的行列数没变；6）行数不变：中文 867、英文 861、日文 861。
- `git diff -- <三份>`（工作区对暂存区）：中文 50 行、英文 80 行、日文 80 行的变动，都是上面这些。

## 没做什么

- 没判定案对不对，没写清单外的句子；没改题（C364、C821 的简称）；没跑 `doc-decisions.sh --write`（并行写回，归主 agent）；没跑门禁阶段与任何 lint；没跑重型复跑（C821 的对拍与变异）；没提交；没读被禁读的文件。
- 没去核「待合入后验证」八笔是否已验，没改八笔正文。
- 没把 C446、C116 销账，没动 C88、C768、C754 正文。
- 没补英、日页的 C860–C862 与 C566、C602 的旧去向句。

## 哈希（三份欠账表）

| 文件 | 开工时 sha256 | 收尾时 sha256 |
|---|---|---|
| `.claude/kb/checks-owed.md` | `772388ccf32201cb96eade3e4188b06f3e1f51a2d25b91a1b3894192b40c6e11` | `a1bc67cc6780e8fee4f12815403ef8042c3eb64c4712842349ca418f494084a3` |
| `.claude/kb/checks-owed.en.md` | `49a520a273892be06c2684566e63a5df9159a1bb5e74c8953f45213c72bbd7e3` | `7ef97151cbd74d9bde249ba9a8bea9dd2121708e1c22359809195c74cc58a877` |
| `.claude/kb/checks-owed.ja.md` | `623043072862436e5957c5cf9e997042f2e3caa0152af085baf8228907e7257f` | `7af6cebae1344ea9ce7a81bfbf8f7b31729096adaf97cb4106ecb4d4d7c7411e` |

`decisions.md`、`decisions-history.md` 本任务不写，没记哈希。`replacements.tsv` sha256 `4cc5cdb585e8a77e24377006ab0def7ae78673fac84e713022df77616b1ff0dc`，121 行。
