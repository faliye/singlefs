# 删除批第 29 批合入的代码：代码轮第一轮正文（2026-10-06）

<!-- doc-lint:not-numbers K1 K2 K3 L1 L2 L3 H-J3a H-J3b O-1 F-W -->

## 一、题面

删除批第 29 批一份补丁 2026-10-06 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-06 那几条）：l 关 m3-batch27-code-r1 判决第三节第 1–3 条。这一轮判两件事：代码做的是不是条款与判决说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch29-code-r1-diff.md`。冻结副本 `~/.local/share/singlefs/work/m3-batch29-code-r1/tree/`，被判的 6 份文件的 sha256 在 `research/prompts/m3-batch29-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索）：`~/.local/share/singlefs/work/m3-batch29/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或判决 |
|---|---|---|---|
| L1 | H-J3a / H-J3b：墓碑、谱系、inode 三棵树的「类型段不认得」与「孩子是另一类」两支，不往下走的孩子每条位置条目都判 I-2.1（登记过的类按那一类长度比，没登记或头读不出的 16 KiB、32 KiB 都比），两份都对不上记走读失败；共用函数 `judge_every_copy_of_a_child_the_walk_does_not_go_into`；GPU 同名 w_ 函数 | `crates/singlefs-checker/src/walk.rs` 第 2272、2299、2312、2575、2606 行；`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` | `.claude/kb/invariants.md` I-2.1、I-4.8、I-7.2；m3-batch27-code-r1 判决第二节 H-J3a、H-J3b 两行、第三节第 1 条 |
| L2 | O-1：扫描内核 `r_instance_pages_after_first`、`r_instance_rows` 的链循环上限改命名常量 `INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED`，走满记判不了（站点 37、38）；快测试钉两内核上限相同；对拍补「被抛弃根 C 自己那张实例表 257 片」 | `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 第 8–18、493、1112 行 | I-9.29、I-3.12、I-7.9；m3-batch27-code-r1 判决第二节 K1 J1 那一行（O-1）、第三节第 2 条 |
| L3 | 「第 1 片读不出」那条关门对拍流补一行变异 | `crates/mutations.tsv` | m3-batch27-code-r1 判决第二节 K1 J2 那一行、第三节第 3 条 |
| 测试 | harness 新坏镜像 5 条；第 27 批那条期望保留并注明只对校验和补齐的那张成立；变异表追加 13 行 | `crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/mutations.tsv` | 同上 |

**这一轮不判的**（各有去处，腿不攻）：
- m3-batch28-code-r1 判决第三节（H-V 子树 F-W、GPU 实例表链 I-1.2、测试尖度、C739 后一半）：第 30 批。
- `crates/mutations.tsv` 第 2935 行锚点被 L1 改断（说明里「不读）」改成「不往下走）」）：第 30 批修。
- GPU 对拍与 GPU 那几行变异：提交时由主 agent 跑。

## 二、实现今天的样子（主 agent 的观测，2026-10-06）

- 合入验证 `merge-and-verify.sh`「✓ 测试全绿（170 个测试二进制）」「✓ fmt 过」「✓ clippy 过」，补丁打上时 code-source-discipline 的 mutation-tables 格红 1 行（上面那一行锚点）（`/tmp/claude-1000/merge-and-verify/2026-10-06-1690267/`）；`cargo clippy --offline --all-targets --all-features -- -D warnings` 经内存包装单跑退 0；`crash_enumeration_sharded_across_processes` 整份「10 passed」。
- 实现员报告说扫描内核新加的「走满记判不了」两句没有一条流走得到（走读内核先记站点 61）；主 agent 读码不认：走读内核唯一调用点（`crash_judge_checker.wgsl` 里 `w_walk_one` 只在 walk_and_hand_off 的循环里调一处）只走最新根、候选根与记录施加的版本，候选集外的根（被崩溃恢复抛弃的那条）只有扫描内核 I-3.12 走到。这一句也是腿要判的。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（L1–L3）
- L1：三棵树两支各调到共用函数没有；比长度的规则（登记过的按那一类、没登记或头读不出的两种都比）、「两份都对不上记走读失败」与 I-2.1 字面是不是一致；GPU 同判。
- L2：扫描内核两处走满记判不了，能不能由一条流单独走到（被崩溃恢复抛弃、滤出候选集的根）；「删掉那两句」该不该有变异、谁能抓到；新站点号与已有不撞。
- L3 与测试：5 条坏镜像与对拍流的期望写全没有；13 行变异点名的测试与参数筛不筛得到。

### K2　哪一格会错（重点 L1、L2）
在冻结副本的拷贝上攻，只跑 harness 档（GPU 不跑，GPU 那一侧读 wgsl、用 CPU 算期望）：
- L1：合法状态上会不会多红 I-2.1（孩子没落盘的一份、没登记的类按两种长度都比时撞上合法单元）；一份头读不出、一份对得上时判什么；三棵树、两支、层级 1 / 2 各一遍。
- L2：链长 255 / 256 / 257、根在候选集里 / 外、链中途断开时，扫描内核判什么，与 CPU 比。
- CPU 与 GPU 判得不一样的一形。
- 上一轮「没打中（一次抽样）」的第二次抽样：m3-batch28-code-r1 的 H-V 合法状态误红、H-B 合法状态误红、CPU 与 GPU 第 28 批改的几支。

### K3　被判的文件名单
`research/prompts/m3-batch29-code-r1-snapshot/crates-sha256.txt` 里那 6 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决格推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 L1、L2 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮。
- GPU 那一侧：读代码找到一支与 CPU 不同、并用 CPU 算出期望写清 GPU 会给什么，算打中；GPU 不跑。
- 误红、漏判、漏红的口径同 m3-batch22-code-r1 正文第五节。

交付：
- 腿的报告写进 `research/prompts/m3-batch29-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写，别等全部攻完才写。
- 模型与脚本放 `research/prompts/m3-batch29-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取，包装脚本另给一个变量时在文件头写明变量名；编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G），不用 `auto`；同一时刻只起一条 cargo；`compare` 比之前排序、自己不依赖 `SINGLEFS_REPO`；用自己的编译目录。草稿放 `~/.local/share/singlefs/work/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
