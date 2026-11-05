# m3-close-b19-reason-r2 攻方腿承接（Opus）报告（2026-11-02）

承接 `research/prompts/m3-close-b19-reason-r2-opus-output.md`（下称「前一条腿」）。这一段不另开攻击面，只做三件：把前一条腿每一处「推的，待主 agent 跑」对着任务 4919 的输出落成读数、核手算；读数落定之后逐格给现在的结论；把输出拷进 `out/` 当复跑参照。没读禁读清单里的三样。

## 〇、读的是哪一份输出，out/ 拷了什么

- 输出：`/home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r2-opus/rerun-first/`（主 agent 交的任务 4919，`type=succeeded exit=0`）。19 件的 `.rc` 都是 0（本段 `head` 现读）。
- 拷进 `research/prompts/m3-close-b19-reason-r2-opus-model/out/` 的 19 份，文件名就是 `rerun.sh` 第 116 行读的 `$MODEL/out/$name.txt`（`i31_mirrors.txt`、`i31_sweep_4.txt`、`q3_mirrors.txt`、`q3_q2-admission-4-32.txt` …… `q3_q2ext-bad-8-128.txt`）。拷完逐份算 sha256、行数，与 `task-server/task log 4919 --all` 里那一份的 `name=<件>_no_reference lines=… sha256=…` 行逐份比：19 份全对上（量的，本段命令 `grep -q` 逐份核，19 行 `match`）。`pilot.txt` 不在比对表里，没拷。
- 模型文件与前一条腿报告第〇节那张表逐份相同（`i31_q1_sweep.py` f07f78d3…、`i31_today.py` 67a369d2…、`tools/rerun.sh` 04a9e643…，本段 `sha256sum` 现查）。
- 本段在模型目录里新加一份 `i31_q1_sweep_quarantine_counted_in_i311.py`（sha256 8a15c96608717a69592fdd9b83bd736bdd1d60f4b53eef2926a63a6ccaa90dcd，162 行，`py_compile` 过了，pyc 落在草稿目录），为什么加见第三节；它不在 `rerun.sh` 的 19 件里，`rerun.sh` 没改。

## 一、各格现在的结论

| 格 | 前一条腿的判定 | 现在 | 依据（细节在第三、四节） |
|---|---|---|---|
| Q1 第 1 问　造得出那一形（窄记录的尾槽落在释放代在 (t, θ_high] 的已释放记录上） | 造得出（手算，模型待跑） | **造得出**（模型上量的；真 checker 上仍是推的） | `out/i31_mirrors.txt` 第 1 行逐字段与手算一致 |
| 同一格的子句「defer 行自洽时除 I-3.18 之外都不红的张数，我推是 0」 | 推的 0 | **前一条腿与模型各错一半**：模型数出 129，是模型 I-3.11 判据少了隔离豁免那一项造出来的（模型错）；按真 checker 的 I-3.11，走到的那一形确实 0（推的，第三节证明），但「X 是隔离单元、没走到」那一变体造得出除 I-3.18 外都不红的一张（推的，手造，前一条腿没想到这一变体） | 第三节 |
| Q1 第 1 问「改宽罩进只被被抛弃根引用的槽（U）造不出，进 U 时 I-3.15 判已死」 | 造不出 | **改：造得出部分没判、只红 I-3.18 的一张**（推的）。窄记录带隔离记号时 I-3.15 不判它，「已死」那一句不成立 | 同上那一张手造镜像就是这一形；第三节 |
| Q1 第 1 问「改宽罩进隔离豁免的槽，造不出」 | 造不出 | **站住**（推的）：能造出部分没判、只红 I-3.18 的一张，但尾槽压的是另一条隔离记录，没有 I-3.18 之外的病 | 第三节手造第二张 |
| Q1 第 3 问　说明里的数不够 | 不够（推的） | **站住，仍是推的**：模型里的两份界与前一条腿手算的数一致；真 checker 说明那一句的字段是前一条腿读代码得的，这一段没另核 | `out/i31_mirrors.txt` 第 1 行 |
| Q2　推荐候选丙与坏镜像造法 | 推荐丙；给造法 | **推荐站住**（模型上量的与手算一致）；字节级坏镜像仍没造，造法那一段仍是推的 | `out/i31_mirrors.txt` 第 1、4、6 行；`out/i31_sweep_4.txt` 第 1 行 |
| Q3 第 3 问　4096 占多少、今天部分没判的比例 | 待跑 | **量的（模型上）**：4096 只在 q2ext、S = 8、J = 128 上起作用，合法那一件占部分没判的 5/976，非法那一件 3/333 且这 3 段不设上界都判红；J = 128 时今天部分没判占合法抬 F 根 3.80%–8.63% | 第四节 |
| Q3 第 3 问　None 不会把该红的落成部分没判 | 推的（单调性论证） | **与模型读数相容**：按格式认第 0 代根那一变体只把部分没判改成成立，非法那几件一段都没变 | 第四节 |
| Q3 打中那一形（第 0 代根住哪块盘，写者与 checker 不一样） | 打中（手算，模型待跑） | **造得出（模型上量的，与手算一致）**；那段历史在真写者上可不可达仍是推的；随机历史 16 件 × 20000 段里一次都没出现 | `out/q3_mirrors.txt` 第 9、10 行；第四节 |

推翻条件见各节末。

## 二、读数表：前一条腿每一处「推的，待主 agent 跑」

每一处一行；「模型输出整行」是 `out/` 里那一份的原样整行（量的，模型是前一条腿照快照代码手抄的 python 槽级模型，不是编出来的二进制）。`q3_q2*` 那 16 份的整行含竖线，放在第四节的代码块里，这里只写结论。

| 处（前一条腿报告） | 原推测 | 模型输出整行 | 一致不一致 |
|---|---|---|---|
| Q1 打中那一形（第二节表；第八节第 2 条第一项）  | `I-3.1=partial`、`others_red=['I-3.11']`、`overlap_after_rewrite=[1]` | 「name=mirror case=tail_on_a_record_released_between_t_and_theta_high I-3.1=partial self=(E_high=1,E_t=2,walked_and_deferred=2,walked_outside=[1]) unit_class=(E_high=2,E_t=2,walked_and_deferred=2,walked_outside=[],E_t_outside=[]) I-3.18_red_at=[0] others_red=['I-3.11'] overlap_after_rewrite=[1] c944_starts=[]」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_mirrors.txt:1`） | 一致 |
| 对照：释放代改成 3（第二节「对照」）  | 自述那一份就成立，不进取宽 | 「name=mirror case=same_with_release_generation_above_theta_high I-3.1=holds self=(E_high=2,E_t=2,walked_and_deferred=2,walked_outside=[]) unit_class=none I-3.18_red_at=[0] others_red=['I-3.11'] overlap_after_rewrite=[1] c944_starts=[]」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_mirrors.txt:2`） | 一致（`I-3.1=holds`） |
| 对照：t = θ_high（第二节「对照」） | (t, θ_high] 空，这一形不在 | 「name=mirror case=same_with_t_equal_to_theta_high I-3.1=partial self=(E_high=1,E_t=1,walked_and_deferred=2,walked_outside=[1]) unit_class=(E_high=2,E_t=2,walked_and_deferred=2,walked_outside=[],E_t_outside=[]) I-3.18_red_at=[0] others_red=[] overlap_after_rewrite=[1] c944_starts=[]」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_mirrors.txt:3`） | 一致：B 的释放代 2 不高于 t = 2，是「已回收」那一类，不是打中那一形；I-3.1 照样报部分没判、`overlap_after_rewrite=[1]`，这一行的判词前一条腿没推 |
| 第一轮宽镜像（第八节第 2 条第一项） | `violated` | 「name=mirror case=round1_wide_record_over_a_neighbor_and_a_free_slot I-3.1=violated self=(E_high=3,E_t=3,walked_and_deferred=2,walked_outside=[]) unit_class=(E_high=1,E_t=1,walked_and_deferred=2,walked_outside=[1],E_t_outside=[]) I-3.18_red_at=[0] others_red=[] overlap_after_rewrite=[] c944_starts=[1]」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_mirrors.txt:4`） | 一致 |
| 第一轮窄记录加泄漏（同上） | `violated` | 「name=mirror case=round1_narrow_record_plus_headerless_leak I-3.1=violated self=(E_high=2,E_t=2,walked_and_deferred=2,walked_outside=[1]) unit_class=(E_high=3,E_t=3,walked_and_deferred=2,walked_outside=[],E_t_outside=[5]) I-3.18_red_at=[0] others_red=[] overlap_after_rewrite=[] c944_starts=[]」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_mirrors.txt:5`） | 一致 |
| C944 那一形（同上） | `holds`、`c944_starts=[1]` | 「name=mirror case=c944_wide_record_exactly_over_a_neighbor_without_its_own_record I-3.1=holds self=(E_high=2,E_t=2,walked_and_deferred=2,walked_outside=[]) unit_class=none I-3.18_red_at=[0] others_red=[] overlap_after_rewrite=[] c944_starts=[1]」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_mirrors.txt:6`） | 一致 |
| Q1 第 3 问说明里的数（第二节） | 自述下沿 1、上沿 2，改记下沿 2、上沿 2，走到而不在记录里的 [1] | 「name=mirror case=tail_on_a_record_released_between_t_and_theta_high I-3.1=partial self=(E_high=1,E_t=2,walked_and_deferred=2,walked_outside=[1]) unit_class=(E_high=2,E_t=2,walked_and_deferred=2,walked_outside=[],E_t_outside=[]) I-3.18_red_at=[0] others_red=['I-3.11'] overlap_after_rewrite=[1] c944_starts=[]」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_mirrors.txt:1`） | 一致（上沿 = min(E_t, walked_and_deferred)） |
| Q2 候选丙抓第一轮宽镜像（第三节表） | 抓到 50252 那一形 | 「name=mirror case=round1_wide_record_over_a_neighbor_and_a_free_slot I-3.1=violated self=(E_high=3,E_t=3,walked_and_deferred=2,walked_outside=[]) unit_class=(E_high=1,E_t=1,walked_and_deferred=2,walked_outside=[1],E_t_outside=[]) I-3.18_red_at=[0] others_red=[] overlap_after_rewrite=[] c944_starts=[1]」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_mirrors.txt:4`） | 一致（`c944_starts=[1]`；同表第 1 行 `c944_starts=[]`，打中那一形丙抓不到，也一致） |
| Q1 第 1 问 `partial_overlap_with_a_record_released_between_t_and_theta_nothing_else_red`（第二节末） | 0（「B 在 defer 里时 I-3.11 差一槽」） | 「name=i31_q1_sweep N=4 worlds=10008000 partial=47471 partial_overlap_after_rewrite=6522 partial_overlap_after_rewrite_nothing_else_red=1237 partial_overlap_with_a_record_released_between_t_and_theta=1900 partial_overlap_with_a_record_released_between_t_and_theta_nothing_else_red=129 partial_overlap_with_a_reclaimed_record_only=4398 partial_widened_into_quarantine_or_deferred=324 partial_widened_nothing_else_red=4 holds_c944=7914 holds_c944_only_released_above_theta_at_the_start=40126 holds_c944_nothing_else_red=1485」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_sweep_4.txt:1`） | **不一致**：129。哪一边错见第三节（模型判据错；前一条腿的 0 只对走到的那一形） |
| Q1 第 1 问改宽罩进隔离豁免或 U（第二节「没打中的两形」、第六节） | 造不出 | 「name=i31_q1_sweep N=4 worlds=10008000 partial=47471 partial_overlap_after_rewrite=6522 partial_overlap_after_rewrite_nothing_else_red=1237 partial_overlap_with_a_record_released_between_t_and_theta=1900 partial_overlap_with_a_record_released_between_t_and_theta_nothing_else_red=129 partial_overlap_with_a_reclaimed_record_only=4398 partial_widened_into_quarantine_or_deferred=324 partial_widened_nothing_else_red=4 holds_c944=7914 holds_c944_only_released_above_theta_at_the_start=40126 holds_c944_nothing_else_red=1485」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_sweep_4.txt:1`） | **不一致**：`partial_widened_into_quarantine_or_deferred=324`、`partial_widened_nothing_else_red=4`。第三节：4 张里有模型判据造出来的，修正之后仍有一形造得出 |
| sweep 其余计数（第八节第 2 条第二项） | 只点名要读，没给推测值 | 「name=i31_q1_sweep N=4 worlds=10008000 partial=47471 partial_overlap_after_rewrite=6522 partial_overlap_after_rewrite_nothing_else_red=1237 partial_overlap_with_a_record_released_between_t_and_theta=1900 partial_overlap_with_a_record_released_between_t_and_theta_nothing_else_red=129 partial_overlap_with_a_reclaimed_record_only=4398 partial_widened_into_quarantine_or_deferred=324 partial_widened_nothing_else_red=4 holds_c944=7914 holds_c944_only_released_above_theta_at_the_start=40126 holds_c944_nothing_else_red=1485」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_sweep_4.txt:1`） | —（读数见第三节） |
| `kind=between` 例子 | 同上 | 「name=i31_q1_sweep_example N=4 kind=between world=([Rec(slot=1, span=1, released=False, gen=1, marked=True), Rec(slot=2, span=1, released=False, gen=1, marked=False), Rec(slot=3, span=1, released=True, gen=2, marked=False)], {2: 2}, [2], 2, 1, 3, [3])」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_sweep_4.txt:2`） | —（这一张按真 checker 的 I-3.11 判红，第三节） |
| `kind=c944` 例子 | 同上 | 「name=i31_q1_sweep_example N=4 kind=c944 world=([Rec(slot=1, span=1, released=True, gen=2, marked=False), Rec(slot=2, span=2, released=False, gen=1, marked=False)], {3: 1}, [3], 2, 1, 2, [3])」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_sweep_4.txt:3`） | — |
| `kind=overlap` 例子 | 同上 | 「name=i31_q1_sweep_example N=4 kind=overlap world=([Rec(slot=2, span=1, released=False, gen=1, marked=False), Rec(slot=3, span=1, released=True, gen=1, marked=False)], {2: 2}, [2], 2, 1, 2, [3])」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_sweep_4.txt:4`） | —（已回收那一类） |
| `kind=widened` 例子 | 同上 | 「name=i31_q1_sweep_example N=4 kind=widened world=([Rec(slot=0, span=1, released=False, gen=1, marked=True), Rec(slot=1, span=1, released=True, gen=2, marked=False), Rec(slot=2, span=1, released=False, gen=1, marked=False), Rec(slot=3, span=1, released=True, gen=2, marked=False)], {0: 2, 2: 2}, [2], 2, 1, 4, [1])」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/i31_sweep_4.txt:5`） | —（这一张按真 checker 的 I-3.11 判红，第三节） |
| Q3 第 0 代根那一种，F = 3、今天（第四节第 4 问表；第八节第 2 条第三项） | `('partial', 'disagree')` | 「name=q3_mirror form=genesis_option raised_F=3 legal=True variant=today window=holds_undetermined possible_rings=3 final=('partial', 'disagree')」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/q3_mirrors.txt:1`） | 一致 |
| 同上，按格式认第 0 代根 | `('holds', '-')` | 「name=q3_mirror form=genesis_option raised_F=3 legal=True variant=genesis_known window=holds_undetermined possible_rings=3 final=('holds', '-')」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/q3_mirrors.txt:2`） | 一致 |
| 同上，F = 4、今天 | `red` | 「name=q3_mirror form=genesis_option raised_F=4 legal=False variant=today window=holds_undetermined possible_rings=3 final=('red', '-')」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/q3_mirrors.txt:3`） | 一致 |
| 同上，F = 4、按格式认 | `red` | 「name=q3_mirror form=genesis_option raised_F=4 legal=False variant=genesis_known window=holds_undetermined possible_rings=3 final=('red', '-')」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/q3_mirrors.txt:4`） | 一致 |
| 往下认两层，F = 6、今天 | 成立 | 「name=q3_mirror form=two_layers_below_u raised_F=6 legal=True variant=today window=holds_undetermined possible_rings=3 final=('holds', '-')」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/q3_mirrors.txt:5`） | 一致 |
| 同上，按格式认 | 成立 | 「name=q3_mirror form=two_layers_below_u raised_F=6 legal=True variant=genesis_known window=holds_undetermined possible_rings=3 final=('holds', '-')」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/q3_mirrors.txt:6`） | 一致 |
| 往下认两层，F = 7、今天 | 红 | 「name=q3_mirror form=two_layers_below_u raised_F=7 legal=False variant=today window=holds_undetermined possible_rings=3 final=('red', '-')」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/q3_mirrors.txt:7`） | 一致 |
| 同上，按格式认 | 红 | 「name=q3_mirror form=two_layers_below_u raised_F=7 legal=False variant=genesis_known window=holds_undetermined possible_rings=3 final=('red', '-')」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/q3_mirrors.txt:8`） | 一致 |
| Q3 打中那一形，F = 5、今天（第四节第 3 问外带） | `red` | 「name=q3_mirror form=genesis_copy_alone_on_a_device raised_F=5 writer_ceiling=5 variant=today window=holds_undetermined possible_rings=2 final=('red', '-')」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/q3_mirrors.txt:9`） | 一致 |
| 同上，照写者的盘归属 | `('partial', 'disagree')` | 「name=q3_mirror form=genesis_copy_alone_on_a_device raised_F=5 writer_ceiling=5 variant=writer_device_for_genesis window=holds_undetermined possible_rings=2 final=('partial', 'disagree')」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/q3_mirrors.txt:10`） | 一致 |
| 同上，F = 6、今天 | 没推 | 「name=q3_mirror form=genesis_copy_alone_on_a_device raised_F=6 writer_ceiling=5 variant=today window=holds_undetermined possible_rings=2 final=('red', '-')」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/q3_mirrors.txt:11`） | —（两种环上限 4、0 都低于 6，红，手算相容） |
| 同上，F = 6、照写者的盘归属 | 没推 | 「name=q3_mirror form=genesis_copy_alone_on_a_device raised_F=6 writer_ceiling=5 variant=writer_device_for_genesis window=holds_undetermined possible_rings=2 final=('red', '-')」（`research/prompts/m3-close-b19-reason-r2-opus-model/out/q3_mirrors.txt:12`） | —（上限 5、4 都低于 6，红，手算相容） |

q3_run 那 16 份的读数（第八节第 2 条第四项与第五节表「今天」那一行）在第四节。

## 三、Q1 两处不一致：哪一边错、怎么错

### 模型错的那一半：I-3.11 判据少了隔离豁免那一项

模型的 I-3.11 是「记账 − defer == 走到的槽数」：「if acct - defer != len(walked_slots):」（`research/prompts/m3-close-b19-reason-r2-opus-model/i31_q1_sweep.py:57`）。真 checker 在走到的槽数上再加隔离豁免的槽数：「+ quarantined_exempted_newest;」（`crates/singlefs-checker/src/walk.rs:21568`），然后「matches!((allocated, deferred), (Some(allocated), Some(deferred)) if deferred.checked_add(referenced_by_the_newest_root) == Some(allocated)),」（`crates/singlefs-checker/src/walk.rs:21571`）。条文同样写了豁免：「隔离的记录与 I-3.1（已分配统计对得上） 那一句相同：只认记号，组里每块盘上那条记录都带隔离记号的整组豁免，不看副本；」（`crates/singlefs-checker/invariants/05-空间记账.md:18`）。走读与快照里 `walk.rs`、`05-空间记账.md` 两份的 sha256 相同（b533596e…、70d8beba…，本段现查）。

后果：世界里有一条起点不被引用、带记号的未释放记录时，模型的 I-3.11 认的记账比真 checker 认的少一份豁免跨度。sweep 打出的两张例子都带这样一条记录（`marked=True`），按真 checker 的 I-3.11 逐张算（推的，照上面两行代码手算）：

| 例子 | defer | 走到 | 隔离豁免 | 真 checker 要的记账 | 例子的记账 | 真 I-3.11 |
|---|---|---|---|---|---|---|
| `kind=between`（`out/i31_sweep_4.txt` 第 2 行） | 1（槽 3，释放代 2 > t = 1） | 2（单元 2 两槽） | 1（槽 1） | 4 | 3 | 红 |
| `kind=widened`（同一份第 5 行） | 2（槽 1、3） | 2 | 1（槽 0） | 5 | 4 | 红 |

所以 `…_between_t_and_theta_nothing_else_red=129` 与 `partial_widened_nothing_else_red=4` 里至少有这两张是模型判据造出来的；别的几张是不是，要修正判据之后重数（第七节命令）。

### 前一条腿对的那一半：走到的那一形，按真 I-3.11 一张都没有（推的，证明）

记号：W 走到的槽数，Q 隔离豁免槽数，D 写者自洽的 defer（已释放、释放代高于 t 的记录跨度和），U_unwalked 是这些 defer 槽里没走到的（模型的 `d`）。除 I-3.18 外都不红时 I-5.4 成立、记录不重叠，|U_unwalked| ≤ D。

1. 真 I-3.11 成立 ⇒ 记账 = D + W + Q。
2. 部分没判要求记账 ≤ 改记那一份的上沿 ≤ |Ω ∪ U| = W + Q + |U_unwalked|（模型 `bounds` 的 `wc`，自述、改记两份同一个数）。
3. 改记加宽的那条记录 A 的单元 X 被走到 ⇒ 加宽多出来的槽都是走到的 ⇒ 压在上面的那条释放代在 (t, θ_high] 的记录 B 至少有一槽走到、在 D 里而不在 U_unwalked 里 ⇒ |U_unwalked| ≤ D − 1。
4. 合起来 D + W + Q ≤ W + Q + D − 1，矛盾。

所以「X 被走到、除 I-3.18 外都不红」在任何记账值上都不报部分没判。前一条腿的「我推后者是 0」对这一形成立。

### 两边都没看到的一形：X 是隔离单元（推的，手造）

上面第 3 步要 X 被走到。X 没走到时，A 不带记号则 I-3.15 判「已死」；**A 带隔离记号时 I-3.15 不判**（条文「不判，只认记号、不看副本」（`crates/singlefs-checker/invariants/05-空间记账.md:22`）），I-3.17 ② 也不红（A 自述那一槽没走到）。于是：

| 项 | 镜像上的样子（一块盘 4 槽，t = 1，θ_high = 2） |
|---|---|
| 槽 0、1 | 两槽单元 Y，最新根引用、走到；记录 (0，跨 1，未释放) |
| 槽 2、3 | 两槽单元 X，没有根引用；记录 A = (2，跨 1，未释放，带隔离记号) |
| 槽 3 | 记录 B = (3，跨 1，已释放，释放代 2) |
| 记账 | 已分配 4 = defer 1 + 走到 2 + 隔离豁免 1；defer 行 1（写者自洽） |

逐判（推的，照 `i31_today.py` 的转写手算）：隔离豁免 {2}；|Ω ∪ U| = 2 + 1 + 1 = 4。自述那一份 E(θ_high) = {0, 2}，槽 1 走到而不在里面，(P) 不成立。改记：Y 那条改成跨 2、A 改成跨 2；E_class(θ_high) = E_class(t) = {0, 1, 2, 3}，逐槽下沿成立，逐槽上沿里槽 2 在隔离豁免、槽 3 在 U，成立；4 ≤ 4 ≤ min(4, 4) ⇒ **I-3.1 部分没判**。I-3.18 红在槽 0 与槽 2；I-5.4 按自述跨度不相交，成立；I-3.15 Y 走到、A 带记号不判；I-3.17 成立；I-3.9 B 起点 3 不是引用起点，成立；I-3.11 按真判据 1 + 2 + 1 = 4，成立（模型判据 4 − 1 = 3 ≠ 2，红——所以原 sweep 数不到这一张）。

病：X 的尾槽 3 正压在 B 那一次释放留下、还在 defer 里的槽上。B 的释放代一低于门槛，槽 3 就被复用，而隔离本来就是要把 X 的槽都留在已分配里。I-3.18 只说「跨 1，那一类占 2 槽」，没说尾槽压在一条还在 defer 的记录上；这一张上除 I-3.18 之外一条都不红。它满足正文 Q1 第 1 问「改记之后两条记录罩同一槽（I-5.4 只按自述跨度判，看不见）」与「一条记录改宽之后……罩进一个只有被抛弃根引用的槽」两个分句（槽 3 在 U 里；「只有被抛弃根引用」这一层模型不建，我按前一条腿的读法把它当 U），原文 `research/prompts/_m3-close-b19-reason-r2-body.md` 第 35 行。

前一条腿第二节「没打中的两形」第二条说进 U 时「I-3.15 判 A 的单元没被引用（「已死」）」，这一张上不成立：A 带记号。所以那一格从「造不出」改成「造得出」（推的）。

打中之后的四句：

| 问 | 答 |
|---|---|
| 分不分辨臂 | 不分辨，同前一条腿那一形：病根在两臂共用的「E 按槽取并集」，逐判混取在这一张上同样部分没判（自述那一份只 (P) 不成立） |
| 被判的系统看不看得到 | 看得到：A 改记之后的区间、B 的区间、A 的隔离记号都在 checker 手里 |
| 满足的是哪一个分句 | 上一段两个分句；与前一条腿那一形的差别只在 X 是隔离单元而不是走到的单元 |
| 改法还中不中 | 前一条腿提的「改记之后的槽区间不与别的任何记录相交，相交判红」：A 改记后 [2, 4) 与 B [3, 4) 相交，红，还中（推的）。候选丙不碰（走到的起点 0 有自己的记录） |

### 改宽罩进隔离豁免的槽：造不出漏判，站住（推的）

同一套几何，槽 3 上换成另一条带记号的未释放记录 C = (3，跨 1)：记账 4 = 0 + 2 + 2。逐判同上，I-3.1 部分没判，只红 I-3.18；改记后 A 罩进的槽 3 在隔离豁免里。这一张两条隔离记录罩同一槽，没有复用的风险，I-3.18 之外没有病，所以「改宽罩进隔离豁免的槽造不出漏判」站住；但原 sweep 的「改宽且除 I-3.18 外都不红」那一计数不是 0，按真判据也不是 0（推的）。

两张手造镜像都写进了 `i31_q1_sweep_quarantine_counted_in_i311.py` 的 `mirrors`（`quarantined_unit_tail_on_a_record_released_between_t_and_theta_high` 与 `quarantined_unit_tail_on_another_quarantined_record` 两个 case）。跑前写下的预期：前一个 case `I-3.1=partial`、`I-3.18_red_at=[0, 2]`、`others_red_model=['I-3.11']`、`others_red_quarantine_counted=[]`、`overlap_after_rewrite=[3]`、`between_on_walked_unit=(False, True)`；后一个 case 同，只 `between_on_walked_unit=(False, False)`；`sweep_example_between`、`sweep_example_widened` 两个 case `others_red_model=[]`、`others_red_quarantine_counted=['I-3.11']`；从 `out/i31_mirrors.txt` 拿来的六个 case 两列相同。sweep 4：`model_` 开头的四个计数应等于原 sweep 的 1237、129、4、1485（对不上就是新脚本写错，整份作废）；`quarantine_counted_between_nothing_else_red_on_a_walked_unit=0`；`…_on_an_unwalked_unit_only`、`…_widened_into_quarantine_…`、`…_widened_into_deferred_…` 都 ≥ 1。量出相反的：走到的那一形 > 0 就是上面的证明错了，回头看第 2 步的 |Ω ∪ U|；没走到的那一形 = 0 就是手造那一张算错了，`quarantined_unit_tail_on_a_record_released_between_t_and_theta_high` 那一行会先露出来。

推翻条件：真 checker 的 I-3.15 或 I-3.18 对带隔离记号的记录另有一判红在这一张上；或 I-5.4 改成按改记跨度判。

## 四、Q3 的读数（q3_run 那 16 份）

整行原样（量的，模型上）；每行前一行写它在哪份文件第几行。`legal_raise_roots` 是写者自己判合法的抬 F 根，`partial_why` 里 `combos` 是 4096 上界、`journal0` 是重建那一层 journal 里一条记录都没有（`ding_today.py` 第 100 行交 `journal%d`）、`disagree` 是可能的环判得不一致。

```
# out/q3_q2-admission-4-128.txt 第 1 行
name=q3_today kind=q2 writer=admission S=4 J=128 seeds=20000 legal_raise_roots=15626 reaches_ding=13725 max_rings_no_bound=48 today[holds=14421 red=0 partial=839 not_judged=366 | partial_why disagree:839] no_bound[holds=14421 red=0 partial=839 not_judged=366 | partial_why disagree:839] genesis_known[holds=14433 red=0 partial=827 not_judged=366 | partial_why disagree:827] genesis_txg_device[holds=14421 red=0 partial=839 not_judged=366 | partial_why disagree:839] combos_then_no_bound[holds=0 red=0 partial=0] changes[genesis_known_changes:partial>holds=12]
# out/q3_q2-admission-4-32.txt 第 1 行
name=q3_today kind=q2 writer=admission S=4 J=32 seeds=20000 legal_raise_roots=15626 reaches_ding=13725 max_rings_no_bound=48 today[holds=14235 red=0 partial=1025 not_judged=366 | partial_why disagree:765,journal0:260] no_bound[holds=14235 red=0 partial=1025 not_judged=366 | partial_why disagree:765,journal0:260] genesis_known[holds=14247 red=0 partial=1013 not_judged=366 | partial_why disagree:753,journal0:260] genesis_txg_device[holds=14235 red=0 partial=1025 not_judged=366 | partial_why disagree:765,journal0:260] combos_then_no_bound[holds=0 red=0 partial=0] changes[genesis_known_changes:partial>holds=12]
# out/q3_q2-admission-8-128.txt 第 1 行
name=q3_today kind=q2 writer=admission S=8 J=128 seeds=20000 legal_raise_roots=27827 reaches_ding=14287 max_rings_no_bound=64 today[holds=25189 red=0 partial=2245 not_judged=393 | partial_why disagree:2245] no_bound[holds=25189 red=0 partial=2245 not_judged=393 | partial_why disagree:2245] genesis_known[holds=25369 red=0 partial=2065 not_judged=393 | partial_why disagree:2065] genesis_txg_device[holds=25189 red=0 partial=2245 not_judged=393 | partial_why disagree:2245] combos_then_no_bound[holds=0 red=0 partial=0] changes[genesis_known_changes:partial>holds=180]
# out/q3_q2-admission-8-32.txt 第 1 行
name=q3_today kind=q2 writer=admission S=8 J=32 seeds=20000 legal_raise_roots=27827 reaches_ding=14287 max_rings_no_bound=24 today[holds=17214 red=0 partial=10220 not_judged=393 | partial_why disagree:1690,journal0:8530] no_bound[holds=17214 red=0 partial=10220 not_judged=393 | partial_why disagree:1690,journal0:8530] genesis_known[holds=17358 red=0 partial=10076 not_judged=393 | partial_why disagree:1546,journal0:8530] genesis_txg_device[holds=17214 red=0 partial=10220 not_judged=393 | partial_why disagree:1690,journal0:8530] combos_then_no_bound[holds=0 red=0 partial=0] changes[genesis_known_changes:partial>holds=144]
# out/q3_q2ext-admission-4-128.txt 第 1 行
name=q3_today kind=q2ext writer=admission S=4 J=128 seeds=20000 legal_raise_roots=15627 reaches_ding=13876 max_rings_no_bound=288 today[holds=13922 red=0 partial=1348 not_judged=357 | partial_why disagree:1348] no_bound[holds=13922 red=0 partial=1348 not_judged=357 | partial_why disagree:1348] genesis_known[holds=13977 red=0 partial=1293 not_judged=357 | partial_why disagree:1293] genesis_txg_device[holds=13922 red=0 partial=1348 not_judged=357 | partial_why disagree:1348] combos_then_no_bound[holds=0 red=0 partial=0] changes[genesis_known_changes:partial>holds=55]
# out/q3_q2ext-admission-4-32.txt 第 1 行
name=q3_today kind=q2ext writer=admission S=4 J=32 seeds=20000 legal_raise_roots=15627 reaches_ding=13876 max_rings_no_bound=288 today[holds=13627 red=0 partial=1643 not_judged=357 | partial_why disagree:1151,journal0:492] no_bound[holds=13627 red=0 partial=1643 not_judged=357 | partial_why disagree:1151,journal0:492] genesis_known[holds=13681 red=0 partial=1589 not_judged=357 | partial_why disagree:1097,journal0:492] genesis_txg_device[holds=13627 red=0 partial=1643 not_judged=357 | partial_why disagree:1151,journal0:492] combos_then_no_bound[holds=0 red=0 partial=0] changes[genesis_known_changes:partial>holds=54]
# out/q3_q2ext-admission-8-128.txt 第 1 行
name=q3_today kind=q2ext writer=admission S=8 J=128 seeds=20000 legal_raise_roots=25678 reaches_ding=12717 max_rings_no_bound=98304 today[holds=24402 red=0 partial=976 not_judged=300 | partial_why combos:5,disagree:971] no_bound[holds=24405 red=0 partial=973 not_judged=300 | partial_why too_many:1,disagree:972] genesis_known[holds=24772 red=0 partial=606 not_judged=300 | partial_why combos:5,disagree:601] genesis_txg_device[holds=24402 red=0 partial=976 not_judged=300 | partial_why combos:5,disagree:971] combos_then_no_bound[holds=3 red=0 partial=2] changes[genesis_known_changes:partial>holds=370]
# out/q3_q2ext-admission-8-32.txt 第 1 行
name=q3_today kind=q2ext writer=admission S=8 J=32 seeds=20000 legal_raise_roots=25678 reaches_ding=12717 max_rings_no_bound=144 today[holds=16082 red=0 partial=9296 not_judged=300 | partial_why disagree:418,journal0:8878] no_bound[holds=16082 red=0 partial=9296 not_judged=300 | partial_why disagree:418,journal0:8878] genesis_known[holds=16351 red=0 partial=9027 not_judged=300 | partial_why disagree:149,journal0:8878] genesis_txg_device[holds=16082 red=0 partial=9296 not_judged=300 | partial_why disagree:418,journal0:8878] combos_then_no_bound[holds=0 red=0 partial=0] changes[genesis_known_changes:partial>holds=269]
# out/q3_q2-bad-4-128.txt 第 1 行
name=q3_today kind=q2 writer=bad S=4 J=128 seeds=20000 illegal_raise_roots=14141 reaches_ding=10547 max_rings_no_bound=48 today[holds=0 red=13281 partial=531 not_judged=329 | partial_why disagree:531] no_bound[holds=0 red=13281 partial=531 not_judged=329 | partial_why disagree:531] genesis_known[holds=0 red=13281 partial=531 not_judged=329 | partial_why disagree:531] genesis_txg_device[holds=0 red=13281 partial=531 not_judged=329 | partial_why disagree:531] combos_then_no_bound[holds=0 red=0 partial=0] changes[-]
# out/q3_q2-bad-4-32.txt 第 1 行
name=q3_today kind=q2 writer=bad S=4 J=32 seeds=20000 illegal_raise_roots=14141 reaches_ding=10547 max_rings_no_bound=48 today[holds=0 red=13115 partial=697 not_judged=329 | partial_why disagree:484,journal0:213] no_bound[holds=0 red=13115 partial=697 not_judged=329 | partial_why disagree:484,journal0:213] genesis_known[holds=0 red=13115 partial=697 not_judged=329 | partial_why disagree:484,journal0:213] genesis_txg_device[holds=0 red=13115 partial=697 not_judged=329 | partial_why disagree:484,journal0:213] combos_then_no_bound[holds=0 red=0 partial=0] changes[-]
# out/q3_q2-bad-8-128.txt 第 1 行
name=q3_today kind=q2 writer=bad S=8 J=128 seeds=20000 legal_raise_roots=175 reaches_ding=175 max_rings_no_bound=6 today[holds=2 red=0 partial=173 not_judged=0 | partial_why disagree:173] no_bound[holds=2 red=0 partial=173 not_judged=0 | partial_why disagree:173] genesis_known[holds=2 red=0 partial=173 not_judged=0 | partial_why disagree:173] genesis_txg_device[holds=2 red=0 partial=173 not_judged=0 | partial_why disagree:173] combos_then_no_bound[holds=0 red=0 partial=0] changes[-]
# out/q3_q2-bad-8-128.txt 第 2 行
name=q3_today kind=q2 writer=bad S=8 J=128 seeds=20000 illegal_raise_roots=24843 reaches_ding=9225 max_rings_no_bound=64 today[holds=0 red=24383 partial=117 not_judged=343 | partial_why disagree:117] no_bound[holds=0 red=24383 partial=117 not_judged=343 | partial_why disagree:117] genesis_known[holds=0 red=24383 partial=117 not_judged=343 | partial_why disagree:117] genesis_txg_device[holds=0 red=24383 partial=117 not_judged=343 | partial_why disagree:117] combos_then_no_bound[holds=0 red=0 partial=0] changes[-]
# out/q3_q2-bad-8-32.txt 第 1 行
name=q3_today kind=q2 writer=bad S=8 J=32 seeds=20000 legal_raise_roots=175 reaches_ding=175 max_rings_no_bound=6 today[holds=2 red=0 partial=173 not_judged=0 | partial_why disagree:173] no_bound[holds=2 red=0 partial=173 not_judged=0 | partial_why disagree:173] genesis_known[holds=2 red=0 partial=173 not_judged=0 | partial_why disagree:173] genesis_txg_device[holds=2 red=0 partial=173 not_judged=0 | partial_why disagree:173] combos_then_no_bound[holds=0 red=0 partial=0] changes[-]
# out/q3_q2-bad-8-32.txt 第 2 行
name=q3_today kind=q2 writer=bad S=8 J=32 seeds=20000 illegal_raise_roots=24843 reaches_ding=9225 max_rings_no_bound=24 today[holds=0 red=18017 partial=6483 not_judged=343 | partial_why disagree:33,journal0:6450] no_bound[holds=0 red=18017 partial=6483 not_judged=343 | partial_why disagree:33,journal0:6450] genesis_known[holds=0 red=18017 partial=6483 not_judged=343 | partial_why disagree:33,journal0:6450] genesis_txg_device[holds=0 red=18017 partial=6483 not_judged=343 | partial_why disagree:33,journal0:6450] combos_then_no_bound[holds=0 red=0 partial=0] changes[-]
# out/q3_q2ext-bad-4-128.txt 第 1 行
name=q3_today kind=q2ext writer=bad S=4 J=128 seeds=20000 illegal_raise_roots=14382 reaches_ding=10942 max_rings_no_bound=288 today[holds=0 red=13225 partial=838 not_judged=319 | partial_why disagree:838] no_bound[holds=0 red=13225 partial=838 not_judged=319 | partial_why disagree:838] genesis_known[holds=0 red=13225 partial=838 not_judged=319 | partial_why disagree:838] genesis_txg_device[holds=0 red=13225 partial=838 not_judged=319 | partial_why disagree:838] combos_then_no_bound[holds=0 red=0 partial=0] changes[-]
# out/q3_q2ext-bad-4-32.txt 第 1 行
name=q3_today kind=q2ext writer=bad S=4 J=32 seeds=20000 illegal_raise_roots=14382 reaches_ding=10942 max_rings_no_bound=288 today[holds=0 red=12917 partial=1146 not_judged=319 | partial_why disagree:727,journal0:419] no_bound[holds=0 red=12917 partial=1146 not_judged=319 | partial_why disagree:727,journal0:419] genesis_known[holds=0 red=12917 partial=1146 not_judged=319 | partial_why disagree:727,journal0:419] genesis_txg_device[holds=0 red=12917 partial=1146 not_judged=319 | partial_why disagree:727,journal0:419] combos_then_no_bound[holds=0 red=0 partial=0] changes[-]
# out/q3_q2ext-bad-8-128.txt 第 1 行
name=q3_today kind=q2ext writer=bad S=8 J=128 seeds=20000 illegal_raise_roots=23360 reaches_ding=9185 max_rings_no_bound=24576 today[holds=0 red=22765 partial=333 not_judged=262 | partial_why combos:3,disagree:330] no_bound[holds=0 red=22768 partial=330 not_judged=262 | partial_why disagree:330] genesis_known[holds=0 red=22765 partial=333 not_judged=262 | partial_why combos:3,disagree:330] genesis_txg_device[holds=0 red=22765 partial=333 not_judged=262 | partial_why combos:3,disagree:330] combos_then_no_bound[holds=0 red=3 partial=0] changes[-]
# out/q3_q2ext-bad-8-32.txt 第 1 行
name=q3_today kind=q2ext writer=bad S=8 J=32 seeds=20000 illegal_raise_roots=23360 reaches_ding=9185 max_rings_no_bound=96 today[holds=0 red=16015 partial=7083 not_judged=262 | partial_why disagree:79,journal0:7004] no_bound[holds=0 red=16015 partial=7083 not_judged=262 | partial_why disagree:79,journal0:7004] genesis_known[holds=0 red=16015 partial=7083 not_judged=262 | partial_why disagree:79,journal0:7004] genesis_txg_device[holds=0 red=16015 partial=7083 not_judged=262 | partial_why disagree:79,journal0:7004] combos_then_no_bound[holds=0 red=0 partial=0] changes[-]
```

从上面整行算出的比例（量的数是整行里的计数；百分比是本段用 awk 从整行算的）：

| 件（写者合法的抬 F 根） | 抬 F 根 | 走到丁 | 今天部分没判 | 占抬 F 根 | 占走到丁 | 其中 4096（combos） | 其中 journal0 | 去掉 journal0 占抬 F 根 |
|---|---|---|---|---|---|---|---|---|
| q2、S = 4、J = 128 | 15626 | 13725 | 839 | 5.37% | 6.11% | 0 | 0 | 5.37% |
| q2、S = 4、J = 32 | 15626 | 13725 | 1025 | 6.56% | 7.47% | 0 | 260 | 4.90% |
| q2、S = 8、J = 128 | 27827 | 14287 | 2245 | 8.07% | 15.71% | 0 | 0 | 8.07% |
| q2、S = 8、J = 32 | 27827 | 14287 | 10220 | 36.73% | 71.53% | 0 | 8530 | 6.07% |
| q2ext、S = 4、J = 128 | 15627 | 13876 | 1348 | 8.63% | 9.71% | 0 | 0 | 8.63% |
| q2ext、S = 4、J = 32 | 15627 | 13876 | 1643 | 10.51% | 11.84% | 0 | 492 | 7.37% |
| q2ext、S = 8、J = 128 | 25678 | 12717 | 976 | 3.80% | 7.67% | 5 | 0 | 3.80% |
| q2ext、S = 8、J = 32 | 25678 | 12717 | 9296 | 36.20% | 73.10% | 0 | 8878 | 1.63% |

| 件（bad 写者非法的抬 F 根） | 抬 F 根 | 今天红 | 今天成立 | 今天部分没判 | 其中 4096 | 其中 journal0 |
|---|---|---|---|---|---|---|
| q2、S = 4、J = 128 | 14141 | 13281 | 0 | 531 | 0 | 0 |
| q2、S = 4、J = 32 | 14141 | 13115 | 0 | 697 | 0 | 213 |
| q2、S = 8、J = 128 | 24843 | 24383 | 0 | 117 | 0 | 0 |
| q2、S = 8、J = 32 | 24843 | 18017 | 0 | 6483 | 0 | 6450 |
| q2ext、S = 4、J = 128 | 14382 | 13225 | 0 | 838 | 0 | 0 |
| q2ext、S = 4、J = 32 | 14382 | 12917 | 0 | 1146 | 0 | 419 |
| q2ext、S = 8、J = 128 | 23360 | 22765 | 0 | 333 | 3 | 0 |
| q2ext、S = 8、J = 32 | 23360 | 16015 | 0 | 7083 | 0 | 7004 |

读出来的几样（都是模型上量的）：

1. **4096 上界**：16 件里只有 q2ext、S = 8、J = 128 两件碰到。合法那一件 5 段，占今天部分没判 976 段的 0.51%；不设上界时 3 段成立、2 段仍部分没判（`combos_then_no_bound[holds=3 red=0 partial=2]`，其中 1 段超过模型 65536 的封顶报 `too_many`，最大可能环数 98304）。非法那一件 3 段，占 333 段的 0.90%，不设上界时 3 段都判红（`combos_then_no_bound[holds=0 red=3 partial=0]`）：4096 在这一格把 3 段该红的落成了部分没判，占非法抬 F 根 23360 段的 0.013%。别的 14 件可能环数最多 288 种，上界不起作用。
2. **今天部分没判的比例**：第一轮判决引的 J = 128 那一列，合法抬 F 根上 3.80%–8.63%（占走到丁的 6.11%–15.71%），几乎全是 `disagree`。J = 32、S = 8 时 journal 记录被回收，`journal0` 占部分没判的八成以上，整体升到 36% 上下；去掉 journal0 是 1.63%–6.07%。前一条腿没给推测值，这一格没有一致不一致可比。
3. **合法写者不被判红**：8 件写者合法的抬 F 根今天 `red=0`，16 份里没有一行 `name=q3_example_false_red`（本段 `grep -c q3_example` 合计 0）。
4. **非法抬 F 一段都不判成立**：8 件非法的 `holds=0`，与前一条腿第八节「应为 0」一致。
5. **None 的单调性**：`genesis_known` 那一变体只把部分没判改成成立（合法 8 件 `genesis_known_changes:partial>holds` 分别是 12、12、180、144、55、54、370、269），非法 8 件一段都没变（`changes[-]`）。没有一段因为知道了第 0 代根的状态而由部分没判变红，与前一条腿的单调性论证相容。这一条与论证共用同一份代码转写（同一个作者照同一份快照抄的），不算独立的第二条路。
6. **第 0 代根的盘归属**：`genesis_txg_device` 那一变体在 16 件里与今天逐项相同，没有一行 `name=q3_example_genesis_device`。随机历史里没长出打中那一形，它只在 `q3_mirrors.py` 手排的那段历史上出现。
7. 没解释的一处：`q3_q2-bad-8-128.txt`、`q3_q2-bad-8-32.txt` 第 1 行，bad 写者自己判合法的抬 F 根 175 段，今天 173 段部分没判、2 段成立。我没查历史模型里 bad 写者这 175 段是怎么来的，只列在这里。

## 五、Q3 两格现在的结论

- **第 3 问 4096 与部分没判比例**：量的（模型上），数在上一节。4096 只在 S = 8、可能环多到上万种时起作用；它藏掉的那 3 段非法抬 F 是今天唯一一处「上界把该红的落成部分没判」的读数（模型上量的）。推翻条件：真 checker 上可能环的个数比模型多（模型把每槽的可能取法照快照抄，没对拍二进制）。
- **打中那一形（第 0 代根住哪块盘）**：造得出（模型上量的，`out/q3_mirrors.txt` 第 9、10 行与手算一致）。仍是推的有两样：那段五次都崩在区域 1 落点上的历史在真写者上可不可达（前一条腿限度第 4 条，写者一次挂载发几版没核）；哪一读法对（D16 已定项 1 按字面算 checker 对）交主 agent。随机历史 16 件里一次都没出现，说明它要专门排出来，不说明它不可达。推翻条件：真写者在 S = 4 的几何上挂载一次的发布数与手排的不一样，排不出区域 1 连崩五次。

## 六、没打中的形状与这一段自己的限度

- 读数表里除那两处之外每一处都一致，没有一处手算与模型输出对不上而该怪手算的。一致只说明手算与前一条腿自己的转写相符：两边是同一个作者照同一份快照写的，共用一份读代码的前提，不构成独立校验（`evidence-discipline.md` 校验那一步要的不共用）。
- 模型判据的错只查了 I-3.11 那一处（因为 129 与前一条腿的推测对不上才去查）。`other_reds` 里别的几条（I-5.4、I-3.15、I-3.17、I-3.9）与真 checker 对没对上，这一段没逐条核；`holds_c944_nothing_else_red=1485` 也受 I-3.11 那一处影响，修正后的数待第七节命令。
- 第三节的证明与两张手造镜像都是推的：照 `i31_today.py` 的转写手算，没跑；修正判据的新脚本写好、语法过了，没跑。
- 两张手造镜像要写者有窄记录那一类缺陷才有，合法写者造不出（与前一条腿那一形同一个前提）；它们是判别力的问题，不是误红。
- 第四节第 7 条那 175 段没查来历。
- 没造字节级坏镜像，没对拍 GPU，没读真 checker 部分没判说明那一句的实现（Q1 第 3 问那一格的字段仍照前一条腿读的）。

## 七、要主 agent 跑的命令

都不是重型（python 小模型，不编 cargo、不碰 checker 档包）。在仓根 `/home/fy5090/code/singlefs` 起，命令里全是绝对路径。范围声明闸要是要求这一类带 `SINGLEFS_RUN_SCOPE`，照它拒绝说明给的写法补。

1. 整份模型带参照复跑（`out/` 已拷好，`--work` 是新目录，`rerun.sh` 第 70 行要求它不存在或为空）：
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r2-opus --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r2-opus-c2/rerun-with-reference`
   - 看：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里 19 行都是 `name=<件>_output_identical`，没有 `_output_differs`、`_exit_`、`_no_reference`；`name=rerun_pilot` 的估时不超过 2400 秒。上一次挂钟 336 秒（量的，任务 4919 的 `name=rerun_wall_seconds seconds=336`）。
2. 修正 I-3.11 判据之后的镜像（几秒）：
   - 命令：`task-server/task submit -- python3 -B /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-opus-model/i31_q1_sweep_quarantine_counted_in_i311.py mirrors`
   - 看：`exit=0`；10 行 `name=mirror_quarantine_counted`，与第三节末写下的预期逐项比。
3. 修正判据之后的小世界，先 N = 3 估时：
   - 命令：`task-server/task submit -- python3 -B /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-opus-model/i31_q1_sweep_quarantine_counted_in_i311.py sweep 3`
   - 看：`exit=0`，一行 `name=i31_q1_sweep_quarantine_counted N=3 …` 与若干例子行；挂钟按任务时长读。
4. 同上 N = 4（与原 sweep 同一个世界空间）：
   - 命令：`task-server/task submit -- python3 -B /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-opus-model/i31_q1_sweep_quarantine_counted_in_i311.py sweep 4`
   - 估时（推的）：原 sweep 4 在任务 4919 里从起到 `.rc` 落盘约 6 分钟（按 `rerun-first/` 下 `b1-i31_sweep_4.rc` 与别的 `.rc` 的修改时刻差推的），新脚本每个成立或部分没判的世界多判一次别的不变量，估不超过 15 分钟，远低于 40 分钟；第 3 条跑出来超过 1 分钟就按 N = 3 与 N = 4 的世界数之比（`worlds=` 那一项）再估一次。
   - 看：`exit=0`；`model_` 开头的四个计数等于 `out/i31_sweep_4.txt` 第 1 行的 1237、129、4、1485（对不上整份作废）；再按第三节末的预期读 `quarantine_counted_` 那几项。
5. 命名检查不用跑：没改 `crates/`、`scripts/` 下的 `.rs`。

第 2–4 条跑完，派同立场的腿承接，把结果落进第一节那几格（第三节的「推的」改成量的或撤回）。

## 八、没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；Q3 打中那一形哪一读法对交主 agent。
- 没往统一任务服务交任务，没跑任何模型；本段的读数全来自任务 4919 的输出，判决要引的数由主 agent 照第七节第 1 条复跑。
- 没改前一条腿的模型文件与 `rerun.sh`；新加的那一份不在 `rerun.sh` 里，要并进去由主 agent 定（并进去之后件数变 20，参照要一起补）。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r2-opus-c2` 里只有认领文件与 `task-4919-log.txt`（`task-server/task log 4919 --all` 的输出，19 行比对用）与 `cite.txt`（`cite-check.py` 的输出）；`py_compile` 留下的 pyc 目录交回前删掉。没有仓副本、编译目录与库拷贝。
