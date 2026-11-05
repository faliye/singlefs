# E16 重跑登记（第 7 次，差分）：journal 记录头 487 → 496 之后照今天的口径改装置，接准入重跑三行，出带准入键的新产物

写于 2026-10-29，装置改成 496 之前、这一次的任何产物之前。**差分重跑**：臂、几何点、批轴、判据、容差、作废与停机条款照上一次登记 `research/prompts/e16-r6-prereg.md`（第 6 次；已归档，取法 `git show 04ceaa904^:research/prompts/e16-r6-prereg.md`，sha256 `b08fe45be3c6c47cd5b8733ce42caa29f68d37fa84dde2db9b03bb88adafff4a`），它又照第 5 次登记 `research/prompts/e16-r5-prereg.md`（已归档，取法 `git show 31128273b^:research/prompts/e16-r5-prereg.md`，sha256 `3e858988461c9778562e011b0a8de45b1c7099b541a23de8bf12b6b7f09d9932`）连同那份第十二节的修订（Q3 / Q4 以预测比值为分母那一路为准，两路都报）。这里只写变了哪些输入、哪几格要重判、怎么判；没写到的「同上一次登记」。跑出数之后要改，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」三步走。

**装置写在哪（写死）：沿用 `scripts/experiment-apparatus/e16_fifth_run_peak_under_today_widths.rs`（独立手写计数模型，不与 `crates/` 共用代码），不是入库装置，不另起新装置。** 变异表 `scripts/mutation/tables/e16_fifth_run_peak_under_today_widths.tsv`；复跑行是 `scripts/experiment-apparatus/replay.sh` 里 `grep -n '^E16R5|'` 命中的三行（2026-10-29 在第 1611–1613 行），键照旧是 `E16R5`、`E16R5/pc3`、`E16R5/second-segment` 三把准入键（`.claude/gate.d/stage-inputs.tsv` 第 102–104 行）。原装置 `scripts/experiment-apparatus/e16_journal.rs` 与复跑表里两行 `E16|e16-journal|…` 一字不动。

**这一次的起点**：装置在工作区里已经有上一位执行员接准入的改动（未提交；`git diff -U0` 的改动块只落在第 902 行之后的 `this_apparatus_source_path`、`repository_root_above`、`preflight`、`RunMode`、`main` 与测试模块，第 900 行之前产出各行的函数一行没动），装置 1363 行、sha256 `a110f17e6edd7f0717451c002cfafe7b14618456694d13d0266f1a2b6c1e1997`；变异表 22 条、sha256 `4306a85c814c582ae37a642ae681a5713bd5357d453698a837ff0939703af5a4`。这一次在它们上面改，不撤那些改动。

**准入**：派发提示写「准入已判：E16R5 三把键已在准入登记里；这次重跑是装置输入变了（`JOURNAL_HEADER_BYTES` 487 → 496）加接准入，不是登记过的输入变动触发」。我没有另跑 `admission.py experiment`；三把键在改 496 之后会变，执行员在第六节 K2 那一步现算。

## 一、问题

英文名：fifth_run_peak_under_today_widths（照抄上一次登记，装置沿用）

**主 agent 给的问题（派发提示「关的是哪几条」一节，逐字）**：

> 执行员报告 `~/.local/share/singlefs/work/m3-close-b19-e16r5-run/result.md` 停在登记停机条款 S1：`crates/singlefs-format/src/lib.rs:252` 的 `JOURNAL_HEADER_BYTES` 今天是 496（提交 ff7e45d01），装置 `scripts/experiment-apparatus/e16_fifth_run_peak_under_today_widths.rs` 第 51 行与 r6 登记抄的是 487。主 agent 判：照今天的口径改装置成 496。照 r6 的先例写差分重跑登记（上两份 r5、r6 登记已归档，执行员从 git 取出过，用 `git log --all -- 'research/prompts/e16*'` 找），写明：496 之后哪几格的期望要重算、怎么算；三份产物与 `scripts/experiment-apparatus/replay.sh` 第 1611–1613 行的产物名怎么改；新产物头带准入键怎么判；执行员报告「登记落地清单」里没有专门单测的 7 格（Q2、Q4、Q6、8.1 轨迹、8.2 几何敏感性、V2、PC3 逐格对拍）各怎么判（补单测还是写明靠产物判）。

**问题单（`research/prompts/m3-close-b19-p5-forks.md`，sha256 `692299409667b4412527fb0a555b65a2a10c08f40b32fc1fd29a35546f97983a`，表头与第 3 行，quote-kb 机械抄）**：

**出处 `research/prompts/m3-close-b19-p5-forks.md:5-6`（整段抄，未转述）**

```markdown
| # | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
```

**出处 `research/prompts/m3-close-b19-p5-forks.md:9-9`（整段抄，未转述）**

```markdown
| 3 | C777 E16R5：甲 装置调准入、三行重跑出带准入键的新产物且 `admission.py replay-row-keys` 绿 ／ 乙 对不上 | 三份新产物头与 `scripts/experiment-apparatus/replay.sh` 那三行登记 | 三行都重跑、`replay-row-keys` 判过 | 开着 |
```

这份登记只回答问题单第 3 行。第 3 行问的是「装置接准入之后三行重跑，新产物头带不带对得上的准入键、`replay-row-keys` 绿不绿」；要先把 487 → 496 这一处输入跟上，产物才能出（S1 是产物的前提）。上一次登记的问题单 `research/prompts/m3-r1-batch9-reruns-questions.md` 第 4 行与第 5 次问题单 `research/prompts/e16-r5-questions.md` 三行的判定不在这一次重判，由 K5（去掉准入头之后逐字节比）判「沿用」还是「要重判」（第六节）。

**读法写死**（只有一种读法）：

| 名字 | 写死成 |
|---|---|
| 变了的输入（一） | `crates/singlefs-format/src/lib.rs:252` 的 `JOURNAL_HEADER_BYTES` 487 → **496**（提交 ff7e45d01）。装置读它的地方：第 51 行本地常量，只进第 887 行的运行期自洽检查、第 1026 行与第 1127 行两条单测；第 50、54、78 行是注释。产出的任何一行都不读它（第三节） |
| 变了的输入（二） | 上一位执行员接准入的改动（这一节上面「这一次的起点」）：`main` 在产出正文之前先打准入给的 `E7INPUT` 行。正文由第 900 行之前的函数产出，那一段没改 |
| 没变而要核的 | `JOURNAL_NAMED_ENTRIES_PER_RECORD` 照旧是 64：⌊(4096 − 496) ÷ 56⌋ = ⌊3600 ÷ 56⌋ = 64（第七节）。`crates/` 的新根段 364 → 373（`lib.rs:249`）装置不建模（`grep -nE 'NEW_ROOT\|\b364\b\|\b373\b'` 零命中），不进这一次 |
| 两个被比的取值 | 487：复跑表三行今天登记的三份 exact 产物（已归档，取法 `git show 59ce5a865^:research/results/<名>`，名字与 sha256 见第十一节的比对对象表）；496：今天改完的装置 |
| 「产物变不变」 | 新产物删掉 `^E7INPUT ` 开头的行之后，与对应的留存产物逐字节比（`cmp`）。**允许不同的集合是空的**：头宽不进任何产出行，所以除准入头之外一个字节都不许变 |
| 产物头 | 新产物最前面、正文之前的 `E7INPUT` 行。三份各一行，形如 `E7INPUT name=input_fingerprint key=<键> sha256=<64 位十六进制> units=<十进制>`，键按跑法：不带参数是 `E16R5`、`pc3` 是 `E16R5/pc3`、`second_segment` 是 `E16R5/second-segment` |
| 「三行都重跑」 | 三种跑法各出一份新产物，**pc3 也出**：上一次登记写的「pc3 若逐字节不变，不出新产物」这一次不适用，因为第 3 行要的就是三份带准入头的产物 |
| 批轴、流数轴、几何点、PC4 的批集合、两种误差分母 | 同上一次登记，一个不改 |

问题单第 3 行对应第六节 K1–K6；K5 另管「上一次登记的判定能不能沿用」。没有岔路以外的量要登记；第六节标「附带」的不在够判点里。

## 二、被测条款与它引的定义

派发提示给的被测条款有两处：`.claude/kb/checks-owed.md` C777 一行，与 E16 实验页 `.claude/kb/experiments/16-journal的角色WALvs意图日志.md` 第五次跑那一节。

- C777 一行连同那张表的表头，用 `scripts/kb/quote-kb.py` 抄在下面。
- 实验页「第五次跑」一节（第 95 行起）与「第六次跑」一节（第 120 行起）是那两次跑的结果与结论。照设计员定义，结果与结论小节我不读、不抄；这一次不重判那几格的结论（第一节「读法写死」最后两行），所以登记不靠那两节的内容定任何判据。那两节的标题行是在 `grep -n '^#'` 里看到的，列进第四节。
- 496 这个值出自 D23 已定项 4，整节抄在下面（它是第七节 7.1 锚点的出处）。上一次登记第二节抄的 D23 已定项 1、17 与 D25 已定项 1–5 这一次不重抄，同上一次登记。

产物 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-e16r5-design/q-clauses.md`（3 段，quote-kb 回读逐字节一致），整份追加如下。

**出处 `.claude/kb/checks-owed.md:19-20`（整段抄，未转述）**

```markdown
| # | 简称 | 要拦什么 | 怎么拦（会红的形态） | 前置 | 出处 |
|---|---|---|---|---|---|
```

**出处 `.claude/kb/checks-owed.md:485-485`（整段抄，未转述）**

```markdown
| C777 | 五个实验键在 stage-inputs.tsv 没有登记行 | E43（扩展点字节上限）、E49（反向链宽度 32 还是 64）、E116（打包容器的账·补元数据写与整理策略）、E157（并行线一两条条款的计数模型）、E16R5 在 `scripts/experiment-apparatus/replay.sh` 登记了 2026-10-04 的 h487 产物（`exact`），`.claude/gate.d/stage-inputs.tsv` 里没有它们的实验键（`cut -f1 .claude/gate.d/stage-inputs.tsv \| grep -E '^E[0-9]'` 只出 E142（新池新建文件的干跑） 的三个键），准入判不到它们的单元键与上次产物相同 | 先改 `scripts/experiment-apparatus/`：五个装置的 main 都没调 `admit_experiment_run_or_exit`（`grep -ln admit_experiment_run_or_exit scripts/experiment-apparatus/*.rs` 只出 `e142_*.rs` 与 `lib.rs`），只补登记行它们的复跑会被 replay.sh 判「跑不了」（登记了准入的键要求产物头记着那一实验的单元键）；E16R5 在 replay.sh 有三行共用一个键，`scripts/gate/admission.py replay-row-keys` 会红，要在 `ADMISSION_ROW_KEYS` 给三行各指一个键；装置调了 admit 之后存一份带头的新产物；然后才补五行。门禁格「replay.sh 登记了产物而 stage-inputs.tsv 没键的判红」范围要先定：replay.sh 有 158 个不同实验键、stage-inputs 只有 E142（新池新建文件的干跑） 一个，「没接准入、照跑」是合法状态，字面判据红 153 个，只能判产物头已带 `E7INPUT` 的那些（2026-10-13 工具实现员现查，`~/.local/share/singlefs/work/owed-c777-experiment-keys/report.md`） | 定案：接准入，重跑新产物（用户 2026-10-22 弹窗定「接准入，重跑新产物（推荐）」）；工具员在改五个装置与登记行（装置 main 调 `admit_experiment_run_or_exit`、补 stage-inputs.tsv 五行、E16R5 三行在 `ADMISSION_ROW_KEYS` 各指一个键），带头的新产物在验证段之后重跑；去向：里程碑三「待优化」表第 38 行（门禁与自证缺口）（五·11，2026-10-12 判）。 | `records/2026-09-30-里程碑三开工第一轮.md` 里「准入判不到」那一句；`records/2026-10-11-里程碑三知识腐烂治理.md`（2026-10-11 知识腐烂回扫） |
```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:150-179`（整段抄，未转述）**

```markdown
#### 已定项 4：记录头约束在单个原子单元内，头 496 字节

**定案**：**记录头完整落在一个原子单元内**（[invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) I-8.2（记录头不跨原子单元））。头 **496 字节**：

- **十个字段 78 字节**：magic 4 + 类型 2 + 算法类型 1 + 记录标志 1（位 0 = 本次发布末条，已定项 17；其余位写 0，读到非 0 当损坏） + 自述长度 4 + 点名项数 4 + `jsn` **10** + `checkpoint_txg` 8 + nonce 12 + 头部校验和 32。`jsn` 的 10 字节 = 实例代号 32 位 + 计数器 48 位（已定项 9）。`tail_lsn` 不在头里：已定项 3 逐字定了「tail 的权威副本住在一个固定位置，**不内联在记录头**」，而「内联在每条记录头的 `tail_lsn`」正是被否掉的那个 XFS 形态的定义。
- **三笔已定增量 17 字节，不在那十个字段的 78 里**：已定项 7 的事务号 + 提交标记 **9 字节**、已定项 8 的反向链 **4 字节**、已定项 13 的载荷校验和 **4 字节**。
- **本次发布内序号 4 字节**（无符号 32 位，从 1 起：一次发布切成 N 条记录时依次是 1..N，只有一条时是 1；空发布记录也写 1）：已定项 14 注 1 在所选根那条记录读不出时靠它接链首。4 字节的宽度与紧跟事务号、提交标记的落点是主 agent 按「不为省空间牺牲自包含」取的，可推翻。字段在头里的偏移：事务号 78、提交标记 86、本次发布内序号 87、反向链 91、载荷校验和 95、新根段 99、fsid 472、MAC 480、头末 496。一次发布的 N 条记录序号依次 1..N、与 jsn 同步；读者遇到这四格，都当那条记录损坏、断链即止，那次发布整体不施加：序号 0；一次发布之内跳号；锚点读得出时，下一次发布的首条序号不是 1（断在这一条）；同一 (实例代号, checkpoint_txg) 里带末条标志的那条之后还有记录（断在带标志的那条；「之后」按计数器读、不要求紧接着：计数器大于带标志那条的、读得出的同 (实例代号, checkpoint_txg) 记录都算，中间缺号不停；前提是同一实例里 journal 记录的 `checkpoint_txg` 随计数器不减，有这个前提时与「表里任何位置」的全表读法给出同样的施加结果）。与 I-8.9（一次发布的记录序号连续且只有末条带标志） 判的是同一件事。
- **已定项 15 的新根段 373 字节**（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8 + 实例表指针 86 + 分配记录树根指针 86 + 卸载记号 4 + 回收门槛 t 8 + 钉槽 p 1；t 与 p 是根记录里同一对字段，D22（单元原子性怎么合成） 已定项 7）。
- **fsid 8 与 MAC 16，都排在新根段之后**：
  - **fsid 8**（系统配置 fsid 的低 8 字节，与单元头那 8 字节同口径，D18（块里携带什么信息） 已定项 7）。不带的话 I-1.4（块头 fsid 一致） 要给 journal 记录写一条豁免，而全环扫描就再也挡不住「上一个文件系统留在这块盘上的记录」——mkfs 整环写 0 只在 mkfs 真跑完时成立，重 mkfs 中途崩掉就不成立。带 8 字节之后不用豁免。
  - **MAC 16**（放在 fsid 之后；nonce 12 与算法类型 1 十个字段里已有）。理由与根记录那 29 字节同一条（D22（单元原子性怎么合成） 已定项 7）：journal 记录头是自证结构，没有父指针带 MAC ⇒ 不 day-1 留位，开加密那天要么整条走明文豁免（泄漏 `checkpoint_txg`、新根段里的两条指针、点名项位置），要么就是一次布局变更；D9（加密） 已定项 3 的原则逐字是「为加密留的位所有卷都留、不挪作他用」。I-6（加密不变量）不列它。
- 登记值就是 496（`JOURNAL_HEADER_BYTES`）；十个字段的 78 另登记为 `JOURNAL_HEADER_TEN_FIELD_BYTES`。**说 78 时要写明指的是十个字段，连同这一句一起引。** 496 占 512 扇区的 97%，其后还余 **16 字节**（装不下一个点名项，每项 56 字节）；4096 上余 **3600**（64 个点名项，⌊(4096 − 496) ÷ 56⌋ = ⌊3600 / 56⌋）。
   <!-- format-const: JOURNAL_HEADER_BYTES = 496 stale=头 277 字节|的 277 字节头|合计 **277 字节**|JOURNAL_HEADER_BYTES: u64 = 277|JOURNAL_HEADER_BYTES: u64 = 84|JOURNAL_HEADER_BYTES: u64 = 86|JOURNAL_HEADER_BYTES: u64 = 78|512), 428|512), 426|512), 434|4096), 4012|4096), 4010|4096), 4018|512 上 428|512 上 426|头 86 字节|头 **307 字节**|登记值就是 307|⌊3789 / 56⌋|头 **311 字节**|余 **201 字节**|JOURNAL_HEADER_BYTES = 311|512 上余 201|头 **487 字节**|合计 **487 字节**|JOURNAL_HEADER_BYTES = 487|RECORD_HEADER_BYTES: u64 = 487|JOURNAL_RECORD_HEADER_BYTES: usize = 487|487 字节记录头|现行头 487|现行头是 487 -->
   <!-- format-const: JOURNAL_HEADER_TEN_FIELD_BYTES = 78 -->

**射程**：⚠️ **那个原子宽度不由本工程定**：D20（承重面：单元的原子性与自包含）已定自证单元依赖**运行时探测**的 `physical_block_size`，不许把 512 焊进格式。⚠️ **这个值是格式常量，改它要三处一起动**：kb 标记与正文、实验源码的 `const` 与钉死它的单测、重跑实验并更新产物；由门禁阶段「格式常量在 kb 与实验源码之间同步」（`.claude/gate.d/code-source-discipline.sh` 的 format-constants 格）判红。记录尺寸取 4 KiB 定长之后对齐浪费恒为 0（已定项 12）；变长记录的对齐代价表只对变长记录成立。

**依据**：

- E23（journal 几何）：十个字段（跑时还带 `tail_lsn`）的头装得进一个 512 单元，不对齐、多条记录挤同一个原子单元时除第一条外都判不了撕裂——头要完整落在一个原子单元内。
- E75（记录尺寸与环几何）：各个头部读法在 512 与 4096 两档 `physical_block_size` 上都装得进一个单元；定长 2 的幂尺寸下头永远不跨单元。
- fsid 8 与 MAC 16：用户弹窗定案，没有走三方论证，可推翻，原话在变更史。
- 本次发布内序号 4 字节：用户 2026-09-23 定（`research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 的 Z6），是推的、被攻过零轮，原话在变更史。
- 用户定案 2026-09-24：头 311 落地、末条标志放在原「填充 1」那个字节、序号的读者规则（原话在变更史）。
- 用户定案 2026-09-24：锚点读得出时首条序号不是 1、末条标志之后还有同 txg 的记录，两格都当损坏断链（实二十报告 `m2-lastflag-implementer-report.md` 第六节 2、3），原话在变更史；无实验：两格都要 CRC-32C 碰巧对上或改出来的镜像才走得到。
- 第四格的「之后」按计数器读、缺号不停，前提是同一实例里 journal 记录的 `checkpoint_txg` 随计数器不减：三方判决 `research/prompts/m3-formal-r2-main-verification.md` 第二节第 8 行与 `research/prompts/m3-formal-r3-main-verification.md` 第二节第四格补句一行，两条腿同向；前提另立不变量的判决是 `research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 9 行；用户定案（等前提判完一起写回），原话在变更史；无实验：补的是读者规则的射程，没有新的实验读数。
- 新根段加实例表指针、分配记录树根指针与卸载记号三项（头 311 → 487、点名项 67 → 64）：三方判决 `research/prompts/m3-ring-r1-main-verification.md` 第二节第 2 行（第一轮时的施加法在写行、首个文件、卸载三种发布上补出错根，推的）与第四节第 1 条，`research/prompts/m3-ring-r2-main-verification.md` 第二节第 2 行（首个文件、卸载两种发布补进去之后与当初会写的根逐字节相同、checker 绿，攻方在副本上量的），`research/prompts/m3-ring-r3-main-verification.md` 第二节第 1、2 行（两位放进记录标志字节的形态被专用清扫发布打穿；施加处不从新根段来的只有卸载记号、实例表指针、分配记录树根三项）。用户定案「回到甲」（原话在变更史）。卸载记号占 4 字节是第一轮正推腿 H1-b 的算法（86 + 86 + 4），判决没有单独定宽。无实验：补的是施加处的字节，数都在三方副本上量过，没立实验号。

**欠**：无（C94（登记的格式常量与后来的定案对不上） 2026-09-24 已还清，见欠账表已还清表）。

```

## 三、实现今天的样子

2026-10-29 现查。

| 事实 | 出处（文件:行） |
|---|---|
| 停机 S1 那条常量核对命令今天的输出（九行，第十三节命令二原样）：32768、134、147、144、4096、**496**、56、64、4294967296。与上一次登记相比值变了的只有头宽 487 → 496，七个行号随文件挪了 | `crates/singlefs-format/src/lib.rs:20`、`:38`、`:176`、`:179`、`:241`、`:252`、`:255`、`:258`、`:339` |
| `lib.rs` 自己写的点名项算法：「一条 4 KiB 记录装几个点名项：(4096 − 496) ÷ 56 的整数部分」，值 64 | `crates/singlefs-format/src/lib.rs:257-258` |
| 头宽的分项：「78 + 事务号 8 + 提交标记 1 + 本次发布内序号 4 + 反向链 4 + 载荷校验和 4 + 新根段 373 + fsid 8 + MAC 16」；新根段 373 | `crates/singlefs-format/src/lib.rs:251-252`、`:249` |
| extent 树几何与两个扇出常量值没变，几何点 G0–G5 的单元数、根层级、根下孩子数照第 5 次登记第七节 7.2 | `lib.rs:176`、`:179` |
| 装置是单线程（`grep -nE 'thread\|rayon'` 零命中） | 装置全文 |
| 装置在运行期按编译时的源文件路径往上三层取仓根，用那个仓根起 `preflight.py` 与准入 | 装置 `:905-916`、`:918-940`、`:1001` |

**装置里读头宽的每一处与这一次怎么动**（装置 1363 行，sha256 `a110f17e…`，`grep -n 'JOURNAL_HEADER_BYTES\|\b487\b'` 现查）：

| 行 | 是什么 | 这一次怎么动 |
|---|---|---|
| 50 | 注释「lib.rs:223 `JOURNAL_HEADER_BYTES`」 | 行号改成 `lib.rs:252` |
| 51 | `const JOURNAL_HEADER_BYTES: u64 = 487;` | **496** |
| 54 | 注释「(4096 − 487) ÷ 56 = 64。lib.rs:229」 | 「(4096 − 496) ÷ 56 = 64。lib.rs:258」 |
| 78 | 注释「今天 487 字节头」 | 「今天 496 字节头」 |
| 887、1026 | 自洽检查 `JOURNAL_NAMED_ENTRIES_PER_RECORD == (4096 − H) / 56` | 不动（496 时照样成立） |
| 1127 | 单测 `crates_format_literals_used_here_are_pinned_to_the_registered_values` 钉 `JOURNAL_HEADER_BYTES == 487` | 钉 496 |
| 40–56 其余注释里的 `lib.rs:153`、`:156`、`:191`、`:203` | 旧行号 | 可以顺手改成今天的 176、179、241、255，不要求 |

不动的：批轴、流数轴、几何点、两条臂、分类与汇总的全部函数（第 900 行之前）。

## 四、跑之前已经存在的数

| 数 | 在哪读到 / 怎么来的 | 对判据的影响 |
|---|---|---|
| 上两次登记里的全部数（第 5 次第七节 7.2 的锚点、第十二节修订里 PC2 的 10.63% / 25.79% 与 9.61% / 20.50%；第 6 次第四节、第七节与第十三节命令一的全部输出，含乙在 B = 63–73 上的块数、批轴 81 个取值） | 读上两次登记（判据怎么定的） | 这一次不重判 Q1–Q6，那些数照旧；第七节 7.2 拿第 6 次的批轴重算一遍 |
| 上一位执行员的报告：改 496 之前三把准入键 `56bd6f33…`、`d9fe3e25…`、`67298d84…`（units=133）；单测 27 条 30.62 秒；`replay-row-keys` 262 行 0 处 ✗、E16R5 三行 `yes`；转述的上一次挂钟（第一段 release 约 50 秒、第二段约 3 分 48 秒） | 派发提示点名的报告 `~/.local/share/singlefs/work/m3-close-b19-e16r5-run/result.md` | 三把键在改 496 之后一定变（装置源文件进键），不当期望值；K2 要的是改完之后现算的那一份。挂钟只用来估第十四节 |
| 三份留存产物的元数据（没读内容）：行数 3650、58、13366；sha256 见第十一节；每份 `grep -c '^E7INPUT'` 都是 0 | `git show 59ce5a865^:research/results/<名>` 接 `sha256sum`、`wc -l`、`grep -c` | 比对对象表要用；「留存产物没有准入头」是 K5 只删新产物那一边 `E7INPUT` 行的依据 |
| 实验页三语的标题行（`grep -n '^#'` 看到）：中文第 1 行写「2026-10-04 点名项 67 → 64 重跑：产物与 23 单测已出，变异表 18 条改前改后两次整表已跑，改后 18/18 抓到」；第 95 行「第五次跑：2026-09-28 的单元宽度下，峰值落点、预测式误差与适用范围句」；第 120 行「第六次跑：点名项 67 → 64（2026-10-04，第五次跑的装置与几何不动）」 | 找「第五次跑」那一节在哪 | 标题里的「18/18」是上一次变异表的结局，这一次整表 31 条重跑，不拿它当期望 |
| C777 那一行写的「E16R5 在 `scripts/experiment-apparatus/replay.sh` 登记了 2026-10-04 的 h487 产物」 | 第二节抄的条款 | 产物名的改法照 h487 的先例起 h496（第十一节） |
| **我算出来的（第十三节命令一）**：H = 487 与 H = 496 一条记录都装 64 项（余 25 与 16 字节）；装 64 项的头宽区间是 [457, 512]（456 装 65、513 装 63），496 离上沿 16、离下沿 39；第 6 次批轴 81 个取值上乙的块数随 487 → 496 变的批 **0 个**；甲的游标记录 ⌈496 ÷ 4096⌉ = 1 块 | 命令一原样输出 | ⚠️ 这就是「产物正文变不变」的答案：推的是一个字节都不变。K5 是对这个推断的核，不是在不知道答案时量的。判据照样写死：K5 不逐字节相同就停机 S2，不回头改允许集合 |

## 五、臂、阳性对照、真实基线

同上一次登记第五节，只换：

- 两条臂、几何点、流的落点、checkpoint 间隔一字不改。乙一次 fsync 照旧写 ⌈k ÷ 64⌉ 条记录；头宽 496 只让 64 这个数的来历从 (4096 − 487) 换成 (4096 − 496)。
- **PC1 / PC2**（原装置现跑与误差分母核定）这一次**不重跑**：原装置一字没改；标「够判后未跑」。
- **PC3**（新装置在原装置几何 G0 上逐格复现原装置）：`pc3` 跑法照跑、出新产物。逐格对拍这一次靠产物判：pc3 新产物去掉准入头之后与留存的 pc3 产物（第 5 次那一份，当时已与原装置现跑的产物逐字段比过）逐字节相同，就是逐格相同（K5）。另补一条单测 T7 钉 G0 上 S = 2、B = 1 那一格的绝对值（下表）。
- **PC4** 两条单测照旧（每个几何点 B = 1 时甲 − 乙 = L + 1；B ∈ {1, 2, 5, 10, 20, 50, 64, 65, 67} 的格比值 > 1），64 没变，期望不改。

**真实基线**照旧是 G2（1 TiB、今天的扇出、64 项一条）。

**登记落地清单里没有专门单测的 7 格，各怎么判**（执行员的清单要每格写出会红的用例；这一次补的单测如下，名字照写，在装置的测试模块里加）：

| 格 | 定法 | 补的单测（名字）与它钉什么 | 证红的变异（第九节） |
|---|---|---|---|
| Q2 二维最大值 | 补单测；产物上的判定沿用（K5） | T1 `two_dimensional_maximum_summary_flags_axis_endpoints_and_interior`：手搭三张小网格（几何用 G2 的常量，只当名字），比值最大的格分别在 ① S = 2、B = 2（批轴最大值是 4）② S = 2、B = 4 ③ 只在 S = 256。断言 `emit_two_dimensional_maximum_summary` 的输出依次含 `streams_interior=true batch_interior=true`、`batch_interior=false`、`streams_interior=false` | N1 |
| Q4 逐格误差清单 | 补单测；产物上的判定沿用（K5） | T2 `relative_error_summary_positive_fraction_counts_only_strictly_positive_errors`：S = 2 一行 4 格（B = 1、2、5、10，都在原 56 格里），两种误差都设成 0.1、0.0、−0.1、0.2。断言两行 `q4_summary` 都是 `positive_error_fraction=0.5000 cells=4` | N2 |
| Q6 ①②③ | 补单测；产物上的判定沿用（K5） | T3 `batch_beyond_streams_rules_report_rise_settle_and_dip_below_one`：S = 2，B = 1、2、3、4、32 上 ρ = 3.0、3.5、3.0、3.2、0.9。断言 ① `ratio_non_increasing_beyond_streams_holds == false`（3 → 4 涨 6.7%）② `settles_near_one_beyond_sixteen_streams_holds == true`、`settle_beyond_sixteen_streams_batch == 32` ③ `row_minimum_batch == 32`、`row_dips_below_one == true`；另一行同样的格只把 B = 32 改成 ρ = 1.0，断言 `row_dips_below_one == false` | N3 |
| 8.1 轨迹 | 补单测；产物上的判定沿用（K5） | T4 `row_trajectory_counts_rising_falling_and_opposite_steps_around_the_peak`：S = 2，B = 1–6 上 ρ = 3.0、3.2、3.5、3.3、3.4、3.0。断言峰值集合只有 B = 3，`rising_steps_before_peak == 2`、`falling_steps_after_peak == 2`、`opposite_direction_steps == 1`、`final_batch == 6`、`final_ratio == 3.0`；另断言 `cost_statistics(&[3, 5, 4]) == (4.0, 3, 5)` | N4 |
| 8.2 几何敏感性 | 补单测；产物上的判定沿用（K5） | T5 `geometry_sensitivity_reports_unstable_only_where_two_points_classify_a_row_differently`：两个几何名，流数轴每个 S 一行；甲行用格 (B = S, 3.0)、(B = 8S, 2.0)，乙行用 (B = S, 2.0)、(B = 4S, 3.0)、(B = 8S, 2.0)。第一个几何全是甲行，第二个只在 S = 3 是乙行。断言 `emit_geometry_sensitivity` 输出 21 行，只有 `streams=3` 那一行 `stable=false`，`stable=true` 数出 20 行 | N5 |
| V2 fsync 次数为 0、比值非数 | 一半补单测，一半靠产物 | T6 `cost_statistics_refuses_an_empty_fsync_sequence`（`#[should_panic(expected = "空的代价序列")]`）管「fsync 次数为 0」；「比值是 NaN 或无穷」靠产物：三份新产物 `grep -cE '=(NaN\|-?inf)( \|$)\| windows=0( \|$)'` 都是 0（K7） | N6 |
| PC3 逐格对拍 | 靠产物（K5）；补一条单测钉一格绝对值 | T7 `original_geometry_batch_one_cell_matches_the_independent_anchor`：`compute_peak_cell(&ORIGINAL_DEVICE_GEOMETRY, 2, 1, RecordFormat::LegacyOriginal)` 的甲最小 = 最大 = 7、平均 7.0，乙最小 = 最大 = 2、平均 2.0，比值 3.5（第七节 7.1、7.2） | N7 |

为什么 Q2–8.2 不全靠产物：这一次的产物判定靠 K5 沿用上一次，而执行员的落地清单要每一格有会红的用例，单测在装置以后再改时也盯得住这几处。为什么 PC3 不全靠单测：逐格对拍的参照是原装置的产物，把它的 56 格抄进单测就是把答案放进被测方，只钉一格独立算得出的。

**实现量与分段**：改 1 个常量、1 条单测的钉值与 3 段注释；补 7 条单测；变异表加 9 条（22 → 31）；三份产物；复跑表三行改第 4 栏；比对对象表三行；实验页三语另起一节、`experiments-history.md` 一条。一个执行员一次做完，不分段。

## 六、报哪些量与各自的判据

全部对应问题单第 3 行。每个量各占一行、各报各的判定，合取留给主 agent。计时类的量没有（纯计数模型）。门槛都不是臂的定义推出来的：K1–K4 的门槛是准入入口与复跑表的约定（`scripts/gate/admission.py` 用法里 `experiment` 与 `replay-row-keys` 两段），K5 的门槛是「允许不同的集合为空」。

**先后**（因为准入键要在装置最后一次改动之后、产物之前算，K3 要在落盘之后判，这几步串行）：改装置与单测 → 单测全绿 → 变异表整表 → S1 第二次核对 → K2 现算准入键并记下装置 sha256 → 出三份产物 → K1、K5、K7 → `cp` 进 `research/results/`、改复跑表三行第 4 栏 → K3、K4、K6。

| 量 | 怎么算 | 门槛与判定 | 第 3 行：取什么值让它翻面 |
|---|---|---|---|
| **K1 产物头的形态** | 每份新产物 `grep -c '^E7INPUT '`，与第 1 行原样 | 三份都是恰好 1 行 `E7INPUT`、在第 1 行，形如 `E7INPUT name=input_fingerprint key=<该跑法的键> sha256=<64 位小写十六进制> units=<十进制>`（`grep -cE '^E7INPUT name=input_fingerprint key=<键> sha256=[0-9a-f]{64} units=[0-9]+$'` 为 1）⇒ 「符」；缺行、多行、`sha256=refused`、键不是这一跑法的 ⇒ 「不符」，报原样那一行 | 任何一份「不符」⇒ 乙（产物头对不上） |
| **K2 产物头与跑前现算的键逐字相同** | 装置最后一次改动之后、出产物之前，`python3 scripts/gate/admission.py experiment /home/fy5090/code/singlefs <键>` 三把键各跑一次，stdout 存进草稿目录，同时记装置源文件的 sha256；产物出来之后拿产物的 `E7INPUT` 行与那份 stdout 逐字比 | 三份都逐字相同 ⇒ 「同」；不同 ⇒ 先比装置 sha256：变了是 V8（作废），没变才记「不同」 | 「不同」⇒ 乙（装置传的键或打的头与准入入口不一致） |
| **K3 落盘之后准入判「没变」** | 三份产物 `cp` 进 `research/results/`（第十一节的名字）、复跑表三行改完之后，三把键各跑一次 `admission.py experiment … <键>`，记退出码 | 三把都退 77 ⇒ 「认得」；退 0 ⇒ 「不认」，stderr 里点名的变了的单元原样贴出；退 2 / 3 ⇒ 报原样，不判 | 「不认」⇒ 乙（产物头上的键与今天的实验键对不上，下一次准入永远判不了「没变」） |
| **K4 `replay-row-keys`** | 复跑表改完之后 `python3 scripts/gate/admission.py replay-row-keys /home/fy5090/code/singlefs`，记退出码、`✗` 处数、E16R5 三行原样 | 退 0、0 处 ✗、E16R5 三行第 2 栏依次是 `E16R5`、`E16R5/pc3`、`E16R5/second-segment`、第 3 栏 `device`、第 4 栏 `yes` ⇒ 「绿」；否则「红」 | 「红」⇒ 乙；这一格就是第 3 行的够判条件「`replay-row-keys` 判过」 |
| **K5 去掉准入头之后逐字节比** | 三对：`cmp <(sed '/^E7INPUT /d' <新>) <(git show 59ce5a865^:research/results/<留存>)`（配对见第十一节比对对象表） | 三对都相同 ⇒ 「正文不变」，上一次登记 Q1–Q6、8.1、8.2、PC3 的判定原样沿用；任何一对不同 ⇒ 停机 S2，`diff` 的行数与头 20 行贴出 | 不翻第 3 行；它管的是「头宽 496 与接准入有没有碰到正文」。不同时第 3 行在主 agent 判 S2 之前不够判 |
| **K6 复跑表三行指到新产物** | `grep -n '^E16R5|' scripts/experiment-apparatus/replay.sh` 原样；第 4 栏的三个文件名在 `research/results/` 下 `ls` 得到 | 三行第 1–3 栏与改前逐字相同、第 4 栏是第十一节的三个新名、文件都在 ⇒ 「指到」；否则「没指到」 | 「没指到」⇒ 乙（第 3 行的翻面观测「三份新产物头与 replay.sh 那三行登记」对不上） |
| **K7 产物里没有非数与零次 fsync** | 三份新产物各跑下面那条 `grep -cE` | 三份都是 0 ⇒ 「无」；否则 V2（作废） | 不翻第 3 行，是作废闸 |
| **Q1–Q6、8.1、8.2 重判**（附带，够判后不跑） | 同上一次登记 | 同上一次登记 | K5 三对都相同时不跑，记「够判后未跑（正文与第 6 次逐字节相同，判定沿用）」；K5 不同、主 agent 判 S2 之后要重判的，照上一次登记第六节逐个几何点重判 |

K7 的命令（表格里的竖线是转义写的，这里是原样）：

```
grep -cE '=(NaN|-?inf)( |$)| windows=0( |$)' <新产物>
```

**够判点**：S1 过、单测全绿、K1–K7 都判完 ⇒ 第 3 行够判，按 K1、K2、K3、K4、K6 五格记甲或乙；K5 三对相同时 Q1–Q6 的重判标「够判后未跑」。

## 七、钉绝对值的断言

### 7.1 出自被测条款本身的（不符走上一次登记 F1 / F2，不作废）

| 断言 | 出处 |
|---|---|
| 头 **496** 字节；一条 4 KiB 记录装 **64** 个点名项 | D23 已定项 4「头 **496 字节**」「4096 上余 **3600**（64 个点名项，⌊(4096 − 496) ÷ 56⌋ = ⌊3600 / 56⌋）」（第二节） |
| 原装置几何上 S = 2、B = 1 那一格比值 3.50；预测式 (B + min(S, B) × L + 2) ÷ (B + 1) 在那一格是 3.500 | 第 5 次登记 7.1（实验页那张表与式子），T7 与已有单测 `predicted_ratio_uses_node_levels_not_node_levels_minus_one` 核 |
| 上一次登记 7.1 其余各条 | 同上一次登记 |

### 7.2 独立算出、用命令核过的（第十三节命令一、命令三；不符先核我这边的算术，核完仍不符 ⇒ 停机 S3）

| 断言 | 值 |
|---|---|
| C1：头 487 / 496 时一条记录装的点名项与余数 | 64 余 25 / **64 余 16** |
| C2：装 64 项的头宽区间 | [457, 512]；456 装 65，513 装 63 |
| C3：第 6 次批轴（81 个取值）上乙的块数随 487 → 496 变的批 | **0 个**；B = 63、64、65、66、67、68、70、72、73 两边都是 64、65、67、68、69、70、72、74、75 |
| C4：甲的游标记录块数 | ⌈496 ÷ 4096⌉ = 1，与 487 时相同 |
| C5（T7）：原装置几何 r = 3、L = 4；S = 2、B = 1 那一格甲 7、乙 2、比值 3.5、预测 3.5；原装置记录头 48 换成 4048 时乙 3 | 命令三 |
| 上一次登记 7.2 的其余各条（几何点、S = 2 时 B = 1 / 2 的甲乙、流落点、最底层节点序号、B2 的九个批） | 不读头宽，不改 |
| 停机 S1 的常量核对 | 第十三节命令二那九行 |

装置的本地常量照旧写成加法形式的回比断言。

## 八、轨迹与几何敏感性

8.1 同上一次登记（每行报峰值、上升 / 下降步数、期末值；每格报甲乙每次 fsync 块数的最小、最大、平均）。这一次不重判（K5 相同就沿用）；补的单测 T4 钉住轨迹那三样的算法。

8.2 同上一次登记那张旋钮表，加一行「头宽」：

| 旋钮 | 这一次的取样点 | 方向相反的点 | 判什么 |
|---|---|---|---|
| 头宽（只经「一条几项」进模型） | 496（今天） | 487（第 6 次产物，变窄的方向）；区间两端外的 456、513 由变异 N8、N9 与 C2 覆盖 | 「正文不变」这一判定：496 与 487 同在 [457, 512] 里，判「两点同判」；头宽跨出这个区间（513 装 63 项）时判定翻面，变异 N9 证明装置看得见这一步 |

判别力自证（`.claude/rules/mutation-sampling.md` 第六类）：把头宽挪到区间上沿之外（N9：496 → 513），自洽检查与单测 `entries_per_record_today_and_legacy` 必须由绿转红；挪回区间里面的另一点（N8：496 → 487）只有钉值的那条单测红，说明「正文不变」这一判定在区间里不靠钉值以外的任何断言撑着。其余旋钮（树高、扇出、记录宽）的几何敏感性同上一次登记，这一次不重判；补的单测 T5 钉住 `emit_geometry_sensitivity` 的翻面判法。

## 九、变异

**先后**：装置改完（496、7 条单测都加上）之后，在今天的源码上整表跑一次 `scripts/mutation/mutate.sh e16-fifth-run-peak-under-today-widths e16_fifth_run_peak_under_today_widths.rs ../mutation/tables/e16_fifth_run_peak_under_today_widths.tsv`（工作目录 `scripts/experiment-apparatus/`），留日志。原有 22 条（含上一位执行员加的 4 条准入条）与下表新加的 9 条，共 31 条，按 `.claude/rules/mutation-sampling.md` 的符号分栏数；31 条都是 `✅` 才写「31 条全抓」，「无效」「没红」都不算进去。新加的 9 条就是 T1–T7 与头宽钉值的证红。

| 条（变异表第一栏的名字） | 原文 → 替换文 | 在哪个取样点上改变输出 |
|---|---|---|
| N1「二维最大值的批轴内部判定取反」 | `let batch_interior = batches_upper != batch_axis_maximum;` → `==` | T1：网格 ① 由 `batch_interior=true` 变 `false`，网格 ② 反过来 |
| N2「正误差占比把零误差也算正」 | `.filter(\|cell\| selector(cell) > 0.0)` → `>= 0.0` | T2：误差为 0.0 的那一格被算进去，`positive_error_fraction` 由 0.5000 变 0.7500 |
| N3「跌破1的判定把等于1也算」 | `row_dips_below_one: minimum_cell.ratio < 1.0,` → `<= 1.0` | T3 第二行：B = 32 上 ρ = 1.0，`row_dips_below_one` 由 false 变 true |
| N4「峰值前段的上升步数漏掉落在峰值上的那一步」 | `if next.batch <= peak_batch_low {` → `<` | T4：B = 2 → 3 那一步不再计入，`rising_steps_before_peak` 由 2 变 1 |
| N5「几何敏感性恒判两点同判」 | `classes.windows(2).all(\|window\| window[0].1 == window[1].1)` → `classes.windows(2).all(\|_window\| true)` | T5：`streams=3` 那一行由 `stable=false` 变 `true` |
| N6「空的fsync代价序列不报错」 | 第 347–348 行两行（`let minimum = *samples.iter().min().expect("空的代价序列——上游没跑到任何一次 fsync");` 换行 `let maximum = *samples.iter().max().expect("空的代价序列——上游没跑到任何一次 fsync");`，连同缩进，表里用 `\n` 连）→ 两处都换成 `.unwrap_or(&0)` | T6：空序列不再 panic，`should_panic` 那条红 |
| N7「原装置记录头从48换成4048」 | `const LEGACY_RECORD_HEADER_BYTES: u64 = 48;` → `4048` | T7：乙由 2 变 3（命令三）；`entries_per_record_today_and_legacy` 里原装置一块装的项由 72 变 0，也红 |
| N8「记录头宽改回上一代（496→487）」 | `const JOURNAL_HEADER_BYTES: u64 = 496;` → `487` | 只有 `crates_format_literals_used_here_are_pinned_to_the_registered_values` 红（487 仍装 64 项，C1）；这条证的是 S1 的钉值在 |
| N9「记录头宽跨出装64项的区间（496→513）」 | 同一行 → `513` | `entries_per_record_today_and_legacy`：(4096 − 513) ÷ 56 = 63 ≠ 64（C2）；钉值那条也红 |

表格里原文、替换文中的 `\|` 是为了不切格转义的，写进变异表时是一个竖线 `|`（N2、N5 两条）。N8、N9 共用一条原文，与表里第 4、5 条共用 `= 64;` 那一行同例。每条原文在改完的源码里必须恰好命中一次（第七类）；2026-10-29 在改之前的源码上 `grep -c` 现查：N1–N5、N7 与 N6 的第一行各命中 1 次（第十三节）。改完之后执行员再数一次，N8、N9 的原文那时才存在。

## 十、失败条款

上一次登记 F1–F8 照旧；F1–F8 管的是 PC1、PC2 与 Q1–Q6 的重判，这一次 K5 三对相同时都不触发。另加：

| 编号 | 条款 | 什么观测会让它触发 | 触发之后 |
|---|---|---|---|
| F9 | **产物头对不上（正当结果）** | K1 有一份「不符」，或 K2「不同」（装置 sha256 没变），或 K3 有一把键退 0，或 K4「红」，或 K6「没指到」 | 第 3 行记乙，逐格写是哪一份产物、哪一行、哪把键、两边各是什么；不为凑绿改装置之后在同一份登记下重跑，要改就照 `evidence-discipline.md` 三步走 |
| F10 | **头宽与条款写的不同** | S1 核对时 `lib.rs:252` 不是 496，或 D23 已定项 4 写的头宽与 `lib.rs` 不同 | 不作废；先停在 S1，交主 agent 判这份登记作不作数（同上一次的 S1） |

## 十一、作废条款与停机条款

### 作废

上一次登记 V1–V7 照旧（V4 的批集合 {1, 2, 5, 10, 20, 50, 64, 65, 67} 不变），只改两条、加一条：

| 编号 | 条件 |
|---|---|
| V2（改） | 同上一次登记；这一次「比值是 NaN 或无穷、fsync 次数为 0」由 K7 在三份新产物上判，单测 T6 管「代价序列为空」 |
| V5（改） | 确定性：这一次不另跑第二遍。K5 拿新产物与另一天、另一份源码跑出的留存产物比，三对逐字节相同就已经是「两次跑正文相同」；K5 不同时由 S2 管，不在这里判 |
| V8（新） | K2 记下的装置 sha256 与出产物时的装置 sha256 不同（准入键算完之后装置又被改过）：三份产物作废，从 K2 重来 |

### 停机

| 编号 | 条件 | 两边查什么 |
|---|---|---|
| S1 | 装置的本地常量与 `crates/singlefs-format/src/lib.rs` 的字面量有一个对不上。核对命令见第十三节命令二；执行员开工跑一次（那时装置还是 487，预计只在 `JOURNAL_HEADER_BYTES` 一处不同，只记下来、不停）、改完之后出产物之前再跑一次，输出须与那九行逐字相同，且装置第 51 行是 496 | 同上一次登记 |
| S2 | K5 有一对不逐字节相同 | 先 `diff` 看不同的是哪几种行；再查头宽与接准入的改动有没有碰到第 900 行之前产出正文的函数（`git diff` 的改动块）；不许默认新产物对，也不许默认留存产物对 |
| S3 | 同上一次登记（几何配置行与 7.2 不同）；这一次由 K5 一并覆盖 | 同上一次登记 |
| S5（新） | K2 那一步准入入口退 2 / 3，或给出 `sha256=refused` | 装置的 `this_apparatus_source_path` / `repository_root_above` 与 `scripts/gate/admission.py` 的拒算说明两边对 |

### 出新产物、改登记行

- 三份产物都另存新名，旧名的日期换成跑的那天、`-h487` 换成 `-h496`：`research/results/e16-fifth-run-peak-under-today-widths-<日期>-h496.out`、`…-pc3-<日期>-h496.out`、`…-second-segment-<日期>-h496.out`，日期用 `TZ=Asia/Tokyo date +%F` 取。pc3 那一份不读头宽，名字里的 `h496` 记的是出它的那份装置抄的头宽，不是说 pc3 用了它。旧产物在 git 里留着，不恢复进工作区、不覆盖。
- `scripts/experiment-apparatus/replay.sh` 里 `grep -n '^E16R5|'` 命中的三行（2026-10-29 是第 1611–1613 行）只改第 4 栏的文件名，第 1–3 栏与第 5 栏 `exact` 不动。行号不写死：那份文件在工作区里另有别的会话的改动（`git diff --stat` 13 行新增，与 E16 无关）。
- 下面比对对象表里的「重跑产物名」按 2026-10-29 写；跑的那天不是 2026-10-29 的，执行员在第十二节写一行改名修订，同步改这张表的「重跑产物名」那一栏与 `.claude/gate.d/replay-comparison-units.tsv` 里对应的登记，别的不动。

**比对对象表**（门禁 doc-experiments 的 replay-comparison-units 格读它；比对对象都归档在 `59ce5a865^`）：

| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `e16-fifth-run-peak-under-today-widths-2026-10-04-h487.out` | `b6664083cece60898f66a24fffb5325099de2b7cda7172d50f14b88f1a1d1d64` | 3650 | `E16R5\|e16-fifth-run-peak-under-today-widths\|` 那一行，不带参数 | `e16-fifth-run-peak-under-today-widths-2026-11-02-h496.out` | 第一段全网格；K5 去头逐字节比 |
| 2 | `e16-fifth-run-peak-under-today-widths-pc3-2026-09-28.out` | `6180eb590e3c61de8e789d4f99dda3a61eebb56f5403cec32a21eefab34cae01` | 58 | `E16R5\|e16-fifth-run-peak-under-today-widths\|pc3` 那一行，参数 `pc3` | `e16-fifth-run-peak-under-today-widths-pc3-2026-11-02-h496.out` | 原装置几何上的 56 格（PC3）；K5 去头逐字节比 |
| 3 | `e16-fifth-run-peak-under-today-widths-second-segment-2026-10-04-h487.out` | `e4de5e34bff8f972bd882cc31c103273bec4b345c1a2844f79a62f3742647c2e` | 13366 | `E16R5\|e16-fifth-run-peak-under-today-widths\|second_segment` 那一行，参数 `second_segment` | `e16-fifth-run-peak-under-today-widths-second-segment-2026-11-02-h496.out` | 第二段三个旋钮分组；K5 去头逐字节比 |

这张表与 `.claude/gate.d/replay-comparison-units.tsv` 逐行对之前，那一格判红。E16 的装置没有正式跑单元表，三行要么在那份表的文件头逐行写「不登记的比对对象」并指到一条开着的欠账（`checks-owed.md` 里现在没有 E16 这一条，要先登记），要么另定登记法；怎么办交主 agent 定（第十三节 13.4 第 3 条）。

### 够判停机

第六节「够判点」一到就停，第 3 行记甲或乙交回。没跑的量逐条标：PC1、PC2「够判后未跑（原装置未改）」；Q1–Q6、8.1、8.2 的重判「够判后未跑（K5 三对逐字节相同，判定沿用第 6 次）」；V5 的第二遍「够判后未跑（K5 覆盖）」。

## 十二、修订

留空；装置写之后、产物之前由执行员写，只许收严或补臂。

- 2026-11-02（执行员，三份产物一份都还没跑）：补一条变异 N10「不是仓根的目录也认成仓根」，原文 `if root.join(PREFLIGHT_SCRIPT_UNDER_REPOSITORY_ROOT).is_file() {` → 替换文 `if true {`；变异表 31 → 32 条，第九节「31 条都是 `✅` 才写「31 条全抓」」照收严读成 32 条，N1–N9 的判法不动。依据是任务 4702（整表 `mutate.sh`）的基线读数：副本上单测「test result: FAILED. 33 passed; 1 failed」，红的是 `tests::repository_root_is_three_levels_above_the_apparatus_source` 的「仓根下要有 preflight.py：<分片根>/copy-1」（mutate.sh 的副本只拷装置目录，往上三层是副本根）。装置因此把「往上三层是不是真仓」从那条单测挪到 `preflight` 里拒算（退 78，新函数 `repository_root_with_preflight_script_above`），那条单测只钉路径算术，另加单测 `repository_root_is_refused_without_the_preflight_script_and_accepted_with_it`，N10 是它的证红。原判据一条不改；第十一节 S5 的「两边查什么」照旧（新函数在 `repository_root_above` 之上，不改它）。
- 2026-11-02（执行员，三份产物已出、未入库前；改名修订，不动判据）：三份产物是 2026-11-02 跑的，不是 2026-10-29，照第十一节「出新产物、改登记行」第三条把三个重跑产物名的日期段由 2026-10-29 换成 2026-11-02：`e16-fifth-run-peak-under-today-widths-2026-11-02-h496.out`、`e16-fifth-run-peak-under-today-widths-pc3-2026-11-02-h496.out`、`e16-fifth-run-peak-under-today-widths-second-segment-2026-11-02-h496.out`。同步改了三处：第十一节比对对象表「重跑产物名」一栏（#1–#3）、`.claude/gate.d/replay-comparison-units.tsv` 第 41–43 行、`scripts/experiment-apparatus/replay.sh` 的三行 `E16R5|` 第 4 栏。依据是派发提示「跑的那天是 2026-11-02」与第十一节那一条；K6 的「第十一节的三个新名」读成这三个 11-02 的名。第十一节与第十三节里别处写「2026-10-29」的是写登记那天的日期与当时读的行号，不是产物名，不改。

## 十三、读过的文件与跑过的命令

行号是 2026-10-29 读的那一刻的；grep 命中行也列。`research/results/` 下的产物一份都没读内容：三份留存产物只经 `git show` 接 `sha256sum`、`wc -l`、`grep -c '^E7INPUT'` 取了元数据。实验页三语只读了 `grep -n '^#'` 的标题行，结果与结论小节没读。

### 13.1 规则、共用约束、定义与脚本

| 文件 | 行 |
|---|---|
| `.claude/agent-common.md` | `grep -n '^#'`；17–52、53–118（「派发」一节只看到预览） |
| `.claude/singlefs-ai-sop/rules/test-discipline.md` | 1–122（全文） |
| `.claude/singlefs-ai-sop/rules/evidence-discipline.md` | 1–179（全文） |
| `.claude/rules/three-way-inference.md` | 1–166（全文） |
| `.claude/rules/mutation-sampling.md` | 1–93（全文） |
| `.claude/rules/implementation-first.md` | 1–19（全文） |
| `.claude/rules/test-script-discipline.md` | 1–106（全文） |
| `.claude/singlefs-ai-sop/rules/writing-discipline.md`、`command-safety.md`、`verify-before-claiming.md` | `grep -n '^#'`；78–123；10–48；1–25 |
| `.claude/agents/experiment-runner.md` | `grep` 命中 40、57 |
| `.claude/hooks/handback-guard.sh` | 66–76；`grep` 命中 3、68、139–144、218、219 |
| `.claude/hooks/agent-write-scope.tsv` | 1–22；`grep` 命中 23、27 |
| `scripts/experiment-apparatus/replay.sh` | 1600–1620；1770–1780；3455–3480；`grep` 命中 48、1403、1611–1613、1774、1777、3039、3063、3290、3295、3296、3302–3304、3467、3602 |
| `scripts/gate/admission.py` | 185–215；`grep` 命中 26、126–129、141、142、192、195、311、3013、3016、3028、3118、4588 与 `def` 列表 |
| `.claude/gate.d/replay-comparison-units.tsv` | 1–40；`grep` 命中 2、3、8、11、12、18–26、35–37、42、43、49–62 |
| `.claude/gate.d/stage-inputs.tsv` | `grep -n '^E16R5'` 命中 102–104 |
| `.claude/gate.d/doc-experiments.sh` | `grep` 命中 27、34–42、69、86、87、185–194、2878、2951；2951–3000 里 `grep` 命中的切格那两行 |
| `.claude/gate.d/code-source-discipline.sh` | `grep` 命中 19、430 |
| `.claude/heavy-thread-default` | 非注释行与 `grep` 命中 2、5 |
| `scripts/kb/quote-kb.py` | `--help` 前 30 行 |

### 13.2 派发给的输入与上两次登记

| 文件 | 行 |
|---|---|
| `research/prompts/m3-close-b19-p5-forks.md` | 1–9（全文） |
| `~/.local/share/singlefs/work/m3-close-b19-e16r5-run/result.md` | 1–175（全文） |
| `research/prompts/e16-r6-prereg.md`（`git show 04ceaa904^:`，执行员取出的副本 sha256 与它相同） | 1–752（全文） |
| `research/prompts/e16-r5-prereg.md`（`git show 31128273b^:`，同上） | `grep -n '^#'`；1–46；205–419 |

### 13.3 装置、变异表与 `crates/`

| 文件 | 行 |
|---|---|
| `scripts/experiment-apparatus/e16_fifth_run_peak_under_today_widths.rs` | 1–130；196–262；330–1029；1115–1135；1133–1262；`grep` 命中 50、51、54、78、887、1001、1002、1026、1127 与 `fn` 列表；`git diff -U0` 的改动块头 |
| `scripts/mutation/tables/e16_fifth_run_peak_under_today_widths.tsv` | 第一栏全文（22 条）；`git diff -U0` 的改动块头 |
| `crates/singlefs-format/src/lib.rs` | 240–260；`grep` 命中 20、38、176、179、241、248、249、252、255、258、339、448、459、588 |

### 13.4 kb 与交主 agent 认的项

| 文件 | 行 |
|---|---|
| `.claude/kb/checks-owed.md` | `grep` 命中 485（截前 600 字）、575（截前 600 字）；1、605 两个标题；18–20 |
| `.claude/kb/decisions/23-journal的角色与格式.md` | `grep` 命中 150；quote-kb 抄出的已定项 4 整节 |
| `.claude/kb/decisions/23-journal-role-format.en.md`、`23-ジャーナルの役割と形式.ja.md` | `grep 496` 命中 23、150、152、156、161、162、237、319；23、150 |
| `.claude/kb/experiments/16-*.md` 三语 | `grep -n '^#'`（标题行） |

交主 agent 认的项：

1. 第 7 次、登记名 `e16-r7-prereg.md`、装置与英文名沿用、准入键沿用三把 `E16R5…`。
2. 7 格的定法（第五节那张表）：Q2、Q4、Q6、8.1、8.2 补单测 T1–T5；V2 补 T6 加产物 `grep`（K7）；PC3 逐格靠产物（K5）加 T7 钉一格。变异表因此加 9 条。
3. 比对对象表三行在 `replay-comparison-units.tsv` 怎么登：E16 没有正式跑单元表，照 E185、E161、E184、E172 的先例要一条开着的欠账（`checks-owed.md` 里没有 E16 的这一条），由主 agent 定先登欠账还是另定登记法。
4. 实验页「第五次跑」「第六次跑」两节是结果节，我没读、没抄进第二节。
5. 产物名的日期按 2026-10-29 写进比对对象表，跑的那天不同时按第十一节改名。

### 13.5 跑过的命令（原样；读文件的 `sed`、`awk`、`grep -n` 已按行列在上面）

```
bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/m3-close-b19-e16r5-design --purpose '批 19：E16 第五次跑的差分重跑登记（C777、journal 头宽 496）'
  # ✓ 认领了已有的目录 …；rc=0
git log --all --oneline -- 'research/prompts/e16*'
git log --all --name-only --format='%h' -- 'research/prompts/e16*' | grep -E '^research/prompts/e16(-|_)' | sort -u
  # 只有 e16-r5-prereg.md、e16-r6-prereg.md 两份重跑登记，没有 r7
git show 31128273b^:research/prompts/e16-r5-prereg.md | sha256sum     # 3e858988…9932，582 行
git show 04ceaa904^:research/prompts/e16-r6-prereg.md | sha256sum     # b08fe45b…ff4a，752 行
git log --all --diff-filter=D --format='%h %ad' --date=short --name-only -- 'research/results/e16-fifth*'
  # 59ce5a865 2026-10-26 删了三份 2026-10-04-h487 / pc3-2026-09-28 / second-segment-2026-10-04-h487
for n in <三份留存产物>; do git show 59ce5a865^:research/results/$n | sha256sum; … | wc -l; … | grep -c '^E7INPUT'; done
  # 第十一节比对对象表的 sha256 与行数；E7INPUT 行数三份都是 0
sha256sum scripts/experiment-apparatus/e16_fifth_run_peak_under_today_widths.rs scripts/mutation/tables/e16_fifth_run_peak_under_today_widths.tsv scripts/experiment-apparatus/replay.sh crates/singlefs-format/src/lib.rs
  # a110f17e… / 4306a85c… / af00d367… / f0a126bc…
git diff -U0 -- scripts/experiment-apparatus/e16_fifth_run_peak_under_today_widths.rs | grep '^@@'
  # 改动块从 -902 起：@@ -902,5 +902,14 @@、-911、-932、-936、-941、-945、-1251（测试模块）
for a in 'if next.batch <= peak_batch_low {' 'let batch_interior = batches_upper != batch_axis_maximum;' '.filter(|cell| selector(cell) > 0.0)' 'row_dips_below_one: minimum_cell.ratio < 1.0,' 'classes.windows(2).all(|window| window[0].1 == window[1].1)' 'const LEGACY_RECORD_HEADER_BYTES: u64 = 48;' 'let minimum = *samples.iter().min().expect(' 'let maximum = *samples.iter().max().expect('; do grep -cF "$a" $F; done | tr '\n' ' '
  # F=scripts/experiment-apparatus/e16_fifth_run_peak_under_today_widths.rs；八条依次 1 1 1 1 1 1 1 1；N6 的两行在第 347、348 行
nice -n 19 python3 scripts/kb/quote-kb.py <草稿目录>/q-forks.md 'research/prompts/m3-close-b19-p5-forks.md:5-6' 'research/prompts/m3-close-b19-p5-forks.md:9-9'
  # ✓ 2 段整抄进 …，回读逐字节一致；rc=0
nice -n 19 python3 scripts/kb/quote-kb.py <草稿目录>/q-clauses.md '.claude/kb/checks-owed.md:19-20' '.claude/kb/checks-owed.md:485-485' '.claude/kb/decisions/23-journal的角色与格式.md@#### 已定项 4：记录头约束在单个原子单元内，头 496 字节'
  # ✓ 3 段整抄进 …，回读逐字节一致；rc=0。追加进登记之后 tail -n 47 与它 cmp 相同
TZ=Asia/Tokyo date +%F      # 2026-10-29
```

### 13.6 命令一：独立锚点（`anchors_e16_r7.py`，全文）

脚本 sha256 `57b237fc6530950414376d43d8fb74c44242f1b5ec260045c69c6cef6e4c1168`，输出 sha256 `8ae74725b5f55787aa6570c343ab6e8eb040897e31fb8633f071bd15c3ce9dad`；在草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-e16r5-design/` 里跑，第一节到第十一节写之前跑过，第四节与第七节的数取自这一次输出。执行员要复核时原样拷进草稿目录重跑。

```python
#!/usr/bin/env python3
# E16 第 7 次（差分）的独立锚点：头宽 487 -> 496 之后，装置里读头宽的每一处取值变不变。
# 常量手抄自 crates/singlefs-format/src/lib.rs:241-258 与 D23 已定项 4 / 17；批轴手抄自 r6 登记第 13.8 节（含 63、65），不 import 装置。
from math import ceil
REC, ITEM = 4096, 56
for h in (487, 496):
    q = (REC - h) // ITEM
    print(f"H={h} room={REC-h} per_record={q} remainder={REC-h-q*ITEM} q*56+H={q*ITEM+h} (q+1)*56+H={(q+1)*ITEM+h} cursor_record_blocks={ceil(h/REC)}")
hs = [h for h in range(0, REC) if (REC - h) // ITEM == 64]
print(f"per_record_64_header_range=[{min(hs)},{max(hs)}] H={min(hs)-1}->{(REC-min(hs)+1)//ITEM} H={max(hs)+1}->{(REC-max(hs)-1)//ITEM}")
print(f"margin_below_upper_edge={max(hs)-496} margin_above_lower_edge={496-min(hs)}")
streams = sorted(set([2, 3, 4, 6, 7, 8, 11, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256] + [384, 512, 768, 1024]))
base = [1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 14, 16, 20, 24, 28, 32, 40, 48, 50, 56, 63, 64, 65, 67, 68, 70, 72, 73, 80, 96, 100,
        112, 128, 160, 192, 200, 224, 256, 320, 384, 448, 512, 640, 768, 1024, 1280, 1536, 2048, 3072, 4096]
ext = [6144, 8192, 12288, 16384]
union = set(base + ext)
for s in streams:
    union |= {s - 1, s, s + 1}
union.discard(0)
p487 = (REC - 487) // ITEM
p496 = (REC - 496) // ITEM
changed = sorted(b for b in union if b + ceil(b / p487) != b + ceil(b / p496))
print(f"batch_values_union={len(union)} l_blocks_changed_by_487_to_496={len(changed)} {changed}")
for bt in (63, 64, 65, 66, 67, 68, 70, 72, 73):
    print(f"  batch={bt} l_blocks_h487={bt+ceil(bt/p487)} l_blocks_h496={bt+ceil(bt/p496)}")
```

```
$ cd /home/fy5090/.local/share/singlefs/work/m3-close-b19-e16r5-design && nice -n 19 python3 anchors_e16_r7.py > anchors_e16_r7.out 2>&1   # rc=0
H=487 room=3609 per_record=64 remainder=25 q*56+H=4071 (q+1)*56+H=4127 cursor_record_blocks=1
H=496 room=3600 per_record=64 remainder=16 q*56+H=4080 (q+1)*56+H=4136 cursor_record_blocks=1
per_record_64_header_range=[457,512] H=456->65 H=513->63
margin_below_upper_edge=16 margin_above_lower_edge=39
batch_values_union=81 l_blocks_changed_by_487_to_496=0 []
  batch=63 l_blocks_h487=64 l_blocks_h496=64
  batch=64 l_blocks_h487=65 l_blocks_h496=65
  batch=65 l_blocks_h487=67 l_blocks_h496=67
  batch=66 l_blocks_h487=68 l_blocks_h496=68
  batch=67 l_blocks_h487=69 l_blocks_h496=69
  batch=68 l_blocks_h487=70 l_blocks_h496=70
  batch=70 l_blocks_h487=72 l_blocks_h496=72
  batch=72 l_blocks_h487=74 l_blocks_h496=74
  batch=73 l_blocks_h487=75 l_blocks_h496=75
```

### 13.7 命令三：原装置几何那一格的锚点（`anchors_e16_r7_original_geometry.py`，全文）

脚本 sha256 `478dd2613326efda804eaf84a354cb13d4fa1fc6975e06feae489002a5efb3f4`，输出 sha256 `6ce7e0c1f7f267b9d0299589b13b8d8441a66d05495d0162507a0631273b24b9`。

```python
#!/usr/bin/env python3
# E16 第 7 次：原装置几何（扇出 128、叶罩 128、单元 128^4）上 S = 2、B = 1 那一格的甲、乙、比值，单测 T7 的锚点。
# 定义照第 5 次登记第一节与第五节 5.1 手写：甲 = 脏单元 1 + 祖先层数 L + 游标记录 1 + 根槽 1；乙 = 脏单元 1 + ceil((48 + 56k) / 4096)（至少 1）。
from math import ceil
leaf, fanout, units = 128, 128, 128 ** 4
level = 0
while leaf * fanout ** level < units:
    level += 1
node_levels = level + 1
for header in (48, 4048):
    intent = 1 + node_levels + 1 + 1
    wal = 1 + max(1, ceil((header + 56 * 1) / 4096))
    print(f"legacy_header={header} root_level={level} node_levels={node_levels} intent={intent} wal={wal} ratio={intent / wal} predicted={(1 + 1 * node_levels + 2) / (1 + 1)}")
```

```
$ cd /home/fy5090/.local/share/singlefs/work/m3-close-b19-e16r5-design && nice -n 19 python3 anchors_e16_r7_original_geometry.py > anchors_e16_r7_original_geometry.out 2>&1   # rc=0
legacy_header=48 root_level=3 node_levels=4 intent=7 wal=2 ratio=3.5 predicted=3.5
legacy_header=4048 root_level=3 node_levels=4 intent=7 wal=3 ratio=2.3333333333333335 predicted=3.5
```

### 13.8 命令二：停机 S1 的常量核对

执行员开工跑一次（装置还是 487，只记下来）、改完之后出产物之前再跑一次，第二次的输出须与下面九行逐字相同。

```
grep -nE '^pub const (DATA_UNIT_BYTES|DATA_UNIT_PAYLOAD_OFFSET|EXTENT_TREE_INTERNAL_FANOUT|EXTENT_TREE_LOWER_LEAF_DATA_UNITS|JOURNAL_RECORD_BYTES|JOURNAL_HEADER_BYTES|JOURNAL_NAMED_ENTRY_BYTES|JOURNAL_NAMED_ENTRIES_PER_RECORD|TEST_IMAGE_DEFAULT_BYTES):' crates/singlefs-format/src/lib.rs
```

2026-10-29 的输出：

```
20:pub const DATA_UNIT_BYTES: u64 = 32768;
38:pub const DATA_UNIT_PAYLOAD_OFFSET: u64 = 134;
176:pub const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;
179:pub const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;
241:pub const JOURNAL_RECORD_BYTES: u64 = 4096;
252:pub const JOURNAL_HEADER_BYTES: u64 = 496;
255:pub const JOURNAL_NAMED_ENTRY_BYTES: u64 = 56;
258:pub const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = 64;
339:pub const TEST_IMAGE_DEFAULT_BYTES: u64 = 4_294_967_296;
```

### 13.9 另跑过的只读命令（原样，输出摘要在注释里）

```
ls research/results/ | grep -c . ; ls research/results/ | grep e16      # 115；e16 开头的 0 个（e16* 三份都已归档）
git status --short -- <装置> <变异表> scripts/experiment-apparatus/replay.sh      # 三份都是 M
git diff --stat -- scripts/experiment-apparatus/replay.sh      # 13 insertions，改动块里没有 E16
git diff --stat -- .claude/gate.d/replay-comparison-units.tsv      # 3 insertions（别的会话）
grep -nE 'NEW_ROOT|\b364\b|\b373\b|thread|rayon' <装置>      # 零命中
grep -n '单元表' .claude/kb/checks-owed.md      # 开着的「没有单元表」欠账只有 C879（E165）、C895（E185）、C901（E161 与 E184）、C930（E172），没有 E16 的（另命中 C880、C905、C912、C916 不相干）
grep -n 'experiment-designer' .claude/hooks/agent-write-scope.tsv      # 零命中（定义没有 Write / Edit，不进那张表）
python3 scripts/agents/dispatch-prompt.py experiment-runner | grep -n '多机：'      # 第 15 行是占位，2 小时内删掉这一行
python3 scripts/kb/replace-once.py research/prompts/e16-r7-prereg.md <旧串> <新串>      # 定点改这份登记自己写的内容（改句子、插入这一小节、补命令行、改这条注释），每次都命中 1 次
```

## 十四、并行与挂钟

预计挂钟：2 小时内（派发提示同写）。下面的数都是推的，依据写在各行。

独立单位：三种跑法（不带参数、`pc3`、`second_segment`）各是一个进程，互不读对方的输出，可以同时起；变异表 31 条互相独立，由 `mutate.sh` 自己分派；新加的 7 条单测互相独立。
线程：编译、单测与变异表经 `bash scripts/hosts/capped.sh 31` 起，31 取 `.claude/heavy-thread-default` 的登记值；装置本身单线程（`grep -nE 'thread|rayon'` 零命中），三种跑法各占 1 个核。
串行：改装置与单测 → 变异表 → S1 第二次核对 → K2 现算三把准入键 → 三份产物 → K1、K5、K7 → 落盘与改复跑表 → K3、K4、K6 这条链必须串行，因为准入键只对「装置最后一次改动之后」的源码成立（V8），K3 只在产物落进 `research/results/` 之后判得出；链里只有「三份产物」这一步内部可以并行。产物要在本机主工作区的装置目录下编（编译目录放草稿目录）：装置按编译时的源文件路径取仓根去问准入、读 `research/results/`，在第二台编出来的产物头对不上本机的复跑表。
预计挂钟：整段约 1.5 小时。写 7 条单测与改 496 约 40 分钟（人工，估的）；单测一次约 31 秒（上一位执行员的单测日志「27 passed … finished in 30.62s」，量的，新加 7 条都是毫秒级的小网格，推的）；变异表 31 条，按每条一次编译加一次单测约 1 分钟、31 线程并行约 8 条一批，约 4 批共 5–10 分钟（推的）；产物：第一段约 50 秒、`pc3` 秒级、第二段约 3 分 48 秒（转述上一位执行员报告里 b16 那一次在空闲机上的量，我没现查，按推的用），三份并行时约 4 分钟；准入、比对、落盘与复跑表约 10 分钟。
预计在用核数：编译与变异表那一段 31 个；三份产物并行那一段 3 个（串行跑就是 1 个）；准入入口、`cmp`、`grep` 那一段 1 个。

内存：装置的态是一格一个 `HashSet`，量级是一次 checkpoint 间隔里的脏单元数（批轴端点 B = 16384 那一格的间隔是 16384 × ⌈2000 ÷ 16384⌉ = 16384 个操作，脏单元不多于这个数），推的不过百兆；上限取 `scripts/experiment-apparatus/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值（共用约束「跑编译出来的代码经内存包装」），派发提示另给的照提示。上一位执行员遇到的是本机内存包装那个总池满了（报告「本机内存包装的 slice 满」），不是装置要的多；那时排不上就停下交回，不绕开包装。
