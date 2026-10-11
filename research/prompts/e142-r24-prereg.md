# E142 差分重跑登记（第 24 次）：守门收成「写清单那两组只许载荷摘要一列变」之后，装置对 `crates/` 在主点上逐字节同不同；第 23 次够判的四样还成不成立

<!-- doc-lint:not-numbers N22R P8G -->

写于 2026-10-05，这一次的任何产物之前。判据、门槛、作废与停机条款在这里写死；跑出数之后要改，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」三步走，不在这里回改。

**差分登记，文件名取 `r24`，不占号。** 上一次是第 23 次：登记 `research/prompts/e142-r23-prereg.md`（697 行，sha256 `2dcff2fa308fa5d2cae07b0972a5b982bf3c7e4ddb27a68e05dba279eec263bd`，含它第十二节执行员写的「修订一」），问题单 `research/prompts/e142-r23-questions.md`（sha256 `0bda733775cd0c979cefff6b58804187ad696120115c13837dbe9a054fe00033`）。两份冻结不动，连同第 23 次的全部产物。下文「同第 23 次」指第 23 次登记里同名的那一格（它再指到第 22 次的，照它的指法），执行员照原文做，只把产物文件名里的 `r23` 换成 `r24`、把参照产物换成这份写的那几份。编号接第 23 次：读法 R79 起，格 Q142.123 起；这一次不加变异（M 号不动）；失败 / 作废 / 停机条款用 F24*、V24*、S24-*，第 23 次的号不复用。

**变了哪些输入（一句话一项）**：

| 输入 | 改了什么 | 谁改 |
|---|---|---|
| 守门判据 | 第 23 次 Q142.114（只许三组变）换成 Q142.123（许五组变，多出的两组 `write_list_row`、`p8g_write_list_row` 只许 `sha256=` 一列变、新值等于同一次写的区域行 `sha256=`；全文见第六节）。这是放宽，见下一段 | 主 agent 定形态，这份登记写死 |
| 装置 `scripts/experiments/e142_new_pool_file_creation_dry_run.rs` | **不变**：第 23 次冻结版（10866 行，sha256 `43338ce5…5dff`，与第 22 次第二段冻结版只差第 1524 行，第十三节命令十六现核）。这一次一字不改（S24-scope） | — |
| `crates/` | 第 30 批合入后的主工作区，另有别的线合进来的改动（`git status --short crates/` 136 条，设计时现数）。设计时 `crates/singlefs-core/src/system_configuration.rs` sha256 `314660f2…729f`（等于第 23 次起跑前提 P1 记的值），导出 bin `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs` sha256 `a5b419cf…9809`；起跑时执行员现记，第 23 次以来变了哪些文件由 Q142.124 列出、归因 | 别的线（这一次不改） |
| `.claude/kb/layout/01-first-txn.md` 第 149 行 | 第 23 次步 ⓪ 之前已写回成「5000（单位毫秒，即 5 秒）」；今天照第 23 次命令二的八个取法现抄，出口 sha256 `6cc2aae5…c8c1`，等于第 23 次起跑前提 P4 记的值（第十三节命令二） | 已写回 |
| `.claude/gate.d/stage-inputs.tsv` 第 36 行（准入键 `E142`） | 加进这份登记（起跑前提 P2） | 主 agent |
| `scripts/replay.sh` 第 174、175 行 | 第 24 次够判之后改指这一次的主产物与 P8G 导出（步 ④） | 这一次的执行员 |
| 新加草稿脚本 | `q123_guard.py`（256 行）与 `crates_half_compare.py`（142 行），全文与自检输出在第十三节 | 这份登记 |
| 不变 | 变异表 `scripts/mutations/e142_new_pool_file_creation_dry_run.tsv`（210 行，`ef63ea85…f1c3`）；独立 bin `e142_region_diff_independent.rs`（`311ce77c…ae51`）与它的变异表（`8978e80a…a8e`）；第 21–23 次的草稿脚本；`driver_e142`（`scripts/replay.sh` 第 512–518 行）与 `driver_e142_p8g_crates_dump`（第 524–526 行） | — |

**记一次输、放宽在哪、因此是新一次跑**（`evidence-discipline.md`「臂的定义也在「跑前写死」之列」那三步；这一次三步走不全，所以是新一次跑，不是第 23 次的修订）：

| 步 | 写什么 |
|---|---|
| 记一次输（第 23 次） | 第 23 次的守门 Q142.114 按登记字面打中 F23d：N22R → 主产物的归一化 diff 在登记点名的三组（`device_region_bytes`、`p8g_device_region_bytes`、`journal_header_write_digest`）之外，`write_list_row`、`p8g_write_list_row` 各变 2 行。**最弱读法**：第 23 次登记 5.1 臂 N23 的「做完会怎样」在这一格上不成立——它预言只有三组变，产物里五组变；推理漏了写清单行也带载荷摘要 `sha256=`。第 23 次因此照字面记第 1、3 行不够判，登记、问题单与产物一字不改 |
| 放宽在哪 | 第 23 次的判据让「三组之外有一组有差异」触发 F23d；Q142.123 在三组之外再许两组变（各恰 2 行、只 `sha256=` 一列）。第 23 次判 F23d 的那种观测，这里判过——按原来那套判据，新形态判得**更松**，不是收严。所以它不能写进第 23 次的第十二节修订，只能在这一次的任何产物之前写死，当新一次跑 |
| 因此是新一次跑 | 主产物、P8G 导出、快照、比对与锚点全用 `r24` 新文件名；第 23 次的判定照它的字面留着，第 24 次不替它改判。第 23 次已够判的第 2 行（P8G）不重判，引第 23 次读数（下一段） |

**相对主 agent 给的形态，这份登记多要的（收严，写在 Q142.123 里）**：变的写清单行恰 2 行，它们的（step, device）与区域行那一组变的两行相同；旧值也等于参照产物里同一次写的区域行 `sha256=`；两份归一化之后行名序列逐行相同；两份里四组（`device_region_bytes`、`write_list_row` 与两个 `p8g_` 组）各恰 29 行。

**更正第 23 次 5.1 那一句（这一次臂 N23 的「做完会怎样」）**：装置里写清单行与区域行由同一次写的同一段字节算摘要——主点在第 7508 行起的循环里，第 7509 行 `let sha = sha256_hex(&write.bytes);` 写进区域行，第 7514 行 `write_list_row(…, &write.bytes)` 在第 6307 行 `let sha = sha256_hex(bytes);` 再算一次；P8G 那一段在第 5949 行起的循环里同样两行一对（第 5951、5954 行），行名由第 6093 行 `prefix_row_name_with_p8g` 加前缀。所以 t15 两盘那两次写的字节一变，区域行与写清单行的 `sha256=` 一起变、值相同，别的列（`bytes=`、`slot=`、`offset=` 等）不变。**这一半是构造上成立的**（同一个函数算同一段字节），不当判据；Q142.123 的判别力在「只这一列」「只这两行」「别的组零差异」三处。

**哪几格要重判**（逐格在第六节）：

- 问题单第 1 行：Q142.105、Q142.106、Q142.107（主点四行）、Q142.108（主点四行）、Q142.113、Q142.116 照第 23 次原判据在这一次的新产物上重判；Q142.114 换成 Q142.123；新加 Q142.124（第 23 次以来 `crates/` 那一侧动了什么）、Q142.125（N23 没漂）。
- 问题单第 2 行：Q142.117、Q142.118、Q142.119 照第 23 次原判据重判；Q142.120（整张变异表）由主 agent 跑完补判。
- **不重判、引第 23 次读数**：第 23 次问题单第 2 行（P8G）那几格——Q142.109–Q142.112、Q142.107 与 Q142.108 的 P8G 那几行、PC-f23p、PC-k；第 23 次执行员报告第八节第 2 行记「已够判：相同且不翻面」，产物在 `research/results/e142-new-pool-file-creation-dry-run-r23-main-2026-10-05.out` 与 `research/results/e142-r23-p8g-crates-dump-2026-10-05.out`。这一次照样跑 P8G 导出（只为 `replay.sh` 第 175 行有一份今天的产物可指），与第 23 次那份 `cmp` 一次（Q142.126，附带）。
- **不重跑、引第 23 次读数**（输入文件与脚本都与第 23 次相同，确定性）：PC-f23、PC-t2、PC-h3、步 ①(6) 的 B1 / B1b 复算、Q142.115、Q142.121、Q142.122。理由与替代逐条在第五节 5.2。

**装置写在哪（写死）**：同第 23 次文件头。模型是 `scripts/experiments/e142_new_pool_file_creation_dry_run.rs`（独立手写模型，不与 `crates/` 共用代码），**不是入库装置**；这一次**一字不改**。比对侧导出 bin 只跑、不读、不改，主点照 `driver_e142`，P8G 照 `driver_e142_p8g_crates_dump`；独立 bin 只用 `old-new`。不新建 bin，不改 `crates/` 的任何文件，不改两个 driver。这一次没有计时类的量。

**臂 N22R 不重跑（写死）**：今天仓里的装置是 N23；要在今天重跑 N22R，得把仓里这份装置（准入输入）临时换回第 22 次第二段的样子再换回来，这一次不做。Q142.123 的参照取第 23 次的臂 N22R 产物 `research/results/e142-new-pool-file-creation-dry-run-r23-arm-n22r-2026-10-05.out`（冻结，sha256 `428d360c…218b`）。比的只是装置那一侧（R71 剔掉了读导出的行），装置是确定性模型；它在今天的环境里会不会跑出别的样子，由 Q142.125（同一份 N23 源码，第 23 次主产物 → 第 24 次主产物）判：零差异，Q142.123 的差异就只能来自第 1524 行。

**起跑前提（写死；步 ⓪ 逐条核，有一条不齐 ⇒ 停机，一步都不跑）**：

| # | 前提 | 怎么核 | 不齐 ⇒ |
|---|---|---|---|
| P1 | T_time 毫秒仍在 `crates/` 里 | 同第 23 次 P1 那三条 `grep -c`，依次 1、0、1（设计时现核一样，第十三节命令一）；之后 `sha256sum crates/singlefs-core/src/system_configuration.rs crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs` 原样写进报告。两个值与第一节文件头表里设计时的值不同**不停机**，只记，归 Q142.124 | S24-open |
| P2 | 准入表认这份登记 | `awk 'NR==36' .claude/gate.d/stage-inputs.tsv \| grep -c 'research/prompts/e142-r24-prereg.md'` 为 1（第 22、23 次登记那两条留不留由主 agent 定，不核） | S24-open |
| P3 | 装置、表、参照产物都是登记时那一份 | 命令一的前四个 sha256 与行数（装置 `43338ce5…5dff` 10866 行、变异表 `ef63ea85…f1c3` 210 行、独立 bin `311ce77c…ae51`、它的表 `8978e80a…a8e`）；参照产物五份的 sha256：第 23 次主产物 `ea4e6f42…dcc6`、第 23 次臂 N22R `428d360c…218b`、第 23 次 P8G 导出 `e7aa451e…fb75`、第 23 次快照 C `4252e797…07e9`、第 22 次第二段主产物 `4b333e3f…eaed`（全值在第十三节命令一） | S24-sha |
| P4 | 条款没在第 23 次之后变 | 照第十三节命令二的八个取法重抄进另一个出口，sha256 必须是 `6cc2aae548cb709e486779dd5b4723ca39fef59f40594eceda7c9d17c211f8c1`（这一次没有允许的差异）；不同就 `diff` 两份出口、原样进报告 | S24-clause |
| P5 | 第 1524 行仍是唯一的改动 | 第十三节命令十六：仓里装置把第 1524 行换回旧行，写进执行员草稿目录的 `frozen-s2.rs`，sha256 必须是 `c400139b8cfc7c0e4b96d483200055e9891252c9171bbbeb06771046bf63d899`，`diff` 恰 `1524c1524` 一处 | S24-sha |

**准入（写死）**：同第 23 次文件头准入一段：装置 `main()` 开头的准入调用，键 `E142`；准入的 stderr 原样存进执行员草稿目录、贴进报告；退 0 才往下做；退 77、3、2 ⇒ 停机 S24-admission；不设 `SINGLEFS_EXPERIMENT_RERUN_REASON`、不设 `ADMISSION_BREAK`。⚠️ **这份登记是准入输入：步 ② 主产物落盘之后一个字都不改。** 第十二节修订只在步 ②(2) 单测之后、步 ②(4) 主产物之前写，只许收严或补臂，写明时点与依据。内存包装报 250–254 的，照它自己打的出路改（例如它要求的理由变量），不绕开包装；那一次的输出不算产物。

**执行员按什么次序做（一段做完，写死）**：

| 步 | 做什么 | 这一步之前不许读 |
|---|---|---|
| ⓪ | (1) 起跑前提 P1–P5。(2) `ps -o pid,args -u "$(id -u)"` 看负载，照共用约束办。(3) 快照 A：照第 23 次步 ⓪(3)（即第 22 次步 ⓪(4) 的两条命令），存 `research/results/e142-r24-crates-sha256-a-<日期>.txt` 与 `research/results/e142-r24-crates-status-a-<日期>.txt`；随即做 Q142.124 (1)（第十三节命令十七，只打文件名）。(4) 重建草稿脚本八份、核 sha256（第五节 5.4 的表）；跑命令十一（PC-t）、命令十五（PC-q123、PC-c124）、`back_chain_487.py` 不带输入与 `BACK_CHAIN_487_BREAK=1` 各一次（PC-h1、PC-h2）、`label6_anchor.py` 不带输入与 `LABEL6_ANCHOR_BREAK=1` 各一次（PC-j 前两格），输出存 `research/results/e142-r24-controls-<日期>.out` | 第 23 次的 N22R 产物、主产物与第 22 次第二段主产物的内容（P3 只取 sha256） |
| ① | (1) 命令七、命令六、命令十二在仓里装置与变异表上跑（Q142.118）；命令十二另在 `frozen-s2.rs` 上跑（PC-twelve）；命令七另在 HEAD 版草稿树上跑（PC-i，HEAD 版装置 sha256 `2f4ea2cf…2999`，第十三节命令一现核仍是它）；追加进 controls 文件。(2) 照 `driver_e142_p8g_crates_dump` 跑 `crates/` 的 P8G 导出，存 `research/results/e142-r24-p8g-crates-dump-<日期>.out`；随即 `cmp` 它与第 23 次 P8G 导出（Q142.126，附带，只记相同与否）。(3) PC-g23：同第 23 次（合成文件用第 23 次 N22R 产物的拷贝，设备 1 那份窗口系统配置槽区域内偏移 6 那个字节异或 `0x01`；`old-new (合成, N22R)` 与 `old-new (N22R, N22R)`），独立 bin 在仓里编（release）；追加进 controls 文件 | 同上（PC-g23 只经独立 bin 读 N22R 产物） |
| ② | (1) S24-clause 再抄一次（命令二）。(2) 在仓里编装置（release，源码不动）；单测 `cargo test --release -p e7-index-bench --bin e142-new-pool-file-creation-dry-run`（经内存包装）必须 117 过、0 败、1 忽略（第 23 次修订一的数），有一条红 ⇒ 停机 S24-scope，不改单测。(3) 快照 B，与 A `cmp`（V24c）。(4) 照 `driver_e142` 跑，整份存 `research/results/e142-new-pool-file-creation-dry-run-r24-main-<日期>.out`（准入必须放行）。(5) 快照 C，与 B `cmp`；A、B、C 三份都进 `research/results/`。(6) 立即跑 `python3 scripts/admission.py experiment <仓根> E142`（Q142.116） | 同上；`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs` |
| ③ | (1) **从这一格起才读** `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs` 第 80 行（只读那一行）。(2) Q142.125：`q123_guard.py <第 23 次主产物> <第 24 次主产物>`。(3) Q142.123：`q123_guard.py <第 23 次 N22R> <第 24 次主产物>`，再逐组写归因表。(4) Q142.124 (2)：`crates_half_compare.py <第 23 次主产物> <第 24 次主产物>`。(5) Q142.113：独立 bin `old-new (第 24 次主产物, 第 23 次 N22R)`，存 `research/results/e142-r24-region-old-new-independent-<日期>.out`；`ttime_anchor.py --diff <第 24 次主产物> <第 23 次 N22R>` 两种行名各一次。(6) `ttime_anchor.py --decode` 主产物（两种行名）与 `--decode <第 24 次 P8G 导出>`；`label6_anchor.py --decode` 主产物（两种行名）与 `--decode <第 24 次 P8G 导出>`；与 (5) 的 `--diff` 一起存 `research/results/e142-r24-anchors-<日期>.out`。(7) 其余各格（Q142.105、Q142.106、Q142.117、Q142.119）与 (2)(3)(4) 的输出存 `research/results/e142-r24-comparisons-<日期>.out`；Q142.119 在这一格才读第 22 次第二段主产物与第 23 次 N22R 产物里的判决字段 | — |
| ④ | `scripts/replay.sh` 第 174 行的产物名改成步 ②(4) 那份、第 175 行改成步 ①(2) 那份（别的字段不动），**只在**第 1 行够判、第 2 行 (a)(b)(c) 判完（不论成立与否）之后改；第 1 行不够判的不改、交回。写实验页（加第二十四次跑一节）；整张变异表照第九节写成「要主 agent 跑的命令」交回（Q142.120）；交回照第十一节「交回之前」 | 实验页「判决」「影响的决策」两节在这一步才读 |

文件名里的日期取跑的那一天（`TZ=Asia/Tokyo date +%F`）；已存在的同名文件不覆盖，按日期另存。这一次装置不许改一个字（S24-scope），看过步 ③(1) 那一行钉值或任何比对之后同样不许。

## 一、问题

英文名：new_pool_file_creation_dry_run（照抄第 23 次登记第一节；源文件、变异表、bin 名都沿用它）

**问题单（`research/prompts/e142-r24-questions.md`，sha256 `b9e5d4aa7c8e7c58bcbbcae6b6eb32270bfc438244ca427a61c27f719082fc47`，两行；起因段、表头与两行由 `quote-kb.py` 机械抄，出口 14 行、sha256 `1de5d9a78aca5d9006fc428deeca577421901096f7e29a0766efc97c58576654`；不是为岔路建的实验，一行一个要回答的判断）**：

**出处 `research/prompts/e142-r24-questions.md:5-5`（整段抄，未转述）**

```markdown
第 23 次（登记 `research/prompts/e142-r23-prereg.md`，问题单 `research/prompts/e142-r23-questions.md`，报告 `/tmp/claude-1000/e142-r23-run/report.md`，主产物 `research/results/e142-new-pool-file-creation-dry-run-r23-main-2026-10-05.out`）第 2 行够判；第 1、3 行的守门 Q142.114（「改动之外什么都没动」）按登记字面打中 F23d：N22R 到主产物的归一化 diff 除登记点名的三组（`device_region_bytes`、`p8g_device_region_bytes`、`journal_header_write_digest`）外，`write_list_row`、`p8g_write_list_row` 各变 2 行。第 23 次的登记与产物冻结不动，照字面记第 1、3 行不够判。
```

**出处 `research/prompts/e142-r24-questions.md:7-10`（整段抄，未转述）**

```markdown
| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| 1 | 守门收成「写清单那两组只许 `sha256=` 一列变、且每一行的新值等于同一次写的 `device_region_bytes`（或 `p8g_device_region_bytes`）那一行的 `sha256=`」之后，装置（第 23 次那一份，只改过第 1524 行）与 30-3 合入后的 `crates/` 在主点几何上 29 个区域逐字节相同吗（含反向链） | 相同 / 哪几个区域不同 | 比对行 `unequal` 大于 0；或守门在收过的形态下仍打中 | 主点比对行与守门全数出来 | 开着 |
| 2 | 同一组输入下，第 23 次已够判的几样还成立吗：字段表 174 行逐字段相等、装置不剩旧值、判决字段与第 22 次第二段逐个相同、整张变异表全抓 | 都成立 / 哪一样不成立 | `name=layout_field … equal=false` 有一行；`residual_old_values count` 大于 0；判决字段翻面；变异有没红或无效 | 四样各数出来 | 开着 |
```

**派发提示给的问法（一种读法）**：守门收成「写清单那两组只许 `sha256=` 一列变、且每一行新值等于同一次写（同 step、同 device）的 `device_region_bytes` / `p8g_device_region_bytes` 那一行的 `sha256=`」，相对第 23 次是放宽；装置不再改；`crates/` 是今天的主工作区，守门那一格要分得出「别的线改了别的字节」与「T_time 带来的」；第 23 次已够判的第 2 行（P8G）不重判。

**读法写死**（跑之后不改）：

| # | 名字 | 写死成 |
|---|---|---|
| R79 | 第 1 行「守门收成……之后」 | 守门 = 第六节 Q142.123，由 `q123_guard.py`（第十三节全文，sha256 `9f6f9dec…5045`）在 `<第 23 次臂 N22R 产物> <第 24 次主产物>` 上算；`Q123_VERDICT pass=true` ⇒「守门在收过的形态下过」，`pass=false` ⇒「仍打中」（F24d）。前提：Q142.125 零差异（否则 Q142.123 的差异里混着别的来源，见 R84） |
| R80 | 第 1 行「装置（第 23 次那一份，只改过第 1524 行）」 | 仓里装置 sha256 = `43338ce5…5dff`（P3），且把第 1524 行换回旧行之后 sha256 = 第 22 次第二段冻结值 `c400139b…d899`（P5）。**对面读法**：装置照旧写 5（秒）——即第 23 次的臂 N22R，这一次不重跑，取它的冻结产物当参照 |
| R81 | 第 1 行「30-3 合入后的 `crates/`」 | 步 ②(4) 那一刻的主工作区 `crates/`：P1 三条 `grep` 过；快照 A = B = C（V24c）。它与第 23 次时的 `crates/` 同不同，不进第 1 行的判定，由 Q142.124 列出、只给不同的区域归因 |
| R82 | 第 1 行「在主点几何上 29 个区域逐字节相同吗（含反向链）」「比对行 `unequal` 大于 0」 | 同第 23 次 R74：主产物 `name=impl_bytes_equal_summary` 的 `matched=29`、`equal=29`、`unequal=0`、两个 `unmatched` 0 ⇒「相同」，否则按 `region=` 列出连同 `fields=`；反向链三格（t13 对步 ③(1) 现读的钉值、t13 对 7.2 B1、w4 对 7.2 B1b）各判 |
| R83 | 第 2 行「同一组输入下，第 23 次已够判的几样」四样 | 同第 23 次 R77 (a)–(d)，产物换成第 24 次主产物：(a) `layout_field_summary rows=174 equal=174 unequal=0` 且 PC-e 过；(b) 命令七 `count=0` 连同 `287` / `295` 逐处归类、命令十二前两个计数 0 第三个 1，命令六另列；(c) R71 判决字段在第 22 次第二段主产物与第 24 次主产物之间逐个相同；(d) 整张变异表 210 条：抓到 210、无效 0、没红 0。「同一组输入」= 同一份冻结装置、同一张变异表、步 ②(4) 那一份 `crates/` |
| R84 | 派发提示「守门那一格要能分出「别的线改了别的字节」与「T_time 带来的」」 | 分三处量，各管一种来源：**Q142.125** = 同一份 N23 源码在第 23 次与第 24 次各跑一份，装置那一侧（R71 剔除之后）的差异——源码没变，差异只能来自 `crates/` 漏进装置那一侧的行或环境；**Q142.124** = 两份主产物里 `crates/` 导出那一半与 `crates/` 快照的差异——两次都是 T_time 5000，差异只能来自第 23 次以来别的线的改动；**Q142.123** = N22R → 第 24 次主产物，装置那一侧的差异——Q142.125 为零时，它只能来自第 1524 行（T_time）。Q142.105 有不同区域时，逐个看它在不在 Q142.124 (2) 变了的区域里：在 ⇒ 记「别的线改了这一区」，连同 Q142.124 (1) 的文件清单；不在 ⇒ 记「第 23 次以来 `crates/` 这一区没变、装置也没变却不同」，走作废 V24a |

第 1 行挂在 R79–R82、R84；第 2 行挂在 R83。每一格只有这一种读法。

## 二、被测条款与它引的定义

**同第 23 次第二节**（D16（发布语义） 已定项 5；`layout/01` 第 40–41、81、83、149、158 行；判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第 80、95 行），不重抄。今天照第 23 次命令二的八个取法现抄，出口 72 行、sha256 `6cc2aae548cb709e486779dd5b4723ca39fef59f40594eceda7c9d17c211f8c1`，与第 23 次起跑前提 P4 记的出口相同（第十三节命令二）；那一份与第 23 次第二节（`57c56d04…1373`）只差第 149 行，第 23 次报告 P4 记了 `diff`。⇒ 第 23 次第二节里第 149 行那一段今天是下面这一行（`quote-kb.py` 机械抄，出口 `/tmp/claude-1000/e142-r24-designer/e142-r24-section2.md`，5 行，sha256 `f025243e81b9cb345423ab678083a64a33c94271a8ccba9c1d7dba7cf6089c3d`，回读一致后原样追加；它是一整行表格行，不以 `|---|` 收尾），其余七段逐字同第 23 次第二节。

**出处 `.claude/kb/layout/01-first-txn.md:149-149`（整段抄，未转述）**

```markdown
| 可调值 | T_time | 4 | 5000（单位毫秒，即 5 秒） | D16（发布语义） 已定项 5 |
```

## 三、实现今天的样子

### 3.1 `crates/`（2026-10-05 现查，HEAD `31e47115`；`git status --short crates/ | wc -l` 为 136，别的批次的未提交改动也在里面；行号只是那一刻的，执行员按名字现查）

- T_time 的写出：`crates/singlefs-core/src/system_configuration.rs:86` `const TIME_THRESHOLD_MILLISECONDS: u32 = 5000;`、`:291`–`:292` `pub fn time_threshold_milliseconds(self) -> u32 { TIME_THRESHOLD_MILLISECONDS }`、`:523` `writer.put_u32(self.runtime.time_threshold_milliseconds());`。文件 sha256 `314660f2…729f`（`git status` 为 `M`），与第 23 次起跑前提 P1 记的合入之后的值相同。
- 导出 bin：`crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs`（`git status` 为 `M`，sha256 `a5b419cf…9809`）；它是比对侧，只跑不读。第 23 次没有单独记这份文件的 sha256，它在第 23 次快照 C 里；两次之间它变没变由 Q142.124 (1) 回答。
- 反向链钉值：`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:80`（`git status` 为 `M`）；设计时 grep 那个名字读到了第 80 行（第四节第 4 条），执行员步 ③(1) 现读为准。

### 3.2 装置与独立 bin（第十三节命令一今天现核）

- 装置 10866 行 sha256 `43338ce525b4b3e8705b6f05c79a72bec5d8748222ed012aeadcc9df47805dff`，等于第 23 次修订一的冻结值；第 1524 行 `        writer.put_u32(5000); // T_time 毫秒`（`grep -c` 1），旧写法 0 处；把它换回旧行得 `c400139b…d899`，即第 22 次第二段冻结值（命令十六）。`git status` 为 `MM`。
- 变异表 210 行 `ef63ea85…f1c3`；独立 bin `311ce77c…ae51`、它的变异表 `8978e80a…a8e`；都等于第 23 次第三节 3.2。HEAD 版装置 `2f4ea2cf…2999`（PC-i 用），等于第 22 次 PC-i 写的值。
- 区域行与写清单行的摘要：见文件头「更正第 23 次 5.1 那一句」（第 564、5949–5954、6093、6306–6307、7508–7514 行）。写清单行的格式（第 6310、6325、6332 行）：`name=write_list_row step= writer= unit= class= bytes= slot= tree= device= offset= level= entries= entry_width= declared_length= min_key= max_key= birth_sequence= sha256=`，没有 `hexadecimal=`，归一化不动它。
- `name=journal_header_write_digest`（第 7286–7289 行）：`index= kind= device= offset= length= sha256=`，`kind=` 取第 689–697 行的五种之一，系统配置槽是 `system_configuration_slot`。
- 装置的命令行参数（第 7489、7599、7651 行）：第一个是 `crates/` 导出、第二个是臂 N15 参照产物（固定文件）、第三个（G4 一盘导出）`driver_e142` 不给。装置那一侧读导出的行，就是 R71 剔除的那六种与 E2、E3；Q142.125 判这一句在今天还成不成立。

### 3.3 装置要改的地方

没有。这一次一字不改。

## 四、跑之前已经存在的数

写判据之前已经读到或算出的数，照实列。**这一次的门槛不盲**：第 23 次的结果整份读过，Q142.123 的形态就是看过第 23 次的分组表之后定的。

1. **第 23 次执行员报告全文**（派发提示点名要先读，`/tmp/claude-1000/e142-r23-run/report.md`，153 行）：主点 `equal=29 unequal=0`；N22R `equal=27 unequal=2`（设备 0 / 1、偏移 4096、`first_diff_offset=155`、`mismatch_bytes=6`）；t13 `back_chain=2660362816`、w4 `750278362`；八行 `t_time=5000 le_hex=88130000`；四行 `TTIME_DIFF changed_bytes=6 offsets=155,156,157,158,433,434 outside_t_time_and_crc=0`；独立 bin `changed=2 unchanged=27`、`unmapped_regions=2`；字段表 `rows=174 equal=174 unequal=0`、PC-e `injected=4 detected=4 exact=true`；`residual_old_values count=0`、命令六 0 0 0 0、命令十二 0 0 1 0；77 个判决字段与第 22 次第二段逐个相同，`verdict` 里 `control_outcome_matrix_ok`、`g7_states_ok`、`g7_violations_zero` 三个 `false` 与 `layer0_*`、`journal_differing_states` 三个 `not_run`；P8G `equal=29 unequal=0`、`p8g_control_main_point equal=12 unequal=0`；记账两行（主点空闲每盘 3 472 605 184、P8G 7 767 572 480，标签 6 98 386）；**Q142.114 的分组表**：`device_region_bytes` 2/2、`journal_header_write_digest` 12/12（恰是全部 `kind=system_configuration_slot` 行）、`p8g_device_region_bytes` 2/2、`p8g_write_list_row` 2/2、`write_list_row` 2/2（step 31、32，只变 `sha256=`，值与同一次写的区域行相同）；整张变异表没跑；主产物与第 22 次第二段差 22 对行；草稿脚本七份的 sha256。**影响**：① Q142.123 的形态（五组、写清单行只变 `sha256=`）照这张分组表就能预言过——它是主 agent 看过结果之后定的放宽，这一次登记照实写成「不盲」，不把 Q142.123 过当成新的证据，只当「守门在写死的形态下没被别的东西打中」；② Q142.125、Q142.124、Q142.126 是看过上面这些数之后加的，门槛都是「零差异 / 相同」这种不取任何读数的字面；③ 别的格的门槛照第 23 次原样（第 23 次是在它自己的产物之前写的），没有一格取上面的数当门槛；④ 第 2 行 (c) 那三个 `false` 已知，判据是「与第 22 次第二段逐个相同」，不判它们该不该是 `false`。
2. **第 23 次登记**（697 行，整份读了）：判据、读法、锚点值、草稿脚本全文与 sha256、修订一的冻结值。**影响**：照用，差分登记只改写这里点名的格。
3. **第 22 次登记**：标题行与 `R71`、`PC-`、`V22e`、`**命令` 的 grep 命中行（第 82、83、86、1009、1027、1028、1040–1048、1082、1087、1093、1217、1608 行与第十三节各条 `**命令` 标题行）；第 19 次登记 `R37`、`Q142.56` 的 grep 命中行。**影响**：R71 的归一化与剔除照原文写进 `q123_guard.py`；PC 的定义照用。
4. **`crates/` 一侧**：第三节 3.1 那几行；`new_pool_file_creation_publish.rs` 里 `E142_BACK_CHAIN_OF_NEW_POOL_FILE_CREATION` 的 grep 命中第 80、566 行（钉值 2 660 362 816，第 23 次登记与报告里本来就有）。**影响**：只拿来比，不当门槛；Q142.106 的钉值只在步 ③(1) 现读。
5. **第 23 次执行员草稿目录**（`/tmp/claude-1000/e142-r23-run/`）：列了文件名，读了 `norm.py`、`groupdiff.py` 两份脚本全文（R71 归一化与分组的写法），核了 `frozen-s2.rs` 的 sha256。没读那里的任何输出文件。**影响**：`q123_guard.py` 的归一化照第 22 次登记 R71 原文另写，没有拷这两份；两者在同一份输入上会不会给同一个归一化结果，这一次不核。
6. **自己算的**：第十三节命令一（各 sha256、三条 `grep`、HEAD 版装置）、命令二（条款现抄 `6cc2aae5…`，问题单抄录）、命令十五（两份新脚本的合成自检：PC-q123 九份样本全 `ok=True`、弄坏开关下 `bad_sha` 判过、`all_ok=False`；PC-c124 五份全 `ok=True`、弄坏开关下 `slot_outside` 漏判）、命令十六（换回第 1524 行得 `c400139b…`）。
7. **没读的**：`research/results/` 下任何一份文件的内容（命令一只对七份算了 sha256，`ls` 看了 E142 的文件名）；`.claude/kb/experiments/`、实验索引、实验变更史；第 23 次问题单之外的问题单。

## 五、臂、阳性对照、真实基线

### 5.1 臂

问题单两行都不是在两种设计之间选。臂同第 23 次两条，这一次只**跑** N23。

| 臂 | 怎么做 | 做完会怎样（待证命题，由第六节的格核，不当前提） | 谁认它 |
|---|---|---|---|
| **臂 N22R（参照，不重跑）** | 第 23 次的臂 N22R 产物原样（P3 核 sha256 `428d360c…218b`）；它的源码就是 P5 换回第 1524 行的那一份 | 装置那一侧与第 22 次第二段主产物逐行相同（第 23 次 Q142.115，读数引第 23 次） | 「T_time 盘上写 5（秒）」那种读法：用户定案之前 `layout/01` 第 149 行「5 秒」的字面 |
| **臂 N23（真实基线，这一次跑）** | 第 23 次冻结装置一字不改，在步 ②(4) 那一份 `crates/` 上照 `driver_e142` 跑 | 装置那一侧与第 23 次主产物逐行相同（Q142.125）；对 N22R 只在五组变：`device_region_bytes`、`p8g_device_region_bytes` 各恰 t15 两盘那 2 行（偏移 4096、长 4096），`write_list_row`、`p8g_write_list_row` 各恰同两次写的 2 行、只变 `sha256=` 且值等于同一次写的区域行，`journal_header_write_digest` 恰全部 `kind=system_configuration_slot` 那些行（Q142.123）；变了的槽只在 T_time [433, 437) 与 CRC [155, 159) 两段（Q142.113）；反向链不变；判决字段与第 22 次第二段逐个相同（Q142.119） | 用户 2026-10-05 定案、D16 已定项 5 的默认值 5 s、`layout/01` 第 149 行今天的写法 |

**「怎么做」推得出「做完会怎样」**：同第 23 次 5.1 那一段，加上文件头「更正第 23 次 5.1 那一句」（写清单行与区域行同摘要）。装置那一侧与第 23 次主产物逐行相同这一句，靠的是装置确定、源码相同（P3）、装置那一侧不读导出（第三节 3.2 末条）——最后一条是待证的，Q142.125 核。

**对面那条读法**：「T_time 盘上写 5、单位秒」，就是 N22R，不另造。它在 `crates/` 一侧对 30-3 合入后的导出出现过一次（第 23 次 PC-f23、PC-f23p），这一次不重跑。

**没改的（写死，不在这一次的射程里）**：同第 23 次 5.1 末段；另外这一次不改装置、不加单测、不加变异（S24-scope）。

### 5.2 阳性对照（每一条被判的比对都有；能在这一次的臂上跑的都跑）

| # | 对哪条比对 | 怎么做 | 该看到什么 | 看不到时 |
|---|---|---|---|---|
| **PC-q123 守门脚本分得出每一种违例**（新） | Q142.123、Q142.125 | 命令十五：`q123_guard.py --selftest` 一次、`Q123_GUARD_BREAK=1` 一次 | 前一次九行 `ok=True`、`all_ok=True`（`good` 判过、`same` 的 `changed_lines_total=0`，其余七种违例各判不过）；后一次 `bad_sha` 那一行 `ok=False`、`all_ok=False` | 作废 V24d |
| **PC-c124 `crates/` 一半的比对分得出 T_time 与别的字节**（新） | Q142.124 (2) | 命令十五：`crates_half_compare.py --selftest` 一次、`CRATES_HALF_BREAK=1` 一次 | 前一次五行 `ok=True`、`all_ok=True`；后一次 `slot_outside` 那一行 `ok=False`、`all_ok=False` | 作废 V24d |
| **PC-q123b 真产物上分得出 T_time**（判别力自证） | Q142.125 | Q142.123 那一次的 `Q123 changed_lines_total=` | ≥ 1（N22R 与 N23 的第 1524 行不同，同一个脚本必须看得见差异） | 为 0 ⇒ 脚本在真产物上读不出差异（例如行名对不上），Q142.125 的「零差异」不作数，作废 V24d |
| **P2（装置自己的翻位对照）** | Q142.105 | 主产物四行 `name=positive_control_p2` | 四行都是 `regions_flagged=1`、`matches_injection=true` | 作废 V24d |
| PC-t、PC-e、PC-h1、PC-h2、PC-j 前两格、PC-g23、PC-i、PC-twelve | 同第 23 次 | 同第 23 次（PC-g23 的输入取第 23 次 N22R 产物的拷贝；PC-twelve 的 `frozen-s2.rs` 由 P5 生成） | 同第 23 次 | 同第 23 次（作废号换成 V24*） |

**不重跑的对照（写死，理由逐条）**：

| # | 第 23 次的用途 | 这一次为什么不重跑 | 由什么顶 |
|---|---|---|---|
| PC-f23 | 比对器分得出 T_time（N22R 对 30-3 合入后的导出报 t15 两盘不同） | 要在今天重跑 N22R（文件头「臂 N22R 不重跑」）；装置里的比对器源码与第 23 次逐字节相同（P3） | P2 在这一次主产物里重跑（比对器分得出一个翻位）；读数引第 23 次 |
| PC-t2 | 解码读对了那一行（N22R 装置一侧 5、`crates/` 一侧 5000） | 输入是第 23 次的冻结产物、脚本 sha256 同第 23 次，跑出来不会变 | PC-t 重跑（脚本对合成文件解码对）；Q142.107 的 `magic=SFSB` 一栏 |
| PC-h3、B1 / B1b 复算 | 锚点 B1 的输入对得上条款 | 输入是第 23 次 N22R 产物同一行同一段，与第 23 次同一份脚本 | PC-h1、PC-h2 重跑；B1 = 2 660 362 816、B1b = 750 278 362 照第 23 次 7.2 |
| PC-f23p、PC-k | P8G 比对 | 第 2 行（P8G）这一次不重判 | — |
| Q142.115、Q142.121、Q142.122 | N22R 没漂、两份主点导出相同、N22R 的不同区域清单 | N22R 不重跑；这一次只有一份主点导出 | Q142.125（N23 没漂）；快照 A = B = C（V24c） |

### 5.3 真实基线

**真实基线是臂 N23**。第 1 行「相同」与「哪几个区域不同」、第 2 行「都成立」与「哪一样不成立」都是正当结果（第十节），不许写成「装置没测出差别所以坏了」。⚠️ 两份实现照同一个定案改，一起错时由 Q142.107 的绝对值盯（主点两侧都要解出 5000）。

### 5.4 实现量、分段与草稿脚本

量（估计）：没有历史族、崩溃支线、坏镜像、新子系统，装置不改。`driver_e142` 一次、P8G 导出一次；装置编一次、单测一次；独立 bin 三次（PC-g23 两次、Q142.113 一次）；草稿脚本约 20 次；整张变异表 210 条（主 agent 跑）。**一段做完**，不分段：文件头步骤表 ⓪–④。一次做不完时停在步 ② 单测之后（主产物之前），报告写做完的、没开的。问题单两行在这一段都能够判（6.1）；第 2 行 (d) 在主 agent 跑完整表之后补判。

草稿脚本八份（执行员照全文重建进自己的草稿目录，核 sha256；不等 ⇒ 那份作废，照全文重来，仍不等 ⇒ 停机 S24-sha）：

| 脚本 | sha256 | 全文在 | 这一次用在 |
|---|---|---|---|
| `ttime_anchor.py` | `f598eee8f08090dbcadc6338bf05ea811e4ba8fd9c8f0a9c6c366a03d2a55089` | 第 23 次登记第十三节 | PC-t、Q142.107、Q142.113 (2) |
| `make_synthetic_ttime.py` | `3e33a12a08bfbfc079c454ef570733a6b775f24d3f5b232618e6f7198e6f029f` | 同上 | PC-t |
| `label6_anchor.py` | `15c945328a473e21c0e50a604445e54b9b40b98c4996a434a1596883a70725f7` | 第 22 次登记第十三节 | PC-j 前两格、Q142.108 |
| `make_synthetic.py` | `f9936271b8c23c7c4fd7868d9b18bb4978ac18c7d9c443e5b25099f83b0fede8` | 同上 | `label6_anchor.py` 的合成自检（同第 22 次命令四） |
| `back_chain_487.py` | `b267c41bc41b3f41e6d216c3608ad1cdd57f0496af4b69024b8b5e69516a89de` | `research/prompts/e142-r21-prereg.md` | PC-h1、PC-h2 |
| `residual_old_values.sh` | `63ea086247aaee402ff74cf9dd3c9d51c1220f7f379f3f5dc2f2bf595de2a1c0` | 同上 | 命令七、PC-i |
| `q123_guard.py` | `9f6f9decbc61ba7a646377597da07be1244b01849cead338040c700d51225045` | 这份登记第十三节 | PC-q123、Q142.123、Q142.125 |
| `crates_half_compare.py` | `59c8e8688e0e7e9d0896f87d3df58158056c15c2cc7efd951402c88977ede889` | 这份登记第十三节 | PC-c124、Q142.124 (2) |

Q142.119 的判决字段比对照第 23 次 Q142.119（第 22 次 R71 的字段集与剔除，执行员自己写，脚本与它的 sha256 进报告）。

## 六、报哪些量与各自的判据

编号接 Q142.123 起。每格各报各的判定，不合取；合取（某一行「够判」要哪几格都过）由主 agent 对着问题单做。臂 N23 的定义是「第 23 次冻结装置、第 1524 行写 5000、一字不改」；能从定义直接推出的（源码那一行是 5000、写清单行与区域行同摘要）不算判据。判别力在「两份独立实现写出的字节同不同」「两侧解出来是不是定案的数」「改了这一处之后别的组、别的行、别的列动没动」「今天的环境与 `crates/` 下装置那一侧跑不跑得出同一份」。「同第 23 次」一列指照那一格的算法与门槛原样做、只换输入。

| 格 | 同第 23 次 | 怎么算（产物里的行） | 门槛 / 判定 | 对应问题单哪一行 · 取什么值让它翻面 |
|---|---|---|---|---|
| **Q142.105 主点逐区域（R82）** | Q142.105 | 第 24 次主产物 `name=impl_bytes_equal_summary`；不同时取 `equal=false` 与 `impl_bytes_unmatched` 的行 | 同第 23 次：`matched=29`、`equal=29`、`unequal=0`、两个 `unmatched` 0 ⇒「相同」；否则按 `region=` 列出连同 `fields=`，逐个按 R84 归因 | **第 1 行**。不同区域数 ≥ 1 ⇒「哪几个区域不同」（列出、归因），同时停机 S24-crates |
| **Q142.106 反向链（R82）** | Q142.106 | 主产物 `name=root_record` 的 `back_chain=`；`name=back_chain` 的 `chains=` 里 w4 那一条 | 三格各判「相等 / 不等」：t13 对步 ③(1) 现读的钉值；t13 对 7.2 B1；w4 对 7.2 B1b | **第 1 行「含反向链」**。t13 ≠ 钉值 ⇒ 第 1 行翻面、S24-crates；= 钉值而 ≠ B1 ⇒ F24f；两边都不等 ⇒ V24b |
| **Q142.107 T_time 的绝对值（主点四行）** | Q142.107 的主点那一半 | `ttime_anchor.py --decode <主产物>`（装置两盘、`crates/` 两盘） | 四行都是 `magic=SFSB t_time=5000 le_hex=88130000` ⇒「两侧都是 5000」；装置一侧有一行不是 ⇒ F24a；装置一侧是而 `crates/` 一侧有一行不是 ⇒ 记进第 1 行的不同；四行相等而都不是 5000 ⇒ F24a「两侧一起错」 | 第 1 行「相同」的绝对值旁证。另 `--decode <主产物> p8g_device_region_bytes` 两行与 `--decode <第 24 次 P8G 导出>` 两行：**附带，只报不判** |
| **Q142.108 标签 6 的绝对值（主点四行）** | Q142.108 的主点那一半 | `label6_anchor.py --decode <主产物>`（行名缺省） | 装置两盘、`crates/` 两盘都是 `committed_reservation=label6:98386` | 第 1 行守门：不符 ⇒ F24d（改动碰了别处）或 F24a，两边都查。P8G 那几行附带只报 |
| **Q142.113 改动只动了 t15（第 1、2 行守门）** | Q142.113，参照换成第 23 次 N22R 产物 | (1) 独立 bin `old-new (第 24 次主产物, 第 23 次 N22R)` 的汇总行与逐区域行；(2) `ttime_anchor.py --diff <第 24 次主产物> <第 23 次 N22R>` 两种行名，各两行 `TTIME_DIFF` | 同第 23 次：(1) `changed=2`，变的恰是 7.2 A9 那两个，`unchanged=27`，两个 `unmatched` 0；(2) 四行 `changed_bytes` ≥ 1、`outside_t_time_and_crc=0`；`all_changes_mapped` 与 `class=` 只报不判 | 守门。多变或少变一个区域、或有一个字节落在两段之外 ⇒ F24c，第 1、2 行不够判 |
| **Q142.116 指纹** | Q142.116 | 主产物落盘之后立即跑准入 | 退 77 ⇒「指纹对上」 | 不对应问题单的行。退 0 ⇒ 停机 S24-fingerprint |
| **Q142.117 字段表（第 2 行 (a)）** | Q142.117 | 主产物 174 行 `name=layout_field`、`name=layout_field_summary`、`name=layout_field_positive_control` | 同第 23 次 | **第 2 行 (a)**。`unequal` ≥ 1 或 `rows` ≠ 174 ⇒「不成立」（F24i） |
| **Q142.118 旧值残留（第 2 行 (b)）** | Q142.118 | 仓里装置与变异表上跑命令七（附 `287` / `295` 归类表）、命令十二；命令六另报 | 同第 23 次：命令七 `count=0` 且归类表里没有一处指 fsid / MAC，**且**命令十二四个计数 0、0、1、0 ⇒ (b)「成立」；命令六四个计数另报 | **第 2 行 (b)**。命令七 ≥ 1 或命令十二前两个有一个 ≥ 1 ⇒「不成立」（F24j） |
| **Q142.119 判决字段不翻面（第 2 行 (c)）** | Q142.119，N22R 的值取第 23 次 N22R 产物 | 第 22 次第二段主产物与第 24 次主产物里 R71 的判决字段逐行逐字段比；翻面的字段另看第 23 次 N22R 产物与第 23 次主产物里的值 | 同第 23 次：变了的字段数 = 0（30 行 `name=width` 都 `expected == actual`）⇒ (c)「成立」；否则逐个列出（行、字段、第 22 次第二段值、第 23 次 N22R 值、第 23 次主产物值、第 24 次值）：第 23 次主产物已经是新值 ⇒ 记「第 23 次就变了」；只在第 24 次变 ⇒ 看 Q142.124：`crates/` 变了 ⇒ 记「`crates/` 那一侧变的」，否则 F24h | **第 2 行 (c)**。≥ 1 个字段变 ⇒「不成立」，列出 |
| **Q142.120 整张变异表（第 2 行 (d)）** | Q142.120 | 主 agent 跑第九节那条命令；按每条结果行开头的符号数七栏 | 同第 23 次：几栏之和 = 210、收尾有「已还原，基线仍全绿」、`✅` = 210 ⇒ (d)「全抓」；`⏭` 另与 `research/results/e142-mutate-r22-s2-2026-10-05.log`（sha256 `0f3d459e…bc5c`）的 `⏭` 数比 | **第 2 行 (d)**。`❌` 或 `⏭` ≥ 1 ⇒「不成立」；`⚠️`、`⏱`、`🧱` ≥ 1 ⇒ 那一轮失败，重跑，不判 |
| **Q142.123 收过的守门：改动之外什么都没动（第 1、2 行守门，R79）** | 换掉 Q142.114 | `q123_guard.py <第 23 次 N22R 产物> <第 24 次主产物>`（归一化与剔除 = 第 22 次 R71，写在脚本头）；之后执行员逐组写一行「组、变几行、归到哪一处、证据（装置行号）」 | **全部成立才算「守门过」**，每条各报：① `Q123_COUNT` 八行 `rows=29`、`Q123 order_same=true counts_ok=true`；② 有差异的组只许五组：`device_region_bytes` 与 `p8g_device_region_bytes` 各恰 2 行变，都是偏移 4096、长 4096、设备 0 与 1，只变 `sha256=` 与 `hexadecimal_sha256=`；`journal_header_write_digest` 变的行恰是参照里全部 `kind=system_configuration_slot` 的那些行（`Q123_DIGEST` 两数相等），只变 `sha256=`；`write_list_row` 与 `p8g_write_list_row` 各恰 2 行变，**只变 `sha256=` 一列**，它们的（step, device）与同前缀区域行那一组变的两行相同，四行 `Q123_PAIR` 都是 `changed_keys=sha256`、`new_equals_…=true`（新值等于第 24 次主产物里同一次写——同 step、同 device——的区域行 `sha256=`）、`old_equals_…=true`（旧值等于参照里同一次写的区域行 `sha256=`）；③ 其余每一组零差异，`verdict` 一组先删 E3 再比（脚本做）；④ `Q123_VERDICT pass=true ambiguous=0 groups_outside_allowed=none`；⑤ 五组都归到第 1524 行（t15 两次系统配置写经 `to_slot`） | 守门。`pass=false`（五组之外有差异、五组里变的行或列不是上面写的）⇒ F24d；有一组归不到第 1524 行 ⇒ F24e；两种都让第 1、2 行不够判。`ambiguous` ≥ 1 ⇒ 作废 V24d；`Q123_COUNT` 有一行不是 29 ⇒ 作废 V24e |
| **Q142.124 第 23 次以来 `crates/` 那一侧动了什么（R84，归因）** | 新 | (1) 第十三节命令十七：第 23 次快照 C 与第 24 次快照 A 的差，只打 `<` / `>` 与路径；(2) `crates_half_compare.py <第 23 次主产物> <第 24 次主产物>` 的全部输出 | 只报、归因，不设门槛：(1) 列出变了、多了、少了的文件，条数用命令数；(2) 报 `CRATES_HALF identical=`、每个 `CRATES_GROUP`、每个 `CRATES_REGION`（不同的区域连同 `changed_bytes`、`first_changed`、`outside_t_time_and_crc`）与 `CRATES_REGION_SUMMARY` | 第 1 行：自己不让它翻面；给 Q142.105 的每个不同区域归因（R84）。(1) 空而 (2) `identical=false` ⇒ 先看仓根 `Cargo.toml` / `Cargo.lock` 有没有变（`git diff --stat` 原样进报告），解释不了 ⇒ 作废 V24a。`ambiguous` ≥ 1 ⇒ 作废 V24d |
| **Q142.125 N23 没漂（前提，R84）** | 新，顶第 23 次 Q142.115 的位置 | `q123_guard.py <第 23 次主产物> <第 24 次主产物>` | `Q123 order_same=true` 且 `Q123 changed_lines_total=0` ⇒「N23 没漂」（这一次 `Q123_VERDICT pass=false` 是预期的：五组都没变，脚本按 Q142.123 的形态判不过；只读那两行） | 第 1、2 行的前提（它为零，Q142.123 的差异才只归第 1524 行）。≥ 1 行或 `order_same=false`：Q142.124 (1) 为空 ⇒ 作废 V24a；不为空 ⇒ 停机 S24-drift |
| Q142.126 附带：P8G 导出与第 23 次同不同 | 新 | `cmp <第 23 次 P8G 导出> <第 24 次 P8G 导出>` | 只报「相同 / 不同（`cmp` 第一处偏移）」，不判 | **附带**：不对应问题单的行；只因 `replay.sh` 第 175 行要一份今天的产物才跑导出，顺带一次 `cmp`。不同 ⇒ 交回写明「第 23 次 P8G 的读数只覆盖第 23 次那份 `crates/`」（第八节那一格欠着）；够判后不另跑任何 P8G 比对 |

### 6.1 够判点

- **第 1 行够判**：Q142.105、Q142.106、Q142.107（主点四行）判完；P2、PC-t、PC-h1、PC-h2、PC-j 前两格、PC-g23、PC-q123、PC-q123b、PC-c124 过；守门 Q142.113、Q142.123 过，Q142.108 符，前提 Q142.125 过；Q142.124 两项出数（有不同区域时逐个归因完）。打中 F24c、F24d、F24e 时不够判，停下交回。判「不同」照样够判，同时带停机 S24-crates 的报告。
- **第 2 行够判**：Q142.117、Q142.118、Q142.119 判完（PC-e、PC-i、PC-twelve 过；守门 Q142.123 过、前提 Q142.125 过，同第 23 次 Q142.114 管它第 3 行那样），Q142.120 由主 agent 跑完补判。判「不成立」照样够判。

## 七、钉绝对值的断言

同第 23 次第七节，一条不改：7.1 K1–K8 出自被测条款本身，不符走 F24a「条款可能错」（两边都查，不作废）；7.2 A1–A9、B1、B1b、B2–B11 独立算出、用命令核过，不符作废 V24b。K7 的出处今天多一处：`layout/01` 第 149 行「5000（单位毫秒，即 5 秒）」（第二节）。这一次互比（N23 对 `crates/`、N23 对 N22R、第 24 次对第 23 次、第 24 次对第 22 次第二段）旁边的绝对值：Q142.107（T_time = 5000，K7、A8）、Q142.108（标签 6 = 98 386，K1、A1）、Q142.106（B1、B1b）、Q142.113（A9 的两个区域，A7 的两段偏移）。7.3 同第 23 次。

## 八、轨迹与几何敏感性

### 8.1 轨迹

不适用，理由同第 23 次 8.1：这一次的量都不是哪条条款的停机 / 起跑 / 准入谓词的输入；装置是确定性模型，没有轮次。

### 8.2 几何敏感性（`mutation-sampling.md` 第六类）

| 取样点 | 方向 | 判哪一格 | 这一次 |
|---|---|---|---|
| 主点（两块 4 GiB 盘） | — | Q142.105、Q142.106、Q142.107 主点、Q142.117、Q142.119 | 重判 |
| P8G（两块 8 GiB 盘，其余同主点） | 每盘容量加倍 | 第 23 次 Q142.109–Q142.112 | **不重判**，引第 23 次读数（第 23 次判「两点同判」）。那份读数今天还覆不覆盖：装置那一侧由 Q142.125 管（`p8g_` 各行在归一化范围里），`crates/` 那一侧由 Q142.126 管。两样都「没变」⇒ 第 23 次的两点同判照引；有一样变了 ⇒ 这一格记「欠着」，交主 agent（不自己补跑，派发提示定第 2 行不重判） |

**判别力自证**：第 23 次第八节那一步（两点同判时把门槛挪到两点之间、检查必须一边绿一边红）要两个点都在这一次判过才做得成；这一次只有主点重判，不做，交回时写明「没做」。

## 九、变异

**这一次不加变异，变异表不改**（210 行，`ef63ea85…f1c3`），装置不改。

**⚠️ 缺口，照实写（同第 23 次第九节）**：T_time 的值在装置里没有单测钉、也没有变异；把第 1524 行的 5000 改回 5 的变异，整张表的单测抓不到，盯它的只有产物上的 Q142.107 与 Q142.105。新加的两份草稿脚本只有合成自检（PC-q123、PC-c124）与各一个弄坏开关，不进变异表。

**整张表对冻结装置跑一次**（210 条，Q142.120；第 23 次没跑）：照 `mutation-sampling.md`「改了一个格式常量之后，要看「无效」那一栏有没有变多」报七栏，几栏之和等于 210；锚点不再恰好命中一次的（第七类）、替换编不过的（第八类）逐条列出交主 agent（不许改装置，不重锚，记 F24l）；没红的按「没红的三类，判据不同」三分。独立 bin 的变异表不动、不重跑。

**要主 agent 跑的命令（整张变异表，超过前台时限）**：执行员在步 ④ 把下面这条原样写进报告，主 agent 起；放在步 ② 主产物之后跑（装置源码在两者之间不许被别人改，`mutate.sh` 退 5 的那一轮不算数）。工作目录 `/home/fy5090/code/singlefs/research`；内存上限 3G；线程数取派发给执行员的上限（没给取 6）；4 个工作进程。

```
cd /home/fy5090/code/singlefs/research && RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="E142 第 24 次整表：装置是第 23 次冻结版，旧峰值照用" MUTATE_MEMORY_MAX=3G MUTATE_JOBS=4 MUTATE_TARGET_DIR=<执行员草稿目录>/mtarget nice -n 19 bash scripts/capped.sh <线程数> bash scripts/mutate.sh e142-new-pool-file-creation-dry-run scripts/experiments/e142_new_pool_file_creation_dry_run.rs mutations/e142_new_pool_file_creation_dry_run.tsv > results/e142-mutate-r24-<日期>.log 2>&1
```

跑完看：`grep -E '^(✅|⏭|❌|💥|⚠️|⏱|🧱)' results/e142-mutate-r24-<日期>.log | awk '{print $1}' | sort | uniq -c`（几栏之和等于 210）、收尾的「计数：」行、末行「已还原，基线仍全绿」；没有那一行或退出码是 5 ⇒ 这一轮不算数（`mutation-sampling.md` 第七类）。`⏭` 与 `research/results/e142-mutate-r22-s2-2026-10-05.log` 比。跑完 `mtarget` 可删。

## 十、失败条款

每条都是正当结果，照实记，不作废；每条后面紧跟「什么观测会让它触发」。F24g（P8G 那一行）与 F24k 这一次没有对象，号空着不用。

| # | 条款 | 什么观测会让它触发 | 触发之后 |
|---|---|---|---|
| F24a | 条款可能错：装置照定案写，写出的字节却与 7.1 的期望不同；或两侧一起偏离定案的数 | Q142.107 主点装置一侧有一行不是 `t_time=5000`，或四行相等而都不是 5000；Q142.108 不符而 Q142.113、Q142.123 都过；7.1 K6–K8 与 7.2 A7–A9 不符 | 同第 23 次 F23a 的「两边都查」；装置这一次不许改，查出装置错的，记下、交主 agent，改了就是又一次跑 |
| F24b | 第 1 行判「不同」 | Q142.105 不同区域数 ≥ 1；或 Q142.107 装置一侧是 5000 而 `crates/` 一侧不是；或 Q142.106 t13 ≠ 步 ③(1) 现读的钉值 | 按 `region=` 逐个列出不同区域与 `fields=`，逐个照 R84 归因（在不在 Q142.124 (2) 变了的区域里，连同 Q142.124 (1) 的文件）；同时走停机 S24-crates；第 1 行照答「哪几个区域不同」 |
| F24c | 改动碰了不该碰的区域或字节 | Q142.113 `changed` ≠ 2，或变的不恰是 A9 那两个，或两个 `unmatched` 有一个不是 0；或四行 `TTIME_DIFF` 有一行 `outside_t_time_and_crc` ≥ 1 或 `changed_bytes=0` | 照 `evidence-discipline.md` 三步：先按最弱读法记一次输（点名哪几个区域、哪几个字节）；第 1、2 行不够判，停下交回 |
| F24d | 收过的守门仍打中 | Q142.123 `Q123_VERDICT pass=false`（`groups_outside_allowed=` 点名的组），或 `Q123_PAIR` 有一行不是 `changed_keys=sha256`、`new_equals_…=true`、`old_equals_…=true`；或 Q142.108 不符 | 同 F24c 三步：按最弱读法记一次输（「臂 N23 的『做完会怎样』在这一次登记的形态下仍不成立」），照实列 `Q123_GROUP` 与 `Q123_PAIR` 各行；第 1、2 行不够判。**不许**再把形态放宽后在这一份产物上重判：要放宽就是又一次跑 |
| F24e | 有归不了因的差异组 | Q142.123 的归因表里有一组记「未归因」（变了而说不出怎么从第 1524 行来） | 照实列那一组的行名、变几行、第一行与最后一行（归一化之后）；第 1、2 行不够判 |
| F24f | 反向链与独立锚点不符，而与 `crates/` 相同 | Q142.106 t13 `back_chain=` 等于钉值、不等于 B1，或 w4 不等于 B1b | 同第 23 次 F23f |
| F24h | 判决翻面，而且只在第 24 次变、`crates/` 那一侧没变 | Q142.119 有一个判决字段在第 22 次第二段与第 24 次之间变了值，第 23 次主产物里没变，且 Q142.124 (1)(2) 都没变；或 `name=width` 有一行 `expected ≠ actual` | 逐个列出（行、字段、各次的值）；第 2 行 (c) 照答「不成立」；归不到 ⇒ 同时记 F24e |
| F24i | 字段表不等 | Q142.117 `unequal` ≥ 1 或 `rows` ≠ 174 | 同第 23 次 F23i；第 2 行 (a) 照答「不成立」 |
| F24j | 装置里还剩旧值 | 命令七 `count` ≥ 1，或归类表里有一处指 fsid / MAC；命令十二前两个计数有一个 ≥ 1；命令六四个计数有一个 ≥ 1 | 装置这一次不许改：列出每一处，命令七与命令十二那一半让第 2 行 (b)「不成立」，命令六那一半进附表；停下交回 |
| F24l | 变异没抓全 | Q142.120 `❌` 或 `⏭` ≥ 1 | 同第 23 次 F23l；第 2 行 (d) 照答「不成立」 |

## 十一、作废条款与停机条款

### 作废（这一次不出结论，产物改名 `-void.out` 留在执行员草稿目录、不入库，修好重跑；已经入库的上一次产物照留）

| # | 条件 | 什么观测会让它触发 |
|---|---|---|
| V24a | 同一份装置源码在今天跑出与第 23 次不同的装置那一侧，或同一份 `crates/` 跑出不同的导出（环境、依赖或编译器） | Q142.125 `changed_lines_total` ≥ 1 或 `order_same=false`，而 Q142.124 (1) 为空；Q142.124 (1) 为空而 (2) `identical=false`、仓根 `Cargo.toml` / `Cargo.lock` 也没变；Q142.105 有一个不同区域不在 Q142.124 (2) 变了的区域里 |
| V24b | 锚点的输入或算法坏了 | PC-h1、PC-h2 不过；命令十一不复现 7.2 A7–A9；主产物 t13 `back_chain=` 既不等于 B1 也不等于钉值 |
| V24c | 跑的途中 `crates/` 变了 | 快照 A 与 B、或 B 与 C `cmp` 不同 |
| V24d | 阳性对照不过：测量分不出差别，或配不上对 | PC-q123、PC-c124、PC-q123b、P2、PC-t、PC-e、PC-h1、PC-h2、PC-j 前两格、PC-g23 任一条没看到「该看到什么」（PC-i、PC-twelve 不过照第 23 次 5.2 那两行办，不作废）；`Q123_VERDICT ambiguous` ≥ 1；`CRATES_REGION_SUMMARY ambiguous` ≥ 1 |
| V24e | 产物缺行 | 同第 23 次 V23e 的主产物清单（主点与 P8G 两段都核）；`Q123_COUNT` 有一行不是 `rows=29`；`ttime_anchor.py --decode` 主产物行名缺省时不是恰两行装置一侧、两行 `crates/` 一侧；`label6_anchor.py --decode` 主产物同样 |
| V24f | 读数读不到，或内存包装自己的结局 | 任一格的量读出来是空的（读不到 ≠ 读到 0，脚本少打一行也算）；或经 `scripts/run-with-memory-cap.sh` 跑出退出码 250–254 |

### 停机（既不作废，也不当结果；两边都查，写进报告交回主 agent）

| # | 条件 | 什么观测会让它触发 | 怎么办 |
|---|---|---|---|
| S24-open | 开跑条件不齐 | 起跑前提 P1、P2 有一条不过 | 一步都不跑，交回缺哪一样 |
| S24-sha | 装置、表、参照产物或草稿脚本不是登记时那一份 | P3、P5 有一个不等；草稿脚本照全文重建两次 sha256 仍不等 | 不改源码、不跑；交回现值 |
| S24-admission | 准入不放行 | 装置二进制退 77、3 或 2 | 原样交回 stderr；不设强制与破闸变量，不改登记表 |
| S24-clause | 条款在第 23 次之后变了 | P4 或步 ②(1) 重抄的出口 sha256 不是 `6cc2aae5…c8c1` | 停在那一步；交回 `diff`；臂的定义要不要改由主 agent 按 `evidence-discipline.md` 三步定 |
| S24-scope | 要改装置才做得成 | 步 ②(2) 单测不是 117 过、0 败、1 忽略；或为过准入、过编译要动装置的任何一处 | 停在步 ②；交回哪一条单测、为什么；两行都不够判 |
| S24-dump | 导出 bin 要改才跑得了，或被重型闸拒 | `driver_e142` 第一条命令或 `driver_e142_p8g_crates_dump` 编不过、退非 0，或 `.claude/hooks/heavy-test-guard.sh` 拒了 | 不改 `crates/`、不换写法绕过；交回原样输出 |
| S24-crates | 装置与 `crates/` 对不上（`implementation-first.md` 第 4 条） | Q142.105 不同区域数 ≥ 1；Q142.107 两侧值不同；Q142.106 t13 与钉值不等；第三节 3.1 点名的行与 `crates/` 现读不符 | 两边都查：写明定案原文哪一行、装置哪一行、`crates/` 哪个文件哪一行，连同 R84 的归因；不为了相等改装置或锚点；第 1 行照答「不同」 |
| S24-fingerprint | 产物头的指纹已不是今天的输入 | Q142.116 准入不退 77 | 原样交回；列跑的途中谁改了哪份输入（`git status`、准入 stderr）；重不重跑由主 agent 定 |
| S24-drift | 装置那一侧在今天跑出与第 23 次不同的样子，同时 `crates/` 也变了，分不出是哪一个 | Q142.125 `changed_lines_total` ≥ 1（或 `order_same=false`），且 Q142.124 (1) 不为空 | 停在步 ③；交回 Q142.125 的 `Q123_GROUP` 各行与 Q142.124 两项的输出：是不是 R71 的剔除清单漏了一种读导出的行，由主 agent 判；这一次 Q142.123 不判 |

### 交回之前（每一次交回都做）

1. 第 24 次主产物里 `name=impl_bytes_equal_summary`、`name=root_record`、`name=back_chain`、`name=accounting`、`name=layout_field_summary`、`name=layout_field_positive_control`、四行 `name=positive_control_p2`、第一行 `name=verdict`、`name=p8g_verdict`；`ttime_anchor.py` 的全部 `TTIME` 与 `TTIME_DIFF` 行；`label6_anchor.py` 的全部 `DECODE` 行；独立 bin 的汇总行；Q142.123 与 Q142.125 的 `Q123 `、`Q123_COUNT`、`Q123_DIGEST`、`Q123_PAIR`、`Q123_GROUP`、`Q123_VERDICT` 各行；Q142.124 的 `CRATES_HALF`、`CRATES_GROUP`、`CRATES_REGION`、`CRATES_REGION_SUMMARY` 各行与命令十七的条数；Q142.126 的 `cmp` 输出——原样整行抄进报告（`evidence-discipline.md`「引产物就整行抄」）；Q142.123 的归因表、Q142.119 的翻面表、Q142.118 的归类表与命令十二、命令六的计数放进报告文件。
2. 问题单两行逐行写「已够判 / 还差什么 / 剩下的量能不能让它翻面」。问题单文件执行员不改，状态由主 agent 改。
3. 交主 agent 的清单（不改 kb 决策）：`layout/01` 第 36 行改指这一次主产物与它的 `impl_bytes_equal_summary`（第 1 行够判时）；第 23 次报告交回清单里那条 T_time 没有单测与变异的缺口仍在（第九节）；起跑前提 P1 记下的两个 sha256；Q142.124 (1) 列出的文件；Q142.126 不同时第八节那一格欠着。
4. `scripts/replay.sh` 第 174、175 行（步 ④，带条件）；实验页 `.claude/kb/experiments/142-新池新建文件的干跑.md` 加第二十四次跑一节；实验索引与变更史照执行员定义写；准入表 E142 行由主 agent 改，不由执行员改。

### 够判停机

问题单两行都够判（第六节 6.1）就停。Q142.126 只报不判；第 23 次第 2 行（P8G）各格、PC-f23、PC-t2、PC-h3、B1 / B1b 复算、Q142.115、Q142.121、Q142.122（第五节 5.2 的表）不在这一次跑；层 0 主臂枚举（`E142/layer0`）、甲二三态数、撕裂镜像、一盘几何（G13）、独立 bin 第一种调用、由记录重建的根与 t14 逐字节比、墓碑树非零根时 H 的读法，同第 23 次不在射程，实验页记「够判后未跑」。

## 十二、修订

（步 ②(2) 单测全绿之后、步 ②(4) 主产物之前由执行员写，只许收严或补臂；每条写明时点与依据。主产物落盘之后这一节不再写——见文件头准入一段。）

## 十三、读过的文件与跑过的命令

一位设计员（这一次的 experiment-designer）写成；草稿目录 `/tmp/claude-1000/e142-r24-designer/`。

### 13.1 读过的文件（行区间；grep 命中行也算读过）

- 规则（整份）：`.claude/agent-common.md:1-100`；`.claude/singlefs-ai-sop/rules/test-discipline.md:1-122`；`.claude/rules/three-way-inference.md:1-174`；`.claude/singlefs-ai-sop/rules/evidence-discipline.md:1-176`；`.claude/rules/mutation-sampling.md:1-96`；`.claude/rules/implementation-first.md:1-20`。
- 问题单：`research/prompts/e142-r24-questions.md:1-10`。第 23 次执行员报告 `/tmp/claude-1000/e142-r23-run/report.md:1-153`（整份，派发提示要求）。
- 第 23 次登记 `research/prompts/e142-r23-prereg.md:1-697`（整份）。
- 第 22 次登记 `research/prompts/e142-r22-prereg.md`：`R71` 的 grep 命中行（82、83、86、1009、1027、1028、1082、1087、1093、1608）；`^| **PC-` / `^| PC-` / `^| **P2` 的命中行（1040–1048，每行截到 400 字）；`^| V22e`、`**命令` 的命中行（1217、1277、1293、1307、1318、1449、1540、1553、1567、1577、1593，截到 250 字）。第 19 次登记 `research/prompts/e142-r19-prereg.md`：`R37\|Q142.56` 的命中行（45、82、308、341、355、359、367、471、616、1032）。
- `crates/`：`singlefs-core/src/system_configuration.rs` 里 `TIME_THRESHOLD\|time_threshold` 的命中行（86、291、292、523）；`singlefs-harness/tests/new_pool_file_creation_publish.rs` 里 `E142_BACK_CHAIN_OF_NEW_POOL_FILE_CREATION` 的命中行（80、566）；三份文件的 `git status --short` 与 `crates/` 的条数。
- 装置 `scripts/experiments/e142_new_pool_file_creation_dry_run.rs`：`:680-700`、`:5940-5956`、`:6306-6336`、`:7272-7296`、`:7490-7516`、`:7640-7656`；命中行：`T_time`（1524）；`write_list_row\|device_region_bytes` 一类（5951、5954、6267、6306、6310、6325、6332、7495、7511、7514、7601、7653、10189、10701、10725、10732、10746、10786、10801）；`p8g_` 一类（6092、6094、6102、6107、8155、8163、10725、10795、10798–10800、10815、10816）；`admission\|E7INPUT\|input_fingerprint`（7140、7142、7143）；`journal_header_write_digest`（7287、7293）；`"system_configuration_slot"`（694、6786、8365、8683、8689）；`fn tag(`（689、722、2437、4559、4585）；`sha256_hex` 与循环头（564、5949、6093、6237、6307、7508、7509、7605、7613、7660、10814）；`std::env::args`（7489、7599、7651）。
- `scripts/experiments/lib.rs:125-145` 与 `fn .*admission\|admission.py` 的命中行（90、108、129、133、135、147、157）。
- `scripts/replay.sh`：`:26`、`:172-177`、`:505-527`，`driver_e142\|REPLAY_MEMORY_CAP` 的命中行（26、174、175、510、512、521、522、524、1389、1428）。`.claude/gate.d/stage-inputs.tsv` 第 36 行（按空白切开的前 60 段）与 `^E142\|e142-r2` 的命中行（36、37，各截到 200 字）。`scripts/admission.py` 里写文件的命中行（1100、1101、2010、2011、2278、2279、2431、2432、2526、2527；因此没跑它）。`scripts/replace-once.py --help` 的输出。
- 第 23 次执行员草稿目录 `/tmp/claude-1000/e142-r23-run/`：文件名列表（前 40 行）、`norm.py` 与 `groupdiff.py` 全文、`frozen-s2.rs` 的 sha256。
- `research/results/`：**没读内容**；`ls | grep` 看了 E142 的文件名，命令一对七份算了 sha256。`.claude/kb/experiments/`、实验索引、实验变更史：没读。
- 草稿目录里自己写的文件与它们的输出；这份登记自己（回读核子串）。

### 13.2 跑过的命令（原样；输出是 2026-10-05 在仓根或草稿目录跑的原样输出）

命令六、七、十一、十二同第 23 次登记第十三节（命令七、六的全文在第 22 次登记第十三节），执行员把其中的草稿目录换成自己的。下面的 `D=` 是设计员的草稿目录，执行员重跑时同样换成自己的。

**命令一**（第三节的 sha256 与行数、起跑前提 P1 的三条 `grep`、P3 的参照产物 sha256、HEAD 版装置）

```
cd /home/fy5090/code/singlefs && D=/tmp/claude-1000/e142-r24-designer; A=scripts/experiments/e142_new_pool_file_creation_dry_run.rs; R=research/results; { echo '命令一'; sha256sum $A scripts/mutations/e142_new_pool_file_creation_dry_run.tsv scripts/experiments/e142_region_diff_independent.rs scripts/mutations/e142_region_diff_independent.tsv crates/singlefs-core/src/system_configuration.rs crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs research/prompts/e142-r23-prereg.md research/prompts/e142-r24-questions.md $R/e142-new-pool-file-creation-dry-run-r23-main-2026-10-05.out $R/e142-new-pool-file-creation-dry-run-r23-arm-n22r-2026-10-05.out $R/e142-r23-p8g-crates-dump-2026-10-05.out $R/e142-r23-crates-sha256-c-2026-10-05.txt $R/e142-new-pool-file-creation-dry-run-r22-s2-main-2026-10-05.out $R/e142-mutate-r22-s2-2026-10-05.log; wc -l $A scripts/mutations/e142_new_pool_file_creation_dry_run.tsv research/prompts/e142-r23-prereg.md; grep -c 'const TIME_THRESHOLD_MILLISECONDS: u32 = 5000;' crates/singlefs-core/src/system_configuration.rs; grep -c 'TIME_THRESHOLD_SECONDS' crates/singlefs-core/src/system_configuration.rs; grep -c 'writer.put_u32(self.runtime.time_threshold_milliseconds());' crates/singlefs-core/src/system_configuration.rs; grep -c 'writer.put_u32(5000); // T_time 毫秒' $A; grep -c 'writer.put_u32(5); // T_time 秒' $A; git show HEAD:$A | sha256sum; git status --short crates/ | wc -l; git rev-parse --short HEAD; } > $D/cmd1.txt 2>&1; cat $D/cmd1.txt
命令一
43338ce525b4b3e8705b6f05c79a72bec5d8748222ed012aeadcc9df47805dff  scripts/experiments/e142_new_pool_file_creation_dry_run.rs
ef63ea858443d708868f3ca55659f62727e4c867d96b5c234bf4dbf3e1fcf1c3  scripts/mutations/e142_new_pool_file_creation_dry_run.tsv
311ce77c9c73148059f4d8a3425ab8902413c34e7ca45ddc04cc582de92fae51  scripts/experiments/e142_region_diff_independent.rs
8978e80a8a70a454f244a77f650eff04620016b7794042ed8388547933708a8e  scripts/mutations/e142_region_diff_independent.tsv
314660f247de8929d3d44bc9bcf18252c165dbb5fcf8d4a7573719cdb286729f  crates/singlefs-core/src/system_configuration.rs
a5b419cfe04f01c5f3f0ba382b1825e9f1abc4c472feb47c79caf3c258809715  crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs
2dcff2fa308fa5d2cae07b0972a5b982bf3c7e4ddb27a68e05dba279eec263bd  research/prompts/e142-r23-prereg.md
b9e5d4aa7c8e7c58bcbbcae6b6eb32270bfc438244ca427a61c27f719082fc47  research/prompts/e142-r24-questions.md
ea4e6f42019845243055575e8bb55d72f9d586cc1d429bebfcdbf411b4dedcc6  research/results/e142-new-pool-file-creation-dry-run-r23-main-2026-10-05.out
428d360cf318fa740b6c3e683b7747816b4526933d56998cfc26196a1ddd218b  research/results/e142-new-pool-file-creation-dry-run-r23-arm-n22r-2026-10-05.out
e7aa451ecf4ad7bf6c22ab66cc407eb980bbf22b75b9156f0369a145398cfb75  research/results/e142-r23-p8g-crates-dump-2026-10-05.out
4252e797b5cd032fcd15ee9a2b873660d64f5efa1a1b602aa3f23308553b07e9  research/results/e142-r23-crates-sha256-c-2026-10-05.txt
4b333e3f456e036a37743d71f5c4dcb76167efd8476f2b08b6e07397f18feaed  research/results/e142-new-pool-file-creation-dry-run-r22-s2-main-2026-10-05.out
0f3d459e50d7af16a26c6e817560dac04f93691028619b8dd87f58761208bc5c  research/results/e142-mutate-r22-s2-2026-10-05.log
 10866 scripts/experiments/e142_new_pool_file_creation_dry_run.rs
   210 scripts/mutations/e142_new_pool_file_creation_dry_run.tsv
   697 research/prompts/e142-r23-prereg.md
 11773 total
1
0
1
1
0
2f4ea2cfd3f142bbe76e054bfb6747ebe4cfd17509a19e6737f1dcf9ebd62999  -
136
31e47115
```

**命令二**（第二节的八个取法，照第 23 次命令二原样，只换出口名；第 149 行单抄一次进第二节；第一节问题单两段。起跑前提 P4 与步 ②(1) 照第一段重抄进另一个出口，与 `6cc2aae5…c8c1` 比，不同就 `diff` 两份出口）

```
cd /home/fy5090/code/singlefs && D=/tmp/claude-1000/e142-r24-designer; L=.claude/kb/layout/01-first-txn.md; P='.claude/kb/decisions/16-发布语义.md'; V=research/prompts/m3-formal-code-b29-r1-main-verification.md; Q=research/prompts/e142-r24-questions.md; { echo '命令二'; nice -n 19 python3 scripts/quote-kb.py $D/e142-r24-clause-recheck.md "$P@#### 已定项 5：\`T_time\` 与 \`T_dirty\` 取多少，丢失窗口的口径" "$L:40-41" "$L:81-81" "$L:83-83" "$L:149-149" "$L:158-158" "$V:80-80" "$V:95-95"; echo "exit=$?"; wc -l $D/e142-r24-clause-recheck.md; sha256sum $D/e142-r24-clause-recheck.md; echo '命令二·第 149 行'; nice -n 19 python3 scripts/quote-kb.py $D/e142-r24-section2.md "$L:149-149"; echo "exit=$?"; wc -l $D/e142-r24-section2.md; sha256sum $D/e142-r24-section2.md; echo '命令二·问题单'; nice -n 19 python3 scripts/quote-kb.py $D/e142-r24-questions-quote.md "$Q:5-5" "$Q:7-10"; echo "exit=$?"; wc -l $D/e142-r24-questions-quote.md; sha256sum $D/e142-r24-questions-quote.md; } > $D/cmd2.txt 2>&1; cat $D/cmd2.txt
命令二
  ✓ 8 段整抄进 /tmp/claude-1000/e142-r24-designer/e142-r24-clause-recheck.md，回读逐字节一致
exit=0
72 /tmp/claude-1000/e142-r24-designer/e142-r24-clause-recheck.md
6cc2aae548cb709e486779dd5b4723ca39fef59f40594eceda7c9d17c211f8c1  /tmp/claude-1000/e142-r24-designer/e142-r24-clause-recheck.md
命令二·第 149 行
  ✓ 1 段整抄进 /tmp/claude-1000/e142-r24-designer/e142-r24-section2.md，回读逐字节一致
exit=0
5 /tmp/claude-1000/e142-r24-designer/e142-r24-section2.md
f025243e81b9cb345423ab678083a64a33c94271a8ccba9c1d7dba7cf6089c3d  /tmp/claude-1000/e142-r24-designer/e142-r24-section2.md
命令二·问题单
  ✓ 2 段整抄进 /tmp/claude-1000/e142-r24-designer/e142-r24-questions-quote.md，回读逐字节一致
exit=0
14 /tmp/claude-1000/e142-r24-designer/e142-r24-questions-quote.md
1de5d9a78aca5d9006fc428deeca577421901096f7e29a0766efc97c58576654  /tmp/claude-1000/e142-r24-designer/e142-r24-questions-quote.md
```

**命令十五**（PC-q123、PC-c124：两份新脚本的 sha256、行数、合成自检与弄坏开关；在草稿目录跑，两份脚本全文在 13.3）

```
cd /tmp/claude-1000/e142-r24-designer && { echo '命令十五'; sha256sum q123_guard.py crates_half_compare.py; wc -l q123_guard.py crates_half_compare.py; nice -n 19 timeout 60 python3 q123_guard.py --selftest; echo "exit=$?"; Q123_GUARD_BREAK=1 nice -n 19 timeout 60 python3 q123_guard.py --selftest | grep -E 'bad_sha|all_ok'; nice -n 19 timeout 60 python3 crates_half_compare.py --selftest; echo "exit=$?"; CRATES_HALF_BREAK=1 nice -n 19 timeout 60 python3 crates_half_compare.py --selftest | grep -E 'slot_outside|all_ok'; } > cmd15.txt 2>&1; cat cmd15.txt
命令十五
9f6f9decbc61ba7a646377597da07be1244b01849cead338040c700d51225045  q123_guard.py
59c8e8688e0e7e9d0896f87d3df58158056c15c2cc7efd951402c88977ede889  crates_half_compare.py
  256 q123_guard.py
  142 crates_half_compare.py
  398 total
PC_Q123 sample=good pass=true expected=true ok=True changed_lines_total=10 groups_outside_allowed=none
PC_Q123 sample=same pass=false expected=false ok=True changed_lines_total=0 groups_outside_allowed=device_region_bytes,journal_header_write_digest,p8g_device_region_bytes,p8g_write_list_row,write_list_row
PC_Q123 sample=bad_extra_column pass=false expected=false ok=True changed_lines_total=10 groups_outside_allowed=write_list_row
PC_Q123 sample=bad_sha pass=false expected=false ok=True changed_lines_total=10 groups_outside_allowed=write_list_row
PC_Q123 sample=bad_third_row pass=false expected=false ok=True changed_lines_total=11 groups_outside_allowed=write_list_row
PC_Q123 sample=bad_other_group pass=false expected=false ok=True changed_lines_total=11 groups_outside_allowed=segments
PC_Q123 sample=bad_digest_kind pass=false expected=false ok=True changed_lines_total=11 groups_outside_allowed=journal_header_write_digest
PC_Q123 sample=bad_digest_missing pass=false expected=false ok=True changed_lines_total=9 groups_outside_allowed=journal_header_write_digest
PC_Q123 sample=bad_e2_not_removable pass=false expected=false ok=True changed_lines_total=11 groups_outside_allowed=verdict
PC_Q123 all_ok=True
exit=0
PC_Q123 sample=bad_sha pass=true expected=false ok=False changed_lines_total=10 groups_outside_allowed=none
PC_Q123 all_ok=False
PC_C124 sample=identical ok=True
PC_C124 sample=slot_t_time_only ok=True
PC_C124 sample=slot_outside ok=True
PC_C124 sample=extra_region ok=True
PC_C124 sample=other_row ok=True
PC_C124 all_ok=True
exit=0
PC_C124 sample=slot_outside ok=False
PC_C124 all_ok=False
```

**命令十六**（起跑前提 P5：仓里装置把第 1524 行换回第 22 次第二段的旧行，sha256 应是 `c400139b…d899`，`diff` 恰一处；执行员把 `$D` 换成自己的草稿目录，生成的 `frozen-s2.rs` 也给 PC-twelve 用）

```
cd /home/fy5090/code/singlefs && D=/tmp/claude-1000/e142-r24-designer; A=scripts/experiments/e142_new_pool_file_creation_dry_run.rs; { echo '命令十六'; sed '1524s|.*|        writer.put_u32(5); // T_time 秒|' $A > $D/frozen-s2.rs; sha256sum $D/frozen-s2.rs; diff $D/frozen-s2.rs $A; echo "diff_exit=$?"; } > $D/cmd16.txt 2>&1; cat $D/cmd16.txt
命令十六
c400139b8cfc7c0e4b96d483200055e9891252c9171bbbeb06771046bf63d899  /tmp/claude-1000/e142-r24-designer/frozen-s2.rs
1524c1524
<         writer.put_u32(5); // T_time 秒
---
>         writer.put_u32(5000); // T_time 毫秒
diff_exit=1
```

**命令十七**（Q142.124 (1)：第 23 次快照 C 与第 24 次快照 A 的差；设计时还没有快照 A，没跑，执行员在步 ⓪(3) 跑。每行是 `<` 或 `>` 加路径：同一路径两边都有 = 改了，只有 `<` = 第 24 次少了，只有 `>` = 第 24 次多了；第二行打条数）

```
cd /home/fy5090/code/singlefs && R=research/results; diff $R/e142-r23-crates-sha256-c-2026-10-05.txt $R/e142-r24-crates-sha256-a-<日期>.txt | grep -E '^[<>] ' | awk '{print $1, $3}' > <执行员草稿目录>/q124-files.txt; wc -l < <执行员草稿目录>/q124-files.txt; cat <执行员草稿目录>/q124-files.txt
```

### 13.3 两份新草稿脚本全文

`q123_guard.py`（256 行，sha256 `9f6f9decbc61ba7a646377597da07be1244b01849cead338040c700d51225045`）：

```python
# E142 第 24 次差分重跑登记第六节 Q142.123（收过的守门）与 Q142.125（N23 没漂）共用的比对脚本；PC-q123 是它的合成自检。
# 读两份 driver_e142 产物，各取第一行 `E7RESULT name=done ` 之前、删掉 `E7INPUT ` 起头的行与空行，照第 22 次登记 R71 归一化与剔除：
#   每行 hexadecimal=<文本> 换成 hexadecimal_sha256=<那段文本按 ASCII 算的 sha256>；
#   剔除 impl_bytes_equal、impl_bytes_unmatched、impl_bytes_equal_summary、window_segments（只剔 side=crates 的）、window_segments_summary、done 六种，与 E1 self_check_text_numbers；
#   E2（self_check_text_number_unclassified 各行）与 E3（verdict 里 text_numbers_all_classified 那个键值）只在两份里每一行 E2 的 row= 之后都含 name=impl_bytes_equal 时剔除。
# 归一化之后按位置逐行比（行名序列必须相同），按行组（name= 的值）判收过的守门形态。不 import 装置、不读 crates/ 代码。
# 用法：python3 q123_guard.py <旧产物> <新产物>     Q142.123：旧 = 臂 N22R，新 = 第 24 次主产物；Q142.125：旧 = 第 23 次主产物，新 = 第 24 次主产物
#       python3 q123_guard.py --selftest          PC-q123：九份合成样本，逐份打判定与期望
# 弄坏开关 Q123_GUARD_BREAK=1：写清单行不再比「新值等于同一次写的 device_region_bytes 那一行的 sha256=」，自检 bad_sha 那一份必须因此判过、打 ok=False。
import hashlib
import os
import re
import sys
from collections import defaultdict

BREAK = os.environ.get("Q123_GUARD_BREAK") == "1"
DROP = {"impl_bytes_equal", "impl_bytes_unmatched", "impl_bytes_equal_summary", "window_segments_summary", "done", "self_check_text_numbers"}
REGION_TO_WRITE_LIST = {"device_region_bytes": "write_list_row", "p8g_device_region_bytes": "p8g_write_list_row"}
DIGEST = "journal_header_write_digest"
ALLOWED = set(REGION_TO_WRITE_LIST) | set(REGION_TO_WRITE_LIST.values()) | {DIGEST}
T15_SLOT = ("4096", "4096")
SYSTEM_CONFIGURATION_KIND = "system_configuration_slot"


def first_half(text):
    out = []
    for line in text.split("\n"):
        if line.startswith("E7INPUT "):
            continue
        if line.startswith("E7RESULT name=done "):
            break
        if line:
            out.append(line)
    return out


def name_of(line):
    match = re.match(r"E7RESULT name=(\S+)", line)
    return match.group(1) if match else "-"


def fields(line):
    return dict(token.split("=", 1) for token in line.split() if "=" in token)


def changed_keys(old, new):
    old_tokens, new_tokens = old.split(), new.split()
    if [t.split("=", 1)[0] for t in old_tokens] != [t.split("=", 1)[0] for t in new_tokens]:
        return None
    return {a.split("=", 1)[0] for a, b in zip(old_tokens, new_tokens) if a != b}


def e2_removable(lines):
    rows = [line for line in lines if name_of(line) == "self_check_text_number_unclassified"]
    return all(" row=" in line and "name=impl_bytes_equal" in line.split(" row=", 1)[1] for line in rows)


def normalize(lines, drop_e2_e3):
    out = []
    for line in lines:
        name = name_of(line)
        if name in DROP or (name == "window_segments" and " side=crates" in line):
            continue
        if drop_e2_e3 and name == "self_check_text_number_unclassified":
            continue
        if drop_e2_e3 and name == "verdict":
            line = re.sub(r" text_numbers_all_classified=\S+", "", line)
        line = re.sub(r"hexadecimal=(\S*)", lambda m: "hexadecimal_sha256=" + hashlib.sha256(m.group(1).encode("ascii")).hexdigest(), line)
        out.append(line)
    return out


def sha_table(lines, row_name):
    table = defaultdict(list)
    for line in lines:
        if name_of(line) == row_name:
            f = fields(line)
            table[(f.get("step"), f.get("device"))].append(f.get("sha256"))
    return table


def compare(old_text, new_text, expected_rows=29):
    out = []
    old_raw, new_raw = first_half(old_text), first_half(new_text)
    drop = e2_removable(old_raw) and e2_removable(new_raw)
    old, new = normalize(old_raw, drop), normalize(new_raw, drop)
    out.append(f"Q123 lines old={len(old)} new={len(new)} e2_e3_removed={str(drop).lower()}")
    counts_ok = True
    for row_name in sorted(ALLOWED - {DIGEST}):
        for side, lines in (("old", old_raw), ("new", new_raw)):
            got = sum(1 for line in lines if name_of(line) == row_name)
            counts_ok &= got == expected_rows
            out.append(f"Q123_COUNT side={side} name={row_name} rows={got} expected={expected_rows}")
    order_same = [name_of(l) for l in old] == [name_of(l) for l in new]
    out.append(f"Q123 order_same={str(order_same).lower()} counts_ok={str(counts_ok).lower()}")
    if not order_same:
        for row_name in sorted({name_of(l) for l in old + new}):
            a, b = sum(1 for l in old if name_of(l) == row_name), sum(1 for l in new if name_of(l) == row_name)
            if a != b:
                out.append(f"Q123_GROUP name={row_name} rows_old={a} rows_new={b} allowed=no problems=row_count")
        out.append("Q123 changed_lines_total=unknown")
        out.append("Q123_VERDICT pass=false ambiguous=0 groups_outside_allowed=order")
        return out, False
    pairs = defaultdict(list)
    for a, b in zip(old, new):
        if a != b:
            pairs[name_of(a)].append((a, b))
    ambiguous = 0
    problems = defaultdict(list)
    changed_writes = {}
    for region in REGION_TO_WRITE_LIST:
        devices, keys_seen = set(), set()
        for a, b in pairs.get(region, []):
            fa, fb = fields(a), fields(b)
            keys = changed_keys(a, b)
            if keys is None or not keys <= {"sha256", "hexadecimal_sha256"} or "sha256" not in keys:
                problems[region].append("columns")
            if (fa.get("offset"), fa.get("length")) != T15_SLOT or (fb.get("offset"), fb.get("length")) != T15_SLOT:
                problems[region].append("not_t15_slot")
            if (fa.get("step"), fa.get("device")) != (fb.get("step"), fb.get("device")):
                problems[region].append("pairing")
            devices.add(fa.get("device"))
            keys_seen.add((fa.get("step"), fa.get("device")))
        if len(pairs.get(region, [])) != 2:
            problems[region].append(f"changed_rows_{len(pairs.get(region, []))}_not_2")
        if devices != {"0", "1"}:
            problems[region].append("devices")
        changed_writes[region] = keys_seen
    digest_rows = [l for l in old if name_of(l) == DIGEST and fields(l).get("kind") == SYSTEM_CONFIGURATION_KIND]
    expected_indices = {fields(l).get("index") for l in digest_rows}
    changed_indices = set()
    for a, b in pairs.get(DIGEST, []):
        fa, fb = fields(a), fields(b)
        if changed_keys(a, b) != {"sha256"}:
            problems[DIGEST].append("columns")
        if fa.get("index") != fb.get("index") or fa.get("kind") != fb.get("kind"):
            problems[DIGEST].append("pairing")
        if fa.get("kind") != SYSTEM_CONFIGURATION_KIND:
            problems[DIGEST].append(f"other_kind_{fa.get('kind')}")
        changed_indices.add(fa.get("index"))
    if changed_indices != expected_indices:
        problems[DIGEST].append("not_exactly_the_system_configuration_rows")
    out.append(f"Q123_DIGEST system_configuration_rows_old={len(expected_indices)} changed={len(changed_indices)}")
    for region, write_list in REGION_TO_WRITE_LIST.items():
        old_table, new_table = sha_table(old, region), sha_table(new, region)
        seen = set()
        for a, b in pairs.get(write_list, []):
            fa, fb = fields(a), fields(b)
            key = (fa.get("step"), fa.get("device"))
            if changed_keys(a, b) != {"sha256"}:
                problems[write_list].append("columns")
            if key != (fb.get("step"), fb.get("device")):
                problems[write_list].append("pairing")
            seen.add(key)
            new_match, old_match = new_table.get(key, []), old_table.get(key, [])
            if len(new_match) != 1 or len(old_match) != 1:
                ambiguous += 1
            new_equal = len(new_match) == 1 and fb.get("sha256") == new_match[0]
            old_equal = len(old_match) == 1 and fa.get("sha256") == old_match[0]
            if not new_equal and not BREAK:
                problems[write_list].append("new_sha256_not_region_sha256")
            if not old_equal:
                problems[write_list].append("old_sha256_not_region_sha256")
            out.append(f"Q123_PAIR group={write_list} step={key[0]} device={key[1]} changed_keys={','.join(sorted(changed_keys(a, b) or {'shape'}))} new_equals_{region}={str(new_equal).lower()} old_equals_{region}={str(old_equal).lower()}")
        if len(pairs.get(write_list, [])) != 2:
            problems[write_list].append(f"changed_rows_{len(pairs.get(write_list, []))}_not_2")
        if seen != changed_writes[region]:
            problems[write_list].append("not_the_writes_changed_in_region_group")
    for row_name in sorted(set(pairs) - ALLOWED):
        problems[row_name].append("outside_allowed_groups")
    total = 0
    for row_name in sorted({name_of(l) for l in old}):
        changed = len(pairs.get(row_name, []))
        total += changed
        if changed or row_name in ALLOWED:
            verdict = "no" if problems.get(row_name) else "yes"
            out.append(f"Q123_GROUP name={row_name} rows={sum(1 for l in old if name_of(l) == row_name)} changed={changed} allowed={verdict} problems={','.join(sorted(set(problems.get(row_name, [])))) or 'none'}")
    out.append(f"Q123 changed_lines_total={total}")
    outside = sorted(name for name in problems if problems[name])
    passed = counts_ok and not outside and ambiguous == 0
    out.append(f"Q123_VERDICT pass={str(passed).lower()} ambiguous={ambiguous} groups_outside_allowed={','.join(outside) or 'none'}")
    return out, passed


def synthetic(variant):
    sha = lambda data: hashlib.sha256(data).hexdigest()
    old_slot, new_slot = bytes([0x53, 0x46, 0x53, 0x42, 5, 0, 0, 0]), bytes([0x53, 0x46, 0x53, 0x42, 0x88, 0x13, 0, 0])
    root = bytes([7] * 8)
    def product(slot30, slot31, slot32, wl_override=None, extra=None):
        lines = ["E7INPUT name=input_fingerprint key=E142 sha256=00 files=1", f"E7RESULT name=impl_bytes_equal step=31 equal={'false' if slot31 == old_slot else 'true'}"]
        for prefix in ("", "p8g_"):
            for step, device, offset, data in ((30, 0, 8192, slot30), (31, 0, 4096, slot31), (32, 1, 4096, slot32)):
                lines.append(f"E7RESULT name={prefix}device_region_bytes step={step} region=r device={device} offset={offset} length=4096 sha256={sha(data)} hexadecimal={data.hex()}")
                wl_sha = (wl_override or {}).get((prefix, step), sha(data))
                wl_bytes = (extra or {}).get(("bytes", prefix, step), 4096)
                lines.append(f"E7RESULT name={prefix}write_list_row step={step} writer=transaction unit=r class=other bytes={wl_bytes} device={device} offset={offset} sha256={wl_sha}")
        digest = (extra or {}).get("digest", {})
        for index, kind, device in ((0, "system_configuration_slot", 0), (1, "journal_record", 0), (2, "system_configuration_slot", 1)):
            lines.append(f"E7RESULT name=journal_header_write_digest index={index} kind={kind} device={device} offset=0 length=4096 sha256={digest.get(index, 'd' + str(index))}")
        lines.append(f"E7RESULT name=segments count={(extra or {}).get('segments', 3)}")
        lines.append(f"E7RESULT name=self_check_text_number_unclassified row={(extra or {}).get('e2', 'name=impl_bytes_equal_summary')} number=2")
        lines.append(f"E7RESULT name=verdict width_mismatches=0 text_numbers_all_classified={'false' if slot31 == old_slot else 'true'}")
        lines.append("E7RESULT name=done emitted=20")
        lines.append(f"E7RESULT name=device_region_bytes step=31 device=0 offset=4096 length=4096 kind=write sha256={sha(slot31 + b'c')} hexadecimal=00")
        return "\n".join(lines) + "\n"
    changed_digest = {0: "e0", 2: "e2"}
    old = product(root, old_slot, old_slot, extra={"digest": {}})
    if variant == "same":
        return old, old
    if variant == "bad_e2_not_removable":
        old = product(root, old_slot, old_slot, extra={"digest": {}, "e2": "name=segments"})
        return old, product(root, new_slot, new_slot, extra={"digest": changed_digest, "e2": "name=segments"})
    extra = {"digest": dict(changed_digest)}
    override = {}
    if variant == "bad_extra_column":
        extra[("bytes", "", 31)] = 4097
    if variant == "bad_sha":
        override[("", 31)] = sha(b"something else")
    if variant == "bad_third_row":
        override[("", 30)] = sha(b"something else")
    if variant == "bad_other_group":
        extra["segments"] = 4
    if variant == "bad_digest_kind":
        extra["digest"][1] = "e1"
    if variant == "bad_digest_missing":
        del extra["digest"][2]
    return old, product(root, new_slot, new_slot, wl_override=override, extra=extra)


def selftest():
    expectations = [("good", True), ("same", False), ("bad_extra_column", False), ("bad_sha", False), ("bad_third_row", False),
                    ("bad_other_group", False), ("bad_digest_kind", False), ("bad_digest_missing", False), ("bad_e2_not_removable", False)]
    all_ok = True
    for variant, expected in expectations:
        old, new = synthetic(variant)
        lines, passed = compare(old, new, expected_rows=3)
        total = next(l for l in lines if l.startswith("Q123 changed_lines_total="))
        verdict = next(l for l in lines if l.startswith("Q123_VERDICT "))
        ok = passed == expected
        all_ok &= ok
        print(f"PC_Q123 sample={variant} pass={str(passed).lower()} expected={str(expected).lower()} ok={ok} {total.split()[1]} {verdict.split()[-1]}")
    print(f"PC_Q123 all_ok={all_ok}")


def main():
    if sys.argv[1:] == ["--selftest"]:
        selftest()
    elif len(sys.argv) == 3:
        with open(sys.argv[1], encoding="utf-8", errors="replace") as a, open(sys.argv[2], encoding="utf-8", errors="replace") as b:
            lines, _ = compare(a.read(), b.read())
        print("\n".join(lines))
    else:
        raise SystemExit("用法：q123_guard.py <旧产物> <新产物> | --selftest")


main()
```

`crates_half_compare.py`（142 行，sha256 `59c8e8688e0e7e9d0896f87d3df58158056c15c2cc7efd951402c88977ede889`）：

```python
# E142 第 24 次差分重跑登记第六节 Q142.124 (2)：两份 driver_e142 产物里 crates/ 导出那一半（第一行 `E7RESULT name=done ` 之后）逐行组、逐区域比；PC-c124 是它的合成自检。
# 区域行 = 那一半里的 `E7RESULT name=device_region_bytes ` 行（导出侧，不带 region=），按（step, device, offset, length）配对，逐字节比十六进制；
# 设备内偏移 4096、长 4096 的那一格（t15 的系统配置槽 1，7.2 A9）另报变了的字节有几个落在 T_time [433,437) 与整槽校验和 CRC [155,159) 之外（偏移取 7.2 A7）。
# 不 import 装置、不读 crates/ 代码。用法：python3 crates_half_compare.py <第 23 次主产物> <第 24 次主产物>；python3 crates_half_compare.py --selftest
# 弄坏开关 CRATES_HALF_BREAK=1：系统配置槽那一格不再数两段之外的字节（恒报 0），自检 slot_outside 那一份必须因此打 ok=False。
import os
import re
import sys
from collections import Counter, defaultdict

BREAK = os.environ.get("CRATES_HALF_BREAK") == "1"
T_TIME, CRC = 433, 155
T15_SLOT = ("4096", "4096")


def second_half(text):
    out, seen = [], False
    for line in text.split("\n"):
        if not seen:
            seen = line.startswith("E7RESULT name=done ")
            continue
        if line and not line.startswith("E7INPUT "):
            out.append(line)
    return out, seen


def name_of(line):
    match = re.match(r"E7RESULT name=(\S+)", line)
    return match.group(1) if match else "-"


def fields(line):
    return dict(token.split("=", 1) for token in line.split() if "=" in token)


def regions(lines):
    table = defaultdict(list)
    for line in lines:
        if line.startswith("E7RESULT name=device_region_bytes ") and " region=" not in line:
            f = fields(line)
            table[(f.get("step"), f.get("device"), f.get("offset"), f.get("length"))].append(f.get("hexadecimal", ""))
    return table


def compare(old_text, new_text):
    out = []
    old, old_seen = second_half(old_text)
    new, new_seen = second_half(new_text)
    out.append(f"CRATES_HALF lines_old={len(old)} lines_new={len(new)} done_seen_old={str(old_seen).lower()} done_seen_new={str(new_seen).lower()} identical={str(old == new).lower()}")
    old_groups, new_groups = Counter(name_of(l) for l in old), Counter(name_of(l) for l in new)
    for name in sorted(set(old_groups) | set(new_groups)):
        a = [l for l in old if name_of(l) == name]
        b = [l for l in new if name_of(l) == name]
        changed = sum(1 for x, y in zip(a, b) if x != y) if len(a) == len(b) else "row_count"
        if changed != 0:
            out.append(f"CRATES_GROUP name={name} rows_old={len(a)} rows_new={len(b)} changed={changed}")
    old_regions, new_regions = regions(old), regions(new)
    matched = equal = unequal = ambiguous = 0
    for key in sorted(set(old_regions) | set(new_regions), key=lambda k: tuple(int(x) if x and x.isdigit() else -1 for x in k)):
        a, b = old_regions.get(key, []), new_regions.get(key, [])
        if len(a) > 1 or len(b) > 1:
            ambiguous += 1
            continue
        if not a or not b:
            continue
        matched += 1
        before, after = bytes.fromhex(a[0]), bytes.fromhex(b[0])
        if before == after:
            equal += 1
            continue
        unequal += 1
        changed = [i for i in range(max(len(before), len(after))) if i >= len(before) or i >= len(after) or before[i] != after[i]]
        in_slot = (key[2], key[3]) == T15_SLOT
        allowed = set(range(T_TIME, T_TIME + 4)) | set(range(CRC, CRC + 4))
        outside = 0 if BREAK else sum(1 for i in changed if i not in allowed)
        out.append(f"CRATES_REGION step={key[0]} device={key[1]} offset={key[2]} length={key[3]} equal=false changed_bytes={len(changed)} first_changed={changed[0]} in_t15_slot={str(in_slot).lower()} outside_t_time_and_crc={outside if in_slot else 'na'}")
    unmatched_old = sum(1 for k in old_regions if k not in new_regions)
    unmatched_new = sum(1 for k in new_regions if k not in old_regions)
    out.append(f"CRATES_REGION_SUMMARY matched={matched} equal={equal} unequal={unequal} unmatched_old={unmatched_old} unmatched_new={unmatched_new} ambiguous={ambiguous}")
    return out


def synthetic(variant):
    slot = bytearray(4096)
    slot[0:4] = b"SFSB"
    slot[433:437] = (5000).to_bytes(4, "little")
    unit = bytes(range(32))
    def product(slot_bytes, extra_region=False, config="a"):
        lines = ["E7RESULT name=verdict x=1", "E7RESULT name=done emitted=1", f"E7RESULT name=impl_config devices=2 note={config}"]
        lines.append(f"E7RESULT name=device_region_bytes step=31 device=0 offset=4096 length=4096 kind=write sha256=s hexadecimal={bytes(slot_bytes).hex()}")
        lines.append(f"E7RESULT name=device_region_bytes step=1 device=0 offset=822149120 length=32 kind=write sha256=s hexadecimal={unit.hex()}")
        if extra_region:
            lines.append(f"E7RESULT name=device_region_bytes step=40 device=1 offset=0 length=32 kind=write sha256=s hexadecimal={unit.hex()}")
        lines.append("E7RESULT name=done emitted=3")
        return "\n".join(lines) + "\n"
    old = product(slot)
    changed = bytearray(slot)
    if variant == "identical":
        return old, product(slot)
    if variant == "slot_t_time_only":
        changed[433:437] = (200).to_bytes(4, "little")
        changed[155] ^= 0xFF
        return old, product(changed)
    if variant == "slot_outside":
        changed[433:437] = (200).to_bytes(4, "little")
        changed[6] ^= 0x01
        return old, product(changed)
    if variant == "extra_region":
        return old, product(slot, extra_region=True)
    if variant == "other_row":
        return old, product(slot, config="b")
    raise SystemExit(variant)


def selftest():
    expect = {
        "identical": ("identical=true", "unequal=0", None),
        "slot_t_time_only": ("identical=false", "unequal=1", "outside_t_time_and_crc=0"),
        "slot_outside": ("identical=false", "unequal=1", "outside_t_time_and_crc=1"),
        "extra_region": ("identical=false", "unmatched_new=1", None),
        "other_row": ("identical=false", "unequal=0", "CRATES_GROUP name=impl_config"),
    }
    all_ok = True
    for variant, wanted in expect.items():
        text = "\n".join(compare(*synthetic(variant)))
        ok = all(w is None or w in text for w in wanted)
        all_ok &= ok
        print(f"PC_C124 sample={variant} ok={ok}")
    print(f"PC_C124 all_ok={all_ok}")


def main():
    if sys.argv[1:] == ["--selftest"]:
        selftest()
    elif len(sys.argv) == 3:
        with open(sys.argv[1], encoding="utf-8", errors="replace") as a, open(sys.argv[2], encoding="utf-8", errors="replace") as b:
            print("\n".join(compare(a.read(), b.read())))
    else:
        raise SystemExit("用法：crates_half_compare.py <第 23 次主产物> <第 24 次主产物> | --selftest")


main()
```

### 13.4 写登记用的命令

这份登记用 `set -o noclobber` 排他新建、之后每次不超过 150 行 `>>` 追加；第一节与第二节的抄录、第十三节的命令输出与两份脚本全文都是草稿目录里的文件原样追加。几处事实更正用 `scripts/replace-once.py` 定点替换（各「命中 1 次……回读确认」）：步骤表里草稿脚本的份数（九 → 八）、第四节第 3 条读过的第 22 次登记行号（两次）、第八节判别力自证那一句。追加完回读：两份抄录出口、命令一、二、十五、十六的输出文件与两份脚本整份是这份登记的子串；从登记里按 13.3 截出的两份脚本 sha256 与命令十五记的相同；文件头一行在。
