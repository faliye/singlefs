# landing-audit-r2 攻方腿（Opus）：K2 修补本身、K3 产生结论的方法、K4 两轮都没看过的地方（2026-10-23）

正文 `research/prompts/_landing-audit-r2-body.md`，背景材料 `research/prompts/_landing-audit-r2-background.md`，前一轮判决 `research/prompts/landing-audit-r1-main-verification.md`。开工时 `sha256sum -c research/prompts/landing-audit-r2-snapshot/judged-sha256.txt` 63 份全对上。

## 复跑

模型目录 `research/prompts/landing-audit-r2-opus-model/`。这一轮没有冻结的仓副本（开工快照只有 sha256 与 diff），被判的脚本从 `--repo` 读，`--frozen` 给同一个仓根：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/landing-audit-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新的空目录>
```

8G 那一次在内存准入排队排满 235 秒没起来（slice 已占 32.5 GiB、总上限 40.1 GiB），改 4G 起，退 0、108 行输出，原样在 `out/rerun.log`。几处限度：派发闸「改前」那一版取自 `git show :.claude/hooks/runner-dispatch-guard.sh`（暂存区），暂存这一批之后它就是改后的；结局账两步读仓外的会话记录，会话记录还在长；`fill_skeletons.py` 点名的几个草稿目录不建（`--check` 不建锁）。

模型目录每个文件的 sha256（`find . -type f | sort | xargs sha256sum`，在模型目录里跑）：

```
30db0de705a19ad52f30bdc4d9baca5a0394b46fb483f8658502b6f8a011e44c  out/closed-agents.jsonl
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  out/closed-diff.txt
f409bd11533b9350e18418fa238304ff7f03ed6a26c446b588a0fe5d3b8cdb04  out/closed-summary.txt
9811c6cecd1da857f71094f2c4bae17eae49bf6a7ce81eced13bd1bc3815019e  out/dispatch.txt
855d042f0e32d5ef5d8196f32223f295246637f539428caea3a27fbac7f8154f  out/handback-shapes.txt
45d236f965b7ed724cd3231091429905218a1d27901b797f3465f5bee7611cd1  out/kb-directories.txt
d59e22ca0f1341ce7922498fcc45e3feb5418396fa50869f73ef77d3cc7fc0af  out/ledger-base.txt
7e6f68ddb6168d460d241b52a0e6e6a4914d7190933ff39ac2d91e065439c8b9  out/lstrip.txt
c020d26e77a98fb87cf0bfa08d5ae89a7cbcc3e808c964c28b8e7e7f844962b3  out/rekey.txt
98c72268894b359b049147882a09904a1d17d23b42582d92dbaf82cfb9db8a82  out/rerun.log
a6e9f1a588fee6721807b8f012fd67ae753767e8a633476d6505cedc818adc6b  out/rework-fixture.txt
63615437ec983c495f40d831b11e29512758139c94016e5962895dc42e7c6ba8  out/rework-words.txt
b7153bdc0a600bfc0f84cc2031f15718f2adb3f8468b060c43cc39d27f3d552b  out/skeletons.txt
1e0302ffcbca4fb712592bed6b008e1581ff3379ef855bc21919f2031da59621  out/threshold-sweep.txt
07d4db8bbc9e1e0c3fd184dc9bef227816dceb658acd45b613f10aa74f1e8a6a  probes/dispatch_probe.sh
241f364b02fac4434e09071737c5d6341d6c3cf0cf4be993c291630baabedab8  probes/fill_skeletons.py
44046fbcdf1b599342a0b9ed6ae3be4d5421248680f089b3606f8df3e3bd56ae  probes/handback_shapes.py
dcdce79958b52dc2f0b3e0dc1b30dd0294516a6404c2acec7110a728cbe36833  probes/kb_directories_probe.py
5034551fb6ee79db8cb09acae69cc845c612e42e5c627607da673425cdf6f389  probes/ledger_base_probe.py
5b9ef68c17dec9db5850c8c2b7634dffd9d0bb97c51754d25411314d8b744ac5  probes/lstrip_probe.py
71b002909e75b5c16145a4d2cb10f1870a173cddc51b1ebffc1e9cd31c0697e1  probes/outcome_dump.py
162421ec2d84d9d8ccef0b33701e64c28dc6ddce04e34a2e36895fd080afea3a  probes/rekey_probe.sh
4e53c4a9892c7ce8a9fe9061989c87b847a087f03bcf3800edcc749cd9ea83f6  probes/rework_fixture.py
9bf0d0d83ad8f21a05021d149eac1524f99aef6a2a031f0d57de6d59857f02cf  probes/rework_words.py
77fed1a726d635f29c7d6cb8ba864f92d4310e2b1b6053fc5576806c59b1459c  probes/threshold_sweep.py
7155650c7fea1113699845bd7df795a9aaf99dd674d74fc78b985016bcc38156  prompts/landing-audit-r1-attack.txt
3a69a106751f33f3798efbc020bc2fc4833fd10dd452dfe6eeebf5e84662c563  prompts/probe-attack-prose.txt
76698a7c79d0cd956bdd475869fa3fc30e8a5a1647807b63ff0e501aaddf6556  prompts/probe-clerk-prose.txt
9e7b52564831b3b4632d01d39c4cf65e9a46a77d8439508a2cc111258460b80e  prompts/probe-tooling-writer-prose.txt
e021deda1917b2dbd91c814040ff7dfb93ff82eb1af7124d2edbb4410d2c2bd4  prompts/tooling-writer-hooks-fix.txt
d322015ff96565b0bf9c657ba89bdfb5e962bfcad72617d307f16a66cd5ff510  tools/rerun.sh
```

`out/` 下各份是 `tools/rerun.sh` 那一次的原样输出；`out/closed-agents.jsonl` 是结局账闭窗口每个子 agent 一行（类型、描述、桶、写入行数、落地行数、点到与否、折算花费），门槛换值与词表的复算都从它算。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K2-1 七处并进 lib-lag-baseline.py 之后的语义 | 打中（轻） | 四处原来只把行首就是 `#` 的行当注释，并后缩进的 `#` 行与「首列空、第二列 `#` 开头」的行也被静默跳过；今天七张表里一行这种都没有，「逐行相同」对照看不见它；方向是少报一种写坏，不放过违例 |
| K2-2 replay-comparison-units「只缩不涨」能不能绕 | 打中 | 键只有（登记出处，留存产物名），不含实验号与单元名：删掉一行 none 行、换成另一个实验的新比对对象借用同一对键，照绿；改前滞后表的键同为这一对，这一批没堵上 |
| K2-3 派发闸 ④⑰ 共用报告行之后误拦 | 打中（新带进来的） | 报告行不锚行首，正文里用「」引着的「报告：」「报告路径：」也被当成报告行；本会话 19 份首条派发提示里两份合法派发（第五批钩子工具员、第一轮攻方腿）改前退 0、改后退 2；三份最小探针同样改前 0 改后 2 |
| K2-4 交回与起点两个定义变宽 | 交回没打中；起点打中（轻） | 真会话记录 3 266 个交回结果只有三种形状，「不是错误」与「带 success:true」判法逐个相同；起点按 UTC 切天，`--until 2026-10-21` 收进了东京日期 10-22 开工的 45 个（花费 1.66%） |
| K2-5 定义 17 处与钩子、门禁的新矛盾 | 没打中 | 见「没打中的形状」 |
| K3-1 结局账门槛换值 | 打中（部分） | 「看起来没效果」2.0% 与 70% 门槛无关，只随 30% 那道与写入下限走：30%→50% 是 3.4%，严档 90%/70% 是 6.2%；「真浪费约 1% 到 2%」只在下沿门槛不高于 30% 时成立 |
| K3-2 「三方腿」桶 | 打中 | 三方腿交回了就进 verdict-archived，账里不核报告在不在仓、轮有没有判决；28.6% 的花费是按类型名记的「有效」 |
| K3-3 闭窗口「往后复跑得出同一组数」 | 今天复得出；往后推的复不出 | 此刻复跑与存档逐行相同；但「点到」不设上界、落地比对读此刻的工作树，往后的会话会把 report-unreferenced、written-unused 移走 |
| K3-4 返工词表 | 打中 | `\bfix` 命中 fixture / fixtures / fixed-script（6 个）；单字「修」命中 65 个里多是新活（修三份脚本的早有毛病、修复）；「补跑」「第二段」不在词表里。误报漏报两个方向都有，14% 不是上界也不是下界 |
| K3-5 九条缺口的样本 | 打中 | K4、K3 两条的样本是按「同号派 10 次以上」挑的最长几条链，「盲判独立复现」是同一批派发换人归类，证的是归类一致，不是普遍；载体搬家 5/9 是同一天同一次搬家 |
| K3-6 cite-check 与「逐行相同」对照 | 打中 | 第一轮核查用 `--root .` 判活仓、不是快照根；cite-check 只核引文在不在那一行，第一轮 163 处里 84 处裸行号没判。「逐行相同」只走到「基准里有这张表」那一种结局与今天表里有的那几种行形 |
| K4-1 doc-decisions 挂账表取基准 | 打中 | 第九处同一写法，而且正是 lib-lag-baseline.py 要防的那一种：`base_ledger_text` 把 git 取不到当成「基准里没有这张表」，只缩不涨静默不比；不在第一轮「全仓 7 处」里，也不在工具员找到的第八处里 |
| K4-2 `.claude/kb-directories` 的写法 | 打中 | 写成 `./<目录>` 或绝对路径，quote-kb.py 认、doc-registries.sh 认，stale-candidates.py 读登记位之后静默不认那个目录下的 .md |
| K4-3 归档目录搬进 research/prompts/ | 没打中（只记一条去向） | evidence-in-repo 判据二对这批新文件 0 处命中；它会在下一次提交被 archive-past-rounds 删掉、引用改成 git show，照规矩不算丢 |

## K2　第一轮的修补本身

### K2-1　lib-lag-baseline.py 并了七处之后，四处的注释行认法变了（打中，轻）

哪一处：库切表行是 `.claude/gate.d/lib-lag-baseline.py:75`：「if line.strip() and not line.lstrip().startswith("#"):」。七处改前各自的切行（开工快照里删掉的行）：
- multipath-registry：`research/prompts/landing-audit-r2-snapshot/judged-vs-index.diff:556`：「if line.strip() and not line.startswith('#'):」
- replay-comparison-units：`research/prompts/landing-audit-r2-snapshot/judged-vs-index.diff:654`：「if line.strip() and not line.startswith("#"):」
- mutation-tables：`research/prompts/landing-audit-r2-snapshot/judged-vs-index.diff:171`：「if not line.strip() or line.startswith("#"):」
- reading-discipline（read_lag 与 baseline_counts）：`research/prompts/landing-audit-r2-snapshot/judged-vs-index.diff:242`、`:281` 同一句
- doc-process-records：`research/prompts/landing-audit-r2-snapshot/judged-vs-index.diff:838`：「if raw.strip() and not raw.lstrip().startswith("#"):」（本来就 lstrip，没变）

复出来：`probes/lstrip_probe.py` 把这几段原样抄成函数，喂同一张四行表（第 2 行「  # 缩进的注释」、第 3 行「\t#开头的第二列\t理由二」），`out/lstrip.txt` 原样：

```
name=content_rows rows=[1, 4]
name=multipath-registry before_rows=[1, 2, 3, 4] after_rows=[1, 4] same=False
name=replay-comparison-units before_rows=[1, 2, 3, 4] after_rows=[1, 4] same=False
name=mutation-tables before_rows=[1, 2, 3, 4] after_rows=[1, 4] same=False
name=reading-discipline before_rows=[1, 2, 3, 4] after_rows=[1, 4] same=False
name=doc-process-records before_rows=[1, 4] after_rows=[1, 4] same=True
```

改前这两行进各格的形状判法（「不是两列」「不是四列、或有空格子」「不是六列」），判红；并后静默跳过。基准取不到、基准里没有表、表为空三种结局逐处比过，七处都没变（表为空时库交 `""`，各调用方都用 `is not None` 判，没有把空串当「没有表」的）；行尾 `\r` 原来四处里两处去、两处靠 python 文本模式的换行统一去掉，结果相同。

四句：不分辨臂（这一条不涉及臂；七处一样受影响，病根在库一处）；被判的系统看得到（表就在它手里）；满足的是正文 K2 第一条「注释行」那个分句；改法碰得到：库的自证「表的内容行」那一格把缩进注释当注释钉死了（`.claude/gate.d/lib-lag-baseline.py:180` 写进样本的「  # 缩进的注释」），所以并后的认法是有意的，没写进的是「四处语义变了」这句话。

方向：跳过一行只会让豁免少一行、基准少一行（判得更严），丢的只是形状报错，不放过违例。今天七张表里这种行一行都没有（`grep -cE '^[[:space:]]+#|^'$'\t'` 逐张 0），所以第五批「四格红与改前逐行相同」看不见它。

改法：二选一，都推的、没实现。甲：库的文件头与 r1fix-gates.md 第 63 行那张「七处」表各补一句「multipath-registry、replay-comparison-units、mutation-tables、reading-discipline 四处改前只认行首 `#`」；乙：`content_rows` 加一个参数，缩进的 `#` 行交给调用方当形状错。推翻条件：找到一张被这四格读的表里有缩进 `#` 行或首列为空的行，而改后判绿、改前判红的那一行确实是登记错（那就不是「轻」）。

### K2-2　「复跑写 none: 的行只缩不涨」借键就绕得过（打中；改前同样绕得过）

哪一处：只缩不涨比的键是 `.claude/gate.d/doc-experiments.sh:2426`：「return (fields[1].strip(), fields[4].strip())」，即（登记出处，留存产物名）；登记表的去重键 `.claude/gate.d/doc-experiments.sh:2451`：「key = (registration, product)」。实验号、单元名都不在键里；留存产物名这一格不核文件在不在（格的注释「管不到」一段自己写了），登记出处只核文件在。改前滞后表的键同为这一对：`research/prompts/landing-audit-r2-snapshot/judged-vs-index.diff:730`：「key = (fields[0].strip(), fields[1].strip())」。

复出来：`probes/rekey_probe.sh` 从仓里那一份 doc-experiments.sh 现取这一格的 python，在造出来的小仓里起；基准提交里有一行 E165 的 none 行（e165-r5-prereg.md，segment3-r5-legal.out）。`out/rekey.txt` 末三段（输出行太长，截在 200 字）：

```
name=删掉E165的none行换成E200的新比对对象借用同一对键
  ✓ 比对对象 2 行（E165、E200）：登记出处都在，单元名在单元表出处里都恰好出现一次；复跑写驱动名的 1 行在 replay.sh 里都有且产物名相符；复跑写 none 的 1 行只缩不涨，…
exit=0
name=对照登记出处写成新实验自己的登记
  ✗ none 行比基准多：replay-comparison-units.tsv 里这几行复跑写 none:，基准 HEAD 那一版里没有这样的行：
     第 3 行 segment3-r5-legal.out（research/prompts/e200-r1-prereg.md）
exit=1
```

同一个形状的更短写法：在已有的 none 行里只换单元名（与实验号），键不变，照绿。正文点名的三种写法：改登记出处、改产物名都会换键、判红（对照那一段就是）；「none 行改成驱动名再删驱动行」删驱动行时 ③ 判红；三种都绕不过，绕得过的是「键不变、行的身份换了」。

四句：不分辨臂；格当时看得到（实验号、单元名都在同一行里，只是不进键）；满足的是正文 K2 第二条「换个写法能不能绕过」；改法：甲 键改成（实验号，登记出处，单元名，留存产物名）——借键那一形换了单元名就成新键、判红（推的）；乙 另核登记出处的文件名里带这一行实验号的小写 `e<号>-`——只挡跨实验借键，挡不住同实验换单元名（推的）。甲罩得住两形，乙只罩一形。推翻条件：有一条规矩或格要求「同一（登记出处，留存产物名）只能对应一个单元」并在别处判（我没找到）。

### K2-3　报告行不锚行首，正文里引着的「报告：」被当成报告行（打中，这一批带进来的）

哪一处：`.claude/hooks/runner-dispatch-guard.sh:1242`：「REPORT_LINE = re.compile(r"(?<![^\W_])(?:报告(?:路径|写进|写到)[* \t]*[：:]?|报告[* \t]*[：:]|报告(?=」。前面只要求不连着字，「「」是标点、不算字，所以「骨架「报告：」一行默认填 `<草稿目录>/result.md`」这种句子里「报告：」之后到行尾整段当报告行正文，正文里第一个带 / 的词当点名的路径，交 ④（写范围）、⑰（文件名带 report）与三方腿形态三判。改前 ⑰ 只认行首「报告：」（`LINE_START` 起头），④ 认的三种标签后面要紧跟 `research/` 这类路径。

复出来：`probes/dispatch_probe.sh` 拿暂存区那一版钩子（改前）与工作区这一版（改后）各跑 `--check-prompt`。两份是本会话真派出去、真干完的提示（第五批钩子工具员 agent-a303d33b0ccd5e68f，第一轮攻方腿 agent-a3652270eb80e4fd0，原样拷在 `prompts/`），三份是最小探针（骨架填好后各加一句正文）。`out/dispatch.txt` 原样（行尾截在 160 字）：

```
name=tooling-writer-hooks-fix type=tooling-writer before_exit=0 after_exit=2
    改后拒：✗ 派 tooling-writer 的提示报告行点名的文件名里带 report：「…/report.md」。
    改后拒：✗ 提示要 tooling-writer 写进它写范围之外的路径：research/prompts（.claude/hooks/agent-write-scope.tsv）。
name=landing-audit-r1-attack type=three-way-attack before_exit=0 after_exit=2
    改后拒：✗ 派 three-way-attack 的提示报告行点名的文件名里带 report：「report.md」。
    改后拒：✗ 派 three-way-attack 的提示报告行点名的文件不是这一形态：「<草稿目录>/result.md」；三方腿的报告要形如 research/prompts/<…
name=probe-clerk-prose type=clerk before_exit=0 after_exit=2
    改后拒：✗ 派 clerk 的提示报告行点名的文件名里带 report：「…/report.md」。
name=probe-attack-prose type=three-way-attack before_exit=0 after_exit=2
name=probe-tooling-writer-prose type=tooling-writer before_exit=0 after_exit=2
    改后拒：✗ 提示要 tooling-writer 写进它写范围之外的路径：research/prompts（.claude/hooks/agent-write-scope.tsv）。
```

（汇总行「有 2 处不成立」与 probe-attack-prose 的那一行拒绝在 `out/dispatch.txt` 里，这里省了。）三份探针加的句子：clerk「出口：改完之后 `--check` 上「报告路径：…/report.md」退非 0。」；攻方「另判一处：骨架「报告：」一行默认填 `<草稿目录>/result.md`，这是不是矛盾。」；工具员「2. 骨架给「报告：」一行的默认：三方腿填形如 `research/prompts/（轮）-（模型）-output.md` 的占位。」——正是「派人改报告行判法」这类活的提示必然写出的句子。本会话 19 份首条派发提示全量跑过：改后退 2 的 8 份里 6 份是 clerk 写 report.md（真该拒，派在 ⑰ 之前），另 2 份就是上面两份误拦。14 种类型的骨架照填好、只改占位，`--check` 13 种退 0，变异分诊员退 3 是因为我填的「探针内容」不含变异表（`out/skeletons.txt`），骨架本身不误拦。

四句：不分辨臂；钩子看得到（「报告：」前面就是「「」，后面紧跟「」」）；满足的是正文 K2 第三条「会不会误拦已有的合法派发」；改法：甲 「报告：」那一种标签退回 `LINE_START` 锚行首，句中只认「报告路径 / 报告写进 / 报告写到」且后面紧跟路径（推的）；乙 标签前一个字是「「『“`」或标签后正文以「」」起头的不认（推的）。甲罩得住全部五份（五份里出问题的都是引号里的「报告：」「报告路径：」），但会让自证「报告名:句中「，报告 <路径>」也认」那一格转绿为红，要同时删那一格；乙不碰那一格，但罩不住不带引号的转述。只在我的模型上量过、被攻过零轮。推翻条件：主 agent 认定「正文里引着的报告标签被拒」是要的行为（那这两份就不是误拦，是派发提示该改写法）。

### K2-4　交回与起点两个定义变宽（交回没打中；起点打中，轻）

交回：送达的认法在 `scripts/agents/subagent_transcript.py:115`：「and block.get("tool_use_id") in self.calls and not handback_result_is_error(block)):」。看门狗原来认「结果里带 "success":true」，现在认「不是错误」，变宽的那一块是「结果既不是错误、也不带 success:true」。`probes/handback_shapes.py` 扫了这个项目全部 3 286 份子 agent 会话记录里每个 SubagentHandback 的结果，`out/handback-shapes.txt` 原样：

```
files 3286
3071 json success=True message='Report delivered to your caller.' | agent-aa939103fa58bc8ef.jsonl
193 is_error | agent-a55c08b4c49a3067d.jsonl
2 json success=False message='Nothing was sent: your report was already delivere' | agent-a48f2f806614c1a41.jsonl
```

三种形状里变宽的那一块一个都没有，新旧判法在真数据上逐个相同；续做闸、结局账、看门狗三处判反的情形造不出来（造得出的只有手写的「纯文字结果」样本，自证里那一格就是）。推翻条件：哪天工具结果出现第四种形状（例如纯文字的拒绝），而它被认成送达。

起点：窗口按首条记录的时刻、按 UTC 切天，`scripts/agents/agent-cost.py:1430` 写明「上界是第二天零点（UTC，与 --since 同一种切法）」。调度记录与存档文件名写的日期是东京日期（共用约束「规则怎么读」一节，日期用 `TZ=Asia/Tokyo date +%F` 取）。`out/closed-agents.jsonl` 里起点不早于 2026-10-21T15:00Z 的（东京已是 10-22）有 45 个、折算花费 1.66%（复算命令：`python3 -c` 按 start 字段筛，草稿里跑的，没进模型）；窗口下沿同样少了东京 10-01 零点到九点那一段。四句：不分辨臂；脚本看得到（时间戳带时区）；满足的是正文 K2 第四条「三个子命令改按首条记录时刻筛」；改法：`--since`/`--until` 按东京日期切（或文件名写明 UTC），推的。影响只是存档文件名「到 2026-10-21」与内容差半天，不翻任何结论。

### K2-5　第五批定义改动与钩子、门禁（没打中）

查了：执行员第 5 步登记比对对象行 ↔ 写范围表第 20 行 ↔ 归属表第 106 行（三处都是执行员）；攻方、辩方 `required-inputs` 补「报告」↔ 派发闸三方形态判法 ↔ 骨架（三种骨架填好 `--check` 退 0，`out/skeletons.txt`）；共用约束「命名检查交回前跑」写「git worktree 建的副本」↔「不做」一节禁子 agent 做 `worktree`（worktree 由主 agent 建，rsync 副本那一支另有出路，不矛盾）；执行员与工具员「命名检查除外」↔ 共用约束「门禁」一节；主 agent 定义第 86 行 ↔ 交回闸。没找到新矛盾。

## K3　产生结论的方法

### K3-1　结局账的门槛换一个值，「真浪费约 1% 到 2%」翻不翻（打中，部分）

结局账的门槛 `scripts/agents/agent-cost.py:353`：「LANDED_AT, PARTIAL_AT, WRITTEN_MINIMUM = 0.7, 0.3, 10」；「看起来没效果」那一行并的桶 `scripts/agents/agent-cost.py:399`：「("看起来没效果的几桶", ("report-unreferenced", "written-overwritten", "written-unused", "report-used-by-later-sub", "stopped-abandoned", "cut-by-usage-limit")),」。三个数在文件头与调度记录里都没有出处（没量过就定的）。

先核模型：闭窗口照原门槛重算，与账里每个子 agent 的桶不同的 0 个；`out/closed-diff.txt` 为空（此刻复跑与存档 `outcome-summary-2026-10-01-to-2026-10-21.txt` 除末行外逐行相同）。门槛换值，`out/threshold-sweep.txt` 原样：

```
LANDED_AT PARTIAL_AT WRITTEN_MINIMUM | 落地个数 落地花费 | 部分个数 部分花费 | 没效果几桶个数 没效果花费
0.7       0.3        10              |    847   50.0% |     87    4.2% |       87    2.0%
0.9       0.3        10              |    618   35.0% |    316   19.2% |       87    2.0%
0.5       0.3        10              |    903   52.8% |     31    1.4% |       87    2.0%
0.7       0.5        10              |    847   50.0% |     56    2.9% |      118    3.4%
0.7       0.1        10              |    847   50.0% |    100    4.8% |       74    1.5%
0.9       0.7        10              |    618   35.0% |    229   14.9% |      174    6.2%
0.95      0.9        10              |    418   21.8% |    200   13.2% |      403   21.2%
0.7       0.3        1               |    971   53.3% |    106    4.5% |      140    3.1%
0.7       0.3        30              |    742   47.2% |     71    3.7% |       76    1.7%
0.9       0.7        1               |    734   38.1% |    237   15.1% |      246    7.6%
写了≥10 行的交回个数 957 落地率 <0.3: 23  [0.3,0.7): 87  [0.7,0.9): 229  ≥0.9: 618
```

读法：「没效果」只随下沿（30%）与写入下限（10 行）走，与 70% 无关；下沿 10% 到 30% 之间是 1.5% 到 2.0%，抬到 50% 是 3.4%，写入下限放到 1 行是 3.1%。「写了、不低于 70%」这一行随上沿走得很厉害（70%→90% 从 50.0% 掉到 35.0%），调度记录「按结局分」第一行那种读法（写的大半都进了仓）依赖 70% 这个没出处的数。「真浪费约 1% 到 2%」在下沿 ≤ 30% 时成立，下沿 50% 时不成立。结论「丢失的不多」本身不翻（最严的一档 21.2% 是把 90% 以下全算没效果，没人会这么读）。

四句：不分辨臂（这里没有臂，是一个读数的取值依赖）；脚本看得到逐个子 agent 的落地率；满足的是正文 K3 第一条「门槛换一个取值结论翻不翻」；改法：账里加一行「下沿 30%→50% 时没效果 3.4%」或在文件头写门槛的来历（推的）。推翻条件：有人拿出 30% 这道门槛的出处（例如抽查里「低于 30% 的都确实白做、30% 以上的都确实有效」的逐个核对）。

### K3-2　「三方腿」桶按类型名记有效（打中）

`scripts/agents/agent-cost.py:938`：「if agent["type"].startswith("three-way"):」——三方类型交回了就进 verdict-archived，不看报告在不在仓、所在轮有没有判决、判决采没采。闭窗口里这一桶 461 个、28.6%：three-way-attack 237 个 14.58%、three-way-forward 163 个 10.86%、three-way-defense 61 个 3.17%（草稿里按 `out/closed-agents.jsonl` 的 type 与 bucket 两列数的）。调度记录第 38 行「498 条腿里 494 条的报告在仓里、所在轮有判决」是第一次一次性机械层核的，结局账没接着核，也不在 `outcome` 的输出里。第一轮判决不采的那几条腿（K2-7、K2-8 这类）的花费照样记「有效」。四句：不分辨臂；脚本看得到报告路径（交回里写着 research/prompts/…-output.md）；满足的是正文 K3 第一条「三方腿的认法从什么数据推出来」——从类型名推出来；改法：三方腿按「交回点名的 output.md 在 git 历史或工作树里、同轮有 main-verification.md」分两桶（推的）。推翻条件：结局账在别处核了这两样（我在 agent-cost.py 里没找到）。

### K3-3　闭窗口「往后复跑得出同一组数」（今天复得出；往后推的复不出）

调度记录第 178 行写「往后复跑得出同一组数」。今天确实逐行相同（K3-1 第一段）。往后会变的三处（都是读代码推的、没等到它变）：
- 「点到」不设上界：`scripts/agents/agent-cost.py:1027`：「used_main = used_main or referenced_between(main_stamps, end, limit)」，limit 对按路径与 agent 号点到的是无穷，主会话 `scripts/agents/agent-cost.py:972` 按修改时刻收进窗口之后才写的会话；10-21 之后哪个会话点到一份报告，那个子 agent 就从 report-unreferenced 挪进 report-used-by-main。
- 落地比对读此刻的工作树与全部分支（`scripts/agents/agent-cost.py:1002` 的 history_and_worktree_lines），工作树里没提交的行被删、别的工作树没了，落地率会降。
- 会话记录本身会长（窗口内开工、窗口后还在跑的子 agent 的花费）。
推翻条件：过几天再跑一次，与存档逐行相同。这一句改成「当天复跑得出；往后点到与工作树会变」即可（推的）。

### K3-4　返工词表：误报与漏报两个方向都有，「约 14%」不是上界（打中）

词表 `scripts/agents/agent-cost.py:355`：「REWORK_WORDS = re.compile(r"修(?!订)|重跑|重派|再派|重做|返工|续做|续派|接续|接手|接着做|（续）|补写|补做|改名|改号|改钉值|第\s*[2-9]\d*\s*次|redispatch|re-?run|renumber|retry|\bfix", re.IGNORECASE)」。闭窗口命中 327 个、花费 13.7%。`out/rework-words.txt` 与 `out/rework-fixture.txt` 摘出的几行（原样）：

```
命中词（每个派发取第一个命中）: [('fix', 91), ('修', 68), ('重跑', 30), ('改名', 27), ('rerun', 26), ('接手', 14), ('重派', 12), ('第 2 次', 8), ('接续', 8), ('续做', 6), ('renumber', 4), ('重做', 4), …
只因单字「修」命中: 65 2.5% 后接的字: [(' ', 16), ('补', 13), ('红', 7), ('（行尾）', 5), ('复', 2), ('正', 2), …
没命中而带接续类字样: 48 2.0% [('收尾', 21), ('第二段', 12), ('二次', 11), ('续', 3), ('接着', 1)]
name=fixture_only_total count=6 cost_share=0.04%
```

误报的形态：
- 机械的：`\bfix` 前面有词界、后面没有，fixture、fixtures、fixture-claims、fixed-script 都算（6 个只因它命中，例「C531 fixture registrations excluded」「C487 segments-step red fixture」）。
- 语义的：「修」「fix」说的是「这件活是修东西」，不是「这一次是返工」。只因单字「修」命中的 65 个（2.5%）里有「修三份研究脚本自证的早有毛病」「修 doc-experiments worktree 样本」「4-2 测试簇与锚点修复」，修的是早有的毛病或门禁报的红，不是前一个子 agent 的活没做对；「fix」91 个里多数是「Batch 9 merge repin and fixes」「21-2 F3 and b20 verdict fixes」这类合入与判决之后的计划内修补。
漏报的形态：「第二段补跑」「补跑」不在词表（「E165 r2 第二段补跑与第四段」「E165 r2 第二段补跑 2c」没命中）；「收尾」「二次」也不在。48 个（2.0%）里有一部分是计划内分段、不该算，一部分是补跑、该算，按描述分不开。

所以「返工约 14%」是一个两头都有错的关键词计数，不是上界：要当上界，得先证没有漏报（补跑这一形已经漏了）。rework.md 五条链按派发逐次归类的数（本可避免 25 次、这几条链花费的 14.6%）与这个 13.7% 碰巧接近，但量的不是一件事（一个是「描述里带某个词」，一个是「这一次本可避免」）。四句：不分辨臂；脚本只看得到派发描述，看不到「这一次为什么派」——判别子观测不到，按 evidence-discipline「判据自己也会写错」那一节，这是「判别子观测不到」那一形，改判据不改词表；改法：甲 `\bfix\b` 加尾词界（只修机械误报，推的）；乙 调度记录那一句改成「描述带返工字样的派发 327 个、13.7%（关键词计数，误报漏报都有）」，不写「约 14%」当损失（推的）。推翻条件：逐个核 327 个，误报与漏报相抵后仍在 14% 上下，且有一个独立判据（例如前一次派发同一目录、同一实验号）给出同一个数。

### K3-5　九条缺口各从几个样本推出（打中）

逐条（依据列原文见调度记录第 98 到 108 行）：

| 缺口 | 样本与怎么挑的 | 推法站不站得住 |
|---|---|---|
| K4 执行员不逐条自查 | `records/2026-10-22-子agent结果落地审计.md:100`：「盲判独立复现 6/8」。8 次全在 E171、E188 两条链；五条链是按「同号派 10 次以上」挑的（`research/prompts/subagent-landing-audit-2026-10-22/rework.md:14`：「机械层「同号派 10 次以上」里有两个号各混了两个实验」），即派得最多、返工最多的几条 | 按因变量挑样本；盲判是同一批派发换一个人归类，证的是归类一致（6/8），不证普遍。rework.md 列的「登记已写明、装置没落地」只有 E171#3、#8、#25、E188#4 四次、两个实验 |
| K3 设计员不核可跑性 | `records/2026-10-22-子agent结果落地审计.md:101`：「盲判独立复现 4/4」，同两条链 | 同上；4 次 |
| K1 命名检查、K9 占号 | 主 agent 现查定义与脚本；rework.md K1 3 次、K9 2 次（E185 到 E187 一次撞号） | 机制站得住（定义里确实没有、脚本确实只查一个仓根），频率是一两次事件 |
| 延后项没进欠账表 | 8/54，固定种子抽 10 月 records | 抽样方法站得住；输入已删、只核得了算术（第一轮 K4-4） |
| 回扫 `crates/` 行没人接 | 8/15 | 15 行出自知识腐烂回扫，是几次回扫的？没写；「集中在知识腐烂回扫」是 15 对 102 的比较，两边的回扫次数没给 |
| 载体搬家后写回落空 | `records/2026-10-22-子agent结果落地审计.md:106`：「5/9 静默跳过；主 agent 现查两处」 | 9 行全出自同一天的同一次搬家（invariants.md 拆进 crates/singlefs-checker/invariants/），是一次事件；机制（扫描与写范围都没跟着搬）现查属实 |
| 报告写法矛盾 | 209 次全被拒 | 站得住 |
| 结局没有常设的账 | 主 agent 现查 | 站得住（是事实不是频率） |

所以九条里「真」站得住的有两类：机制现查属实的（K1、K9、载体搬家、报告写法、结局账），与有抽样的（延后项）；K4、K3 两条的「真」只有两条长链上的几次，判定表写的「真」没有限定词。四句：不分辨臂；调度记录看得到样本来源（rework.md 第 14 行写了怎么挑的）；满足的是正文 K3 第三条「有没有把一两次事件推成流程缺口」；改法：判定表 K4、K3 两行依据列补「两条最长链里 8 次 / 4 次，没在短链上抽过」（推的）。推翻条件：在派发次数少于 10 次的实验链里随机抽，也找到同比例的「登记写明、装置没落地」。

### K3-6　cite-check 判了什么、「逐行相同」证了什么（打中）

cite-check：第一轮核查写 `research/prompts/landing-audit-r1-main-verification.md:13`：「cite-check.py <两份腿报告> --root . --background」，规则要的是 `.claude/rules/three-way-inference.md:125`：「cite-check.py <全部腿报告> --root <快照根> --background <背景材料>」。`--root .` 判的是活仓，同一条核查记了快照里七份文件对不上，腿引到其中四处靠人工改认行号——cite-check 在这几处判的是「今天那一行」，不是腿看到的那一行。它本身只核「原文在那一行」，不核引文撑不撑得起结论、命令输出复不复得出；第一轮 163 处里 84 处是裸行号，没有引文可比、只核行号不越界。

「与改前那一版门禁对着同一棵树输出逐行相同」（调度记录第 173 行）：证得了「这一批没改今天这棵树上这几格的输出」。证不了：① 七处只走到「基准里有这张表」一种结局（真仓的基准取得到、七张表都在），「取不到」「没有表」两种结局与库的开关只由库自证与一份样本判；② 只走到今天表里有的行形，K2-1 那两种行一行都没有，并后语义变了也逐行相同；③ 对照只做了红的四格（results-cited、quoted-result-lines、doc-process-records 两格、code-source-discipline 两格），replay-comparison-units 这一格本来就改了判法、没法对照；④ doc-process-records 的改前副本缺钩子目录、报了一行 cd 错，改前那一版是在少一份库的环境里跑的。四句：不分辨臂；满足的是正文 K3 第四条；改法：对照再加两轮——基准写成不在的提交名、删掉基准里那份 blob（与库自证同一种造法）各跑一遍新旧两版，比输出（推的）。推翻条件：这两轮新旧输出也逐行相同（那说明三种结局的句子都保住了，只剩 K2-1 的行形差）。

## K4　两轮都没看过、这一批碰得到的地方

### K4-1　doc-decisions 的挂账表：第九处同一写法，正是库要防的那一种（打中）

哪一处：`.claude/gate.d/doc-decisions.sh:3355`：「"""基准那一版的挂账表正文；基准取不到、那一版没有这张表返回 None（不比）。」，取法 `.claude/gate.d/doc-decisions.sh:3359`：「done = subprocess.run(['git', 'show', f'{diff_base}:./{ledger_path}'], capture_output=True, text=True)」，`.claude/gate.d/doc-decisions.sh:3362`：「return done.stdout if done.returncode == 0 else None」。表是 `.claude/gate.d/decision-evidence-lag.tsv`（42 行内容），三格调它：cross-decision-status（第 1268 行）、inference-needs-verdict（第 3449 行）、three-way-leg-record（第 3542 行），都是 `lib.ledger_growth(…, lib.base_ledger_text(…))`。git 取不到（基准名不在、对象库缺了那一份）与「基准里没有这张表」同交 None，`ledger_growth` 见 None 就交 []——只缩不涨静默不比，正是 lib-lag-baseline.py 文件头写「不许把第三种当成第二种静默放过」的那一种。

复出来：`probes/ledger_base_probe.py` 从仓里那一份 doc-decisions.sh 现取 `def base_ledger_text` 那一段执行，与库的 baseline_text 对同一个造出来的小仓比。`out/ledger-base.txt` 原样（第二行截在 200 字）：

```
name=对象库完整 base_ledger_text_HEAD=有表 base_ledger_text_no_such_base=None（不比） lib_baseline_text_HEAD=没有问题
name=基准那一版的对象缺了 base_ledger_text_HEAD=None（不比） base_ledger_text_no_such_base=None（不比） lib_baseline_text_HEAD=问题：读不出基准 HEAD 里的 .claude/gate.d/decision-evidence-lag.tsv（git s
```

谁该管：这一批的第 16 号任务（K2-1「七处一起并」，用户定的是「全仓那几处一起并」）；第一轮攻方报的「全仓 7 处」是并法的依据，盘点漏了它，工具员找到的第八处（doc-experiments.sh 第 1069 行 decision-links 的 `.claude/decision-links-pending`）也不是它。现在谁也没管：九十节 ⑪ 只登了第八处。另有一处同形、没查用途：`.claude/gate.d/checker-independence-and-sync.sh:670`：「return result.stdout if result.returncode == 0 else ""」（取基准那一版的格式常数，取不到交空串；是不是「只缩不涨」我没核，推的）。

找法（能复出来的命令）：`grep -rn --include=*.sh --include=*.py -E "show[\"', ]+[^|]*(base|BASE)[^|]*:" .claude/gate.d .claude/hooks scripts/gate scripts/kb | grep -v /fixtures/ | grep -v lib-lag-baseline.py`，命中 13 行，逐行看过，「只缩不涨」那一类的是 doc-decisions 这一处与 doc-experiments 第 1076 行（第八处）。

四句：不分辨臂；格当时看得到 git 的退出码与 stderr（`done.returncode`）；满足的是正文 K4「两轮都没看过、这一批碰得到」——碰得到，因为这一批立的库就是为它这种写法立的；改法：`base_ledger_text` 改调库的 `baseline_text`，问题句判红（推的）。推翻条件：doc-decisions 在调 `base_ledger_text` 之前另判了基准取不取得到（我在三处调用之前没找到）。

### K4-2　`.claude/kb-directories` 写成 `./<目录>` 或绝对路径，stale-candidates 静默不认（打中）

哪一处：登记位的读法在 `scripts/kb/quote-kb.py:447`：「entry = raw.split('#', 1)[0].strip().rstrip('/')」，只去尾斜杠、要目录在，不规整 `./` 与绝对路径；stale-candidates 拿它拼模式，`scripts/kb/stale-candidates.py:179`：「return CARRIER_PATTERNS + [rf'^{re.escape(directory)}/.+\.md$' for directory in kb_directories(repository_root)]」，去比 git 列出的相对路径。doc-registries.sh 的 `kb_candidate_roots` 用 `find` 读目录内容，`./`、绝对路径都照样读得到。

复出来：`probes/kb_directories_probe.py`，`out/kb-directories.txt` 原样：

```
name=form:crates/x/invariants quote_kb_roots=['.claude/kb', 'crates/x/invariants'] stale_candidates_carrier=True
name=form:./crates/x/invariants quote_kb_roots=['./crates/x/invariants', '.claude/kb'] stale_candidates_carrier=False
name=form:<探针根>/crates/x/invariants quote_kb_roots=['.claude/kb', '<探针根>/crates/x/invariants'] stale_candidates_carrier=False
name=form:crates/x/invariants// quote_kb_roots=['.claude/kb', 'crates/x/invariants'] stale_candidates_carrier=True
```

今天登记的两行都是相对写法，不中；登一个新目录时写成 `./…`，引用核对与不变量计数照常，候选扫描漏掉那个目录——就是调度记录里「载体搬家后写回落空」那一形换了个入口。谁该管：第 16 号任务（K2-5 读登记位）；现在谁也没管（没有格判登记位的写法）。改法：`kb_candidate_roots` 把每行 `os.path.normpath` 后再判，绝对路径落在仓根里的改成相对的、落在仓外的拒（推的）。推翻条件：有格或钩子已经判登记位的写法（我在 doc-registries.sh、quote-kb.py 里没找到）。

同一处另一个潜伏的形：扫描认登记目录下各层子目录（`/.+\.md$`），写回员写范围是 `.claude/hooks/agent-write-scope.tsv:12` 的 `crates/singlefs-checker/invariants/*.md`（`*` 不跨 `/`，`.claude/hooks/lib_write_scope.py` 的 glob_to_regex）。今天 invariants 下没有子目录，不中；哪天分了子目录，扫描报出、写回员写不进。第一轮 K2-5 进九十节的「登记位里每个目录在写回员写范围里有一行」那一格只比目录、不比深度，罩不住这一形（推的）。

### K4-3　归档目录搬进 research/prompts/ 之后（没打中，只记一条去向）

查了：门禁 doc-experiments 的 evidence-in-repo 判据二扫 `research/prompts/` 下这一轮新加的 .md（这批 17 份在 HEAD 里一份都没有，全算新加），拿它的 `citation_form` 正则对 `subagent-landing-audit-2026-10-22/*.md`、两轮的材料与报告逐行跑，命中 0 处（草稿里跑的，命令是照抄那个正则的 `python3 -I -`）。三方轮名：这个目录名不是 `_<轮>-body.md`、`<轮>-snapshot/`、`<轮>-main-verification.md` 任何一种，派发闸 ⑥ 与 abandoned-rounds 格不把它当轮。

去向：`scripts/kb/archive-past-rounds.py:49` 写「删 —— `research/results/` 与 `research/prompts/` 下，**在 HEAD 里有、且这一轮没碰过**的文件」——这一批提交之后，再下一次提交就把它整个删掉、引用改成 `git show <提交>:<路径>`。搬不搬它这一条都一样（research/results/ 同样归档），共用约束说这不算丢。只提一句：调度记录第 178 行「往后复跑得出同一组数」要比的那份存档，下一次提交之后只在 git 里。

## 没打中的形状

- K2-1 七处的三种结局（基准取不到、基准里没有表、表为空）与行尾空白：逐处对着开工快照 diff 删掉的行比过，没有一处语义变（表为空时库交 `""`，七处都用 `is not None` 判；行尾空白各调用方照旧 `strip()`）；code-tooling 两格改走命令行与 `lib_long_run.py` 之后，取基准的当前目录仍是被判的根（`cwd=root`、bash 在根里起），没变。
- `lib_long_run.py exempt-check` 的调用方：全仓 `grep -rn -e exempt-check -e base-table` 只有门禁 code-tooling 第 3309 行一处活的调用；还写着 `--base-table` 的只在早先几轮的模型目录 `tools/rerun.sh` 里，它们调的是各自冻结副本里的那一份库，不受这一次改参数影响。
- K2-2 正文点名的三种写法（改登记出处、改产物名、none 行改驱动名再删驱动行）：都换键或被 ③ 判红，绕不过（绕得过的是借键，见 K2-2）；同一对键写两行被登记表去重判红。
- K2-3 骨架：14 种类型照骨架填好，派发闸不误拦（变异分诊员那一格是我填的内容不含变异表）；本会话另 11 份首条派发提示改后退 0（改前那一版只对误拦的两份跑过）。
- K2-4 交回：真数据三种形状，新旧判法逐个相同；续做闸读不到共用库时放行并写 stderr，这是写明的退路，不算判反。
- K2-5 定义 17 处与钩子、门禁：没找到新矛盾（查了哪几对见 K2-5）。
- K4-3 归档目录：evidence-in-repo 判据二 0 处命中；派发闸 ⑥ 与 abandoned-rounds 格的轮名全集认不到它。
- 取样范围：会话记录全量（本项目 3 286 份子 agent 记录、闭窗口 2 144 个子 agent）；派发提示只取了本会话 19 份与 14 种骨架，没取别的会话的派发提示。

## 这条腿自己的限度

- K2-3 的两份「合法派发」是我按「它们真派出去、真干完、交回被收下」认的；「正文里引着的报告标签该不该被拒」是规矩问题，主 agent 定。
- K3-1 的门槛换值只换了三个数的十组取值，「点到」的认法（它是另一道门槛）没换。
- K3-4 的误报漏报按描述分类，没逐个读 327 个派发的提示核「这一次是不是返工」，比例是推的。
- K3-5 的判断只读了 rework.md、blind-rework.md 与调度记录，没重做抽样。
- K4-1 的 checker-independence-and-sync 那一处没核它是不是「只缩不涨」。
- 我提的改法（K2-1 甲乙、K2-2 甲乙、K2-3 甲乙、K3-4 甲乙、K4-1、K4-2）都只在我的模型上量过或只是推的，被攻过零轮。改法修哪一格：

| 改法 | 修的格 | 量过还是推的 |
|---|---|---|
| K2-2 甲 键加实验号与单元名 | K2-2 借键两形 | 推的 |
| K2-2 乙 登记出处文件名带实验号 | K2-2 跨实验借键一形 | 推的 |
| K2-3 甲 「报告：」锚行首 | K2-3 五份全部 | 推的（会让自证一格转红） |
| K2-3 乙 引号里的标签不认 | K2-3 带引号的五份 | 推的 |
| K3-4 甲 `\bfix\b` | K3-4 fixture 那 6 个 | 推的 |
| K4-1 改调库 | K4-1 | 推的 |
| K4-2 规整登记行 | K4-2 两种写法 | 推的 |

## 没做什么

- 没判 K1（辩方那一格），没替主 agent 采纳。没改仓里除报告与模型目录之外的任何文件。
- 没编译，没跑门禁阶段、样本档与重型测试；replay-comparison-units 那一格只取出它的 python 在造出来的小仓里起，没跑整道 doc-experiments。
- 结局账复跑读的是仓外会话记录；副本上的数不是入库装置上的数（这一轮的「装置」就是仓里的 agent-cost.py，模型只导入它、没改它）。
- 要主 agent 跑的命令：无（这条腿起的都已前台跑完，输出在 `out/`）。

## 草稿与副本

草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-r2-opus/`：会话提示的抽取、两次复跑的工作目录（rerun1 排队超时没起来、rerun2 是 `out/` 的出处）。交回前删掉了几个造出来的小 git 仓与暂存区那一版钩子目录的拷贝（`tools/rerun.sh` 重跑会重建）。没有仓副本与编译目录。
