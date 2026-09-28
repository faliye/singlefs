# E162 第四段（S4 掉电装置）执行员报告（2026-09-27）

## 结论
- 装置 `research/e7-index-bench/src/bin/e162_verdict_store_power_cut.rs`（4 971 行，sha256 57ebd8a7…）按补 12 第四段写完：日志解析、四种取法的重放、标记、分桶与挑块、写盘/验盘两个来宾角色、核对子进程、`--role s4-stage`、`--role s4-drive`（含 `--dry-run`）、`--role selftest`。没起过 QEMU。
- `Cargo.toml` 末尾经 `insert-row.py` 追加 `[[bin]] e162-verdict-store-power-cut`（`required-features = ["e162-block-stores"]`）。
- 单测 23 个（`grep -c '^    #\[test\]' <源文件>` → `23`），最终源码上五遍全绿：`test result: ok. 23 passed; 0 failed` ×5。
- 变异 M16–M26 共 11 条，最终源码上（第三遍）：抓到 11 / 无效 0 / 没红 0；💥0 ⚠️0 ⏱0 🧱0；收尾「计数：内存撞顶 0 条（上限 16G）、超时 0 条」「已还原，基线仍全绿」，exit=0。日志入库 `research/results/e162-verdict-store-power-cut-mutate-2026-09-27-r2.log`（第一遍 `…-mutate-2026-09-27.log` 也是 11/11，之后改了两处源码与 strip，所以重跑）。
- `--role s4-drive` 不带 `--dry-run` 且 `SINGLEFS_HEAVY_TESTS` 不是 user-request/commit 时退 64（实测：`exit=64`，stderr「s4-drive 起 QEMU 虚机，是重型测试…」）。

推翻条件：第五段真跑时 PC4-R（V16）或 PC4-C（V17）对不上、或来宾里 sfdisk/mkfs/loop 起不来（H8）、验盘附加根装不进 initramfs（H10），说明装置在真虚机上的某一环没被单测罩住——单测全在宿主上，来宾路径（分区、mkfs、O_DIRECT 标记、loop 挂载、核对子进程）一次都没在虚机里跑过。

## 产物
| 文件 | 内容 |
|---|---|
| `research/results/e162-verdict-store-power-cut-2026-09-27-selftest-r2.out` | selftest：47 行锚点全 `matches_registration=true`，`name=done emitted=49`；replay 登记指这一份 |
| `research/results/e162-verdict-store-power-cut-2026-09-27-selftest.out` | 同一份输出（strip 改动之前跑的），与 -r2 逐字节一致（sha256 均为 bef93669…） |
| `research/results/e162-verdict-store-power-cut-2026-09-27-dry-run.out` | `--role s4-drive --dry-run --arms R1,R0,K,F`：锚点、宿主资源、附加根（18 个文件、23 040 896 字节，搭好的加载器跑 selftest 通过，mke2fs 1.47.0、sfdisk util-linux 2.39.3），18 次写盘开机与各自验盘开机的 vm-bench 变量与参数，`name=done emitted=88` |
| 两份变异日志 | 见上 |

复跑：`bash research/scripts/replay.sh E162` → `E162  @driver_e162_power_cut_selftest 字节一致 e162-verdict-store-power-cut-2026-09-27-selftest-r2.out`，末行 `✓ 复跑判过的 2 行都对得上（字节一致 2、仅计时不同 0），结论断言全中`，exit=0。登记行与驱动函数 `driver_e162_power_cut_selftest`（`grep -c` 起写前为 0）已加进 `research/scripts/replay.sh`。
干跑产物带宿主上的临时路径与 pid，不登记逐字节复跑。

判决行（4c）：`grep -n 'name=verdict'` 两份产物各 1 行 `name=verdict part=anchors anchor_mismatches=0`（selftest-r2 第 48 行、dry-run 第 49 行）。判决行 2 条，没有 false / not_run 字段，违例、不匹配、歧义、失败类的整数计数都是 0。
锚点里 `PC4-C_synthetic`、`M26_control_forms`、`B9d` 三个 id 不是补 13 脚本的输出，是装置按登记正文推出的自检（合成 PC4-C 日志的四个计数、对照形态恰好差一处、标记分区容量），名字里的 matches_registration 对它们指「与登记正文相符」。

## 第五段的整条命令（主 agent 填 `--arms` 与 `--kernel`，在仓根起）
先编（不起虚机）：`cd research && BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo build --release -p e7-index-bench --features e162-block-stores --bin e162-verdict-store-power-cut`
再跑：
`SINGLEFS_HEAVY_TESTS=user-request bash research/scripts/run-with-memory-cap.sh 20G research/target/release/e162-verdict-store-power-cut --role s4-drive --arms <R1,R0,K,F 的子集> --kernel <内核路径> --out research/results/e162-verdict-store-power-cut-<跑的那天 YYYY-MM-DD>-s4.out`
- 产物文件名：`research/results/e162-verdict-store-power-cut-<YYYY-MM-DD>-s4.out`（`--out` 排他新建，已存在就退 2）。
- 二进制在 `research/target/release/`，不是补 12 写的 `research/e7-index-bench/target/release/`（后者不存在；修订八）。
- 开跑前装置自己判：可用内存 ≥ 16 GiB、`${TMPDIR:-/tmp}` 可用 ≥ 40 GiB、`/proc` 里没有 qemu-system / fio / e152 装置 / vm-bench.sh，不满足退 3；锚点对不上退 1。补 5.3 其余前提（KVM、内核、busybox、cpio）照旧由主 agent 现查。
- 开机次数（`--arms R1,R0,K,F`）：写盘 18 次（PC4-C 1、PC4-L 4、PC4-NB 4、PC4-FD 1、主格 4、S4-bs10 4），K 主格按 V25 至多再重做 2 次；验盘至多 4×10 + 9 + 4 = 53 批。跑多久没估。
- 风险（推的，没量过）：验盘附加根要把写盘开机的设备侧日志整段带进 initramfs，块值是随机数压不动，主格日志估计数百 MB 到 GB 级；QEMU 直接引导时 initrd 大小有上限，装不下会表现为 vm-bench「读不到退出标记」→ 装置重做一次（V30）后停在 H10。

## 登记修订（写进 `research/prompts/e162-preregistration.md` 第十二节，标题「修订（第四段 S4 装置……）」，在 S4 任何产物之前）
十条 S4-修订一至十，只补写法与两处动作，判据、门槛、臂、对照、作废与停机一条没动；第六、七、九节与补 6、7、9 的去留没动。要点：
- 一：PC4-C 来宾末尾、主臂 umount 之后各补一次 flush，让日志系统配置的声明条目数罩住全部写（k 不变）。
- 二：中位数取下中位；C-commit 64 等距点取 ⌊s(n−1)/63⌋。
- 三：S4-bs10 没有 C-commit；PC4-S/P/O 在「条目总数」那个断电点（U-keep）上做。
- 四：F-nofsync 连建目录的 fsync 一起拿掉；F-nodirsync 只拿掉改名后的父目录 fsync。
- 五：带数据的 FLUSH、MARK/METADATA、未知标志 → H9；DISCARD 重放不动盘面；FUA 写不进 U(k)。
- 六：核对子进程崩掉记 check=crashed、单列「未判」，不并进 Q4f；loop / tmpfs 镜像失败 → H8。
- 七：V25 翻倍重做由 s4-drive 自己做，产物里 `name=v25_redo` 行，第六段据此补修订行。
- 八：第五段二进制路径。九：拷自 S1 源文件的行号段（S1 源 sha256 38f1d92e…）。十：附加根里的二进制 `strip --strip-debug`（release 带 debug，456 766 512 字节；搭好之后 23 040 896 字节的整个附加根）。
- 另有一处没进修订、主 agent 要知道的：`name=verdict part=s4_*` 等判定行是装置按补 6/补 10/补 11 现算的，第六段执行员照登记复核，不当最终判定。

## 门禁（登记给 experiment-runner 的阶段，各贴末行与退出码）
| 阶段 | 退出码 | 末行（截断到 230 字） | 是不是这一段的文件 |
|---|---|---|---|
| 27 | 0 | `✓ 格式常量同步（51 个已登记，51 个在源码里被钉住）` | — |
| 33 | 0 | `✓ 153 个实验二进制都有成形的变异表，1746 条变异的原文各命中源码一次；…` | 含本表 11 条 |
| 52 | 1 | 红在 E142 段序列登记表（`✗ 段序列登记表与 E142 产物的 name=segments 行…对不上，共 2 处`） | 不是，照写不修 |
| 80 | 0 | 绿 | 本文件在射程里 |
| 85 | 0 | `写明「原始输出未留存」不判的 0 页：（没有）` | — |
| 96 | 0 | 绿 | 本文件在射程里 |
| 99 | 0 | `✓ 登记了路径与共用项的实验页判过了（5 份、21 条路径）…` | — |
| 34 | 0 | `✓ 实验索引行与正文标题一致（索引 162 行、正文 162 份）` | — |
| 40 | 1 | 点名本段五个 results 文件没被 experiments.md 点名（另有别人的 E163 等） | 是：这一次不写实验页，交主 agent 派人点名 |
| 69 | 1 | 红在 E142 装置与三份 research/prompts 里引 /tmp 的句子 | 不是 |
| 75 | 1 | `[没回看] .claude/kb/experiments/162-崩溃放量判定块存储选型.md：这次改动里 E162 产物 …power-cut…dry-run.out 变了，影响的决策表一行都…` | 是：E162 实验页的「影响的决策」表要回看，这一次不写页，交主 agent |
| 84 | 0 | `✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 15 个）` | — |
| 86 | 0 | 绿 | — |
| 88 | 77 | `! 这次改动新增或改写的 kb 正文里没有整行抄的 E7RESULT 行，本阶段无对象可判` | 本次未跑 |
naming-lint 新文件 0 处；clippy（bin 目标）0 条。clippy 带 `--tests` 时被别人的 e108 的 deny 挡住编不完，测试代码的 clippy 没单独判到。

## 岔路表（`research/prompts/m2-crash-store-r1-forks.md`）
| 行 | 已够判 / 还差什么 | 登记里剩下的量能不能让它翻面 | 出处命令 |
|---|---|---|---|
| S4 | 还差全部：这一段只写装置，产物里没有任何断电点的计数（补 12：第四段「本身不补任何一行」） | 能：第五段各臂 U-drop 的 Q4a–Q4f 任一 > 0 就翻（F11）；取样点上翻照补 14 第 6 条只记限定词 | `grep -c 'name=s4_point' research/results/e162-verdict-store-power-cut-2026-09-27-dry-run.out` → 0（干跑没起虚机） |
| S3 | 不归这一段（另一个执行员在做回环与跨机） | — | — |

## 没做什么
- 没起 QEMU、没跑 vm-bench.sh（含 `--selftest`）、没跑 s4-drive 真跑：重型测试，交主 agent 按上面的命令问用户后跑。
- 来宾角色（分区、mkfs、O_DIRECT 标记、tmpfs 镜像、loop 挂载、核对子进程、RocksDB LOG 计数）一次都没在虚机里执行过，单测只覆盖宿主上能跑的：解析、重放、标记编解码、分桶挑块、写入循环次序、四个候选在宿主目录上的 PC4-S/P/O 与干净库。
- 没写实验页、索引行、experiments-history；五个新 results 文件还没被实验页点名（门禁 40 号），E162 实验页「影响的决策」表要回看（75 号），交主 agent 派人。
- 门禁 86 号出路里的「删掉 research 下那些文件」没照做。没判结论、没跑门禁全量、没提交。
- 没碰 S3 的两个源文件与变异表、S1 的源文件与变异表；`Cargo.toml` 只经 insert-row.py 插了自己那一段。
- 草稿目录 /tmp/claude-1000/e162-s4-apparatus/ 里只有日志、进度与本报告；没建编译目录与仓副本（变异用的是 mutate.sh 自己的 /tmp/singlefs-mutate-target*，跑完它自己收了副本）。
