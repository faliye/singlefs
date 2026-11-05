# E172 第 3 次跑前登记（重跑）：删除写出的字节的干跑

写于 2026-10-29，装置改动之前、第 3 次产物之前。重跑，不占新号。第 2 次重跑登记 `research/prompts/e172-r2-prereg.md` 在 04ceaa904 归档删了，取法 `git show 04ceaa904^:research/prompts/e172-r2-prereg.md`（931 行，sha256 `f1a81a1f4efcb57dddb268843fd05361bb73b6381a1b6c141c0a2bad9d2f89eb`）；下文「第 2 次登记」都指这一份、行号是它自己的。

这不是差分重跑：输入（`crates/`）变了，装置也要加退役项（C775（E172 装置没有退役项） 的被测项），问题单换成一行、判据新加了标签 6 的退役项、退役容器与退役树根几格。第 2 次登记里没变的读法、格、阳性对照与条款逐条引它的节与行，不重抄。

<!-- doc-lint:not-numbers H1 H2 H3 R1.a R1.b R1.t R2.a R2.b R2.c R3.a R3.b R3.c R3.d R3.e R4.a R4.b R4.c R5.a R5.b R6.a R6.b R7 R8.a R8.c R-ret R-flip R-dump R-traj R-det R-pc PC-A PC-B PC-C PC-D PC-F PC-G PC-H PC-I PC-J PC-K PC-L L03-1 L03-2 L03-3 L03-4 L03-5 L03-6 L03-7 L03-8 L03-9 L03-10 L03-11 L03-12 L03-13 L03-14 L03-15 L03-16 L03-17 S0 S1 S2 S3 S4 S5 S6 F1 F2 F3 F8 F9 F10 F11 V1 V2 V3 V4 V5 V6 V7 M10b M29 M31 M32 M33 M34 M35 M36 M37 M38 M39 M40 M41 M42 T14 T15 T16 T17 T18 T19 T20 T21 T22 T23 T24 F12 改16 改17 改18 改19 改20 改21 改22 改23 改24 改25 改26 改27 改28 改29 改30 -->

改装置之前的对照（本登记写之前现取，执行员改装置之前照第十一节 S0 再核一次）：

| 文件 | sha256 | 说明 |
|---|---|---|
| `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs` | `2960edb522958a0eada6f955689537b5381d5812d947bdaa3bed65a281a4d2a5` | 第 2 次的装置，4433 行；`git status` 干净（最后一次进 git 是 f4605214d） |
| `scripts/mutation/tables/e172_deletion_write_dry_run.tsv` | `fc01f6d3d58c5f51ee784924160755f0b70000c74a770796c86c37dbbe4a25ed` | 第 2 次的变异表，32 行 |
| `crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs` | `176c86c23c459226befdf8cedb77c8e18e264d5fbeaec710d36e79442f062270` | `crates/` 侧只读导出，386 行 |
| `crates/singlefs-harness/src/scenario.rs` | `e16f73ef003eb56132c0b811e15131a81c264d86b9ed1542665606fbb607975d` | 三段删除历史的场景 |
| `research/prompts/m3-close-b19-p5-forks.md` | `692299409667b4412527fb0a555b65a2a10c08f40b32fc1fd29a35546f97983a` | 这一次的问题单（三行，本登记只管第 2 行） |

## 一、问题

英文名：deletion_write_dry_run

（照抄第 2 次登记第 20 行。装置仍是 `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs`、bin 名 `e172-deletion-write-dry-run`、变异表 `scripts/mutation/tables/e172_deletion_write_dry_run.tsv`，原地改，不另起新装置；中文简称「删除写出的字节的干跑」只用在 kb 页。）

主 agent 给的问题（派发提示逐字，与问法有关的几行）：

> C775（E172 装置没有退役项）：重跑已有实验 E172，第 3 次，写重跑登记 `research/prompts/e172-r3-prereg.md`。上一份重跑登记在 04ceaa904 归档删了（`git show 04ceaa904^:research/prompts/e172-r2-prereg.md` 可读，931 行），照它的形态与固定节名写。登记要写死（执行员已现查、交来的三件）：① 退役项从哪一侧来——导出侧（`crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs`，`scripts/experiment-apparatus/replay.sh:3173` `driver_e172` 先起它）还是解码侧（`scripts/experiment-apparatus/e172_deletion_write_dry_run.rs`），导出侧要改 `crates/` 的话写明归实现员；② 比对对象 196 760、196 678、196 608 是准入预付的标签 6（`crates/singlefs-core/src/admission.rs:2099`、`crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs:198` 与第 110 行注释），三个数各对删除的哪一步、按哪种历史比写死；③ `replay.sh:1755` 登记的产物 `e172-r2-deletion-write-dry-run-2026-10-05.out` 已在 59ce5a865 删了，新产物名与复跑行怎么改写明。判别臂：装置去掉退役项必不等。开工照 `.claude/rules/implementation-first.md` 先读 `crates/` 今天的删除写法。
> 准入已判：这次重跑不是因为输入变了，是装置要加退役项（C775 的被测项）；E172 在 `.claude/gate.d/stage-inputs.tsv` 没有准入键，登记里写明要不要接准入（照 E142 那一行），接的话交执行员改

### 1.1 问题单逐行（`research/prompts/m3-close-b19-p5-forks.md` 第 8 行原样抄，表头第 5–6 行）

| # | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 2 | C775 E172（删除写出字节的干跑）：甲 装置加退役项之后逐字段与 `crates/` 相等 ／ 乙 有字段不等 | 重跑产物逐字段与 `crates/` 用例钉的 196 760、196 678、196 608 比 | 重跑产物出来、逐字段比完；装置去掉退役项必不等（判别力） | 开着 |

问题单第 1、3 行归 E173、E16R5，本登记不管。下文「第 2 行」都指上表这一行。第 2 次的五行问题单（`research/prompts/e172-r2-questions.md`，已归档）这一次不再判，它的五行对到的量只在第六节 6.4 当附带照报。

### 1.2 读法写死

没写的照第 2 次登记第一节 1.2（第 2 次登记第 49–68 行）原样适用：窗口、删除类发布、H1 / H2 / H3 的定义、第 1 行格表的「全等」「命中」、段串与种类串的读法、H2 只拿意图那一次进段串判定（读法乙）。下面只写这一次改了或新加的。

- **退役项是什么**（主 agent 派发提示「装置要加退役项」的读法；出处是 layout/03 文件头那一句「核的是退役树加进之前那一版：退役树带来的 tr@0、tr 两格与标签 6 的退役项没核（C775（E172 装置没有退役项））」，第二节抄了）：两样一起加。
  - 甲份「标签 6 的退役项」：layout/03「记账标签 6 的值」那一行式子里 `2 + H_退役 + 1` 那三槽与 `35 × n_记录` 那一项。
  - 乙份「退役树写出的形态」：退役容器 tr@0（码 3，打包记录类型 3、记录宽 34、出生树 65548，layout/03「退役容器」那一行）、容器里的退役记录（layout/03 第五节，34 字节）、退役树根 tr（码 2，「与墓碑树同形」、条目宽 120）、树表第 12 条（退役树 65548、种类 13、头 ID 65537，「树表退役树条目」那一行）、树 ID 水位 65549（「树表四条新条目」那一行与 layout/01「树 ID 水位」那一行）、最后一批角色集合与段串里的 tr@0、tr（layout/03 第四节「最后一批」那一行）。
  - 只加甲份不加乙份的读法不登记：乙份那几格不加，L03-12 条目数（期望 11）、L03-14 水位（期望 65548）、S3 角色对拍（装置不认 tr@0、tr）与 R2.a（退役单元的字节没有格认领）在今天的导出上由构造就不等（推的：从 `crates/` 今天写 12 条树表、写 tr@0 与 tr 推，第三节），「甲 逐字段相等」那一支在这种读法下根本判不出来。
- **「逐字段与 `crates/` 相等」**：两层，各报各的。
  - 条款层：装置按 layout/01、layout/03 今天的写法算出的期望，与导出里 `crates/` 写出的字节逐格比（第 2 次第 1 行那一套 R1.a、R1.b、R1.t、R2.a–R2.c、R3.a–R3.e，加这一次新加与改了的格，第六节 6.1）。
  - 钉值层：标签 6 在每个删除版本解出的值，与 `crates/` harness 用例钉的三个数逐版比（下一条）。两层不合成一个 true / false。
- **196 760、196 678、196 608 各对哪一步、按哪种历史比**（派发提示 ②）：三个数出自 `crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` 第 198 行 `vec![196_760, 196_678, 196_678, 196_678, 196_608]`，那条用例（第 113–202 行）的历史是三单元文件、批大小 1、两块盘、同一个进程里意图接三批（`crates/singlefs-harness/tests/common_images_for_the_checker_tier/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` 第 42–109 行），与 E172 的 H1 同一种（`crates/singlefs-harness/src/scenario.rs` 第 277–319 行 `IntentAndEveryBatchInProcess`）。所以按 H1 比；H2 的删除类发布按同几何的那一次比（续做批之后那一版的 n_live、n_记录、两个 H 与会话里同几何的批相同）；H3 不拿钉的值比，比第七节由 layout/03 式子算的 H3 值。

| harness 第 198 行那一格 | 那一版 | n_live | n_记录 | 钉的值 | E172 里比哪一版 |
|---|---|---|---|---|---|
| 第 1 个 | 删除之前（第 116 行 `pool.output`，新池新建文件与顺序写之后） | 1 | 1 | 196 760 | 不比：H1、H2 的窗口从意图那次发布之前起（`deletion_write_dump.rs` 第 6–11 行），导出不打窗口之前那一版的字节；报一行 `harness_version=before_intent in_window=false` |
| 第 2 个 | 意图 | 0 | 1 | 196 678 | H1 意图（txg 5）；H2 意图 |
| 第 3 个 | 3 → 2 | 0 | 1 | 196 678 | H1 3 → 2；H2 续做 3 → 2 |
| 第 4 个 | 2 → 1 | 0 | 1 | 196 678 | H1 2 → 1；H2 续做 2 → 1 |
| 第 5 个 | 最后一批 | 0 | 0 | 196 608 | H1 最后一批；H2 续做最后一批 |

  - 窗口里等于 196 760 的那一版是 H3 最后一批（n_live = 1：2 号；n_记录 = 1），式子的四个输入与 harness 第 1 个那一版相同、历史与版本都不同：照报一行 `same_value_as_harness_first=true`，不算与钉的值比过；它在钉值层之外，进条款层的 A-tag6（第七节 7.1）。
  - H2 窗口里卸载、写行、暖机那几次不是删除类发布的版本也带标签 6，只进 R-traj 轨迹（第八节），不判。
- **判别臂「装置去掉退役项必不等」**：装置在每个删除版本上同一份解码读数算两份期望——带退役项的（式子全项）与去掉退役项的（`2 × ((2 + H_墓碑 + 1) × 16384 + 41 × n_live)`，就是第 2 次装置那一种）——各与解出的值比，报一行 R-ret（第六节 6.2）。判别力成立 = 每个删除版本上带退役项的那一份与解出的值相等时，去掉退役项的那一份不等。两份差 `2 × ((2 + H_退役 + 1) × 16384 + 35 × n_记录)` ≥ 98 304（推的，式子算的），所以「带的相等、去掉的也相等」不会同时出现；这一行证的是装置那一格分得出这一项，不是证导出对。
- **第 2 次问题单其余四行的量**（R4.a–R4.c、R5.a、R5.b、R6.a、R6.b、R8.a、R8.c）：装置照算照报，附带，不进第 2 行判定（问题单这一次没有一行问它们）；它们的阳性对照照跑（第五节 5.4），没转红只作废它们自己那几行（第十一节 V4）。

## 二、被测条款与它引的定义

被测条款（主 agent 点名）：`.claude/kb/checks-owed.md` C775（E172 装置没有退役项） 一行；E172（删除写出的字节的干跑） 实验页。条款层的期望出自 layout/03 与 layout/01 今天的写法（下表）。第 2 次登记第二节抄过的十二段（第 2 次登记第 93–565 行）这一次照样是解码要用的定义，退役树那几行之外没有重抄：它们的现状要不要重抄，看第十一节 S0 第三条（layout/03、layout/01 第 2 次抄过的那几段今天变没变，由执行员开工前 `diff` 核）。

小节清单（抄 / 不抄）：

| 源 | 小节 | 抄不抄 | 为什么 |
|---|---|---|---|
| checks-owed | C775 一行 | 抄 | 被测条款 |
| E172 实验页 | 一级标题与 `### 做了什么` 之前的正文、`### 判决` 起到 `### 影响的决策`、`## 历史版本` | 不抄 | 是答案（结果与结论），定义「实验页与产物照样不读结果与结论」；标题行与 `### 做了什么` 之前那段正文开工时读到了，读到的数列进第四节 |
| E172 实验页 | `### 做了什么` | 不抄 | 记的是第 2 次跑了什么、两份导出与产物的 sha256 与行数；这几个数列进第四节与第十一节比对对象表，不当定义 |
| E172 实验页 | `### 复跑` | 抄 | 复跑命令与内存上限，第五节 5.7 的跑法照它改 |
| layout/03 | 文件头那一段（第 5 行） | 抄 | 「退役树带来的 tr@0、tr 两格与标签 6 的退役项没核」是第一节 1.2「退役项是什么」的出处 |
| layout/03 | 第一节「退役容器」「树表四条新条目」「树表退役树条目」「记账标签 6 的值」四行 | 抄 | 乙份、甲份与水位的期望值 |
| layout/03 | 第一节其余各行、第二、三节 | 不抄 | 第 2 次登记第 95–158 行抄过（那时的 layout/03 全文），今天变没变由 S0 第三条核 |
| layout/03 | 四、段序列（整节） | 抄 | 最后一批角色集合 12（含 tr@0、tr）、段串 `24+2+1+2`、整条流四串相接、每批新写上界 |
| layout/03 | 五、退役记录（整节） | 抄 | 退役记录 34 字节的字段表 |
| layout/01 | 第 80 行（t12 树表单元第 1 版 12 条） | 抄 | L03-12 条目数 |
| layout/01 | 「树 ID 水位」一行 | 抄 | L03-14 期望 65549 |
| layout/01 | 「树表条目」一行 | 抄 | 第 1 版 12 条的逐条字段 |
| layout/01 | 树表码 2 头「key 区间」一行 | 抄 | [65536, 65548]，树表 key 区间那一格 |
| layout/01 | 其余各节 | 不抄 | 理由同第 2 次登记第 84 行：装置里每个常量旁带 layout/01 行号 |
| D18 已定项 24、D8 已定项 9 / 11、D3 已定项 2 | 退役树与标签 6 的决策分项 | 不抄 | layout/03 那几行逐字段写了取值并点名这几项；装置只从 layout 抄偏移与取值（第 2 次登记第 597 行同一做法） |

下面由 `scripts/kb/quote-kb.py` 抽出（出口 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-e172-design/quote-section2.md`，脚本回读逐字节一致，十三段 121 行），整份追加进来。

**出处 `.claude/kb/checks-owed.md:483-483`（整段抄，未转述）**

```markdown
| C775 | E172 装置没有退役项 | 第 50 批之后 `crates/` 的删除版本记账标签 6 含退役项（`crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` 钉的 196 760、196 678、196 608），E172（删除写出的字节的干跑） 装置 `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs` 里没有退役项（`grep -c 退役` 为 0），页上标签 6 的读数（H1、H2 每个删除版本 98304，H3 98386）也不含它；E172（删除写出的字节的干跑） 的逐字段相等是退役树加进之前的 `crates/` 上的读数 | 重跑 E172（删除写出的字节的干跑）：装置加退役项，追平之后逐字段与 `crates/` 比，不等即红 | 无；改装置与重跑归实验设计员与执行员；去向：里程碑三三·6（E172（删除写出的字节的干跑） 装置加退役项，重跑后逐字段比，同 C775（E172 装置没有退役项） 旧注）（五·11，2026-10-12 判）。 | `records/2026-10-11-里程碑三知识腐烂治理.md`（2026-10-11 知识腐烂回扫） |
```

**出处 `.claude/kb/layout/03-third-txn.md:5-5`（整段抄，未转述）**

```markdown
**里程碑「删除与多次 COW」三·6 的登记表**（[03-third-txn.md](../milestone/03-third-txn.md)），2026-10-01 立。新池新建文件的每一段字节在 [layout/01-first-txn.md](01-first-txn.md)，覆盖写、释放、回退与复用新写出的形态在 [layout/02-second-txn.md](02-second-txn.md)；这里只登记删除（单文件 unlink）与管理员回退的谱系记录新写出的形态。同一个字段的宽度与落点以 layout/01-first-txn.md 为准、不另抄。每一行指一条决策分项，指不到分项的写成「格式级空白」。删除已经实装：`crates/singlefs-core/src/transaction.rs` 的 `publish_unlink_intent`（意图）、`publish_unlink_step`（一批）、`walk_the_deletion_queue_until_it_is_empty`（走空），挂载续做在 `crates/singlefs-core/src/mount.rs` 的 `continue_the_deletion_queue_until_it_is_empty`；`crates/singlefs-harness/tests/unlink_single_file_publishes.rs` 钉住意图、一批、最后一批三种发布的重写角色集合。盘上字节由 E172（删除写出的字节的干跑） 照 E142（新池新建文件的干跑） 的形态逐字段核过，核的是退役树加进之前那一版：退役树带来的 tr@0、tr 两格与标签 6 的退役项没核（C775（E172 装置没有退役项））；段序列表里标「推的，待干跑核」的几格照旧是推的。
```

**出处 `.claude/kb/layout/03-third-txn.md:21-21`（整段抄，未转述）**

```markdown
| 退役容器 | 有退役记录的那次发布（叶删空摘掉、合并被吸收）；有新记录的那次发布才重写开放容器，没有的发布照抄；满了才关；第一版不回收 | 码 3 打包记录单元，打包记录类型 = 3、记录宽 = 34、出生树 = 退役树的树 ID；含 29 字节预留位的头 136，一个 32 KiB 容器 ⌊(32768 − 136) ÷ 34⌋ = 959 条；容器号从 1 起，下一个号 = 退役树里最大号 + 1，零根时 1，0 不用（规则同墓碑容器）；树根与墓碑树同形：码 2 根、条目宽 120，孩子是码 3 退役容器，根满时长高；字段在「五、退役记录」那一节 | D18（块里携带什么信息） 已定项 24 / 已定项 11 | 三·6 |
```

**出处 `.claude/kb/layout/03-third-txn.md:22-22`（整段抄，未转述）**

```markdown
| 树表四条新条目 | 第一个文件版本那次发布 | `deleted_inodes`、墓碑树、谱系树、logged_ops（零根，D8（核心索引结构） 已定项 5）各一条，条目字段照 [layout/01-first-txn.md](01-first-txn.md)「七、发布（根记录与根槽）」那一节的树表条目；根指针零；树表 7 → 11 条（再加退役树一条为 12，见「树表退役树条目」那一行）；四棵的树 ID：`deleted_inodes` 65544、墓碑树 65545、谱系树 65546、logged_ops 65547（原来的八棵占 65536 到 65543），种类码：`deleted_inodes` 9、墓碑树 10、谱系树 11、logged_ops 12；树 ID 水位 mkfs 时 65536、第一个文件版本那次之后 65549（四棵占到 65547，退役树 65548，见「树表退役树条目」那一行）；头 ID（树表条目里 8 字节）：`deleted_inodes`、墓碑树写 inode 树的树 ID 65537，谱系树与 logged_ops 写 0 | D8（核心索引结构） 已定项 9 / 已定项 11 / 已定项 16；D18（块里携带什么信息） 已定项 24；D5（快照 / 空间记账机制） 已定项 9 | 三·2、三·6 |
```

**出处 `.claude/kb/layout/03-third-txn.md:23-23`（整段抄，未转述）**

```markdown
| 树表退役树条目 | 第一个文件版本那次发布（与上一行同一次） | 退役树一条，零根，条目字段照 [layout/01-first-txn.md](01-first-txn.md)「七、发布（根记录与根槽）」那一节的树表条目；根指针零；树表 11 → 12 条；树 ID 65548，种类码 13；树 ID 水位第一个文件版本那次之后 65549；头 ID（树表条目里 8 字节）写 inode 树的树 ID 65537（同墓碑树） | D8（核心索引结构） 已定项 9 / 已定项 11；D18（块里携带什么信息） 已定项 24 第 6 条 | 三·6 |
```

**出处 `.claude/kb/layout/03-third-txn.md:24-24`（整段抄，未转述）**

```markdown
| 记账标签 6 的值 | 第一个文件版本那次发布起，每一版都按那一版现算 | 记账树叶里 key (6, 0, 0xFFFF_FFFF, 代) 那一行的 value 8 字节（行的形态在 [layout/01-first-txn.md](01-first-txn.md)「记账 key」那一行），新池新建文件里这个值是式子的值（n_live = n_记录 = 1、两个 H 都是 0 时 196 760，[layout/01-first-txn.md](01-first-txn.md)「记账条目」那一行的已承诺预留就是它）：副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)，单位字节、按副本之和记（C375（待删占用与已承诺预留按份还是按和记没有条款））；H_墓碑、H_退役 是现行那一版墓碑树、退役树的内部层数（零根与只有根时 0），n_live 是这一版 inode 树里 nlink ≥ 1 的记录数，n_记录 是这一版 inode 树里全部记录数（含 nlink = 0 的），41 = ⌈2 × 16384 ÷ 815⌉、35 = ⌈32768 ÷ 959⌉ 每份副本、每个对象先取整再乘 n_live、n_记录；树表里没有 `deleted_inodes` 条目的版本取 0；行内偏移与值随实现，连带记账树单元的载荷校验和变 | D3（空间分配） 已定项 2；D5（快照 / 空间记账机制） 已定项 4 | 三·2、三·6 |
```

**出处 `.claude/kb/layout/03-third-txn.md:50-65`（整段抄，未转述）**

```markdown
## 四、段序列

删除类发布（意图、不是最后一批的批、最后一批、挂载续做里的批）和管理员回退都是普通发布的形状：[单元 × 2 盘] 屏障 [journal 记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] 屏障（[layout/01-first-txn.md](01-first-txn.md) 段序列登记表「普通发布（新池新建文件）」那一行）。单元数一栏里 harness 用例钉住重写角色集合的，写明用例；没钉住的标推的。段串（两块盘）全是推的，三·6 干跑核。角色名照 `TransactionUnit::tag`：t2 extent 树根、t2下@层级.序号 extent 下段叶、t3 inode 叶容器（最左一片）、t4 inode 树根、t5 分配记录树根、t5@层级.盘.序号 分配记录树根之下的节点、t6 记账树根、t7 映射树、t8 树表、td `deleted_inodes` 根、tt@叶序 墓碑容器、tt 墓碑树根、tr@叶序 退役容器、tr 退役树根。

| 发布 | 单元数（每盘） | 段串（两块盘） | 出处 |
|---|---|---|---|
| 意图 | 13：t3 t4 t5@0.0.61 t5@0.1.61 t5@1.0.0 t5@1.1.0 t5 t6 td tt@0 tt t7 t8（harness 钉住）；副本原型量到的 10，加 `deleted_inodes` 根、墓碑容器、墓碑树根各 1 | `26+2+1+2`（推的，待干跑核） | `crates/singlefs-harness/tests/unlink_single_file_publishes.rs` 的 `intent_rewrites_the_inode_leaf_and_root_the_deleted_inodes_root_and_the_tombstone_container_and_root`；`research/prompts/m3-delete-r3-main-verification.md` 第三节判 F1 状态数那一格 |
| 不是最后一批：3 → 2（批大小 1） | 12：t2下@0.0 t2 t3 t4 t5@0.0.61 t5@0.1.61 t5@1.0.0 t5@1.1.0 t5 t6 t7 t8（harness 钉住）；`deleted_inodes` 根与墓碑树照抄，不写 td、tt | `24+2+1+2`（推的，待干跑核） | `crates/singlefs-harness/tests/unlink_single_file_publishes.rs` 的 `a_batch_of_one_unit_rewrites_the_cut_lower_leaf_and_the_extent_root_and_releases_the_tail_unit` |
| 不是最后一批：2 → 1（批大小 1） | 11（推的，待干跑核）：同 3 → 2 那一批去掉 t2下@0.0，截到一个单元不建下段、只重写 extent 根 | `22+2+1+2`（推的，待干跑核） | `crates/singlefs-core/src/unlink.rs` 的 `a_deletion_batch_down_to_one_data_unit_drops_the_whole_lower_segment`（钉 extent 树的规划，没钉整次发布的角色集合） |
| 每一批的新写上界 | 随文件几何：只重写被截掉那一片 extent 下段叶与它的祖先；每批新写上界 W = 24 + (6 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26)) 槽（H_退役 是退役树的内部层数；6 + H_退役 是一次发布新写的退役单元上界） | — | D8（核心索引结构） 已定项 16；D28（挂载期承诺量） 已定项 1 |
| 最后一批（1 → 0，批大小 1） | 12：t2 t4 t5@0.0.61 t5@0.1.61 t5@1.0.0 t5@1.1.0 t5 t6 tr@0 tr t7 t8（harness 钉住）。比 2 → 1 那一批（11）多 1（推的，待干跑核）：少 inode 叶容器 t3，被删的那条记录是那片叶里唯一一条，叶删空整片不落盘；多退役容器 tr@0 与退役树根 tr，那片叶的身份消失，退役树开第一片容器、写根；叶里还有别的记录时那一片照写；删空的不是最右一片时只重写内容变了的叶与祖先、右边各片不挪位重写（判决 `research/prompts/m3-retire-r1-main-verification.md` 第六节 R3，第 50 批），这两支是代码文档注释的读法，harness 没钉。比 3 → 2 那一批少 t2下@0.0 与 t3（extent 树只剩上段零条目根），多 tr@0 与 tr，个数同为 12。`deleted_inodes` 删空取根指针置零，不写它的节点；不是最后一批的批同样不写 td（队列没变就照抄），所以 td 不是最后一批比一批少的那一个 | `24+2+1+2`（推的，待干跑核） | `crates/singlefs-harness/tests/unlink_single_file_publishes.rs` 的 `the_final_batch_removes_the_record_the_queue_key_and_every_remaining_unit_in_the_same_publish`；`crates/singlefs-core/src/inode_tree.rs` 的 `remove_record_from_leaf_containers`；`research/prompts/m3-delete-r2-main-verification.md` 第三节判空形态那一格 |
| 挂载续做 | 每一批与会话里同几何的那一批不同形：续做 2 → 1 与最后一批各比会话批多写分配记录树第 62 片叶一对（t5@0.0.62、t5@0.1.62），有条款依据（D8（核心索引结构） 已定项 14 按位置寻址），差别跟落点有没有跨进第 62 片叶走（E172（删除写出的字节的干跑） 第 2 次，2026-10-05）；harness 没钉续做批的角色集合 | 同几何的会话批的段串，落点跨进第 62 片叶时再多一对叶（E172（删除写出的字节的干跑） 实测，不再标推的） | `crates/singlefs-core/src/mount.rs` 的 `continue_the_deletion_queue_until_it_is_empty`（写行与暖机之后、交回之前走空）；`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` 的 `the_mount_hands_in_every_continuation_step_after_its_warm_up_in_publish_order`（钉续做三步接在暖机之后、按 txg 连着：留 2、留 1、最后一批） |
| 管理员回退 | 见 [layout/01-first-txn.md](01-first-txn.md) 段序列登记表「实例切换 / 管理员回退」那一行（第 412 行），这里不另抄 | 同那一行 | `research/prompts/m3-delete-r3-main-verification.md` 第三节判其余数那一格 |

整条流：unlink 一次调用 = 走空 + 意图 + 全部批（D8（核心索引结构） 已定项 16 第 6 条）；队列本来就空时走空一个写都不发，挂载续做的走空接在写行与暖机之后。发布与发布之间，上一次发布末尾的 2 个系统配置槽写自成一段（[layout/01-first-txn.md](01-first-txn.md) 段序列登记表后面「发布与下一次发布之间有发布末尾那道屏障」那一段），所以整条流的段串是各次发布的段串首尾相接：3 单元文件、批大小 1 的一次 unlink 是 `26+2+1+2`、`24+2+1+2`、`22+2+1+2`、`24+2+1+2` 四串相接（推的，待干跑核；崩溃状态数没算）。

```

**出处 `.claude/kb/layout/03-third-txn.md:66-80`（整段抄，未转述）**

```markdown
## 五、退役记录

| 偏移 | 字段 | 宽度 | 取值 | 指向 |
|---|---|---|---|---|
| 0 | 被退役身份·出生树 | 8 | 被退役的叶容器身份的出生树（单头时是 inode 树的树 ID） | D18（块里携带什么信息） 已定项 24 |
| 8 | 被退役身份·打包记录类型 | 2 | 被退役的叶容器身份的打包记录类型（inode 叶容器是 2） | D18（块里携带什么信息） 已定项 24 |
| 10 | 被退役身份·容器号 | 8 | 被退役的叶容器身份的容器号 | D18（块里携带什么信息） 已定项 24 |
| 18 | 被退役身份·容器出生代 | 8 | 被退役的叶容器身份的容器出生代 | D18（块里携带什么信息） 已定项 24 |
| 26 | 死亡 txg | 8 | 身份消失那次发布的 checkpoint_txg（叶删空摘掉、合并被吸收那一次）；不取 unlink 意图那次，不取最后一版的诞生代号 | D18（块里携带什么信息） 已定项 24 |

小端，共 34 字节，无填充、无 flags、无预留；前四段与 `PackedIdentity` 同序同宽。

<!-- format-const: RETIREMENT_RECORD_BYTES = 34 -->
<!-- format-const: RETIREMENT_CONTAINER_RECORDS = 959 -->

```

**出处 `.claude/kb/layout/01-first-txn.md:80-80`（整段抄，未转述）**

```markdown
| t12 | 事务 | 树表单元第 1 版（12 条条目：extent、inode、分配记录、记账四棵有根 + livelist、稀疏旁表、deadlist、`deleted_inodes`、墓碑树、谱系树、logged_ops、退役树八棵根指针为零）——最末 | 2 | 16384 | 50252 | — | 七 |
```

**出处 `.claude/kb/layout/01-first-txn.md:378-378`（整段抄，未转述）**

```markdown
| 树 ID 水位 | 8 | mkfs 65536，发布后 **65549**（deadlist 树 ID 65543 day-1 注册，D5（快照 / 空间记账机制） 已定项 11；`deleted_inodes` 等四棵 65544..65547 同批；退役树 65548 同批；只注册前八棵时 65544） | D8（核心索引结构） 已定项 8 ② / 已定项 11；D22（单元原子性怎么合成） 已定项 7 | 已定 |
```

**出处 `.claude/kb/layout/01-first-txn.md:392-392`（整段抄，未转述）**

```markdown
| 树表条目 | **树 ID 8**（打头，= 码 2 条目的 key）+ 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8 + **头 ID 8** + 预留 76 | 200 | 第 1 版 **12 条**（按树 ID 升序）：(树 ID 65536, extent, 种类 1, 根 50240, 头 ID 65537)、(65537, inode, 2, 根 50244, 头 ID 65537)、(65538, 分配记录, 3, 根 50249, 头 ID 0)、(65539, 记账, 4, 根 50250, 头 ID 0)、(65541, livelist, 6, 根指针零, 头 ID 0)、(65542, 稀疏旁表, 7, 根指针零, 头 ID 0)、(65543, deadlist, 8, 根指针零, 头 ID 0)、(65544, `deleted_inodes`, 9, 根指针零, 头 ID 65537)、(65545, 墓碑树, 10, 根指针零, 头 ID 65537)、(65546, 谱系树, 11, 根指针零, 头 ID 0)、(65547, logged_ops, 12, 根指针零, 头 ID 0)、(65548, 退役树, 13, 根指针零, 头 ID 65537)，十二条诞生 txg 都是 3、`previous_snapshot_txg` 0（0 = 不适用；只有 inode 树那条承载头的前驱快照，D5（快照 / 空间记账机制） 已定项 9）；树的种类的码按 D8（核心索引结构） 已定项 9 的登记表；头 ID 按 D5（快照 / 空间记账机制） 已定项 9 | D8（核心索引结构） 已定项 8 / 已定项 11 | 已定（2026-09-14 用户定案重排并加头 ID；2026-09-16 用户定案加宽到 200、预留 76） |
```

**出处 `.claude/kb/layout/01-first-txn.md:419-419`（整段抄，未转述）**

```markdown
| key 区间（偏移 52，两个 key 宽） | 2 × 8 | [65536, 65548]：最小 key 取 extent 树的树 ID，最大 key 取末条条目（退役树）的树 ID（`crates/singlefs-core/src/transaction.rs:15744 publish_admitted` 调 `build_index_node` 时传的两把 key；用例 `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:1419 tree_table_holds_twelve_entries_keyed_by_tree_identifier` 钉住这两个数） | D18（块里携带什么信息） 已定项 2 / 已定项 18 | 已定 |
```

**出处 `.claude/kb/experiments/172-删除写出的字节的干跑.md:28-43`（整段抄，未转述）**

````markdown
### 复跑

```
# 仓根：两份导出（峰值表里实测 30M、17M）
nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 60M cargo run -q -p singlefs-checker-tier --bin deletion_write_dump > <H1H2 导出>
nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 34M cargo run -q -p singlefs-checker-tier --bin deletion_write_dump -- --history h3 > <H3 导出>
# research/：装置（峰值 88M）
nice -n 19 cargo build --release --bin e172-deletion-write-dry-run
nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 176M ./target/release/e172-deletion-write-dry-run results/e172-r2-deletion-write-dump-h1h2-2026-10-05.out results/e172-r2-deletion-write-dump-h3-2026-10-05.out
# 登记的复跑（driver_e172：先现跑两份导出进临时文件，再跑装置）
bash scripts/experiment-apparatus/replay.sh E172
# 单测 38 条与变异表 32 条
cd scripts/experiment-apparatus && nice -n 19 bash ../hosts/run-with-memory-cap.sh 898M cargo test --release --bin e172-deletion-write-dry-run
cd scripts/experiment-apparatus && nice -n 19 bash ../mutation/mutate.sh e172-deletion-write-dry-run e172_deletion_write_dry_run.rs ../mutations/e172_deletion_write_dry_run.tsv
```

````

## 三、实现今天的样子

主工作区今天（2026-10-29 现查）的 `crates/`，行号是各文件自己的：

- 标签 6 的式子：`crates/singlefs-core/src/admission.rs` 第 347–385 行 `committed_reservation_for_tombstones_and_retirement_records`（`副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)`，第 338–342 行文档注释）；常量第 305 行（41 = ⌈32768 ÷ 815⌉）、第 315 行（35 = ⌈32768 ÷ 959⌉）；n_live 第 283 行 `live_objects_of_the_inode_tree`（nlink ≠ 0 的记录数），n_记录 第 295 行 `inode_records_of_the_inode_tree`（全部记录数）；输入枚举第 324–336 行（行 336 是枚举的收尾括号）。发布路径调它：`crates/singlefs-core/src/transaction.rs` 第 6817 行 `committed_reservation_for_tombstones_and_retirement_records_after_this_publish`，两个 H 从各自根节点码 2 头现读（第 6841 行 `retirement_tree_internal_levels_read_from_the_root_node_header`）。
- 钉值：`crates/singlefs-core/src/admission.rs` 第 2081 行起的用例钉 n = 1 ⇒ 196 760、n = 3 ⇒ 197 064（第 2099–2100 行），第 2105 行起 H_墓碑 = 1 ⇒ 229 528，第 2123 行起 H_退役 = 1、n_live = 0、n_记录 = 1 ⇒ 229 446；`crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` 第 113 行用例，第 107–111 行注释，第 169–177 行从镜像现算式子，第 198 行钉 `196_760, 196_678, 196_678, 196_678, 196_608`，第 247 行回退钉 `(196_608, 196_760)`。
- 退役树：`crates/singlefs-core/src/retirement_tree.rs`（490 行）文件头第 1–12 行（码 3 打包记录类型 3、959 条、34 字节记录，根码 2、条目 120，同墓碑树形态），第 182 行追加记录 `RetirementRecord { retired, death_txg: checkpoint_txg }`；记录字节 `crates/singlefs-core/src/records.rs` 第 680 行 `RetirementRecord`；角色名 `crates/singlefs-core/src/transaction.rs` 第 2829–2830 行 `tr@<叶序>`、`tr`；harness 钉最后一批角色含 `tr@0`、`tr`：`crates/singlefs-harness/tests/unlink_single_file_publishes.rs` 第 172 行用例、第 194–195 行。
- 删除写法（意图、一批、最后一批）：`crates/singlefs-core/src/transaction.rs` 第 10987 行 `publish_unlink_intent`、第 11047 行 `publish_unlink_step`、第 11827 行 `publish_deletion_step`；删记录 `crates/singlefs-core/src/inode_tree.rs` 第 1692 行 `remove_record_from_leaf_containers`。第 2 次登记第三节列的行号（第 5945、6000、6119 行等）今天都挪了，函数名照旧。
- 导出：`crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs`（386 行，sha256 `176c86c2…`，与第 2 次的 `9b3f565e…` 不同）文件头第 9–10 行：「**不带任何位置寻址的布局知识**」，段的边界与种类来自场景函数、角色名来自 core 交回的 `TransactionOutput::rewritten`；输出行只有 `impl_config`、`before_window_summary`、段边界、`device_region_bytes`、`window_barrier`、`device_region_bytes_summary`、`done`（第 110–266 行的 `println!` 与格式串），不打记账值、不打树表、不打任何解码出的字段。`grep -c 退役` 对它是 0。
- 场景：`crates/singlefs-harness/src/scenario.rs` 第 277–319 行 `DeletionHistory` 三个成员与名字 `h1`、`h2`、`h3`；第 614 行 `run_deletion_history`；第 634–645 行 H3 建 size 0 的 2 号 inode。
- 准入：`python3 scripts/gate/admission.py keys .` 列出的实验键里没有 E172（第十三节命令 9）；`.claude/gate.d/stage-inputs.tsv` 第 43–45 行是 E142 的三行（入口 `scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs::main` 等）。研究侧的准入入口 `scripts/experiment-apparatus/lib.rs` 第 135 行 `admit_experiment_run_or_exit`，E142 装置在 `main` 开头调它（`scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs` 第 7220–7225 行）。
- 复跑：`scripts/experiment-apparatus/replay.sh` 第 1755 行 `E172|@driver_e172||e172-r2-deletion-write-dry-run-2026-10-05.out|exact`，第 3167–3179 行 `driver_e172`（第 3173 行起函数体：先 `cargo run -q -p singlefs-checker-tier --bin deletion_write_dump` 两次进临时文件，再跑装置）；这份文件 `git status` 是 ` M`（别的会话的改动在里面，行号以开工时现取为准）。行键：`scripts/gate/admission.py` 第 2895–2945 行，驱动里 `-p <包> … --bin <名>` 点名的仓根工作区 bin（这里是 `deletion_write_dump`）的 `main` 进行键。

**退役项落哪一侧（派发提示 ①，定案：解码侧）**：导出只把窗口里每一次写的原字节、段边界与 core 交回的角色名打出来（上面「导出」那一条），退役容器与退役树根作为写出的单元、标签 6 作为记账树叶里的 8 字节，今天都已经在导出的 `device_region_bytes` 里；`rewritten_roles=` 由 core 的 `TransactionUnit::tag` 给，已经带 `tr@0`、`tr`。缺的是解码与期望：装置不认 `(3, 65548)`、`(2, 65548)` 两种单元（`role_name_of` 落到 `unknown@…`），标签 6 期望是按历史名分的字面常量 98 304 / 98 386（装置第 3129–3135 行），式子没有退役项（第 1616–1618 行），树表期望 11 条（第 1953 行）、水位 65548（第 104 行）。全部改在 `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs`，`crates/` 一个字不改，不派实现员。导出侧要改的情形只有一种：主 agent 要把 harness 第 1 个那一版（删除之前，196 760）也拿进 E172 比——那要导出把窗口往前挪到顺序写那次发布或另打那一版的记账叶，归实现员改 `deletion_write_dump.rs`；本登记不登记这一种（第一节 1.2 表第 1 行），列在交回的「待主 agent 认的项」里。

## 四、跑之前已经存在的数

照实列，不删。

| # | 在哪读到 / 怎么来的 | 是什么数 | 对判据的影响 |
|---|---|---|---|
| 1 | 第 2 次登记第四节第 1–9 条（第 2 次登记第 583–593 行，这一次整份读了第 2 次登记） | 第 1、2 次跑之前就有的那些数（参考导出的 txg、角色集合 13 / 12 / 11 / 10、取号 2 次、H3 锚点等） | 影响照第 2 次那一栏；这一次的门槛不取它们 |
| 2 | `.claude/kb/checks-owed.md` 第 483 行（C775 一行，第二节抄了） | 第 2 次产物上标签 6 的读数：H1、H2 每个删除版本 98 304，H3 98 386 | 第 2 次装置（没有退役项）在第 2 次导出上的读数；这一次 A-tag6 与 R-ret 的门槛来自 layout/03 式子与 harness 钉的值，不取它们。R-ret 里「去掉退役项的那一份」在今天的导出上会算出 98 304 / 98 386（第 7 条），与这两个数同值是式子决定的，不是答案泄漏 |
| 3 | E172 实验页一级标题与 `### 做了什么` 之前的正文（第 1–19 行，开工判问法时读到） | 第 2 次：问题单五行都够判；279 行 L03 格行可分辨的 0 行不等；38 次系统配置槽写 T_time 都是 5000；没登记的字节 0；角色与段串相同；R8.c 归错 0、取号 2 次；适用的 18 对阳性对照先绿后红；变异表 32 条全抓；A-journal 在 H1、H3 上共 8 行比实际少 8192 字节（登记推锚点用的「计数器 = txg」推错） | 第 2 次在退役树加进之前那一版 `crates/` 上「逐字段相等」是跑之前就知道的；第 2 行问的是加了退役项之后、在今天的导出上还相不相等，这一次全部重判，不沿用。A-journal 是登记自己推锚点的错，不是字段：这一次照第 2 次的推法原样保留、附带照报，不进第 2 行判定（第六节 6.4），也不拿第 2 次的读数改它（改了就是把答案写进锚点） |
| 4 | E172 实验页 `### 做了什么`（第 21–26 行） | 第 2 次两份导出 sha256 `b7e87c4e…`（462 行）、`9571c904…`（150 行）；第 2 次产物 sha256 `4a03f73e…`、817 行；第 2 次 `deletion_write_dump.rs` `9b3f565e…`、`scenario.rs` `9d007793…` | 进第十一节比对对象表（现取 sha256 与行数核过，第十三节命令 7）；与今天的 `176c86c2…`、`e16f73ef…` 不同，⇒ 今天的导出与第 2 次不会逐字节相同，是跑之前就知道的 |
| 5 | 读 `crates/`（第三节） | harness 钉的 196 760 / 196 678 / 196 608、回退 (196 608, 196 760)，`admission.rs` 钉的 196 760、197 064、229 528、229 446；harness 钉最后一批角色含 `tr@0`、`tr`，layout/03 写最后一批 12 个单元 | **钉值层的答案在 `crates/` 里已经存在**，那是比对的目标本身（问题单第 2 行就是拿它们比）。这一次能新加的证据：独立解码器在今天的导出（H1、H2、H3 三段，H2 走挂载续做、H3 叶里多一条记录）的字节里解出的标签 6、退役容器、退役记录、退役树根、树表第 12 条与段串，与条款、与钉的值逐格比 |
| 6 | 读装置（第三节、第五节 5.8） | 第 2 次装置的期望：标签 6 字面常量 98 304 / 98 386、式子不带退役项、树表 11 条、水位 65548、最后一批角色不含 tr；单测里断言 98 304 / 98 386 的四处（第 3907–3909、4106–4115、4287–4295 行） | 这一次改掉的就是这些（第五节 5.8）；它们是第 2 次登记的期望，不是结果 |
| 7 | 判问法时自己算的（第十三节命令 6，`anchors_r3.py`） | 带退役项：H1 / H2 意图、3 → 2、2 → 1 = 196 678，最后一批 196 608；H3 意图、3 → 2、2 → 1 = 196 830，最后一批 196 760；删除之前那一版 H1 196 760、H3 196 912。去掉退役项：H1 / H2 98 304（每一版）；H3 98 386（每一版）。两份之差 98 374 / 98 304 / 98 444。⌈32768 ÷ 959⌉ = 35，⌊(32768 − 136) ÷ 34⌋ = 959；H_退役 = 1 时 229 446，H_墓碑 = 1 时 229 528；树表 12 × 200 = 2400 | 第七节锚点、R-ret 的两份期望与合成单测 T14–T16 的取样点。判别力（第 3a 步 ②）：标签 6 是 8 字节整数，比较的分辨率是 1 字节；判别臂要分辨的差 ≥ 98 304，远大于分辨率；「相对变化」类形式不用，没有退化 |
| 8 | 第 3a 步 ③ 现算的触发条件 | 退役记录只在「叶容器身份消失」的发布里写（layout/03「退役容器」那一行；`retirement_tree.rs` 第 182 行）。H1、H2：被删的记录是那片 inode 叶里唯一一条，最后一批删空整片 ⇒ 写 1 条退役记录、开容器 1、写根，H_退役 = 0（根直接管容器，层级 1）。H3：叶里还有 2 号，最后一批照写那片叶 ⇒ 不写退役记录，退役树一直零根、树表里它的根指针一直是零。退役树长高要一个根装满（推的：一个根至多 (16384 − 131) ÷ 120 取整 = 135 片容器，每片 959 条），三段历史各至多 1 条退役记录 ⇒ 不会长高；墓碑树同理不长高 | 三段历史里 H_退役、H_墓碑恒 0：标签 6 的两个 H 只在 0 这一个取样点上量（第八节记「只在一个取样点量过」，合成单测补 H = 1 的取样点）。tr@0、tr 在 H1、H2 最后一批出现、H3 不出现——H3 是方向相反的取样点（第八节） |
| 9 | `ls research/results/ \| grep -i e172` 与 `git log --diff-filter=D` 的文件名 | 工作区只剩第 2 次两份导出；第 2 次产物在 59ce5a865 删了，第 1 次三份与第 2 次变异日志在 04ceaa904 删了 | 只看了文件名、sha256 与行数，内容没读；这一次的产物取新文件名（第五节 5.7） |

## 五、臂、阳性对照、真实基线

**装置写在哪（写死）**：照第 2 次，装置写在 `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs`（研究侧，`implementation-first.md` 第 4 条：不 `use` 任何 `singlefs-*` crate，偏移与取值从 layout/01、layout/03 抄成本地常量、每个常量旁写「`<kb 文件>` 第几行」）。它不是入库装置，只做解码与比对、不手写独立模型。`crates/` 侧只读导出照旧由 `deletion_write_dump`（不带参数与 `--history h3` 各一份）在主工作区里跑，装置读这两份输出文件；`deletion_write_dump.rs` 不改（第三节末「退役项落哪一侧」）。新加的常量（退役树树 ID 65548、种类码 13、打包记录类型 3、记录宽 34、容器装 959 条、退役份额每条 35、水位 65549、树表 12 条、harness 钉的三个数）都写成本地常量：前六个旁写 layout/03 的行（「退役容器」「树表退役树条目」「记账标签 6 的值」那几行与第五节），水位与 12 条旁写 layout/01 的行，harness 钉的三个数旁写 `crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` 第 198 行——只抄数、不引 `crates/` 的常量与函数。

### 5.1 被比的两边

- 「实装」：今天主工作区 `crates/` 在 H1、H2、H3 上写出的每一个字节（两份新导出）。
- 「登记」，两层（第一节 1.2）：条款层 = layout/03、layout/01 今天的取值与偏移（连第 2 次登记第二节抄的 D8、D18 偏移表）；钉值层 = harness 第 198 行钉的三个数，按第一节 1.2 那张表对版本。
- 这次没有设计上的对立臂；第 2 行的两个候选在 5.2。判别臂是「去掉退役项的期望」（第一节 1.2 最后第二条，第六节 R-ret）。

### 5.2 第 2 行的两个候选（写成支持它的人认的样子，不带倾向）

- 「甲 加退役项之后逐字段相等」：退役树是照墓碑树的形态加的（layout/03「退役容器」那一行写「规则同墓碑容器」「树根与墓碑树同形」），第 2 次装置对墓碑容器、墓碑树根、树表四条新条目、标签 6 的墓碑份额都核过相等；退役那几格与它们同一套码 3 / 码 2 写法、同一个记账行，只换树 ID、打包记录类型、记录宽与式子里多出来的一项，装置照着加上之后，今天的导出逐格对得上，标签 6 每一版等于 harness 钉的值。
- 「乙 有字段不等」：退役树是第 50 批之后才加进 `crates/` 的，第 2 次之后 `crates/` 别处也动过（导出的 sha256 已经不同，第四节第 4 条）；退役记录的「容器出生代」「死亡 txg」、退役树根的 key 区间、树表第 12 条的头 ID、水位、最后一批的段串，任何一格都可能与 layout/03、layout/01 今天的写法不同；挂载续做（H2）与叶里多一条记录（H3）这两种历史 harness 没钉标签 6，可能与式子不同；或者装置之外的老格（第 2 次判过的那几百行）在今天的导出上有不等。

### 5.3 历史

H1、H2、H3 定义照第 2 次登记第一节 1.2（第 2 次登记第 53–56 行）；H4 不跑。三段的退役树触发情形写在第四节第 8 条。

### 5.4 真实基线与阳性对照

- 真实基线：今天主工作区两份导出的原样输出。
- 阳性对照：装置内建，对**每一段历史**各跑一遍（循环遍历 H1、H2、H3，不适用的历史报一行 `applies=false` 与原因）。每条在内存里改坏那一段历史导出的一份副本，重跑全部检查，报「改坏之前目标行是绿的」与「改坏之后目标行转红」。
- 第 2 次的八条（PC-A、PC-B、PC-C、PC-D、PC-F、PC-G、PC-H、PC-I）定义、适用历史、目标行照第 2 次登记第五节 5.4 那张表（第 2 次登记第 620–629 行）原样，不改。新加三条：

| 对照 | 适用 | 怎么造 | 目标行（改坏之前必须绿） | 改坏之后预期红在哪 |
|---|---|---|---|---|
| PC-J | 最后一批写了退役容器的历史（H1、H2）；H3 报 `applies=false`（不写退役记录，第四节第 8 条） | 最后一批 tr@0 里第一条退役记录偏移 26 起 8 字节死亡 txg 加 1，按码 3 规则重封载荷校验和（同 PC-I 的重封，第 2 次登记第 629 行） | L03-16 `death_txg` 相等 | L03-16 `death_txg` 不等，别的 L03-16 字段照旧相等 |
| PC-K | H1、H2、H3 | 意图那次发布写出的记账树叶（t6）里 key 标签 6 那一行的 value 减 70（= 2 × 35，去掉一条记录的退役份额），按 D18 已定项 18 重封载荷 CRC（同 PC-B、PC-C 的重封法） | 意图那一版 A-tag6 相等、L03-13 相等、钉值层那一行（H1、H2）相等 | 三行在意图那一版都不等（`actual` = 期望 − 70）；R-ret 那一版「去掉退役项的一份」照旧不等 |
| PC-L | H1、H2；H3 `applies=false`（退役树零根） | 最后一批 tr 的第一条条目偏移 0 起 8 字节分隔 key（容器号）1 改成 2，按 D18 已定项 18 重封载荷 CRC | L03-17 那一行相等 | L03-17 那一行不等（`separator2`） |

  - 适用的 (对照, 历史) 对共 25 个：第 2 次的 18 个（第 2 次登记第 632 行）加 PC-J 2、PC-K 3、PC-L 2。装置报 `positive_controls_run=` 应为 25；不是 25 时逐行列出哪一对没跑，按第十一节 V4 判。
  - PC-F、PC-G 的目标行是 R8.c（附带）：它们没转红只作废 R8.c 那几行，不作废第 2 行用到的格（第十一节 V4）。其余 23 对的目标行都在第 2 行射程里。

### 5.5 分段（按岔路排序；每段交回附问题单第 2 行的岔路表，主 agent 对着它判续不续）

只有一段：一份产物同时报两层与判别臂，第 2 行要「逐字段比完」才够判，拆不开。段内的次序就是做的次序，做不完时从前往后交：

1. S0–S6 停机核对（第十一节）、第 2 次登记第七节的锚点在今天的导出上照报。
2. 钉值层与判别臂：A-tag6 改成按版本的字面常量、harness 钉值那几行、R-ret、PC-K、合成单测 T14–T16。补完这一步，第 2 行的翻面观测（与钉的三个数比）与判别力两样就有了。
3. 条款层的退役那几格：L03-12（12 条与退役树那一条）、L03-13（式子带退役项）、L03-14（水位 65549）、L03-15、L03-16、L03-17、角色名与最后一批角色集合（R3.c）、段串（R3.a、R3.b、R3.d）、覆盖（R2.a 认领退役单元），连同 PC-J、PC-L。
4. 第 2 次第 1 行那一套其余各格（R1.a、R1.b、R1.t、R2.b、R2.c、R3.e、R7）在今天的导出上照判。补完 1–4，第 2 行够判。
5. 附带：R4、R5、R6、R8 照算照报（装置已有，不另写），R-flip、R-dump（第六节 6.4）。

变异表整表（第九节）由主 agent 在这一段交回之后跑一次，不算一段：它不翻第 2 行，只管证据强度。

### 5.6 实现量粗估（推的，没量过）

- 装置改动：常量与角色名约 20 行；最后一批角色集合两支各改一处；L03-12 / 13 / 14 改期望约 30 行；L03-15、L03-16、L03-17 三格照 L03-9、L03-10、L03-11 的写法新写约 120 行；覆盖认领 `(3, 34)` 约 5 行；A-tag6 按版本、钉值层几行、R-ret 约 80 行；PC-J、PC-K、PC-L 约 120 行；main 接准入约 5 行；R3.d 期望与段串表约 20 行。合计约 400 行（第 2 次装置 4433 行）。
- 单测：改 6 条、新加 11 条（第 5.8 节 T14–T24）；变异表加 12 条（第九节 M31–M42）。
- 历史 × 几何 × 臂：三段历史 × 一个几何（批大小 1、两块 4 GiB 盘）× 两份期望（带 / 去掉退役项）；没有崩溃支线、没有坏镜像（阳性对照的改坏副本在进程内，25 对）。

### 5.7 跑法（执行员照这个次序；标「主 agent 跑」的由主 agent 跑）

0. 核 S0、S1（第十一节）。第 2 次的装置与变异表已进 git（`git status` 干净，最后一次是 f4605214d），原地改之前不另存副本；要第 2 次那一份时 `git show f4605214d:scripts/experiment-apparatus/e172_deletion_write_dry_run.rs`。
1. 导出（主 agent 跑：`cargo run -p singlefs-checker-tier` 由重型测试闸按重型判，子 agent 起不了；`<日期>` 取跑的那天 `TZ=Asia/Tokyo date +%F`）：
   - `cd /home/fy5090/code/singlefs && SINGLEFS_HEAVY_TESTS=… nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 4G cargo run -q -p singlefs-checker-tier --bin deletion_write_dump > research/results/e172-r3-deletion-write-dump-h1h2-<日期>.out`（重型前缀照 `.claude/hooks/heavy-test-guard.sh` 文件头写全，范围声明照 `.claude/hooks/run-scope-guard.sh` 文件头）
   - 同上，命令末尾加 `-- --history h3`，输出到 `research/results/e172-r3-deletion-write-dump-h3-<日期>.out`
   - 两条各再跑一遍写到草稿目录，两两比 `sha256sum`（V3）。两份都整份入库。
2. 准入（主 agent 定接，第十一节 11.3）：主 agent 先在 `.claude/gate.d/stage-inputs.tsv` 加 E172 一行（执行员的写范围里没有这份表），执行员再改装置 `main`；不接就跳过这一步，`main` 不动这一处。
3. 装置按第 5.8 节改，单测：`cd /home/fy5090/code/singlefs/scripts/experiment-apparatus && nice -n 19 bash ../hosts/capped.sh <线程上限> bash ../hosts/run-with-memory-cap.sh 4G cargo test --release --bin e172-deletion-write-dry-run`（范围声明照写）。
4. 产物：`cargo build --release --bin e172-deletion-write-dry-run`，再 `nice -n 19 bash ../hosts/run-with-memory-cap.sh 4G ./target/release/e172-deletion-write-dry-run ../../research/results/e172-r3-deletion-write-dump-h1h2-<日期>.out ../../research/results/e172-r3-deletion-write-dump-h3-<日期>.out > ../../research/results/e172-r3-deletion-write-dry-run-<日期>.out`。同一个二进制跑两遍比 sha256（R-det）。接了准入的，产物最前面是装置打的 `E7INPUT` 行。
5. `scripts/experiment-apparatus/replay.sh`（派发提示 ③）：第 1755 行改成 `E172|@driver_e172||e172-r3-deletion-write-dry-run-<日期>.out|exact`；`driver_e172` 两个临时文件改名 `$OUT_DIR/e172-r3-deletion-write-dump-h1h2.tmp`、`$OUT_DIR/e172-r3-deletion-write-dump-h3.tmp`，命令不变；第 3167–3172 行注释改指本登记第五节 5.7 第 5 步与两份 r3 导出的文件名，加一句「第 2 次产物 `e172-r2-deletion-write-dry-run-2026-10-05.out` 在 59ce5a865 归档删了、不再登记复跑：退役树加进 `crates/` 之后导出变了，它复现不出来」。行号以改的那天现取为准（这份文件别的会话也在改）。只有一行复跑，`ADMISSION_ROW_KEYS` 不加行（准入键默认是实验号、产物头由装置写，`scripts/gate/admission.py` 第 126–131 行）。
6. 变异表整表由主 agent 跑：`cd /home/fy5090/code/singlefs/scripts/experiment-apparatus && nice -n 19 bash ../mutation/mutate.sh e172-deletion-write-dry-run e172_deletion_write_dry_run.rs ../mutation/tables/e172_deletion_write_dry_run.tsv`（照实验页「复跑」那一段的写法，路径以 `mutate.sh` 文件头为准）；收尾要有「已还原，基线仍全绿」，按 `mutation-sampling.md` 那几栏数，整表 44 条。
7. 比对对象（第十一节 11.4）：R-flip、R-dump 两样由执行员在草稿目录里算，结果行进产物之外的一份 `research/results/e172-r3-comparisons-<日期>.out`。

### 5.8 装置要改的地方（行号是今天这一份 `2960edb5…` 的；编号接第 2 次登记的改1–改15）

| # | 今天的位置 | 改成什么 |
|---|---|---|
| 改16 | 常量，第 88–104 行一带 | 加退役树树 ID 65548、种类码 13、打包记录类型 3、记录宽 34、每条退役份额 35、树表第 1 版条目数 12（各旁写出处行，第五节开头）；第 104 行水位 65548 → 65549（旁写 layout/01「树 ID 水位」那一行）；加 harness 钉值两个常量 196 678、196 608（旁写 harness 第 198 行，不引 `crates/`） |
| 改17 | `role_name_of`，第 1131–1163 行；它的调用方第 1323–1328 行 | 加 `(3, 退役树, _) => tr@<叶序>`、`(2, 退役树, _) => tr`；叶序照墓碑容器那一种算（从这一段写出的退役树根、没有就取现行那一版的，取它条目里的容器号排序） |
| 改18 | `registered_roles`，第 1522–1553 行 | 最后一批 `OnlyTheDeletedRecord` 那一支在 t6 之后加 `tr@0`、`tr`（layout/03 第四节「最后一批」那一行的次序）；`BesideOtherRecords` 那一支照旧写 t3、不加 tr。段串、种类串与整窗段串（R3.a、R3.b、R3.d）由这一份角色集合算，跟着变：H1 最后一批 `24+2+1+2`，H1 整窗四串 `26+2+1+2`、`24+2+1+2`、`22+2+1+2`、`24+2+1+2` 首尾相接；H3 照旧 |
| 改19 | `claim_container_records`，第 898–920 行 | 加 `(3, 34)` 那一类整条记录认领成 layout/03 格（`ByteClass::LayoutThreeCell`），同墓碑记录那一行 |
| 改20 | `judge_tree_table_accounting_and_watermark_cells`，第 1951–1985 行 | L03-12 条目数期望 11 → 12；四棵新树那张小表加 `(退役树, 13, inode 树, 根指针零)`，根指针零 = 不是「最后一批且走 `OnlyTheDeletedRecord` 那一支」；另加一格树表码 2 头 key 区间期望 `[65536, 65548]`（layout/01 树表头「key 区间」那一行） |
| 改21 | `expected_committed_reservation`，第 1616–1618 行；`tombstone_internal_levels` 第 1609–1614 行；L03-13 第 1972–1978 行 | 式子加 `2 + H_退役 + 1` 三槽与 `35 × n_记录`；照 `tombstone_internal_levels` 写 `retirement_internal_levels`（退役树根指针零时 0，否则根层级 − 1），照 `live_inode_count` 写 `inode_record_count`（inode 树里全部记录、含 nlink = 0 的）；L03-13 的 field 名带四个输入 |
| 改22 | 新加，放在 L03-11 之后 | L03-15 退役容器：写了的那一次（H1、H2 最后一批）期望 `code3/type3/width34/birth_tree65548/records_at136/container1/count1`；别的删除类发布报 `retirement_container_written` 期望 `false` |
| 改23 | 新加 | L03-16 退役记录（layout/03 第五节，五格各一行）：出生树、打包记录类型、容器号、容器出生代四格的期望取「上一版里装着被删记录的那片 inode 叶」的码 3 头（出生树 = 头里偏移 43、打包记录类型 = 偏移 51、容器号 = 偏移 53、容器出生代 = 偏移 61，装置第 985–990 行已解）——条件是条款的输入、不是答案，同 L03-7 的取法；死亡 txg 期望 = 这一次发布的 txg（layout/03 第五节偏移 26「身份消失那次发布的 checkpoint_txg」） |
| 改24 | 新加 | L03-17 退役树根：写了的那一次期望 `tag2/tree65548/key_width8/width120/count1/separator1/birth_tree65548/type3/container1/children<tr@0 的落点>`，同 L03-11 的拼法；层级与 key 区间只记值，加一行 R7 那一类的 `key_range_line`（`tr`） |
| 改25 | `anchor_lines_of_history` 第 3099–3100 行；`registered_committed_reservation_of_history` 第 3128–3135 行 | A-tag6 改成按（历史，删除类发布）分的字面常量：H1、H2 意图与不是最后一批的批 196 678、最后一批 196 608；H3 不是最后一批 196 830、最后一批 196 760（第七节 7.1）。不由解码的 n_live、n_记录现算 |
| 改26 | 新加，挨着 A-tag6 | 钉值层：H1、H2 每个删除类发布一行 `name=harness_pin history= txg= publish= harness_index= pinned= actual= equal=`（`harness_index` 是第 198 行那一格的序号 2–5，对法照第一节 1.2 那张表）；每段历史一行 `name=harness_pin_out_of_window history= harness_index=1 pinned=196760 in_window=false`；H3 最后一批一行 `name=harness_pin_same_value history=h3 txg= actual= same_value_as_harness_first=` |
| 改27 | 新加 | R-ret：每个删除类发布一行 `name=r_ret history= txg= publish= with_retirement= without_retirement= actual= with_equal= without_equal= discriminating=`（`discriminating` = `!without_equal`：去掉退役项的那一份不等）；两份期望都由同一个式子函数算，去掉退役项的那一份只把 H_退役、n_记录 两项与那三槽置零，写成一个能单独改的参数（M31 的锚点） |
| 改28 | `PositiveControl` 第 2734 行起（名字映射到第 2782 行），`corrupted_copy` 第 2862 行起，`positive_control_line` 第 3040 行起 | 加 PC-J、PC-K、PC-L（第 5.4 节）；`positive_controls_expected` 18 → 25 |
| 改29 | `trajectory_lines`，第 3157 行起 | 每版加报 `inode_records=`、`retirement_records=`、`retirement_levels=`，连同标签 6 照报峰值、为正的版本数、期末值 |
| 改30 | `main` 第 3769 行起；文件头第 13 行；单测常量第 3813–3814 行 | 接准入的：`main` 开头照 E142 装置第 7220–7225 行调 `e7_index_bench::admit_experiment_run_or_exit("E172")`、把交回的行打在产物最前面，文件头那一行照装置接了准入改写；单测常量指到两份 r3 导出。第 2 行的判定行加 `name=verdict row=c775 pin_not_equal= clause_not_equal= retirement_cells_not_equal= r_ret_not_discriminating= positive_controls_failed_in_scope=`，各报各的数，不合成 |

单测（`mod tests`，第 3806 行起）：

- 改 6 条：`committed_reservation_formula_gives_98304_without_live_inodes_and_98386_with_one`（第 3907 行）改名、期望换成带退役项的四个值（第七节 7.1）；`committed_reservation_rows_read_98304_in_every_h1_deletion_version_and_expect_98386_in_every_h3_one`（第 4106 行）改名、期望按版本；`h3_committed_reservation_anchor_expects_98386`（第 4289 行）改成按版本的四个常量；PC 那条（第 4180 行一带，`not_applicable` 加 `PC-J@h3`、`PC-L@h3`，held 25）；R3.d 那条（第 4201 行一带，H1 整窗串换成改18 那一串，H3 不变）；两个导出常量（第 3813–3814 行）。
- 新加 11 条：T14 合成式子 H_退役 = 1、n_live = 0、n_记录 = 1 ⇒ 229 446；T15 合成 H_墓碑 = 1、n_live = n_记录 = 1 ⇒ 229 528；T16 合成 n_live = 0、n_记录 = 1 ⇒ 196 678 与 n_live = 1、n_记录 = 0 ⇒ 196 690（两个系数互换就分得开）；T17 合成单元 `(3, 65548)`、`(2, 65548)` 认成 `tr@0`、`tr`；T18 `registered_roles` 最后一批两支：唯一一条那支含 `tr@0`、`tr` 不含 t3，另有记录那支含 t3 不含 tr；T19 三段历史 R-ret 行共 12 行、期望两栏等于第七节 7.1；T20 钉值层 H1、H2 各 4 行、`pinned` 一栏 196 678 ×3 与 196 608，另有 `in_window=false` 一行；T21 PC-J 的副本恰改了一条退役记录的死亡 txg、码 3 载荷校验和过，另断言 H1 最后一批 L03-16 `death_txg` 的期望一栏是 8（A-txg H1 最后一批）；T22 PC-K 的副本意图标签 6 减 70、PC-L 的副本分隔 key 为 2，两份载荷 CRC 都过；T23 L03-12 条目数期望一栏 12、水位期望一栏 65549、退役树那一条期望 `root_zero=false` 只出现在 H1、H2 最后一批；T24 合成码 3 单元（打包记录类型 3、记录宽 34、两条记录）经 `claim_container_records` 之后那 68 字节全归 layout/03 格。
- 新加的单测断言期望一栏、结构与合成输入，不把 r3 导出的实际读数写成断言（第 2 次留下的按回归照留，期望改了的跟着改）。

## 六、报哪些量与各自的判据

每个量各占一行、各报各的判定，不合成一个 true / false；第 2 行「甲 / 乙」的合取由人做。门槛都是「与条款取值、钉的值或第七节锚点相等」或「计数为 0」，不从「实装」那一边的定义推（实装那一边是「`crates/` 写出的字节」，不含任何取值）；判别臂 R-ret 的门槛「带的相等、去掉的不等」也不从两份期望的定义推出——两份期望各自等不等于解出的值，是读数决定的。

规模与挂钟（第 3a 步 ①，推的，依据是第 2 次实验页「复跑」那一段的峰值与第四节第 4 条的行数）：两份导出各几百行（第 2 次 462、150 行，今天多了退役单元那几次写，同一量级）；删除类发布 12 次（H1 4、H2 4、H3 4），L03 格行几百行，阳性对照 25 对、每对重跑一遍全部检查——都是一个进程里秒级的量；挂钟由编译（`singlefs-checker-tier` 一次、研究工作区一次）与变异表整表（44 条，每条一次编译加单测）主导，见第十四节。峰值内存：导出实测 30M / 17M、装置 88M（实验页「复跑」），给 4G 上限。

### 6.0 第 2 行对到哪些量

| 量 | 层 | 什么值让第 2 行翻向乙 |
|---|---|---|
| P-pin（钉值层，6.2） | 钉值层 | H1、H2 任一个删除版本 `equal=false` |
| A-tag6、L03-13（6.2） | 条款层 | 任一段历史任一个删除版本不等 |
| L03-12、L03-14、L03-15、L03-16、L03-17（6.1、6.2） | 条款层（退役那几格） | 任一格任一段历史 `equal=false` |
| R3.a–R3.d 的最后一批（6.2） | 条款层（角色与段串） | H1、H2 最后一批角色集合或段串与改18 的期望不等；H3 最后一批不等 |
| R2.a、R2.c（6.2） | 条款层（覆盖） | 任一段历史没登记字节 > 0 或恒 0 那一类非零 > 0 |
| R1.a、R1.b、R1.t、R2.b、R3.e、R7（6.2） | 条款层（第 2 次那一套） | 照第 2 次登记第六节 6.2 那一栏（第 2 次登记第 723–736 行）；R7 只记值、不翻面 |
| R-ret（6.2） | 判别力 | 任一删除版本 `discriminating=false`（去掉退役项的那一份反倒相等）⇒ 乙，走 F12 两边都查；带与去掉两份同时相等 ⇒ 第十一节 V7 |

够判（第 2 行的够判条件「重跑产物出来、逐字段比完；装置去掉退役项必不等」）= 产物出来、上表除 R-ret 外每一行在三段历史上都报齐（R1.b 每格每段历史命中 ≥ 1），R-ret 每个删除版本 `discriminating=true`（与 `with_equal` 分开报），第 2 行射程里 23 对阳性对照都先绿后红。

### 6.1 新加与改了的格（射程 = H1、H2、H3 的删除类发布）

格的定义、字段偏移、登记取值与出处照第 2 次登记第六节 6.1（第 2 次登记第 709–719 行，连它引的第 1 次登记那张表）原样适用；下面只写这一次改了与新加的。

| 格 | 期望 | 出处 |
|---|---|---|
| L03-12 | 条目数 12；退役树那一条 `kind13/head65537/root_zero<…>`，`root_zero` 在 H1、H2 最后一批为 false、其余为 true；树表 key 区间 [65536, 65548] | layout/03「树表退役树条目」那一行；layout/01 第 80、392、419 行 |
| L03-13 | 式子带退役项（改21）；三段历史两个 H 都是 0 | layout/03「记账标签 6 的值」那一行 |
| L03-14 | 根记录与两份 journal 记录的树 ID 水位 65549 | layout/03「树表四条新条目」那一行；layout/01 第 378 行 |
| L03-15 | H1、H2 最后一批：退役容器恰 1 片（每盘），`code3/type3/width34/birth_tree65548/records_at136/container1/count1`；其余删除类发布与 H3 每一次：没写退役容器 | layout/03「退役容器」那一行 |
| L03-16 | H1、H2 最后一批那条退役记录：偏移 0 出生树 = 上一版装着被删记录那片 inode 叶的出生树（65537）；偏移 8 打包记录类型 = 2；偏移 10 容器号、偏移 18 容器出生代 = 那片叶头里的两格；偏移 26 死亡 txg = 这一次发布的 txg | layout/03 第五节 |
| L03-17 | H1、H2 最后一批：退役树根 `tag2/tree65548/key_width8/width120/count1/separator1/birth_tree65548/type3/container1/children<tr@0 落点>`；层级与 key 区间只记值；其余删除类发布与 H3：没写退役树根 | layout/03「退役容器」那一行末段「树根与墓碑树同形」；墓碑树根那一行的条目宽 120 |

第 2 次的 L03-10、L03-11（墓碑容器与墓碑树根）、L03-8（`deleted_inodes`）照旧。

### 6.2 第 2 行的量

| 量 | 怎么算 | 门槛与判定 | 第 2 行；翻面取值 |
|---|---|---|---|
| P-pin | 改26：H1、H2 每个删除类发布解出的标签 6 与 harness 第 198 行对应那一格比；删除之前那一格报 `in_window=false`；H3 最后一批报同值一行 | 每行 `equal=true`；计数 `pin_not_equal=`（H1、H2 分开） | 任一 ≥ 1 ⇒ 乙，列版本、钉的值、解出的值 |
| A-tag6 | 改25：三段历史每个删除版本解出的标签 6 与第七节 7.1 的字面常量比 | 每行相等；`anchor_not_equal` 里 A-tag6 的条数另报 | 任一不等 ⇒ 乙，走 F3（出自条款的锚点，两边都查） |
| L03-12 – L03-17 | 6.1 那张表，每格每版一行 `name=l03_cell …` | 每行 `equal=true`；`retirement_cells_not_equal=` 计数（L03-12 退役那一条、L03-14、L03-15、L03-16、L03-17 与 L03-13 合计，逐段历史分开） | 任一 ≥ 1 ⇒ 乙 |
| R3.a、R3.b、R3.c、R3.d | 照第 2 次登记第 731–734 行，期望按改18 的角色集合 | 照第 2 次 | 最后一批或整窗任一不等 ⇒ 乙（第 2 次没有退役单元，这几行今天是新期望） |
| R2.a、R2.c | 照第 2 次登记第 728、730 行，退役记录由改19 认领 | 没登记 0、恒 0 区非零 0 | 任一段 > 0 ⇒ 乙 |
| R1.a、R1.b、R1.t、R2.b、R3.e、R7 | 照第 2 次登记第 725–727、729、735–736 行 | 照第 2 次 | 照第 2 次那一栏；不等 ⇒ 乙（与退役无关的不等，F1 两边都查时归「`crates/` 别处变了」还是「登记错」） |
| R-ret | 改27：每个删除版本两份期望各与解出的值比 | `discriminating=true` 每版都成立；`r_ret_not_discriminating=` 计数 | 判别力那一条；≥ 1 ⇒ 乙、走 F12；`with_equal`、`without_equal` 同为 true ⇒ V7（第十一节） |

规模（第 3a 步 ①）：P-pin 8 行、A-tag6 12 行、R-ret 12 行、L03 新格每段历史每版 6–10 行；比较是整数与定长串的相等，分辨率 1（第 3a 步 ②，第四节第 7 条）。

### 6.3 公共的量

| 量 | 怎么算 | 门槛与判定 | 对应 |
|---|---|---|---|
| R-traj | 照第 2 次登记第 765 行，三段历史逐版，加改29 的三列 | 报峰值、为正的版本数、期末值 | 第 2 行（标签 6 是准入谓词的输入，第八节） |
| R-det | 两份导出各跑两遍、装置产物跑两遍，比 sha256 | 相等 | 第十一节 V3 |
| R-pc | 第 5.4 节 25 对，每对一行 `pc= history= target_green_before= target_red_after=`，不适用的另报 `applies=false` | 每对 `true true`；`positive_controls_run=25` | 第十一节 V4；第 2 行够判条件 |

### 6.4 附带的量（装置照报，不进第 2 行判定）

| 量 | 怎么算 | 为什么附带 |
|---|---|---|
| R4.a、R4.b、R4.c、R5.a、R5.b、R6.a、R6.b、R8.a、R8.c | 照第 2 次登记第六节 6.3、6.4（第 2 次登记第 740–758 行），装置已有，不另写 | 问题单这一次没有一行问续做批同不同形、最后一批少写哪一个、意图时间、挂载那一截切不切得开；L03-4、L03-5 两格本身在 R1.a 里照判 |
| A-journal | 照第 2 次登记第七节 7.1（第 2 次登记第 784 行）的推法原样 | 登记自己推锚点的前提错，不是字段（第四节第 3 条）；不符照 F3 两边都查、记一笔，不进第 2 行 |
| R-flip | 第 3 次产物与第 2 次产物（第十一节 11.4 表第 1 行）按行键（行名加 `cell`、`field`、`history`、`publish`、`txg` 那几格）对齐，列出「第 2 次相等、这一次不等」与「这一次新出现」的行 | 只给 F1 两边都查时归因用（不等是退役那几格，还是第 2 次之后 `crates/` 别处变了），不翻面；够判后不跑：第 2 行判甲（没有不等的行）就不算 |
| R-dump | 两份 r3 导出与第 2 次两份导出（11.4 表第 2、3 行）按（历史、段的边界行、步号）对齐，数 `device_region_bytes` 不同的行数，按段的 `rewritten_roles` 里的角色分开报 | 同 R-flip，归因用；够判后不跑：同上 |

## 七、钉绝对值的断言

互比断言（R-ret 两份期望之间、P-pin 的 H2 对 H1、R3 最后一批对 2 → 1、R1.a 各批对上一版的照抄）旁边各钉一条绝对值，防「两边一起错」。第 2 次登记第七节（第 2 次登记第 771–797 行）的锚点除 A-tag6 外原样适用（A-journal 附带，第六节 6.4）；下面列这一次新加与改了的。

### 7.1 出自被测条款本身（不符走第十节 F1 / F3「条款可能错」，两边都查，不作废）

| 锚点 | 值 | 出处与算法 |
|---|---|---|
| A-tag6 | H1、H2：意图、3 → 2、2 → 1 各 196 678，最后一批 196 608；H3：意图、3 → 2、2 → 1 各 196 830，最后一批 196 760 | layout/03「记账标签 6 的值」那一行：2 × ((2 + 0 + 1 + 2 + 0 + 1) × 16384 + 41 × n_live + 35 × n_记录)；n_live、n_记录 照第四节第 7、8 条（H1 / H2：0 与 1、最后一批 0 与 0；H3：1 与 2、最后一批 1 与 1）。装置写成按（历史，发布）分的字面常量（改25），不由解码的计数现算 |
| A-ret-container | H1、H2 最后一批：退役容器 1 号、记录 1 条、记录宽 34、打包记录类型 3、出生树 65548、记录从 136 起 | layout/03「退役容器」那一行：「容器号从 1 起……零根时 1」「含 29 字节预留位的头 136」 |
| A-ret-record | H1 最后一批死亡 txg 8，H2 = 续做最后一批那次的 txg（由导出的段边界给，不另钉） | layout/03 第五节偏移 26；H1 txg 5–8 照第 2 次登记 A-txg（H1 意图 5、三批 6 / 7 / 8） |
| A-tree-table | 条目数 12；退役树那一条树 ID 65548、种类 13、头 ID 65537；树表 key 区间 [65536, 65548]；水位 65549 | layout/03「树表退役树条目」「树表四条新条目」两行；layout/01 第 80、378、392、419 行 |
| A-roles-final | H1 最后一批每盘 12 个单元 `t2 t4 t5@0.0.61 t5@0.1.61 t5@1.0.0 t5@1.1.0 t5 t6 tr@0 tr t7 t8`、段串 `24+2+1+2`；H1 整窗四串 `26+2+1+2`、`24+2+1+2`、`22+2+1+2`、`24+2+1+2` | layout/03 第四节「最后一批」那一行与「整条流」那一段 |

P-pin 的三个数不是锚点：它们是第 2 行的翻面观测本身，出自 `crates/` 的用例，不出自被测条款，也不是登记独立算的；它们与 A-tag6 在 H1 上同值是第四节第 5、7 条跑之前就知道的。

### 7.2 独立算出、用命令核过（不符就作废那一轮，第十一节 V5）

| 锚点 | 值 | 怎么算的 |
|---|---|---|
| A-content、A-crc、A-lines | 照第 2 次登记第 793–795 行 | 同第 2 次 |
| A-arith | 第四节第 7 条那张表的十二个值（带 / 去掉退役项）、T14–T16 的 229 446、229 528、196 678、196 690 | 第十三节命令 6（`anchors_r3.py`）与命令 10 现算；装置的式子函数在这几个输入上算出的数与它不同 ⇒ 装置算术错（T14–T16、T19 判） |

每一条互比断言旁边那一条绝对值：R-ret ↔ A-tag6 与 A-arith；P-pin（H2 对 H1 同值）↔ A-tag6 的 H1 / H2 字面常量；R3 最后一批 ↔ A-roles-final；L03-16 的四格身份（取自上一版那片叶）↔ A-ret-container 的容器号 1 与 A-ret-record 的死亡 txg。

## 八、轨迹与几何敏感性

### 8.1 轨迹

照第 2 次登记第 801–803 行，三段历史逐版报 R-traj，加 `inode_records`、`retirement_records`、`retirement_levels` 三列，各报峰值、为正的版本数、期末值；只报期末值的，结论里不写「从来」「恒」。标签 6 是准入式子的输入：H1、H2 预期 196 678 → 196 608，H3 196 830 → 196 760，照报轨迹；H2 窗口里那几次不是删除类发布的版本一并报，不判。

### 8.2 几何敏感性（`mutation-sampling.md` 第六类）

| 判决 | 取样点 | 方向相反的取样点 | 怎么报 |
|---|---|---|---|
| L03-15 / L03-16 / L03-17、R3 最后一批（退役树写不写） | H1、H2：被删记录是那片叶里唯一一条，最后一批删空整片、写 tr@0 与 tr | H3：叶里还有 2 号，最后一批照写 t3、不写退役单元、退役树一直零根 | 三段历史各报；判别力自证：把 H1 最后一批的角色期望（含 tr@0、tr、不含 t3）拿去判 H3 最后一批、把 H3 的（含 t3、不含 tr）拿去判 H1，各报一行 `exactly_one_green=`（恰一套判绿才说明这一格分得出两种形状；两套都红交 F2） |
| A-tag6 的 n 两项 | H1 / H2：n_live 0、n_记录 1 → 0 | H3：n_live 1、n_记录 2 → 1 | 判别力自证：H1 的四个常量拿去判 H3 的四版、H3 的拿去判 H1，各报一行 `exactly_one_green=`；T16 合成取样点把 41 与 35 两个系数分开 |
| A-tag6 的两个 H | 三段历史 H_墓碑 = H_退役 = 0 | 没有：这几段历史里两棵树都长不高（第四节第 8 条） | 记「只在 H = 0 一个取样点量过」；式子在 H = 1 上的敏感由 T14、T15 两条合成单测给（M33、M34 的取样点） |
| P-pin | H1（与 harness 同一种历史） | H2（挂载续做，harness 第 198 行没钉这一种） | 两点各报；H2 与 H1 每一版同值才说明钉值与续做无关，不同交 F11 |
| 第 2 次那一套格（R1、R2、R3 其余、R7） | 照第 2 次登记第 805–815 行 | 照第 2 次 | 照第 2 次 |

## 九、变异

变异表仍是 `scripts/mutation/tables/e172_deletion_write_dry_run.tsv`，改的是装置自己的期望与判定代码（不改 `crates/`）。单测读两份新入库的 r3 导出（找不到就判红，不跳过），另有不依赖导出的合成单测。

**第 2 次的 32 条**原样留着，含义不变。第 5.8 节的改动会挪动或改写其中几条的原文（A-tag6 那几条：M10b 打式子里的 41、M29 打 H3 的字面常量 98 386）：执行员把原文改成改动之后源码里唯一命中的那一段、替换文按同一个意思改（M29 改打 H3 最后一批的常量 196 760 → 196 678），逐条在报告里写「旧原文 → 新原文」，不改它要打的那件事。

**新加 12 条**：

| 变异 | 改什么 | 在哪个取样点上改变输出 |
|---|---|---|
| M31 | R-ret 去掉退役项那一份的参数改成与带的那一份相同（判别臂退化成被测臂） | 三段历史每个删除版本：`without_equal` 变 true、`discriminating` 变 false；T19（两栏期望不同）红 |
| M32 | 式子里退役份额每条 35 → 34 | H1 意图那一版：L03-13 期望 196 676 ≠ 196 678；T16 红 |
| M33 | 式子里退役三槽 `2 + H_退役 + 1` → `2 + H_退役` | 每一版期望少 32 768；T14、T16 红 |
| M34 | `retirement_internal_levels` 取根层级不减一 | H1、H2 最后一批（退役树有根、层级 1）：期望多 32 768 = 229 376 ≠ 196 608；H3 与前三版不敏感（零根，第三类）；改过的标签 6 那条单测（第 4106 行一带，按版本的期望一栏）红 |
| M35 | `inode_record_count` 只数 nlink ≥ 1 的记录 | H1 意图、3 → 2、2 → 1：n_记录 读成 0，期望 196 608 ≠ 196 678；最后一批不敏感（两者都是 0，第三类）；同上那条单测红 |
| M36 | A-tag6 字面常量 H1 最后一批 196 608 → 196 678 | H1 最后一批；改过的 H3 / H1 常量那条单测（第 4289 行一带）红 |
| M37 | P-pin 的对法偏一格（意图那一版对第 1 格 196 760） | H1、H2 意图；T20（`pinned` 一栏）红 |
| M38 | `role_name_of` 把 `(2, 65548)` 认成 `tt` | 合成单元：T17 红；导出上 H1、H2 最后一批 S3 角色对拍不等 |
| M39 | `registered_roles` 最后一批唯一一条那支去掉 `tr@0`、`tr` | T18 红；H1 整窗段串那条单测（第 4201 行一带）红 |
| M40 | `claim_container_records` 去掉 `(3, 34)` 那一类 | T24 合成单元红；导出上 H1、H2 最后一批 R2.a 没登记字节 68 |
| M41 | L03-16 死亡 txg 的期望取意图那次的 txg（layout/03 第五节明写「不取 unlink 意图那次」） | H1 最后一批期望 5 ≠ 8；T21 红 |
| M42 | L03-12 条目数期望 12 → 11 | 每段历史每个删除类发布；T23 红 |

整表 44 条；每条都要被抓（`✅`），无效（`⏭`）与没红（`❌`）按 `mutation-sampling.md` 三分处置，不记成等价变异留档。M34、M35 在部分取样点上是第三类（取样点不敏感），这里已点名敏感的取样点；跑出来没红的先按那一行核取样点有没有被单测断言到。

## 十、失败条款

| 条款 | 判什么 | 什么观测会让它触发 |
|---|---|---|
| F1 | 第 2 行判乙：每个不等的格两边都查：装置那一格的偏移与取值对 layout/03、layout/01 那一行逐字核（引行号），`crates/` 写那一格的代码核（引文件与行号）；归「登记错」「实装错」或「两边都对得上而值不同」，不默认哪边对；R-flip、R-dump（第六节 6.4）此时才算，用来分「退役那几格」与「`crates/` 别处变了」 | R1.a、L03-12–L03-17 任一段历史任一行 `equal=false` |
| F2 | 第 2 行判乙（角色与段串），列出哪段历史哪几行 | 进判定的 R3.a、R3.b、R3.c、R3.d 任一不等，或 R3.e 根槽 FUA 次数 ≠ 1 或落点 ≠ 第 2 次登记的 A-根槽；8.2 第 1 行判别力自证报「两种都不是」 |
| F3 | 出自条款的锚点（7.1，含 A-tag6）不符：走「条款可能错」，同 F1 两边都查（`crates/singlefs-core/src/admission.rs` 第 347–385 行与 layout/03「记账标签 6 的值」那一行并排），记进第 2 行的结果，不作废 | 7.1 任一锚点与解出的值不同 |
| F8 | 第 2 行判乙（覆盖），列出没登记的区间 | 任一段历史 R2.a 没登记字节数 > 0，或 R2.c 恒 0 那一类里有非零字节 |
| F9 | 照第 2 次登记第 856 行（T_time 不是 5000） | R1.t 任一段历史 `t_time_not_5000` ≥ 1 |
| F10 | 第 2 行判乙（钉值层）：两边都查——harness 那条用例今天绿不绿（S6 开工时核过）、harness 那一份历史与导出那一份的差（`pool_with_a_three_unit_file` 对 `run_deletion_history` 的 H1，参数、时间、实例号）、装置解标签 6 那一行的偏移（记账 key 打头 2 字节、value 在条目偏移 22，装置第 1583–1587 行 `accounting_value`）；不默认哪边对 | P-pin 任一行 `equal=false` |
| F11 | 第 2 行判乙（钉值层只在 H1 一种历史上成立）：H2 或 H3 的标签 6 与 7.1 不同而 H1 相同，列出版本；两边都查续做 / 多一条记录这两支在 `crates/` 里算标签 6 的读法（`transaction.rs` 第 6817 行起的输入）与 layout/03 式子的 n_live、n_记录 定义 | H1 的 A-tag6 全等、H2 或 H3 有一版不等；或 8.2 第 4 行 H2 与 H1 不同值 |
| F12 | 第 2 行判乙（判别臂没分开）：去掉退役项的期望反倒与解出的值相等，说明导出里那一版标签 6 不含退役项，或装置把 H_退役、n_记录 解成了 0；两边都查导出那一版记账叶的原字节与 `crates/` 那一次发布走的标签 6 算法 | R-ret 任一版 `discriminating=false` 而 `with_equal=false` |

这几条打中都是正当结果，照实记，不改判据重跑；判据自己写错的情形照 `evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四问先过一遍再判。

## 十一、作废条款与停机条款

### 11.1 作废（那一轮的输出不算结果，原因写进交回）

| 条款 | 什么观测会让它触发 |
|---|---|
| V1 | 任一次导出或装置进程退出码 ≠ 0（接了准入的，装置退 77 是「输入没变」、不是作废：报主 agent），或内存包装退 250–254 |
| V2 | 任一份导出的行数、`grep -c '^E7RESULT '`、末行 `emitted=` 三者不全等（A-lines，两份分开判） |
| V3 | 同一份代码同一条命令跑两遍导出，sha256 不同（两份导出各自判）；装置同一个二进制跑两遍产物 sha256 不同（R-det） |
| V4 | 第 2 行射程里的 23 对阳性对照（第 5.4 节，PC-F、PC-G 之外）任一没转红，或改坏之前目标行已经是红的：这套测量分不出差别，那段历史第 2 行的全部判定作废。PC-F、PC-G 没转红只作废 R8.c 那几行（附带）。适用的对数不是 25 时，缺对照的那段历史同样作废（`applies=false` 只许出现在第 5.4 节两张表写了不适用的那几对上） |
| V5 | 7.2 任一锚点不符：A-crc 不符是装置自己的 CRC32C 错；A-content 不符是导出跑的参数不是登记假定的那一组；A-lines 同 V2；A-arith 不符是装置的式子函数算错 |
| V6 | 导出的 `impl_config` 与登记假定的参数不同，照第 2 次登记第 871 行（`devices=2`、`image_bytes=4294967296`、`physical_block_size=512`、`batch_data_units=1`；第一份恰是 `h1`、`h2`，第二份恰是 `h3`；H3 带 `intent_write_time_seconds=` 且 = `sequential_write_time_seconds=` + 60） |
| V7 | R-ret 有一版 `with_equal` 与 `without_equal` 同为 true：两份期望之差 ≥ 98 304（第四节第 7 条），同时相等只能是装置的式子或 R-ret 那一处写错 |

### 11.2 停机（两边都查，既不作废也不当结果，停在那一处交回主 agent；`implementation-first.md` 第 4 条）

| 条款 | 什么观测会让它触发 | 停下之后做什么 |
|---|---|---|
| S0 | 改装置之前：装置或变异表的 `sha256sum` 不等于文件头表里那两个值；或 `.claude/kb/layout/03-third-txn.md` 不是 `9824eff738cdd1d1a65e2998db65785fcc0a89988c1eb7adf0e9ff1e842c8e3a`、`.claude/kb/layout/01-first-txn.md` 不是 `986d36ba11106c3735d94595486d09781f1429ad98a1757abdc92ab7d9b6c4e7`（条款在登记之后动过，期望要重抄）；或主 agent 还没认第十一节 11.3 接不接准入 | 不改装置，报主 agent |
| S1 | 照第 2 次登记第 878 行：主工作区的 `deletion_write_dump.rs` 没有 `--history`，或 `scenario.rs` 的 `DeletionHistory` 没有名字是 `h3` 的成员 | 不开跑，报主 agent |
| S2 | 照第 2 次登记第 879 行，三段历史都核结构检查；另加：退役容器（码 3）「记录数 × 记录宽 ≤ 声明长度」、退役树根（码 2）「条目数 × 条目宽 = 声明长度」 | 同第 2 次：装置那一侧的偏移逐个对 layout 的行，`crates/` 那一侧找写那个字段的代码，两边的读法并排写进交回；不改判据、不改导出 |
| S3 | 照第 2 次登记第 880 行：装置独立认出的角色与导出那一段 `rewritten_roles=` 的多重集合不同（只差序号写法的记一行，不停）；L03-7 选到第二支的历史不是恰好 H3；另加：写了退役单元的历史不是恰好 H1、H2 | 同 S2 |
| S4 | 照第 2 次登记第 881 行：导出的段不首尾相接铺满窗口，或某一段删除类发布里根槽 FUA 次数 ≠ 1 | 同 S2 |
| S5 | 够判停机：第 2 行够判（第六节 6.0 末段）就停 | 没跑的量逐条标「够判后未跑」（R-flip、R-dump 在第 2 行判甲时默认就是这个结局） |
| S6 | 开工时 harness 用例 `tags_four_and_six_of_every_deletion_version_equal_the_values_recomputed_from_its_image`（`crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` 第 113 行）在主工作区不绿，或它第 198 行钉的不再是 `196_760, 196_678, 196_678, 196_678, 196_608`（文件 sha256 不是 `48009dde1780cf30b7c38d6f1b8ec246e21c352c41d744846df6dd5c746596f1` 时先看这一行） | 钉值层的目标本身不成立：不开跑，报主 agent（这是 `singlefs-harness` 的测试，不是 checker 档；执行员照共用约束带范围声明与内存包装跑这一条） |

### 11.3 准入接不接（派发提示「准入已判」那一条；定案：接，待主 agent 认）

- **接**。依据：E172 是「`crates/` 的删除写法改了就该重跑」的那一类实验——退役树加进 `crates/` 之后它没跟着重跑，留下 C775。复跑表这一行的行键已经收了驱动闭包与它点名的 `deletion_write_dump` 的 `main`（`scripts/gate/admission.py` 第 2895–2945 行），接上准入之后 `python3 scripts/gate/admission.py experiment . E172` 就能机械地答「`crates/` 的删除路径自上次产物以来变没变」，下一次不必靠人发现。照 E142（新池新建文件的干跑） 那一行接（同一类：`crates/` 侧导出 + 研究侧解码器，`.claude/gate.d/stage-inputs.tsv` 第 43 行）。
- 要改的三处：
  1. `.claude/gate.d/stage-inputs.tsv` 加一行 `E172<制表符>scripts/experiment-apparatus/e172_deletion_write_dry_run.rs::main<制表符># E172（删除写出的字节的干跑）默认调用（replay.sh 的 driver_e172）：装置的 main；driver_e172 起的 crates 导出 bin deletion_write_dump 由驱动闭包点出来`。**执行员的写范围里没有这份表**（`.claude/hooks/agent-write-scope.tsv` 第 15–28 行），由主 agent 或工具员在执行员改装置之前加；准入条件一栏不写（E172 没有前提）。
  2. 装置 `main` 开头调 `e7_index_bench::admit_experiment_run_or_exit("E172")`，把交回的行打在产物最前面（改30，执行员）。
  3. `replay.sh` 的 `ADMISSION_ROW_KEYS` 不加行：E172 在复跑表里只有一行，键默认是实验号、产物头由装置写（`scripts/gate/admission.py` 第 126–131 行）。
- 次序：第 1 处在先。装置先接了、表里还没有这一行，`admit_experiment_run_or_exit` 拒（键没登记），产物出不来；第 3 次产物出来之后再接，产物头上没有 `E7INPUT` 行，`replay.sh` 判「登记了准入、装置却没打实验键行」（`scripts/gate/admission.py` 第 239 行那一格）。所以三处与第 3 次产物在同一批里做完。
- 主 agent 不认「接」的：三处都不动，装置文件头第 13 行 `admission: always` 照旧，第五节 5.7 第 2 步跳过；判据不受影响。

### 11.4 比对对象（这一次要与之比的上一次产物）

sha256 与行数是本登记写之前现取的（第十三节命令 7）。三条都不逐字节比：装置与导出都变了，第 3 次产物与第 2 次那一份由构造就不同；它们只是 R-flip、R-dump 的旧那一边（第六节 6.4，附带）。

| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `e172-r2-deletion-write-dry-run-2026-10-05.out` | `4a03f73e37175aba3fe5169c2fe0a5c36110c16a1c66e36712db320a3f82537b` | 817 | `scripts/experiment-apparatus/replay.sh` 第 1755 行（`E172\|@driver_e172\|\|e172-r2-deletion-write-dry-run-2026-10-05.out\|exact`）；工作区已没有这份产物，取 `git show 59ce5a865^:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out`；这一次第五节 5.7 第 5 步把这一行改指重跑产物 | `e172-r3-deletion-write-dry-run-<日期>.out` | 第 2 次产物；R-flip 的旧那一边 |
| 2 | `e172-r2-deletion-write-dump-h1h2-2026-10-05.out` | `b7e87c4ed73978b2f6d89a00449bb46cb70a4d36a5e7a5a614ea64464bb090c0` | 462 | `driver_e172` 第一条命令（不带参数的 `deletion_write_dump`）；第五节 5.7 第 1 步 | `e172-r3-deletion-write-dump-h1h2-<日期>.out` | 第 2 次 H1、H2 导出；R-dump 的旧那一边 |
| 3 | `e172-r2-deletion-write-dump-h3-2026-10-05.out` | `9571c90498286a6ddfd4ac8718f3aedb8380132fa22496303342d58c0c09c45e` | 150 | `driver_e172` 第二条命令（`-- --history h3`）；第五节 5.7 第 1 步 | `e172-r3-deletion-write-dump-h3-<日期>.out` | 第 2 次 H3 导出；R-dump 的旧那一边 |

E172 没有正式跑单元表：三条在 `.claude/gate.d/replay-comparison-units.tsv` 里都只能写成「# 不登记的比对对象：research/prompts/e172-r3-prereg.md #<号>：…；欠在 C<号>（简称）」，而那一行要点名 `.claude/kb/checks-owed.md` 开着的一笔欠账（那张表文件头的规则）；今天没有一笔欠账管「E172 重跑的比对对象没有单元表」（C895 管 E185、C901 管 E161 与 E184）。登记那几行归执行员（写范围第 21 行），欠账那一行要主 agent 先立。立之前门禁 doc-experiments 的 replay-comparison-units 格对本登记判红。

## 十二、修订

（留空；装置写之后、产物之前由执行员写，只许收严或补臂。）

2026-10-29，执行员写，时点：装置改完、单测跑过（`cargo test --release --bin e172-deletion-write-dry-run` 52 条过，读两份 r3 导出的 30 条因导出还没入库报本次未跑），第 3 次产物一次没跑。原判据原样保留，下面只补不放宽；第六、七、九节各条的去留与报告方式没动。

| # | 改了什么 | 依据 | 收严在哪 |
|---|---|---|---|
| 修订 1 | L03-16 在每一次删除类发布都报一行 `field=retirement_records_written`：那片叶删空的那一次期望 1，别的期望 0（实际值 = 这一次写出的退役记录条数） | 第六节 6.0 够判要「R1.b 每格每段历史命中 ≥ 1」，而 6.1 的 L03-16 只在 H1、H2 最后一批有行：H3 不写退役记录，不补这一行 H3 的 L03-16 由构造命中 0、第 2 行判不到够判。装置在第 2 次两份导出上冒烟跑（草稿目录，不入库）读到 `cells_without_hits_h3=0`，说明补了之后三段历史每格都有命中 | 多核一格：没写退役记录的那几次发布若写了退役记录，这一行转红 |
| 修订 2 | 覆盖认领除改19 的 `(3, 34)` 退役记录外，另认领退役树根条目 `(树 65548, 条目宽 120)` 归 layout/03 格、取它偏移 34 的子指针进父指针核对；树表里 65548 那一条与 65544..65547 一样归 layout/03 格 | 改19 只点名了退役记录；退役树根「与墓碑树同形」（layout/03 第 21 行），墓碑树根条目第 2 次就这样认领。不认领时 R2.a 的没登记字节由构造不为 0（120 字节条目），退役容器也找不到父指针、S2 的父指针整单元校验和那一格核不到它 | 多核：退役容器的父指针整单元校验和进 S2，退役树根条目的每个字节有格认领 |
| 修订 3 | R5.a（附带，第六节 6.4）的 `matches_registered` 改按 `registered_roles` 现算登记差：2 → 1 独有 {t3}、最后一批独有 {tr, tr@0}（第 2 次是「最后一批独有为空」） | 改18 之后最后一批的登记角色多了 tr@0、tr（layout/03 第 60 行「多退役容器 tr@0 与退役树根 tr」），第 2 次写死的「最后一批独有为空」与今天的条款矛盾；R5.a 照报、不进第 2 行，报告方式没动。交主 agent 认 | 期望改成今天条款的写法，判法不放宽：两份差集逐元素比 |
| 修订 4 | A-容量加一行 `retirement_container_records`，期望 959 = ⌊(32768 − 136) ÷ 34⌋ | 第四节第 7 条与第七节 7.2 A-arith 已列 959，装置原来没有对应的锚点行 | 补一条钉绝对值的锚点 |
| 修订 5 | `harness_pin_same_value`、`harness_pin_out_of_window` 照改26 写；另把 8.2 第 1、2、4 行的判别力自证写成三种行：`name=r3_geometry_roles`（H1、H3 最后一批各拿两套角色期望判）、`name=a_tag6_geometry`（两套 A-tag6 常量互判）、`name=harness_pin_geometry`（H2 对 H1 逐版） | 第八节 8.2 要报这几格，登记没给行名 | 不改判据，只定行名 |

2026-11-02，执行员补一条（第二段，时点：两份 r3 导出已入库、装置的读导出单测与第 3 次产物一次没跑；依据是第 1 段报告「登记修订」一节交主 agent 的那一处字段改名，派发提示没有叫停）：

| # | 改了什么 | 依据 | 收严在哪 |
|---|---|---|---|
| 修订 6 | `name=verdict row=c775` 里登记写的 `r_ret_not_discriminating=`（改30、6.2 R-ret 一行）落成 `retirement_arm_not_discriminating=`；V7 那一条「带的与去掉的同时相等」的条数另报 `retirement_arm_both_expectations_equal=`；6.2 A-tag6 一行「`anchor_not_equal` 里 A-tag6 的条数另报」落成 `committed_reservation_anchor_not_equal=`。`name=r_ret` 行名与它的字段（`with_retirement`、`without_retirement`、`actual`、`with_equal`、`without_equal`、`discriminating`）、`name=harness_pin` 行与它的字段照登记 | 命名门禁 code-source-discipline 的 data-names 格不收 `r_ret_*`、`a_tag6_not_equal` 这类缩写式键名（第 1 段报告「登记修订」一节）；`grep -n 'name=verdict row=c775' scripts/experiment-apparatus/e172_deletion_write_dry_run.rs` 命中装置里实际的字段名 | 不改任何门槛与判据，只定字段名；三个计数仍各报各的、不合成；V7 多一个独立计数 |

2026-11-02，执行员补一批（第四段，时点：装置改完、单测与第 3 次产物一次没跑；读导出的单测第一次跑红在两条，三位调查员已定位（报告 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-e172-reds/result.md`、`…-pcg/result.md`、`…-journal-anchor/result.md`），副本读数见各行依据）。下面前三行（7.1–7.3）的定案出自主 agent 的派发提示（「主 agent 的定案：PC-G 采『预期取原归属』……A-journal 不改……其余照两位调查员给的改法改」），第四行（7.4）是「不改」的记录。原判据原样保留；第六、七、九节各条的去留与报告方式没动。

| # | 改了什么 | 依据 | 收严在哪 / 不收严在哪 |
|---|---|---|---|
| 修订 7.1 | 装置解码常量跟上 layout/01 今天的格式：`JOURNAL_HEADER_BYTES` 487 → 496（点名项区起点；新根段末尾多回收门槛 t 463、钉槽 p 471，fsid 472、MAC 480 往后挪 9 字节）；`ROOT_RECORD_FIELD_BYTES` 457 → 466（回收门槛 t 428、钉槽 p 436 在分配记录树根指针之后），单测 `layout_field_tables_tile_their_registered_totals` 的宽度表在第四个 86（分配记录树根指针）之后插 `8, 1`；两处注释的 layout/01 行号与偏移表跟着改。判据一个字没动：登记第 85 行把 layout/01 其余各节交给 layout/01、第 250 行条款层取「今天的取值与偏移」，装置的 487 / 457 是 C773（回收门槛与钉槽进新根段、根记录）之前的数 | 任务 4893（`cargo test --release --bin e172-deletion-write-dry-run`，读两份 r3 导出）：`test result: FAILED. 50 passed; 2 failed`，两条红是 `decoded_structure_passes_every_stop_check_on_the_export`（`history=h1 step 28 设备 0：journal 载荷校验和不对` 等 8 条）与 `every_positive_control_turns_its_target_red_on_every_history_it_applies_to`（`name=r_pc pc=PC-B history=h2 applies=true target_green_before=false target_red_after=true`）；调查员独立脚本逐条算 CRC32C：三段历史 34 条 journal 记录从 496 起算 34/34 与存的值相等、从 487 起 0/34（reds 报告证据 1），点名项位置条目从 496 起 752/752 指到真写过的单元（证据 2）；PC-B@h2 的 2 个非零字节是被错划进补齐的最后一个点名项的末 9 字节（证据 3），按 496 / 466 重划后三段历史都是 0（证据 4）；layout/01 第 360、362 行（496）、第 372–388 行（466）；`crates/singlefs-format/src/lib.rs` 的 `JOURNAL_HEADER_BYTES` 496、`ROOT_RECORD_BYTES` 466；任务 4916（只改 496 的副本）：`exit=101`、`51 passed; 1 failed`，两条红只剩 PC-G@h2 | 收严：载荷校验和一格从「34 条全不符」变成按真起点核；常零补齐不再吞点名项末 9 字节与 MAC 末 9 字节。不收严：`name=r2a` 的 `layout01_cell_bytes` 加 9 × 根槽写次数、`constant_zero_bytes` 减同数（`sum_matches_total` 照旧），不改任何判据 |
| 修订 7.2 | 阳性对照 PC-G 的预期：`expected_misattribution_of` 里 PC-G 一支只取「R8.c 对那一次写的原归属」，删掉「那一步是系统配置槽写时 `decoded=instance_acquisition`」那半句（第 2 次登记第五节 5.4 的 PC-G 一行写了两半，第 3 次登记 5.4 把它「原样，不改」）。R8.c 本身不改：取号第二轮的两次系统配置槽写照登记归 `other_mount_step`。第 2 次登记 5.4 的 PC-G 目标判据（施加前归错 0；施加后恰 1 行归错，且 `step`、`exported_segment`、`decoded` 都等于预期）不动，PC-G 的造法（只挪段边界、不改字节）不动 | 原预期是按「取号写一轮」定的：第 2 次的 h2 导出里挂载那一段只有步 95、96 一对系统配置槽写（`git show 18b30adbf:research/results/e172-r2-deletion-write-dump-h1h2-2026-10-05.out`，pcg 报告证据 3），段内最后一次写就是取号写，两半恰好一致。D18 已定项 11 在 2026-10-20 改成「分两轮写每块盘两槽」（`.claude/kb/decisions-history.md` 2026-10-20、`crates/singlefs-core/src/transaction.rs` 的 `ACQUISITION_WRITE_ROUNDS`），今天 h2 的挂载那一段有步 95、96（槽 0 世代 12）与步 99、100（槽 1 世代 13）四次系统配置槽写，段内最后一次写（步 100）是第二轮，R8.c 第 ① 支只认第一轮，归 `other_mount_step`：「原归属」与「系统配置槽写时取号」两半在今天的数据上互相矛盾（pcg 报告证据 1、2）。任务 4916：`pc=PC-G history=h2 applies=true target_green_before=true target_red_after=false`；任务 4940（三处解码常量加 PC-G 取原归属，另带一条探针单测）：`test result: ok. 53 passed; 0 failed`，探针打出 `PROBE r_pc` 25 行 `holds=Some(true)`、8 行 `holds=None`，`PROBE PC-G_expected` 的 `decoded: Some(OtherMountStep)`、`PROBE PC-G_misattributed` 恰一行 `step=100 … decoded=other_mount_step exported=mount_instance_row_txg8`。另照报：装置单测 `mount_writes_are_attributed_to_the_same_segments_the_export_cuts` 的「取号 2 次」只数第一轮（第二轮两次归别的挂载步），归错数 0 不受影响 | 这一行是改预期，不是收严：写进来是因为主 agent 在派发提示里定了，且判据的另一半（恰 1 行归错、三个字段全等）仍然要求；PC-G 的目标行是 R8.c（附带），它没转红只作废 R8.c 那几行（V4），不碰第 2 行的 23 对 |
| 修订 7.3 | 7.1 的 A-系统配置锚点：期望世代号 `txg + 2` → `txg + 3`（槽 = 世代号 mod 2 不变）；单测 `(5) → (8, 0)`、`(6) → (9, 4096)`，T13（H3 期望一栏）的四行改成 txg 6 / 7 / 8 / 9 → (9, 4096) / (10, 0) / (11, 4096) / (12, 0) | 取号两轮之后槽世代号 mkfs 1、取号 2 / 3、暖机 4 / 5、新池新建文件发布 6（layout/01 第 97、119、399 行），装置原式「取号 2、暖机 3 / 4」是一轮时的数。pcg 报告证据 4（独立脚本，量的）：h1 txg 5 / 6 / 7 / 8 → 世代 8 / 9 / 10 / 11，h3 txg 6 / 7 / 8 / 9 → 9 / 10 / 11 / 12。任务 4940 的探针 `PROBE unequal` 16 行（A-系统配置 8 行、A-journal 8 行）；任务 4941（再加这一处）：`53 passed; 0 failed`，`PROBE unequal` 只剩 A-journal 8 行 | 登记 7.1 的 A-系统配置不符走 F1 / F3、不作废；改后产物里这一格期望在 h1、h3 各四行应全 `equal=true`，不再带 8 行已知不等 |
| 修订 7.4 | **不改**：A-journal 锚点（第六节 6.4 表那一行，第 397 行）。产物里 h1 txg 5–8、h3 txg 6–9 共 8 行 `equal=false`、每行差 8192 字节照原样保留、照报，不进第 2 行；不拿读数反改期望式子 | 登记第四节第 3 条与 6.4 表本行已写「登记自己推锚点的前提错，不是字段；不符照 F3 两边都查、记一笔，不进第 2 行」。F3 两边都查的结果：任务 5070（`e172_journal_anchor_probe` 探针，调查员副本）：h1 窗口之前顺序写那次发布（txg 4）写 3 条 journal 记录，计数器 4 / 5 / 6（`ordinal=1 / 2 / 3`），窗口里计数器 = txg + 2；h3 顺序写（txg 5）计数器 5 / 6 / 7；任务 5071（关掉按数据单元切记录的副本，推翻条件）：h1 意图（txg 5）计数器 5、偏移 16793600，h3 意图（txg 6）计数器 6、偏移 16797696，等于装置期望。layout/01、D23（journal 的角色与格式）已定项 7 / 17 / 18、`crates/` 三方一致；错的是登记推式里「顺序写只记一个计数器」（journal-anchor 报告） | 不收严也不放宽：只记录这一处不改 |

## 十三、读过的文件与跑过的命令

### 13.1 读过的文件（行区间；grep 命中行也算）

| 文件 | 行 |
|---|---|
| `.claude/agents/experiment-designer.md` | 派发时随定义给的全文 |
| `.claude/agent-common.md` | 1–56、57–110（`cut -c1-900` 截过的行） |
| `.claude/singlefs-ai-sop/rules/test-discipline.md` | 1–122 |
| `.claude/rules/three-way-inference.md` | 1–166 |
| `.claude/singlefs-ai-sop/rules/evidence-discipline.md` | 1–179 |
| `.claude/rules/mutation-sampling.md` | 1–93 |
| `.claude/rules/implementation-first.md` | 1–19 |
| `.claude/rules/test-script-discipline.md` | 1–106 |
| `research/prompts/m3-close-b19-p5-forks.md` | 1–9 |
| `git show 04ceaa904^:research/prompts/e172-r2-prereg.md`（草稿目录 `e172-r2-prereg.old.md`） | 1–100、560–931；标题 grep；第 95–560 行的「出处」grep |
| `.claude/kb/checks-owed.md` | grep `C775` 命中第 483 行 |
| `.claude/kb/layout/03-third-txn.md` | 1–79（整份） |
| `.claude/kb/layout/01-first-txn.md` | grep `退役` 命中第 50、80、113、378、392、419 行 |
| `.claude/kb/experiments/172-删除写出的字节的干跑.md` | 标题 grep（第 1 行一级标题全文与各节标题）；2–43（`### 判决` 起的结果与结论、`## 历史版本` 没读） |
| `crates/singlefs-core/src/admission.rs` | 280–395、2060–2140；grep 命中行 41、214、305、315、316、324、326、330、341、342、347、352、355、375、380、2081、2085、2094、2096、2105、2108、2123、2126、2167 |
| `crates/singlefs-core/src/transaction.rs` | 6790–6880；grep 命中行 142、2577、2771、2829、2830、3275、3959、3960、4737、6712、8469、10987、11047、11755、11827、13878、14289 |
| `crates/singlefs-core/src/retirement_tree.rs` | 1–30（文件头）；grep 命中行 164、176、182、245、246 |
| `crates/singlefs-core/src/records.rs`、`inode_tree.rs`、`unlink.rs`、`container_tree.rs`、`mount.rs`、`allocator.rs`、`make_filesystem.rs`、`journal.rs`、`root_record.rs`、`mounted_read.rs` | grep 命中行（records.rs 680、685、696、2113–2145；inode_tree.rs 1692；unlink.rs 18、36、45、55；container_tree.rs 51、3344–3491 中命中的 7 行；mount.rs 11640、11642；其余各一两行 `TREE_IDENTIFIER_WATERMARK_AT_MKFS`） |
| `crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` | 1–250（整份） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` | 1–110；grep 命中行 |
| `crates/singlefs-harness/tests/common/mod.rs` | grep 命中行 39、96、100、152、168、189、219、322、351、765 |
| `crates/singlefs-harness/tests/unlink_single_file_publishes.rs` | grep 命中行 172、194、195 |
| `crates/singlefs-harness/src/scenario.rs` | grep 命中行（21、99、253、277–343、352–453、614、634、645、695–699 一带） |
| `crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs` | 1–60；`println!` 与 `name=` grep 命中行 6、11、110、114、183、202、226、236、246、256、266、278 |
| `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs` | 1–40、80–130、584–600、895–920、975–995、1125–1175、1515–1660、1948–1990、3069–3135、1318–1330；grep 命中行（`98_304` 等、`fn ` 定位、`PC-`、`EXPORT`、`admission`）；逐个 `awk` 核过第三节、第五节 5.8 引的行 |
| `scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs` | 7216–7228；grep `admission` |
| `scripts/experiment-apparatus/lib.rs` | 135–141 |
| `scripts/experiment-apparatus/replay.sh` | grep `e172\|E172` 命中行 1755、3167、3170、3172–3175、3178；3160–3185；grep `E142` 命中行；2007–2025；3285–3310 |
| `scripts/experiment-apparatus/Cargo.toml` | grep `e172` 命中行 704、706、707 |
| `scripts/gate/admission.py` | 1–70、126–200、2895–2945；grep 命中行（实验、`--bin`、没登记） |
| `.claude/gate.d/stage-inputs.tsv` | 1–30（每行前 500 字）；grep `E142\|E172\|e172` 命中第 43–45 行 |
| `.claude/gate.d/replay-comparison-units.tsv` | 1–59（每行前 330–400 字） |
| `.claude/gate.d/doc-experiments.sh` | grep `replay-comparison-units` 命中行 27、34、36、37、39、40、42、69、86、87 |
| `.claude/hooks/agent-write-scope.tsv` | grep 命中第 15–28 行 |
| `.claude/hooks/lib_long_run.py` | grep `CANONICAL_HOSTS_SENTENCE` 命中行 11、19、138 |
| `.claude/heavy-thread-default` | 1–6 |
| `research/prompts/e139-r3-prereg.md`、`e165-r5-prereg.md`、`e165-r7-prereg.md` | 比对对象表表头那一行起的 4 行（看表的写法） |
| `scripts/kb/quote-kb.py`、`scripts/kb/replace-once.py` | 1–45；`--help` 前 15 行 |
| `research/results/` 目录 | `ls \| grep -i e172` 两个文件名；两份导出只取了 sha256 与行数，内容没读 |

### 13.2 跑过的命令（原样；草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-e172-design`，下面写成 `$D`）

1. `bash scripts/hosts/work-directory-ownership.sh claim $D --purpose '批 19：E172 第 3 次重跑登记（C775）'` → 认领了已有的目录。
2. `git show 04ceaa904^:research/prompts/e172-r2-prereg.md > $D/e172-r2-prereg.old.md; wc -l …; grep -n '^#' …` → 931 行；`sha256sum` → `f1a81a1f…89eb`。
3. `git log --oneline -3 -- crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs crates/singlefs-harness/src/scenario.rs; sha256sum <导出 bin> <scenario.rs> <装置>; git status --short …` → 文件头表里的三个 sha256；装置与导出 bin `git status` 干净；`sha256sum scripts/mutation/tables/e172_deletion_write_dry_run.tsv` → `fc01f6d3…25ed`，32 行。
4. 行号定位与核对：`grep -n` 与 `awk 'NR==n'`（第三节、第五节 5.8、第十节引的每一个行号都这样现取过）。
5. 第二节：`nice -n 19 python3 scripts/kb/quote-kb.py $D/quote-section2.md` 加十三个取法（checks-owed `~^\| C775 \|`；layout/03 `~核的是退役树加进之前那一版`、`~^\| 退役容器 \|`、`~^\| 树表四条新条目 \|`、`~^\| 树表退役树条目 \|`、`~^\| 记账标签 6 的值 \|`、`@## 四、段序列`、`@## 五、退役记录`；layout/01 `~^\| t12 \|`、`~^\| 树 ID 水位 \|`、`~^\| 树表条目 \|`、`~^\| key 区间（偏移 52，两个 key 宽） \|`；实验页 `@### 复跑`）→ `✓ 13 段整抄进 …，回读逐字节一致`，121 行；整份 `cat >>` 追加进登记，`sed -n "90,\$p" 登记 | cmp - $D/quote-section2.md && echo QUOTE_MATCH` → `QUOTE_MATCH`。
6. `nice -n 19 python3 $D/anchors_r3.py`，输出原样：
   - `h1h2 before_intent n_live=1 n_records=1 with_retirement=196760 without_retirement=98386 difference=98374`
   - `h1h2 intent n_live=0 n_records=1 with_retirement=196678 without_retirement=98304 difference=98374`
   - `h1h2 three_to_two n_live=0 n_records=1 with_retirement=196678 without_retirement=98304 difference=98374`
   - `h1h2 two_to_one n_live=0 n_records=1 with_retirement=196678 without_retirement=98304 difference=98374`
   - `h1h2 final n_live=0 n_records=0 with_retirement=196608 without_retirement=98304 difference=98304`
   - `h3 before_intent n_live=2 n_records=2 with_retirement=196912 without_retirement=98468 difference=98444`
   - `h3 intent n_live=1 n_records=2 with_retirement=196830 without_retirement=98386 difference=98444`
   - `h3 three_to_two n_live=1 n_records=2 with_retirement=196830 without_retirement=98386 difference=98444`
   - `h3 two_to_one n_live=1 n_records=2 with_retirement=196830 without_retirement=98386 difference=98444`
   - `h3 final n_live=1 n_records=1 with_retirement=196760 without_retirement=98386 difference=98374`
   - `ceil(32768/959) = 35  ceil(32768/815) = 41  floor((32768-136)/34) = 959`
   - `h_retirement_one n_live0 n_rec1 = 229446  h_tombstone_one n_live1 n_rec1 = 229528`
   - `tree_table_declared_length 12 x 200 = 2400`
7. `sha256sum` 与 `wc -l` 两份 r2 导出；`git show 59ce5a865^:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out | sha256sum`、`| wc -l` → `4a03f73e…537b`、817；`git log --oneline --diff-filter=D --name-only -- 'research/results/*e172*'`；`awk 'NR==1755'`、`awk 'NR==3173'` `replay.sh`。
8. `cat .claude/heavy-thread-default; nproc` → 登记值 31、本机 32 核；`bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts`（输出含第二台的主机名与路径，不抄进登记）；`python3 scripts/agents/dispatch-prompt.py experiment-runner | grep 多机：…`（看规范句的写法）。
9. `nice -n 19 python3 scripts/gate/admission.py keys .` → `E142`、`E142/layer0`、`E142/p8g-crates-dump`、`E43`、`E49`、`E116`、`E157`、`E16R5`、`E16R5/pc3`、`E16R5/second-segment`，没有 E172。
10. `python3 -c "print(2*(98304+41))"` → `196690`；`python3 -c "print(2*((2+0+1+2+0+1)*16384+35)-2, 2*((2+0+1+2+1+1)*16384))"` → `196676 229376`（M32、M34 的取样点）。
11. `sha256sum .claude/kb/layout/03-third-txn.md .claude/kb/layout/01-first-txn.md .claude/kb/checks-owed.md crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` → `9824eff7…8e3a`、`986d36ba…c4e7`、`02227f58…5234`、`48009dde…96f1`（S0、S6）。
12. 登记的写法：`set -o noclobber` 后 `cat >` 排他新建，之后 `cat >>` 分段追加（每段不超过 150 行）；行号与措辞用 `python3 scripts/kb/replace-once.py research/prompts/e172-r3-prereg.md <旧串> <新串>` 定点改（每次命中 1 次）。

## 十四、并行与挂钟

独立单位：两份导出（H1H2、H3）互相独立；装置的单测与变异表的 44 条变异各自独立（`mutate.sh` 按条派）；装置一趟里三段历史与 25 对阳性对照在一个进程里顺序跑，秒级，不拆。
线程：照登记值 31（`.claude/heavy-thread-default`），执行员起编译与单测用 `bash scripts/hosts/capped.sh <派发提示的线程上限>`；主 agent 跑导出与变异表照它的默认。
串行：导出 → 装置产物 → 复跑行改指 → 变异表，四步有先后（后一步读前一步的产物）；两份导出各跑两遍比 sha256 的那两遍可以并行；产物跑两遍比 sha256 必须是同一个二进制，前后串行即可，不计时（这一次没有计时类的量）。
预计挂钟：推的，依据是第 2 次实验页「复跑」那一段与第 2 次登记第 651 行——导出两份连编译约 5–15 分钟（`singlefs-checker-tier` 编一次）；装置改动与单测由执行员做，编译加单测几分钟；产物秒级；变异表 44 条每条一次编译加单测，约 30–60 分钟；合计 2 小时内。
预计在用核数：推的——导出与装置编译时 cargo 用满线程上限 31；产物与单测跑时 1–4 核；变异表按 `mutate.sh` 的并行度，至多 31。
