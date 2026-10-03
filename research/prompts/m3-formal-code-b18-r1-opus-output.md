# m3-formal-code-b18-r1 云端攻方报告（2026-10-04）

攻击面：K2（P2 接线用例判不判得出接错）、K1（V242u 在第三截判别力用例上的抽样命中率、判官在第三截合法态上会不会误红、没崩过的那一次挂载被拒时漏判哪一类态）。
代码引文全部指冻结副本 `/tmp/claude-1000/m3-formal-code-b18-r1/tree/` 里的路径与行号（与快照清单 `research/prompts/m3-formal-code-b18-r1-snapshot/crates-sha256.txt` 逐个核过：`crash_injection.rs`、`transaction.rs`、`mount_transparency.rs` 三份的 sha256 与清单相同）。
所有「量过」的数都是**副本上的数**：冻结树拷进草稿目录，装三处运行时开关（V242u、接线变体、追踪），另建一个原型包 `b18-attack-proto` 经公开入口 `inject_crashes_into_history` 照两条 checker 档内联用例的输入跑。没跑任何 checker 档测试二进制；原型链接 checker 档的库，按定义 3b「在自己的原型里调枚举函数跑小流」办，全部跑下来第一次崩溃态 8863 个、二次崩溃态 26547 个（命令数出，见「这条腿自己的限度」）。

## 复跑

工作目录：仓根。内存：每趟经 `research/scripts/run-with-memory-cap.sh`，上限 64M（原型单趟峰值约 24M）；线程：`B18_THREADS`（默认 6）。

```
bash research/prompts/m3-formal-code-b18-r1-opus-model/tools/rerun.sh quick   # 约 3 分钟：t1 × 5 个开关、t3 种子 0 × 7 个开关 × {基线, 加 V242u}、F3 原型两趟
bash research/prompts/m3-formal-code-b18-r1-opus-model/tools/rerun.sh sweep   # 约 9 分钟：t3 种子 k×4096（k=1–42）三批
bash research/prompts/m3-formal-code-b18-r1-opus-model/tools/rerun.sh rand    # 约 3 分钟：随机历史 60 段基线
```

第二、三个参数是冻结树根与工作目录（默认 `/tmp/claude-1000/m3-formal-code-b18-r1/tree` 与 `${TMPDIR:-/tmp}/b18-opus-rerun`）；结果读工作目录 `out/` 下的 `t1 …`、`t3 …`、`rand …` 行与 `per-trial.txt`。`quick` 我自己用这份脚本从头跑过一遍（拷树、打补丁、编、跑），21 件 rc 全 0，输出逐行与草稿里先跑的一致，原样放在 `outputs/quick/`。`sweep` 一趟超过子 agent 前台 240 秒的上限，没经脚本整跑；`outputs/sweep/` 里是我在草稿目录分批跑的同一组命令的输出（批次与估时见「这条腿自己的限度」）。

模型目录 `research/prompts/m3-formal-code-b18-r1-opus-model/` 的工具与原型（全部 117 个文件的 sha256 见文末附录）：

```
99b12c1d4c4eca4545673a09290ac24cac27be5146ef9c88014409e6cdd1222a  ./proto/Cargo.toml
5f4781c54bac2aec9d839cc3e9cdb3479f29b28b40040d9c5fe0807b6452b262  ./proto/main.rs
89679393c8b4ade6b95607d1b415d2fe11f0b6ecb680bbd3db31f698ba9ae070  ./tools/patch_copy_2.py
fe441cee18690c9116bfbf626c1193f2d04414e92ac55c8c2398078cdde43768  ./tools/patch_copy_3.py
f1518d16dd92d9a313f3856ef9d57f436706d5e5dbfa4adf5efb09d67dfc5001  ./tools/patch_copy.py
cfbf4b49bbe852e22c92af927cda536512501bf3bff285810597186de1b873c9  ./tools/per_trial.py
1e768bf89e1c01a3658938f8107d6c2fc12d9e79985b993f9486b323b8788fef  ./tools/rerun.sh
1c85082220659b80fd684b64d1e74b41adce82828918f08ab836effe41a6414e  ./tools/summarize_trace.py
```

开关（副本里读环境变量，不设时与冻结树逐行同义）：`B18_V242U=1` 照 `crates/mutations.tsv` 第 2283 行把带单元的发布改成先轮换、后根槽 FUA；`B18_VARIANT=<名>` 打开下面 K2 表里的一种接错（`s2_call_injected`、`s2_old_judge` 照第 2284、2285 行；`s3_decorrelate` 是我的改法 F3 的原型）；`B18_TRACE=1` 每个二次崩溃打一行。

## 各格判定一览

| 格 | 判定 | 量过还是推的 |
|---|---|---|
| K2-a 第三截调用点三种接错（读故障恒交 `Injected`、镜像取第一次崩溃的、镜像取不含崩溃那一段的基线）三条内联用例全绿 | **打中**：三条用例对它们都不红，而且每一种都让 V242u 那一行由红转绿（6 行变异里没有一行落在这个调用点上） | t1、t3 两条量过（副本）；用例 2 绕过调用点是读代码推的 |
| K2-b 第二截所选那一版取错（交 `None`）用例 1 照绿 | **打中**：拒得不合法改由 U6「判据复算为假」那一条给出，用例 1 只钉不合法计数 1、不钉是哪一条 | 量过（副本） |
| K2-c 读故障认定的下标取错（写表下标当录制流下标）三条全绿 | **打中**，但今天端到端看不出差别：用例与随机历史里一个注入过读故障的崩溃态都摆不出来 | 量过（副本：随机 60 段里 15 段带「崩溃恢复抛弃根」，注入过读故障的态 0 个） |
| K2-d 第三截所选那一版取第一次崩溃的那一版 | 三条全绿、连 V242u 也分不出；只会让判官多报，不漏报 | 量过（副本） |
| K1-a V242u 在第三截判别力用例上红不红 | 种子 0 **红**（第三截不合法 2 次），但只靠 1 个独立抽样；换种子 35/42 红 | 量过（副本） |
| K1-b 实现员「每个第一次崩溃态抽中约 3 ÷ 写行写数 × 7/12」 | 单次抽样的概率对（量到 n=13 时 0.130、n=19 时 0.100）；**「每个第一次崩溃态各抽一次」不对**：二次崩溃的种子只随（历史种子、第一次崩溃段号）变，41 个第一次崩溃态只有 14 次独立抽样 | 量过（副本） |
| K1-c 判官在第三截合法态上误红 | 没打中：基线上第三截再挂一次都没被拒，判官根本没被调到 | 量过（副本，固定历史 43 个种子、随机历史 60 段） |
| K1-d 没崩过的那一次挂载被拒时第三截不判，漏哪一类态 | 漏的是「挂载写过东西之后才报错」那一类；这类第一次崩溃态在第二截已经按理由不在单子上报红，漏掉的只是同一个已红态上多出的第三截红；样本里 0 个 | 分类是读代码推的；0 个是量过（副本） |

## K2　P2 接线用例判不判得出接错

### 三条用例各自经过哪一段

- 用例 1（`a_crash_state_where_only_the_rotation_…`）只经第二截：它断言的是「(0, 1),」（`crates/singlefs-checker-tier/src/crash_injection.rs:5236`），即第二截合法 0、不合法 1；不看不合法是哪一条条件给的。
- 用例 2（`a_remount_refused_on_a_second_crash_image_…`）自己拼 `RefusalOnTheCrashImage`，直接调「judge_the_remount_after_a_second_crash(」（`crates/singlefs-checker-tier/src/crash_injection.rs:5279`），读故障自己交「read_faults: ReadFaultsInTheCrashState::NoneInjected,」（`crates/singlefs-checker-tier/src/crash_injection.rs:5287`）。它**不经过**第三截的调用点 `judge_second_crashes_inside_the_writable_mount`：那里拼输入的四行（镜像「crash_image_materialized: &second_crash_image_materialized,」（`crates/singlefs-checker-tier/src/crash_injection.rs:2732`）、所选那一版「version_landed_on_after_the_crash: report.effective_root,」（`crates/singlefs-checker-tier/src/crash_injection.rs:2733`）、读故障「read_faults: read_faults_in_the_first_crash_state,」（`crates/singlefs-checker-tier/src/crash_injection.rs:2739`））写错什么它都看不见。
- 用例 3（`no_remount_after_a_second_crash_without_read_faults_is_refused_…`）经过调用点，但只断言基线上第三截不合法为 0（「tally.remount_refusals_with_the_witness_criterion_true_without_read_faults,」（`crates/singlefs-checker-tier/src/crash_injection.rs:5374`））。接错只会让判官少报或多报；基线上第三截再挂一次都没被拒（下面 K1-c），少报的接错在它这里永远是 0，照绿。

所以第三截调用点上任何「让判官少报」的接错，三条用例结构上都抓不到；唯一会红的是 V242u 那一行变异（它要求用例 3 在坏实现上红）。6 行变异里落在第三截的三行（`crates/mutations.tsv` 第 2286–2288 行）都改在被调函数里面，用例 2 直接调被调函数，所以抓得到；调用点那四行一行都没有变异。

### 接错变体逐个量（副本，`outputs/quick/`）

t1 = 用例 1 的输入（同一段历史、同一个靶向崩溃态），打出第二截合法 / 不合法计数与不合法那一次的条件；t3 = 用例 3 的输入（种子 0、短段整段枚举），打出第三截判过 / 不合法计数；「加 V242u」是同一变体再叠 V242u 那一行。

| 变体（开关名） | 改了哪一行 | 用例 1（t1） | 用例 2 | 用例 3（t3 基线） | 加 V242u 之后第三截不合法 | 在 6 行变异里吗 |
|---|---|---|---|---|---|---|
| 无（对照） | — | 合法 0、不合法 1，条件含 C670 那一条、不含「复算为假」 | 绿 | 0 | 2（红） | — |
| `s2_call_injected` | 第二截调用点读故障恒交 `Injected`（同第 2284 行） | 合法 1、不合法 0（**红**） | 不经过 | 0 | 不跑 | 在 |
| `s2_old_judge` | 第二截换回旧判官（同第 2285 行） | 合法 1、不合法 0（**红**） | 不经过 | 0 | 不跑 | 在 |
| `s2_version_none` | 第二截所选那一版交 `None`（第 1322 行） | 合法 0、不合法 1（**绿**）；条件变成「复算为假」、没有 C670 那一条 | 不经过 | 0 | 2 | 不在 |
| `s2_index_write` | 第二截读故障认定拿写表下标当录制流下标（第 1281 行） | 合法 0、不合法 1（绿） | 不经过 | 0 | 2 | 不在 |
| `s3_call_injected` | 第三截调用点读故障恒交 `Injected`（第 2739 行） | 绿 | 不经过（推的） | 0（绿） | **0**（判过 2、不合法 0） | 不在 |
| `s3_call_first_image` | 第三截调用点镜像交第一次崩溃的（第 2732 行；这个字段同时是再挂的盘） | 绿 | 不经过（推的） | 0（绿） | **0**（判过 0） | 不在 |
| `s3_call_base_without_segment` | 第三截调用点镜像交没叠崩溃那一段的基线 `second_base` | 绿 | 不经过（推的） | 0（绿） | **0**（判过 0） | 不在 |
| `s3_call_first_version` | 第三截调用点所选那一版交第一次崩溃的（第 2733 行） | 绿 | 不经过（推的） | 0（绿） | 2（与对照相同） | 不在 |

对照组会红：`s2_call_injected`、`s2_old_judge` 两行在 t1 上由「合法 0、不合法 1」变成「合法 1、不合法 0」，V242u 在 t3 上由 0 变 2——t1、t3 这两件装置抓得到已知的坏接线与坏实现，表里「绿」的格不是装置瞎了。原样输出（每行前面是 `outputs/quick/` 下的文件名，后面是那份文件里的整行）：

```
t1-none.out: t1 crash_points=1 injected=0 model_contents_compared=1 legal=0 illegal=1 new_findings=1
t1-none.out: t1 conditions c670=true criterion_false=false read_back_outside=false
t1-s2_version_none.out: t1 crash_points=1 injected=0 model_contents_compared=1 legal=0 illegal=1 new_findings=1
t1-s2_version_none.out: t1 conditions c670=false criterion_false=true read_back_outside=false
t1-s2_call_injected.out: t1 crash_points=1 injected=0 model_contents_compared=1 legal=1 illegal=0 new_findings=0
t3v242u-none.out: t3 seed=0 crash_points=41 injected=0 overwrite_points=12 second_crash_points=123 row_publish_second=41 remounts=123 c670_judged=2 c670_illegal=2 stage3_new_findings=1 stage2_illegal=0 test3_red=true
t3v242u-s3_call_injected.out: t3 seed=0 crash_points=41 injected=0 overwrite_points=12 second_crash_points=123 row_publish_second=41 remounts=123 c670_judged=2 c670_illegal=0 stage3_new_findings=0 stage2_illegal=0 test3_red=false
t3v242u-s3_call_first_image.out: t3 seed=0 crash_points=41 injected=0 overwrite_points=12 second_crash_points=123 row_publish_second=41 remounts=123 c670_judged=0 c670_illegal=0 stage3_new_findings=0 stage2_illegal=0 test3_red=false
t3v242u-s3_call_base_without_segment.out: t3 seed=0 crash_points=41 injected=0 overwrite_points=12 second_crash_points=123 row_publish_second=41 remounts=123 c670_judged=0 c670_illegal=0 stage3_new_findings=0 stage2_illegal=0 test3_red=false
t3v242u-s3_call_first_version.out: t3 seed=0 crash_points=41 injected=0 overwrite_points=12 second_crash_points=123 row_publish_second=41 remounts=123 c670_judged=2 c670_illegal=2 stage3_new_findings=1 stage2_illegal=0 test3_red=true
```



### 各变体为什么那样

- `s2_version_none` 照绿的原因：判官里 U6 那一半用所选那一版复算判据，没有所选那一版就判假（`crates/singlefs-harness/src/mount_transparency.rs:1603`：「.push(RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage);」），C670 那一半见到 `None` 直接不成立（「let selected_version = selected_version?;」（`crates/singlefs-harness/src/mount_transparency.rs:1379`））。两样一减一加，不合法计数还是 1。用例 1 若要分辨，得看条件表（副本上 t1 已把它打出来，见上面 `conditions` 那两行）。
- `s2_index_write` 照绿：用例与随机历史里没有一步真在读故障下往盘上发过东西（K2-c），下标取在哪都判成没注入。
- `s3_call_first_version` 连 V242u 也分不出：V242u 的红态是「写行那次轮换落了、根没落」，新实例的记录不跨实例边界施加，二次崩溃恢复落到的那一版就是第一次崩溃落到的那一版，两个输入恰好相等。它只在两版不同、又有一次 N-配置 拒的态上起作用；所选那一版取旧了，c_见证 比它末条计数器大的机会只多不少，所以这一接错只让判官多报。

### K2-c　读故障认定这一支端到端没有会红的检查

读故障认定只由「崩溃恢复抛弃根」一类步造（「HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => true,」（`crates/singlefs-checker-tier/src/crash_injection.rs:1033`）），而那一步在 C554 乙之后通常在取号之前就拒、一个写都不发。副本上 BROAD 比重随机 60 段历史（8 步、每段 16 个崩溃态）里 15 段带这一步（共 16 次），注入过读故障的崩溃态一个都没有：

```
$ grep -h '^rand seed' outputs/rand/rA-*.out | grep CrashRecoveryAbandoningTheNewestRoot | grep -o 'injected=[0-9]*' | sort | uniq -c
     15 injected=0
```

所以「读故障认定取错」这一类接错里，往少报方向的（第二截恒交 `Injected`）由用例 1 抓；往多报方向的（任一截恒交 `NoneInjected`、窗口末尾下标取错）今天在任何崩溃注入跑法上都不改一个计数，`crates/mutations.tsv` 第 2221、2222 行两条只钉住 `read_faults_in_a_crash_state` 自己（单元用例给写死的录制流下标），调用点把哪个下标交进去没人钉。这一格是「判别子观测不到」那一形：被判的系统里没有可达的注入态，判据分不出两种写法，不是用例写漏了一条。

## K1　P1 第三截接判官对不对

### K1-a / K1-b　V242u 的抽样：种子 0 红，但只靠一次抽样

二次崩溃怎么抽（`draw_second_crash_points`）：种子只取历史种子、盐与第一次崩溃的段号，「^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64"),」（`crates/singlefs-checker-tier/src/crash_injection.rs:2544`）；每一段（取号、写行、暖机、续做）在那一段的写里均匀挑一个 w（「source.below(u64::try_from(writes_of_the_phase.len()).expect("写数装得进 u64")),」（`crates/singlefs-checker-tier/src/crash_injection.rs:2563`）），w 所在那一段里 w 之前的写都落、w 不落、w 之后的各抛一次硬币（「std::cmp::Ordering::Greater => source.below(2) == 1,」（`crates/singlefs-checker-tier/src/crash_injection.rs:2572`））。种子里没有第一次崩溃那个子集，所以**同一段里的各个第一次崩溃态，只要挂载那一段的写表切得一样，就抽到同一个二次崩溃**。

V242u 下写行那次发布的末尾变成一段「轮换盘 0、轮换盘 1、根槽 FUA」三个写（`SSR`）。红态是根没落、至少一槽轮换落了：w 取盘 0 轮换时要盘 1 落、根不落（1/4），取盘 1 轮换时要根不落（1/2），取根时必红（1）。一次抽样命中的概率是 (1/4 + 1/2 + 1) ÷ n = 7/(4n)，n 是写行那一段的写数——与实现员的「3 ÷ n × 7/12」是同一个数。

副本上种子 0 的追踪（`outputs/quick/seed0-row-publish-groups.txt`）：41 个第一次崩溃态在写行那一段只有 18 组切法，**每一组里不同的二次崩溃只有 1 个**；按（第一次崩溃段号, n）去重是 14 次独立抽样（n=13 九次、n=19 五次，n=5 那一组是写行不带单元、V242u 不改它）。命中的只有第一次崩溃第 12 段那一抽，两个第一次崩溃态共用它，所以计 2：

```
first_seg=11 mount_segments=2+8+2+3+2+1+2 n=13 first_states=3 distinct_second_points=1 [(2, 'JJ', '00')] refused=0 c670_illegal=0
first_seg=12 mount_segments=2+8+2+3+2+1+2 n=13 first_states=2 distinct_second_points=1 [(3, 'SSR', '010')] refused=2 c670_illegal=2
first_seg=14 mount_segments=2+14+2+3+12+2+3 n=19 first_states=7 distinct_second_points=1 [(1, 'UUUUUUUUUUUUUU', '10110101010101')] refused=0 c670_illegal=0
```

换种子量这一抽的分布（种子取 k×4096，k=1–42；种子是连续小整数时历史种子与段号的异或会撞，几十个种子只用到约 64 个不同的随机流，k=0–41 那一批 42/42 全红就是撞出来的，见 `outputs/sweep/per-trial-vA.txt`，不作数）：

```
$ cat outputs/sweep/per-trial-vB.txt
seeds=42 seeds_hit=35
n=5 trials=42 hits=0 rate=0.000 7/(4n)=0.350
n=13 trials=378 hits=49 rate=0.130 7/(4n)=0.135
n=19 trials=210 hits=21 rate=0.100 7/(4n)=0.092
```

按均匀抽样算，这段历史 14 次抽样全不中的概率是 (1 − 7/52)^9 × (1 − 7/76)^5 = 0.168；实测 42 个种子里 7 个不中（`outputs/sweep/vB-*.out` 里 `test3_red=false` 的那 7 行：种子 16384、61440、102400、118784、135168、143360、147456）。

结论（副本上的数，推翻条件：入库装置上同一命令下种子 0 第三截不合法不是 2，或同一组里出现两个不同的二次崩溃）：

1. V242u 那一行在今天这份树上对用例 3 是红的（种子 0、第一次崩溃第 12 段那一抽），判别力不是「抽不中」。
2. 但它只挂在一次抽样上。历史、几何或挂载写表的切法一变（写行多一个单元、起点多一段，段号与 n 就跟着变），等于换了一个种子，约六分之一的机会这一行变异悄悄变成「没杀死」——不是代码坏了，是抽样换了一个落点。
3. 实现员「每个第一次崩溃态抽中约 7/(4n)」那句对一次抽样成立，把它乘到每个第一次崩溃态上不成立：独立抽样数是第一次崩溃的段数（再乘切法数），不是第一次崩溃态数。

### K1-c　判官在第三截的合法态上会不会误红：没打中

第三截的判官只在「再挂被拒」那一支里调。基线上它一次都没被调到：

- 固定历史（用例 3 那一段）种子 k×4096、k=1–42：4410 个二次崩溃，再挂被拒 0 次（`outputs/sweep/bB-remount-refused-count.txt`）；种子 0 另跑一次，第三截判过 0。
- BROAD 随机 60 段（8 步，含覆盖写、关掉重挂、冷启动、抬 F、挂着回退、删除、建 inode、抛弃根）：每段 `s3_judged=0 s3_illegal=0`，`not_judged={}`（60 段 60 行，`outputs/rand/rA-*.out`）。

按代码推：判官报红要同时有「再挂以 N-配置 拒」「第一次崩溃那个态没注入读故障」「撕裂的槽跳过之后判据复算为真」。没注入读故障的态上判据为真本身就是 C670 说的不许出现的态（「没有任何读故障的崩溃态，不许让 D23（journal 的角色与格式） 已定项 14 可写挂载那条见证判据为真」（`.claude/kb/checks-owed.md:529`））；第二截那次挂载起在第一次崩溃镜像上、不注入读故障，二次崩溃镜像的读故障认定沿用第一次崩溃的，方向是对的（注入过的整段不判，只会漏、不会误红）。我找过几种合法态会不会让 c_见证 越过所选那一版末条计数器——取号写（写的是取号那一刻读到的 c_见证，不抬高最大值）、抬 F 先写系统配置那一步（tail 取现行那一版末次轮换的，不抬高）、新实例写行的记录不跨实例边界施加（只在轮换先于根落盘时才露出来，正确顺序下根槽 FUA 单独一段、先于轮换那一段）——都不行。推翻条件：入库装置上任何一次基线跑出第三截 `remount_refusals_with_the_witness_criterion_true_without_read_faults` 大于 0。

能让判官**多报**的接错只找到一种（K2 表 `s3_call_first_version`），它在 V242u 红态上两版相等、量不出差别；落到合法态上它要先有一次 N-配置 拒，而基线上一次都没有。

### K1-d　没崩过的那一次挂载被拒时不判，漏的是哪一类态

`once` 是没判的理由时直接返回（「Err(reason) => {」（`crates/singlefs-checker-tier/src/crash_injection.rs:2803`）），再挂与 C670 那一判都在它后面。挂载在任何写之前被拒时挂载写表是空的，`draw_second_crash_points` 一个二次崩溃都不摆，没有可漏的；漏的只能是**挂载写过东西之后才报错**的第一次崩溃态。按 `publishes_before_the_continuation_of_a_mount_that_failed` 与挂载的错误表，写过东西之后才返回的只有 `MountError::Publish`（取号之后的发布或续做里的一步没发成、不是空间不够）与 `MountError::FloorRaiseFailedAfterTheMountsPublishes`（写行、暖机之后推的抬 F 自己报了不是空间不够的错）。这两种的理由按 `refusal_reason_of_mount_error` 取它们里面那个错的理由：续做那一路照挂载的注释只在不是空间不够时才报 `Publish`，写行与暖机的空间不够在取号之前的准入里就拒了（`RowPublishAdmissionRefusedBeforeAcquisition`、`WarmUpAdmissionRefusedBeforeAcquisition`），所以它们落不到单元区墙那一格（推的，没造盘面），第二截已经按「崩溃后镜像上的可写挂载拒了」报红（「tally.writable_mounts_after_the_crash_refused += 1;」（`crates/singlefs-checker-tier/src/crash_injection.rs:2167`））——漏掉的是同一个已经判红的第一次崩溃态上多出来的第三截红，不是一类没人判的态。副本样本里这一类一个都没出现（随机 60 段 865 个第一次崩溃态，`not_judged` 全空；固定历史那几批的 t3 行没打这一项，不算证据）。

再挂本身不用 `once`：把 `once` 那一判挪到再挂被拒那一支后面，C670 照判、只跳过 I-8.4，多一次挂载只落在这类（样本里 0 个）态上（推的，没实现）。

第三截不判 U6（理由不在单子上、判据复算为假、盘变了、读出不在允许集合里的拒都不报）是正文第二节已记的欠（「U6 不延到第三截」），不重报。

## 打中之后的四句

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到判别它的东西 | 满足的是哪一句的哪一个分句 | 跑前条款的改法在打中的格上还中不中 |
|---|---|---|---|---|
| K2-a 第三截调用点三种接错三条全绿 | 这一轮没有臂，被判的是「三条用例对接错红不红」；三种接错各自让 V242u 由红转绿，分得出「用例抓得到」与「抓不到」 | 看得到：t3 加 V242u 那一趟里判官被调到 2 次，三种接错下第三截不合法都变 0 | 正文 K2「三条内联用例对……各自红不红；有没有一种接错（下标取错、所选那一版取错）照样全绿」的后一分句（读故障认定、镜像两类，不止括号里那两样） | 跑前条款只给「改法并进下一批实现员」；下面 F2 修这一格（推的，没实现），F1、F3 碰不到它 |
| K2-b 第二截所选那一版交 `None` 用例 1 照绿 | 同上；t1 的条件表分得出（C670 那一条在 / 不在） | 看得到：判官交回的条件表里有，只是用例 1 没断言它 | 同一句的「所选那一版取错」分句 | F1 修；F2、F3 碰不到 |
| K2-c 读故障认定的下标取错全绿 | 分不出：今天没有可达的注入态 | **看不到**：被判的崩溃注入跑法里注入过读故障的态 0 个，两种下标给出同样的计数 | 同一句的「下标取错」分句；按证据纪律这是「判别子观测不到」那一形 | F1–F4 都碰不到；要么造一条真在读故障下发过写的历史，要么把调用点算窗口末尾录制流下标那一步抽成函数单独钉（F5，推的） |
| K1-a/b V242u 只靠一次抽样 | 不涉及臂；涉及「V242u 那一行是不是稳定的判别力」 | 看得到：追踪里每组二次崩溃只有 1 个 | 正文 K1「V242u 在 checker 档用例上推得出红吗（抽样会不会抽不中）」：种子 0 抽得中，换一个落点约 1/6 抽不中 | F3 修（量过）；F1、F2 碰不到 |

## 我提的改法（只在我的模型上量过、被攻过零轮）

- **F1** 用例 1 加一条断言：第二截那一条发现的没满足条件里有 `WitnessCriterionTrueOnACrashImageWithoutReadFaults`、没有 `CriterionFalseWhenRecomputedOnTheCrashImage`（或给第二截另记一个「C670 那一条不合法」的计数，与第三截那一对计数同形）。
- **F2** 第三截加一条**经调用点**的正向用例：给二次崩溃加一种靶向摆法（同 `CrashPointDraw::ExactlyOneCrashState`，持久集合可以越过段尾），在用例 1 那段历史的一个普通第一次崩溃态上摆「写行那次轮换落了、根槽 FUA 没落」，经 `inject_crashes_into_history` 跑，断言第三截判过 1、不合法 1。正确的持久顺序下这个二次崩溃只能越过根槽那一段的段尾摆出来，与用例 1 越过段尾的造法同一个理由。
- **F3** 二次崩溃的种子再混进第一次崩溃那个子集（副本开关 `s3_decorrelate`，`tools/patch_copy_3.py`），同一段的各个第一次崩溃态各抽各的；二次崩溃态的个数不变。
- **F4** `once` 那一判挪到再挂被拒那一支之后：没崩过的那一次挂载被拒时照样再挂、判 C670，只跳过 I-8.4。
- **F5** 第二截算「崩溃窗口末尾那个写在录制流里的下标」那一步抽成一个函数，单元用例钉住它（写表下标与录制流下标不同的流上，交进 `read_faults_in_a_crash_state` 的是录制流下标）。

| 改法 | K2-a 第三截调用点三种接错 | K2-b 第二截所选那一版 | K2-c 读故障认定下标 | K1-a/b V242u 只靠一次抽样 | K1-d 挂载写过东西后报错的态 |
|---|---|---|---|---|---|
| F1 | 碰不到（推的：用例 1 不经第三截） | 修（量过：t1 在 `s2_version_none` 下条件表 `c670=false criterion_false=true`，正确接线下 `c670=true criterion_false=false`，断言条件表就分得开） | 碰不到（量过：`s2_index_write` 下条件表与正确接线相同） | 碰不到（推的） | 碰不到（推的） |
| F2 | 修（推的：没实现；它经过调用点，三种接错下第三截不合法都会变 0，同 t3 加 V242u 那三行） | 碰不到（推的） | 碰不到（推的：历史里没有注入态） | 部分：给第三截一条不靠抽样的判别力，V242u 那一行可以改挂它（推的） | 碰不到（推的） |
| F3 | 碰不到（推的） | 碰不到（推的） | 碰不到（推的） | 修（量过：种子 k×4096 那 42 个里 41 个红，原来 35 个；种子 0 第三截不合法 3，原来 2；种子 0 基线照旧 0。按均匀抽样算漏的概率由 0.168 降到 0.0084，推的） | 碰不到（推的） |
| F4 | 碰不到（推的） | 碰不到（推的） | 碰不到（推的） | 碰不到（推的） | 修（推的：样本里这类态 0 个，量不出） |
| F5 | 碰不到（推的） | 碰不到（推的） | 修调用点那一半（推的） | 碰不到（推的） | 碰不到（推的） |

F3 的原样输出（`outputs/quick/` 与 `outputs/sweep/vC-*.out`）：

```
t3v242u-s3_decorrelate.out: t3 seed=0 crash_points=41 injected=0 overwrite_points=12 second_crash_points=123 row_publish_second=41 remounts=123 c670_judged=3 c670_illegal=3 stage3_new_findings=1 stage2_illegal=0 test3_red=true
t3-s3_decorrelate.out: t3 seed=0 crash_points=35 injected=0 overwrite_points=9 second_crash_points=105 row_publish_second=35 remounts=105 c670_judged=0 c670_illegal=0 stage3_new_findings=0 stage2_illegal=0 test3_red=false
vC 六件的 summary 行：seeds_with_a_hit=7、6、7、7、7、7（漏的那一个是种子 57344）
```

F3 的代价（推的）：它改的是每个崩溃态摆哪一个二次崩溃，已有的复现行与钉住的红例里凡是第三截的，会摆到别的二次崩溃上；换之前要按 `.claude/gate.d/e165-pinned-red-examples.tsv` 与战役的复现行逐条看有没有落在第三截的。只改判别力那条用例（短段整段枚举写行那几段）不动战役，是另一条路，没量。

## 顺带看到（不在攻击面里，交主 agent）

副本上 BROAD 随机 60 段基线里 6 段在第二截报新发现，6 个都落在「删除之后的建 inode」那一步（`outputs/rand/rA-*.out` 的 `rand finding` 行），种子 32768 那一个的整条（`outputs/rand/rand-32768-full.out`）里实现那一半是 `崩溃后可写挂载之后的那一次发布拒了：WriteToAnInodeThatIsNotInTheTree { inode: 1 }`。看着像第二截挂载之后那一次发布按「现行那一版带文件」去覆盖写 inode 1，而那一版里 inode 1 已删、新建的是别的 inode（推的，没查）。与 18-1、18-2 无关，同时合入的删除批相关；复跑：`rerun.sh rand` 之后看 `out/rA-*.out` 的 `rand finding` 行。

## 没打中的形状

- 第三截合法态上误红：固定历史 43 个种子（4410 + 105 个二次崩溃）、BROAD 随机 60 段（2611 个二次崩溃）基线上再挂被拒 0 次，判官没被调到；按代码找的三种「合法而 c_见证 越过所选那一版」的态都不成立（K1-c）。
- 第三截读故障认定沿用第一次崩溃那个态：方向对（只漏不误红），没找到反例。
- 没崩过的那一次挂载被拒、又写过东西的态：样本里 0 个（K1-d）。
- 第二截「读故障恒交 `NoneInjected`」、第三截同样恒交 `NoneInjected`：多报方向，今天没有注入态，量不出；只读代码。
- 我的历史里由用户决定的那几步：固定历史就是用例 3 那一段，没放开；放开的是随机 60 段（BROAD 比重、8 步），判官误红那一格两边都 0。

这条腿只抽了一次样：上面「没打中」的几格按三方规则要再抽一次（`-opus-s2`）才算没打中。

## 这条腿自己的限度

- 全是副本上的数。原型照用例 1、用例 3 的输入经公开入口跑（同一段历史、同一个执行、同一种摆法、同一个种子），比的是它们断言的那几个计数；用例 2 调的是私有函数，没跑，表里它那一列全是读代码推的。V242u 做成运行时开关，开关打开时的写序与 `crates/mutations.tsv` 第 2283 行改成的那一段逐步相同（先轮换、再根槽 FUA、再屏障，零单元发布照旧）；接线变体同理逐个对着冻结树的那一行写（`tools/patch_copy*.py` 里每一处替换都断言旧串恰好命中一次）。
- 量：第一次崩溃态 8863 个、二次崩溃态 26547 个（把草稿与 `rerun-check` 里全部 `t1`、`t3`、`rand` 行的计数加起来数的）；最大的一次是一段随机历史（每趟不过百来个态），没有一次调层 0 的枚举函数；用例 3 那段历史整条录制流的闭式状态数是 2113583（`outputs/quick/count.out`），原型只摆用例 3 那种短段整段、长段各抽 2 个的态，没碰全量。
- 批次（每批 6 件并行，每件单线程）：vA（V242u，种子 0–41，每件 7 个，估 170 秒，实测 182 秒）、vB（V242u，种子 k×4096，同上）、bB（基线，同上；第一次 6 件全 rc=252，同会话已有别的 agent 的 cargo 在跑，照包装的出路设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 重跑）、vC（V242u + F3，同上）、rA（随机 60 段，每件 10 段，估 170 秒，实测 162 秒）、变体批（7 件，前两次分别撞上限超过峰值表两倍 rc=251、管道吞了退出码，整批作废重跑；第三次改 `auto` 全 0）。包装的 251、252 那几次输出都没当结果用，记在草稿 `progress.md`。
- 种子 k=0–41 那一批 42/42 全红是种子异或撞出来的，我换成 k×4096 才算概率；同样的坑会让任何「连续小种子扫一遍都红」的说法高估判别力。
- 我只从 C670 的前提出发找「合法而判据为真」的态，没有换一个独立的判据核（例如拿 O3 或 RefFS 判同一批二次崩溃）。

## 没做什么

- 没跑任何 checker 档测试二进制、变异表与门禁；6 行变异里第 2286–2288 行（用例 2）红不红是读代码推的，第 2283–2285 行是在副本原型上照用例输入量的。
- F2、F4、F5 没实现；F3 只在副本原型上量过。
- K3（卸载那一格）不归这条腿。没读正推腿的报告与模型（禁读清单）。
- 草稿目录 `/tmp/claude-1000/m3-formal-code-b18-r1-opus/` 里建的仓副本与编译目录（`copy/`、`target/`、`rerun-check/copy/`、`rerun-check/target/`）交回之前已删；留下的是日志与原始输出（`runs/`、`rerun-check/out/`、`progress.md`、`scripts/`，约 8 MB），入库的那一份在模型目录 `outputs/`。`runs/` 里的 `*.trace`（每个二次崩溃一行，`per_trial.py` 的输入）没入库，`rerun.sh sweep` 会重新生成。

## 附录：模型目录全部文件的 sha256

```
fba311df550d377adb5cf8c50fbd20b95fbc08a63d4110548134f42cec954d9a  ./outputs/quick/count.out
336601133303f9d6d898e78b8b2bb4942d42fd02ca54a71f706780d1c658de7c  ./outputs/quick/seed0-row-publish-groups.txt
7ddf9b35d8bd1d70ead48c62b1ea19034de0f2ad0a357dcc1e13d3e9bbc816d0  ./outputs/quick/t1-none.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t1-none.rc
5223a849dfaaf629a2b67ac2b1b71144d0f1dfc0ffa0a03a5f8f45c28a5a1a49  ./outputs/quick/t1-s2_call_injected.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t1-s2_call_injected.rc
7ddf9b35d8bd1d70ead48c62b1ea19034de0f2ad0a357dcc1e13d3e9bbc816d0  ./outputs/quick/t1-s2_index_write.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t1-s2_index_write.rc
5223a849dfaaf629a2b67ac2b1b71144d0f1dfc0ffa0a03a5f8f45c28a5a1a49  ./outputs/quick/t1-s2_old_judge.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t1-s2_old_judge.rc
686a76d54403b4a94bc951c0ca95867637118246027a9c046c7338b794e0ad16  ./outputs/quick/t1-s2_version_none.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t1-s2_version_none.rc
e6bd873f6571556b51b0d5af4af6efaee34bf3610fdd301be0b3566cd61acdf9  ./outputs/quick/t3-none.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3-none.rc
e6bd873f6571556b51b0d5af4af6efaee34bf3610fdd301be0b3566cd61acdf9  ./outputs/quick/t3-s2_index_write.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3-s2_index_write.rc
e6bd873f6571556b51b0d5af4af6efaee34bf3610fdd301be0b3566cd61acdf9  ./outputs/quick/t3-s2_version_none.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3-s2_version_none.rc
e6bd873f6571556b51b0d5af4af6efaee34bf3610fdd301be0b3566cd61acdf9  ./outputs/quick/t3-s3_call_base_without_segment.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3-s3_call_base_without_segment.rc
e6bd873f6571556b51b0d5af4af6efaee34bf3610fdd301be0b3566cd61acdf9  ./outputs/quick/t3-s3_call_first_image.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3-s3_call_first_image.rc
e6bd873f6571556b51b0d5af4af6efaee34bf3610fdd301be0b3566cd61acdf9  ./outputs/quick/t3-s3_call_first_version.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3-s3_call_first_version.rc
e6bd873f6571556b51b0d5af4af6efaee34bf3610fdd301be0b3566cd61acdf9  ./outputs/quick/t3-s3_call_injected.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3-s3_call_injected.rc
e6bd873f6571556b51b0d5af4af6efaee34bf3610fdd301be0b3566cd61acdf9  ./outputs/quick/t3-s3_decorrelate.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3-s3_decorrelate.rc
f30f858660058a3a0633d1a065c3d37215419fad065a18206755182056404c74  ./outputs/quick/t3v242u-none.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3v242u-none.rc
09d0514ffbecf9accd83eb0e0d6bc6c36220c63919e74815d97c2b8d215896b3  ./outputs/quick/t3v242u-none.trace
f30f858660058a3a0633d1a065c3d37215419fad065a18206755182056404c74  ./outputs/quick/t3v242u-s2_index_write.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3v242u-s2_index_write.rc
f30f858660058a3a0633d1a065c3d37215419fad065a18206755182056404c74  ./outputs/quick/t3v242u-s2_version_none.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3v242u-s2_version_none.rc
3f34152ee1dc208ebec601c28309120395edb5a673efb63ca0b93af2c5f69099  ./outputs/quick/t3v242u-s3_call_base_without_segment.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3v242u-s3_call_base_without_segment.rc
3f34152ee1dc208ebec601c28309120395edb5a673efb63ca0b93af2c5f69099  ./outputs/quick/t3v242u-s3_call_first_image.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3v242u-s3_call_first_image.rc
f30f858660058a3a0633d1a065c3d37215419fad065a18206755182056404c74  ./outputs/quick/t3v242u-s3_call_first_version.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3v242u-s3_call_first_version.rc
e6ea3c67247c23f55f16319886483e76b3bbc17dc1f541057f5ebb5a0d310908  ./outputs/quick/t3v242u-s3_call_injected.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3v242u-s3_call_injected.rc
392b0ca924a9da8e484316278d19e494c5fdfdfd4b268defdcda77df8ce03004  ./outputs/quick/t3v242u-s3_decorrelate.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/quick/t3v242u-s3_decorrelate.rc
800f9aa7fd41980a8d94857e3347ac410270bc7961e6fa514adc10cdc15390f8  ./outputs/rand/rA-0.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/rand/rA-0.rc
ccbb9435dcd49eaba2f0f021bb12aa0f6d03258c0eb8f77a000b0feb06ebc9db  ./outputs/rand/rA-1.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/rand/rA-1.rc
88386da41372704273f05267465421ebf369231187eb0467d9dbea65e106d73a  ./outputs/rand/rA-2.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/rand/rA-2.rc
43e8f37ace2677845910ffdad5a930a583a6568169b1bd438395d3e6d8ebb5f3  ./outputs/rand/rA-3.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/rand/rA-3.rc
7000378c37e81192c3a61cf9a204aee872305d1d43fbbe55affb7e19db87eb7e  ./outputs/rand/rA-4.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/rand/rA-4.rc
2852b8ce17d269f5a8f4e4256f231dc286639859bf23fec2207477dce9256047  ./outputs/rand/rA-5.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/rand/rA-5.rc
63ba5db370b6e3d0a1a3ca8a977cebbc4557c6301febd099d7b6295241ff4cda  ./outputs/rand/rand-32768-full.out
6e2f34a5a3e7f8db8e3b7f4367710ec7964aac366b83207be14bc6a5b0892a08  ./outputs/sweep/bB-0.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/bB-0.rc
543fd749ff84b652ed62a275e36a8833be9c76c3aecf66d334537dfeb139da7f  ./outputs/sweep/bB-1.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/bB-1.rc
d125a827f51bdfe4304f2a2e7b3996155d4d3188463c37e76ac9ccea50b204e9  ./outputs/sweep/bB-2.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/bB-2.rc
03b86bbe6627761492fc0fddacc658042d8bdb502972d06e6c103852be0ea850  ./outputs/sweep/bB-3.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/bB-3.rc
998b0ccd40cf5767c371ee039952f0e0aa11e146c8a40ceb53ec8bf3ab8190de  ./outputs/sweep/bB-4.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/bB-4.rc
9cd4244f5b4781dadae376d0602cdfe16808e1122634ee2fc01062950cc975cf  ./outputs/sweep/bB-5.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/bB-5.rc
03072e09a1cba4eeef3016e1f12eb414634f1010a44f1a14ab158909d9a010bc  ./outputs/sweep/bB-remount-refused-count.txt
7e494a35be1f6b43e97bf887c9ab18e5d60444e204ea3293c12ff8c0a5e35111  ./outputs/sweep/per-trial-vA.txt
0a0b3c54a194dbd3edd9ff230719d2e526fc80ac7e871b830b300fa3243d7497  ./outputs/sweep/per-trial-vB.txt
2d20d82d46366bc5ce4705c00b56134f870cf9b0c764f1d9dca3db97faa6a52c  ./outputs/sweep/vA-0.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vA-0.rc
849e32a3ef9dede248f4d161cf4e8d130e962dc465cd768b9ef004a129a96b0f  ./outputs/sweep/vA-1.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vA-1.rc
18d47970a3ee08fe497b95bbc4286d9e00479941e586deeeec1eb0028a84d404  ./outputs/sweep/vA-2.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vA-2.rc
4e035254980ede819b0a483e073d21c5e4e7536aabe110e74148d31549a9bc80  ./outputs/sweep/vA-3.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vA-3.rc
659cf7fb9db43f477db1a9b5fe0afc017e473d80e6e1629bc64b84c1edd0130c  ./outputs/sweep/vA-4.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vA-4.rc
2476541d50b9da70ad7708762a65ec92cbe6fde51ea038407612fb8612e596de  ./outputs/sweep/vA-5.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vA-5.rc
dacfd40add001c65ef01f2615a02761f0a5c1af2cba15f37a5a4fdb80374170a  ./outputs/sweep/vB-0.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vB-0.rc
804f308809f97d1acf5bbc498dc7146f59c6fb0726854bf76f22153092a41562  ./outputs/sweep/vB-1.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vB-1.rc
5b22a0f9e51f2bdb211074d979a5440ad5d95174a306882bbfdeb8dafe26c1e7  ./outputs/sweep/vB-2.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vB-2.rc
b962a7e05641310767169d61fff0816818552e51649097847b37e0876a07bf48  ./outputs/sweep/vB-3.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vB-3.rc
4643e91e5c7c52c92aa8fc0172ccf7b9fe677b513fa93a35fb973067787e50a2  ./outputs/sweep/vB-4.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vB-4.rc
8cbf75cf0d8e111efcd3a8a5d25ed6bee412a2c36dd1e748c6d4857704207899  ./outputs/sweep/vB-5.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vB-5.rc
b6ba2ee22e65b871a5331379def5a04cf9a777e618e9b5ff1146337f9d0573be  ./outputs/sweep/vC-0.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vC-0.rc
d6069a6f92aadac3bd660ff16f3e6793cf0d411606a7eae4cc07a747b9dd9776  ./outputs/sweep/vC-1.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vC-1.rc
a22b66487f4aae5e7e4884c7230c45a3a19c2baa81e1462107a13956c870b3ca  ./outputs/sweep/vC-2.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vC-2.rc
c59afbadfcabd167d9128f5096465b8ff0793fe3d968b8826dfc07196daa26de  ./outputs/sweep/vC-3.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vC-3.rc
f9b8ea3257c0895a680fac19b19bb26f689896659844fd2a01bc54ba280b002f  ./outputs/sweep/vC-4.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vC-4.rc
30dbfddc4621570d3e838e6c224d0b1d6aaed7b26de9998f861af9ca06fb0a34  ./outputs/sweep/vC-5.out
9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa  ./outputs/sweep/vC-5.rc
99b12c1d4c4eca4545673a09290ac24cac27be5146ef9c88014409e6cdd1222a  ./proto/Cargo.toml
5f4781c54bac2aec9d839cc3e9cdb3479f29b28b40040d9c5fe0807b6452b262  ./proto/main.rs
89679393c8b4ade6b95607d1b415d2fe11f0b6ecb680bbd3db31f698ba9ae070  ./tools/patch_copy_2.py
fe441cee18690c9116bfbf626c1193f2d04414e92ac55c8c2398078cdde43768  ./tools/patch_copy_3.py
f1518d16dd92d9a313f3856ef9d57f436706d5e5dbfa4adf5efb09d67dfc5001  ./tools/patch_copy.py
cfbf4b49bbe852e22c92af927cda536512501bf3bff285810597186de1b873c9  ./tools/per_trial.py
1e768bf89e1c01a3658938f8107d6c2fc12d9e79985b993f9486b323b8788fef  ./tools/rerun.sh
1c85082220659b80fd684b64d1e74b41adce82828918f08ab836effe41a6414e  ./tools/summarize_trace.py
```

## 引文核对

```
$ python3 research/scripts/cite-check.py research/prompts/m3-formal-code-b18-r1-opus-output.md --root /tmp/claude-1000/m3-formal-code-b18-r1/tree --root . --background research/prompts/_m3-formal-code-b18-r1-background.md
  ✓ 核了 16 处引文，对上 16 处，没判 0 处（逐处列在上面）
```
