# 删除线第 59 批：代码轮第一轮正文（2026-10-10）

<!-- doc-lint:not-numbers M1 M2 M3 L1 L2 L3 L4 L5 L6 K1 K2 K3 -->

## 一、题面

这一轮攻第 59 批三份补丁，照判决 `research/prompts/m3-batch58-code-r1-main-verification.md` 第三节 L1–L5 改（L6 归根环线），已合进主工作区（底是第 58 批合入之后）。三份都只改测试，`crates/*/src/` 一行没动：

| 份 | 关的 | 落点 |
|---|---|---|
| 甲 | L1 GPU 三棵树各一条旋钮 2 下 [2,1,2,1]、转一圈根环的对拍，变异是 GPU 扫描内核只判倒数第二片 | checker 档 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 末尾 |
| 乙 | L2 harness 三棵树各一张 [2,1,2,2,1]（少的那片离最右三片、转一圈根环），变异是只判最右之前两片；L5 J3 三条与新三条 debug 下各单跑，最长 8.35 秒，去掉 `#[ignore]` | harness `checker_known_bad_images.rs` |
| 丙 | L3 J1 ② 那张盘加可写挂载一条，第 3861 行换成「重建读完记账树先数映射条目、再判分配记录」；L4 「这一版自己那行水位坏了只剩台阶」那条换造法：删这一版自己那行、补上一版原样的 (1, 0, 0, 3) 让条目数对上 | harness `corrupt_on_disk_content_is_refused_instead_of_panicking.rs`、`core_review_tree_table_duplicates_and_slot_one_search.rs` |

条款：`.claude/kb/invariants.md` I-1.10（判定顺序）、I-3.13、I-9.22；`.claude/kb/decisions/18-块里携带什么信息.md` 已定项 24 第 3 条；`.claude/kb/decisions/05-快照-空间记账机制.md` 已定项 2（射程「inode 号水位的台阶行」）与已定项 4（统计量登记表）。I-9.36、I-9.37 定义在 `~/.local/share/singlefs/work/m3-batch57/container-judges/report.md`。

改动全文：`research/prompts/_m3-batch59-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-batch59/gpu-ring-turn/report.md`、`~/.local/share/singlefs/work/m3-batch59/far-left-short/report.md`、`~/.local/share/singlefs/work/m3-batch59/rebuild-order-watermark/report.md`。上一轮判决与腿报告：`research/prompts/m3-batch58-code-r1-main-verification.md`、`m3-batch58-code-r1-opus-output.md`、`m3-batch58-code-r1-sonnet-output.md`；攻方模型 `research/prompts/m3-batch58-code-r1-opus-model/`（可以拷来改臂）。

## 二、实现今天的样子（主 agent 的观测，2026-10-10）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch59-code-r1/repo/`（主工作区合入第 59 批之后的 `crates/`、`Cargo.toml`、`Cargo.lock`）。被判文件与 `crates/mutations.tsv` 的 sha256 在 `research/prompts/m3-batch59-code-r1-snapshot/crates-sha256.txt`。代码行号按冻结副本。
- 主 agent 在主工作区跑过：`merge-and-verify.sh` 打三份补丁，「✓ 测试全绿（194 个测试二进制，共 2015 条测试）」、fmt、clippy、build 过。在干净构建上单跑甲的三条：谱系「states=489 … cpu_checker_red=393 gpu_checker_red=393 … mismatches=0」、墓碑「states=561 … 429 / 429 … mismatches=0」，都「1 passed」；退役「states=621 … gpu_red=24 … recovery_mismatches=24」，这 24 个状态 GPU 第 1 字 3（条目太多），判不了。根环线打上 GPU 分配记录表与已释放表抬到 2048 的补丁之后，退役那条「mismatches=0」「1 passed」。墓碑、谱系两行 GPU 变异证红「点名 2 条：跑了 2 条，抓到 2 条」。checker 档两趟与变异、崩溃两条轻型的结果写进判决。
- 实现员自定、主 agent 接受、被攻过零轮的：甲用「建 30 次空 inode」转根环；乙在 [2,1,2,2,1] 上也转根环（没转时中途几版里少的那片离最右不超过两片）；乙判六条都不标慢档（忙时几十上百秒是 nice 19 被抢 CPU）；丙 L4 借 I-3.13（重建今天不逐条判记账行代段等于这一版的 txg）补那一行，判别力用已有第 3668 行。
- 已知、记作欠账或下一批线索、报出来不算打中的：退役那条对拍要等根环线 GPU 上限补丁合入才绿、它那一行变异那时再证；J4（产品容量墓碑 [815, 1] 改号）归根环线，抬了两张表之后又撞 MAX_RECORDS 512 那一族；「只判最右之前 k 片」对足够大的 k 有限组排布关不死；拷别处编译目录当种子会让 cargo 判 Fresh（`records/2026-09-16-subagent拆分提案.md` 第六十九条）；形式化轻型 pinned-red-examples-complete 格因归档提交删掉一份红例账而红，不归这一批；第 58 批判决第四节与更早判决列的欠账照旧。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### M1　GPU 旋钮流转根环（甲 L1）
三条新对拍是不是只因旋钮支判红（CPU 那一棵自己那条、GPU 逐位相同），有没有在正确判法下因别的理由红？转过根环后候选集里还剩哪几版，「只判倒数第二片」在哪一版上与正确判法分得开？「建 30 次空 inode」会不会改变那棵容器树、让被判那一版不再是 [2,1,2,1]？

### M2　旋钮支离最右更远与慢档（乙 L2、L5）
[2,1,2,2,1] 三张盘是不是只由点名的那一种写错而红？「只判最右之前两片」以外，同一族里 k = 3、4 的改坏在已有盘上哪一张分得开？去掉 `#[ignore]` 后这几条在快档里跑会不会因别的用例的状态而变？

### M3　重建次序补钉与水位用例换造法（丙 L3、L4）
J1 ② 可写挂载那条是不是只由「分配记录判在条目宽与逐条判之前」分得出？第 3861 行换成的那一形是不是只红 J1 ① 可写挂载那条？L4 新造法下，用例是不是还钉在「重建要这一版自己那行水位、不认台阶顶替」上：补进来的 (1, 0, 0, 3) 会不会被别的判先拒、或让「水位行缺」在错的理由上报出？读者将来逐条判 I-3.13 时它红在哪？

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | M1–M3 | 核「测试测的是不是条款与判决点名的那件事」，逐条对条款原文与冻结副本的行 |
| 云端攻方（Opus） | M1–M3 | 在冻结副本的拷贝上造反例：用例在错的理由上绿或红、改坏照绿、新造法漏掉原造法罩到的东西 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷编译目录当种子之前先 `find crates -type f -exec touch {} +`。读码推出的「没打中」尽量换成造出来的反例尝试。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 10G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑；线程上限 15；用自己的编译目录，几份变异副本各用各的。前台命令预计超过 4 分钟的拆成小段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试。
