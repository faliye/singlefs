# m3-close-b19-reason-r2 辩方腿承接报告（Sonnet，2026-11-02）

承接：`research/prompts/m3-close-b19-reason-r2-sonnet-output.md`（下称前一份）。主 agent 跑了任务 4924（`type=succeeded exit=0`），输出在 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r2-sonnet-run1/`。本腿把九份输出逐字节拷进 `research/prompts/m3-close-b19-reason-r2-sonnet-model/out/`（`cmp` 九份全同），下面「整行抄」的位置都指 out/ 里的行号。
口径：量的 = out/ 里的行；推的 = 读模型源码、kb、walk.rs 推出来、没有对应输出行。本腿没跑模型、没往任务服务交任务；禁读清单里的两个路径与目录一个没读。立场不变：复核第一轮判决，Q1–Q3，不另开问题。

## 零、开工核对

- 模型源码五份 sha256 与前一份第零节一致（量的，本腿此刻跑的 `sha256sum`）：`q1q2/q1q2_unit_class_reading.py` a886289e23bf644b3b912c6dc464347c685778864c1b9b575186022b4163befc、`q3/ding_today.py` 9d82ea84af46e17507be7fe4d50876c1e85f79d7ada01a7a55c3b0e5cbbd588a、`q3/history_today.py` 9e104860650b913100bbc97c2c3e1213ae20c317044d199b2649287d39302346、`q3/q3_today_run.py` 95d1eab9003da71f0176e37788b752471fd22fc1e4dff93a26a64482db8e5e57、`tools/rerun.sh` 4ee52bdc02bca324e1848c2d2d802bb4c0722e62cabf88366b253ff022bbe72d，输出文件的 sha256 在第五节。
- 九个 `b1-*.rc` 全是 0（量的，读 run1 目录）。
- 模型取 20000 个固定种子，同一份输入重跑逐字节相同；复跑是核「没有非确定性」，不是新抽一次样。
- 移植印证（量的）：`cfg=verdict` 把本腿 `ding_today.py` 与今天实现的四处差别全关掉（口径同第一轮攻方模型），四个格的整数与第一轮 `q2ext.txt` 的 `all[...]` 逐位相等。两份代码不共用（攻方的 `ding_port.py` 与本腿按 `walk.rs` 重写的 `ding_today.py`）：
  - S=4 合法：「partial=1293 not_judged=357」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-128.txt:2`) 对 「all[final holds=13977 red=0 partial=1293 not_judged=357 | ding_always holds=14292 red=1 partial=1334 | partial_why ambiguous:1293]」(`research/prompts/m3-close-b19-reason-r1-opus-model/out/q2ext.txt:3`)
  - S=8 合法：「partial=604 not_judged=300」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-128.txt:2`) 对 「all[final holds=24774 red=0 partial=604 not_judged=300 | ding_always holds=25068 red=0 partial=610 | partial_why combos:3,ambiguous:601]」(`research/prompts/m3-close-b19-reason-r1-opus-model/out/q2ext.txt:6`)
  - S=4 坏写者：「red=13232 partial=831 not_judged=319」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-4-128.txt:2`) 对 「all[final holds=0 red=13232 partial=831 not_judged=319 | ding_always holds=0 red=13542 partial=840 | partial_why ambiguous:831]」(`research/prompts/m3-close-b19-reason-r1-opus-model/out/q2ext.txt:9`)
  - S=8 坏写者：「red=22768 partial=330 not_judged=262」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-8-128.txt:2`) 对 「all[final holds=0 red=22768 partial=330 not_judged=262 | ding_always holds=0 red=23027 partial=333 | partial_why combos:1,ambiguous:329]」(`research/prompts/m3-close-b19-reason-r1-opus-model/out/q2ext.txt:12`)

## 一、读数表（Q1 / Q2：`out/q1q2.txt`）

前一份第六节的预测在「预测」一列；「对不对」只比那一句预测。小世界的口径（模型源码文件头第 7 至 23 行）：7 槽一块盘，至多两个单元，每单元一条记录加至多一条一槽宽的额外记录，θ_high=5，t 取 5 或 3。

| 读数 | 写之前的预测 | 读数（整行抄） | 对不对 |
|---|---|---|---|
| `name=q1q2 ... worlds=` | 无数 | 「name=q1q2 slots=7 worlds=641608」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:1`) | 无预测 |
| `name=healthy_but_not_holds` | 没有这一行 | 这一行不存在：`grep -c healthy_but_not_holds out/q1q2.txt` 退出前打出 0（量的，本腿跑的）；模型源码对每个非空的键打一行（「for k in sorted((k for k in tally if k[0] == 'healthy_but_not_holds'), key=str):」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/q1q2/q1q2_unit_class_reading.py:267`)），没有行即空 | 一致 |
| `name=verdict mode=ne\|gt\|none` | 无数 | 「name=verdict mode=ne holds=74382 partial=50406 red=516820」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:4`)；「name=verdict mode=gt holds=74382 partial=10658 red=556568」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:5`)；「name=verdict mode=none holds=74382 partial=0 red=567226」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:6`) | 无预测；三种读法 `holds` 都是 74382，改记读法不动健康盘面 |
| `name=side upper_slot_judge_alone_turns_partial_into_red_mode_ne` | 为 0 说明逐槽上沿被个数夹住 | 「name=side upper_slot_judge_alone_turns_partial_into_red_mode_ne=6226」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:3`)；gt：「name=side upper_slot_judge_alone_turns_partial_into_red_mode_gt=1754」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:2`) | 一致于「不为 0 即逐槽上沿单独起作用」：ne 下 6226 个盘面三判都成立、只有逐槽上沿把部分没判改判红（占 `partial` 50406 的 12.4%），不是摆设 |
| `name=partial_ne ... only_I-3.18`，`kinds` 不是 `only_span_wrong` | 没有这样的行 | 十二行，见下面小表 | **不一致**：十二行，合计 11650 个盘面 |
| `name=partial_ne ... only_I-3.18`，`kinds=only_span_wrong` | 无数 | 「name=partial_ne kinds=only_span_wrong only_I-3.18 worlds=13016」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:35`) | 无预测 |
| `name=partial_ne ... named_by_another_invariant`（全部行合计） | 无数 | 例：「name=partial_ne kinds=released_over_unit named_by_another_invariant worlds=7390」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:41`)；「name=partial_ne kinds=only_span_wrong named_by_another_invariant worlds=4040」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:34`) | 无预测；合计：`only_span_wrong` 4040、其余 21700（行数加总，本腿算的） |
| `name=partial_ne_i318_red` | 只有 `True` | 「name=partial_ne_i318_red {True: 50406}」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:45`) | 一致 |
| `name=only_span_wrong mode=ne` | 无数 | 「name=only_span_wrong mode=ne verdict=holds worlds=6140」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:10`)；「name=only_span_wrong mode=ne verdict=partial worlds=17056」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:11`)；「name=only_span_wrong mode=ne verdict=red worlds=3344」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:12`) | 无预测 |
| `name=only_span_wrong mode=gt` | 无数 | 「name=only_span_wrong mode=gt verdict=holds worlds=6140」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:7`)；「name=only_span_wrong mode=gt verdict=partial worlds=2996」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:8`)；「name=only_span_wrong mode=gt verdict=red worlds=17404」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:9`) | 无预测；`none`：「name=only_span_wrong mode=none verdict=holds worlds=6140」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:13`)；「name=only_span_wrong mode=none verdict=red worlds=20400」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:14`) |
| `name=partial_ne_but_gt_says` | `other_kinds` 的 `red` 是「`!=` 藏别的病」的判据 | 「name=partial_ne_but_gt_says ('red', 'only_span_wrong') 14060」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:47`)；「name=partial_ne_but_gt_says ('red', 'other_kinds') 26752」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:48`) | 判据字面打中（26752），但这一行把「别的不变量已点名」的盘面也数进去了，见第二节 Q1.1 |
| `name=partial_gt_but_ne_says` | 无数 | 「name=partial_gt_but_ne_says ('red',) 1064」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:46`) | 无预测：有 1064 个盘面 `>` 部分没判而 `!=` 判红 |
| `name=triple` | 无数 | 「name=triple ne=holds gt=holds none=holds worlds=74382」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:49`)；「name=triple ne=partial gt=partial none=red worlds=9594」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:50`)；「name=triple ne=partial gt=red none=red worlds=40812」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:51`)；「name=triple ne=red gt=partial none=red worlds=1064」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:52`)；「name=triple ne=red gt=red none=red worlds=515756」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:53`) | 无预测；行与行互为镜像：50406 = 9594 + 40812 |
| `name=c944_pure`，`mode=ne` 十二行 | 空 | 见下面 c944 小表 | **不一致**：非空，三种读法逐行同数（行 54 至 89），合计 18239 |
| `name=c944_ne_not_red` | 无数 | 「name=c944_ne_not_red verdict=holds alone worlds=18239」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:90`)；「name=c944_ne_not_red verdict=holds with_another_red worlds=24610」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:91`)；「name=c944_ne_not_red verdict=partial with_another_red worlds=17986」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:92`) | 无预测；第 90 行的 18239 等于 c944 十二行（`mode=ne`）逐行相加 |

`partial_ne` 里 `kinds` 不是 `only_span_wrong`、尾字段是 `only_I-3.18` 的十二行（预测里写的「出现这样的行就推翻」）：

| 行 | 整行 | 盘面数 |
|---|---|---|
| 16 | 「name=partial_ne kinds=accounting_off+duplicate_unreleased+missing only_I-3.18 worlds=60」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:16`) | 60 |
| 19 | 「name=partial_ne kinds=accounting_off+missing only_I-3.18 worlds=3840」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:19`) | 3840 |
| 21 | 「name=partial_ne kinds=accounting_off+missing+released_over_unit only_I-3.18 worlds=1152」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:21`) | 1152 |
| 25 | 「name=partial_ne kinds=accounting_off+released_over_unit only_I-3.18 worlds=204」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:25`) | 204 |
| 29 | 「name=partial_ne kinds=duplicate_unreleased+missing only_I-3.18 worlds=720」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:29`) | 720 |
| 30 | 「name=partial_ne kinds=duplicate_unreleased+missing+released_live only_I-3.18 worlds=100」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:30`) | 100 |
| 32 | 「name=partial_ne kinds=missing only_I-3.18 worlds=120」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:32`) | 120 |
| 33 | 「name=partial_ne kinds=missing+released_over_unit only_I-3.18 worlds=334」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:33`) | 334 |
| 37 | 「name=partial_ne kinds=released_live only_I-3.18 worlds=4176」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:37`) | 4176 |
| 39 | 「name=partial_ne kinds=released_live+released_over_unit only_I-3.18 worlds=100」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:39`) | 100 |
| 42 | 「name=partial_ne kinds=released_over_unit only_I-3.18 worlds=640」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:42`) | 640 |
| 44 | 「name=partial_ne kinds=stray_unreleased only_I-3.18 worlds=204」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:44`) | 204 |

十二行加总 11650（60+3840+1152+204+720+100+120+334+4176+100+640+204，本腿算的），占 `partial` 的 11650/50406 = 23.1%，占全部 641608 个盘面的 1.8%。`partial_ne` 全部行：`only_span_wrong` 17056（与第 11 行相等）加其余 33350 = 50406（与第 4 行的 `partial=50406` 相等）。

`c944_pure` 的 `mode=ne` 十二行（行 66 至 77）与 `gt`（行 54 至 65）、`none`（行 78 至 89）逐行同数，所以它不依赖取宽的读法：

| `kinds` | 盘面数 | 整行（`mode=ne`） |
|---|---|---|
| released_live | 7371 | 「name=c944_pure mode=ne verdict=holds kinds=released_live worlds=7371」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:77`) |
| accounting_off+missing | 4595 | 「name=c944_pure mode=ne verdict=holds kinds=accounting_off+missing worlds=4595」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:67`) |
| accounting_off+missing+released_live | 2004 | 「name=c944_pure mode=ne verdict=holds kinds=accounting_off+missing+released_live worlds=2004」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:68`) |
| accounting_off+missing+released_over_unit | 1450 | 「name=c944_pure mode=ne verdict=holds kinds=accounting_off+missing+released_over_unit worlds=1450」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:70`) |
| accounting_off+missing+released_live+released_over_unit | 588 | 「name=c944_pure mode=ne verdict=holds kinds=accounting_off+missing+released_live+released_over_unit worlds=588」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:69`) |
| missing | 702 | 「name=c944_pure mode=ne verdict=holds kinds=missing worlds=702」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:73`) |
| missing+released_over_unit | 453 | 「name=c944_pure mode=ne verdict=holds kinds=missing+released_over_unit worlds=453」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:76`) |
| missing+released_live | 330 | 「name=c944_pure mode=ne verdict=holds kinds=missing+released_live worlds=330」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:74`) |
| duplicate_unreleased+missing | 302 | 「name=c944_pure mode=ne verdict=holds kinds=duplicate_unreleased+missing worlds=302」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:71`) |
| missing+released_live+released_over_unit | 216 | 「name=c944_pure mode=ne verdict=holds kinds=missing+released_live+released_over_unit worlds=216」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:75`) |
| duplicate_unreleased+missing+released_live | 144 | 「name=c944_pure mode=ne verdict=holds kinds=duplicate_unreleased+missing+released_live worlds=144」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:72`) |
| accounting_off+duplicate_unreleased+missing | 84 | 「name=c944_pure mode=ne verdict=holds kinds=accounting_off+duplicate_unreleased+missing worlds=84」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:66`) |

十二行加总 18239（与第 90 行相等）。

用到的最小例（`example` 行，每个标签一条最小的盘面；元组 = (单元布局, (记录串, 额外记录, t, 记账已分配), 病)）：

- 行 93：「example ('c944_pure', 'gt', 'holds', 'accounting_off+duplicate_unreleased+missing') ((2, 1, 7), (((0, 1, 'W'), (1, 1, 'Q')), 'absent+absent', 'x_u@0', 5, 1), ['accounting_off', 'duplicate_unreleased', 'missing'])」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:93`)
- 行 98：「example ('c944_pure', 'gt', 'holds', 'duplicate_unreleased+missing') ((1, 1, 7), (((0, 1, 'W'),), 'absent', 'x_u@0', 5, 1), ['duplicate_unreleased', 'missing'])」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:98`)
- 行 100：「example ('c944_pure', 'gt', 'holds', 'missing') ((1, 1, 7), (((0, 1, 'Q'),), 'absent', 'x_r4@1', 3, 1), ['missing'])」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:100`)
- 行 104：「example ('c944_pure', 'gt', 'holds', 'released_live') ((1, 1, 7), (((0, 1, 'W'),), 'r7', 'none', 5, 1), ['released_live'])」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:104`)
- 行 129：「example ('partial_ne_hidden', 'accounting_off+duplicate_unreleased+missing') ((2, 1, 7), (((0, 1, 'Q'), (1, 2, 'W')), 'absent+absent', 'x_u@1', 5, 2), ['accounting_off', 'duplicate_unreleased', 'missing'])」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:129`)
- 行 130：「example ('partial_ne_hidden', 'accounting_off+missing') ((2, 1, 7), (((0, 1, 'W'), (1, 1, 'Q')), 'u2+absent', 'none', 5, 1), ['accounting_off', 'missing', 'span_wrong'])」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:130`)
- 行 133：「example ('partial_ne_hidden', 'duplicate_unreleased+missing') ((1, 1, 7), (((0, 2, 'W'),), 'absent', 'x_u@0', 5, 2), ['duplicate_unreleased', 'missing'])」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:133`)
- 行 135：「example ('partial_ne_hidden', 'missing') ((2, 2, 7), (((0, 1, 'Q'), (1, 2, 'W')), 'absent+u1', 'x_r4@3', 3, 3), ['missing', 'span_wrong'])」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:135`)
- 行 139：「example ('partial_ne_hidden', 'released_over_unit') ((1, 2, 7), (((0, 2, 'W'),), 'u1', 'x_r4@1', 5, 2), ['released_over_unit', 'span_wrong'])」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:139`)
- 行 140：「example ('partial_ne_hidden', 'stray_unreleased') ((2, 3, 7), (((0, 1, 'W'), (2, 2, 'W')), 'u2+u1', 'x_u@3', 5, 3), ['span_wrong', 'stray_unreleased'])」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:140`)

## 二、读数表（Q3：`out/q3-*.txt`，丁全放开那一形）

配置名（「CONFIGS = {」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/q3/ding_today.py:23`) 起的六项）：`today` = 今天的实现；`verdict` = 第一轮判决的口径（第 0 代根只记一种、已知空树、无窗口外一停）；`today_both_fixed` = 前一份 3b 建议的两处都改；`today_genesis_only` = 只去掉「空着」那一种；`today_known_genesis` = 只把第 0 代根的树表记成已知空树；`today_no_stop` = 今天的实现去掉窗口外一停。后三种是本腿为拆开两处改法的作用多放的格，前一份没写预测，下面标「没预测」。

预测（前一份第六节）：合法写者 `red` 全是 0；坏写者 `today_both_fixed` 的 `red` 不少于 `today`；`cfg=verdict` 的合法 `partial_ratio` 接近第一轮的 8.3%（S=4）、2.4%（S=8）；`q3enum` 在 `p_rootfail=0` 的格 `NOT_COVERED=0`、`p_rootfail=0.03` 的格大于 0；`q3stop` 没有 `DIFFERENT_*`。

### 合法写者，S=4，J=128，`p_rootfail=0`（`q3-admission-4-128.txt`）

- 行 1，`today`：「q3 writer=admission S=4 J=128 p_rootfail=0.0 cfg=today legal_raise_roots=15627 reaches_ding=13876 final[holds=13922 red=0 partial=1348 not_judged=357] partial_why[ambiguous:1348] partial_ratio=0.0863」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-128.txt:1`)　→ `red=0` 一致；`partial_ratio=0.0863`
- 行 2，`verdict`：「q3 writer=admission S=4 J=128 p_rootfail=0.0 cfg=verdict legal_raise_roots=15627 reaches_ding=13876 final[holds=13977 red=0 partial=1293 not_judged=357] partial_why[ambiguous:1293] partial_ratio=0.0827」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-128.txt:2`)　→ `partial_ratio=0.0827`，预测「接近 8.3%」一致（逐位等于第一轮 1293）
- 行 3，`today_known_genesis`（没预测）：「q3 writer=admission S=4 J=128 p_rootfail=0.0 cfg=today_known_genesis legal_raise_roots=15627 reaches_ding=13876 final[holds=13977 red=0 partial=1293 not_judged=357] partial_why[ambiguous:1293] partial_ratio=0.0827」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-128.txt:3`)　→ 只改第二处就到 1293，与 `verdict` 同数
- 行 4，`today_genesis_only`（没预测）：「q3 writer=admission S=4 J=128 p_rootfail=0.0 cfg=today_genesis_only legal_raise_roots=15627 reaches_ding=13876 final[holds=13922 red=0 partial=1348 not_judged=357] partial_why[ambiguous:1335,between:13] partial_ratio=0.0863」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-128.txt:4`)　→ 只改第一处，partial 与 `today` 同为 1348，原因里多出 `between:13`
- 行 5，`today_both_fixed`：「q3 writer=admission S=4 J=128 p_rootfail=0.0 cfg=today_both_fixed legal_raise_roots=15627 reaches_ding=13876 final[holds=13977 red=0 partial=1293 not_judged=357] partial_why[ambiguous:1293] partial_ratio=0.0827」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-128.txt:5`)　→ `red=0` 一致；与 `verdict` 同数
- 行 6，`today_no_stop`（没预测）：「q3 writer=admission S=4 J=128 p_rootfail=0.0 cfg=today_no_stop legal_raise_roots=15627 reaches_ding=13876 final[holds=13922 red=0 partial=1348 not_judged=357] partial_why[ambiguous:1348] partial_ratio=0.0863」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-128.txt:6`)　→ 与 `today` 同数
- 行 7，`q3enum`：「q3enum writer=admission S=4 J=128 p_rootfail=0.0 legal slots_overwritten_since_the_raise=85060 covered=85060 NOT_COVERED=0 not_recognized=0」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-128.txt:7`)　→ `NOT_COVERED=0` 一致
- 行 8，`q3stop`：「q3stop writer=admission S=4 J=128 p_rootfail=0.0 legal same=15627」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-128.txt:8`)　→ 没有 `DIFFERENT_*` 一致

### 合法写者，S=8，J=128，`p_rootfail=0`（`q3-admission-8-128.txt`）

- 行 1，`today`：「q3 writer=admission S=8 J=128 p_rootfail=0.0 cfg=today legal_raise_roots=25678 reaches_ding=12717 final[holds=24402 red=0 partial=976 not_judged=300] partial_why[combos:5,ambiguous:971] partial_ratio=0.0380」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-128.txt:1`)　→ `red=0` 一致；`partial_ratio=0.0380`，`combos:5`
- 行 2，`verdict`：「q3 writer=admission S=8 J=128 p_rootfail=0.0 cfg=verdict legal_raise_roots=25678 reaches_ding=12717 final[holds=24774 red=0 partial=604 not_judged=300] partial_why[combos:3,ambiguous:601] partial_ratio=0.0235」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-128.txt:2`)　→ `partial_ratio=0.0235`，预测「接近 2.4%」一致（逐位等于第一轮 604）
- 行 3，`today_known_genesis`（没预测）：「q3 writer=admission S=8 J=128 p_rootfail=0.0 cfg=today_known_genesis legal_raise_roots=25678 reaches_ding=12717 final[holds=24772 red=0 partial=606 not_judged=300] partial_why[combos:5,ambiguous:601] partial_ratio=0.0236」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-128.txt:3`)　→ 606，比 `verdict` 多 2（多的是 `combos:5` 对 `combos:3`）
- 行 4，`today_genesis_only`（没预测）：「q3 writer=admission S=8 J=128 p_rootfail=0.0 cfg=today_genesis_only legal_raise_roots=25678 reaches_ding=12717 final[holds=24404 red=0 partial=974 not_judged=300] partial_why[combos:3,ambiguous:611,between:360] partial_ratio=0.0379」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-128.txt:4`)　→ 974，与 `today` 的 976 几乎相同，原因里多出 `between:360`
- 行 5，`today_both_fixed`：「q3 writer=admission S=8 J=128 p_rootfail=0.0 cfg=today_both_fixed legal_raise_roots=25678 reaches_ding=12717 final[holds=24774 red=0 partial=604 not_judged=300] partial_why[combos:3,ambiguous:601] partial_ratio=0.0235」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-128.txt:5`)　→ `red=0` 一致；与 `verdict` 同数
- 行 6，`today_no_stop`（没预测）：「q3 writer=admission S=8 J=128 p_rootfail=0.0 cfg=today_no_stop legal_raise_roots=25678 reaches_ding=12717 final[holds=24402 red=0 partial=976 not_judged=300] partial_why[combos:5,ambiguous:971] partial_ratio=0.0380」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-128.txt:6`)　→ 与 `today` 同数
- 行 7，`q3enum`：「q3enum writer=admission S=8 J=128 p_rootfail=0.0 legal slots_overwritten_since_the_raise=190088 covered=190088 NOT_COVERED=0 not_recognized=0」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-128.txt:7`)　→ `NOT_COVERED=0` 一致
- 行 8，`q3stop`：「q3stop writer=admission S=8 J=128 p_rootfail=0.0 legal same=25678」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-128.txt:8`)　→ 没有 `DIFFERENT_*` 一致

### 坏写者（非法抬 F），S=4，J=128（`q3-bad-4-128.txt`）

- 行 1，`today`：「q3 writer=bad S=4 J=128 p_rootfail=0.0 cfg=today illegal_raise_roots=14382 reaches_ding=10942 final[holds=0 red=13225 partial=838 not_judged=319] partial_why[ambiguous:838] partial_ratio=0.0583」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-4-128.txt:1`)
- 行 2，`verdict`：「q3 writer=bad S=4 J=128 p_rootfail=0.0 cfg=verdict illegal_raise_roots=14382 reaches_ding=10942 final[holds=0 red=13232 partial=831 not_judged=319] partial_why[ambiguous:831] partial_ratio=0.0578」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-4-128.txt:2`)
- 行 3，`today_known_genesis`（没预测）：「q3 writer=bad S=4 J=128 p_rootfail=0.0 cfg=today_known_genesis illegal_raise_roots=14382 reaches_ding=10942 final[holds=0 red=13225 partial=838 not_judged=319] partial_why[ambiguous:838] partial_ratio=0.0583」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-4-128.txt:3`)　→ 与 `today` 同数：只改第二处，红不增
- 行 4，`today_genesis_only`（没预测）：「q3 writer=bad S=4 J=128 p_rootfail=0.0 cfg=today_genesis_only illegal_raise_roots=14382 reaches_ding=10942 final[holds=0 red=13230 partial=833 not_judged=319] partial_why[ambiguous:833] partial_ratio=0.0579」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-4-128.txt:4`)　→ 红 +5
- 行 5，`today_both_fixed`：「q3 writer=bad S=4 J=128 p_rootfail=0.0 cfg=today_both_fixed illegal_raise_roots=14382 reaches_ding=10942 final[holds=0 red=13232 partial=831 not_judged=319] partial_why[ambiguous:831] partial_ratio=0.0578」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-4-128.txt:5`)　→ 红 13232 对 `today` 的 13225，多 7，预测「不少于」一致；`holds=0` 在六个格全是 0
- 行 6，`today_no_stop`（没预测）：「q3 writer=bad S=4 J=128 p_rootfail=0.0 cfg=today_no_stop illegal_raise_roots=14382 reaches_ding=10942 final[holds=0 red=13225 partial=838 not_judged=319] partial_why[ambiguous:838] partial_ratio=0.0583」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-4-128.txt:6`)
- 行 7，`q3enum`：「q3enum writer=bad S=4 J=128 p_rootfail=0.0 illegal slots_overwritten_since_the_raise=72743 covered=72743 NOT_COVERED=0 not_recognized=0」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-4-128.txt:7`)　→ `NOT_COVERED=0` 一致
- 行 8，`q3stop`：「q3stop writer=bad S=4 J=128 p_rootfail=0.0 illegal same=14382」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-4-128.txt:8`)　→ 没有 `DIFFERENT_*` 一致

### 坏写者，S=8，J=128（`q3-bad-8-128.txt`）

- 行 1，`today`：「q3 writer=bad S=8 J=128 p_rootfail=0.0 cfg=today illegal_raise_roots=23360 reaches_ding=9185 final[holds=0 red=22765 partial=333 not_judged=262] partial_why[combos:3,ambiguous:330] partial_ratio=0.0143」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-8-128.txt:1`)
- 行 2，`verdict`：「q3 writer=bad S=8 J=128 p_rootfail=0.0 cfg=verdict illegal_raise_roots=23360 reaches_ding=9185 final[holds=0 red=22768 partial=330 not_judged=262] partial_why[combos:1,ambiguous:329] partial_ratio=0.0141」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-8-128.txt:2`)
- 行 3，`today_known_genesis`（没预测）：「q3 writer=bad S=8 J=128 p_rootfail=0.0 cfg=today_known_genesis illegal_raise_roots=23360 reaches_ding=9185 final[holds=0 red=22765 partial=333 not_judged=262] partial_why[combos:3,ambiguous:330] partial_ratio=0.0143」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-8-128.txt:3`)　→ 与 `today` 同数
- 行 4，`today_genesis_only`（没预测）：「q3 writer=bad S=8 J=128 p_rootfail=0.0 cfg=today_genesis_only illegal_raise_roots=23360 reaches_ding=9185 final[holds=0 red=22767 partial=331 not_judged=262] partial_why[combos:1,ambiguous:330] partial_ratio=0.0142」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-8-128.txt:4`)　→ 红 +2
- 行 5，`today_both_fixed`：「q3 writer=bad S=8 J=128 p_rootfail=0.0 cfg=today_both_fixed illegal_raise_roots=23360 reaches_ding=9185 final[holds=0 red=22768 partial=330 not_judged=262] partial_why[combos:1,ambiguous:329] partial_ratio=0.0141」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-8-128.txt:5`)　→ 红 22768 对 22765，多 3，预测「不少于」一致；`holds=0` 六个格全是 0
- 行 6，`today_no_stop`（没预测）：「q3 writer=bad S=8 J=128 p_rootfail=0.0 cfg=today_no_stop illegal_raise_roots=23360 reaches_ding=9185 final[holds=0 red=22765 partial=333 not_judged=262] partial_why[combos:3,ambiguous:330] partial_ratio=0.0143」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-8-128.txt:6`)
- 行 7，`q3enum`：「q3enum writer=bad S=8 J=128 p_rootfail=0.0 illegal slots_overwritten_since_the_raise=146341 covered=146341 NOT_COVERED=0 not_recognized=0」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-8-128.txt:7`)　→ `NOT_COVERED=0` 一致
- 行 8，`q3stop`：「q3stop writer=bad S=8 J=128 p_rootfail=0.0 illegal same=23360」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-8-128.txt:8`)　→ 没有 `DIFFERENT_*` 一致

### 合法写者，J=32（日志只留 32 行，前一份没写预测的两格；`partial_ratio` 随 J 剧烈变化，见第三节 Q3.3c）

`q3-admission-4-32.txt`（S=4）：

- 行 1，`today`：「q3 writer=admission S=4 J=32 p_rootfail=0.0 cfg=today legal_raise_roots=15627 reaches_ding=13876 final[holds=13627 red=0 partial=1643 not_judged=357] partial_why[journal0:492,ambiguous:1151] partial_ratio=0.1051」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-32.txt:1`)
- 行 2，`verdict`：「q3 writer=admission S=4 J=32 p_rootfail=0.0 cfg=verdict legal_raise_roots=15627 reaches_ding=13876 final[holds=13256 red=0 partial=2014 not_judged=357] partial_why[journal0:1051,ambiguous:963] partial_ratio=0.1289」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-32.txt:2`)
- 行 3，`today_known_genesis`：「q3 writer=admission S=4 J=32 p_rootfail=0.0 cfg=today_known_genesis legal_raise_roots=15627 reaches_ding=13876 final[holds=13681 red=0 partial=1589 not_judged=357] partial_why[journal0:492,ambiguous:1097] partial_ratio=0.1017」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-32.txt:3`)
- 行 4，`today_genesis_only`：「q3 writer=admission S=4 J=32 p_rootfail=0.0 cfg=today_genesis_only legal_raise_roots=15627 reaches_ding=13876 final[holds=13627 red=0 partial=1643 not_judged=357] partial_why[journal0:492,ambiguous:1138,between:13] partial_ratio=0.1051」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-32.txt:4`)
- 行 5，`today_both_fixed`：「q3 writer=admission S=4 J=32 p_rootfail=0.0 cfg=today_both_fixed legal_raise_roots=15627 reaches_ding=13876 final[holds=13681 red=0 partial=1589 not_judged=357] partial_why[journal0:492,ambiguous:1097] partial_ratio=0.1017」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-32.txt:5`)
- 行 6，`today_no_stop`：「q3 writer=admission S=4 J=32 p_rootfail=0.0 cfg=today_no_stop legal_raise_roots=15627 reaches_ding=13876 final[holds=13202 red=0 partial=2068 not_judged=357] partial_why[journal0:1051,ambiguous:1017] partial_ratio=0.1323」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-32.txt:6`)
- 行 7，`q3enum`：「q3enum writer=admission S=4 J=32 p_rootfail=0.0 legal slots_overwritten_since_the_raise=85060 covered=84483 NOT_COVERED=0 not_recognized=577」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-32.txt:7`)　→ `NOT_COVERED=0`；`not_recognized=577` 是丁在那一槽返回错（「if isinstance(got, tuple):」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/q3/ding_today.py:202`) 交回 `None`）、不计入覆盖
- 行 8，`q3stop`：「q3stop writer=admission S=4 J=32 p_rootfail=0.0 legal DIFFERENT_partial_vs_partial=134 same=15068 stop_resolves_what_no_stop_leaves_partial_journal0=425」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-32.txt:8`)　→ 字面上出现了 `DIFFERENT_*`（134），分析见第三节 Q3.2
- 行 9，`example`：「example ('stop_different', 'legal') (168, 38, {'floor_before': 13, 'final': ('partial', 'ambiguous'), 'window': 'holds_undetermined'}, {'floor_before': 13, 'final': ('partial', 'journal0'), 'window': 'holds_undetermined'})」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-32.txt:9`)

`q3-admission-8-32.txt`（S=8）：

- 行 1，`today`：「q3 writer=admission S=8 J=32 p_rootfail=0.0 cfg=today legal_raise_roots=25678 reaches_ding=12717 final[holds=16082 red=0 partial=9296 not_judged=300] partial_why[journal0:8878,ambiguous:418] partial_ratio=0.3620」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-32.txt:1`)
- 行 2，`verdict`：「q3 writer=admission S=8 J=32 p_rootfail=0.0 cfg=verdict legal_raise_roots=25678 reaches_ding=12717 final[holds=15042 red=0 partial=10336 not_judged=300] partial_why[journal0:10249,ambiguous:87] partial_ratio=0.4025」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-32.txt:2`)
- 行 3，`today_known_genesis`：「q3 writer=admission S=8 J=32 p_rootfail=0.0 cfg=today_known_genesis legal_raise_roots=25678 reaches_ding=12717 final[holds=16351 red=0 partial=9027 not_judged=300] partial_why[journal0:8878,ambiguous:149] partial_ratio=0.3515」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-32.txt:3`)
- 行 4，`today_genesis_only`：「q3 writer=admission S=8 J=32 p_rootfail=0.0 cfg=today_genesis_only legal_raise_roots=25678 reaches_ding=12717 final[holds=16082 red=0 partial=9296 not_judged=300] partial_why[journal0:8878,ambiguous:152,between:266] partial_ratio=0.3620」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-32.txt:4`)
- 行 5，`today_both_fixed`：「q3 writer=admission S=8 J=32 p_rootfail=0.0 cfg=today_both_fixed legal_raise_roots=25678 reaches_ding=12717 final[holds=16351 red=0 partial=9027 not_judged=300] partial_why[journal0:8878,ambiguous:149] partial_ratio=0.3515」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-32.txt:5`)
- 行 6，`today_no_stop`：「q3 writer=admission S=8 J=32 p_rootfail=0.0 cfg=today_no_stop legal_raise_roots=25678 reaches_ding=12717 final[holds=14773 red=0 partial=10605 not_judged=300] partial_why[journal0:10249,ambiguous:356] partial_ratio=0.4130」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-32.txt:6`)
- 行 7，`q3enum`：「q3enum writer=admission S=8 J=32 p_rootfail=0.0 legal slots_overwritten_since_the_raise=190088 covered=128249 NOT_COVERED=0 not_recognized=61839」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-32.txt:7`)　→ `NOT_COVERED=0`；`not_recognized=61839`，占 190088 的 32.5%
- 行 8，`q3stop`：「q3stop writer=admission S=8 J=32 p_rootfail=0.0 legal DIFFERENT_partial_vs_partial=62 same=24307 stop_resolves_what_no_stop_leaves_partial_journal0=1309」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-32.txt:8`)　→ 字面上出现了 `DIFFERENT_*`（62）
- 行 9，`example`：「example ('stop_different', 'legal') (230, 43, {'floor_before': 14, 'final': ('partial', 'ambiguous'), 'window': 'holds_undetermined'}, {'floor_before': 14, 'final': ('partial', 'journal0'), 'window': 'holds_undetermined'})」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-32.txt:9`)

### 根槽写失败而同一实例接着发（`p_rootfail=0.03`，D23 第 406 行那一段落地之后的写法；今天的写者走不到）

`q3-rootfail-4-128.txt`：

- 行 1，`today`：「q3 writer=admission S=4 J=128 p_rootfail=0.03 cfg=today legal_raise_roots=15596 reaches_ding=13611 final[holds=13446 red=216 partial=1292 not_judged=642] partial_why[ambiguous:1292] partial_ratio=0.0828」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-rootfail-4-128.txt:1`)　→ 合法抬 F 根里有 216 个被判红（216/15596 = 1.4%，本腿算的）
- 行 2，`today_no_stop`：「q3 writer=admission S=4 J=128 p_rootfail=0.03 cfg=today_no_stop legal_raise_roots=15596 reaches_ding=13611 final[holds=13446 red=216 partial=1292 not_judged=642] partial_why[ambiguous:1292] partial_ratio=0.0828」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-rootfail-4-128.txt:2`)
- 行 3，`q3enum`：「q3enum writer=admission S=4 J=128 p_rootfail=0.03 legal slots_overwritten_since_the_raise=81147 covered=79061 NOT_COVERED=2086 not_recognized=0」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-rootfail-4-128.txt:3`)　→ `NOT_COVERED=2086`，预测「大于 0」一致
- 行 4，`q3stop`：「q3stop writer=admission S=4 J=128 p_rootfail=0.03 legal same=15596」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-rootfail-4-128.txt:4`)
- 行 5，`example`：「example ('not_covered', 'legal') (4, 35, 0, 2, 18, 2, {1: 2, 2: 22, 3: 25, 4: 35}, [11, 30])」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-rootfail-4-128.txt:5`)

`q3-rootfail-8-128.txt`：

- 行 1，`today`：「q3 writer=admission S=8 J=128 p_rootfail=0.03 cfg=today legal_raise_roots=25829 reaches_ding=12830 final[holds=24077 red=246 partial=955 not_judged=551] partial_why[combos:4,ambiguous:951] partial_ratio=0.0370」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-rootfail-8-128.txt:1`)　→ 246 个合法抬 F 根被判红（246/25829 = 0.95%，本腿算的）
- 行 2，`today_no_stop`：「q3 writer=admission S=8 J=128 p_rootfail=0.03 cfg=today_no_stop legal_raise_roots=25829 reaches_ding=12830 final[holds=24077 red=246 partial=955 not_judged=551] partial_why[combos:4,ambiguous:951] partial_ratio=0.0370」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-rootfail-8-128.txt:2`)
- 行 3，`q3enum`：「q3enum writer=admission S=8 J=128 p_rootfail=0.03 legal slots_overwritten_since_the_raise=185603 covered=181882 NOT_COVERED=3721 not_recognized=0」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-rootfail-8-128.txt:3`)　→ `NOT_COVERED=3721`，预测「大于 0」一致
- 行 4，`q3stop`：「q3stop writer=admission S=8 J=128 p_rootfail=0.03 legal same=25829」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-rootfail-8-128.txt:4`)
- 行 5，`example`：「example ('not_covered', 'legal') (4, 35, 2, 3, None, 2, {1: 2, 2: 22, 3: 25, 4: 35}, [11, 30])」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-rootfail-8-128.txt:5`)

小结（预测对读数）：合法 `red` 在 `p_rootfail=0` 的全部格是 0（一致）；坏写者 `today_both_fixed` 红多于 `today`（一致，+7 与 +3）；`cfg=verdict` 的合法 `partial_ratio` 8.27% 与 2.35%（一致，且逐位等于第一轮）；`NOT_COVERED` 两头都对（一致）；`q3stop` 在 J=128 无 `DIFFERENT_*`（一致）、在 J=32 字面不一致（实质一致，见 Q3.2）。

## 三、各问现在的结论

前一份的结论叫「前一份」，现在的叫「现在」。每条写「什么现象会推翻它」。

### Q1.1　改记读法本身漏不漏（含 `!=` 对 `>`）

现在：拆成三句，前一份那一句整体不成立。

1. **站得住（量的）**：部分没判的盘面里 I-3.18 全红：「name=partial_ne_i318_red {True: 50406}」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:45`)。健康盘面（`kinds` 为空）在三种读法下全是 `holds`（`healthy_but_not_holds` 没有行）。所以部分没判不会让任何一个盘面在池级变绿，也没有一个健康盘面被它误红。推翻它的现象：`partial_ne_i318_red` 出现 `False`，或 `healthy_but_not_holds` 出现行。
2. **被推翻（量的 + 推的）**：前一份写的「造不出部分没判、盘上又有 I-3.18 之外的病、别的已建不变量都不红」。读数是 11650 个盘面（占部分没判的 23.1%）：十二个标签行加总见第一节小表。按标签含什么拆（行数加总，本腿算的，互不重叠）：含 `missing` 的 6326（60+3840+1152+720+100+120+334）；不含 `missing`、含 `released_live` 的 4276（4176+100）；其余含 `released_over_unit` 的 844（204+640）；`stray_unreleased` 204。再读模型源码与最小例（推的；最小例只是每个标签里最小的一个，整批是不是都这样，输出没拆）：
   - 含 `missing` 的 6326：最小例（行 130、行 135）里缺记录的是 Q 单元。模型的 Q 是「没人引用、记录带隔离记号」，没有记录就没有记号，不成其为单元；模型却把它记进应有的已分配（「a_true = len(ctx.walked) + sum(c for (s, c, k) in units if k == 'Q')」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/q1q2/q1q2_unit_class_reading.py:154`)），所以又报 `accounting_off`。行 133 的 `duplicate_unreleased+missing` 是另一种假病：该单元唯一的一条记录是通过额外记录通道放进来的、起点正好在单元起点，模型把它同时标「重复」和「缺记录」（「if extra[0] in ctx.classes and not extra[3]:」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/q1q2/q1q2_unit_class_reading.py:176`)），真病只是跨度错。这两种是模型造法的毛病，不是改记读法的洞。
   - `released_live` 的 4276：单元的记录是已释放、释放代高于 θ_high，I-3.1 按「该算已分配」数它、所以成立。kb 点名了这一形：I-3.9 写「若每一条有效根都还引用它，它不许带已释放标志」(`crates/singlefs-checker/invariants/05-空间记账.md:16`)；I-3.11 补的正是「写者把活单元记成已释放、放进 defer，已分配与空闲两个和照样对得上」(`crates/singlefs-checker/invariants/05-空间记账.md:18`)。这两条模型都没建（模型源码文件头第 20 行写「没建：I-3.10、I-3.11、I-3.16、I-5.2」，I-3.9 也不在模型的判里），所以「别的已建不变量都不红」在真 checker 上对这一类不成立（推的，要用真 checker 的坏镜像核）。
   - 剩下 844 + 204 = 1048 个（占部分没判的 2.1%）是本腿认为真有洞的一形：**窄记录加尾槽上另一条记录**。最小例行 139：二槽单元的记录只跨 1 槽，另有一条已释放记录压在它的尾槽；行 140：宽记录、窄记录、一条未释放记录落在窄记录那个单元的尾槽。自述的跨度互不相交，I-5.4 看不见；改记之后窄记录变宽、与尾槽那条重叠，但尾槽在 Ω 里，逐槽上沿判不了，三判也成立，所以部分没判。池级仍红（I-3.18），但 I-3.1 这一格说「判不了」，没人点名尾槽那条记录；I-3.15 对尾槽没有对象（「头用不了的那一格没有对象」(`crates/singlefs-checker/invariants/05-空间记账.md:22`)）。这一形碰到的正是第二轮判决 Q9 的后半句：「那块畸形若没有别的不变量判红，那一处判红、不报部分没判」(`research/prompts/m3-close-b19-design-r2-main-verification.md:25`)——尾槽上那条多出来的记录没有别的不变量点名，I-3.1 却报了部分没判（Q9 说的是 C701 那十七处的口径，I-3.1 取宽这一支是不是其中一处按同族算，是推的）。这一形与判决里第二次抽样的窄向镜像（泄漏记录罩全零槽 63）不同：那一形泄漏槽在 Ω 之外，逐槽上沿能判；这一形泄漏槽在窄记录自己的单元里，判不了。
3. **对替代方案**：不取宽（`none`）下这些盘面全判红，不藏任何东西（「name=verdict mode=none holds=74382 partial=0 red=567226」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:6`)），代价是失去判决 Q9 那一条「配不上的那块畸形别的不变量判红时，那一处报部分没判」。`>` 对窄记录的改记与 `!=` 是同一份函数，窄向这一形（1048 个）两者同样打中（推的）；模型没有按读法分开打印「藏起来」的盘面数，所以不能说 `>` 比 `!=` 干净。
4. **一个收严的建议（被攻过零轮，推的，没造模型、没跑）**：取宽那一支里，改记之后再加一判「改记那一份里任意两条记录罩住的槽互不相交」。预期：行 139、行 140 两个最小例变红；`only_span_wrong` 的部分没判盘面（17056 个，「name=only_span_wrong mode=ne verdict=partial worlds=17056」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:11`)）不变，因为那些盘面里每个单元只有一条记录，改记之后就是单元自己占的槽，互不相交；健康盘面不进这一支。推翻它的现象：加了这一判之后，某个合法写者镜像（含崩溃点重放的崩溃态）判红，或 `only_span_wrong` 的部分没判盘面数减少。

**`!=` 对 `>`（附）现在**：判决否 `>` 的理由（「只挡宽方向，对窄方向那一形不起作用」）是对旧的逐判混取说的，对整份改记读法够不着（窄向同一份改记）；`!=` 站得住的理由仍是判决 Q9 的条件，不是「不挡窄向」；但 Q9 的条件有两半，前半（别的不变量判红时报部分没判）支持 `!=` 报部分没判，后半（没有别的不变量判红就判红）正是上面 1048 个盘面该判红的依据，所以 Q9 不为 `!=` 站台到底。量的差别只在宽记录上：`only_span_wrong` 下 `!=` 部分没判 17056（「name=only_span_wrong mode=ne verdict=partial worlds=17056」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:11`)）、`>` 部分没判 2996（「name=only_span_wrong mode=gt verdict=partial worlds=2996」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:8`)），差 14060，等于 「name=partial_ne_but_gt_says ('red', 'only_span_wrong') 14060」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:47`) 里「只有跨度错」那一行。前一份写的「`partial_ne_but_gt_says` 的 `other_kinds` 行出现就推翻 `!=`」按字面打中（26752），但这一行把别的不变量已点名的盘面也数进去了（`partial_ne` 里 `named_by_another_invariant` 的其余行合计 21700），不是「藏起来」的数；严格的数（`!=` 部分没判、`>` 判红、且只有 I-3.18 点名）输出里没有，上界是 11650。另有 1064 个盘面 `>` 部分没判而 `!=` 判红（「name=partial_gt_but_ne_says ('red',) 1064」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q1q2.txt:46`)），所以两者互有出入，谁也不支配谁。推翻「谁也不支配谁」：按读法分开打印的「藏起来」盘面数显示一方为另一方的子集。

### Q1.2　合法写者的镜像进不进这一支

现在：站得住，量的部分只有一半。量的：健康盘面三种读法全是 `holds`（`healthy_but_not_holds` 无行），所以任何三判成立的盘面不进取宽那一支。推的：合法写者镜像上 I-3.18 恒绿（前一份第二节第 2 问的论证，没动）。模型的世界不含崩溃点重放的崩溃态，这一半仍是推的。推翻：任一合法写者镜像（含崩溃态）上 I-3.18 红。

### Q1.3　部分没判说明里的数

现在：不变，仍是推的。模型没有打印说明里的数，本轮读数帮不上。前一份第 104 行写的推翻现象（改记前后下沿之差小于真被改记的槽数）没有对应输出。若采纳上面第 4 点那一判，说明里还要补「改记后重叠的是哪两条」。

### Q2.1　合法写者写得出「走到的单元起点上没有未释放记录」吗

现在：不变，仍是推的（读 walk.rs 与 kb，模型没有这一项）。

### Q2.2　C944 归哪条

现在：推荐新立一条（暂名 I-3.21，最新根范围）不变；**优先级不升**。这一条要先交代一次输。

- **记一次输（最弱读法）**：前一份的跑前规则是「`c944_pure` 为空 ⇒ γ 是诊断条，可排在 GPU 对拍之后；有行 ⇒ 缺口是真的，优先级升」。输出是有行：十二个标签、18239 个盘面，三种读法逐行同数（第一节小表）。按字面规则打中，优先级该升。若主 agent 不采下面的读法，就按「升」办。
- **我的读法（推的）**：这条规则的判据是「小世界里有 holds / partial 的盘面且已建不变量都不红」，满足这个字面的盘面不全是 C944 的病形。C944 的病形是「走到的单元起点没有自己的未释放记录，而邻居记录的跨度罩着它」。十二个标签每一个都含 `missing` 或 `released_live`：含 `missing` 的最小例是 Q 单元没有记录（行 93、行 100，假病，见 Q1.1 第 2 点）或单元的记录经额外记录通道进来（行 98，真实情形是一条正常的未释放记录），不含的 `released_live`（行 104）是已释放记录，kb 的 I-3.9（第 16 行）与 I-3.11（第 18 行）点名，模型没建这两条。三种读法逐行同数说明它与取宽无关，是 I-3.1 本来就判成立的盘面，不是改记读法带出来的。
- **C944 病形本身**：邻居记录若是宽记录（单元起点处的跨度大于那一类）则 I-3.18 红，被 `c944_pure` 的「别的已建不变量都不红」条件排除；邻居记录若起点在头用不了的槽上（模型的额外记录只有一槽宽，造不出），它多罩的槽在 Ω 之外，I-3.1 的上沿（遍历加 defer 加隔离）红，我手算过一遍（推的）。所以模型这一版并没有扫到 C944 的病形，不能拿它的非空去升优先级，也不能拿它的非空去证明缺口为空。
- 推翻我的读法的现象：模型里补上 I-3.9 与 I-3.11 之后 `released_live` 的盘面仍全绿（则 γ 有独立价值，优先级升）；或造出一个 W 单元起点无任何记录、无已释放记录、且 I-3.18 / I-5.4 / I-3.1 都不红的盘面。

### Q2.3　γ 的字节级坏镜像

现在：不变，仍是预测，没造、没跑。

### Q3.1　丁的枚举全不全

现在：站得住，限定在「丁认得出的槽」。量的：`p_rootfail=0` 的六个格 `NOT_COVERED=0`（四个 J=128 格与两个 J=32 格，行见第二节）：覆盖槽数 85060、72743、190088、146341，J=32 的两格是 84483 与 128249，另有 577 与 61839 个槽丁自己报错、不计入。核不是空核：`p_rootfail=0.03` 的两格 `NOT_COVERED` 是 2086 与 3721（「q3enum writer=admission S=4 J=128 p_rootfail=0.03 legal slots_overwritten_since_the_raise=81147 covered=79061 NOT_COVERED=2086 not_recognized=0」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-rootfail-4-128.txt:3`)；「q3enum writer=admission S=8 J=128 p_rootfail=0.03 legal slots_overwritten_since_the_raise=185603 covered=181882 NOT_COVERED=3721 not_recognized=0」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-rootfail-8-128.txt:3`)）。限定：J=32 下 S=8 有 32.5% 的槽丁认不出（日志里没有那一实例的行，丁返回错，I-7.9 既不判成立也不判红），覆盖结论对这些槽没有说话。推翻它的现象：J=128 或 `p_rootfail=0` 的格 `NOT_COVERED` 非零，或 `not_recognized` 在 J=128 的格非零（现在是 0）。

### Q3.2　窗口外那一停与「住着无效根」是不是处处等价

现在：实质站得住；字面判据在 J=32 打中，要改判据的措辞。J=128 的四个格 `same` 等于全部（行见第二节）。J=32 的两格出现 `DIFFERENT_partial_vs_partial`：S=4 是 134，S=8 是 62。模型源码里这个键的意思是今天与不停两边的判词都是部分没判、只是原因标签不同（「elif b[0] == 'partial' and a[0] != 'partial':」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/q3/q3_today_run.py:79`) 是「停救回来的」，其余落进「tally[('stop', key, 'DIFFERENT_%s_vs_%s' % (a[0], b[0]))] += 1」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/q3/q3_today_run.py:82`)）。最小例是 「example ('stop_different', 'legal') (168, 38, {'floor_before': 13, 'final': ('partial', 'ambiguous'), 'window': 'holds_undetermined'}, {'floor_before': 13, 'final': ('partial', 'journal0'), 'window': 'holds_undetermined'})」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-32.txt:9`)：今天部分没判（原因 `ambiguous`），不停也部分没判（原因 `journal0`）。前一份写的是「两边都判得出结论时判词不同」，两边都不是结论，所以实质没打中；是我把推翻现象写成「`DIFFERENT_*` 非零」太宽。改成：`DIFFERENT_` 后面两个判词不都是 `partial`，才推翻。另一向是净收益：停救回来的盘面 425 个（S=4）与 1309 个（S=8），键 `stop_resolves_what_no_stop_leaves_partial_journal0`（行 8，两个 J=32 文件）。

### Q3.3a　第 0 代根的树表记「判不了」会不会让本该判红的环落到部分没判

现在：站得住，量级很小。量的：坏写者（非法抬 F）`today` 的 `partial` 减去 `today_both_fixed` 的 `partial`，S=4 是 838 减 831 等于 7，S=8 是 333 减 330 等于 3，与 `red` 的增量（+7、+3）逐个相等：多出来的红全部来自原来的部分没判。占非法抬 F 根的 7/14382 = 0.05% 与 3/23360 = 0.013%。`holds` 在坏写者的全部格里是 0，所以落到部分没判不会翻成绿（漏红不是误绿）。前一份手算的盘面（年轻池第一圈内）这一类存在，但在这批历史里一万多个非法根中只占几个。推翻它的现象：`today_both_fixed` 在坏写者上 `holds` 非零。注意模型的状态约定：历史生成器里状态 0 既当空树又在写者一侧数成非空（前一份第七节最后第二条），读数只比相对差。

### Q3.3b　用 mkfs 树表 0 条判，改哪两处

现在：前一份「改两处」要加一句：两处各管一件事，要同批改。拆开的量（四个 J=128 格）：
- 只改第二处（9030，第 0 代根的树表记已知空树）= `today_known_genesis`：合法 `partial` 1348 到 1293（−55，S=4）、976 到 606（−370，S=8）；坏写者红不变（13225、22765）。
- 只改第一处（9022，槽 0 且抬之前 F 为 0 时不记「空着」）= `today_genesis_only`：合法 `partial` 1348 到 1348、976 到 974，几乎不降，原因里多出 `between:13` 与 `between:360`；坏写者红 +5、+2。
- 两处 = `today_both_fixed`：合法 1293 与 604，坏写者红 +7 与 +3（多于两处各自之和 5+0 与 2+0，有交互）。
所以单改第二处拿不到坏写者的红，单改第一处拿不到合法的部分没判下降，两处同批。GPU 的对应改动（前一份第四节 3b：1865–1876 与 1602–1605）没有模型读数，仍是推的。推翻「要同批」：`today_both_fixed` 的合法 `partial` 高于 `today_known_genesis`，或坏写者红低于 `today_genesis_only`（现在两者相反）。

### Q3.3c　4096 与合起来的部分没判比例

现在：
- 4096 几乎不咬（量的）：合法写者 S=8 J=128 `today` 的 `combos:5`（5/25678 = 0.019%），`verdict` 的 `combos:3`；S=4 没有 `combos`；坏写者 S=8 `today` 的 `combos:3`，`verdict` 的 `combos:1`（行见第二节）。
- 今天实现的部分没判比例（量的，J=128，合法抬 F 根）：S=4 是 8.63%，S=8 是 3.80%；`verdict` 口径是 8.27% 与 2.35%（逐位等于第一轮 `q2ext`）；两处都改后与 `verdict` 同数。今天比 `verdict` 高 0.36 与 1.45 个百分点，与前一份预测的方向一致（今天多出的两处保守）。判决里「保守取法 30.0%–54.0%」描述的是改动之前的实现，不是这个模型里的 `today`（后者照开工快照的 walk.rs 移植，推的：该快照已是全放开那一形加两处保守）。
- **没预测、但要读：比例是 J 的强函数**。J=32 下 `today` 是 10.51%（S=4）与 36.20%（S=8），`verdict` 是 12.89% 与 40.25%；S=8 的 9296 个部分没判里 8878 个是 `journal0`（行见第二节）。第一轮判决引的 5.1% / 2.2%（`q2`）与第一轮攻方 `q2ext` 的 8.3% / 2.4%（本节 `verdict` 逐位复现的那一批）都在 J=128。模型的 J 是日志保留的行数（「self.journal = deque(maxlen=J)」(`research/prompts/m3-close-b19-reason-r2-sonnet-model/q3/history_today.py:58`)）。真池里日志保留多少行、这个 J 在真池上取多少，本腿没核（evidence-discipline 的第六问：这个数是没扫过的参数的函数）。
- 旁读（没预测）：`p_rootfail=0.03` 的合法抬 F 根被判红 1.4%（S=4）与 0.95%（S=8），方向与判决 Q1 第 1 问写的 157/15706、91/28058 一致；两份历史参数不同，不能比大小。
推翻：J=128 的 `partial_ratio` 低于 `verdict` 口径，或 `today_no_stop` 在 J=128 与 `today` 不同数（现在同数）。

### Q3.4　CPU 与 GPU

现在：不变，仍是推的（读代码）；模型不涉及，两张缺的对拍镜像（第 0 代根被抬 F 盖掉；往下认两层以上）照前一份第四节第 4 问。

## 四、一问一个现在的结论（汇总）

| 问 | 现在 | 类别 | 什么现象推翻 |
|---|---|---|---|
| Q1.1 部分没判会不会让池变绿 | 不会：50406 个部分没判盘面 I-3.18 全红，健康盘面全 `holds` | 量的 | `partial_ne_i318_red` 出现 `False` |
| Q1.1 前一份「造不出别的病藏在部分没判里」 | 被推翻：11650/50406，其中约 6326 个多半是模型假病，4276 个是 I-3.9 / I-3.11 的形（模型没建），1048 个是窄记录加尾槽记录，真有洞 | 量的（数）+ 推的（归类） | 按读法分开打印、并建进 I-3.9 / I-3.11 之后，1048 一类不剩 |
| Q1.1 建议 | 取宽那一支加「改记后两两不相交」一判，预期两个最小例变红、`only_span_wrong` 的 17056 不变 | 推的，零轮 | 合法镜像判红，或 17056 减少 |
| Q1.1 附 `!=` 对 `>` | 判决否 `>` 的理由对新读法够不着；`!=` 与 `>` 互有出入，谁也不支配谁；Q9 后半句不为 `!=` 站台 | 量的（差 14060、1064）+ 推的 | 一方的「藏起来」盘面是另一方的子集 |
| Q1.2 合法镜像 | 站得住，一半量的一半推的 | 量的 + 推的 | 合法镜像 I-3.18 红 |
| Q1.3 说明里的数 | 不变 | 推的 | 见前一份 |
| Q2.1 合法来源 | 不变 | 推的 | 见前一份 |
| Q2.2 归属 | 推荐新立一条不变；优先级字面该升，本腿读法不升，主 agent 定 | 量的 + 推的 | 补 I-3.9 / I-3.11 后 `released_live` 仍全绿 |
| Q2.3 字节级镜像 | 不变，没造没跑 | 推的 | 见前一份 |
| Q3.1 枚举 | 站得住，限定于丁认得出的槽 | 量的 | J=128 或 `p_rootfail=0` 的格 `NOT_COVERED` 非零 |
| Q3.2 窗口外一停 | 实质站得住；判据措辞改为 `DIFFERENT_` 后两判词不都是 `partial` | 量的 | 出现 `DIFFERENT_holds_vs_red` 一类 |
| Q3.3a 第 0 代根判不了 | 站得住，量级 7 / 3 个非法根 | 量的 | 坏写者 `holds` 非零 |
| Q3.3b 两处改法 | 要同批，各管一件事 | 量的 | `today_both_fixed` 不优于单改任一处 |
| Q3.3c 4096 与比例 | 4096 不咬（5 个）；J=128 下 8.63% / 3.80%；比例随 J 剧变，J 对真池的取值没核 | 量的 + 一个没核的旋钮 | J=128 的比例低于 `verdict` 口径 |
| Q3.4 CPU 与 GPU | 不变 | 推的 | 见前一份 |

## 五、out/ 参照与复跑

`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/` 里九份输出（从 run1 目录逐字节拷来，`cmp` 全同；模型源码五份 sha256 没变，见第零节）：

| 文件 | 行数 | sha256 |
|---|---|---|
| `out/q1q2.txt` | 140 | 277556cedd6d0fd74bdea4fe0537f119ca699874a34490d70660bac32402fe98 |
| `out/q3-admission-4-128.txt` | 8 | 3f993480c6ab95232400b8be9b3fdd7d62c9f137debec809c9ab6e50d12b890b |
| `out/q3-bad-4-128.txt` | 8 | d833e4c633d7097b230c4b7074cf7677ef532229db32138a9dd8571214113c1d |
| `out/q3-admission-4-32.txt` | 9 | c019f459aa0a3525872ea631be431218ab14467672d5ee84da0b755089f92fa3 |
| `out/q3-rootfail-4-128.txt` | 5 | acd0cb34c89db86fca10ec7d052319880b0c2f001ff4691bb1e0f2be1cafa077 |
| `out/q3-admission-8-128.txt` | 8 | a7d3d2f2e8d6a773bd84c59574d378d39db820b373210af96dcda082ede8e860 |
| `out/q3-bad-8-128.txt` | 8 | 42e799855e9b01fc3e2fc3cd7fad8ab053f2e57f99990455a2db34a3201077de |
| `out/q3-admission-8-32.txt` | 9 | 01d6d17bc90bd3edbd087b5874c29dcd93d750ed31d707e8a97c9d9f6b675d60 |
| `out/q3-rootfail-8-128.txt` | 5 | 24a7d8bc616ee6af52406a53fafc838a10ca57b661c742d9f5660256bdfb81af |

`rerun.sh` 比对用的就是这九份：有 `out/<名>.txt` 就 `cmp`，相同打 `name=<名>_output_identical lines=<行数>`，不同打 `name=<名>_output_differs`。

## 六、要主 agent 跑的命令

**命令一（非重型：纯 python，不起 cargo、不碰 GPU、不进 checker 档；上一次任务 4924 的耗时主 agent 有，本腿不读）**

```
task-server/task submit --cwd /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-sonnet-model -- bash tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-snapshot --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r2-sonnet-run2
```

- 在哪个目录起：`--cwd` 就是模型目录 `/home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-sonnet-model`。`--work` 要不存在或空的新目录（`...-sonnet-run2` 此刻不存在，本腿 `ls` 过）。`--frozen` 这个模型不读。
- 跑完看哪几行：`TASK_SERVER_RESULT` 那一行 `type=succeeded` 与 `exit=0`；`task-server/task log <号>` 里九行都是 `_output_identical`：`name=q1q2_output_identical lines=140`、`name=q3-admission-4-128_output_identical lines=8`、`name=q3-bad-4-128_output_identical lines=8`、`name=q3-admission-4-32_output_identical lines=9`、`name=q3-rootfail-4-128_output_identical lines=5`、`name=q3-admission-8-128_output_identical lines=8`、`name=q3-bad-8-128_output_identical lines=8`、`name=q3-admission-8-32_output_identical lines=9`、`name=q3-rootfail-8-128_output_identical lines=5`；没有 `_exit_`、`_differs`、`rerun_batch_incomplete`。
- 这条只证模型确定、输出可复现；不是新抽一次样（种子固定）。判决要引的数以主 agent 经这条命令在自己的副本里复跑出的为准。

**不需要交的命令**：命名检查（本腿没改 `crates/` 或 `scripts/` 下的 `.rs`）；`--selftest` 与门禁阶段（没碰）。

**下一步可以另派（本腿没做，不在这一段的范围里）**：第三节 Q1.1 的 1048 个盘面与 Q2.2 的归类都靠最小例和读代码，输出没拆。要把它们落成量，需要改模型再跑一次：Q 单元必须带记号记录（不允许 Q 单元没有记录）、额外记录起点落在单元起点时算作该单元的记录、按读法分开打印「别的病但只有 I-3.18 点名」的盘面数、把 I-3.9 与 I-3.11 的形建进来。这是新模型，不是本腿的复跑。

## 七、没做什么

- 没往统一任务服务交任务（`task-server/task` 的 `submit`、`wait`、`cancel`、`clean` 一条没起），没跑 cargo、没碰 GPU 与 checker 档；本腿读的模型数全部来自主 agent 的任务 4924。判决要引的数由主 agent 复跑（命令一）。
- 没改模型、没造入库的字节级坏镜像：Q1.1 的收严建议、Q2.3、Q3.4 只给预期。
- 第三节里「模型假病」「I-3.9 / I-3.11 的形」「窄记录加尾槽」三类的归类是读模型源码与每个标签的最小例得出的，没有逐盘面分；整批的比例没量。
- 没核真池的日志保留行数与模型 J 的对应（Q3.3c）；没核 Q9 的条款适用不适用 I-3.1 取宽这一支。
- 没读禁读清单里的 `research/prompts/m3-close-b19-reason-r2-opus-output.md`、`research/prompts/m3-close-b19-reason-r2-opus-c2-output.md`、`research/prompts/m3-close-b19-reason-r2-opus-model/` 与 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r2-opus/`。
- 没替主 agent 撤销或维持判决，只交复核结果。
