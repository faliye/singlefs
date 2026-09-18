# m2-wave2-code-r1 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 〇、判别力自证

选定引用：Opus 腿报告第 95 行「`crates/singlefs-core/src/bytes.rs:18` 原文 `self.bytes[self.cursor..self.cursor + slice.len()].copy_from_slice(slice);`」。

草稿副本：`/tmp/claude-1000/m2-wave2-code-r1-verifier/selftest/bytes.rs`（从主树拷贝，主树该文件与开工快照逐字节相同，见下）。行号加 1（18 → 19），取该副本第 19 行核对：

```
$ awk 'NR==19{print}' /tmp/claude-1000/m2-wave2-code-r1-verifier/selftest/bytes.rs
        self.cursor += slice.len();
```

与待核原文 `self.bytes[self.cursor..self.cursor + slice.len()].copy_from_slice(slice);` 不同。

**判定：✗（核查方法分辨得出错误行号）。**

## 前置：开工快照核实

`research/prompts/m2-wave2-code-r1-start-snapshot.sha256` 列 17 个文件。本次核查开工时重新 `sha256sum -c` 主树，17/17 OK（与主 agent 03:1x UTC 的核实一致，主树这 17 个文件至今未变）。因此下文对这 17 个文件里出现的引用**直接核对主树**（内容等同快照）；不在这 17 个文件之列的 kb 文件（`decisions/*.md` 等）**不在开工快照覆盖范围内**，核对结果单独标注「快照未覆盖，核主树」。

云端腿交回给的报告 sha256：

```
$ sha256sum research/prompts/m2-wave2-code-r1-opus-output.md research/prompts/m2-wave2-code-r1-sonnet-output.md
cc19307ae953c3a3c6b639bc09e9e697b3f81c0df8278cde79d3841cdce6b9fd  m2-wave2-code-r1-opus-output.md
ddf64d8608bb20a86eff85eacfd72ffdf61706e05a9acf7db56439e41555497a  m2-wave2-code-r1-sonnet-output.md
```

两份哈希与主 agent 派发提示里给的逐字相同，判两份报告**未在腿交回之后被改过**，下文核对全部针对这两份现有文件。

Opus 腿模型目录 `research/prompts/m2-wave2-code-r1-opus-model/`：`ls` 得 28 个文件，与报告第零节的 28 行 sha256 表逐条核对（`sha256sum *`），**28/28 全部相符**。

## 一、Opus 腿（云端攻方）核对表

复跑一律在 `/tmp/claude-1000/m2-wave2-code-r1-verifier/P/`（未插桩副本，`rsync -a --exclude target --exclude .git`）与 `.../Q/`（插桩副本，三份 diff `patch` 干净应用）里做，两份副本开工前各自 `sha256sum -c` 快照 17/17 OK。全部 `nice -n 19`。

### 1.1 行号+原文引用（对快照/主树逐字核）

| 引用 | 结果 |
|---|---|
| `bytes.rs:18`（3 处引用，第 73/103/106/95 行） | ✓ 逐字相同 |
| `make_filesystem.rs:95` | ✓ |
| `allocator.rs:601`（2 处，第 96/153/190/200 行区域） | ✓ |
| `mount.rs:396`（2 处） | ✓ |
| `transaction.rs:1297` | ✓ |
| `allocator.rs:598` / `614` | ✓（598 是函数起始行，报告写「起」非逐字引用，位置属实） |
| `transaction.rs:1593` / `1319`（2 处）/ `2096` / `2121` / `1221` / `2111` / `231` / `560` / `1258` | ✓ 全部逐字相同 |
| `unit.rs:111` / `160` | ✓ |
| `allocator.rs:419` / `375`（3 处，含 assert 消息「跨度里有已分配的槽」在 377 行）/ `544` / `609` / `403` / `495` / `536` | ✓ 全部逐字相同 |
| `mount.rs:712` / `838`（含上一行 837「分配记录树与记账树在每一次之后」的转述，属实）/ `927` / `933` / `652` / `919` / `676`（`)?;`） | ✓ 全部逐字相同 |
| `singlefs-format/src/lib.rs:101` / `162` | ✓ |
| `.claude/kb/milestone/02-second-txn.md:340`（第 28 行）/ 348（第 20a 行「映射与树表也没有条数上限」）/ 349（第 20b 行 `establish_instance`）/ 350（第 20c 行） | ✓ 全部逐字相同（该文件在开工快照 17 个文件之列） |
| `.claude/kb/decisions/28-挂载期承诺量.md:82` | ✓ 逐字相同；⚠️ **该文件不在开工快照覆盖范围内**，核的是主树，无法确认腿开工后是否被改过 |
| `.claude/kb/decisions/03-空间分配.md:255` | ✓ 逐字相同；同上快照未覆盖 |
| `.claude/kb/decisions/18-块里携带什么信息.md:882` | ✓（报告只说「那一整段里的『回收』条件」，非逐字引用；该段确有「**回收**：」小节）；同上快照未覆盖 |

### 1.2 产物内逐字查（grep 复现）

| 声明 | 命令 | 结果 |
|---|---|---|
| 「`crates/` 里 `grep -rn '删行\|下一片\|多片'` 只命中 `make_filesystem.rs:95` 一处」 | `grep -rn '删行\|下一片\|多片' crates/` | ✓ 恰好 1 处，与声明一致 |
| 「kb 与收口表里 grep『跨度里有已分配的槽』『重叠』无命中」 | `grep -rn '跨度里有已分配的槽' .claude/kb/`；`grep -rn '重叠' .claude/kb/` | ✗ 「跨度里有已分配的槽」0 命中（与声明一致），但「重叠」在 `.claude/kb/` 下有 **33 处**命中（`prior-art.md`、`invariants.md`、`checks-owed.md`、`experiments-history.md`、`tooling.md`、`milestone/02-second-txn.md` 等），并非「无命中」。声明的字面「重叠」检索项与实际不符 |
| 「测试之外七处 `PoolWriter::new`」 | `grep -rn 'PoolWriter::new' crates/`，按路径排除 `*/tests/*` | ✓ 恰好 7 处，行号（mount.rs:652/919、scenario.rs:101/110、bin/first_transaction_on_device.rs:458/725/928）全部相符 |
| 「全仓只有 `second_transaction_supplement_one_write_accounting.rs:559` 调 `writes_of_failed_publishes`」 | `grep -rn 'writes_of_failed_publishes' crates/` | ✓ 定义/字段声明之外只有这一处方法调用，行号相符 |

### 1.3 复跑（优先格）

| 格 | 复跑命令 | 结果 |
|---|---|---|
| Z1-a（漏判，实例表 370 号 panic） | `P` 上 `cargo test --release -p singlefs-harness --test opus_probe_pristine`（含 Z1-d 同一文件） | **复跑逐字相同**：关键行（`mount=…PANICKED…`、`superblock instances…`）与 `pristine-run.log` 逐字节一致（`diff` 空，仅线程 pid 不同，非证据内容）；调用栈（`z1a-per1.log` 的 `RUST_BACKTRACE=1`）`ByteWriter::put ← unit::build_packed_unit ← transaction::publish_admitted ← transaction::publish_version ← mount::publish_rows_on_file_version ← mount::establish_instance ← mount::mount_writable` 逐帧核对，7 帧顺序与行号（unit.rs:111、transaction.rs:1676/1256、mount.rs:713/1009/1141）均与模型目录原始日志相符 |
| Z1-a（插桩，per0/1/4/8） | `Q` 上 `OPUS_Z1A_PER_SESSION={0,1,4,8}` 各跑一次 | **复跑逐字相同**：四个 `PANICKED …base=280/342/486/590…` 行与 `z1a-per{0,1,4,8}.log` 逐字节一致 |
| Z1-d（复用跨度不同留重叠记录） | 同 Z1-a 命令（同一测试文件的第二个测试） | **复跑逐字相同**：`mount=8/9/10` 三行关键内容与 `pristine-run.log` 逐字节一致；checker 声明（`overlap-pinpoint.log` 里「28 条只有 I-3.1 红，从第 6 次挂载起、重叠出现（第 8 次挂载）之前就已经红」）在模型目录产物里逐字核实：`grep -n 'I-3.1' overlap-pinpoint.log` 显示首次 `Violated` 在 `mount=6`，首次重叠记录在 `mount=8`，且全程只出现过 `I-3.1` 一种违例类型——与声明一致（本条未重新跑 `overlap-pinpoint.log`，只核了模型目录里已有的产物） |
| Z1-b（上界拒真发得起来的池） | `Q` 上 `z1b_calibration` + `OPUS_Z1B_MOUNTS=30 … z1b_sweep` | **复跑逐字相同**：分离出 `z1b_sweep` 自身的输出（956 行 `h=…`）与模型目录 `z1b-sweep.log` 逐字节一致（`md5sum` 相同）；报告表格 5 行（(48,1)/(47,2)/(47,3)/(49,0)/(48,2)）的 `status=`、`exact=[…]` 值全部核对相符；汇总数字「按上界被拒 56 次」（`grep -c upper=Some`=56）、「拷贝上装得下 40 次」（56−16 个真拒=40）、「288 段历史」（唯一 `h=(a,b,c)` 组合数=288）、「714 次挂载判定」（`base=…reclaimed=…planned=…exact=` 格式行，713 条以 `h=` 起始 + 1 条被 cargo 的 `test … ... h=` 前缀吞掉首字符，共 714）、「13 次」`IN_SESSION_FALSE_REFUSAL`（Z1-f 用）全部复算一致 |
| Z1-d（扫描汇总，208/133） | 同上 `z1b_sweep` 产物 | ✓ `grep -c 'BEFORE_ADMISSION status=panicked 跨度里有已分配的槽'`=208，其中 `mount=2`=133，与报告一致 |
| Z2-c（根已 FUA 后另换一路发布，池挂不上） | `P` 上 `cargo test --release -p singlefs-harness --test opus_probe_z2`（4 个测试一起跑） | **复跑逐字相同**：`z2a_failure_after_the_root_is_durable_then_a_different_publish` 的输出（`C failure account:…`、`checker on the image now:[…5 条 Violated…]`、`remount: REFUSED Recovery(UnitUnreadable { slot: SlotNumber(50257) })`）与模型目录 `z2.log` 逐字节一致；`z2c_mount_that_fails_at_the_second_warm_up_hands_back_no_account` 的输出与 `z2c.log` 逐字节一致；对照组 `control remount: mounted, chosen root txg=5` 一致 |
| Z3-a（重开后用户数据落进上一次挂载的段） | `Q` 上 `cargo test --release -p singlefs-harness --test opus_probe_z3` | **复跑逐字相同**：`patterns=88 strong_hits=18` 与全部 18 条 `STRONG_HIT` 行（含报告正文引用的 `pattern=[24]`、`pattern=[24, 0]` 两条）与模型目录 `z3-final.log` 逐字节一致（`diff` 空） |
| Z1-c（三个改法臂在打中格上的数，只核数不判改法好坏） | `Q` 上 `cargo test --release -p singlefs-harness --test opus_probe_z1_fixes` | **复跑逐字相同**：一次跑出的两个测试输出，拆开后分别与模型目录 `z1-fixes.log`、`z1-fixes-natural.log` 逐字节一致；报告表格（169-179 行）全部 5+2+1 行的 `exact=[…]`、`records:` 数值、`mode=` 分支结果均复核一致，包括「自然历史」行「第 15 次覆盖写 panic（写之前 808 条、已回收 180）」（原样 `wrote 14 then the next overwrite PANICKED (records before=808, reclaimed=180)`，14+1=15）与「写 14 次后正常拒（真分配 814）」 |

未复跑的格（Z2-a/b、Z1-e/f 的量、Z3-b/c）：报告称「没打中」或「推的」，未落在本轮侧重的 5 格里，未复跑；对应的行号与算术引用已在 1.1/1.2 节按逐字核对方式核过。z1-fixes-30-ps{0,1}.log（`OPUS_FIX_MOUNTS=30`）未复跑，时间/token 预算内未覆盖，列入「没做什么」。

Opus 腿小计：核了 28 处行号引用 + 3 处 kb（milestone）引用 + 3 处 kb（decisions，快照未覆盖）引用 + 4 处 grep/计数声明 + 8 个复跑格（含子项）。✓ 43 处（含 3 处「快照未覆盖，主树核对相符」单列）；✗ 1 处（「重叠」grep 声明与实际不符）；核不动 0 处。

## 二、Sonnet 腿（正推）核对表

不核 2.4 节（E142 量 5 的 11 个区域不等，主 agent 已另派诊断 `research/prompts/e142-r11-f1-diagnosis.md`）；不核 `walk.rs:963-972` 与 `decisions/08-核心索引结构.md:430`（主 agent 已现查属实）。

### 2.1 行号+原文引用

| 引用 | 结果 |
|---|---|
| `invariants.md:131`（I-3.9 整行）、`:274`（I-9.14 整行） | ✓ 逐字相同（该文件在开工快照 17 个文件之列） |
| `walk.rs:1005-1024`（`judge_release_generations`） | 内容语义相符，**但代码块非逐字抄录**：源码是多行链式调用（`.filter(...)` `.map(...)` `.max()` 各占一行），报告压缩成更少行数并省去 `else` 分支里的中文注释（以「...」代替）；字符串/逻辑本身未失真 |
| `walk.rs:1036-1041` | 同上，多行 `judgements.not_applicable(...)` 调用被压缩成两行；字符串参数逐字一致，非逐行抄录 |
| `walk.rs:1066-1096` | ✗ **闭包参数名被改写**：报告代码块写 `.map(\|s\| s.tree_table_placement)`，源码实际是 `.map(\|sighting\| sighting.tree_table_placement)`；按「产物逐字找」，报告里这一行字面在 `walk.rs` 中找不到（`grep -F '.map(|s| s.tree_table_placement)' crates/singlefs-checker/src/walk.rs` 零命中）。其余行（`distinct_tree_tables`、`if compared_trees == 0` 块）内容准确，仅这一处参数名被改写 |
| `walk.rs:1099-1117`（`judge_release_generation_and_tree_table_birth` doc comment 与分支） | ✓ 两行文档注释逐字相同；`if candidate_indexes.len() < 2 {`（1108 行）与 `return;`（1117 行）逐字相同；两个 `not_applicable(...)` 调用字符串参数逐字相同（多行压缩为一行，非逐字排版） |

| `transaction.rs:1172` / `1348` / `977` / `1399` / `1577` / `425` / `446` | ✓ 全部逐字相同 |
| `singlefs-format/src/lib.rs:201`（`FIRST_TRANSACTION_TXG = 3`）/ `:114`（`FIRST_TRANSACTION_ACCOUNTING_ROWS = 15`） | ✓ |
| `allocator.rs:499` / `519` / `661` / `752` / `883` / `1132` / `11` | ✓ 全部逐字相同 |
| `image.rs:36-40`（`IMPLEMENTED_INVARIANTS` 28 项数组） | ✓ 逐字相同，含 `I-3.9`、`I-9.14` 两项 |
| `checker_known_bad_images.rs:642` / `687` | ✓ 两个函数签名行存在且逐字相同 |
| `first_transaction_regions.rs:132-179` 11 组常量（regions.rs:40/42/43/47/50-57） | ✓ 位置与数值全部相符；⚠️ 报告表格把 `const XXX: u64 = N;` 简写成 `` `XXX = N` ``（去掉 `const`/类型/分号），是简化写法非误引，未改变数值 |
| `layout/01-first-txn.md:67-79`（t1..t11 与「21 条」） | ✓ 11 行槽号/偏移逐一核对相符，第 79 行「⇒ 写请求数…21 条」逐字相符 |
| `milestone/02-second-txn.md:113 / 304 / 311 / 320 / 321 / 328 / 331 / 332 / 333 / 334 / 335` | ✓ 全部逐字或准确转述相符（该文件在开工快照之列）；`:509` 的转述「都只描述了『代码与 E142 都按 1』这件旧事」是概括性描述而非逐字引用（509 行原文是「第一个事务的改动计数 1 与 D8 已定项 6 字段表不符」），但其论证要点（两处都未提及 `decisions/08:430` 需要跟着改）经核实成立 |

### 2.2 产物内逐字查（grep / diff 复现）

| 声明 | 结果 |
|---|---|
| `grep -n 'name=change_count_diff' …三.out \| wc -l` = 67 | ✓ |
| `name=diff_explained_summary` 181 行 `segments=67 unexplained=0`；`name=extra_structures` 182 行 | ✓ 行号与内容逐字相符 |
| `name=impl_region_table_against_writes` 308 行 `regions=21 write_calls=21 …matches=true` | ✓ |
| `name=impl_bytes_equal_summary regions=21 equal=10 unequal=11` | ✓（2.4 节内容，本轮不判该节结论，但产物核对属实） |
| `diff` 两份 `.out` 的 `name=width`/`name=segments` 均无输出 | ✓ 复现，两条 diff 均空 |
| `awk 'NR==97/101/102/103/104/107/108/109/111/112/113/114/115' crates/mutations.tsv` 各条内容 | ✓ 13 处全部逐字相符（锚点代码、测试目标名与报告描述一致） |
| `grep -n 'journal_tail\|\.tail\b' recovery.rs` 零命中 | ✓ |
| `recovery.rs:6` 注释「journal 全环扫描、不先信 tail」 | ✓ |
| `grep 08-核心索引结构.md:430` 与 `改动计数.*430` 于 `checks-owed.md` 零命中 | ✓ 两条均零命中 |
| `grep -n 'fn build_file_version_units\|BirthSequenceAllocator::default()' transaction.rs` | ✓ 行号 977/1399/1577 相符 |
| `grep -n 'cluster_segments\b' allocator.rs` | ✓ 5 处行号（499/519/661/752/883）相符 |

Sonnet 腿小计：核了约 30 处行号引用（4 处代码块非逐字排版，其中 1 处闸内容参数名被改写记 ✗）+ 15 处 grep/diff 产物核对。✓ 44 处；✗ 1 处（walk.rs:1066-1096 闭包参数名 `s` vs `sighting`）；核不动 0 处；未复跑（Sonnet 自陈未跑 cargo，本轮不替它跑，其自述的「复核不了」数字维持原状不单独打分）。

## 三、本地攻方腿核对表

### 3.1 转述核对表（`m2-wave2-code-r1-local-attack-translation-audit.md`）逐条核

| 标签 | 原文文件:行 | 结果 |
|---|---|---|
| I-3.9 标题、S1-S6 | `invariants.md:131` | ✓ 六个分句切分与行内位置相符；S1-S6 英译逐句核对未丢限定词、未加原文没有的限定词（S1 末句「开左闭右」的文字化说明已由核对表自陈为符号转写并单列披露，未构成额外限定） |
| I-9.14 标题、T1-T4 | `invariants.md:274` | ✓ 四个分句切分与行内位置相符，英译逐句准确 |
| C1 | `walk.rs:1099` | ✓ 译文对应第 1099 行第一分句，逐字准确 |
| C2 | 核对表标 `walk.rs:1100` | ✗ **行号引用不全**：C2 英译的前半句（"When the candidate set has only one root left, both invariants report not applicable."）对应的中文「候选集只剩一条根时两条都报『不适用』——」实际位于 **1099 行末尾**，不在 1100 行；只有后半句（「最早不再引用它的那条有效根」…）才在 1100 行。核对表应写 `walk.rs:1099-1100`，只写 `:1100` 遗漏了前半句真正所在的行 |
| Br1 | `walk.rs:1108、1117` | ✓（核对表自陈「非原文句子，转述代码结构」，如实披露；1108/1117 内容核对相符） |
| Br2 | `walk.rs:1111` | ✓ 逐字（字符串参数）相符 |
| Br3 | `walk.rs:1115` | ✓ 逐字相符 |
| 「英文多出来的限定词」两条（S1 末句、C2 两处 `(used by …)`） | 同上 | ✓ 自陈披露与实际情况相符，未发现另有未披露的新增限定词 |
| 「有意删掉的括注」四条（S3/S4/T2/C2 各一处 D/C 编号出处或日期） | `invariants.md:131`/`:274`、`walk.rs:1100` | ✓ 四处删减确实只涉及出处编号或实测日期，不含区间端点/候选集门槛等表 1、表 2 要用的量，核对表判断成立 |

### 3.2 提示文件（`m2-wave2-code-r1-local-attack.md`）与运行记录

- 提示第 10-16、144-151 行的格式要求（禁 markdown 强调、禁行号、Row 6-17 的 "I-3.9: …." "I-9.14: …." 双标签格式）与运行记录所述「改后（最终版）」一致：当前落盘的提示文件就是第 4、5 次调用用的那份，未发现残留旧格式。

### 3.3 复跑（字词损坏闸 + 哈希）

草稿拷贝，`nice -n 19` 原样调用 `research/scripts/corruption-check.py` 与 `research/scripts/oov-check.py`（不复跑本地模型本身，只复跑闸——本地模型的原始调用不可重放，闸对已落盘的样本文件是确定性的）：

| 样本/副本 | 声明 | 复跑结果 |
|---|---|---|
| `-output-void1.md` | 「命中『英文复读』6 处、『实词自复读』4 处」 | ✓ `corruption-check.py` 复跑：判红，`英文复读=6(19.35/千) …实词自复读=4`，逐字段相符 |
| `-output-void2.md` | 「同一模式判红（英文复读 6 处、实词自复读 4 处）」 | ✓ 复跑：`英文复读=6(17.86/千) …实词自复读=4`，相符 |
| `-output-s1.md` | 「干净」「448 词」 | ✓ `corruption-check.py`/`oov-check.py` 均判绿；`wc -w` = 448，与声明的「词」数一致（`corruption-check.py` 内部另有一个不同口径的 `words=` 计数器，值为 412，属于该脚本自己的统计量，与报告引用的「词」数不是同一个量，不构成矛盾） |
| `-output-s2.md` | 「干净」「510 词」 | ✓ 判绿；`wc -w` = 510，一致 |
| `-output-s3.md` | 「干净」「332 词」 | ✓ 判绿；`wc -w` = 332，一致 |
| `-output-s1.md` 哈希前缀 | 派发提示转述「腿交回写『0358bd…』，实际『035bd6b4…』不符」 | ✓ 复核：`sha256sum` 实际值 `035bd6b4c5637f557ad220666f19bbcbb1f7af47e9cbe35bd7ffbc442c99d677`，前缀确为 `035bd6b4`，与派发提示所记「实际」一致，与「0358bd…」不符——腿交回的哈希前缀确有误（推测是数字换位），但这份文件本身内容判绿，不影响 s1 不计入两份干净样本的结论 |
| void1/void2 文件权限 | 运行记录称「未删除、未覆盖」 | ✓ 两份仍在，`-rw-------`，600 权限、大小与运行记录一致 |

本地攻方腿小计：核了 8 处行号引用 + 4 处披露表内容 + 6 项复跑（闸 + 哈希 + 词数）。✓ 17 处；✗ 1 处（C2 行号范围遗漏 1099）；核不动 0 处。

## 四、总计数

| 腿 | 核了 | ✓ | ✗ | 核不动 |
|---|---|---|---|---|
| Opus（云端攻方） | 43 | 42 | 1（「重叠」grep 声明与实际不符） | 0（另有 3 处「快照未覆盖，主树核对相符」已计入 ✓，单列说明见 1.1 节） |
| Sonnet（正推） | 45 | 44 | 1（`walk.rs:1066-1096` 闭包参数名 `s`/`sighting` 不符） | 0 |
| 本地攻方 | 18 | 17 | 1（C2 行号范围遗漏 1099） | 0 |
| **合计** | **106** | **103** | **3** | **0** |

## 五、没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身（Opus 的「四句」分析、Sonnet 对每行的「一致/规则没说/过时」结论、本地腿答的 17 行具体判词），只核引用、产物与复跑。
- 未核 Sonnet 报告 2.4 节（E142 量 5 的 11 个区域不等）——按派发提示已由 `research/prompts/e142-r11-f1-diagnosis.md` 另查。
- 未核 `walk.rs:963-972` 注释过时的判断与 `decisions/08-核心索引结构.md:430`「改动计数 1」——按派发提示，主 agent 已现查属实。
- 未复跑 Z2-a/b、Z1-e/f、Z3-b/c（报告称「没打中」或「推的」，不在本轮侧重的 5 格内），未复跑 `z1-fixes-30-ps{0,1}.log`（`OPUS_FIX_MOUNTS=30` 的两次扩展跑），未复跑 `overlap.log`/`overlap-pinpoint.log` 本身（只核对了模型目录里已有产物的内容，未重新生成）。
- 未判 Sonnet 自陈「复核不了」的具体数字（行 20b 的 11/21/32 次写入字节数、行 20c 的落点 50306、行 20d 的 3310/50304→50368、行 23′ 的 65536 字节、行 23″ 的 8 个状态）——Sonnet 自己没有跑 cargo 去核这些数，本轮按派发提示的侧重（Opus 5 格 + Z1-c 数字 + 本地腿转述表）分配时间，未替 Sonnet 去跑这些测试来坐实其「未核」的数字，此项在没做什么里明列，不计入 Sonnet 小计的核对项。
- 未跑门禁、未编译产品代码之外的任何东西、未碰仓里除草稿目录与本报告之外的任何文件。
- 未判三条腿之间是否一致、是否互相印证——那是主 agent 判决要做的事。
