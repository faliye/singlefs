# m4-gpugc-design-r3 攻方腿（Opus）报告（2026-10-24）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 -->

立场：假设前两轮站住的形态、第二轮判决与正文第三节的读法都是错的，拿 E185 第 4 次重跑的三份产物去打。同一轮辩方腿的报告在禁读清单里，我没读。开工快照 `research/prompts/m4-gpugc-design-r3-snapshot/sha256sums.txt` 23 行与 `kb-sha256.txt` 4 行，`sha256sum -c` 全 OK（腿跑着时快照里的文件没变）。下文「卡 0」是 RTX 5090，「卡 1」是 RTX 5060 Ti；候选 A、B、C、D 在产物里是 `scan_checksum_only`、`scan_checksum_and_choice`、`every_stage`、`release_check_only`。「复现臂」是控漂移段不绑核、照第 3 次重跑原样量卡 0 的那一臂，「受控臂」是绑一个核、两张卡交替的那一臂。凡写「推的」都没量；凡写「量的」都贴了命令与输出。这一轮新冒出的改法都是零轮，只在我的模型上量过、被攻过零轮，汇总在第五节交用户表。

## 复跑

模型是五个只读产物的 python（`hanging_checks.py` 是续做这一条腿时加的），不编 Rust、不起 GPU。这一轮没有冻结副本，`--frozen` 给仓根即可（脚本只读 `$REPO_ROOT/research/results/` 下的产物与第一轮攻方模型的 `out/g6.out`，先核其中五份的 sha256）。

```bash
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m4-gpugc-design-r3-opus-model/tools/rerun.sh --repo <仓根> --frozen <仓根> --work <新的空目录>
```

续做时连跑三次（`--work` 是续做草稿目录里的 `rerun1`、`rerun2`、`rerun3`，三次都退 0，三份日志的 `rerun_compare` 行全是 `identical=true`），第三次的原样输出：

```text
name=positive_control item=controlled_arm_equals_the_product_summaries summaries=576 mismatches=0 holds=true
name=positive_control item=controlled_arm_equals_the_product_summaries summaries=576 mismatches=66 holds=false
name=rerun_compare model=crt_shift identical=true
name=rerun_compare model=round_spread identical=true
name=rerun_compare model=rejudge_arms identical=true
name=rerun_compare model=numbers_check identical=true
name=rerun_compare model=hanging_checks identical=true
```

头两行是 `rejudge_arms.py` 的阳性对照与它的「会红」：我自己写的重判（不 import 仓里的重判脚本）在受控臂上逐格复现重判产物的 576 行 `baseline_rejudge_summary`（`crt_source=r4_drift_control_controlled` 那一半，0 处不同）；把门槛读高 1%（`plant`）就有 66 行对不上。之后拿同一套判法换成复现臂的 c_rt 去判，见第一节。

模型目录每个文件的 sha256：

```text
d29cf56e08d846d2dfb94d25495b46e7db04b6aa6fb1de7e4e168b84ea8fec5b  m4-gpugc-design-r3-opus-model/crt_shift.py
f342bdeb9d8106cd4c1b503b536916f03c597a367f6c6d081eaff8ce4b0135f3  m4-gpugc-design-r3-opus-model/hanging_checks.py
d3d79da479066a417bd61875b07af2d10ba7f83c25dcddad21d15bd48130ef18  m4-gpugc-design-r3-opus-model/numbers_check.py
85354e913b282a3085f4f3e0abace02ea82382a02fedfe3a1546199ba2810299  m4-gpugc-design-r3-opus-model/out/crt_shift.out
dae2cde7c144c96a6f49030f1799e1c12da181b6f95e2b7ad7075221debf8281  m4-gpugc-design-r3-opus-model/out/hanging_checks.out
cedfac204f0f35cce0171f74bb7db07b054576c2db795ff27b5cd1db7de49306  m4-gpugc-design-r3-opus-model/out/numbers_check.out
216a734a1d66dddd23ff28df5ade462bfd80b7bf4e15a51b3cd935eefcc7be39  m4-gpugc-design-r3-opus-model/out/rejudge_arms.out
cc8271513d5dfb0e0a9d67ba34010ee78cf660e1975a115cc9411cef77df23fc  m4-gpugc-design-r3-opus-model/out/round_spread.out
bf26f07fc94b7497d300e67fd8317e4984b5149c5c89b82ee8559bd6dac62a83  m4-gpugc-design-r3-opus-model/rejudge_arms.py
3fa8babf190c5a223a490d9b3af99e038b6850a4d40a8c6d0e8488b3e3274a3f  m4-gpugc-design-r3-opus-model/round_spread.py
0bb72452e7cccfe17242ed6f8ba4aae1deac31db5dd1b9c6dd6345d6c5fe5983  m4-gpugc-design-r3-opus-model/tools/rerun.sh
```

（路径相对 `research/prompts/`。）读的产物与开工快照登记的 sha256 相同：r3 第一段 `840b6835…`、控漂移段 `917cb645…`、写法段 `5b00c937…`、重判 `f12a985d…`。

## 各格判定一览

| 格 | 判定 | 最要紧的一处（一句话） | 证据形态 |
|---|---|---|---|
| 关 1：G1-① 「A、B、D 两张卡都出局」 | **按间隔翻面** | 间隔 50 ms 下第 4 次重跑照旧坐实（A、D 两张卡、两条臂一格不赔都没有）；间隔 0 下卡 0 翻了：今天的写法 A 不赔 44（受控）/ 64（复现），D 50 / 68，B 下界不赔 84 / 92（各 108 格） | 产物现数＋我的重判（量的） |
| 关 1：G1-① 「C 的赢依赖两处慢路径」 | **按间隔翻面** | 两处都换（三路、不计建索引）之后，间隔 50 ms 卡 0 不赔 0/36（受控）、2/36（复现），卡 1 0/36：第二轮攻方「一格不赔都没有」在入库装置上坐实；间隔 0 卡 0 仍有 55/72、67/72 不赔 | 同上 |
| 关 1：G1-③ | 站得住 | 第 4 次重跑卡 0 每次提交挂钟仍约 5.3 ms，代码那几行没变 | 产物＋代码行 |
| 关 1：G2 | 「跨批攒不成」站得住（更硬）；「卡 0 拟合不可信」站得住、理由换了 | 648 行 `merging_shape` 一行跨意图都没有；卡 0 哪一档拟合线性在两次重跑之间对调（第 3 次间隔 0 全不线性，第 4 次两臂间隔 0 全线性、间隔 50 ms 全不线性），没有漂移的复现臂照样不线性，第二轮攻方「不线性跟着漂移走」解释不了第 4 次 | 产物现数 |
| 关 1：G4 ①、G6 | 站得住 | 第 4 次重跑没有碰到这两格的读数 | 代码行 |
| 关 2：数能不能核 | **对不上四处，其余核得出** | ① 第三节「C 卡 0 120 格」里有 12 格是今天的写法（只报、不当第 1 行读数）；② 第三节末段「卡 0 仍多半不赔」把两种间隔混着数，间隔 50 ms 的读数恰好坐实第二轮攻方；③ 正文 G1 把「全部显卡」写成第 4 条，它在第 2 条；④ 第一轮判决 g6 真盘臂一批写成 23.728–123.369 ms，下端实为 21.671。另：卡 0 的 c_rt 水平第 4 次比第 3 次低到 0.38 倍（108 个键每一个都低），控漂移段的判定只比一趟之内的轮次，看不见这一跳 | 命令＋原样输出 |
| G3 多卡分活 | **打中（最终）** | 受控臂两张卡同一天同一跑法，轮转让 C 在卡 0 不赔的 101 格里 13 格赔、9 格不稳定（今天的写法）；两处都换之后 55 格里 21 格赔 | 我的重判（量的） |
| G4 ② | **打中（最终）** | 条款「CPU 只改位图，不再抽样复核」与发布路径逐条核、留存条目读盘核校验和（`transaction.rs` 第 10133 行文档注释）对不上；第 4 次重跑不改这一格 | 代码行 |
| G4 的 N | N 取几：构造不出（最终）；计数住哪：第一轮那一序列仍可达，这一轮另找到「一轮」随候选不同（推的） | 第 4 次重跑两份核的格有效 1080 格（另 11 格作废）、计时提交里 7065000 次单元比对零不确定，但输入是同一个单元重复 512 次，不加宽数据相关的上界；计数住内存那一条（第一轮）没有被推翻 | 产物现数＋条款字面 |
| G5 控制接口 | **打中，但只在 G8 选「挪出」时可达（最终）** | 今天整理在 `publish_user_change` 里、会话 `&mut self` 独占，切换请求落不进一批中间；挪出之后第二轮那一序列成立 | 代码行＋条款字面 |
| G6 改法「挪出调用线程」 | **够不着「等多久」那一格（最终）** | 挪出之后 fsync 至多等一批：第 4 次重跑写法段一批主机 CPU 12.2–13.9 ms（P = 1024）、96.4–112.7 ms（P = 8192） | 产物现数 |
| G7 GPU 代码住哪 | **打中（最终），换了形态** | 第二轮的「只能用同一份 naga」过头（Cargo 允许两个不兼容大版本并存）；站得住的是：驱动编译器换不掉，而且门禁 ① 只拿 `singlefs-checker` 与 `singlefs-core` 两个闭包求交，checker 档（它本来就依赖 core）与产品 GPU 包共用什么都不会红 | 门禁文件头＋Cargo 文件 |
| G8 GC 挪不挪出 | **两边都有代价（最终），交用户** | 第 4 次重跑没给新代价数；「不挪出」另撞 D24 已定项 2 定案句「不满足就不许做」 | 条款字面 |
| 关 4：岔路单漏的 | **漏 9 行** | c_rt 取哪一次哪种布置、按哪种间隔读、CPU 模式取哪种写法（硬件 CRC、建不建索引）、多卡怎么分活、控制接口与出错次序、产品 GPU 包与 checker 档共用边界、第 8 条与逐条核、主机 CPU 用哪种钟、带快照的池 | 表行 |

「打中」只是线索，由主 agent 现查；每处打中都写了四句。

## 一、第二轮判决打穿的五处，在第 4 次重跑读数下（关的第 1 条）

### G1-①　两半都「按间隔翻面」，翻面的那一档是间隔 0

第二轮判决 G1 那一格的原话是「候选 A、B、D 在登记读法下两张卡都出局；C 的赢依赖 CPU 模式今天的两处慢路径，那两处换掉之后符号没量过」。第 4 次重跑把「那两处换掉之后」量了，第三节报的是间隔 0 与 50 ms 合起来的格数。按间隔拆开（门槛来源「这一趟产物直接算」、`machine_scaled`、线程钟、只数 GPU 模式的形态，不含今天的写法；受控臂的数与重判产物逐格相同，复现臂是我拿同一套门槛与判法换 c_rt 判的）：

```text
$ grep 'name=gap_summary' out/rejudge_arms.out | grep 'threshold_source=direct_from_the_r4_product' | grep 'threshold_kind=machine_scaled clock=thread forms=gpu' | grep -E 'writing=(today|both_three_way_without_build) ' | grep -E 'index=0|candidate=every_stage' | sed -E 's/name=gap_summary //; s/threshold_source=direct_from_the_r4_product //; s/threshold_kind=machine_scaled clock=thread forms=gpu //'
arm=controlled writing=both_three_way_without_build candidate=every_stage index=0 gap_milliseconds=0 lines=72 not_losing=55 not_losing_lower_bound=0 losing=3 unstable=14
arm=controlled writing=both_three_way_without_build candidate=every_stage index=0 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=0 losing=2 unstable=34
arm=controlled writing=both_three_way_without_build candidate=every_stage index=1 gap_milliseconds=0 lines=72 not_losing=10 not_losing_lower_bound=0 losing=55 unstable=7
arm=controlled writing=both_three_way_without_build candidate=every_stage index=1 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=0 losing=36 unstable=0
arm=controlled writing=both_three_way_without_build candidate=release_check_only index=0 gap_milliseconds=0 lines=72 not_losing=0 not_losing_lower_bound=0 losing=72 unstable=0
arm=controlled writing=both_three_way_without_build candidate=release_check_only index=0 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=0 losing=36 unstable=0
arm=controlled writing=both_three_way_without_build candidate=scan_checksum_and_choice index=0 gap_milliseconds=0 lines=72 not_losing=0 not_losing_lower_bound=48 losing=9 unstable=15
arm=controlled writing=both_three_way_without_build candidate=scan_checksum_and_choice index=0 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=0 losing=5 unstable=31
arm=controlled writing=both_three_way_without_build candidate=scan_checksum_only index=0 gap_milliseconds=0 lines=72 not_losing=0 not_losing_lower_bound=0 losing=72 unstable=0
arm=controlled writing=both_three_way_without_build candidate=scan_checksum_only index=0 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=0 losing=36 unstable=0
arm=controlled writing=today candidate=every_stage index=0 gap_milliseconds=0 lines=72 not_losing=72 not_losing_lower_bound=0 losing=0 unstable=0
arm=controlled writing=today candidate=every_stage index=0 gap_milliseconds=50 lines=36 not_losing=29 not_losing_lower_bound=0 losing=0 unstable=7
arm=controlled writing=today candidate=every_stage index=1 gap_milliseconds=0 lines=72 not_losing=51 not_losing_lower_bound=0 losing=13 unstable=8
arm=controlled writing=today candidate=every_stage index=1 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=0 losing=30 unstable=6
arm=controlled writing=today candidate=release_check_only index=0 gap_milliseconds=0 lines=72 not_losing=50 not_losing_lower_bound=0 losing=4 unstable=18
arm=controlled writing=today candidate=release_check_only index=0 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=0 losing=4 unstable=32
arm=controlled writing=today candidate=scan_checksum_and_choice index=0 gap_milliseconds=0 lines=72 not_losing=0 not_losing_lower_bound=70 losing=0 unstable=2
arm=controlled writing=today candidate=scan_checksum_and_choice index=0 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=14 losing=0 unstable=22
arm=controlled writing=today candidate=scan_checksum_only index=0 gap_milliseconds=0 lines=72 not_losing=44 not_losing_lower_bound=0 losing=6 unstable=22
arm=controlled writing=today candidate=scan_checksum_only index=0 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=0 losing=8 unstable=28
arm=reproduction writing=both_three_way_without_build candidate=every_stage index=0 gap_milliseconds=0 lines=72 not_losing=67 not_losing_lower_bound=0 losing=4 unstable=1
arm=reproduction writing=both_three_way_without_build candidate=every_stage index=0 gap_milliseconds=50 lines=36 not_losing=2 not_losing_lower_bound=0 losing=16 unstable=18
arm=reproduction writing=both_three_way_without_build candidate=release_check_only index=0 gap_milliseconds=0 lines=72 not_losing=0 not_losing_lower_bound=0 losing=72 unstable=0
arm=reproduction writing=both_three_way_without_build candidate=release_check_only index=0 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=0 losing=36 unstable=0
arm=reproduction writing=both_three_way_without_build candidate=scan_checksum_and_choice index=0 gap_milliseconds=0 lines=72 not_losing=0 not_losing_lower_bound=62 losing=7 unstable=3
arm=reproduction writing=both_three_way_without_build candidate=scan_checksum_and_choice index=0 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=0 losing=20 unstable=16
arm=reproduction writing=both_three_way_without_build candidate=scan_checksum_only index=0 gap_milliseconds=0 lines=72 not_losing=0 not_losing_lower_bound=0 losing=72 unstable=0
arm=reproduction writing=both_three_way_without_build candidate=scan_checksum_only index=0 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=0 losing=36 unstable=0
arm=reproduction writing=today candidate=every_stage index=0 gap_milliseconds=0 lines=72 not_losing=72 not_losing_lower_bound=0 losing=0 unstable=0
arm=reproduction writing=today candidate=every_stage index=0 gap_milliseconds=50 lines=36 not_losing=29 not_losing_lower_bound=0 losing=0 unstable=7
arm=reproduction writing=today candidate=release_check_only index=0 gap_milliseconds=0 lines=72 not_losing=68 not_losing_lower_bound=0 losing=0 unstable=4
arm=reproduction writing=today candidate=release_check_only index=0 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=0 losing=12 unstable=24
arm=reproduction writing=today candidate=scan_checksum_and_choice index=0 gap_milliseconds=0 lines=72 not_losing=0 not_losing_lower_bound=72 losing=0 unstable=0
arm=reproduction writing=today candidate=scan_checksum_and_choice index=0 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=20 losing=0 unstable=16
arm=reproduction writing=today candidate=scan_checksum_only index=0 gap_milliseconds=0 lines=72 not_losing=64 not_losing_lower_bound=0 losing=2 unstable=6
arm=reproduction writing=today candidate=scan_checksum_only index=0 gap_milliseconds=50 lines=36 not_losing=0 not_losing_lower_bound=0 losing=20 unstable=16
```

（卡 1 上 A、D 在每种写法下 108 格全赔，重判产物第 100773 行一类的汇总行就有，这里不重列。）

读法：
- **「A、B、D 两张卡都出局」**：间隔 50 ms 下 A、D 在卡 0 两条臂、两种写法一格不赔都没有，这一半坐实；B 在今天的写法下卡 0 还有 14（受控）/ 20（复现）格「下界不赔」，换成三路 CRC 之后 0 格。间隔 0 下卡 0 翻了：今天的写法 A 不赔 44 / 64、D 50 / 68、B 下界不赔 70 / 72（各 72 格）。翻面的原因不在写法，在 c_rt：卡 0 第 4 次重跑的每次提交主机 CPU 只有第 3 次的 0.38 倍左右（下一段）。
- **「C 的赢依赖今天的两处慢路径」**：间隔 50 ms 下坐实——两处都换（`both_three_way_without_build`），卡 0 不赔 0/36（受控）、2/36（复现），卡 1 0/36；今天的写法下卡 0 是 29/36。第二轮攻方在它模型上推的「两处都换掉之后，间隔 50 ms 下 C 一格不赔都没有」，入库装置受控臂一格不差地给出了 0。间隔 0 下翻了：卡 0 两处都换仍有 55/72、67/72 不赔。
- 所以第二轮那一句要站住，得先定「按哪种间隔读」。第二轮攻方第二节第 6 条主张产品两批之间至少隔一批的主机 CPU，第 4 次重跑写法段一批（`kind` 不是 `issue` 的调用）是 12.2–13.9 ms（P = 1024）、96.4–112.7 ms（P = 8192）（第三节 G6 那一段贴了命令）；照这个读，间隔 0 不是产品形态，G1-① 两半都站住。这一主张第二轮判决没判，岔路单也没有这一行（第四节 M2）。

卡 0 的 c_rt 水平，两次重跑同一键逐个比（线程钟、候选 A 那一行的 c_rt，候选 A、C 的 c_rt 逐字相同）：

```text
$ cat out/crt_shift.out
name=crt_shift arm=controlled index=0 clock=process forms=gpu keys=76 median_r4_over_r3=0.367 min=0.254 max=0.685 below_one=76
name=crt_shift arm=controlled index=0 clock=thread forms=gpu keys=108 median_r4_over_r3=0.392 min=0.232 max=0.676 below_one=108
name=crt_shift arm=controlled index=1 clock=process forms=gpu keys=12 median_r4_over_r3=0.937 min=0.734 max=1.010 below_one=10
name=crt_shift arm=controlled index=1 clock=thread forms=gpu keys=108 median_r4_over_r3=0.950 min=0.721 max=1.247 below_one=82
name=crt_shift arm=reproduction index=0 clock=process forms=gpu keys=82 median_r4_over_r3=0.338 min=0.245 max=0.720 below_one=82
name=crt_shift arm=reproduction index=0 clock=thread forms=gpu keys=108 median_r4_over_r3=0.377 min=0.225 max=0.676 below_one=108
```

一个键的两行：`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out:975`：「E7RESULT name=host_cpu_per_submission_point fraction=0.25 pool_data_units=1024 batch_units=4 candidate=scan_checksum_only index=0 form=shared wait=blocking gap_milliseconds=0 clock=thread threshold_kind=machine_scaled threshold_milliseconds=0.082553 submission_units=8,8,4 host_cpu_per_submission_milliseconds_per_round=0.197957,0.221157,0.228220,0.110429,0.058907 judgement_per_round=losing,losing,losing,losing,not_losing judgement=unstable floor_host_cpu_per_submission_milliseconds_per_round=0.154266,0.172242,0.160558,0.085450,0.037638 floor_judgement_per_round=losing,losing,losing,losing,not_losing floor_judgement=unstable」

`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:1395`：「E7RESULT name=drift_control_point arm=reproduction fraction=0.25 pool_data_units=1024 batch_units=4 candidate=scan_checksum_only index=0 form=shared wait=blocking gap_milliseconds=0 clock=thread threshold_kind=machine_scaled threshold_milliseconds=0.069208 submission_units=8,8,4 host_cpu_per_submission_milliseconds_per_round=0.047215,0.050992,0.052688,0.049617,0.046762 judgement_per_round=not_losing,not_losing,not_losing,not_losing,not_losing judgement=not_losing floor_host_cpu_per_submission_milliseconds_per_round=0.034298,0.034795,0.038040,0.034667,0.035568 floor_judgement_per_round=not_losing,not_losing,not_losing,not_losing,not_losing floor_judgement=not_losing」

同一天、同一装置、同一卡、不绑核：第 3 次的第 5 轮 0.059 ms 已经是它五轮里最低，第 4 次五轮全在 0.047–0.053 ms。卡 1 只差到 0.95 倍。第 3 次重跑开跑时卡 0 上有别的计算进程（`.err` 第 4–7 行「卡 0 上有别的计算进程 [76757]，等它」），两次的清场都是「跟着别人那一趟用」（`research/results/e185-r4-quiesce-host-cpu-drift-control-2026-10-24.txt` 第 1 行「GPU_QUIESCE step=quiesce result=joined-and-read-back holders=1」）；我这一次 `ps` 看到别的会话在跑的崩溃放量命令里 `GPU_CARDS_BY_PRIORITY` 把本机卡 0 排第一。第 3 次卡 0 读数偏高、往下走，可能是同时段别人的 GPU 活压在卡 0 上（推的，产物里没有逐格的「卡上别的进程」读数可核）。要紧的是：第 3 次那一趟每格前后的校准比、开格前的「卡上别的进程」检查全过了，作废谓词没有拦下一个 2.6 倍的水平差。

四句：
1. 分不分辨臂：分辨。间隔 0 的翻面只动 A、B、D 在卡 0 上的判定与 C 在两处换掉之后的判定；间隔 50 ms 下不动。
2. 判别子看不看得到：产品挂载时看不到「这一刻卡 0 的 c_rt 是 0.05 ms 还是 0.13 ms」（产品里不量 c_rt），装置也看不到（作废谓词没拦）——这一半是「判别子观测不到」。间隔看得到（产品两批之间隔多少由一批的 CPU 定）。
3. 满足的分句：D26 已定项 13 第 7 条（`.claude/kb/decisions/26-后台整理与放置回收.md` 第 251 行，指路不抄）的收益式子里减号后那一项（GPU 模式下的 CPU 时间）。
4. 跑前条款的改法还中不中：第 4 次重跑登记第一节第 52 行写「第 1 行读 r3 第一段那一端（问题单原文）；另一端并列报，两端判定不同的记「随 c_rt 来源翻面」」，照这一句，间隔 0 下卡 0 那些格本该记「随 c_rt 来源翻面」、不当已判；正文第三节改读受控臂一端，登记里没有这一选法。

### G1-③　站得住

今天整理在用户改动的发布函数里、返回之前跑至多一批：`crates/singlefs-core/src/mounted_session.rs:1935`：「CompactionRunReach::AtMostOneBatchAfterAUserChange,」，入口 `crates/singlefs-core/src/mounted_session.rs:1849`：「pub fn publish_user_change<Device: BlockDevice>(」（开工快照里这份文件的 sha256 与现在相同）。第 4 次重跑复现臂卡 0、共用输入、至多 24 个单元的一次提交挂钟逐格中位 5.30–6.59 ms（180 格）：

```text
$ grep 'name=submission_cell arm=reproduction index=0 form=shared' research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out | awk '{for(i=1;i<=NF;i++){split($i,a,"="); v[a[1]]=a[2]} if(v["units"]<=24 && v["invalid_reason"]=="none") {w=v["wall_median_nanoseconds"]/1e6; if(min==""||w<min)min=w; if(w>max)max=w; n++}} END{printf "cells=%d wall_ms_min=%.2f max=%.2f\n", n, min, max}'
cells=180 wall_ms_min=5.30 max=6.59
```

c_rt 降了，挂钟没降（等 GPU 的那一段在睡）。第 4 次重跑没有让这一格翻面的读数。

### G2　「跨批攒不成」更硬；「卡 0 的拟合不可信」站得住、理由换了

```text
$ grep 'name=merging_shape' research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out | awk '{for(i=1;i<=NF;i++){split($i,a,"="); v[a[1]]=a[2]} print v["arm"], v["index"], v["gap_milliseconds"], v["category"]}' | sort | uniq -c
    144 controlled 0 0 one_submission_per_batch
     58 controlled 0 50 one_submission_per_batch
     14 controlled 0 50 unstable
    104 controlled 1 0 one_submission_per_batch
     26 controlled 1 0 one_submission_per_intent
     14 controlled 1 0 unstable
      2 controlled 1 50 fit_not_linear
     54 controlled 1 50 one_submission_per_intent
     16 controlled 1 50 unstable
    144 reproduction 0 0 one_submission_per_batch
     60 reproduction 0 50 one_submission_per_batch
     12 reproduction 0 50 unstable
$ grep 'name=fixed_and_per_byte_fit' research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out | awk '{for(i=1;i<=NF;i++){split($i,a,"="); v[a[1]]=a[2]} print v["arm"], v["index"], v["fit_not_linear"]}' | sort | uniq -c
      6 controlled 0 false
      3 controlled 0 true
      8 controlled 1 false
      1 controlled 1 true
      6 reproduction 0 false
      3 reproduction 0 true
```

- 648 行一行跨意图都没有（第 3 次重跑 432 行里 1 行）。卡 0 在第 4 次的 c_rt 下连「意图内一次交」都用不着，一批一交就不赔或不稳定。「跨批攒不成」站得住。
- 卡 0 的 9 条拟合，两条臂各是间隔 0 的 6 条全线性、间隔 50 ms 的 3 条全不线性。例：`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:2958`：「E7RESULT name=fixed_and_per_byte_fit arm=reproduction index=0 form=shared wait=blocking gap_milliseconds=0 fixed_milliseconds=0.039728 per_mebibyte_milliseconds=0.040741 maximum_relative_residual=0.068267 points=8 fit_not_linear=false fixed_milliseconds_per_round=0.038063,0.039568,0.037871,0.040961,0.038768 per_mebibyte_milliseconds_per_round=0.038836,0.039988,0.049432,0.040652,0.045327」；同一份第 2959 行：「E7RESULT name=fixed_and_per_byte_fit arm=reproduction index=0 form=shared wait=blocking gap_milliseconds=50 fixed_milliseconds=0.223644 per_mebibyte_milliseconds=0.049264 maximum_relative_residual=0.598167 points=8 fit_not_linear=true fixed_milliseconds_per_round=0.284952,0.314035,0.113775,0.211240,0.243865 per_mebibyte_milliseconds_per_round=0.041646,0.058862,0.059117,0.049873,0.041584」。
- 第二轮攻方 G2-① 说卡 0 拟合不线性「跟着卡 0 的漂移走」。第 3 次卡 0 是间隔 0 的 6 条全不线性、间隔 50 ms 的 3 条只 1 条不线性（第二轮攻方报告 G2-① 那张表）；第 4 次两臂都反过来，间隔 0 全线性、间隔 50 ms 全不线性。复现臂没有漂移（轮次比 1.056747），间隔 50 ms 的三条照样不线性，每轮固定部分 0.11–0.31 ms 来回跳：「跟着漂移走」只解释得了第 3 次间隔 0 那 6 条，解释不了第 4 次间隔 50 ms 那 3 条。卡 0 哪一档拟合线性，本身随趟次翻。间隔 50 ms 的固定部分是间隔 0 的约 5 倍（0.2236 对 0.0397）：卡等了 50 ms 再接活，主机这一侧每次多花的那一截不随字节涨，五轮之间也不稳（推的归因：卡在空闲时降了状态；产物的 `clock_reading` 行可以核，我没核）。
- 改判吗：G2 的结论（跨批攒不成）不改。「卡 0 的拟合不可信」站得住，但理由换了：不是漂移污染了一次读数，而是卡 0 哪一档线性在两次重跑之间整个对调，单趟的拟合都不能拿来判攒批（「卡 1 上可用」照旧：卡 1 第 4 次 9 条里 8 条线性）。

### G4 ①、G6　站得住

G4 ①（同输入对拍抓不到输入准备这一类）是测试设计与条款射程的事；G6（「前台有 fsync 在等」今天观测不到，`.claude/kb/decisions/26-后台整理与放置回收.md:250`：「6. **起跑之后的节奏**：GC 占盘写带宽有一个上限（挂载参数）；前台有 fsync 在等时不发下一批。」）是今天整理在用户改动里同步跑造成的。第 4 次重跑三段都没有碰这两格的读数：控漂移段两份核与 CPU 的比对 0 处不同（`gpu_card_verdict` 三行 `result_mismatching_cell_rounds=0`），这是同输入对拍，正是 G4 ① 说抓不到那一类的那种对拍；写法段只量 CPU 侧。两格照旧。

## 二、两轮判决与正文第三节报出的数，逐个从产物现数（关的第 2 条）

### 1. 正文第三节

行数与判定行：

```text
$ grep -c '' research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out research/results/e185-gc-batch-wall-clock-split-cpu-writings-r4-2026-10-24.out research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz
research/results/e185-gc-batch-wall-clock-split-cpu-writings-r4-2026-10-24.out:2362
research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:13808
research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz:108438
$ grep -c 'name=baseline_rejudge_point ' research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz
96696
```

三行 `verdict`（第 13807、2361、108437 行）与正文表里抄的逐字相同（`grep -n 'name=verdict'` 现查）。控漂移段的判定与失败条款：

`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:13794`：「E7RESULT name=drift_judgement quantity=controlled_card_zero_round_ratio arm=controlled index=0 value=1.079304 judgement=drift_gone」；第 13795 行：「E7RESULT name=drift_judgement quantity=reproduction_card_zero_round_ratio arm=reproduction index=0 value=1.056747 judgement=drift_not_reproduced」；第 13796 行：「E7RESULT name=drift_judgement quantity=controlled_card_one_round_ratio arm=controlled index=1 value=1.000256 judgement=drift_gone」；第 13800 行：「E7RESULT name=drift_judgement quantity=controlled_unstable_card_zero_over_card_one arm=controlled index=none value=8.629032 judgement=card_zero_less_stable」；第 13804 行：「E7RESULT name=failure_clause clause=L5 triggered=true」。都对得上。

重判透视（正文第三节第 38–41 行那几个数）对重判产物的 `baseline_rejudge_summary` 行，全对得上，例：`research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz:100789`：「E7RESULT name=baseline_rejudge_summary writing=today threshold_source=direct_from_the_r4_product crt_source=r4_drift_control_controlled candidate=every_stage index=0 threshold_kind=machine_scaled clock=thread lines=120 not_losing=113 not_losing_lower_bound=0 losing=0 unstable=7 rounds_insufficient=0 not_run=0 void_positive_control=0 void_card=0 changed=32 changed_to_losing=0 changed_to_unstable=1 changed_to_not_losing=31 changed_to_other=0」；第 101301 行：「E7RESULT name=baseline_rejudge_summary writing=both_three_way_without_build threshold_source=direct_from_the_r4_product crt_source=r4_drift_control_controlled candidate=every_stage index=0 threshold_kind=machine_scaled clock=thread lines=120 not_losing=64 not_losing_lower_bound=0 losing=6 unstable=50 rounds_insufficient=0 not_run=0 void_positive_control=0 void_card=0 changed=43 changed_to_losing=6 changed_to_unstable=27 changed_to_not_losing=10 changed_to_other=0」；第 100773 行（A 卡 0 不赔 44）：「E7RESULT name=baseline_rejudge_summary writing=today threshold_source=direct_from_the_r4_product crt_source=r4_drift_control_controlled candidate=scan_checksum_only index=0 threshold_kind=machine_scaled clock=thread lines=108 not_losing=44 not_losing_lower_bound=0 losing=14 unstable=50 rounds_insufficient=0 not_run=0 void_positive_control=0 void_card=0 changed=78 changed_to_losing=4 changed_to_unstable=30 changed_to_not_losing=44 changed_to_other=0」；L1 那一行第 108411 行：「E7RESULT name=failure_clause clause=L1 writing=both_three_way_without_build threshold_source=direct_from_the_r4_product crt_source=r4_drift_control_controlled changed_to_losing=78 triggered=true」。

**对不上的三处**：

① **C 的 120 格里有 12 格是今天的写法**。登记把今天的写法（一份核）写成只报、不当第 1 行的读数（第二轮攻方第二节第 1 条在第 3 次重跑上指出过同一件事，这一次正文又照合起来的数报）。拆开：

```text
$ grep 'name=arm_summary' out/rejudge_arms.out | grep 'arm=controlled writing=\(today\|both_three_way_without_build\|both_single_with_build\) threshold_source=direct_from_the_r4_product candidate=every_stage' | grep 'threshold_kind=machine_scaled clock=thread' | sed -E 's/name=arm_summary //; s/threshold_source=direct_from_the_r4_product //; s/threshold_kind=machine_scaled clock=thread //'
arm=controlled writing=both_single_with_build candidate=every_stage index=0 forms=gpu lines=108 not_losing=108 not_losing_lower_bound=0 losing=0 unstable=0
arm=controlled writing=both_single_with_build candidate=every_stage index=0 forms=today lines=12 not_losing=12 not_losing_lower_bound=0 losing=0 unstable=0
arm=controlled writing=both_single_with_build candidate=every_stage index=1 forms=gpu lines=108 not_losing=80 not_losing_lower_bound=0 losing=21 unstable=7
arm=controlled writing=both_single_with_build candidate=every_stage index=1 forms=today lines=12 not_losing=9 not_losing_lower_bound=0 losing=3 unstable=0
arm=controlled writing=both_three_way_without_build candidate=every_stage index=0 forms=gpu lines=108 not_losing=55 not_losing_lower_bound=0 losing=5 unstable=48
arm=controlled writing=both_three_way_without_build candidate=every_stage index=0 forms=today lines=12 not_losing=9 not_losing_lower_bound=0 losing=1 unstable=2
arm=controlled writing=both_three_way_without_build candidate=every_stage index=1 forms=gpu lines=108 not_losing=10 not_losing_lower_bound=0 losing=91 unstable=7
arm=controlled writing=both_three_way_without_build candidate=every_stage index=1 forms=today lines=12 not_losing=0 not_losing_lower_bound=0 losing=12 unstable=0
arm=controlled writing=today candidate=every_stage index=0 forms=gpu lines=108 not_losing=101 not_losing_lower_bound=0 losing=0 unstable=7
arm=controlled writing=today candidate=every_stage index=0 forms=today lines=12 not_losing=12 not_losing_lower_bound=0 losing=0 unstable=0
arm=controlled writing=today candidate=every_stage index=1 forms=gpu lines=108 not_losing=51 not_losing_lower_bound=0 losing=43 unstable=14
arm=controlled writing=today candidate=every_stage index=1 forms=today lines=12 not_losing=8 not_losing_lower_bound=0 losing=4 unstable=0
```

GPU 模式的数：今天的写法卡 0 108 格不赔 101、不稳定 7，卡 1 不赔 51、赔 43、不稳定 14；两处都换卡 0 不赔 55、赔 5、不稳定 48，卡 1 不赔 10、赔 91；`both_single_with_build` 卡 0 108 格全不赔，卡 1 不赔 80。方向不变，正文的格数口径错。

② **正文第三节末段「入库装置上的读数是卡 0 仍多半不赔」**：把间隔 0 与 50 ms 合着数。第二轮攻方那一句是「间隔 50 ms 下」；同一间隔下受控臂卡 0 是 0/36 不赔（第一节那张表第二行）。这一句该写成「间隔 50 ms 下坐实，间隔 0 下卡 0 仍多半不赔」。

③ **正文 G1 写「「全部显卡」（D26 已定项 13 第 4 条）」**：「不给就用本机全部显卡」在第 2 条（`.claude/kb/decisions/26-后台整理与放置回收.md:246`：「2. **用哪几张卡**：挂载参数 `background_gpu_devices` 是一张卡的列表（可以点几张）；不给就用本机全部显卡。」）；第 4 条是配错与 GPU 用不上时拒挂。

**核得出、但判定本身靠不住的两处**（数对，读法要改）：

④ `drift_gone`（受控臂卡 0）只看第 5 轮 ÷ 第 1 轮、只看点行：`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:13776`：「E7RESULT name=round_ratio arm=controlled index=0 scope=point_lines lines=108 lines_expected=108 median_round_two_over_round_one=0.991428 median_round_three_over_round_one=0.777432 median_round_four_over_round_one=1.095680 median_round_five_over_round_one=1.079304 round_ratio_peak=1.095680 rounds_off_by_more_than_ten_percent=1 final_round_ratio=1.079304」；第 13777 行（按格）：「E7RESULT name=round_ratio arm=controlled index=0 scope=cells lines=80 lines_expected=80 median_round_two_over_round_one=0.968613 median_round_three_over_round_one=0.841194 median_round_four_over_round_one=1.175283 median_round_five_over_round_one=1.159713 round_ratio_peak=1.175283 rounds_off_by_more_than_ten_percent=3 final_round_ratio=1.159713」。第 3 轮 ÷ 第 1 轮 0.777 落在「漂移仍在」那一档（< 0.80）里；按格算第 5 轮 1.160 落在「之间」。受控臂卡 0 不是「漂移没了」，是一轮高一轮低。

⑤ D4 的 8.629 是两张卡 `unstable` 的原数相除，两张卡的行数不同（第 13798 行「lines=1704 unstable=535」、第 13799 行「lines=984 unstable=62」），按行数折算是 4.98。更要紧的是 `unstable` 多不多取决于 c_rt 离门槛近不近：卡 1 的 c_rt 远在大多数门槛之上，五轮都赔，自然稳。直接量噪声的是五轮极差：

```text
$ cat out/round_spread.out
name=round_spread run=controlled index=0 lines=108 max_over_min_median=1.930 max_over_min_max=5.214 c_rt_median_of_point_medians_ms=0.0900
name=round_spread run=controlled index=1 lines=108 max_over_min_median=1.136 max_over_min_max=1.531 c_rt_median_of_point_medians_ms=0.3653
name=round_spread run=r3 index=0 lines=108 max_over_min_median=2.615 max_over_min_max=6.227 c_rt_median_of_point_medians_ms=0.2569
name=round_spread run=r3 index=1 lines=108 max_over_min_median=1.332 max_over_min_max=2.265 c_rt_median_of_point_medians_ms=0.3564
name=round_spread run=reproduction index=0 lines=108 max_over_min_median=1.149 max_over_min_max=2.584 c_rt_median_of_point_medians_ms=0.0796
```

不绑核的复现臂卡 0 五轮极差中位 1.149，与受控臂卡 1 的 1.136 一样稳；只有受控臂（绑一个核、与卡 1 交替）的卡 0 是 1.930。「卡 0 更不稳」在这一份产物里是受控臂的布置造的，不是卡 0 的性质（布置里哪一样造的，绑核还是交替，分不出，推的）。而重判与正文第三节恰好取的是受控臂当 c_rt 来源（重判脚本第 394 行注释「复现臂不当 c_rt 来源」；问题单第 2 行状态写「后续用受控臂的读数」）。

### 2. 第二轮判决报出的数

两个模型的输出（`numbers_check.py` 是前一条腿写的，`hanging_checks.py` 是续做时加的，都只读产物；命令在复跑脚本里）：

```text
$ cat out/numbers_check.out
name=check item=r3_point_lines lines=2684 losing=1137 not_losing=398 not_losing_lower_bound=95 unstable=1054
name=check item=c_and_a_same_c_rt groups=672 identical=672
name=check item=r3_round_five_over_round_one index=0 lines=108 median=0.528
name=check item=r3_round_five_over_round_one index=1 lines=108 median=0.958
name=check item=function_split_share_sum fraction=0.25 pool=1024 scope=batches sum=0.888 remainder=0.110173
name=check item=function_split_share_sum fraction=0.25 pool=1024 scope=issue sum=1.628 remainder=-0.630513
name=check item=function_split_share_sum fraction=0.25 pool=8192 scope=batches sum=1.309 remainder=-0.309107
name=check item=function_split_share_sum fraction=0.25 pool=8192 scope=issue sum=2.517 remainder=-1.516256
name=check item=function_split_share_sum fraction=0.75 pool=1024 scope=batches sum=1.167 remainder=-0.169035
name=check item=function_split_share_sum fraction=0.75 pool=1024 scope=issue sum=1.578 remainder=-0.578534
name=check item=function_split_share_sum fraction=0.75 pool=8192 scope=batches sum=1.675 remainder=-0.675966
name=check item=function_split_share_sum fraction=0.75 pool=8192 scope=issue sum=2.183 remainder=-1.182773
name=check item=r3_merging_shape lines=432 merge_across_intents=1
name=check item=g8_foreground_increment card0_one_submission_of_8_units_min=5.50 max=6.54 card1_two_submissions_of_24_units_min=12.03 max=14.76
name=check item=r3_one_submission_wall_median_range min=5.32 max=32.04 cells=720
name=check item=r1_verdict_stage_share line=989 share=0.049067
name=check item=r1_verdict_intercepts line54=0.004492 line136=0.010300
$ grep -E 'r2_check|r1_check' out/hanging_checks.out
name=r2_check item=one_submission_wall cells=84 min=5.46 max=8.05
name=r2_check item=calibration_end_over_start ratio=0.876478 faster_by=0.124
name=r2_check item=r3_fit index=0 gap_milliseconds=0 fit_not_linear=true lines=6
name=r2_check item=r3_fit index=0 gap_milliseconds=50 fit_not_linear=false lines=2
name=r2_check item=r3_fit index=0 gap_milliseconds=50 fit_not_linear=true lines=1
name=r2_check item=r3_fit index=1 gap_milliseconds=0 fit_not_linear=false lines=5
name=r2_check item=r3_fit index=1 gap_milliseconds=0 fit_not_linear=true lines=1
name=r2_check item=r3_fit index=1 gap_milliseconds=50 fit_not_linear=false lines=2
name=r2_check item=r3_fit index=1 gap_milliseconds=50 fit_not_linear=true lines=1
name=r1_check item=g6_batch_wall arm=memory lines=12 wall_ms_median_min=12.508 max=111.090
name=r1_check item=g6_batch_wall arm=real_disk lines=12 wall_ms_median_min=21.671 max=123.369
name=r2_check item=g8_share low=0.0447 high=1.1768 from_increment_ms=5.50,14.76 over_batch_ms=123.135,12.542
```

第一节 ① 那一格的旧数另数了一次（`judgement` 与 `floor_judgement` 两列合起来）：

```text
$ grep 'name=host_cpu_per_submission_point ' research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out | grep -o ' judgement=[a-z_]*\| floor_judgement=[a-z_]*' | awk -F= '{print $2}' | sort | uniq -c
   1879 losing
    956 not_applicable
    731 not_losing
    273 not_losing_lower_bound
   1529 unstable
```

| 第二轮判决里的数 | 从哪数 | 结果 |
|---|---|---|
| 2684 行；losing 1137、not_losing 398、not_losing_lower_bound 95、unstable 1054 | r3 第一段点行的 `judgement` | 对上 |
| 旧写的「5368 行、含 unstable 1529 行」是把 `floor_judgement` 一起数了 | 两列合起来 | 对上：五类合计 5368，unstable 1529 |
| C 与 A 672 组逐字相同 | 同一键下两候选的五轮读数 | 对上 |
| 第 5 轮 ÷ 第 1 轮中位，卡 0 0.528、卡 1 0.958（各 108 格） | 候选 A、`machine_scaled`、线程钟 | 对上 |
| 辩方「终点校准比起点快 12.4%」 | `research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out:8004`：「E7RESULT name=calibration_baseline segment=host_cpu_per_submission position=end runs=21 median_nanoseconds=5493404.0 chase_median_nanoseconds=102919583.000000 anchors_hold=true ratio_of_the_end_to_the_start=0.876478 drifted=true」 | 对上（1 − 0.876478） |
| 第三段第一行 0.888、余项 0.110173；其余七行 1.167–2.517、余项为负 | 八行 `function_split_summary` | 对上 |
| G1「一次提交挂钟 5.46–8.05 ms」 | 共用输入、至多 32 个单元、按（卡、等法、间隔、单元数）取中位，84 格 | 对上；判决没写口径，这是我试出来能给出这两个端点的那一种。放开单元数是 5.32–32.04 ms（720 格） |
| G2「432 行只 1 行跨意图」「卡 0 的 9 条拟合 7 条不线性」 | `merging_shape`、`fixed_and_per_byte_fit` | 对上（间隔 0 的 6 条加间隔 50 ms 的 1 条） |
| G3 卡与卡判定不同 `pairs=1096 flipped=630` | `research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out:4738`：「E7RESULT name=knob_flip_summary quantity=host_cpu_per_submission knob=card clause=two_cards_judge_differently pairs=1096 flipped=630」 | 对上 |
| G4 零次上界 1.428497e-6 | `research/results/e185-gc-batch-wall-clock-split-recompute-disagreement-r3-2026-10-24.out:176`：「E7RESULT name=disagreement_card_summary index=0 natural_unit_times=2097120 first_round_kernel_disagreements=0 upper_bound_at_95_percent_when_zero=1.428497e-6」 | 对上；是每张卡两种输入合起来的上界，单一种输入是 2.856992e-6（同一份第 109 行） |
| G8「前台每次净增 5.50–14.76 ms」 | 卡 0 一次 8 单元提交、卡 1 两次 24 单元提交，各减同字节的 CPU CRC | 对上 |
| G8「占 CPU 模式一批挂钟 4.5%–118%」 | 分母是第一轮攻方模型 g6 的两格：真盘 P = 8192 的 123.135 ms、内存盘 P = 1024 的 12.542 ms | 对上（0.0447、1.1768）。**口径要标**：分子是第 3 次重跑（2026-10-24）的读数，分母是第 1 次重跑（2026-10-22）的一批挂钟，两趟机器状态不同；一个比是两次跑拼的 |
| G3「4 格由不赔翻成赔、23 格翻成不稳定」、G8「200 拍 213 次」 | 第二轮攻方模型的输出，不是产物 | 第二轮判决复跑过（15 行 `identical=true`），我没再跑 |
| 「3.98 倍」「52/54 → 25/54」 | 腿的探针与代理数 | 判决已标不进 kb；入库装置第 4 次重跑量了写法，不再用这两个数 |

### 3. 第一轮判决报出的数

| 第一轮判决里的数 | 从哪数 | 结果 |
|---|---|---|
| 「一批最大 4.9067%」 | `research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out` 第 989 行三项相加 ÷ `call_cpu_nanoseconds` | 对上（0.049067） |
| 每次提交固定 4.492 ms、10.300 ms | `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out` 第 54、136 行 `intercept_seconds` | 对上 |
| g1 门槛 0.216–0.721 ms；g4 0 处与 3 处；g2 至少 8–38 趟、攒 125 批 8–13 趟 | 第一轮攻方模型 `out/g1.out`、`g4.out`、`g2.out` | 对上（`g2.out` 里 `merged_batches=125` 的 6 行是 8、9、10、10、13、13，全部 12 行 8–38） |
| **g6 真盘臂一批 `wall_ms_median=23.728`–`123.369`** | 第一轮攻方模型 `out/g6.out` 真盘臂 12 行 | **对不上**：下端是 21.671（f = 0.25、P = 1024、K = 4），23.728 是 f = 0.75 那一格；f = 0.25、P = 1024 三格 21.671、22.764、23.195 都低于判决写的下端。第一轮攻方报告第 381 行自己写的是「真盘臂 21.671–25.408 ms 与 114.157–123.369 ms」。结论不受影响（G6-② 用的是一批随 P 涨这一点） |
| 正推的 4.59%、0.54 ms、φ ≥ 0.89、70–94 MiB，乙增量 0.349–2.414 ms | 正推腿自己的量 | 判决逐处标了「正推量的」，不是产物数，我没核 |

两轮判决与第三节一共对不上四处：上面第 1 小节的 ①②③ 与上表 g6 那一处。另有一处口径该标而没标（G8 的 4.5%–118% 是两趟拼的比）。

## 三、挂着的六处，各给最终判定（关的第 3 条）

### G3　多卡分活：打中（最终）

第二轮攻方的反例（按窗口轮转让两张卡的平均替了卡 0）用的是第 3 次重跑，两张卡不在同一时段量。第 4 次重跑受控臂正是两张卡同一天、同一跑法、一轮里交替量，拿它重做（模型 `rejudge_arms.py` 的 `round_robin` 行：同一键两张卡逐轮取平均，门槛来源「这一趟产物直接算」、`machine_scaled`、线程钟、只数 GPU 模式的形态；量的是 c_rt，轮转是推的分法）。只列「只用卡 0 不赔」的格：

```text
$ grep 'name=round_robin' out/rejudge_arms.out | grep -E 'candidate=(every_stage|scan_checksum_only|release_check_only) ' | grep -E 'judgement_card0_alone=not_losing ' | sed 's/name=round_robin arm=controlled //'
writing=both_three_way_without_build candidate=every_stage judgement_card0_alone=not_losing judgement_round_robin=losing cells=21
writing=both_three_way_without_build candidate=every_stage judgement_card0_alone=not_losing judgement_round_robin=not_losing cells=23
writing=both_three_way_without_build candidate=every_stage judgement_card0_alone=not_losing judgement_round_robin=unstable cells=11
writing=today candidate=every_stage judgement_card0_alone=not_losing judgement_round_robin=losing cells=13
writing=today candidate=every_stage judgement_card0_alone=not_losing judgement_round_robin=not_losing cells=79
writing=today candidate=every_stage judgement_card0_alone=not_losing judgement_round_robin=unstable cells=9
writing=today candidate=release_check_only judgement_card0_alone=not_losing judgement_round_robin=losing cells=34
writing=today candidate=release_check_only judgement_card0_alone=not_losing judgement_round_robin=not_losing cells=2
writing=today candidate=release_check_only judgement_card0_alone=not_losing judgement_round_robin=unstable cells=14
writing=today candidate=scan_checksum_only judgement_card0_alone=not_losing judgement_round_robin=losing cells=32
writing=today candidate=scan_checksum_only judgement_card0_alone=not_losing judgement_round_robin=unstable cells=12
```

- 今天的写法：C 卡 0 不赔 101 格，轮转后 13 格赔、9 格不稳定；A 卡 0 不赔 44 格，轮转后 32 格赔、12 格不稳定、0 格不赔；D 50 格里 34 格赔、只剩 2 格不赔。两处都换之后 C 55 格里 21 格赔、11 格不稳定。第二轮那一形在同一时段的两张卡上更重了（第二轮是 132 格里 4 格赔）。
- 这一轮新看到的一层：两张卡同时用，布置本身就抬卡 0 的 c_rt、搅它的稳定。受控臂一张卡「后量的那几轮」比「先量的那几轮」高 10–12%：`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:13801`：「E7RESULT name=card_order_effect arm=controlled index=0 first_rounds_median=0.091490 second_rounds_median=0.102226 ratio=1.117348」；第 13802 行：「E7RESULT name=card_order_effect arm=controlled index=1 first_rounds_median=0.349332 second_rounds_median=0.385741 ratio=1.104226」。卡 0 的不稳定行，不绑核、只起卡 0 的复现臂 1564 行里 208 行（第 13797 行「E7RESULT name=unstable_count arm=reproduction index=0 lines=1564 unstable=208」），绑核、两卡交替的受控臂 1704 行里 535 行（第 13798 行「E7RESULT name=unstable_count arm=controlled index=0 lines=1704 unstable=535」）；五轮极差中位 1.149 对 1.930（第二节 ⑤ 那张表）。绑核与交替哪一样造的分不出（推的）。
- 本机第三张卡（AMD 集显）这一趟仍没量。

四句：
1. 分不分辨臂：分辨「只用 c_rt 低的那张卡」与「本机全部显卡、轮转」两种分法；A、C、D 三个候选都被它分开。
2. 判别子看不看得到：产品运行时看不到哪张卡赔（不量 c_rt）；挂载时量一次也靠不住——卡 0 同一键第 4 次只有第 3 次的 0.38 倍（第一节 `crt_shift`），后量与先量差 10% 以上。这一半是「判别子观测不到」。
3. 满足的分句：`.claude/kb/decisions/26-后台整理与放置回收.md:246`：「2. **用哪几张卡**：挂载参数 `background_gpu_devices` 是一张卡的列表（可以点几张）；不给就用本机全部显卡。」与第 7 条（第 251 行，指路不抄）。
4. 改法还中不中：第二轮攻方提的「挂载时每张卡量一次 c_rt、赔的卡不用」（零轮）在这一格上照中：它的判别子就是上面第 2 句说靠不住的那个读数。「已验厂商独显」碰不到（卡 1 是已验厂商独显）。「不点卡时只用一张」碰得到，但改的是第 2 条字面，要用户答（零轮，第五节）。

### G4 ②　CPU 只改位图与逐条核：打中（最终），形态是条款字面错，不分辨臂

第一轮攻方 G4-② 打中「这句话与发布路径相左」，第二轮没有攻。这一轮补两样第一轮没有的：

- **与哪个候选都无关**。逐条核在 `crates/singlefs-core/src/transaction.rs:10138`：「fn relocation_checked_against_the_version<Reader: PoolReader + ?Sized>(」，文档注释第 10133 行：「/// 或是留存的映射条目（字节从盘上按映射位置读、核位置项里的校验和，两份取先核得上的那一份）；源落点要就是映射位置、跨度合它的类。」，读盘核那一句在第 10250 行：「.filter(|copy| crc32_castagnoli(copy) == location.unit_checksum)」。第 4 次重跑写法段量的阶段只有扫描 CRC、扫描、释放前核 CRC、选段四样（`cpu_writing_stage` 行的字段），候选 A–D 换的也只在这四样里；逐条核映射位置这一步在 A、B、C、D 下都留在 CPU 上。所以「CPU 只改位图，不再抽样复核」对四个候选都不成立，这是「打中不分辨臂」：病在条款的括注，不在哪条臂。
- **「拿掉」那一读法被另一条已定项挡住**。`.claude/kb/decisions/19-块指针的结构与宽度预算.md:109`：「1. **释放一律经映射，不经提示**；经映射核到那条映射条目之后，释放之前还要按它位置项里带的单元校验和读盘核一次。**核出对不上时隔离那个槽，发布照成**：逻辑上照样释放（映射条目去掉），物理槽不还回空闲池，记进隔离并计数报出。」整理一批释放源落点，照这一条要逐个读盘核。第一轮表里的两种读法，「七·3 照字面拿掉这些核」撞这条硬规则，只剩「留着、第 8 条改字」一种。候选 D 把释放前核的 CRC 放上 GPU 时，硬规则 1 的结果就由两份 GPU 核的「一致」定，D19 没写这一层（推的；D 在第 4 次重跑两处都换之后两张卡全赔，第一节表）。

四句：不分辨臂（四个候选一起中）；判别子在代码里看得到；满足的是 D26 已定项 13 第 8 条（`.claude/kb/decisions/26-后台整理与放置回收.md` 第 252 行，指路不抄）括注里讲 CPU 只做什么的那半句；改法只有改字（零轮，第五节），跑前没有别的改法可核。

### G4 的 N　N 取几：构造不出（最终）；计数住哪：第一轮那一序列仍可达，这一轮另找到一处「一轮」没定义（推的）

第 4 次重跑控漂移段的数（`hanging_checks.py`；有效的格里每次计时提交两份核与 CPU 逐单元比，间隔 0 的格 100 次、50 ms 的格 50 次，照登记第五节「控漂移段怎么量」那一句）：

```text
$ grep -E 'n_count arm=all|n_units' out/hanging_checks.out
name=n_count arm=all index=all kernels=two_kernels valid_cells=1080 uncertain_entries=0 cells_disagreeing_with_the_cpu=0 unit_comparisons_in_timed_submissions=7065000 invalid_cells=11
name=n_count arm=all index=all kernels=one_kernel_today valid_cells=120 uncertain_entries=0 cells_disagreeing_with_the_cpu=0 unit_comparisons_in_timed_submissions=942000 invalid_cells=1
name=n_units units_4=151 units_8=153 units_12=151 units_16=150 units_20=152 units_24=153 units_32=150 units_512=152
```

（判定一览原先写的「1091 格、7154200 次」把 11 个作废格也数进去了，已改成有效格的数。）

- 零不确定，但输入是同一个工作负载单元重复 512 次：`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:15544`：「let unit = super::gpu_submission::one_data_unit_of_the_workload();」，第 15548 行：「unit_bytes: unit.repeat(count),」，控漂移段在第 17928 行取它：「let segment_units = segment_units();」。705 万次比对只覆盖一种字节内容，对「某种内容让两份核分歧」这一类不加宽第 3 次重跑第二段那 1.428497e-6 的上界（那一段是随机字节）。N 取几仍由不出现的东西定，构造不出能让 N 翻面的读数；第 3 次重跑登记本来就写了 N 不由零次翻面（第二轮攻方第六节）。
- 计数住哪：第一轮攻方 G4-③ 的序列（重挂清零、永远到不了 CPU 判）不依赖任何读数，第 4 次重跑不改它；`crates/singlefs-core/src/` 里今天没有不确定计数（`grep -rn 'uncertain\|不确定' crates/singlefs-core/src/*.rs | grep -ic gpu` 得 0），条款外没有别的东西顶着。
- 另一处（推的，读条款与代码形状）：第 8 条「同一个条目连续 N 轮都不确定」的「一轮」随候选不同。A、C 在扫描里算校验和，而整理每批之前对 R 重扫（第一轮判决 G2 那一格），R 里一个不确定的单元每批都被重算一次，不管它在不在这一批里；D 只核这一批要释放的那几个，不在这一批的单元这一轮根本不算。N 取同一个数，在 A、C 下是 N 批，在 D 下是「这个单元进了 N 次批」；再配上岔路单第 7 行 B 读法（只那几个条目不搬），不在批里的那几轮算不算「连续」也没写。

四句（计数那一半）：分辨「计数随意图持久」与「住内存」，也分辨「一轮 = 一批」与「一轮 = 这个单元被算一次」；判别子系统看得到（在不在这一批里、算没算它）；满足第 8 条那一句；改法「重挂与切换时不确定的条目直接交 CPU」（第一轮攻方，零轮转一轮）碰得到重挂那一半，碰不到「一轮」没定义那一半。

### G5　控制接口：打中只在 G8 选「挪出」时可达；选「不挪出」构造不出（最终）

第二轮攻方的反例是 S（请求切到 `cpu`）、E（这一批 GPU 出错）、B（下一批开头）的次序 S、E、B：GC 停着、模式已是 `cpu`，再也走不到「进门读开关」。攻它要看 S 能不能落在一批的中间。

- 不挪出（G8 甲）：整理在用户改动的发布函数里跑，会话是独占借用，`crates/singlefs-core/src/mounted_session.rs:1849`：「pub fn publish_user_change<Device: BlockDevice>(」，开关在一趟开头读一次，同一文件第 2641 行：「let switches = allocator.compaction().switches;」。控制接口也是会话方法，要等这次用户改动返回才进得来，S 只能落在 E 之后（E、S、B，条款下照常）或 B 之前没有 GPU 批（S、B、E 不发生）。S、E、B 这一序在甲下构造不出。
- 挪出（G8 乙-1、乙-2）：切换请求与一批并行到是常态，第二轮那一序成立。第二轮攻方的补法「停着时收到切换请求当场生效、起 GC」（零轮）碰得到它。
- 试过、没成的：① 第 3 条「把连续几批攒起来一次交」让「下一批」落进一次提交的中间——G2 打穿了「跨批攒不成」，第 4 次重跑 648 行一行跨意图都没有（第一节 G2），这一序在今天的约束下到不了；② 「修好 GPU 再切」那一支（`gpu` 切 `gpu`）与第二轮的「`cpu` 切 `cpu`」是同一个缺口，不另算。

四句：分辨「进门读开关」与「切换请求自己能拉起停着的 GC」，但只在 G8 乙下；判别子（请求与出错谁先到）乙下看得到；满足第 1 条的生效时点与第 4 条的恢复办法（`.claude/kb/decisions/26-后台整理与放置回收.md` 第 245、248 行，指路不抄）两句合起来；改法同第二轮。这一格的去留跟 G8：G8 定了甲，它就不用补。

### G6 的改法「挪出调用线程」：够不着「等多久」那一格（最终，这一轮是量的）

第二轮攻方推的「挪出之后 fsync 要等这一批」，第 4 次重跑写法段给了入库装置上的数：一批（`kind` 不是 `issue` 的真调用）的主机 CPU，与同一次调用把扫描 CRC、释放前核 CRC 换成三路硬件指令、扫描换成按身份查（不计建索引）之后的值：

```text
$ grep batch_cpu out/hanging_checks.out
name=batch_cpu fraction=0.25 pool_data_units=1024 calls=20 today_ms_min=12.3 today_ms_max=13.3 both_three_way_without_build_ms_min=12.1 both_three_way_without_build_ms_max=13.1 largest_share_saved=0.0195
name=batch_cpu fraction=0.25 pool_data_units=8192 calls=20 today_ms_min=100.0 today_ms_max=110.4 both_three_way_without_build_ms_min=99.8 both_three_way_without_build_ms_max=110.0 largest_share_saved=0.0038
name=batch_cpu fraction=0.75 pool_data_units=1024 calls=55 today_ms_min=12.2 today_ms_max=13.9 both_three_way_without_build_ms_min=12.0 both_three_way_without_build_ms_max=13.3 largest_share_saved=0.0436
name=batch_cpu fraction=0.75 pool_data_units=8192 calls=55 today_ms_min=96.4 today_ms_max=112.7 both_three_way_without_build_ms_min=95.7 both_three_way_without_build_ms_max=111.6 largest_share_saved=0.0095
```

一批 12.0–13.9 ms（P = 1024）、95.7–112.7 ms（P = 8192），换写法至多省 4.4%。挪出之后 fsync 在一批开头之后到，要等这一批的 CPU，GPU 模式再加一次飞着的提交（第 4 次重跑复现臂一次提交挂钟 5.30–6.59 ms，第一节 G1-③）。第 6 条「前台有 fsync 在等时不发下一批」管得到「下一批」，管不到「这一批」；这一批的长度随 P 涨、不随写法降。改法让判别子看得到（第一轮打中的那一半），「等多久」那一格它不碰。这是「改法碰不到打中的格」那一形：交出去时写明「挪出对等多久不起作用，要的是把一批切小或让一批可中途让出」（零轮，第五节）。

### G7　GPU 代码住哪：打中（最终），形态换成「门禁看不见 checker 档与产品 GPU 包共用」

- 第二轮攻方说「产品只能用 `wgpu` 30.0.1、同一份 naga」：钉法在 `crates/singlefs-checker-tier/Cargo.toml:27`：「wgpu = { version = "=30.0.1", default-features = false, features = ["vulkan", "wgsl", "std"], optional = true }」，`Cargo.lock` 里 `name = "wgpu"` 只有一条（`grep -c '^name = "wgpu"$' Cargo.lock` 得 1）。Cargo 在同一个工作区里只把语义版本相容的需求并成一份，产品包写 `wgpu = "31"` 就会另解一份（推的，读的是 Cargo 的版本规则，没在本仓试）。所以「只能用同一份」过头；站得住的是：要和 checker 档不同，就得不同大版本或另开工作区，而驱动里的着色器编译器两边换不掉（同一台机器同一个 Vulkan 驱动，第一轮攻方 G4-④）。
- 这一轮新打中的一层：守「checker 与实现不共用」的门禁只拿两个包求交。`.claude/gate.d/checker-independence-and-sync.sh:24`：「#   ① 按 crates/*/Cargo.toml 建内部依赖图，singlefs-checker 与 singlefs-core 各取传递闭包求交，减去 singlefs-format 必须为空。」checker 档本来就依赖 core：`crates/singlefs-checker-tier/Cargo.toml:10`：「singlefs-core = { path = "../singlefs-core" }」。checker 档的 GPU 判器（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 一类；D26 已定项 13 第 254 行那一段写着 checker 档的崩溃判器已经用 WGSL 经 `wgpu` 起，指路不抄）若直接 use 产品 GPU 包里的核，① 不红；产品 GPU 包挂在 core 下面（core 可选依赖它）时，checker 档的闭包里本来就有它，按包粒度的门禁连「不许依赖」都写不出来。D13 已定项 5「校验和各用一份独立实现」（`.claude/kb/decisions/13-验证路线.md` 第 156 行，指路不抄）对 GPU 判器与产品 GPU 核这一对没有会红的检查。
- 序列（推的，产品 GPU 代码还不存在）：七·3 写产品 GPU 核时，为省事从 checker 档的 GPU 判器里搬一个 WGSL 的 CRC 函数进共用模块，两边各 use 一次；这个函数把单元末尾几个字节漏算，产品两份核（第 8 条要求不共用核函数，但这个辅助函数不是核函数本身）与 checker 档判器都判「对」，崩溃点重放与 GPU 判器一起放过。

四句：分辨「产品 GPU 包与 checker 档不共用源码」与「共用」；判别子（use 语句、依赖表）看得到，门禁不看；满足 D13 已定项 5 的字面，加门禁 ① 的射程；改法（零轮）只有一个形态门禁写得出来：产品 GPU 包不进 core 的依赖、由挂载的调用方把后端递进来（第一轮攻方 G7 两种依赖方向里的前一种），再加一条「checker 档不依赖产品 GPU 包」。另一种方向（core 可选依赖它）下这条检查按包写不出来，要按模块或按 use 判。

### G8　GC 挪不挪出调用线程：两边都有代价（最终），其中「不挪出」先撞已定项的「不许做」

- 不挪出那一边不是一笔代价，是一道禁令。`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:32`：「**定案**：三条，不满足就不许做——」，第 35 行：「2. **不许进前台路径。** PCIe 往返是几十微秒量级，前台读写延迟吃不起这个；它只能服务已经攒批的后台重活。」不挪出又用 GPU，就是 G1-③ 那一形（打穿）；照定案句，要做只能用户改这一条或给「前台路径」另下定义，不能拿「前台多等几毫秒值不值」去比。岔路单第 11 行把甲的翻面观测写成「是否违 D24 已定项 2 第 2 条」，违了的结局条款已经写死。
- 第 35 行自己的数也对不上：它说「几十微秒量级」，第 4 次重跑卡 0 一次提交的挂钟 5.30–6.59 ms（第一节 G1-③，180 格），第 1 次重跑每次提交固定 4.492 ms（第二节第 3 小节），是它的约 100 倍。方向与禁令一致（更吃不起），但条款里那个数不是量的。
- 挪出那一边：第二轮攻方的三样在今天的代码上一样都没补。预算 `.max(1)` 还在：`crates/singlefs-core/src/compaction_plan.rs:862`：「.max(1)」（这份文件开工快照之后被别的会话改过，倒推不出快照那一份；函数体与第二轮引的相同）；签下一条的判据还是一趟里的局部变量：`crates/singlefs-core/src/mounted_session.rs:2704`：「let mut last_completed_intent: Option<CompactionIntent> = None;」；整理批仍不过准入：`crates/singlefs-core/src/transaction.rs:9667`：「/// （固定点，空间归 checkpoint 保留池），搬迁副本不进重写的角色、不进需求，所以两判都不判；合取 7 免（带搬迁的发布不是前台写入，」（同样是快照之后改过的文件）。挪出之后的主机 CPU 代价，第 4 次重跑没有量（c_rt 不随线程变是辩方推的）。
- 第 4 次重跑给 G8 的新数只有一样：c_rt 降到 0.38 倍，挂钟不降（G1-③）。前台多等的是挂钟，所以「不挪出」那一边的代价不随 c_rt 那一跳变小。

四句：分辨甲、乙；判别子看得到；满足 D24 已定项 2 定案句加第 2 条（甲）与第二轮三样（乙）；改法：甲要用户改条款，乙要补三样，都交用户（第五节）。

## 四、岔路单漏了什么（关的第 4 条）

对照的是 `research/prompts/m4-gpugc-design-r1-forks.md` 的十一行加 C906（`.claude/kb/checks-owed.md` 第 585 行，整理批不过准入），与测量问题单 `research/prompts/m4-gpugc-measure-r4-forks.md` 的三行。测量问题单第 1 行问的是「量哪几种写法」，量完了；岔路单第 1 行问「GPU 上算哪几段」，而它要拿来比的那把尺（CPU 模式一批取哪种写法、c_rt 取哪一次、按哪种间隔读、用哪种钟）一行都没有。下面九行照岔路单的写法：只写候选、翻面观测、够判条件，不写倾向、不写已有的数（数在前面各节）。

| # | 候选（各自的定义） | 翻面观测 | 够判条件 |
|---|---|---|---|
| M1 c_rt 取哪一次、哪种布置 | 甲 第 3 次重跑第一段；乙 第 4 次复现臂（不绑核、只起卡 0）；丙 第 4 次受控臂（绑一个核、两卡交替）；丁 几次里取每键最坏 | 同一键两次之间 c_rt 的水平比；水平比让哪些格的判定翻面（岔路单第 1 行、第 11 行的读数都挂在它上面） | 同一布置在不同的日子至少再量一次，同一键的水平比落在登记写死的容差内；或作废谓词能拦下这种水平差（今天的谓词没拦） |
| M2 按哪种间隔读 | 甲 间隔 0（连发）；乙 间隔 50 ms；丙 间隔取产品两批之间的真实间隔（不短于一批的主机 CPU） | 同一候选、同一写法在两种间隔下判定相反的格数（C 两处都换之后卡 0 在两种间隔下相反） | G8 定了之后量产品两批之间的间隔分布；或第一段补量长间隔几档，覆盖到一批 CPU 的上端 |
| M3 CPU 模式那一侧取哪种写法当第 7 条的减数 | 甲 今天的写法；乙 硬件 CRC 单路或三路；丙 按身份查，建索引计入；丁 按身份查，建索引不计；戊 两处都换，建索引计入或不计 | 候选 C 在卡 0 上「不赔」随写法从全不赔变到约一半（建索引计不计是翻面最大的那一刀） | 用户定第 7 条的「CPU 模式下它的 CPU 时间」指今天的实现还是指写法换掉之后的实现；建索引算不算一次性代价、摊到几批 |
| M4 多卡怎么分活 | 甲 本机全部显卡轮转；乙 只用一张（用户点名或挂载时挑）；丙 按每张卡的 c_rt 分量；丁 两份核分在两张卡（岔路单第 5 行丙） | 轮转之后「只用卡 0 不赔」的格里翻成赔的有几成；两卡同时起着时一张卡后量比先量高多少 | 本机三张卡（含 AMD 集显）同一时段同一跑法量第一段；用户答第 2 条「全部」是不是指同时用 |
| M5 控制接口与 GPU 出错的次序 | 甲 进门读开关（第一轮正推）；乙 切换请求在 GC 停着时当场生效并拉起 GC | 「请求切换、这一批出错、下一批开头」这一序下 GC 起不起得来 | G8 定了；定乙的话，条款补一句或实现里一条单测走这一序 |
| M6 产品 GPU 代码与 checker 档的边界 | 甲 产品 GPU 包不进 core 的依赖、挂载时递进来，门禁加「checker 档不依赖它」；乙 core 可选依赖产品 GPU 包，按模块或按 use 判不共用；丙 产品另开工作区或另一个 `wgpu` 大版本 | checker 档的 GPU 判器与产品 GPU 核有没有一段共用的着色器源码；门禁在哪一种下写得出会红的检查 | 七·3 动笔之前用户定 D13 已定项 5 的「独立实现」管不管 GPU 判器与产品 GPU 核这一对 |
| M7 第 8 条「CPU 只改位图，不再抽样复核」与逐条核 | 甲 括注改字，逐条核映射位置与 D19 已定项 5 硬规则 1 的读盘核留在 CPU；乙 逐条核也放上 GPU、经两份核比对；丙 只留硬规则 1，映射位置那一核拿掉 | 每个候选下 CPU 侧还剩哪些逐条的核、各要多少主机 CPU；拿掉的那一核原来抓得到的错在新形态下谁抓 | 写法段那样把逐条核单列一个阶段量出 CPU；用户定括注怎么改 |
| M8 主机 CPU 用哪种钟 | 甲 量提交那条线程的线程钟；乙 进程钟（含驱动与 `wgpu` 在本进程起的线程） | 线程钟少算一成以上的格占多少；换成进程钟之后哪些格的判定翻 | 每个点都出进程钟的行（今天只在线程钟少算超过登记那条线时出）；用户定第 7 条的「主机 CPU 时间」算不算驱动线程 |
| M9 带快照的池 | 甲 门槛只在不带快照的池上量（E185 今天这样）；乙 补量带快照的池：留存的映射条目逐条从盘上读、核校验和，这一段在 CPU 上 | 带快照时一批里那段读盘核 CPU 占多少；它算进哪个候选的「省下的」 | 写法段在至少一档带快照的池上重跑一次 |

另有一处是已有一行缺一列：第 6 行（N）没写「一轮」按什么数（第三节 G4 的 N 那一段）；够判条件要加「用户定一轮是一批，还是这个条目被两份核算一次」。

## 五、交用户表：这一轮新冒出的改法（都是零轮）

照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」：第三轮新冒出的零轮形态不再开一轮，写进这张表。全部只在我的模型或我对代码与条款的读法上，被攻过零轮。

| 改法 | 修哪一格 | 在打中的那几格上还中不中 | 标 |
|---|---|---|---|
| 不点卡时只用一张卡（用户点名或挂载时按设备类型挑一张），不默认全部显卡轮转 | G3 | 碰得到轮转那一形；碰不到「那一张卡自己的 c_rt 随趟次跳」（第一节 `crt_shift`） | 推的（依据的读数是量的） |
| 第 8 条括注改成：GPU 两份核一致的条目交 CPU，CPU 照现行发布路径逐条核映射位置与读盘核，不再另外抽样复核 GPU 的结果 | G4 ② | 让条款与 `relocation_checked_against_the_version` 一致；不改任何候选的判定（打中不分辨臂） | 推的 |
| 第 8 条「一轮」定义成「这个条目被两份核算一次」，计数跟意图走（重挂、切换时按第一轮攻方的改法交 CPU） | G4 的 N | 碰得到「一轮随候选不同」与重挂清零两处 | 推的 |
| 一批切小、或一批在 GPU 等的时候可以让出、fsync 到了就停在批内的安全点 | G6 | 碰得到「等多久」；它与 G8-④（等的时候放开会话，计划会过期被逐条核拒）同一件事，要一起定 | 推的 |
| 产品 GPU 包不进 core 的依赖；门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格加一条「checker 档不依赖产品 GPU 包」 | G7 | 碰得到「门禁看不见共用」；碰不到驱动编译器那一层共同路径 | 推的 |
| 作废谓词加一条：同一键与上一次重跑的 c_rt 水平比超出登记容差就不当已判 | M1、G1-① 间隔 0 那一半 | 碰得到「2.6 倍的水平差没被拦」；碰不到「哪一次是对的」 | 推的 |
| 第 7 条的比较对象写明 CPU 模式取哪一种写法、建索引算不算 | M3 | 定了它，第 4 次重跑的重判就能读成一个判定 | 推的 |

不在这张表里但要交用户的两件（不是改法，是用户的定夺）：
- G8 甲（不挪出、只在调用线程上用 GPU）撞 D24 已定项 2「不满足就不许做」：要么用户改第 2 条或给「前台路径」下定义，要么甲出局。
- 第二轮攻方的零轮改法这一轮的去向：「挂载时每张卡量一次 c_rt、赔的卡不用」被这一轮打中（G3 第 4 句）；「停着时收到切换请求当场生效」没被攻（只在 G8 乙下要）；G8 的三样补法没被攻（第三节 G8 那一段只核了今天代码上三样都还没补）。

## 附：M8、M9 的读数

```text
$ grep -E 'clock_pair|snapshot_note' out/hanging_checks.out
name=snapshot_note the_pools_have_no_snapshot=210
name=clock_pair arm=controlled index=0 keys=108 keys_with_a_process_line=100 process_over_thread_median=1.112 min=1.040 max=1.227
name=clock_pair arm=controlled index=1 keys=108 keys_with_a_process_line=12 process_over_thread_median=1.051 min=1.036 max=1.072
name=clock_pair arm=reproduction index=0 keys=108 keys_with_a_process_line=84 process_over_thread_median=1.124 min=1.030 max=1.281
```

进程钟的行只在线程钟少算超过 10% 时才出（装置 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:11610`：「    pub(super) const PROCESS_CLOCK_EXCESS_ABOVE_FOR_UNDERESTIMATE: f64 = 0.10;」）。卡 0 上 108 个键里 100 个（受控臂）、84 个（复现臂）有进程钟的行，中位多 11%–12%；同一份重判里候选 C 今天的写法卡 0 在进程钟下 100 行不赔 90、不稳定 10，线程钟下 108 行不赔 101、不稳定 7（模型 `out/rejudge_arms.out` 的 `arm_summary` 行）。

## 没打中的形状

| 格 | 试过什么 | 为什么没成 |
|---|---|---|
| G1-③ | 找 c_rt 降了之后挂钟也降、让前台多等变小的读数 | 复现臂卡 0 至多 24 单元 180 格挂钟 5.30–6.59 ms，没降 |
| G2 | 找受控臂与复现臂里跨意图攒得成的格 | 648 行 `merging_shape` 0 行跨意图 |
| G4 ① | 找第 4 次重跑里同输入对拍红的格 | 三行 `gpu_card_verdict` 不同都是 0；同输入对拍本来就是 G4 ① 说抓不到那一类 |
| G4 的 N | 找能让 N 取值翻面的读数 | 两份核 1080 格零不确定，输入只有一种字节内容 |
| G5 | G8 甲下让切换请求落进一批中间；拿第 3 条的攒批边界造「下一批」落在一次提交里 | 会话独占借用；跨批攒不成（G2） |
| G6 改法 | 找一种写法让一批短到 fsync 的等可以忽略 | 三路 CRC 加按身份查至多省 4.4% |
| G7 | 在本仓里实际让两个 `wgpu` 大版本并存 | 没试：不编译。Cargo 的版本规则是推的 |
| G3 | 第三张卡（AMD 集显）的 c_rt | 不起 GPU |
| G8 | 找挪出调用线程之后主机 CPU 的代价数 | 三份产物里都没有这一项 |

取样范围：控漂移段 4252 行点行全读（两条臂、三种等法、两种输入、两种间隔、两种门槛、两种钟），1212 行 `submission_cell` 全读；写法段 210 行真调用与 210 行写法阶段全读；重判产物只读 `baseline_threshold` 与 `baseline_rejudge_summary` 两种行。

## 这条腿自己的限度

- 这是续做。前一条腿写了复跑、判定一览、第一节与第二节第 1 小节，模型目录里四个 python 与复跑脚本。续做核了现场（报告 249 行、模型目录文件与报告里列的 sha256 对得上、复跑退 0），已写完的节没重写；前一条腿写的部分只改了两块：复跑那一段（模型数、三次复跑、sha256 表加 `hanging_checks`），与判定一览的「关 2」「G4 的 N」「关 4」三行（对不上的个数、N 的格数原先把作废格数了进去、漏的行数）。
- 模型只读产物、做算术，没编 Rust、没跑 `crates/`、没起 GPU。轮转（G3）是把两张卡同一轮的 c_rt 取平均，是推的分法，不是量的轮转。
- 引的 `crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/compaction_plan.rs`、`crates/singlefs-checker-tier/Cargo.toml` 在开工快照之后被别的会话改过（`sha256sum -c` 三份 FAILED，`scripts/agents/snapshot-tree.py m4-gpugc-design-r3 --list sha256sums.txt` 倒推不出这三份），行号是今天工作区的；引的那几行内容与第二轮判决引的相同。E185 装置源码不在快照清单里，行号也是今天工作区的。
- 「推的」：Cargo 允许两个大版本并存；G7 的共用序列；G4 的 N「一轮随候选不同」；候选 D 下硬规则 1 由两份 GPU 核定；卡 0 第 3 次读数偏高的原因；受控臂卡 0 更不稳是绑核还是交替造的。
- 「没打中」只抽了一次样，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」不支撑「没打中」的结论。

## 没做什么

- 没读同一轮辩方腿的报告（禁读）。
- 没跑门禁、没跑 checker 档测试、没起 GPU、没编 Rust、没用第二台；复跑脚本的 `--selftest`（模板自证）没跑。
- 没在入库装置上重量任何数；本报告里模型上的数都不是入库装置上的数。
- 续做草稿目录 `~/.local/share/singlefs/work/m4-gpugc-design-r3-opus-2/` 里三次复跑的输出目录与倒推出的快照树交回前已删（删之前 `du -sh`：`rerun1`、`rerun2`、`rerun3` 各 1.6M，`tree` 78M）；留着的是模型草稿、三份复跑日志与几份替换规格。前一条腿的草稿目录 `~/.local/share/singlefs/work/m4-gpugc-design-r3-opus/`（它认领的）里还有它的 `rerun1`（1.6M，复跑输出，不是仓副本）与模型草稿，我没动，交主 agent。
- 要主 agent 跑的命令：无。
