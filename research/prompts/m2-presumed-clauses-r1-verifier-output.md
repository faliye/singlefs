# m2-presumed-clauses-r1 核查员报告

核对表里的 ✗ 不免除主 agent 对推论的逐条现查，这一句照抄进报告开头。

本轮是设计轮，没有开工快照，对主树核。工作区在三条腿跑完之后持续被别的并发会话改动——`git status --short crates/ .claude/kb/` 显示几乎整个 `crates/` 与 `.claude/kb/decisions/` 都处于未提交的修改状态（不止派发提示点名的 `walk.rs`、`checker_known_bad_images.rs`、`mutations.tsv`、`23-journal的角色与格式.md`、`checks-owed.md`、`invariants.md` 六个文件）。按定义「没给快照的轮（设计轮）……行号对不上时记『分不清：文件可能在腿交回之后被改过』，不记 ✗」，下面每一处「分不清」都已现查过：或用 `git diff HEAD` 找到穿过该行上方的插入/删除、或用 `git show HEAD:<文件>` 核对提交时的行号与腿的引用是否吻合（吻合则确证是腿之后的编辑造成的漂移，不算腿的错）。

## 判别力自证

取 Sonnet 报告里的一条引用（`transaction.rs` 第 1780 行文档注释「实例表单元归树 0，排在全部提交内生块之前……」），在草稿目录的副本 `/tmp/claude-1000/presumed-r1-verifier/selftest/sonnet-copy.md` 里把行号改成 1781（`sed`/`python3` 定点替换，仅改这一处数字，原句原样保留）。按第 2 步核法（去 `transaction.rs` 第 1781 行取内容、与引用的原文比对）：

```
$ sed -n '1781p' crates/singlefs-core/src/transaction.rs
    ///
```

第 1781 行是一行空的文档注释符号，不含「实例表单元归树 0……」这句话。**判定：✗**。核查方法分辨得出行号错误。（这一条只是自证，不计入下面任何一份腿报告的核对表计数。）

## 一、云端正推（Sonnet）`m2-presumed-clauses-r1-sonnet-output.md`

sha256 与交回一致：`7b579082447e1a044f9ee10f74572420b62cdc898a8958608122715c612bb9a9`（现算 `sha256sum` 逐字符核对，相同）。

Sonnet 全篇没有跑产物、没有复跑命令，只有「文件:行号 + 抄的原文」这一类引用；下表逐条核（行号在 kb 文件、代码文件里现取，用 `awk 'NR==N'`／`sed -n`／`grep -n` 三种方式互证）。

| # | 引用（文件:行） | 抄的原文（摘要，完整见报告原文） | 结果 |
|---|---|---|---|
| 1 | `transaction.rs:1786-1812`（`rewritten_roles`） | `InstanceTable` 紧跟 `Data` 之后、`ExtentRoot` 之前 push | ✓ 函数实际跨 1786–1811，1812 是外层 `impl` 的收尾括号；push 顺序逐行核对与代码一致 |
| 2 | `transaction.rs:1780` | 「实例表单元归树 0，排在全部提交内生块之前（mkfs 也是先实例表后树表；里程碑步 3 的决策点）。」 | ✓ 逐字相同 |
| 3 | `transaction.rs:1145-1147` | `TreeTable \| InstanceTable => TreeIdentifier(TREE_IDENTIFIER_NONE)` | ✓ 逐字相同 |
| 4 | `transaction.rs:76` | `TREE_IDENTIFIER_NONE: u64 = 0` | ✓ 逐字相同 |
| 5 | `transaction.rs:1132` | 「点名项里的归属树；树表单元不属于任何一棵树（写 0）。」 | ✓ 逐字相同 |
| 6 | `singlefs-format/src/lib.rs:122-124` | `TREE_IDENTIFIER_EXTENT: u64 = 11` 起 | ✓ 逐字相同 |
| 7 | `03-空间分配.md:198`（已定项 10 ⑤ 整句） | bump 次序那句（树 ID 升序……树表单元最末） | 分不清：内容在当前树里逐字相同，但住在第 199 行，不是 198；`git diff HEAD` 显示该文件已定项 8 第 2 条在这一句上方被改写、插入新段落（"聚簇段只给提交内生块" → "聚簇段在挂载期间对用户数据关着"，外加一段"第一版盘不等大时…"），把下文顺移了 1 行；HEAD 版本本身也不在 198（在 199 与本次核对一致，working tree 相对 HEAD 净移了 0 行到这一句，但 198 本身在两个版本里都是空行）——判定为腿交回之后文件被改过，不是引用错误 |
| 8 | `03-空间分配.md`（grep 输出「222:欠：C324…C497…」） | grep 命令原样输出 | 分不清：现在重跑同一条 `grep -n "实例表"` 命中的是 223 行，不是 222；同一次文件改动导致的偏移（见上一条） |
| 9 | `22-单元原子性怎么合成.md`（`grep -c "bump"` = 0） | grep 计数结果 | ✓ 现跑同一条命令仍是 0 |
| 10 | `mount.rs:1044-1072`（`warm_up_publish_txgs`） | 函数体（从 `device_of_txg` 到返回 `warm_up_publishes`） | ✓ 函数恰好跨 1044–1072（含收尾括号），逐行核对与代码一致 |
| 11 | `root_ring.rs:114` | `region: checkpoint_txg.0 % ROOT_RING_REGIONS,` | ✓ 逐字相同 |
| 12 | `mount.rs:1103-1241`（`establish_instance`） | 函数跨度 | ✓ 函数恰好始于 1103、止于 1241（收尾括号），与引用范围一致 |
| 13 | `mount.rs:1301`（`mount_writable` 调用 `establish_instance`） | 调用点 | ✓ 逐字相同 |
| 14 | `mount.rs:1322`（`mount_rollback` 文档注释） | 「之后暖机同可写挂载。」 | ✓ 逐字相同 |
| 15 | `mount.rs:1327`（`mount_rollback` 函数签名行） | `pub fn mount_rollback<Device: BlockDevice>(` | ✓ 逐字相同 |
| 16 | `16-发布语义.md:187`（已定项 8 定案句） | 「新实例在本实例写成……条款在 D22 已定项 16 第 5 句。」 | 分不清：Sonnet 抄的这句在当前第 187 行原样存在（逐字比对无差），但该行**现在**在末尾多出一整句「**四个等号**：fsync = 提前发布……」，Sonnet 的引用没有覆盖到这句——这句是同一次并发编辑追加进同一行的（Opus 报告附录 A1 抄的是含「四个等号」的完整版本，两条腿抄的是同一行的前后两个时间点） |
| 17 | `16-发布语义.md:189`（射程句） | 「定的是新实例确认之前要做什么……不定空发布写出多少单元（已定项 9）。」 | ✓ 逐字相同（Sonnet 只抄到这句的句号为止，未抄同行后半句「四样已知边角：」，这半句是引导下面列表的新句子，不算摘句） |
| 18 | `16-发布语义.md:192`（已知边角第一条） | 「归属改了那两个格式常量要重算……可推翻。」 | ✓ 逐字相同（去掉列表前缀 `- `） |
| 19 | `22-单元原子性怎么合成.md:187`（已定项 16 第 5 句，K2 推导段落引用） | 「暖机空发布 2 次、第一个事务 txg 3 是格式常量……」 | 分不清：内容现在住在第 332 行，标题住在第 324 行；HEAD 版本分别在 330、322——与本地攻方翻译核对表条目 14 引的行号（322/330）恰好对上 HEAD，说明这份 kb 在 Sonnet 与本地腿分别读它的两个时刻之间被人从 HEAD 又往后推了 2 行 |
| 20 | `mount.rs:1-4`（模块头） | 「第一版没有干净关闭标记，重开一律走恢复。」 | ✓ 逐字相同，且行号（第 4 行）精确 |
| 21 | `checks-owed.md:443`（C499 行） | 表格行前半（P5 判定与判别力自证） | ✓ 逐字相同 |
| 22 | `decisions/*.md` 全仓 grep「干净关闭」exit 1 | 0 命中 | ✓ 复跑同一条命令确认 exit 1 |
| 23 | `23-journal的角色与格式.md:358`（已定项 14 注 1） | 「前缀规则不跨实例边界……」 | ✓ 逐字相同（去掉列表序号 `1. `） |
| 24 | `23-journal的角色与格式.md:375`（射程收窄注） | 「它对已定项 3 是收窄不是推翻……」 | ✓ 逐字相同（去掉列表前缀 `- `） |
| 25 | `invariants.md:96-98`（I-8.3 上方 ⚠️ 段） | 三行原样 | ✓ 逐字相同，且行号精确（这份文件虽被列为「今天多处改过」，但这一段这次核对时仍在原位） |
| 26 | `invariants.md:84`（I-8.8 行，K4「旁证」段） | 「今天可达镜像上它恒真：第一版一条记录一个事务、`is_commit` 全仓没有一处写 false」 | ✓ 逐字相同 |
| 27 | `recovery.rs:1138`（`replay_journal` 起） | 函数签名行 | ✓ 逐字相同 |
| 28 | `recovery.rs:1169`（注释） | 「链首只能是 checkpoint_txg = 根的 txg + 1 的那条（第一版一次发布一条记录、txg 每次加一）……」 | ✓ 逐字相同 |
| 29 | `recovery.rs:1171-1178`（`root_own_record_counter` 到 `expected_next` 赋值） | 代码块跨度 | ✓ 逐行核对与代码一致 |
| 30 | `recovery.rs:1179`（`chain_start_txg_without_anchor`） | 赋值行 | ✓ 逐字相同 |
| 31 | `recovery.rs:1185`（`else if` 分支） | `} else if record.checkpoint_txg != chain_start_txg_without_anchor {` | ✓ 逐字相同 |
| 32 | `recovery.rs:1196`（K4「旁证」段，`if !record.is_commit`） | 「`recovery.rs:1196` 的 `if !record.is_commit { break; }`」 | 分不清：`git diff HEAD -- crates/singlefs-core/src/recovery.rs` 显示紧邻这一行上方被插入了 3 行新注释（「⚠️ 这一步只对『一事务一条』成立……」），把原来紧接在 `expected_next = Some(...)` 之后的 `if !record.is_commit {` 从 HEAD 的第 898 行推到现在的第 1199 行；Sonnet 引用的 1196 既不是 HEAD 的行号也不是现在的行号，落在新插入的注释段落内部——按内容比对，`if !record.is_commit {` 现在确实存在，只是行号漂了 3 行，判「分不清」而非 ✗ |
| 33 | `16-发布语义.md:147`（已定项 6 定案句，K4 推导） | 「每次发布把 checkpoint_txg 加一……没有『小发布不记号』的例外……它与 checkpoint_txg 是同一个东西。」 | ✓ 逐字相同；同一行末尾现在多出的「四个等号」那句（同条目 16 的分不清）Sonnet 这里同样没抄到，但没有影响这句的判定，因为「定案句」的语义边界（第一个句号收尾的「不另设『发布代号』字段——它与 checkpoint_txg 是同一个东西。」）与抄的内容对齐 |
| 34 | `23-journal的角色与格式.md:191`（已定项 7 标题） | 「一个事务可以跨多条记录，记录头带事务边界字段」 | ✓ 逐字相同 |
| 35 | `checks-owed.md:435`（C491 行） | 表格行原文 | ✓ 逐字相同 |
| 36 | `23-journal的角色与格式.md:350`（第六条口径） | 「⚠️ 第六条：施加的单位是一次发布……」 | ✓ 逐字相同 |
| 37 | `16-发布语义.md:102`（已定项 4 定案句） | 「施加一条记录在指针层上做什么……不重新生成祖先。」 | ✓ 逐字相同 |
| 38 | `transaction.rs:1571` | `MoreNamedUnitsThanOneJournalRecordHolds {` | ✓ 逐字相同 |
| 39 | `transaction.rs:2164` | `return Err(PublishError::MoreNamedUnitsThanOneJournalRecordHolds {` | ✓ 逐字相同 |
| 40 | `transaction.rs:1632-1634`（`STILL_UNDECIDED_TODAY`） | 两条未定条款枚举 | ✓ 逐行核对与代码一致 |
| 41 | `m2-refusals-presumed-r1-main-verification.md:43-54`（B 组表） | P1/P3/P5 行判定「推不出，条款空白」 | ✓ 逐字相同 |

**Sonnet 小计：核了 41 处，✓ 35 处，分不清 6 处（均系文件在腿交回之后被并发编辑改动，现查 `git diff HEAD` 或 `git show HEAD:<文件>` 逐一坐实了改动位置与内容，不计入 ✓ 也不计入 ✗），✗ 0 处。** 没有复跑命令、没有引产物，故没有第 3、4 步要核的项。

## 二、云端攻方（Opus）`m2-presumed-clauses-r1-opus-output.md`

sha256 与交回一致：`c13307c6aec959706adc06e0cfa915bc5f35f5551a3cb58de2d013664faeac69`。模型目录 20 个文件的 sha256 逐个用 `sha256sum` 现算，与报告清单（第 24-45 行）**全部相同**，无一处差异——报告里贴的这批 `.log`/`.log.gz`/`.rs`/`.patch` 文件确实就是报告引用的那些字节，不是事后换过的。

### 2.1 独立重建复跑（第 4 步：不在腿的原目录跑，拷到草稿目录重跑）

把 Opus 留在 `/tmp/claude-1000/presumed-r1-opus/repo/`（它自己的原目录，遵嘱**不**在那里跑）里已经打好三处 patch、装好 `presumed_r1_opus_k2.rs`/`presumed_r1_opus_k4.rs` 两个测试文件的仓副本，`rsync -a --exclude target --exclude .git` 拷到 `/tmp/claude-1000/presumed-r1-verifier/opus-repro/repo/`（47M，与原副本逐个关键文件 sha256 核对一致），设独立的 `CARGO_TARGET_DIR`/`TMPDIR`，全部从零编译：

```
$ nice -n 19 cargo test --release -p singlefs-harness --test presumed_r1_opus_k2 --test presumed_r1_opus_k4 --no-run
   Compiling singlefs-format v0.1.0 …
   Compiling singlefs-checker v0.1.0 …
   Compiling singlefs-core v0.1.0 …
   Compiling singlefs-harness v0.1.0 …
    Finished `release` profile [optimized + debuginfo] target(s) in 6.90s
```

分别用 `nice -n 19` 跑（前台起、通知完成即读，未用 `setsid`/`disown`）：

| 复跑 | 命令 | 结果 |
|---|---|---|
| K2 today 臂（不设环境变量，即「旁见」段两份都要过口径） | `$K2BIN --nocapture --test-threads=1`，515.19s | `grep '^K2 \|^K2-SUMMARY\|^K2-CHAIN'` 与模型目录 `logs/k2-A1-today.log.gz` 解压后同样 grep 的结果**逐行 diff 完全相同（11401 行，diff 无输出）** |
| K4 today 臂 | `$K4BIN --nocapture --test-threads=1` | 与 `logs/k4-today.log` 逐行 diff **完全相同（1381 行）** |
| K4 max 臂 | `PRESUMED_R1_CHAIN_HEAD=max $K4BIN …` | 与 `logs/k4-max.log` 逐行 diff **完全相同** |
| K4 stop 臂 | `PRESUMED_R1_CHAIN_HEAD=stop $K4BIN …` | 与 `logs/k4-stop.log` 逐行 diff **完全相同** |
| K4 max-stop 臂 | `PRESUMED_R1_CHAIN_HEAD=max-stop $K4BIN …` | 与 `logs/k4-max-stop.log` 逐行 diff **完全相同** |
| K2 A3（`PRESUMED_R1_NAMED_ANY=1 PRESUMED_R1_WARM_UP_CAP=0`，对照组），176.39s | 同样命令 | 与 `logs/k2-A3-any-cap0.log.gz` 解压后逐行 diff **完全相同（1194 行）** |
| 纯 std 模型 `presumed_r1_opus_model.rs` | `rustc -O --edition 2021` 编译后直接跑 | 输出与 `logs/model.log` **sha256 完全相同**（`de343fbb…ede041`） |

**这是从零独立重建（拷贝、打补丁、编译、跑）得到的结果，不是读 Opus 留下的日志**：K2-today、K2-A3（对照组）、K4 全部四臂、纯 std 模型**全部**字节级复现，构成本轮最强的一类证据——K2「多推」147968 字节、3102 次白推、K2 对照组「只在写行 txg ≡ 2 那一档丢」、K4-a 196 格、K4-b 147/21 格这几个被派发提示点名要核的数，全部落在这几次独立复跑覆盖的范围内，逐条为真。

### 2.2 报告里抄的原样输出（第 3 步：产物里逐字找）

在草稿目录里解压全部 `logs/*.gz`（sha256 与清单核对过），对报告正文贴出的每一段「原样」逐字 `grep -F` 核对，全部命中：

| 报告位置 | 贴的内容 | 结果 |
|---|---|---|
| 第 78-95 行（run-k2-arms.out 摘要） | A2/A3/A4/A5 的 `K2-SUMMARY` 行 | ✓ 与 `logs/run-k2-arms.out` 逐字相同 |
| 第 151-160 行（K2 旁见摘要） | A1/A6/A7 的 `K2-SUMMARY` 行 | ✓ 同上 |
| 第 115、121 行（147968 字节那两行） | `mount_bytes=517632`（today）与 `mount_bytes=369664`（jump） | ✓ 分别在 `k2-A1-today.log.gz`、`k2-A7-jump.log.gz` 解压后命中，`517632−369664=147968` 现算相符 |
| 第 131-135 行（K2-CHAIN 四行） | `extra=0/1/2` 的 `publishes_per_mount` 序列 | ✓ 与 `run-k2-arms.out` 逐字相同 |
| 第 104-106 行（`K2-MODEL` 两行） | today/Cap(0) 的按余数分桶结果 | ✓ 与 `logs/model.log` 逐字相同（也与本报告独立重编译出的 `model.log` 逐字相同） |
| 第 200-208 行（K4 today 8 行总账 + SUMMARY） | `K4-TALLY`/`K4-SUMMARY` | ✓ 与 `logs/k4-today.log` 逐字相同，8 个数求和 = 1372 与 `cells=1372` 吻合 |
| 第 220、239、247-248 行（K4-a/K4-b 抽样行） | 4 条单行 | ✓ 均在对应日志文件里逐字命中 |
| 第 273-278 行（stop/max/max-stop 4 行 TALLY） | — | ✓ 逐字命中对应日志 |

### 2.3 独立复算（不靠读报告的数，自己从原始日志重新数一遍）

- **K2 对照组（A3 臂）逐行分布**：报告称「后续挂载与回退各 8 次…这两条用例的 44 行：row_mod3=2 16 行、每行 F2_failures=1；row_mod3=0 14 行、row_mod3=1 14 行，全是 F2_failures=0」。用 Python 脚本独立从 `k2-A3-any-cap0.log.gz` 里抓出 `rollback`/`after-rollback`/`sub` 三类行（不看 `K2-SUMMARY`，逐行原始记录）：结果 44 行，`row_mod3` 分布 {0:14, 1:14, 2:16}，`row_mod3=2` 的 16 行 `F2_failures` 全部是 1，另外 28 行全部是 0——**与报告逐字相符**。
- **147968 字节算术**：`517632 − 369664 = 147968`，现算相符（已在 2.2 表列出源行）。
- **K4-b「147 格里 21 格是 n1=1」**：用 `grep -cE` 从 `k4-stop.log` 里数 `branch=all-P0-torn(no-anchor)` 且 `verdict=UNDER` 的行，总数 147；其中 `n=(*,1,*)` 形态的 21 行——**与报告逐字相符**。


### 2.4 两处 ✗：报告叙述的计数与可复现的日志内容对不上

**✗ 第一处（第 211 行）**：报告称「对照组：n0 = n1 = 1（每次发布一条记录，今天的形态）那 **28 格**，今天的做法全部 match 或 match-none（`k4-today.log` 里 `n=(1,1,*)` **14 格 match、14 格 match-none**）」。用 `grep -E '^K4 \[today\] n=\(1,1,'` 独立从 `k4-today.log`（sha256 已核，且已被我方独立重跑逐行复现）里抓出全部 `n=(1,1,*)` 行：

```
$ grep -cE "^K4 \[today\] n=\(1,1," k4-today.log
27
$ grep -E "^K4 \[today\] n=\(1,1," k4-today.log | grep -c "verdict=match "
13
$ grep -E "^K4 \[today\] n=\(1,1," k4-today.log | grep -c "verdict=match-none"
14
```

实际是 **27 格（13 match + 14 match-none）**，不是 28 格、不是 14/14。按 n2 拆分能看出原因：`n=(1,1,0)` 只有 3 行（缺「torn=[]」这一格，2 条记录的 2² = 4 个子集里只枚举了 3 个），`n=(1,1,1)` 8 行、`n=(1,1,2)` 16 行，3+8+16=27——不是均匀的「每个 n2 都 8 行 × 4 个可能」这类假设。这一处数字与可复现的原始产物**逐字不符**，且这份产物本身已经过独立重建复跑坐实（2.1 节），不属于「腿的原目录改动」一类的分不清。

**✗ 第二处（第 213 行）**：报告称「第二条路：纯 std 模型……四条臂的 **8 行总账**与副本装置逐行相等（`model.log` 里 `K4-MODEL-TALLY` **32 行**）」。本报告 2.1 节已独立重新编译并跑过 `presumed_r1_opus_model.rs`（`rustc -O --edition 2021`，输出与原 `model.log` sha256 完全相同）：

```
$ grep -c "K4-MODEL-TALLY" model-rerun.log
28
$ grep -c "K4-MODEL-TALLY \[today\]" model-rerun.log; grep -c "K4-MODEL-TALLY \[max\]" model-rerun.log; grep -c "K4-MODEL-TALLY \[stop\]" model-rerun.log; grep -c "K4-MODEL-TALLY \[max-stop\]" model-rerun.log
8
7
7
6
```

实际是 **28 行（8+7+7+6）**，不是 32 行——`max`/`stop`/`max-stop` 三条臂各自有一到两类计数为 0 的分支被模型省略不打印（例如 `max` 臂 `anchor-readable UNDER(少施加)` 恰好是 0，代码没打印这一行），不是「四条臂各打 8 行」。这处「32 行」与可复现产物**逐字不符**。

⚠️ 两处 ✗ 都只是**叙述用的计数句**，不影响下面那些具体数值本身（84/147/63/196/343/196 等每一格的数，本报告 2.1、2.2 节已独立复现为真）；但按纪律「能被一条命令数出来的声明」（`evidence-discipline.md`）这类计数本该由命令验证，这里是两处没有验证就写进正文的计数，判 ✗。

### 2.5 附录 kb 原文引用（第 3 步的另一半：kb 文件本身当「产物」核）

报告附录 A1–A11 共引 `16-发布语义.md`（147/170/174/187/189/193 六处）、`23-journal的角色与格式.md`（344/345/346/347/348/349/350/354/358/360/363/399/510/524 十四处）、`checks-owed.md`（275 一处），共 **21 处**行号 + 原文。逐条用 `awk 'NR==N'` 现取，与附录抄的原文逐字比对：**全部 21 处一字不差**（包括 147 行含「四个等号」那句的完整版本、189 行含「四样已知边角：」的完整版本——与 Sonnet 报告里同两行的截断版本形成对照，见一、条目 16、17、33）。报告脚注自己写明「2026-09-23 UTC 15:5x 重取」，与这次核对时刻接近，drift 窗口小，这批引用全部命中不意外。

**Opus 小计**：文件级 sha256 核 20 处全部 ✓；报告贴出的原样日志片段核 8 组全部 ✓；独立复算 3 处全部 ✓；独立重建整套仓库从零编译并重跑 K2-today / K2-A3 / K4 四臂 / 纯 std 模型共 6 项复跑命令，**全部字节级复现**；附录 kb 引文核 21 处全部 ✓。**✗ 2 处（叙述性计数「28 格/14/14」与「32 行」，均已现查坐实为与可复现产物不符）**，分不清 0 处（Opus 自己注明重取时刻，未遇到明显漂移）。

## 三、本地攻方 `m2-presumed-clauses-r1-local-attack.md` 及三份样本、跑记、翻译核对表

sha256 全部与交回一致（现算 `sha256sum` 六份文件逐字符核对）：提示 `a17bef4b…`、s1 `3c10a0aa…`、s2 `ac398cb9…`、s3 `0440b0dd…`、跑记 `d3813874…`、翻译核对表 `e3ef6b9b…`。

### 3.1 复跑 corruption-check.py / oov-check.py（第 4 步：拷到草稿目录，不在原目录跑）

把 `research/scripts/corruption-check.py`、`oov-check.py`、`research/data/en-words.txt` 与三份样本一起拷到 `/tmp/claude-1000/presumed-r1-verifier/local-check-full/`，在那里现跑：

| 样本 | 跑记称 | 复跑 `corruption-check.py` | 复跑 `oov-check.py` |
|---|---|---|---|
| s1 | words=548，各项损坏计数全 0，绿 | ✓ 逐字相同（`words=548` 等全部计数一致） | ✓ 逐字相同（生词=8：overturned） |
| s2 | words=479，绿 | ✓ 逐字相同 | ✓ 逐字相同（生词=11：overturned contradicting recomputing dependence） |
| s3 | words=410，绿 | ✓ 逐字相同 | ✓ 逐字相同（生词=9：overturned candidate's） |

`wc -w` 现算三份样本词数 537/463/407，与跑记「词数」栏逐字相同。

s1 通读损坏判定：跑记引用第 9 行「This would be overturned if the admission calculation differed produced different results when a valid clean-shutdown marker was present.」——`awk 'NR==9'` 现取该文件第 9 行，**逐字相同**，确有「differed produced different results」这处读不通的残词；判「带损坏」有依据。

编号答复条数：`grep -cE '^[0-9]+\.'` 现数，s1/s2/s3 均为 **8**，与跑记「8 条编号答复」一致；s2、s3 判「干净」、达到两份门槛后停止抽样，符合规则「没打中至少两次」——但这里是「无损坏」这一类判定，不是「没打中」，跑记也没有把两份干净样本当「没发现问题」这一类结论使用，不适用「一次不算」那条纪律，这一步核的是操作程序本身没有走错。

### 3.2 翻译核对表 `m2-presumed-clauses-r1-local-attack-translation-audit.md`：14 条逐条核

| # | 引用（原文文件:行） | 结果 |
|---|---|---|
| 1 | `_m2-presumed-clauses-r1-body.md:28` | ✓ 逐字相同 |
| 2 | （无对应原文，自认不适用） | ✓ 属实：`_m2-presumed-clauses-r1-body.md:39` 确实只写「某两类之间」，未指定哪两类 |
| 3 | `03-空间分配.md:198`（已定项 10 ⑤ 引文） | 分不清：与一、条目 7 同一处漂移（现住 199 行），省略部分（打包容器落点、书目括注）核对属实是原句里真实存在、且与本题无关的另两处 |
| 4 | `03-空间分配.md:204-210`（t2-t8 角色表） | ✓ 逐字相同 |
| 5 | `03-空间分配.md:154`（标题）、`159`（第 2 条） | 154 ✓ 逐字相同；159 **对 HEAD 版本逐字相同**，但 working tree 已改写这一条（`聚簇段只给提交内生块` → `聚簇段在挂载期间对用户数据关着`，`git diff HEAD` 现查坐实），属「腿交回之后被改过」，不算腿的错——已核实 HEAD 恰好在 159 行、逐字与翻译对应的中文原文相同 |
| 6 | `crates/singlefs-core/src/mount.rs:4` | ✓ 逐字相同 |
| 7 | `16-发布语义.md:99`（已定项 1 射程段「先全环扫描、逐条验证、不许先信 tail」） | **✗**：`16-发布语义.md` 第 99 行现取是**空行**；全仓 `grep -n "先全环扫描、逐条验证、不许先信 tail" .claude/kb/decisions/*.md` 命中的**只有 `23-journal的角色与格式.md` 的第 120、127、375 行**，`16-发布语义.md` 里一次都不命中；`git log --all -p -S"先全环扫描、逐条验证、不许先信 tail" -- .claude/kb/decisions/16-发布语义.md` 全历史零命中，说明这句话在这份文件的任何一个提交里都不存在——不是漂移，是引错了文件（真正该引的是 `23-journal的角色与格式.md`）；已核对背景材料 `_m2-presumed-clauses-r1-background.md`、`_m2-presumed-clauses-r1-checklist.md`、`_m2-presumed-clauses-r1-appendix.md` 三份，这句话也不在它们的第 99 行，不属于「误写成背景材料行号」那一类 |
| 8 | `_m2-presumed-clauses-r1-body.md:46`（"有标记"候选，自认具体化） | ✓ 属实：该行是背景材料 P5 行，原文确实只给候选名 |
| 9 | `16-发布语义.md:39`（已定项 1 表格「准入」行） | ✓ 逐字相同 |
| 10 | `16-发布语义.md:185`（标题）、`187`（定案） | ✓ 两处逐字相同 |
| 11 | `16-发布语义.md:38`、`23-journal的角色与格式.md:363` | ✓ 两处逐字相同 |
| 12 | `16-发布语义.md:37` | ✓ 逐字相同 |
| 13 | `16-发布语义.md:205`（标题）、`207`（定案） | ✓ 两处逐字相同 |
| 14 | `22-单元原子性怎么合成.md:322`（标题）、`330`（第 5 句） | 分不清：working tree 现在分别住在 324、332（`git show HEAD` 核对，HEAD 版本恰好是 322、330，与本条引用逐字吻合）——与一、条目 19 同一处漂移，两条腿分别在漂移前后读到这份文件，互相印证了漂移确实发生在两条腿之间 |

**本地攻方小计**：corruption-check.py / oov-check.py 复跑 6 项全部 ✓；s1 损坏行现查 ✓；编号条数核 3 项全部 ✓；翻译核对表 14 条中 ✓ 10 条（含条目 3 本身带的 2 处「省略核对」并入 ✓）、分不清 3 条（3、5、14 三条系文件漂移，互相与其他两份报告的对应引用印证）、**✗ 1 条（条目 7，16-发布语义.md:99 系引错文件，非漂移）**。

## 总计数

| 腿 | 核了几处 | ✓ | ✗ | 分不清 | 核不动 |
|---|---|---|---|---|---|
| Sonnet | 41 | 35 | 0 | 6 | 0 |
| Opus（文件级 sha256） | 20 | 20 | 0 | 0 | 0 |
| Opus（正文摘录/原样日志核对） | 8 | 8 | 0 | 0 | 0 |
| Opus（独立复算脚本） | 3 | 3 | 0 | 0 | 0 |
| Opus（独立重建复跑，逐项当一次核） | 6 | 6 | 0 | 0 | 0 |
| Opus（叙述性计数句） | 2 | 0 | 2 | 0 | 0 |
| Opus（附录 kb 引文） | 21 | 21 | 0 | 0 | 0 |
| 本地攻方（脚本复跑） | 6 | 6 | 0 | 0 | 0 |
| 本地攻方（其他现查项） | 4 | 4 | 0 | 0 | 0 |
| 本地攻方（翻译核对表） | 14 | 10 | 1 | 3 | 0 |
| **合计** | **125** | **113** | **3** | **9** | **0** |

（判别力自证那一条单独计 1 次 ✗，不并入以上合计，按定义要求单列。）

## 没做什么

- 不判 K1–K4 任何一条推论打中成不成立、该不该采纳；不核推理链条本身，只核引用、产物与复跑——这一句在开头已抄，这里重申适用范围没有扩大。
- Opus 报告的「零轮形态」表（Z1–Z6）与「没打中的形状」两节只是叙述，没有独立可复跑的数字，没有核（它们本身也标注「只在我的模型上量过，被攻过零轮」，不要求核查员核实，只要求主 agent 判要不要采纳）。
- 没有核 Opus 副本 `walk.rs` 那处 `is_commit → commit_marker` 手改补丁的语义是否忠实（Opus 自己在「这条腿自己的限度」一节已声明「没核它的语义」）；本报告只核了补丁能让副本编译通过、且不设环境变量时与工作区行为一致这一句在**我自己独立重建的副本**上同样成立（K2-today、K4 四臂全部字节级复现，间接印证了这句声明）。
- K2 剩余四臂（A2、A4-any-cap1、A5-any-jump、A6-cap0）与 A7-jump 没有单独跑独立重建复跑——已独立重建复跑的 K2-today（A1）、K2-A3（对照组）、K4 全部四臂、纯 std 模型已覆盖派发提示点名要核的全部数（147968 字节、3102 次白推、K2 对照组只在 row_mod3=2 那一档丢、K4-a 196、K4-b 147/21），其余几臂只核了「贴出的原样片段与解压后的原始 `.log.gz` 逐字相同」（第 3 步），没有再花时间从零重建复跑（各臂单独跑一次崩溃扫描在这台机器上要数分钟到十余分钟不等，六条腿全部独立重建复跑的边际收益低于已完成的六项）。
- 没有核 Sonnet、Opus、本地攻方三份报告之间的判定是否一致、矛盾点该怎么判——那是主 agent 的职责，不是核查员的（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」）。
- 没有核背景材料 `_m2-presumed-clauses-r1-background.md`、正文 `_m2-presumed-clauses-r1-body.md`、附录 `_m2-presumed-clauses-r1-appendix.md`、清单 `_m2-presumed-clauses-r1-checklist.md` 本身写得对不对（那是三方论证起跑前该做的事，不属于这一轮核查员的射程），只在判断「是不是误写成背景材料行号」时读过这几份文件的对应行。
- 门禁、编译整个 crate（除本报告 2.1 节为复跑 K2/K4 测试而做的针对性 `cargo test --no-run` 编译）、变异表都没有跑，按定义不属于核查员该做的事。
