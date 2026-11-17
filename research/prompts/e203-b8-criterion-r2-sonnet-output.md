# e203-b8-criterion-r2 辩方腿报告（Sonnet，2026-11-13）

<!-- doc-lint:not-numbers B1 B4 B8 F1 F2 F10 K100 K1000 FTB W1 W4 W7 C1 C2 C3 -->

立场：替被判出局或被攻的一方辩护，复核第一轮判决（`research/prompts/e203-b8-criterion-r1-main-verification.md`）够不够得着。不替主 agent 撤销或维持判决。

## 〇、范围与读数口径

- 没读禁读清单里的文件（另一条腿的报告、执行员各份 result.md）。读了：背景材料正文第一至五节、第一轮判决与岔路单、E207 与 E209 实验页（`.claude/kb/experiments/207-严格判据下的扫类步数.md`、`209-尖峰成因.md`）、E203 页、登记 `research/prompts/e207-preregistration.md` 与 `e209-preregistration.md`、`crates/singlefs-core/src/transaction.rs` 与 `mount.rs` 里被点名的行、E207 与 E203 装置里的行表、`research/results/` 下被引的合并报告与一份 E207 产物。
- 读数标法：「量的」= 我在被引产物文件里逐字找到的整行；「算的」= 我用 `awk` 对产物里的数做的算术或计数（脚本在草稿目录，sha256 与复跑命令在末尾「要主 agent 跑的命令」一节）；「推的」= 由量的或算的推出、没被任何产物直接读到。我的算的与推的都待主 agent 复跑才能进判决。
- 背景材料第二节有一句口径我现查后不能照抄：「登记里计数的扫类步数不数这几处的拷贝与比较」。E207 的产物里这几处有行（第六节给行与数），B8 那一列把它们按差量定价。详见第六节。

## 一、结论一览

| 题 | 复核结论 | 一句话 |
|---|---|---|
| 1 第一轮判决 | 站住；两处够不着 | 第三节形态在 E207 上跑出了它设计要给的结果（记输、作废、样本外）。够不着：窗口取法没写；`C1` 对改动 1000 单元那一档的力度没写。两处都是同形打中所有替代形态，不是第一轮判错 |
| 1 B8 那一行 | 守住一半 | 守得住：「典型的后来发布（中位）步数不随总量涨」（两种状态、六点序列都 `stable_does_not_grow`）。守不住：「每一次发布」——挂载后第一次（`first` 的 `all` 格 α 0.608–1.180，六点序列 `stable_grows`）；后来发布里最坏的一次（`later_peak`）读不稳，既不能说随也不能说不随 |
| 2 尖峰另算 | 「开着」站住；够判的比第一轮多，够不到登记写的两个条件 | E209 把载体定到了「挂载的写行那一步」（小几何，5 点），没把规则写成能预测的条件（行一、二、三都不成立），没到大几何 |
| 2 三条出路 | 都不放宽判据的两条（跑过四小时、存读入口）这一刻都走不通或不值；改最大点是放宽；登记已写了回落 | 作废条款 7 与 F10 已写好：大点记「本次未跑」，Q10 只报低端与中点（十倍）。此外我推：大点即使量出，`C1` 预计不红 |
| 3 整份拷贝与比较 | 罩它（依据更硬）；B8 那一行不罩它，是模型已按差量定价 | 「口径少数了什么」：没少数；B8 列隐含「这几处已换成按差量」，这一条无实现、无实验 |
| 4 E209 三处口径 | 都改不动已判的五行；第一处是「判别子观测不到」，要另立一轮才能用；第三处是「打中不分辨臂」，作废条款 1 已处理 | 第一处我算得：只用下限 28 的旗与窄规则在 742 个操作上 0 处不符（算的） |
| 5 关系 | D8 已定项 16：两页都写备料；D28 已定项 1：两页都写不影响；E203 选臂：证据上动不了 | 第一轮「交用户」前两项用户 2026-11-09 都已定，但岔路单第 9、10 行的够判条件没满足 |

## 二、判决条目表（第一轮判决第二节各格、第三节、第四节三行）

「站得住」= E207、E209 的读数不改这一格；「够不着」= 读数没碰到这一格；「同样打中别的臂」= 读数打中了，但同一形状也打中第一轮没选的做法。

| 判决条目（`e203-b8-criterion-r1-main-verification.md` 行） | 辩方判定 | 证据 |
|---|---|---|
| 记一次输（18） | 站住 | E207 登记第一节原样记下同一行；产物行在 `research/results/e206-fixed-point-cost-report-2026-11-09.out:706`：「name=q8_peak arm=p2a axis=data_units low_end=3763 high_end=3231038 alpha=0.978」 |
| 题面写法「步数除以碰到的单元数」（19） | 站住（代数），经验上没被试过 | 理由是 α(步 ÷ 分母) = α(步) − α(分母)，不依赖读数。E207 里这种读法只用在两端 `user_units` 不同的格（Q3，D100、D10），那一档被作废条款 1 作废：「quantity=void_clauses segment=new tier=user_units_equal_declared applicable_cells=54 scan_control_red=12 linear_control_red=54 scan_control_void=true」（`research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out:5250`），Q3 的 24 格带同一个 `scan_control_void=true`。没有一格读数反对它，也没有一格支持它 |
| 分母取各轮释放之和（20） | 够不着 | E207 的读数不用这个分母 |
| 分母内生（21） | 站住 | E207 用外生的 `user_units`；`declared_against_mapped` 14 行 `ratios_outside=0` `enough_first_touches=true`（实验页 16 行）。`files_1000_units_100` 那次尖峰 `final_roles=452` 的操作，E207 第二段读到「pulled_nodes_first_merge=410 final_nodes=433」（`research/results/e207-strict-scan-steps-report-second-2026-11-13.out:3882`），即 433 个节点里 410 个是释放链闭包新并进的，与「分母由被判实现算出」的担心同向 |
| 改动大小不变的发布读原步数（22） | 站住 | `one_user_unit` 档两端 `user_units` 都是 1，读原步数；`C1`、`C3` 42/42 红（实验页 17 行） |
| 分母写成哪个（23）：取外生 | 站住 | 同上；E207 的 `user_units` 由历史驱动按构造声明，不读实现输出（登记读法 2） |
| x 取旋钮还是取量到的总量（24） | 站住 | `one_user_unit` 档 x 取记录数，`C1`、`C3` 42/42 红；`user_units_equal_declared` 档 `C1` 只红 12/54，该档作废（实验页 17 行）。这一格同时说明：阳性对照不红时作废这一条真的拦下了一个本会读出「不随」的档 |
| 新形态判 B8（25）：「W1 第 2 次起 0.20–0.26」 | 够不着（统计量不同） | 第一轮读的是 W1 第 2 次起；E207 的 `later_median` 两点 α 0.152–0.192（K100），`later_peak` 0.470–0.502。读法不同，不能拿来互相证。同样的区别在第三节说明 |
| B1（26）、B4（27） | 站住 / 够不着 | B1：E207 产物里每次发布的整份拷贝行有数（第六节 6.2）；B4：E207 没量 |
| `files_then_big_*`（28） | 站住 | E207 把 FTB 当样本外量了：同一次挂载 `later_*` 的 `steady` 格 α 0.207–0.241 不随；重挂载 `later_median` 0.321、`later_peak` 0.253 介于；`first` 的 `all` 格 0.608 随（`…-r3.out` 4776 行起）。FTB 进不了「改动大、总量更大」那一格，判决那句仍对 |
| fs-design 那句（29）：「单次绝对代价任何比值都罩不住」 | 站住，并多了一个量的 | 第四段日志每次发布约 9.7 秒（实验页 310 行，执行员量的），E207 不量挂钟，这是第一个绝对时间读数 |
| 第三条路（30）：整份覆盖写走分批 | 够不着 | E207 不量分批；用户 2026-11-09 已定整份覆盖写登欠账（`records/2026-10-13-里程碑四预开发.md:1587`） |
| 第三节：新登记形态，「这些条件全部零轮」（34） | 站住；有两处第一轮没写 | ①窗口：`later_peak` 取一个点自己窗里最大的操作，登记没写两端窗口要不要相同；② `C1` 的力度：每次发布多扫一遍、每条记录 1 步，登记没写它在「改动 1000 单元」那一档上够不够红。第三节 3.3 说明 E207 的读数打中了这两处，替代形态（`later_median`、除以分母）同样会被打中 |
| 第四节交用户三行（40–42） | 前两行已定；第三行动不了 | 第八节 8.4 |

## 三、第 1 条：E207 的读数下，第三节新登记形态与 B8 那一行

### 3.1 读数

- 36 格的分布（算的：我在 `research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out` 里数 `name=cell segment=new … tier=one_user_unit … arm=tested`，集合 K100、K1000、FTB 各 12 行）：随 5、介于 8、不随 23。随的 5 格：K100 同一次挂载 `first` 的 `all`（1.180）、`later_peak` 的 `all` 与 `steady`（各 0.502）、K1000 同一次挂载 `first` 的 `all`（1.005）、FTB 同一次挂载 `first` 的 `all`（0.608）。介于的 8 格：K100 重挂载 `first` 两格 0.484、`later_peak` 两格 0.470，FTB 重挂载 `later_median` 两格 0.321、`later_peak` 两格 0.253。
- 三个布尔量怎么算（装置源码）：`row_one_holds` 要求全部 `steady` 格「不随」且两条控制臂红（「let row_one_holds = !steady.is_empty()」（`scripts/experiment-apparatus/e207_strict_scan_steps.rs:1518`））；`row_two_flips` 只要任一 `later_*` 的 `steady` 格「随」且控制臂红（「.any(|cell| cell.pick != Pick::First && cell.outcome.band_of(Arm::Tested) == Some(Band::Grows) && controls_red(cell));」（`scripts/experiment-apparatus/e207_strict_scan_steps.rs:1522`））；`only_the_first_grows` 是对（集合、状态、档）各组取「或」（「only_the_first_grows |= first_all_grows && rest_do_not;」（`scripts/experiment-apparatus/e207_strict_scan_steps.rs:1534`））。所以 `row_two_flips=true` 与 `only_the_first_grows=true` 同时为真不矛盾：K1000 与 FTB 同一次挂载两组满足 F2 的形，K100 同一次挂载那一组不满足。引这条总判时要带这个限定，不能读成「只有第一次随」。
- `row_one_holds=false` 不靠 0.502 那一格：`steady` 格里不是「不随」的有 5 个（0.502 一格随，K100 重挂载 `first`、`later_peak` 与 FTB 重挂载 `later_median`、`later_peak` 四格介于），去掉 0.502 仍是 false。`row_two_flips=true` 只靠 0.502 那一格。
- 0.502 那一格的两端窗口：高端的峰 22641 是 `files_1000_units_100` 的 W7 第 1 次，低端 `files_100_units_100` 没有扩展窗口（登记「设计时已经存在的数」一条：「这是量窗的定义与点的大小定的，不是读法造成的」（`research/prompts/e207-preregistration.md:509`）；扩展窗口只在少数点：「3. **扩展窗口**（只在 `files_1000_units_100` 的四个探针项里」（`research/prompts/e207-preregistration.md:223`））。同一次挂载高端 401 个操作、低端 80 个（实验页：`W[567] ops=321`，`.claude/kb/experiments/207-严格判据下的扫类步数.md:196`）。`later_peak` 是「其余里步数最大的一次」（`research/prompts/e207-preregistration.md:37`），样本数不等的最大值天然偏向样本多的一端。
- 两点 α（算的，`window_and_time_arithmetic.sh`）：公共窗 ln(15167/7167) ÷ ln(241260/25346) = 0.3327，自己窗 ln(22641/7167) ÷ ln(250190/25346) = 0.5024；输入的数都在产物行里（第一段合并报告 4552 行 `ends=7167/1/25346,22641/1/250190`；复量段 2636 行 `ends=…7167/1/25346…15167/1/241260`）。实验页 231 行说公共窗两点 0.333 「没有落成文件」，我这个是对产物里的数重算的，不是那次钉的读数。
- 复量段 12 格（公共窗、被测臂、`one_user_unit` 档）的判语（算的，`research/results/e207-strict-scan-steps-report-hundred-units-per-file-repeat-2026-11-13.out` 2612–2736 行里数）：同一次挂载 `first` 的 `all` 是 `stable_grows`（β 1.058，2612 行），`first` 的 `steady` 与两种 `later_median` 是 `stable_does_not_grow`（β 0.177–0.187，2652 行是 `later_median` 的 `steady`）；两种 `later_peak` 的 `unstable`（β 0.345，2636 行）。重挂载 `later_median` 两格 `stable_does_not_grow`（β 0.220），`first` 与 `later_peak` 各两格 `unstable`（β 0.401 与 0.448，相邻 α 0.488、1.376、0.049、−0.042、0.561，2732 行）。重挂载没有扩展窗口、两端窗口相同，这一半 `unstable` 与窗口不对称无关。

### 3.2 B8 那一行（岔路单 9 行「准入之前的步数只随用户这一次的改动涨」）拆开读

| 句子 | 辩方判定 | 依据 |
|---|---|---|
| 典型的后来发布（`later_median`）的步数不随总量涨 | 守得住 | 两种状态 × `all`/`steady` 共 4 格 `stable_does_not_grow`，β 0.177–0.220；两点 α 0.152–0.192（K100）、0.163–0.169（K1000）。这些格 `C1`、`C3` 在 `one_user_unit` 档 42/42 红，不是读不出 |
| 最坏的后来发布（`later_peak`）的步数不随总量涨 | 读不稳，不能写站住也不能写翻面 | 两点 α 贴着 0.5（0.502、0.470）；六点序列 `unstable`，相邻局部 α 在同一次挂载上从 −0.055 到 0.869、在重挂载上从 −0.042 到 1.376。0.502 那一格靠窗口不对称多出约 0.17（0.502 对 0.333）；重挂载那一半窗口相同而同样 `unstable`，所以「窗口不对称」解释不了不稳，只解释那一格的 0.502 |
| 挂载后（历史后）第一次发布不随总量涨 | 守不住 | `first` 的 `all` 格三组随：1.180、1.005、0.608；六点序列 `stable_grows`（β 1.058）。这是 F2 的形，第一轮判决 25 行已写「都来自量窗第一次覆盖写的固定点尖峰」，E207 读数与它一致 |
| 「每一次发布」这个全称 | 守不住 | 上两行的并 |

B8 在 `one_user_unit` 档既没「站住」也没「翻面」：站住要求全部 `steady` 格不随（有 5 格不是），翻面要求读得稳（唯一一格不稳，且复量判 `unstable`）。用户 2026-11-13 已定这一格现状留着（`records/2026-10-13-里程碑四预开发.md:1688`），我没有重判它的去留；上面只是回答「B8 那一行守得住哪一半」。

### 3.3 第一轮判错了吗，同样打中替代方案吗

- 没有一条读数说明第一轮判错。第一轮写的：新形态下 B8 今天不是干净的站住（「**不放宽的写法下 B8 今天不是干净的站住**」（`research/prompts/e203-b8-criterion-r1-main-verification.md:25`）），E207 读到的正是这样。
- 第一轮没写、E207 读数打中的两处：
  1. 窗口：「后来发布里最大的一次」在样本数不等的两端上不可比。换成 `later_median` 做主读会读出「不随」，但那是换成更宽容的统计量：按 `.claude/rules/evidence-discipline.md:111` 「新形态要被**原来那套判据**判得更严或相同；放宽一个字都不行」，不行。同一形状也打中第一轮没选的做法（题面写法、分母取释放和）：它们读的也是 `later_peak`。
  2. `C1` 的力度：每次发布多扫一遍、每条记录 1 步，在改动 1000 单元那一档上被每单元 218–272 步（「被测臂每单元 218–272 步」，`.claude/kb/experiments/207-严格判据下的扫类步数.md:17`）淹没，K1000 同一次挂载首格 `C1` 只有 0.313（「arm=c1 ends=240503/1000/22420,488927/1000/216376」（`research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out:4685`）），与登记事先推的 0.315 一致。任何要求「每次发布多扫一遍」必须红的形态在这一档都一样作废，不是 B8 独有。这是「打中不分辨臂」（`.claude/rules/evidence-discipline.md:124`）：共用前提是对照臂的力度，该另立一笔账（更强的对照臂），不拿它判臂。
- 什么现象会推翻「第一轮没判错」：若 K100 两端窗口取成相同、并且重挂载那一半的 `later_peak` 也稳稳地随（`stable_grows`），则第一轮「B8 站住」的路径被打穿。现有读数是 `unstable`。

## 四、第 2 条：「尖峰另算」那一行（岔路单 11 行）

### 4.1 够判条件两句

岔路单 11 行的够判条件：「把尖峰的成因定到发布里的具体一步，并写清它重现的条件」（`research/prompts/e203-b8-criterion-r1-forks.md:11`）。两句各自对着 E209 的五行判定。

### 4.2 E209 读到了什么（量的，`research/results/e209-spike-cause-report-third-2026-11-13.out`，行前缀 `E7RESULT E209RESULT` 略）

- 载体（行四，成立）：五个复现点挂载自己的写行一步带出了尖峰的节点，share 0.949–0.981，整行如「name=mount_carried_step point=files_100_units_100 carried_nodes_by_first_step=write_row:51」（`research/results/e209-spike-cause-report-third-2026-11-13.out:850`），同一行里 `first_merge_site=release_not_covered plan_function=row_publish_plan_on_file_version plan_position=crates/singlefs-core/src/mount.rs:5841`。并入站点 `release_not_covered` 对着 `crates/singlefs-core/src/transaction.rs:14087` 那一处 `merge_into_the_rewrite_set_walking_the_release_chain(`（落点站点是 14116）；该函数的现行位置我现查过（`sed -n 14087p` 与 `14116p` 都是这个调用）。
- 触发（行二，不成立）：旗跟着被碰的操作走，不跟序号走：「name=ordinal_shift point=files_100_units_100 walk_forward_first_flagged_ordinal=1 fillers=8 verdict=follows_the_operation」（`research/results/e209-spike-cause-report-third-2026-11-13.out:848`）。
- 触发（行三，不成立）：压低攒下的量，最小的取样点就已经不旗：「name=presweep point=files_100_units_100 flagged_counts=0 unflagged_counts=1,2,4,8,16,32,64 between=0..1」（`research/results/e209-spike-cause-report-third-2026-11-13.out:986`）。
- 规则（行一，不成立）：宽读法 27 处训练失败，窄读法（`fresh ≥ 18`）训练 0 处失败、检验各 1 处不符（`research/results/e209-spike-cause-report-third-2026-11-13.out:832`）。
- 步数落在哪几行（行五，不成立）：分辨够的 8 个旗上操作全是「分散」，前三大行占多出步数的 0.717–0.777，门槛 0.9（实验页 772–779 行）。
- E207 一侧（算的：第一段合并报告 `q6_spike_flags` 各行）：同一次挂载 W1 第 1 次被旗的点：`files_100_units_100`（53）、`files_1000_units_100`（433）、`files_100_units_1000`（257）、`files_10000`（73）、`files_then_big_10000`（75）、`files_10_units_10000`（219）；这些点重挂载一侧的 W1 第 1 次都没被旗；`files_30000` 同一次挂载 W1 第 1 次旗上 155 个节点、重挂载 80 个操作旗上 0（「name=q6_spike_flags point=files_30000 state=remounted operations=80 median_final_nodes=11 line=44 flagged=0」（`research/results/e207-strict-scan-steps-report-third-2026-11-13.out:966`））。

### 4.3 两句各到哪一步

| 够判条件 | 到哪一步 | 说明 |
|---|---|---|
| 成因定到发布里的具体一步 | 到了，限小几何 | 载体是「历史之后第一次碰到没被带出的旧叶的那次发布」：没有挂载时是用户那次覆盖写，有挂载时是挂载自己的写行那一步；代码站点是固定点第一轮释放站点并入的闭包（14087）。五个复现点 share ≥ 0.949 |
| 写清重现条件 | 没到：只有否定句加一个载体 | 行一、二、三三条能预测旗的规则都不成立；行四说的是挂载吃掉了尖峰，不是什么时候出现。没有一条能写成「历史怎样、操作怎样，则旗」的谓词。我对窄规则另算了一个读数（第七节 7.1），那条路要另立一轮登记 |
| 步数落在发布里哪几行 | 没到（行五不成立） | 尖峰多出的步数分散在很多行；这与「多出来的是节点，每个节点在很多行都要付步」同向（推的，Q13 的 `steps_per_extra_node` 是登记预留的旁证，页里没据它下结论） |

### 4.4 三样各削掉多少

- 只在小几何上量：E209 的复现点是 F = 75、100、200 与两个 U 变体，旗上操作 39–105 个节点；E207 里最大的尖峰是 433 个节点（`files_1000_units_100`），4 倍于 E209 最大的。行四「定位到写行一步」在大尖峰上没有直接读数，是推的。E207 的前提半边在大点上有读数：重挂载一侧第一次都不旗（4.2 末条），与行四的前提一致。削掉的是载体的位置与占比，不是「重挂载一侧不旗」这件事。
- `files_10_units_1000` 不复现：E207 自己在这个点也没有尖峰（「name=q6_spike_flags point=files_10_units_1000 state=same_mount operations=80 median_final_nodes=7 line=28 flagged=0」（`research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out:3248`）），所以不复现与 E207 一致，不是 E209 装置的毛病。削掉的只是「F 小而 K 大」这一侧没有可判点；K = 1000 的 `files_100_units_1000` 在 E207 里有尖峰（257 个节点，2604 行），E209 没取它。
- E207 第二段 `none_of_the_three`（`research/results/e207-strict-scan-steps-report-second-2026-11-13.out:3892`）：它说 E207 登记的三种前缀假设都不是重现条件。E209 的行二、行三否定的是序号型与门槛型，与它同向，没有削掉 E209；削掉的是「用 E207 登记的三种假设里任何一种当重现条件」这条路，所以重现条件只能从 E209 的否定句加行四拼，拼不成谓词。

### 4.5 复核结论

- 「尖峰另算」那一行记「开着」站得住：够判条件两句里第一句只在小几何到了，第二句没到。还差的比第一轮少得多：（一）大几何（至少 `files_1000_units_100` 一点）上载体与大小；（二）一条能预测的谓词（要另立登记，原因见第七节 7.1）。
- 「另算」这个读法本身（尖峰是固定点把攒下的角色一次带出、挂载把它提前带出）：小几何上有据，大几何上是推的。
- 提醒一条：若用「另算」把 B8 改写成「除一次性的挂载后第一次之外」，是放宽，不是收严；按 `.claude/rules/evidence-discipline.md:111` 与「推翻一个旧结论时，要比确立时更严，不是更松」（`.claude/rules/evidence-discipline.md:93`），要交用户、标零轮，第一轮记的那一次输不撤。
- 什么现象会推翻这一节：（一）在 F ≥ 1000 的点上量到写行没带出尖峰节点（share < 0.5），或尖峰在挂载后的非第一次发布上重现；（二）E209 的行五换口径后判「定到三行」。

## 五、第 3 题（背景材料第四节第 2 问）：量不出的那一格，三条出路

### 5.1 登记已经写好的回落（量的，`research/prompts/e207-preregistration.md`）

- 已定项 1：「加总量跨度 100 倍的大点，放进独立的最后一段（第四段）；不改 `C1` 的系数、不改判据」（`research/prompts/e207-preregistration.md:56`）；同一条写明预告「**贴线，不保证红**」与「b 的分界约 230（b ≤ 230 红、以上不红）」。
- 作废条款 7：「Q10 缺这一点，K1000L 当本次未跑，不拿两点算 α 冒充三点；只剩低端和中点时 Q10 只报 {低端, 中点}（总量十倍），不当作跨 100 倍」（`research/prompts/e207-preregistration.md:409`）。
- F10：「后者：大点记「本次未跑」，另两个小点照报，第一至三段结论不受影响」（`research/prompts/e207-preregistration.md:393`）；前者（`C1` 不红）：「第四段 `uK` 档作废只报数，`C2` 照报，交主 agent 定改强系数」（同一行）。
- 所以「都站不住的话这一格该怎么记」登记自己答了：大点记「本次未跑」，`user_units_equal_declared` 档在第四段没判，行一那一半仍开；第一至三段该档作废只报数不变。不需要为此改登记。

### 5.2 事实与用时（量的、算的、推的分开）

- 量的：最大点整项一片跑 142 分钟到 `publishes_done=1837`（共 4000），最后一段每次发布约 9.7 秒（「最后一段每次发布约 9.7 秒」（`.claude/kb/experiments/207-严格判据下的扫类步数.md:310`））；中点 434.584 秒（实验页 305 行）。
- 算的（`window_and_time_arithmetic.sh`）：按「每次发布用时随已放发布数线性涨」纯二次推，4000 次约 40486 秒，14400 秒内能放约 2386 次；按执行员拟合的 2.48 次幂推约 55796 秒；中点 434.584 秒乘 100 约 43458 秒（每次发布十倍、发布数十倍）。三个数都在 14400 秒的 2.8–3.9 倍。中点乘 100 与前两条共用「每次发布用时随记录数线性」这个前提，不算独立的校验。
- 分片不缩短最后一片：入口文件头写「放前缀不比量快多少，所以第 i 片的挂钟约等于不分片时放到序号 to 的那一段，最后一片仍要放完整段历史」（`scripts/experiment-apparatus/e207-strict-scan-steps-run.sh:41`）。
- 服务的硬上限：「确认放不过四小时硬上限」，预计到四小时的进队时就打回，出路是收小范围或分片重交（`.claude/agent-common.md:69`）。

### 5.3 三条出路逐条辩护与反驳

| 出路 | 辩护（买到什么、不放宽什么） | 反驳（这一刻走得通吗、值得吗） | 判 |
|---|---|---|---|
| 一 整条跑过四小时 | 判据、点、`C1` 都不动，不碰 `crates/`，量到的读数可进 Q10 | ①服务在进队时就拒（`.claude/agent-common.md:69`），绕开服务直接起项目代码被项目代码闸拒（同文件项目代码闸一条）；要先改工具规则，那是另一个决定。②独占一台约 11–16 小时（推的）。③**期望买到的少**：登记的分界是 b ≈ 230，被测臂每单元步数在第一段 K1000 上从 218 涨到 272（「arm=tested ends=218083/1000/22420,272551/1000/216376」（`research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out:4684`）），随总量涨；大点高端 b 预计不低于 272，`C1` 预计落在「不红」一侧，该档按作废条款 1 作废只报数（推的，登记自己的推算输入是猜的） | 不放宽判据，成立；走不通且预期买不到判 |
| 二 `crates/` 加会话与读数状态存读入口 | 每片接着上一片，每片都短；判据与点不动 | 改产品代码只为实验；类型没有存读（「`grep -n serde` 两个 `Cargo.toml` 无命中，量的」，实验页 312 行），要决策、实现员、变异、一条「存读之后与不存读逐字节相同」的钉；读数状态（插在副本里的计数）本来就在装置里，要另存。同一个 `C1` 预期不红的问题照样挡在后面 | 不放宽判据，成立；代价大，买到的同出路一 |
| 三 改登记里的最大点（换成跑得完的规模） | 能量出东西，总量跨度小一些 | 见 5.4：在 F10 已打中之后改点，不是收严，是放宽；已定项 1 写的是跨度 100 倍 | **放宽**，不成立（除非作为追加的新点另立，见出路四） |

### 5.4 「改登记里的最大点」算不算放宽

- 规则：「失败条款打中一条写得含糊的臂时，不许不记一次输就把臂「澄清」成强的那个读法、再宣布它赢。」（`.claude/rules/evidence-discipline.md:104`）；合法出路三步：记一次输、只许收严（「新形态要被**原来那套判据**判得更严或相同；放宽一个字都不行」（`.claude/rules/evidence-discipline.md:111`））、写明收严在哪；缺一步算新实验。跑产物之前修订臂或判据合法（`.claude/rules/evidence-discipline.md:117`）；第四段已经跑过两次没量出，F10 的后者条款已打中，不在「跑之前」。
- 换成更小的点：跨度变小，`C1` 的解析项占比降、「判别力随跨度涨」（「跨度小时对数比的分母小，一个整数步的差就能移动 α；跨度大时 `C1` 的解析项…占比升高，判别力随跨度涨」（`research/prompts/e207-preregistration.md:347`））；用它读「大点 S」（行一站住）比登记写的 100 倍松。写不出「比旧形态多要求了什么」，所以是放宽。
- 已定项 1 允许到哪一步：它允许「加总量跨度 100 倍的大点」，没写换点；100 倍是用户 2026-11-09 选 (c) 的内容（同一行「用户 2026-11-09 定「加大点，分段跑」」）。所以换点不在主 agent 的权限里。
- 允许的相近做法：设计员修订 2 在产物之前「只追加」点（`research/prompts/e207-preregistration.md:507`，六点序列）。追加一个中等点（我推：`files_316_units_1000_unit_area_2_pow_21`，总量约 31.6 倍；用时按中点乘 10 约 4340 秒，推的）作为新集合另报，不顶替大点，是合法的先例；但它读出来的 `C1` 也预计不红（跨度 31.6 倍介于第一段的十倍与 100 倍之间，十倍时已是 0.313），买不到判。
- 登记自带的 {低端, 中点} 十倍报数不是改点，是作废条款 7 的原话；第一段 K1000（十倍，U = 2¹⁹）已经读过同一形状，`C1` 0.313 作废。

## 六、第 4 题（背景材料第四节第 3 问）：每次发布按全部分配记录做一遍的拷贝与比较

### 6.1 位置（量的，`crates/singlefs-core/src/transaction.rs` 现查）

- 通用发布路径：`:13213` 「let allocator_before_this_publish = allocator.clone();」（函数 `publish_version_of_trees_relocating_units_after_a_check`，13193 起）；固定点 `settle_the_allocation_record_tree`（13994 起）循环 `:14031`，每轮 `:14076` 「let mut rehearsal = allocator.clone();」，比较 `:14085`、`:14105`，持有记录的节点 `:14093`、`:14106`。
- 背景材料只列了这一份。同文件还有一份同形的：无文件版本的行发布 `settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file`（1975 起）里 `:1930` 拷贝、`:2007` 循环、`:2038` 拷贝、`:2044`、`:2060` 比较、`:2061` 持有（「let allocator_before_this_publish = allocator.clone();」在 `:1930` 也有一处）。挂载的写行发布走 `crates/singlefs-core/src/mount.rs:5841`。
- 它们在准入哪一边：固定点调用在 `:13435`，第二道准入（D28 的式子）在它之后、同一个发布前判函数里：「空间准入（D28（挂载期承诺量） 已定项 1 的式子逐设备合取）：读数取分配器此刻的计数——这次的释放与取落点都还没做——」（`crates/singlefs-core/src/transaction.rs:13488`），「需求 = 这次新写的全部角色（普通分配加固定点，」（`crates/singlefs-core/src/transaction.rs:13489`）：固定点的结果是这道准入的输入。拷贝 `:13213` 在两道准入之前。全部在准入之前。

### 6.2 E203、E206、E207 的口径少数了什么：没少数；B8 那一列把它们定价成按差量

- E203 口径：「扫类的步按最内层循环体每进一次记 1 步，整份拷按被拷值里每个集合的元素个数之和记」（`.claude/kb/experiments/203-准入之前扫类的步代价.md:120`）。装置里两个拷贝行都是扫类行：`("whole_allocator_copy_before_the_judgement", RowKind::Scan),`（`scripts/experiment-apparatus/e203_admission_scan_cost.rs:776`）、`("whole_allocator_copy_in_a_settling_round", RowKind::Scan),`（`scripts/experiment-apparatus/e203_admission_scan_cost.rs:800`）；比较与「持有记录的节点」都经 `records_of_each_leaf`，它也是扫类行（`("RecordsOfEachLeaf", "records_of_each_leaf", "scan"),`，`scripts/experiment-apparatus/e203_admission_scan_cost_counters.py:72`）。
- E207 产物里有这些行，数不小：「segment=caller row=whole_allocator_copy_before_the_judgement calls=1 steps=4575824」（`research/results/e207-strict-scan-steps-files_1000_units_100-head-2026-11-13.out:622`，`files_1000_units_100` 的 W1 第 1 次）。同一次操作之前分配记录 238324 条（登记 163 行，装置钉 `SPIKE_PIN_STATE`），每条记录约 19.20 步（算的：4575824 ÷ 238324）。这是那一行一行的数；同一次操作的 B8 列合计是 168373 步（第一段 `first` 的 `all` 格高端，`research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out` 的 K100 同一次挂载）。
- B8 那一列怎么定价：第一次拷贝记 0（「("whole_allocator_copy_before_the_judgement", 0),」（`scripts/experiment-apparatus/e207_strict_scan_steps.rs:188`））；每轮拷贝记「这一轮改了的槽与记录」（「input.changed_slots_and_records_per_round.iter().sum(),」（`scripts/experiment-apparatus/e207_strict_scan_steps.rs:269`））；`records_of_each_leaf` 记弄脏的叶数（「("records_of_each_leaf", input.dirty_per_call.iter().sum()),」（`scripts/experiment-apparatus/e207_strict_scan_steps.rs:295`））。登记明写：「B8 不是实现」…「所以「B8 站住」说的是这份式子在今天的固定点、今天的发布上的读数，不是某段代码的性质」（`research/prompts/e207-preregistration.md:131`）。
- 所以背景材料第二节那句「登记里计数的扫类步数不数这几处的拷贝与比较」要改写：计数层数了（B1 臂与产物行里有），B8 列把它们记成 0 或按差量。没有被任何实验测过的是 B8 列的前提：这几处换成按差量之后产品仍成立（拷贝 `:13213` 在 `:13230`、`:13235` 做回滚，按差量要换一个回滚办法，D16 的发布语义要不要动，我没查）。登记也不量挂钟（`research/prompts/e207-preregistration.md:395`），这几处各占多少用时没量（实验页 309 行）。

### 6.3 「B8 那一行不罩它」与「罩它」各辩一次

- 辩「不罩」：①B8 那一行的对象是那份式子，式子里这几处已按差量定价（6.2），B8 的读数对它们不设限；②D8 已定项 16 射程的末句「准入之后装单元、拷字节不算运行时决策路径」可以被读成给拷贝开口；③`.claude/rules/fs-design.md:33` 说「被禁止的不是「遍历」，是两件事」：固定点是在拷贝上预演、不是需要答案时没有答案；条款原文：「准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径。」（`.claude/kb/decisions/08-核心索引结构.md:517`）。
- 辩「罩」：①条文：运行时决策路径一格写「**不许**，且代价不许随盘容量增长」，该格列的就有「ENOSPC 准入」（`.claude/rules/fs-design.md:23`），D8 已定项 16 把它读成「只管准入之前」（`.claude/kb/decisions/08-核心索引结构.md:517`）；②位置：这几处都在准入之前（6.1），那句末句说的是准入之后；③E203 自己把它们数成扫类并判过：「B1 今天」一行「够判：留不住」（`.claude/kb/experiments/203-准入之前扫类的步代价.md:54`），「整份拷在基线上占扫类合计 95.7%」（`.claude/kb/experiments/203-准入之前扫类的步代价.md:161`，攻方推的）。
- 判哪边更硬：「罩」。辩「不罩」的三条中，①说的是 B8 这个模型不设限，不是条款不罩；②与位置不符（末句限定准入之后）；③只驳「遍历」，没驳「代价不许随盘容量增长」。辩「罩」的三条是条文、位置、E203 自己的口径，各自独立。

### 6.4 对判决的含义

- B8 那一行不罩这几处，但条款罩。所以「B8 站住」不等于「条款在今天的实现上成立」，也不等于任何实现成立：它成立要有一个前提——这几处换成按差量——而这个前提既没实现也没实验。E207 登记 131 行自己写了这个限定。
- 第四段最大点跑不出来，原因和这几处同一个位置（实验页 309 行：「这几处各占多少用时没量」；9.7 秒一次发布）。这是推的归因，要靠同一点 plain 对拍才分得开。它与第一轮判决 29 行的「单次绝对代价任何比值都罩不住」同向。
- 什么现象会推翻这一节：同一个点的 plain 构建（只拷不插）实测发布用时里，这几处不到一成；或代码里第二道准入并不读固定点的结果（`:13489` 那句不成立）。

## 七、第 5 题（一）：E209 三处登记没写死的口径

登记指 `research/prompts/e209-preregistration.md`。已判的五行：行一、二、三、五不成立，行四在五个复现点上成立。

### 7.1 旗线在窗里操作很少时退化（登记读法 1）

- 登记原文只预见了「旗上操作超过半数」：「一个项里旗上操作超过半数时，中位数本身被拉高，旗线失去意义」（`research/prompts/e209-preregistration.md:24`）；窗里只有一两个操作的情形没写。
- 打中的两处（量的）：「name=operation_row item=files_100_units_100__head__old_mount_old mount_epoch=0 ordinal_user=1 step_index=1 operation=overwrite_old final_nodes=53 flagged=false flagged_at_the_low_line=false flagged_at_the_high_line=false」（`research/results/e209-spike-cause-report-third-2026-11-13.out:374`），窗里只有这一个操作，4 × 53 = 212 大于任何下限 L，下限 14、28、56 三条线都救不了它；另一处「name=operation_row item=files_75_units_100__head__first_mount_first mount_epoch=0 ordinal_user=1 step_index=1 operation=overwrite_first final_nodes=39 flagged=false」（`research/results/e209-spike-cause-report-third-2026-11-13.out:391`），窗里两个操作 39 与 11，线 4 × 11 = 44。
- 算的（`floor_only_flag_check.sh`，对 742 个 `name=operation_row` 数）：登记的旗 52 个；只用下限 28（`final_nodes ≥ 28`）的旗 54 个；窄规则预测（`fresh_mount_narrow ≥ 18`）54 个；窄规则对登记旗 2 处不符（就是上面两处），对只用下限 28 的旗 0 处不符；登记旗与只用下限的旗只在这两个操作上不同。限定：`m = 18` 是用训练集旗上操作 `fresh` 的最小值定的（登记 Q2），`final_nodes` 与 `fresh` 都由同一批节点算出，部分是同义反复，这个 0 不当作机制的证据。
- 能不能改动已判的五行：行一的窄读法两个变体在这份数据上会从「不成立」变成 0 处不符（算的）；但登记 F1 写的是「不改 `m`、不换读法重判（改了就是新实验）」（`research/prompts/e209-preregistration.md:344`），作废条款 6：「判据（第六节门槛、第五节脚本表、第一节读法）在产物落盘之后改动：改了就是新实验，重跑，不作废这一份、也不回头改」（`research/prompts/e209-preregistration.md:366`）。行二、三、五读的是操作很多的窗；行四的前提「挂载后第一次不旗」我数了五个点 `mount_then_first` 的挂载后窗（各 5 个操作，首个操作 `final_nodes` 13、13、19、13、13，都不到下限 28），不靠这条线（算的，同一份合并报告 `operation_row`）。所以这一处只可能动行一。
- 分类：「判别子观测不到」（`.claude/rules/evidence-discipline.md:125`）：旗是拿窗内自己的中位数当线，窗里只有一个操作时旗在任何 L 下都不可能为真，判别子在这个窗里看不见尖峰。不是新鲜度规则被证伪，那一处不符的源头是读法 1 的 4 × 中位数那个分句（也同「打中归错了判据」同族，`.claude/rules/evidence-discipline.md:126`）。该行的处置：「改判据，改完是新一轮」。
- 要不要另立一轮：要，如果想把窄规则升成可用的重现条件（第四节 4.3 第二行）。新一轮的写法：旗线不依赖窗内中位数；原读法与新读法并列、原判不撤（只许收严）。E209 页自己写了「窗里操作很少时旗读法要不要另立，这里答不了（交主 agent 定）」（`.claude/kb/experiments/209-尖峰成因.md:839`）。

### 7.2 行五的可比操作不限挂载次序（登记读法 7）

- 登记原文：「为什么跨项：第一个文件（选择子 `first`）历史里只有一个，它第一次覆盖写的 `old_units` 与之后各次不同，同项内找不到同种。」（`research/prompts/e209-preregistration.md:30`），可比操作按项名字典序取前两个。
- 它只碰行五。行五不成立的依据是分辨够的 8 个旗上操作全是「分散」，前三大行占 0.717–0.777（`.claude/kb/experiments/209-尖峰成因.md:772`），门槛是 0.9，那个 0.9 登记自己写了「0.9 是我在知道第四节第 2 条的数之后写的」（`research/prompts/e209-preregistration.md:269`）。
- 动得了吗：不能。分辨够的定义是 `extra ≥ 10 × noise`（`research/prompts/e209-preregistration.md:269`）；换一个可比操作，多出来的步数最多改变 noise 那么多（假设替代可比操作与现用的差不超过观测到的 noise，推的），占比最多从 s 变成 s ÷ 0.9：0.777 ÷ 0.9 = 0.863、0.717 ÷ 0.9 = 0.797，都不到 0.9。口径二造成的是 26 个 `not_resolved` 与 15 个 `no_comparable_operation`（`.claude/kb/experiments/209-尖峰成因.md:782`），那些是没判，不是判反；而「一处分散即不成立」。
- 分类：口径二本身不属于「判别子观测不到」或「打中不分辨臂」，是 F8（分辨不够）登记预见过的情形。但行五的前三大行 ≥ 0.9 这个判别子在 0.72–0.78 稳定（8 个值跨 F = 75…200、两个 U），多出的步数是「多出的节点数 × 每个节点在很多行都付步」的结构（推的，登记预留的 Q13 没跑），这个判据对「定到哪一步」这件事本身观测不到。不必为口径二单独另立一轮；若另立（7.1），顺带把行五换成「多出的步数是否正比于多出的节点数」加行四的节点位置，这是换判据，不是收严，要写明。

### 7.3 不复现的点算不算几何敏感性的取样点

- 作废条款 1：「某个点的 Q0 判「不复现」：该点上行一至行四的判记「装置走不到」，不是作废全部（F3）」（`research/prompts/e209-preregistration.md:361`）；F10：「任一量在第八节几何敏感性表的某个取样点上与训练点（F = 100、U = 2¹⁹、L = 28）判不同」（`research/prompts/e209-preregistration.md:353`）。两句对 `files_10_units_1000` 一点给出不同的读法，E209 页并列写了（`.claude/kb/experiments/209-尖峰成因.md:805`）。
- 动得了五行吗：不能。五个复现点之间五行的判一个都不翻（同一页 805 行：「五行的判（不成立、不成立、不成立、成立、不成立）在五个复现点上都不翻」）；两种读法的差别只在几何敏感性那几行的 `stable` 标签。
- 分类：「打中不分辨臂」（`.claude/rules/evidence-discipline.md:124`）：这一点上复现闸不过，五条候选一起没有输入，共用前提是「尖峰在这个几何上出现」。该条的处置是「判臂只看分辨得出臂的那几格」，作废条款 1 已经这样办。不必另立一轮；登记缺的是一句话：不复现的点不进几何敏感性比较，或进但只对复现闸一量报不稳。这句话交主 agent 定，不改已判。

### 7.4 汇总

| 口径 | 动得了已判五行吗 | 分类 | 要另立一轮吗 |
|---|---|---|---|
| 一 旗线在窗里操作很少时退化 | 登记不许；数据上只会让行一窄读法从「不成立」变「0 处不符」，不碰行二、三、四、五 | 判别子观测不到 | 要，若要把窄规则升成重现条件；原判不撤 |
| 二 行五可比操作不限挂载次序 | 不能（占比上界 0.86 < 0.9） | 无，F8 预见；行五的 0.9 判据疑盲 | 不必单立；若另立顺带换行五判据 |
| 三 不复现的点算不算取样点 | 不能 | 打中不分辨臂 | 不要；补一句登记 |

什么现象会推翻这一节：（一）把 `final_nodes ≥ 28` 的旗放进装置重算后窄规则出现 ≥ 1 处不符（我的 0 是 awk 对合并报告的数，不是装置读数）；（二）行五的替代可比操作使前三大行占比 ≥ 0.9。

## 八、第 5 题（二）：对已定条款的关系，与 E203 选臂

### 8.1 四种关系怎么定

「关系是 支撑 / 推翻 / 备料 / 不影响 之一。支撑、推翻要写到分项，那条分项的 `**依据**：` 要引回这个实验」（`.claude/rules/format-evolution.md:39`）；「**实验必须对应决策**：表里至少一行关系是支撑、推翻或备料」（`.claude/rules/format-evolution.md:38`）。

### 8.2 D8（核心索引结构） 已定项 16：E207、E209 两页都写「备料」，不升

- 条款里被这两个实验碰到的那一句：「准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径。」（`.claude/kb/decisions/08-核心索引结构.md:517`）。它的依据是用户定案、不带实验：「无实验：用户定案，原话在变更史」一类（`.claude/kb/decisions/08-核心索引结构.md:533`；同条 521 行另写「无实验：数都在三方腿的副本原型上量」）。
- 不写「支撑」：支撑要读到这一句成立。E207 读到的是：`row_one_holds=false`（第三节 3.1，5 个 `steady` 格不是「不随」）、`user_units_equal_declared` 档没判（实验页 394 行）、`first` 的 `all` 格随总量涨（F2）、B8 是模型不是实现（登记 131 行）。
- 不写「推翻」：推翻要读到这一句不成立、且是条款本身的问题。E207 的 F2 与 E203 的 B1（U 轴 0.95–1.01）读到的是候选设计与今天的实现离这一句有多远，这一句是用户 2026-10-31 定的要求；E203 岔路里「B4 改射程」还等用户（`.claude/kb/experiments/203-准入之前扫类的步代价.md:56`）。若写推翻，D8 已定项 16 的依据要改成引回 E207，等于替用户在 B8 与 B4 之间选了。
- 不写「不影响」：两页都在量这一句。
- E209 同样写「备料」：它解释了 B8 模型里唯一随总量涨的那一类发布（尖峰）的载体，只在小几何，不动这一句。E209 页 853 行末句「这一格该不该升成支撑交主 agent」：不升。
- 回看理由建议写的要点（不是替主 agent 落笔）：B8 那一半在 `one_user_unit` 档既没站住也没翻（读不稳，用户 2026-11-13 定那一格留开）；`user_units_equal_declared` 档没判且预期 `C1` 不红；这一句对今天的实现由这几处整份拷贝罩着（第六节）；条文归用户定。

### 8.3 D28（挂载期承诺量） 已定项 1：两页都写「不影响」，或不加这一行

- 条款里相关的句子：「发布路径在 `transaction::prepare_the_version_publish` 里判：算定这次发布的形状之后、读盘核与动分配器之前」（`.claude/kb/decisions/28-挂载期承诺量.md:35`）；「需求 = 这次发布新写的全部槽（普通分配加这次写出的固定点，不加这次换下的槽）；判不判照旧：没有普通分配的发布（空发布、写行、暖机、抬 F）不判」（`.claude/kb/decisions/28-挂载期承诺量.md:38`）。代码注释同向：「一个普通分配都没有的发布（空发布、写行、暖机、抬 F）不判，它们的空间在保留池与切换预留里」（`crates/singlefs-core/src/transaction.rs:13490`）。
- E207：数的是准入之前的步（产生需求的那一段的代价），不测式子；该分项接线上的失败事件，固定点走不完，在 E207 没发生：「14 份 `item_checks` 判决行 `did_not_settle=0`」（`.claude/kb/experiments/207-严格判据下的扫类步数.md:16`）。E206 写「备料」是因为 P1 走满 64 轮交回 `AllocationRecordTreeRewriteSetDidNotSettle`，发生在两道准入之间（`.claude/kb/experiments/206-发布固定点轮数的代价.md:481`），E207 没有这件事。写「不影响」。
- E209：行四把尖峰载体定在挂载的写行一步（`crates/singlefs-core/src/mount.rs:5841`），而这个分项把写行列入不判式子的发布，空间归保留池与切换预留。这不改这个分项的句子，所以对已定项 1「不影响」。但写行那一步写出的固定点节点数（小几何 51–103，E207 里最大的尖峰是 433 个节点，推的）是否在保留池的上界之内，要看已定项 4：「拿得准的上界只有整棵树」（`.claude/kb/decisions/28-挂载期承诺量.md:119`），尖峰 433 个节点对 551 个节点的树（登记 163 行，`previous_nodes=551`）在整棵树的上界之内；这条我没核到代码，标未核，也不是这一题的范围。
- 两页正文都没写 D28：`grep -c '准入'` E209 页是 0，E207 页 3 处讲「准入之前」，没点名 D28；按 `.claude/rules/format-evolution.md:37` 「实验正文…提到的每条决策都要有一行」，没提到就不要求。加不加「不影响」行交主 agent。

### 8.4 E203 选臂（第一轮「交用户」第三行）：证据上动不了，前置已满足

- 第一轮原行：「改判据不等于选 B8；前两项定了才选得动」「等前两项」（`research/prompts/e203-b8-criterion-r1-main-verification.md:42`）。
- 前两项（量的，`records/2026-10-13-里程碑四预开发.md`）：判据新形态是否采用，用户 2026-11-09 定了（「用户弹窗定三项：① B8 判据改成按这次改动」（`records/2026-10-13-里程碑四预开发.md:1586`））；整份覆盖写走单次发布是否违反已定项 5，用户定登欠账（「整份覆盖写用户定登欠账」（`records/2026-10-13-里程碑四预开发.md:1587`））。前置在字面上满足。
- 但选臂靠岔路单 9、10 行够判，够判条件没满足：行 9 的够判条件要求「含「改动至少 1000 个单元、改动远小于总量」的新点；每种发布两种挂载状态分开判；阳性对照（每次发布全表扫描、B8 最大一行换回线性）都判红」（`research/prompts/e203-b8-criterion-r1-forks.md:9`）：`user_units_equal_declared` 档阳性对照只红 12/54，作废（第三节 3.3），第四段最大点本次未跑。行 10 同条件，且复量格 `unstable`。用户 2026-11-09 另有「今天不选臂，查清之前 B1 照旧」（`records/2026-10-13-里程碑四预开发.md:1582`）。
- 结论：选臂不能从 E207 的读数里导出。用户可以在没有够判的情况下选，那是设计决定：带着这几句——B8 站住没证明；第一次发布随总量涨（F2）；条款罩着今天实现里的整份拷贝，B8 那一列「已换成按差量」的前提没实现也没实验（第六节）。
- 岔路单三行的状态（我的读法，交主 agent 定）：9 行开着（`user_units_equal_declared` 档没判、5 个 `steady` 格不是「不随」）；10 行开着（用户 2026-11-13 已定那一格留开）；11 行开着，还差大几何的载体与一条能预测的谓词（第四节 4.5）。三行都没到「够判」。

### 8.5 第一轮没采的与只记延后的：理由此刻还成不成立

| 项（判决行） | 第一轮的理由 | 此刻 |
|---|---|---|
| 题面写法（19） | α(步 ÷ 分母) = α(步) − α(分母)，没有一格比原判据严 | 成立（代数）。E207 没有读数反对，也没有读数支持（该档作废） |
| 分母取各轮释放之和（20） | 比值恒为 4 × lg(R_a)，步差 10 倍的臂比值反而低 | 成立；E207 不用这个分母 |
| 分母内生（21）与分母取第一轮释放加重写角色（23） | 分母由被判实现算出 | 成立；E207 外生声明全过（实验页 16 行） |
| x 取旋钮（24） | 旋钮 x 下阳性对照不红 | 成立（间接）：x 取记录数时 `one_user_unit` 档 42/42 红；E207 没重跑旋钮 x 那一臂 |
| 第三条路（30） | 条款字面没列整份覆盖写、代价没量 | 成立；用户已登欠账；E207 不量分批 |
| 延后：尖峰为什么只出现一次（46 行） | 随判据定案那一批的实验登记 | 部分失效：E209 已量，载体到小几何，剩大几何与谓词（第四节） |
| 延后：新登记由实验设计员写（47 行） | 用户定了再派 | 失效：E207、E209 已写已跑 |

什么现象会推翻这一节：用户 2026-11-09 之后又有改动前两项的定案（我只查了 `records/2026-10-13-里程碑四预开发.md` 里的几行）；或 B8 在 `user_units_equal_declared` 档换对照臂后有了红的读数。

## 九、要主 agent 跑的命令

两个脚本都只读文件、不起项目代码，不是模型，所以没有再包一层「干净目录重跑」的外壳；输入都是 `research/results/` 下已有的产物。脚本与它们的 sha256（我在草稿目录里写的，没入仓）：

```text
093dd4a37e517962625c4324a4f931326357d70b86633f788ee89312e6ac52a9  /home/fy5090/.local/share/singlefs/work/e203-b8-criterion-r2-sonnet/floor_only_flag_check.sh
a1e5ebb986eee76e512d66ce0305fe2bede919e4b995a42d77b1e9c85b5cccaf  /home/fy5090/.local/share/singlefs/work/e203-b8-criterion-r2-sonnet/window_and_time_arithmetic.sh
```

一、命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/.local/share/singlefs/work/e203-b8-criterion-r2-sonnet/floor_only_flag_check.sh /home/fy5090/code/singlefs/research/results/e209-spike-cause-report-third-2026-11-13.out`
- 在哪起：仓根。
- 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 末行应是 `operations=742 registered_flagged=52 floor_only_flagged=54 predicted_narrow=54 registered_vs_predicted_mismatches=2 floor_only_vs_predicted_mismatches=0`，上面列两行 `registered_vs_predicted_mismatch:`（`old_mount_old` 与 `files_75_units_100__head__first_mount_first`）。对得上，第七节 7.1 的「算的」才进判决。
- 我读到的数现在是「算的，待主 agent 跑」。

二、命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/.local/share/singlefs/work/e203-b8-criterion-r2-sonnet/window_and_time_arithmetic.sh`
- 在哪起：仓根。
- 看哪几行：`exit=0`；log 应是六行：`common_window_two_points_alpha=0.3327`、`own_window_two_points_alpha=0.5024`、`pure_quadratic_total_seconds_at_4000_publishes=40486`、`pure_quadratic_publishes_that_fit_in_14400_seconds=2386`、`power_2.48_total_seconds_at_4000_publishes=55796`、`middle_point_times_100=43458`。前两行是产物里的数的算术；后四行是推的（第五节 5.2），进判决只能写「推的」。

## 十、没做什么

- 没往统一任务服务交任何任务，没起 `task-server/task` 的 `submit`、`wait`、`cancel`、`clean`；没起装置、没编译、没跑门禁。第九节两条是判决要引的数的复跑命令，主 agent 跑之前我读到的数一律是「算的」或「推的」。
- 没读禁读清单里的任何文件；没读执行员草稿目录里的 result.md。
- 没重判复量那一格的去留（用户 2026-11-13 已定，`records/2026-10-13-里程碑四预开发.md:1688`）；第三节只用它回答「B8 那一行守得住哪一半」。
- 没拿同一个点的 plain 构建对拍：这几处整份拷贝各占多少发布用时没量，第六节「第四段跑不出来与这几处同一位置」是推的。
- 没核已定项 4 的保留池上界对写行那一步的固定点写入量是否够（8.3，标未核）；没核第三条路（整份覆盖写分批）的发布语义。
- 没为 Q10 的 {低端, 中点} 十倍报数写命令：低端点的产物不在 `research/results/`（只有中点 `research/results/e207-strict-scan-steps-files_100_units_1000_unit_area_2_pow_21-head-2026-11-13.out`），取法（判定库按键复用）我没核，不写成确定的命令。
- E209 只核了三处口径涉及的行与五行的汇总行，没有逐行重核行一至行五的全部读数；E207 只核了本报告引到的读数行。
- 没改任何 kb、登记、判决与岔路单；提到的「回看理由要点」「加一句登记」都交主 agent 落笔。
- 追加的「中等点」`files_316_units_1000_unit_area_2_pow_21` 是我举的例子，点名与用时（4340 秒）都是推的，没在装置的点表里核。
