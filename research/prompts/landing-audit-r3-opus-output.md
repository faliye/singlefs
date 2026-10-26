# landing-audit-r3 攻方腿（Opus）：K2「已定」项撑不撑得住、K3 报出的数能不能核（2026-10-23）

正文 `research/prompts/_landing-audit-r3-body.md`，背景材料 `research/prompts/_landing-audit-r3-background.md`，前两轮判决 `research/prompts/landing-audit-r1-main-verification.md`、`research/prompts/landing-audit-r2-main-verification.md`。开工时 `sha256sum -c research/prompts/landing-audit-r3-snapshot/judged-sha256.txt` 74 份全对上。这是第三轮：下面每一条改法都是这条腿推的，只在我的模型上量过、被攻过零轮。

## 复跑

模型目录 `research/prompts/landing-audit-r3-opus-model/`。这一轮没有冻结的仓副本，被判的脚本从 `--frozen` 读，给仓根：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/landing-audit-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新的空目录>
```

我跑的那一次退 0、127 秒（`out/rerun.log`），末尾与 `out/` 逐份比全是 same，只有 `fix-word-end.txt` 差 12 行（那一次的探针按字典遍历顺序打，后来改成排序后打，`out/` 里是改后的）。读的是会话记录（`~/.claude/projects/-home-fy5090-code-singlefs/`）与 git，会话记录在长：往后复跑，结局账闭窗口那几份不变（窗口闭着、两份存档 sha256 都在 `out/archive-sha256.txt`），真派发提示那几份（report-line-*、sentence-start-extra、negation-corpus、write-name-refusals、handback-shapes-today、fix-word-end）的份数会随新派发涨。`inputs/agent-cost-before-r2fix.py` 是第七批钩子工具员开工时存的原样（`~/.local/share/singlefs/work/landing-audit-r2fix-hooks/orig/agent-cost.py`，sha256 f1c67258…，与它报告第 41 行记的开工那一版一致）。`prompts/` 是从会话记录取的真派发首条提示原样（`probes/first_prompt.py`），带 `-own-result`、`-delivery-colon`、`-section-colon` 尾巴的是我在原样上改一处的变体（改了哪一处在 K2 各条写明）。

模型目录每个文件的 sha256（在模型目录里 `find . -type f | sort | xargs sha256sum`）：

```
f1c672582761a7b91920fe682dcbb2fcbf4d1bf30aa800ebb45f2d584d7d1243  ./inputs/agent-cost-before-r2fix.py
4f64987c606115e0d60274f27b034f51a1f769829672e2551b4c97af1351b749  ./out/archive-sha256.txt
b41d21795348b058ec1d36fa13c60733213abf138624cfdb56769d9284ca77d8  ./out/before-r2fix-utc-agents.jsonl
f409bd11533b9350e18418fa238304ff7f03ed6a26c446b588a0fe5d3b8cdb04  ./out/before-r2fix-utc-summary.txt
4391baeaf30d01864a1bc32ab465dd91ae81a8b0f7719c7123ff6afdc865e725  ./out/decompose.txt
8a2262dc95dc2b0b89a893135c687d112a7c0893e757e8ce048fcef5759ace02  ./out/dispatch-check.txt
d111ee3db3a18d78deeea7756c1bdb5fb0af80e549a537b21a04ef7386c43555  ./out/fix-word-end.txt
568c8de918d7dbaab1b047ba1dbb0f183f82b4421b743e882c7c0e75e5d5cae1  ./out/handback-shapes-today.txt
3f543f673bf05cde421ada60d579ab8e155ca0bc0e824a7a9c8846748d91ca4b  ./out/leg-own-report.txt
8c4eaf4b75005babe5d3070c2db75910370ff27911efed2231207dbf641438c3  ./out/name-verdict-variants.txt
8ee62f185d8ebf521a716add79e909c0148e3ab27c5f616e606a02f8c0471f8b  ./out/negation-corpus.txt
c58d17b5f0157dd96208b39acfe62187880f8139db50037884666bab7012ed3d  ./out/r2-cite-check-rebuilt.txt
daf875cf11e5c71ad2e3adc45d1481b6bef53e9fb59b5946d1f2b42e14342d15  ./out/r2-rebuild.txt
a100df5311576083156fa164c5791432acd7c8700727718021126f64068e6d7c  ./out/report-line-corpus.txt
36f83382dceb4aafe43e1e65580d5d087129a72b853e37c8cf7dd23e9230d4a9  ./out/report-line-examples.txt
f14cd34d4be95b8983784c250ac5c864f9ca8e37014480971f1a3c1ffe9e9aba  ./out/rerun.log
e494af439178170b72505de814ce702c43bc0e72f4daad5c6e77937e8b680273  ./out/sentence-start-extra.txt
1e0302ffcbca4fb712592bed6b008e1581ff3379ef855bc21919f2031da59621  ./out/threshold-sweep-before-r2fix.txt
8cfc3cd0682d2f677d6602566f35ded4cf48791fad4af7ea934f80f57090852f  ./out/threshold-sweep-today.txt
b2abfe5c0fdea1e364760b7bc8930cb7966f4b94f226f1bda05d34321e2f5149  ./out/today-tokyo-agents.jsonl
d0117f83c1a9faeb287fd878a69acee68cc1f30eebccb7a1462e5734e60a5226  ./out/today-tokyo-summary.txt
c62d29c4ae0ea65f1cc266177ddddc028bd12af083673725dc91897cc0539bae  ./out/write-name-refusals.txt
1cac8687e8ba05bb7e96c6e95019af493129e04a013fd9e6f7d5c528c5958622  ./out/write-refusals.txt
5dac3778c63a01eb7ad267f55a5bb4cbf51a9b0fac2ce993ae12a1ea4e42aa8b  ./probes/decompose.py
1d192b0748f9944f3906316fe24b670fba40103c3af60613c4cad94752d31c90  ./probes/first_prompt.py
4bd1028ccbd3f27b88ad9f9d3c8c89b147581569bb0beff1d0e18706e1dfa092  ./probes/fix_word_end.py
44046fbcdf1b599342a0b9ed6ae3be4d5421248680f089b3606f8df3e3bd56ae  ./probes/handback_shapes.py
6e91afe821861304d37d13a0b40ad98b8c255e136526665431576762ed2250f6  ./probes/leg_own_report.py
90e5484ad518a32b872420e99521e04917ad405c8d390c317264d381f357ed4b  ./probes/name_verdict_variants.py
e034b5d2674157736a69d74e949cd57b84c483fadb5db2efeabaa8dbe91dfcc3  ./probes/negation_corpus.py
82d89e7b901677269969b0e928729db82ab78af2de2a082dad5fbce2c229f843  ./probes/outcome_dump.py
fd5f0c84333684a444fb00a4a115ffa720e46dd413d662799eb8b12f78baff8e  ./probes/rebuild_snapshot.py
ffec8d07146d0666aac34dba48e7ae32427006e62ccecb6680ff56a7e495d49c  ./probes/report_line_corpus.py
3d378a943fbafa2f5d7c8f02687feb83b9dacb513f9cf31ed5fe0a3e564e21d1  ./probes/report_line_examples.py
2cae785336d2f6f2f908dc6aa3d3ccfe1415ae81e26f7dffbb425a690a27b878  ./probes/sentence_start_extra.py
77fed1a726d635f29c7d6cb8ba864f92d4310e2b1b6053fc5576806c59b1459c  ./probes/threshold_sweep.py
43973c2b392ee561573a2247cf625fa3ad8ea9ec5222cece465c5e3af407cc74  ./probes/write_name_refusals.py
a245e13e5900d7633b8f54c69d9e8cd4ab1be7b683d1a53d3923824a5fead03e  ./probes/write_refusals.py
f38acb831b6696f967c02caa39ed99ab70a2250bf49d4c6817136a02c6635b0d  ./prompts/a0a2dfee322e26786.md
537865dd7d924786d4633c60888070975639e4ff2c9a1c5c1e508ee5dcb12ebb  ./prompts/a28fe10b58d88beec.md
e021deda1917b2dbd91c814040ff7dfb93ff82eb1af7124d2edbb4410d2c2bd4  ./prompts/a303d33b0ccd5e68f.md
9832e848f67c4eef8878e60190dba887f304e3478835d9c45703aaca712f7c87  ./prompts/a3652270eb80e4fd0-delivery-colon.md
7155650c7fea1113699845bd7df795a9aaf99dd674d74fc78b985016bcc38156  ./prompts/a3652270eb80e4fd0.md
4dda75158e387a8c7b1c40719210ace881515d302d3e22d20caedad0b9624105  ./prompts/a3652270eb80e4fd0-section-colon.md
089035c008e9c601d71f245626dfef6aff114491c7f9a5c0f63e41f5b0e3ddb2  ./prompts/a4737c6f3660f4f7e.md
c469195bef2c16482a1aded21a490a7a36f2009ce7a66b902247e3483c498c2c  ./prompts/a4737c6f3660f4f7e-own-result.md
d93588072534e3b3f246b8c587c32697f8adb9a8699ea71f83ef6711cf3c9378  ./prompts/a789e7091d91430aa.md
e6604276c0d2617952e59d0a6dfa59a446b283ebf292ae20e7e02a99d2162408  ./prompts/a86c5c8fb8ec28789.md
dfe14fd65f857490f9534b49d9256ef2a752b5c344e90303c792c9e8d87ff1d9  ./prompts/a8afb2185cfa840cc.md
6ffe7b19d753d8eae9f73c4c00e83727ebe3f371e15c83e682ad2732855b752c  ./prompts/a9717cf07fb6e7c21.md
d365fba837d920d040f7c38cf6ce04a11f0e391505448669493fc07d8369fd81  ./prompts/aa9ec3423c1f54bba.md
7bca8a78304266e42377a376b7c21386d6ed4cb793a13ecadb510043937ebb9e  ./prompts/aa9ec3423c1f54bba-own-result.md
bbe1dd6cb50a3cac18c00c8e50548c977393661a553f52b1aaf8308d88f154bb  ./tools/rerun.sh
```

## 各格判定一览

| 格 | 条 | 判定 | 一句话 |
|---|---|---|---|
| K2 | K2-1 派发闸「句首也认」（派发外） | 打中 | 正文里「。」「；」之后转述别的子 agent 的 `…/report.md` 被当成报告行，⑰ 拒；2026-10-14 起的真派发提示里 3 份（实现员、写回员、执行员各一），把它们自己那一行改成 result.md 之后今天照拒，「只行首」认法下放行 |
| K2 | K2-2 判决 K2-3 的「逗号之后不认」 | 打中（规格本身） | 真提示里自己的报告行写在逗号之后（「，报告写到 `…/report.md`」）的，今天 ⑰ 不判、放行；全部 3292 份里 78 份，2026-10-14 起 7 份，其中 4 份点名的就是自己的 report.md（2026-10-22 别的会话 3 份）；那几份子 agent 没用 Write 写 report 名字的文件，工具层拒写没真发生 |
| K2 | K2-3 三方腿报告形态（第五批立的判法） | 打中（轻） | 「交付：报告 research/prompts/…-output.md」「「交付」：报告写进 …」两种真用过的写法今天判「一行报告行都没有」拒；真提示 27 份，全在 2026-10-11 及以前，之后 0 份 |
| K2 | K2-4 报告行紧跟否定词不判（派发外） | 没打中 | 3292 份真提示上与「否定词也判」逐份同判（0 份不同） |
| K2 | K2-5 `--since` 只认 `YYYY-MM-DD`（派发外） | 没打中 | 仓里写到的调用全是日期写法，代码里没有别的调用方 |
| K2 | K2-6 库的 `git show` 改按当前目录取（派发外） | 没打中 | 调库而不传 cwd 的格都先 `cd "$ROOT"`；在仓顶层两种写法取同一份 |
| K2 | K2-7 `.claude/kb-directories` 写 `.` 也拒（派发外） | 没打中 | 真登记位两行都是子目录，不碰这一判 |
| K2 | K2-8 判决 K2-2 甲：只缩不涨的键 | 打中（轻，文字） | 代码与样本改成四样进键，登记表自己的文件头第 9 行还写「按登记出处、留存产物名成对」；照文件头加行的执行员读到的是旧键 |
| K2 | K2-9 判决 K4-2：doc-registries 改起 python | 打中（轻，推的） | 工具员报告写「python 起不来…那几格会因为找不到不变量表而红，不静默变绿」；读代码 inline-sums、invariant-count-elsewhere 两格拿空候选退 77（本次未跑），不红；阶段被重型测试闸拒、没实跑 |
| K2 | K2-10 判决 K3-2：三方腿两桶 | 没打中（真数据）；构造得出 | 认的是交回里点名的任一 `-output.md`，不是这条腿自己那一份；真数据 463 条里用派发闸的报告行解析取自己那一份，446 条自己的报告与本轮判决都在、0 条靠别的报告记有效、17 条取不到报告行 |
| K2 | K2-11 判决 K3-4 甲：`\bfix` 尾词界 | 没打中 | 全部 3292 条描述里只因尾词界不再算返工的 11 条：fixture 一族 6 条、轮名 `gate-fix-forks-r*` 5 条，没有真返工被漏 |
| K2 | K2-12 第五批判法：续做闸与看门狗的交回定义 | 没打中 | `continuation-guard.sh` 20 种、`subagent_transcript.py` 12 项照绿；交回结果仍只三种形状 |
| K2 | K2-13 `implementation-writer.md` 第 4 步 | 打中（轻，措辞） | 同一步里「验证只到这几样：」那串列举没有命名检查，后一句又把命名检查从「都不跑」里除外；与共用约束、钩子不矛盾，同一步自己前后两句对不齐 |
| K3 | K3-1 闭窗口结局账 2144 对 2125 | 没打中（解释得全） | 逐个子 agent 对齐：旧窗口独有 45 个全在东京 10-22 那 9 小时，新窗口独有 26 个全在东京 10-01 零点到 9 点，共有的 2099 个桶一个没变；三方腿分桶贡献 0；返工 327→313 = −10 −6（fixture）+2；两版存档 sha256 今天都原样复出 |
| K3 | K3-2 门槛换值表在新版上 | 复得出，一格差 0.1 | 10 档里「下沿 50% 时 3.4%」「70%→90% 由 50.0% 掉到 35.0%」在新版上照旧；(0.9, 0.7) 由 6.2% 变 6.1%；调度记录第 138 行把闭窗口的换值读数写在 2268 个那一次的句子里，两个窗口混在一句 |
| K3 | K3-3 各条自证的种数与两份真提示 | 复得出 | 派发闸 460、交回闸 23、agent-cost 18、dispatch-prompt 90、续做闸 20、会话记录库 12、lib-lag-baseline 9；两份真提示 `--check` 一份退 0、一份退 3 只剩认领行 |
| K3 | K3-4 第二轮判决「`--root .` 与快照根同义」 | 打中（轻） | 第二轮快照没收 `doc-decisions.sh`、`handback-guard.sh`、`quote-kb.py`，两条腿引了它们；今天用活仓跑出 24 处对不上（全在第七批改过的文件上）；我按快照 diff 头的改前 blob 重建出快照树（63 份全对上），以它为根只判得了 39 处 |
| K3 | K3-5 第二轮判决「没核动的」那组 45 个、1.66% | 复得出 | 45 个、1.66%（`out/decompose.txt`） |
| K3 | K3-6 记录里别的数 | 复得出 / 复不出各列 | 单元表 19、登记 13 种单元名（14 行）、快照 74 份、清单抄 3 行复得出；2268 那一组记录自己写了复不出 |

## K2　「已定」项撑不撑得住

派发闸三处（K2-1 到 K2-4）的读数都来自同一套装置：把 `.claude/hooks/runner-dispatch-guard.sh` 内嵌的 python 抽成模块载入（不跑 main），在本项目全部子 agent 会话记录的首条派发提示（3292 份）上，分别用三种报告行认法起 ④ 写范围、⑰ 文件名、⑰ 三方形态：今天（行首或句首）、只行首（第二轮判决 K2-3 写的那一种）、不锚（第五批）。`out/report-line-corpus.txt` 原样的计数行：

```
prompts 3292
today_vs_line_only:form:today=False:other=True 33
today_vs_line_only:name:today=True:other=False 172
today_vs_unanchored:form:today=False:other=True 2
today_vs_unanchored:form:today=True:other=False 27
today_vs_unanchored:name:today=False:other=True 78
today_vs_unanchored:scope:today=False:other=True 28
```

### K2-1　「句首也认」把正文里转述别人的 report.md 当成报告行（打中，派发外改动带出来的误拦）

认法在 `.claude/hooks/runner-dispatch-guard.sh:1253`「REPORT_LINE = re.compile(r"(?P<lead>^[ \t>*_\-]*(?:\d+[.)][ \t]+[ \t*_]*)?|(?<=[。！？；])[ \t*_]*)" + REPORT_LABEL」。判决要的是 `research/prompts/landing-audit-r2-main-verification.md:70`「改成报告标签只在行首认（列表记号、粗体之后也算）」；句首那一支是工具员加的，理由在 `research/prompts/subagent-landing-audit-2026-10-22/r2fix-hooks.md:61`「认句首。攻方腿那份真提示的报告行写在一行中间」。正文 `research/prompts/_landing-audit-r3-body.md:47` 问的正是「句首也认之后，正文里「。报告：」一类的转述会不会又被当报告行」。

会。`probes/sentence_start_extra.py` 列出 2026-10-14 起开工的真提示里「句首多认、点名带 report 的文件」12 处，其中点名的不在这份提示自己的草稿目录下、读上下文是在引别的子 agent 报告的 3 处（`out/sentence-start-extra.txt` 原样）：

```
  2026-10-16 aa9ec3423c1f54bba implementation-writer 与草稿目录同棵=False | 、logged_ops 种类码 2、上段叶标签 3 与 4；报告 `~/.local/share/singlefs/work/m4-trunc-core-writer/patch/report.md` 第十节列了格式常量的派生形态）：
  2026-10-20 a28fe10b58d88beec kb-scribe 与草稿目录同棵=False | 04 → 1678377220，到 D 36 → 38 段。报告 `~/.local/share/singlefs/work/m3-close-w9-landing-pins/report.md`，数去补丁现查。
  2026-10-20 a4737c6f3660f4f7e experiment-runner 与草稿目录同棵=False | 1c27958e8d，64 条单测，编过、clippy 0；报告 /home/fy5090/.local/share/singlefs/work/m4-gpugc-runner-7/report.md）。
```

（其余 9 处里 5 处同棵、3 处写「草稿目录 report.md」、1 处 a86c5c8fb8ec28789 读不出是不是自己的，都当它自己的报告行，⑰ 拒是对的。）这三份当时自己的报告行也写 report.md，拒是两处并在一起的。把它们自己那一行改成 result.md（照今天的写法，转述那一句一字不动），今天的 `dispatch-prompt.py --check`（`out/dispatch-check.txt`）：

```
== aa9ec3423c1f54bba-own-result
✗ 派 implementation-writer 的提示报告行点名的文件名里带 report：「~/.local/share/singlefs/work/m4-trunc-core-writer/patch/report.md」。
== a4737c6f3660f4f7e-own-result
✗ 派 experiment-runner 的提示报告行点名的文件名里带 report：「/home/fy5090/.local/share/singlefs/work/m4-gpugc-runner-7/report.md」。
```

两种认法对照（`out/name-verdict-variants.txt`，行尾截了）：

```
name=aa9ec3423c1f54bba-own-result.md 认法=今天（行首或句首） ⑰文件名=拒 ✗ 派 implementation-writer 的提示报告行点名的文件名里带 report：「~/.local/share/singlefs/wo
name=aa9ec3423c1f54bba-own-result.md 认法=只行首 ⑰文件名=放行 
name=a4737c6f3660f4f7e-own-result.md 认法=今天（行首或句首） ⑰文件名=拒 ✗ 派 experiment-runner 的提示报告行点名的文件名里带 report：「/home/fy5090/.local/share/singlefs/wo
name=a4737c6f3660f4f7e-own-result.md 认法=只行首 ⑰文件名=放行 
```

写回员那一份（a28fe10b58d88beec）自己的报告行在行首、也写 report.md，两种认法都拒，转述多拒一处，没另改。

另一面：只行首会把 33 份真三方提示判「一行报告行都没有」（计数行 `today_vs_line_only:form:today=False:other=True 33`，第一轮攻方腿 a3652270eb80e4fd0 就在里面），句首那一支救的就是它们。所以行首、句首两种锚法各有真提示上的误拦：锚在哪儿都分不出「这是自己的报告行」与「这是在引别人的报告」。

四句：不分辨臂（这里没有臂，是一条判法的两种读法）；判别子观测得到：提示里另有「草稿目录：」一行，转述的那 3 处路径都不在它下面（同棵=False），派发闸当时看得到；满足的是正文 K2 第一条那一问的字面；改法：⑰ 只判报告行里落在本提示「草稿目录：」下的路径与不带目录的裸文件名（`report.md`），别的目录下的不判（推的：12 处里按这一条判，3 处转述放行、5 处同棵与 3 处裸名照拒，a86c5c8fb8ec28789 那一处要看它的草稿目录，没核；只在我的模型上量过、被攻过零轮）。推翻条件：这三份提示里那一句其实是派它写进那个路径（读原提示上下文，三处都是「…第十节列了」「数去补丁现查」「编过、clippy 0；报告 …」这种引出处的写法）。

### K2-2　逗号之后的真报告行，今天 ⑰ 不判（打中，判决规格本身带出来的误放）

计数行 `today_vs_unanchored:name:today=False:other=True 78`：第五批的不锚认法拒、今天放行的 78 份，`out/report-line-examples.txt` 汇总行：

```
== name_passes_today 78; 开工于东京 2026-10-14 及以后的 7; 其中会话里有工具层拒写的 2
```

（「工具层拒写的 2」这一列粗：它只数会话里有没有出现那句拒写原文，那两份是 a303d33b0ccd5e68f 与 a3652270eb80e4fd0，它们读到的文件里抄着那句话，不是自己的 Write 被拒；精确的核在下面。）2026-10-14 起 7 份里，4 份点名的是它们自己的 report.md、写在逗号之后：a9717cf07fb6e7c21（2026-10-14 工具员，「（自己建，排他），报告 `…/tooling-two-stage-followup/report.md`」）与 2026-10-22 别的会话派的三个设计员（a0a2dfee322e26786、a789e7091d91430aa、a8afb2185cfa840cc，都是「，报告写到 `…/report.md`：…」）。今天与第五批两种认法（同一个模块，只换 `REPORT_LINE`）：

```
name=a0a2dfee322e26786 experiment-designer 今天=放行 第五批不锚=拒
name=a789e7091d91430aa experiment-designer 今天=放行 第五批不锚=拒
name=a8afb2185cfa840cc experiment-designer 今天=放行 第五批不锚=拒
name=a9717cf07fb6e7c21 tooling-writer 今天=放行 第五批不锚=拒
```

（这四行是我现跑的一段内联 python 的原样输出，与 `probes/name_verdict_variants.py` 同一种载入法，只把对照换成 `REPORT_LINE_UNANCHORED`，没收进模型；同一判法的整批数在 `out/report-line-examples.txt`。）这四个子 agent 有没有因此被工具层拒写：`probes/write_refusals.py` 逐个找对名字带 report 的文件的 Write 与它的工具结果，四份都是 0 次 Write（`out/write-refusals.txt` 只打出提示行，没有一行 Write），所以这一形的害处在真数据上没发生。

四句：不分辨臂；判别子观测得到（同 K2-1，路径在「草稿目录」下）；满足的是正文 K2 第一条「做没做到第二轮判决那一条要的」——做到了，判决自己的规格（逗号之后不认）在真提示上放过了自己的报告行；改法：同 K2-1 那一条（报告标签在哪都认，按路径是不是落在本提示草稿目录下、是不是裸名来定判不判），它同时修 K2-1 的误拦与这里的误放（推的，被攻过零轮）。推翻条件：工具层只拦 Write 不拦 Bash，而这几类子 agent 一律经 Bash 写报告——那 ⑰ 拦的东西本来就不发生，这一条只剩措辞。

### K2-3　三方腿报告形态：「交付：报告 …」写法被判「一行报告行都没有」（打中，轻）

计数行 `today_vs_unanchored:form:today=True:other=False 27`：27 份真三方提示第五批放行、今天拒；汇总行「== form_refused_today 27; 开工于东京 2026-10-14 及以后的 0」。写法两种：「交付：报告 research/prompts/m3-formal-code-b30-r1-opus-output.md（分段落盘…」与「照正文第五节「交付」：报告写进 research/prompts/m3-w3-r1-opus-output.md，分段落盘…」，标签前面是全角冒号，不是行首也不是句首。最小复现：拿第一轮攻方腿那份今天放行的真提示，只把「报告：research/prompts/landing-audit-r1-opus-output.md」换成这两种写法（`out/dispatch-check.txt`）：

```
== a3652270eb80e4fd0-delivery-colon
✗ 派 three-way-attack 的提示一行报告行都没有（或报告行里没有点名文件）；三方腿的报告要形如 research/prompts/<轮>-<模型>-
== a3652270eb80e4fd0-section-colon
✗ 派 three-way-attack 的提示一行报告行都没有（或报告行里没有点名文件）；三方腿的报告要形如 research/prompts/<轮>-<模型>-
```

四句：不分辨臂；看得到（路径就在那一行）；满足的是正文 K2 第二条「第五批立的判法（三方腿报告形态）有没有被改坏」——拒绝说明会把人引到骨架写法，派不出的只是一次重写，2026-10-14 之后没人这么写，所以记轻；改法：三方形态那一判对整份提示里 `research/prompts/<名>-output.md` 形的路径也认（只要一处，且不在「」与反引号里）（推的，被攻过零轮）。推翻条件：主 agent 往后只用骨架起三方提示。

### K2-8　登记表文件头还写旧键（打中，轻，文字）

判决 K2-2 甲要键改成（实验号，登记出处，单元名，留存产物名）。门禁里改了：`.claude/gate.d/doc-experiments.sh:2400`「# 只缩不涨：复跑写 none: 的行按（实验号，登记出处，单元名，留存产物名）与基准提交那一版登记表里复跑写 none: 的行比」。登记表自己的文件头没改：`.claude/gate.d/replay-comparison-units.tsv:9`「格拿它与 gate 基准那一版里复跑写 none: 的行比（按登记出处、留存产物名成对），基准里没有的判红」。这份表不在第七批的文件清单里（判决「三、判决」第一张表只点了门禁与样本），工具员也没报。加行的是执行员（`.claude/gate.d/stage-owners.tsv` 的归属），它读的是表头。四句：不分辨臂；看得到；满足的是正文 K2 第一条「做没做到」——判法做到了，表头这一处的说明没跟上；改法：第 9 行括号改成「按（实验号，登记出处，单元名，留存产物名）」（推的，被攻过零轮）。推翻条件：表头这一句不算给人照着加行的说明（例如执行员定义另有一句写全了键）——我在 `.claude/agents/` 下 grep「成对」0 处。

### K2-9　doc-registries 改起 python 之后「起不来会红」只对一半（打中，轻，推的）

工具员报告 `research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md:417` 写「那几格会因为找不到不变量表而红，不静默变绿」。读代码：`kb_candidate_roots` 起不来时只往 stderr 打 `.claude/gate.d/doc-registries.sh:3282`「这一格的 kb 候选目录是空的」，返回 1 被 `< <(…)` 吞掉；六格里 declared-counts 那一族拿空候选会红在「候选 kb 目录里一个带不变量表行的文件都没扫到」，可 inline-sums 走到 `.claude/gate.d/doc-registries.sh:3813`「print(f"  ! 本次无对象可判：扫了 {scanned} 份 kb 文件，一条「整数 = 项 + 项 + …」的算式都没有")」之后退 77，invariant-count-elsewhere 走到 `.claude/gate.d/doc-registries.sh:3529`「没有一处写「N 条在用」的声明」那一句之后同样退 77。退 77 是「本次未跑」，不是红。我在副本里把 `quote-kb.py` 头上加一行 `raise SystemExit` 想实跑六格，命令被重型测试闸当「提交时才跑的检查」拒了（原样拒绝说明的头一句：「✗ 提交时才跑的检查被拒：门禁阶段 doc-registries.sh（扫仓的门禁归提交前主 agent 跑的 scripts/gate/gate-staged.sh）」），没绕，这一条只读了代码。四句：不分辨臂；看得到（stderr 那一句在）；满足的是正文 K2 第一条「派发外改动有没有带出新的误放」——这一处是判决 K4-2 那一份的改法，退 77 而不是红；改法：`kb_candidate_roots` 起不来时那几格直接判红（函数退非 0 时调用方 `exit 1`，不经 `< <(…)` 吞），交主 agent 照「要主 agent 跑的命令」第 1 条在暂存后带一份「quote-kb.py 起不来」的红样本核（推的，被攻过零轮）。推翻条件：阶段级把格里 stderr 带「✗」的行算红（我没读到这样的汇总规则）。

### K2-13　`implementation-writer.md` 第 4 步：列举里没有命名检查（打中，轻，措辞）

第 4 步是一整行（`.claude/agents/implementation-writer.md:40`）。同一行里先写「验证只到这几样：第 1d 步的形式化轻型」，列到 fmt、clippy、build 为止，没有命名检查的两条；后面又写「门禁阶段（第 1d 步形式化那一道门禁的 `--quick`、共用约束「门禁」一节的命名检查除外）」，即命名检查要跑。按「验证只到这几样」读会不跑，按「除外」读要跑；共用约束 `.claude/agent-common.md`「命名检查交回前跑」一条点名了 implementation-writer，共用约束第 3 行又写与定义冲突时以定义为准。与钩子：heavy-test-guard 只放行 `--check` 全在四个命名格之内的那一种，与这一步不冲突；派发闸不判这一句。四句：不分辨臂；满足的是正文 K2 第三条「与共用约束、钩子有没有矛盾」——与共用约束、钩子不矛盾，是这一步自己两句对不齐；改法：「验证只到这几样」那串列举末尾补「共用约束「门禁」一节的两条命名检查（rsync 副本里改的写进「要主 agent 跑的命令」）」（推的，被攻过零轮）。推翻条件：实现员的真交回里，这一步之后都照跑了命名检查（我没扫会话记录核这一点）。

## K2 里没打中的几条（查了什么）

- K2-4 报告行紧跟否定词不判（`.claude/hooks/runner-dispatch-guard.sh:1257`「REPORT_NAME_NEGATION_BEFORE = re.compile(r"(?:不|别|勿|禁止)[^，,；;。（）()]{0,4}$")」）：`probes/negation_corpus.py` 在 3292 份真提示上比开着与弄坏开关 `report-name-negation-judged` 下的 ⑰，`out/negation-corpus.txt`：「提示 3292 份，两种判法不同 0 份」。构造得出的放过：「报告：不变量的 report.md」（「不」后隔 3 个字），真提示 0 份，不记。
- K2-5 `--since` 只认日期（`scripts/agents/agent-cost.py:1587`「elif flag == "--since" and rest and re.fullmatch(r"\d{4}-\d{2}-\d{2}", rest[0]):」）：全仓 `.md`、`.sh`、`.py`、`.tsv`、`.json` 里写到 `agent-cost … --since` 的 72 行（`grep -rnE` 数的，去掉 landing-audit 各轮腿报告），值是 `YYYY-MM-DD`、`<开工日期>`、用法行里的 `…` / `YYYY-MM-DD`，另两处是前一轮腿报告转述（「--since 同一种切法」、git log 的 `--since={begin}`），不是调用；`scripts/`、`.claude/` 下引 agent-cost 的四份代码（`subagent_transcript.py`、`agent-watch.py`、`code-tooling.sh`、`continuation-guard.sh`）都不带 `--since` 起它。没有调用会断。
- K2-6 库的 `git show` 改成 `<基准>:./<路径>`（`.claude/gate.d/lib-lag-baseline.py:119`）：调库不传 cwd 的五处（doc-process-records、doc-decisions、doc-experiments 两处、code-source-discipline 两处）所在的阶段都先进根（例 `.claude/gate.d/doc-experiments.sh:167`「cd "$ROOT" 2>/dev/null ||」），传 cwd 的两处传的就是根；在仓顶层 `./x` 与 `x` 是同一个对象。只在当前目录是仓的子目录时两种写法不同，而那时 ls-tree 本来就按当前目录认，改后是对齐。
- K2-7 `.claude/kb-directories` 写 `.` 也拒：真登记位两行是 `.claude/kb` 与 `crates/singlefs-checker/invariants`，不碰这一判；没造新写法再攻（第二轮 K4-2 已攻过写法这一面）。
- K2-10 三方腿两桶：`scripts/agents/agent-cost.py:1002`「for stem in sorted(set(LEG_OUTPUT_MENTION.findall(handback_text or ""))):」——交回里点名的任一 `-output.md` 在仓、同轮判决在，就记有效，不核那是不是这条腿自己的报告。构造：一条腿自己的报告没留下，交回里又引了前一轮的报告，就进 verdict-archived。真数据：`probes/leg_own_report.py` 用派发闸的报告行解析取每条腿首条提示里自己的 `-output.md`，`out/leg-own-report.txt` 首行「{'legs': 463, 'own_report_and_own_round_verdict': 446, 'own_report_known': 416, 'own_round_verdict_known': 416, 'own_report_not_found_in_prompt': 17}」：0 条靠别的报告记有效（没有 credited_by_another 一项），17 条提示里取不到报告行（看了 17 条里列出的几条，结局账认到的都是同名同轮的那一份）。own_report_known 416 比 446 少，是我的探针只读主工作树与 git 历史、不读别的工作树，结局账读了。
- K2-11 `\bfix` 尾词界：`out/fix-word-end.txt` 首行「描述 3292 只因尾词界不再算返工的 11 {'fix-forks-r1': 1, 'fix-forks-r2': 2, 'fix-forks-r3': 2, 'fixed-script': 1, 'fixture': 3, 'fixture-claims': 1, 'fixtures': 1}」，`fix-forks` 是轮名 `gate-fix-forks-r*`（「核 gate-fix-forks-r2 三条腿」），都不是返工；fixing、fix-up 这类改前算、改后不算的写法在全部描述里 0 条。
- K2-12 第五批判法：续做闸与会话记录库自证在副本里 20 种、12 项照绿（K3-3 那张表）；交回结果的形状今天仍是三种（`out/handback-shapes-today.txt`：「files 3292」，3082 条 success=True、193 条 is_error、2 条 success=False）。replay-comparison-units 的形状判：读代码，第七批只换了 none 行的键与红行明细写法，行形状正则没动；阶段被闸拒、样本没跑，靠工具员报告里的四份样本读数。
- 设计员第 3a 步 ② 与「实验开跑之前，答案不许已经存在」那一节的张力：第一轮 K3-7、第二轮辩方 K3-7 已攻过，不重复。
- 共用约束第 108 行：第七批只删了「或名字里带 report 的文件」半句。`probes/write_name_refusals.py` 扫全部 Write：以 report 开头的 `.md` 被拒 217 次、写成 2 次，中间带 report 的 `.md` 写成 3 次（第二轮辩方的读数同向）；以 summary、findings、analysis 开头的 `.md` 一次都没写过（`out/write-name-refusals.txt`），那三样「会被拒」没有数据，但这三样不是第七批改的，只记限度。

## K3　报出的数能不能核

### K3-1　闭窗口结局账 2144 个与 2125 个：差由切天方式解释全，三方桶与词表不动个数（没打中）

两版都今天现跑：今天的 `scripts/agents/agent-cost.py` 按东京日期切，第七批开工前那一份（`inputs/agent-cost-before-r2fix.py`）按 UTC 切，同一份 `probes/outcome_dump.py` 逐个子 agent 落盘。两版各自的汇总与存档逐行相同：今天那一份与 `research/prompts/subagent-landing-audit-2026-10-22/outcome-summary-2026-10-01-to-2026-10-21.txt` 只差主程序自己打的末行成功句，改前那一份与第二轮攻方模型的 `out/closed-summary.txt` 逐字节相同。各补上那一版的成功句之后的 sha256（`out/archive-sha256.txt`）：

```
6dde89a84129fa1b3b22da055372594d71bb11b262df5f2979c20db5a92e6ab0  -
083c9216dd4c96ee09d92d7e296629b60d07b2b001cb799d75c8bc1a911cf841  -
```

即调度记录 `records/2026-10-22-子agent结果落地审计.md:178` 写的两版（「按系统时钟日期切的第一版 sha256」6dde89a8… 与新版 083c9216…）今天都原样复出。本机系统时钟是 UTC（`timedatectl` 报 Etc/UTC），所以「系统时钟日期」就是 UTC 日期。逐个对齐（`probes/decompose.py`，`out/decompose.txt` 原样）：

```
name=counts old 2144 new 2125 only_old 45 only_new 26 both 2099
name=only_old_in_top_edge 45 of 45
name=only_new_in_bottom_edge 26 of 26
name=cost old 7076.36 new 7004.91 only_old 117.13 (1.66% of old) only_new 45.68 (0.65% of new)
name=bucket_table  bucket old new | minus_only_old plus_only_new | changed_in_both_net
  cut-by-api-error                           2     5 | -  0 +  3 |   +0 ok
  cut-by-usage-limit                        36    36 | -  0 +  0 |   +0 ok
  report-unreferenced                        9     9 | -  0 +  0 |   +0 ok
  report-used-by-later-sub                   5     5 | -  0 +  0 |   +0 ok
  report-used-by-main                      609   591 | - 23 +  5 |   +0 ok
  stopped-abandoned                         14    15 | -  0 +  1 |   +0 ok
  stopped-salvaged                          51    51 | -  0 +  0 |   +0 ok
  verdict-archived                         461   463 | -  2 +  4 |   +0 ok
  written-landed                           847   842 | - 17 + 12 |   +0 ok
  written-overwritten                       23    22 | -  2 +  1 |   +0 ok
  written-partial                           87    86 | -  1 +  0 |   +0 ok
name=changed_in_both {}
name=rework old 327 new 313 only_old 10 only_new 2 flip_in_both {'True->False': 6}
```

读法：top_edge 是 [2026-10-21T15:00Z, 2026-10-22T00:00Z)（东京 10-22 零点到九点），bottom_edge 是 [2026-09-30T15:00Z, 2026-10-01T00:00Z)。个数 2144 − 45 + 26 = 2125，全是切天；两版都有的 2099 个子 agent 桶一个没变，所以「三方腿分两桶」在这一窗口里一个也没挪（新桶 0）、门槛与落地读数也没漂；verdict-archived 461→463 是窗口两头 −2 +4。返工字样 327→313：窗口 −10 +2，另 6 个是尾词界（6 个全是 fixture 一族与 fixed-script，`out/decompose.txt` 下面逐条列了描述）。第二轮判决 `research/prompts/landing-audit-r2-main-verification.md:115`「攻方 K2-4 的「45 个、1.66%」是腿在草稿里按」那一条主 agent 没复算的数，这里复出：45 个、1.66%。推翻条件：换一台不在 UTC 的机器跑改前那一份，个数不是 2144（那时「系统时钟日期」就不是 UTC 日期了，调度记录那几个字要改成「UTC 日期」才准）。

### K3-2　门槛换值表在新版上（复得出；一格差 0.1；记录里窗口混写）

第二轮攻方的 `probes/threshold_sweep.py`（原样拷进我的 `probes/`）对改前那一版逐个子 agent 的落盘跑，与第二轮的 `out/threshold-sweep.txt` 逐字节相同（`out/threshold-sweep-before-r2fix.txt`）；对新版跑（`out/threshold-sweep-today.txt` 原样）：

```
LANDED_AT PARTIAL_AT WRITTEN_MINIMUM | 落地个数 落地花费 | 部分个数 部分花费 | 没效果几桶个数 没效果花费
0.7       0.3        10              |    842   50.0% |     86    4.2% |       87    2.0%
0.9       0.3        10              |    610   35.0% |    318   19.2% |       87    2.0%
0.5       0.3        10              |    897   52.8% |     31    1.4% |       87    2.0%
0.7       0.5        10              |    842   50.0% |     55    2.8% |      118    3.4%
0.7       0.1        10              |    842   50.0% |    100    4.7% |       73    1.4%
0.9       0.7        10              |    610   35.0% |    232   15.0% |      173    6.1%
0.95      0.9        10              |    411   21.8% |    199   13.3% |      405   21.1%
0.7       0.3        1               |    963   53.3% |    106    4.4% |      141    3.1%
0.7       0.3        30              |    737   47.2% |     70    3.6% |       76    1.7%
0.9       0.7        1               |    723   38.1% |    240   15.2% |      247    7.5%
```

（首行「原门槛重算与账里桶不同的个数 0」。）记录与判决引的三个读数：下沿 50% 时 3.4%、上沿 70%→90% 由 50.0% 到 35.0%，新版照旧；第二轮判决 `research/prompts/landing-audit-r2-main-verification.md:78`「(0.9, 0.7) 是 6.2%」新版是 6.1%（窗口挪了半天）。另一处：调度记录 `records/2026-10-22-子agent结果落地审计.md:138` 那一句报的是 2268 个那一次（只带 `--since`），括号里「下沿 30% 抬到 50% 时是 3.4%」是闭窗口（2144 个）上量的，两个窗口写在一句里没说明。2268 那一次复不出（记录自己写了），所以 2268 窗口上的 3.4% 没法核。改法：括号里补「（闭窗口 2026-10-01 到 10-21 上量的）」（推的）。

### K3-3　自证种数与两份真提示（复得出）

副本（`.claude/` 与 `scripts/{agents,kb,hosts,gate}` 拷进草稿 `copy/`）里逐个 `--selftest`，经 `run-with-memory-cap.sh 4G`，草稿 `logs/selftests.txt` 原样（行尾截 90 字）：

```
== bash .claude/hooks/runner-dispatch-guard.sh --selftest rc=0
  ✓ 自检通过（查了 460 种）：没写「重型测试：不跑」一行的拒（�
== bash .claude/hooks/handback-guard.sh --selftest rc=0
  ✓ handback-guard 自检通过（查了 23 种）：子 agent 交回时自己起的后�
== python3 scripts/agents/agent-cost.py --selftest rc=0
  ✓ 自检通过（查了 18 项，其中 outcome 的 20 个子 agent）：调用按消�
== python3 scripts/agents/dispatch-prompt.py --selftest rc=0
  ✓ 自检通过（查了 90 项）：有定义的类型按 required-inputs 逐组一行
== bash .claude/hooks/continuation-guard.sh --selftest rc=0
  ✓ 自检通过（查了 20 种）：上下文大小不论、收件人没有会话记�
== python3 scripts/agents/subagent_transcript.py --selftest rc=0
  ✓ subagent_transcript 自证通过（查了 12 项）：送达按调用之后对应的
== python3 .claude/gate.d/lib-lag-baseline.py --selftest rc=0
  ✓ lib-lag-baseline.py 自证：9 项都过
```

`quote-kb.py`、`stale-candidates.py` 自证也退 0。两份真提示（从会话记录取原样，`prompts/a3652270eb80e4fd0.md` 37 行、`prompts/a303d33b0ccd5e68f.md` 30 行）`dispatch-prompt.py --check`：前一份「✓ 派发闸照这份提示放行（three-way-attack，查了 37 行提示）」退 0；后一份放行句之后另有「✗ 认领行：…」退 3（`out/dispatch-check.txt`）。与记录第 195 行、正文节二写的一致。

### K3-4　第二轮判决「快照全对上，`--root .` 与快照根同义」（打中，轻）

原文 `research/prompts/landing-audit-r2-main-verification.md:83`「第二轮这次快照全对上，`--root .` 与快照根同义」。今天照第二轮判决第 33 行那条命令对活仓跑：

```
  ✗ 核了 85 处引文、49 处裸行号，24 处对不上，49 处没判（其中裸行号没有引文可比 48 处；逐处列在上面）
```

24 处按被引文件分：`agent-cost.py` 6、`doc-experiments.sh` 4、`doc-decisions.sh` 3、`handback-guard.sh` 2、`runner-dispatch-guard.sh` 2、`stale-candidates.py` 2、`agent-common.md`、`experiment-designer.md`、`lib-lag-baseline.py`、`quote-kb.py`、调度记录各 1，全是第七批之后改过的文件（快照里有的 8 份今天 sha256 都对不上；`doc-decisions.sh`、`handback-guard.sh`、`quote-kb.py` 三份根本不在第二轮快照的 63 份里）。也就是说第二轮那 85 处「对上」只在判决当天的活仓上成立。

要在冻结的样子上复，得有快照树，第二轮只存了 sha256 与 diff。`probes/rebuild_snapshot.py` 取 diff 头「index 改前..改后」的改前 blob（对象库里还在；diff 里删掉的文件也放回去）、再 `git apply` 那份 diff，7 份不在 diff 也不在 git 里的归档文件从工作树拷（其中结局账存档用 K3-1 复出的 6dde89a8 那一版）：`sha256sum -c research/prompts/landing-audit-r2-snapshot/judged-sha256.txt` 63 份全对上（`out/r2-rebuild.txt` 末行 63）。以它为根：

```
  ✓ 核了 85 处引文、49 处裸行号，对上 39 处，没判 95 处（其中裸行号没有引文可比 38 处；逐处列在上面）
```

（`out/r2-cite-check-rebuilt.txt`。）冻结的样子上 39 处对上、0 处对不上；其余引到快照外的文件，判不了。所以「同义」不成立：快照只收被判文件，腿引的文件（攻方 K4-1 的 `doc-decisions.sh`、K4-2 的 `quote-kb.py`、辩方正推 8 的 `handback-guard.sh`）不在里面，`--root .` 判的是活仓，过了那一天就判不回去。这一轮同样：第三轮快照 74 份收了这三份，但不收 `.claude/kb-directories`、`scripts/agents/cite-check.py`、`.claude/hooks/lib_heavy_tests.py` 这类我也引到的文件。四句：不分辨臂；满足的是正文 K3 第一条「两份判决引的数照写明的命令复不复得出」——复不出，原因是快照的范围；改法：甲 判决跑 cite-check 之前把腿引到的每一份文件（`cite-check.py` 已经能列出它判了哪些文件）补进快照的 sha256 表；乙 快照里另存 `rebuild_snapshot.py` 这种按 blob 重建的法子，判决写「冻结根上 39 / 85」而不是「85 全对上」（都是推的，被攻过零轮）。推翻条件：cite-check 的「对上」本来就只想说「判决那天对上」，判决文字不承诺往后复得出——那这一条只剩措辞。

### K3-5　别的数

| 出处 | 写的数 | 今天 | 怎么核的 |
|---|---|---|---|
| 调度记录第 162 行「单元表 19 个、登记 13 个」 | 19、13 | 复得出：单元表字面量 19 行，登记表 14 行、第 3 列不同单元名 13 个 | `awk` 数 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 的 `formal_run_units` 数组里带引号的行；`cut -f3 \| sort -u \| wc -l` |
| 调度记录第 199 行「快照 74 份」「清单抄 3 行」 | 74、3 | 复得出：74 行全对上；`_landing-audit-r3-checklist.md` 表里 3 行 | `sha256sum -c`；读那份清单 |
| 第二轮判决第 32 行「63 份全对上」 | 63 | 今天对活仓 11 份对不上（第七批改的与重出的结局账存档）；重建的快照树 63 份全对上 | K3-4 |
| 第二轮判决第 34 行交回形状 files 3287、送达 3075 | 3287、3075 | 今天 3292 份、3082 条 success=True，形状仍三种 | `probes/handback_shapes.py`（第二轮攻方的原样） |
| 调度记录第 138 行 2268 个那一组 | — | 复不出，记录自己写了 | — |
| 第二轮判决第 34 行四份攻方输出逐字节相同 | — | 门槛表复得出（K3-2）；别的三份读的是被第七批改过的文件，没复 | — |

## 没打中的形状

- K2 派发闸：三种锚法 × 三样判定在 3292 份真提示上全比过；否定词判法 3292 份 0 份不同；骨架没逐种重试（工具员的 `skel/` 目录在它的草稿里已不在，`--check` 两份真提示复得出）。
- K2 结局账：`--since` 写法 72 行全扫；`\bfix` 尾词界在 3292 条描述上逐条比；三方腿 463 条逐条取自己的报告比；轮名按前缀取判决会不会取错：463 条里自己的报告与自己那一轮判决都对上的 446 条，没有一条取到别的轮。
- K2 库：七个调用点逐个看 cwd；`./` 写法只在子目录起时有差，阶段都先进根。
- K3 结局账：两版、两种切法、逐个子 agent 对齐，三方桶、门槛、点到三样在共有的 2099 个上 0 个挪动。

## 这条腿自己的限度

- 门禁阶段（doc-registries 六格、replay-comparison-units 样本档）被重型测试闸当「提交时才跑的检查」拒，K2-9 只读了代码；样本档与开关复跑没做。
- 真派发提示的「自己的报告」是按派发闸的报告行解析取的，K2-1 那三处「是转述」是我读上下文判的（同棵=False 加句子形态），没问派发它们的会话。
- K2-2 只核了那几个子 agent 有没有用 Write 写名字带 report 的文件，没核它们是不是经 Bash 写了 report.md。
- 会话记录在长：`out/` 里按真提示数的几份往后会涨，结局账闭窗口那几份不变（窗口闭着；但「点到」没有上界，往后某个会话点到窗口里的报告，report-used-by-main 会涨，第二轮判决 K3-3 那一条）。
- 我提的改法（K2-1/K2-2 按草稿目录判、K2-3 整份提示认三方形态、K2-8 表头、K2-9 起不来判红、K2-13 列举补全、K3-2 括注、K3-4 快照补引到的文件）都只在我的模型上看过、被攻过零轮，没在副本上实现、没跑。

## 没做什么

- 没判 K1（辩方的格）；没替主 agent 采纳；副本上的数（自证种数、`--check` 读数）不算入库装置上的数。
- 没跑重型测试、没跑门禁阶段与各类 lint、没编译、没提交。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-r3-opus/` 里交回前删掉了建的副本与树（`copy/`、`brokenkb/`、`r2tree/`、`rerun1/`），留 `logs/`（自证读数、`--since` 调用清单、模型 sha256 表、复跑日志）。
