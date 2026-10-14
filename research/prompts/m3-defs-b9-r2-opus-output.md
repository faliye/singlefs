# m3-defs-b9-r2 攻方腿（Opus）报告（2026-10-13）

<!-- doc-lint:not-numbers Z1 Z2 Z4 Z5 Z6 Z7 P1 P2 P3 P4 P5 P6 P7 S1 S2 S3 S4 S5 S6 S7 F-S1a F-S1b F-S3a F-S5a F-S7a -->

攻击面：正文「一」表 S1、S3、S5、S6、S7 与正文「三」各格专问；第二轮分工（第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方）。
记号：r2 = 这一轮冻结副本 `~/.local/share/singlefs/work/m3-defs-b9-r2/repo/`（after-b10-fixup）；r1 = 第一轮冻结副本 `~/.local/share/singlefs/work/m3-defs-b9-r1/repo/`（after-b9-fixup）。「改前也中」= 同一段历史喂 r1 给出同样的错；「这一批带进来」= r1 判对、r2 判错。
钩子与判法只在草稿目录的硬链接拷贝上跑（拷贝里删掉 `multi-host.env` 那一条链接，要改的文件先换新 inode）：派发闸喂 `--check-prompt`；门禁格只抽格里的 python / 函数原样跑，`--check` 交主 agent。开工核 r2 快照：`sha256sum -c research/prompts/m3-defs-b9-r2-snapshot/frozen-copy-sha256.txt` 非 OK 0 行。

## 复跑

整条经内存包装（上限 2G），分两趟（第二趟 s3 含一次真树扫描，挂钟约 3 分钟，大半在排内存包装的队），`--work` 给工作根底下新的空目录：

```
cd /home/fy5090/code/singlefs
RERUN_PARTS="s1 s5 s6 s7" bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b9-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-defs-b9-r2/repo --work <新目录1>
RERUN_PARTS="s3" bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b9-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-defs-b9-r2/repo --work <新目录2>
```

r1 默认取 `--frozen` 同级的 `m3-defs-b9-r1/repo`（`R1_FROZEN` 可改）。末尾逐份打 `same` / `DIFF` / `新出`（`$WORK`、`$MODEL`、`$FROZEN` 先换成占位再比）。腿从仓里的模型目录最后复跑一次：第一趟 7 份全 `same`，第二趟 2 份全 `same`，共 9 份 0 份 DIFF。`RERUN_CORPUS=1` 另跑 S1 的现有派发提示一组（读工作根下各草稿目录的 `prompt.md`，不在仓里、会变，只打合计行，不与 `out/` 比；腿那一次的输出在 `corpus/s1-corpus-run.txt`）。门禁格两组要主 agent 跑：`gate-samples/要主agent跑.txt`。

模型目录 `research/prompts/m3-defs-b9-r2-opus-model/` 每个文件的 sha256（24 份）：

```
e307b70987fe9f7e027094e6336343e757e45cda7e0e2569d520afaf4d0f2c51  corpus/s1-corpus-run.txt
4bc6e6cc975d7d70b7cccaafaa1f9b7d14ce064b19fa8f62facfb5e79fd0b614  feeds/s3/make_tree.sh
e89058d7d8948e562ebcc1b03c5fe1a61ace601d4e9b2afb8fcd0db44cdbc28d  gate-samples/make_trees.sh
e28b8564f77980f89d6332d6f694a33eb5050f6fc3d3a480e19823bf59acae37  gate-samples/要主agent跑.txt
33912e7e3601806545deaa6cb72913e9e4ecbf4aecf48065485b5848b9fb179a  out/s1-ambiguity-prototype.txt
459ab2c7911591ab85c1e4da710518212328639749083f1e37d6fa5f17dd17af  out/s1-fence.txt
12939c00172902e25114954698fc33c524712ee5952c7f028328d49095a63102  out/s1-hosts-fallback.txt
c3f1737007aca8d11a1876fd129457077ec1974bcebea77218b32ec99e0d74a3  out/s3-fixtures-and-real-tree.txt
38e6945d8af7a5d47f28476bfed776753ebcb3ac711883810beccc1bc6a0a099  out/s3-samples.txt
1a358f2f959e795fad112fa042386578a2b2f8a96c42925bf10587d028ad1a17  out/s5-combos.txt
7b4e5180e214c110d03089533fb9b4dd069e03f73710bdc6193c0e8498e5a3b2  out/s5-real-misread.txt
b9cd7a742633ef1d8349e4c6e20ebd7d7c06397a257e80074b592fab5e96240b  out/s6-mask-shapes.txt
cf3ac7226f868c97e809e9f6a8523bee941333e8dcee78692fd8f39484a34a5c  out/s7-user-per-host.txt
6e7bf89bdb237c7b3a773f890427db237909040ad0b42c9da1b2c66934e4606a  tools/rerun.sh
00f3bce445cec45ad63208a3a03192c0d3707be309357b44bb328b3610d42dfc  tools/s1_ambiguity_prototype.py
495ec1de434bac199325f0f6047070cfa649bddf932e8b5f7bd2ffef6ea343c0  tools/s1_corpus.py
fd67d80a12149955ffc0efbc0d2165d0be8780ad71b96e5f15faa8aad7747371  tools/s1_fence_feed.py
ba7a9cb2b75397dc105313909016987e6a0409cf0e18fe186ca346c6bd5ae8c4  tools/s1_hosts_and_fallback.sh
1b1883bcd9c5df70e179baf530633022f81a5052c83269540b02ef9f8d11f156  tools/s3_htd_cell.py
8d6a1f434ffd55c5d6d4cd57320d3e71589d39bab4b8599b931583f7b39df367  tools/s3_patch.py
56b810ceaa4ec61280862595a9ece63836f204ef30197adedd84643dc01f4510  tools/s5_combos.py
b2d13494fa27542603642a4ad9c2bd0acf15b34c0487f6b3f94a5e9267534b46  tools/s5_real_misread.sh
2bb1e52c00b07fa63c7b86cc675b3169f34b22720a39154a52df325f2bb963a4  tools/s6_mask_shapes.py
f5d4d2db3c969c40f0c8d61ae09542b2cdb0a0ab3998b3cba04eca08849fa5fb  tools/s7_user_per_host.sh
```

## 各格判定一览

「量过」= 在拷贝上真跑了钩子入口或抽出的判法，原样输出在模型目录 `out/`；「推的」= 按代码或文字推，没跑。

| 格 | 判定 | 打中的那一步（一句） | r1 上同一段历史 | 哪一版带进来 | 量过 / 推的 |
|---|---|---|---|---|---|
| S1-甲 | 打中（照定义做、结果错：违反「默认 31、固定不看忙闲」） | 列表项里开的围栏、关的那一行顶格写；或 `<details>` 折起来的原文里有一个空行：CommonMark 在那里另开一个围栏块，把块外真的「线程上限：16」/「多机：现算」遮掉，骨架代码块里的 31 / 规范句反被当声明，放 | r1 拒 | P1（批 10 改用 markdown-it） | 钩子量过 `out/s1-fence.txt` 第 2、4 行、`out/s1-hosts-fallback.txt` 第 1–2 行；「读者眼里是 16」推的 |
| S1-乙 | 打中（钩子拒而定义没写） | 缩进代码块、到文末没闭合的围栏、在列表项结束处结束的围栏：照 CommonMark 都是代码块，定义写「代码块（照 CommonMark）里的行不判」，钩子照判、拒 | r1 同拒，但 r1 定义写的是「三个反引号围起来的代码块」，那时字面一致 | P1 改的定义字面 | 量过 `out/s1-fence.txt` 第 8–10 行 |
| S1-丙 | 打中（改前也中，另立账） | 引用块里开的围栏关的那一行忘了写 `>`、关的那一行多缩进 4 格或前面是制表符、开四个反引号关三个：CommonMark 与钩子都把真「线程上限：16」算进代码块，放 | r1 同放 | 改前也中 | 钩子量过 `out/s1-fence.txt` 第 3、5–7 行；读者一侧推的 |
| S1-丁 | 专问（退路触发了，误伤 0 份） | PATH 上第一个 `python3` 是没装 markdown-it-py 的 venv 时退路触发，闭合了的代码块里的示例照判、拒；工作根下 50 份现有派发提示 0 份有围栏块，退路下判定变了的 0 份 | — | P1 | 量过 `out/s1-hosts-fallback.txt` 第 3 行、`corpus/s1-corpus-run.txt` 末行 |
| S3-甲 | 打中（文件头说算而判法不算） | 赋给线程变量的那个变量在同一份文件里只经登记位读法赋值（`\|\| X=$default_threads`），但实际取值是 `:=` 补的 8、被 source 的文件给的 8、`${SINGLEFS_THREAD_CAP:-$另一个变量}` 退回的 8、登记值除以一个变量：四份都不点名 | r1 四份都点名 | P3 | 量过 `out/s3-samples.txt`（r2 段 3 行、r1 段 8 行） |
| S3-乙 | 打中（照格的出路做仍红，Z2-乙 改后又中） | 照仓里 `run-with-memory-cap.sh` 的写法 `"$(heavy_thread_default 2>/dev/null)"` 取进变量，`2` 被当成「别的数字」，`\|\| X=$变量` 点名；经自己的小函数取登记值同样点名 | r1 同点名 | 改前也中（P3 没修到这两形） | 量过 `out/s3-samples.txt` r2 段 |
| S5-甲 | 打中（照规则做、结果错） | 真记录第 1594 行（「判决写完走提交前那几步」，这一轮接着收尾）也带这一轮的名字与「」原话；登记行〔〕里写转述、行号写 1594，P5 照「找不到而行号提示那一行有名字」收，⑥ 放实现员改那一轮快照里的文件 | r1 不收、⑥ 拒 | P5 | 量过 `out/s5-real-misread.txt` 第 1–2 行 |
| S5-乙 | 专问（三种日常改动单做都收；两两合做里两种照旧不收） | 漂移、重复、改字单做 r2 都收；漂移加重复、漂移加改字 r2 不收（r1 也不收）；漂移加改字而提示落到开轮那一行（带名字与「」）时 r2 收错的那一行 | 见 S5 一节表 | 合做不收：改前也中；收错行：P5 | 量过 `out/s5-combos.txt` |
| S6 | 没打中（P6 的范围内） | 挪片段与路径打码在本腿喂的九形里六形干净；剩下的两种形状（两个私有值首尾重叠、主机名换了大小写）是打码只认逐字整词，改前也中 | r1 同 | 改前也中 | 量过 `out/s6-mask-shapes.txt`（函数级，没起格） |
| S7-甲 | 打中（照定义做、结果错：违反「给了参数照参数」） | 用户给了逐台不同的线程数，主 agent 能写进提示的两种写法 ⑭ ⑮ 都放；子 agent 照共用约束对另起一行的「一概不照」，照已判行则「双机时两台各取这一个数」，两种都跑不成用户给的参数 | r1 钩子同放；r1 定义没有「一概不照」 | P7 的文字 | 钩子量过 `out/s7-user-per-host.txt`；子 agent 一侧推的 |
| S7-乙 | 专问 | 钩子判不到的那一半（子 agent 实际起多少线程、几台）没有任何钩子或门禁判：钩子目录里没有一处拿派发提示的 N 去核子 agent 的命令；主 agent 定义第 55 行没有「一概不照」那一句，只叫说明另起一行写 | — | P7 | grep 量过（见 S7 一节）；谁判推的 |

## S1　⑮ ⑭ 认代码块改用 markdown-it-py 的 commonmark 预设（P1）

被判的字面（都取 r2）：
- `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/main-agent.md:53`：「代码块（照 CommonMark）里的行 ⑮ 不判」；同一行「子 agent 起重活只照…那一个整数，提示里别处写的线程数一概不照」。
- `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/hooks/runner-dispatch-guard.sh:74`：「到文末没闭合的、在列表项或引用块结束处结束的照判；缩进代码块与 HTML 块（含 <!-- … --> 注释）不遮，里面露出的三个反引号也不开块」
- 遮不遮只看 markdown-it 给的围栏块与块的最后一行：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1163`：「if re.fullmatch(re.escape(token.markup[0]) + "{" + str(len(token.markup)) + r",}[ \t]*", closing):」

### S1-甲　列表项里的围栏关在顶格、`<details>` 里有空行：CommonMark 另开一块，遮掉块外的真声明（钩子量过，读者一侧推的）

历史（investigator，2 小时内）：主 agent 照用户的话写 16（与第一轮 Z1-甲 同一个起因：忘了已判行，或把「机器忙」写成数），顺手贴上一次派发时的起法与骨架。贴法是列表项里缩进两格开围栏、关的那一行顶格写——LLM 写 markdown 常见的手误：

````
- 上一次派发的原文：
  ```
  线程上限：31
```
线程上限：16
上一次的骨架：
```
线程上限：31
```
````

（外层四个反引号是本报告的，提示里只有里面那几行，前面是开工先读、重型测试、草稿目录、报告、活、预计挂钟六行。）CommonMark：顶格那一行三个反引号不是列表项里那块的结尾（列表项在它前面结束，那块没闭合），而是新开一块，一直到「上一次的骨架：」下面那一行才关；于是「线程上限：16」在代码块里，骨架里的「线程上限：31」露在块外。钩子照此遮掉 16、只看到两行 31，放。r1（逐行配对）把顶格那一行当关块，16 露出来，拒。

`research/prompts/m3-defs-b9-r2-opus-model/out/s1-fence.txt:2`：「C1-list-fence-closed-at-column-0	r2=0	r1=2」
`research/prompts/m3-defs-b9-r2-opus-model/out/s1-fence.txt:2`：「CommonMark 代码块外：15:线程上限：31」

同一种另开一块换一个口子：`<details><summary>上一次派发的原文</summary>` 折起来的原文里有一个空行。HTML 块到空行结束，块里那一行三个反引号不算围栏；空行之后的下一行三个反引号（读者眼里是那块的结尾）在 CommonMark 里是开块，又遮掉块外的 16：

`research/prompts/m3-defs-b9-r2-opus-model/out/s1-fence.txt:4`：「C3-details-fold-with-blank-line	r2=0	r1=2」

⑭ 同形（派执行员、超过 2 小时）：块外真的「多机：每台线程按此刻空闲现算，机器忙时按负载降」被遮，骨架代码块里的规范句被当声明：

`research/prompts/m3-defs-b9-r2-opus-model/out/s1-hosts-fallback.txt:1`：「H1-list-fence-closed-at-column-0	r2=0	r1=2」
`research/prompts/m3-defs-b9-r2-opus-model/out/s1-hosts-fallback.txt:2`：「H2-details-fold-with-blank-line	r2=0	r1=2」

四句：
- 分不分辨臂：分。r1 拒、r2 放，P1 带进来的；第一轮 Z1-甲 那四形 r2 都拒（P1 修到）。
- 被判的系统看不看得到：看得到。顶格那一行三个反引号、`<details>` 与空行都在提示文字里；本腿原型（F-S1a）用同一份提示就算得出「这一行两种认法不一致」。
- 满足的分句：不是「定义说拒而钩子不拒」——照定义字面，16 那一行在 CommonMark 的代码块里，不判是对的。是「照定义做却违反用户定案（默认 31、固定不看忙闲）」：子 agent 不跑 CommonMark，它照「线程上限：」那一行起重活，读到的是 16（推的：没派真 agent）。病根在 P1 把「读者眼里」换成了 CommonMark：第一轮攻方拿 markdown-it 当读者一侧时自己在限度里写了，`research/prompts/m3-defs-b9-r1-opus-output.md:352`：「那一侧是 markdown-it-py 的解析，不是 LLM 的读法」，P1 把这个代理当成了定义。
- 跑前条款给的改法：P1 是来源。F-S1a（本腿，原型量过、被攻过零轮）在 C1、C3、H1 上拒，见「我提的改法」。

### S1-乙　定义写「代码块（照 CommonMark）」，CommonMark 的另三种代码块照判（量过）

CommonMark 的代码块有缩进代码块与围栏代码块两种，围栏代码块没有关的那一行时到文末或容器结束为止（规范里仍是代码块）。钩子只遮「闭合了的围栏块」，文件头写明另三种照判（上面第 74 行）；十份定义、共用约束与主 agent 定义只写「代码块（照 CommonMark）」，没有「闭合了的围栏块」这个限定。三种各放一个示例行，r2 都判、拒：

`research/prompts/m3-defs-b9-r2-opus-model/out/s1-fence.txt:8`：「D1-indented-code-block-example	r2=2	r1=2」
`research/prompts/m3-defs-b9-r2-opus-model/out/s1-fence.txt:9`：「D2-unclosed-fence-at-end-example	r2=2	r1=2」
`research/prompts/m3-defs-b9-r2-opus-model/out/s1-fence.txt:10`：「D3-list-fence-ended-by-container-example	r2=2	r1=2」

四句：钩子两版同判，分辨的是定义字面（r1 写「三个反引号围起来的代码块」，与钩子一致；r2 改成「照 CommonMark」，多出这三种）；看得到；分句「钩子拒而定义没写」；改法 F-S1b（文字，推的）：定义与共用约束写成「闭合了的围栏块（照 CommonMark 认）里的行不判」，与钩子文件头同义。

### S1-丙　CommonMark 与读者分歧、改前也中的四形（另立账）

引用块里开的围栏、关的那一行忘了写 `>`；关的那一行多缩进 4 格；关的那一行前面是制表符；开四个反引号、关三个。四形 CommonMark 都把块外的 16 算进代码块（关的那一行不算关），r1 的逐行配对也不认那一行是关块，两版都放：

`research/prompts/m3-defs-b9-r2-opus-model/out/s1-fence.txt:3`：「C2-blockquote-fence-closed-without-marker	r2=0	r1=0」
`research/prompts/m3-defs-b9-r2-opus-model/out/s1-fence.txt:5`：「C4-closing-fence-indented-4	r2=0	r1=0」

改前也中，照跑前条款另立一笔账；F-S1a 原型在这四形上也拒。

### S1 专问

| 写法 | r2 | r1 | CommonMark 与读者 | 结论 |
|---|---|---|---|---|
| HTML 块 | `<details>` 里有空行那一形放（S1-甲 C3）；HTML 注释里露出三个反引号那一形（第一轮 T3）拒 | C3 拒 | HTML 块到空行结束，读者多半按反引号配对 | 打中 S1-甲 |
| 链接引用定义 `[线程上限]: <16>` 加真行 31 | 放 | 放 | 链接引用定义不当声明行，两边一致 | 没打中（`out/s1-fence.txt` 第 14 行） |
| setext 标题（16 下一行 `---`） | 拒 | 拒 | 标题照判 | 没打中（第 12 行） |
| 表格单元里的 `\| 线程上限：16 \|` | 报缺行、拒 | 同 | 行首是竖线，不当声明行；读者也许把它读成 16，结局都是拒 | 没打中（第 13 行；行首是竖线） |
| 制表符缩进 | 关的那一行前面是制表符：放（S1-丙） | 放 | CommonMark 制表符当 4 格 | 改前也中 |
| `\r\n` | `--check-prompt` 入口放（第 11 行，文本方式读文件把 `\r\n` 并成 `\n`）；钩子 stdin 入口（JSON 里的串保留 `\r`）同一份提示写对的「线程上限：31」拒，说「整数后面接了别的话」 | stdin 同拒 | — | 改前也中、从未被看过；草稿量的（没进模型目录），影响推的：主 agent 写的提示是 `\n` 换行，碰不上 |

库载不进的退路（量过）：钩子文件头 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/hooks/runner-dispatch-guard.sh:5`：「# run-condition: command python3」，判法由 `python3 /dev/fd/3` 起（同一份第 449 行）：取 PATH 上第一个 `python3`。系统的 `python3-markdown-it` 只在 `/usr/lib/python3/dist-packages` 里；从激活了 venv（或 conda、pyenv、uv 的解释器排在前面）的 shell 起 Claude Code，钩子的 `python3` 就载不进它。本腿建一个不带 pip 的 venv 放在 PATH 最前面，同一份「闭合了的代码块里写示例」的提示，照常放、退路下拒：

`research/prompts/m3-defs-b9-r2-opus-model/out/s1-hosts-fallback.txt:3`：「F1-closed-fence-example	r2=0	r1=0	r2-venv=2	派发闸载不进 markdown-it-py（ModuleNotFoundError("No module named 'markdown_it'")）」

误伤多少现有派发提示（量过）：工作根下各草稿目录里主 agent 存的 `prompt.md` 50 份（跳过禁读的那一个目录），逐份照常判一次、开弄坏开关 `code-block-library-missing` 判一次（与载不进同一条路）：

`research/prompts/m3-defs-b9-r2-opus-model/corpus/s1-corpus-run.txt:51`：「合计：50 份提示，认得出类型 50 份，判了 50 份，载不进时判定变了的 0 份」

0 份变，是因为 50 份里一份围栏块都没有（草稿里用 markdown-it 数的：有围栏块的 0 份）。反过来说：P1 换进来的认法在这批真提示上一次都没起作用，起作用的只有本腿与第一轮造的提示；换进来的代价是 S1-甲 那几形。这一条是「产生结论的方法」：P1 的收益没有在真提示上量过。

markdown-it 自己抛异常会不会让整道闸放行（从未被看过）：`without_code_blocks` 只把 import 包在 try 里，解析抛了就一路抛出 `judge_dispatch`。本腿拿 37 种零件（各种围栏、缩进、引用、HTML、实体、孤立代理字符、NUL、` `）随机拼了 412,474 份、再加一万层引用块与一万层列表，markdown-it-py 3.0.0 一次都没抛（草稿量的）。没打中。

## S3　heavy-thread-defaults：`|| X=$var` 看 var 在同一份文件里怎么赋值（P3）

被判的字面：
- `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/code-tooling.sh:1880`：「#   如 default_threads="$(heavy_thread_default)"；registry_sourced_variables）的也不算钉数；有一处赋了字面数、算术或别的值的照算。」
- `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/code-tooling.sh:2027`：「（上限变量带了字面回退数、算术里另夹了数这类不算）；有一处赋了别的值就不算。」
- 判法：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/code-tooling.sh:2031`：「from_registry = REGISTRY_READING.search(rhs) is not None and not REGISTRY_READING_OTHER_NUMBER.search(REGISTRY_READING.sub("", rhs))」——右边「含」登记位读法、去掉之后「没有数字」就算；赋值只认 `名=` 形，`${名:=…}`、被 source 进来的文件里的赋值都看不见。

样本树 `feeds/s3/make_tree.sh`（`heavy-threads.sh`、`heavy_threads.py`、`capped.sh`、登记与那三份的排除行从冻结副本拷），八份脚本最后一行都是 `[[ -n "${SAMPLE_SWEEP_THREADS:-}" ]] || SAMPLE_SWEEP_THREADS=$default_threads`，抽出格的判法原样跑（`tools/s3_htd_cell.py`，拷自第一轮攻方模型，只把 `heavy_threads.py` 改取门禁所在仓那一份，与格同）。

### S3-甲　实际取值不是登记值的四形不点名（量过）

| 样本 | `default_threads` 怎么来 | 实际默认 | r2 | r1 |
|---|---|---|---|---|
| `colon_equals_fallback.sh` | `default_threads="$(heavy_thread_default)" \|\| true` 再 `: "${default_threads:=8}"` | 登记文件写坏、不在时 8（登记位的文件头写「拒了不退回」） | 不点名 | 点名 |
| `sourced_literal.sh` | source 同目录 `sample_defaults.sh`（里面 `default_threads=8`），本文件只在它空时从登记位取 | 一直是 8 | 不点名 | 点名 |
| `cap_fallback_via_variable.sh` | `fallback_threads=8`、`default_threads="${SINGLEFS_THREAD_CAP:-$fallback_threads}"` | 不经 capped.sh 起时 8 | 不点名 | 点名 |
| `registry_divided_by_variable.sh` | `divisor=2`、`default_threads="$(( $(heavy_thread_default) / divisor ))"` | 登记值的一半 | 不点名 | 点名 |

`research/prompts/m3-defs-b9-r2-opus-model/out/s3-samples.txt:1`：「== 门禁 r2 判本腿样本」，下面只点名 `conditional_literal.sh`、`stderr_silenced.sh`、`via_own_function.sh` 三处（第 2–4 行）；同一份第 6 行起 r1 点名 8 处，含上表四份：
`research/prompts/m3-defs-b9-r2-opus-model/out/s3-samples.txt:8`：「scripts/experiments/cap_fallback_via_variable.sh:4  写死的线程数默认（|| / && 赋值）」
`research/prompts/m3-defs-b9-r2-opus-model/out/s3-samples.txt:12`：「scripts/experiments/sourced_literal.sh:5  写死的线程数默认（|| / && 赋值）」

四句：
- 分不分辨臂：分。r1 四份都点名，r2 都放，P3 带进来的。`:=` 那一形让第一轮另立账的 Z2-丙（`${X:=8}` 不点名）变得更糟：原来只在直接写线程变量时漏，现在经一个中间变量也漏，而 `|| X=$中间变量` 在 r1 是点名的。
- 看得到：看得到。字面 8 与 `divisor=2` 在同一份文件里；`sourced_literal.sh` 那一形的 8 在被 source 的文件里，格也扫那一份，只是没把两份连起来。
- 满足的分句：「文件头说算而判法不算」：第 2027 行写「上限变量带了字面回退数、算术里另夹了数这类不算」，丙丁两形就是这两样换成经一个变量；甲乙两形的值也不是登记值。结局是「照规则做、结果错」：违反用户定案「默认 31、固定不看忙闲」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/heavy-thread-default:6`：「机器忙时也固定 31、不看忙闲」）。
- 改法：P3 是来源；F-S3a（本腿，原型量过、被攻过零轮）四形都点名，见「我提的改法」。

### S3-乙　照格的出路取登记值、只是丢掉 stderr 或经自己的小函数：仍被点名（量过，Z2-乙 改后又中）

格的出路：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/code-tooling.sh:2430`：「shell 先 source scripts/hosts/heavy-threads.sh 再 heavy_thread_default 取」。仓里现成的取法就丢 stderr：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/scripts/hosts/run-with-memory-cap.sh:505`：「if registered="$(heavy_thread_default 2>/dev/null)"; then printf '%s' "$registered"」。照它写 `default_threads="$(heavy_thread_default 2>/dev/null)" || exit 2`，`2>` 里的 2 被「别的数字」那一条认成钉数：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/code-tooling.sh:2022`：「REGISTRY_READING_OTHER_NUMBER = re.compile(r"(?<![\w.])[0-9]+(?![\w.])")」。

`research/prompts/m3-defs-b9-r2-opus-model/out/s3-samples.txt:3`：「scripts/experiments/stderr_silenced.sh:4  写死的线程数默认（|| / && 赋值）」
`research/prompts/m3-defs-b9-r2-opus-model/out/s3-samples.txt:4`：「scripts/experiments/via_own_function.sh:5  写死的线程数默认（|| / && 赋值）」

四句：不分辨（r1 同点名）；看得到（那一行就是登记位读法）；分句「照格的出路做仍红」，与第一轮 Z2-乙 同一个分句——P3 修了 `"$(heavy_thread_default)"` 原样那一形，仓里自己用着的 `2>/dev/null` 那一形没修到；改法 F-S3a 放行 `2>/dev/null` 那一形（量过，`out/s3-samples.txt` 第 16 行起 patched 段不再点名 `stderr_silenced.sh`），经函数那一形照旧点名（F-S3a 碰不到）。

### S3 专问：var 来源判法的边界

| 边界 | r2 怎么判 | 依据 |
|---|---|---|
| var 在函数里赋 | 赋值按整份文件找、不分函数，`local threads="$(heavy_thread_default)"` 与顶层同判；经函数的输出赋（`default_threads="$(pick_threads)"`）右边没有登记位读法，点名 | 前一半推的（按第 2031 行）；后一半量过（`via_own_function.sh`） |
| var 在 source 进来的文件里赋 | 看不见：被 source 的文件里给字面数、本文件只在空时从登记位取，不点名（S3-甲 `sourced_literal.sh`）；反过来被 source 的文件里从登记位取、本文件直接 `\|\| X=$var`，本文件里没有它的赋值，点名 | 前一半量过；后一半推的 |
| 赋两次 | 两次都是登记位读法不点名；一次登记位、一次字面数点名 | 量过（`assigned_twice_registry.sh` r2 不点名、`conditional_literal.sh` 点名） |
| 条件赋 | if 一支登记位、else 一支字面 8：点名；`\|\| true` 后面 `${var:=8}` 补默认：不点名（S3-甲） | 量过 |

## S5　叫停登记的出处：行号提示参与取那一行（P5）

被判的字面：
- `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/rules/three-way-inference.md:121`：「〔〕里逐字照抄出处那一行里的一段原话」；同一行「原话找不到（被删或改了字）而行号提示那一行还有这一轮的名字的，取提示那一行」。
- `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/hooks/lib_abandoned_rounds.py:15`：「原话找不到（追加、改名、删日期改了字）而行号提示那一行还有这一轮的名字」
- `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/hooks/lib_abandoned_rounds.py:232`：「if not hits and hint_usable and name_in(cited_lines[line_number - 1], round_name):」——取提示那一行之前不看〔〕里的字与那一行有没有一点关系。
- 门禁格会把「照行号提示取」的那几行列进成功句（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/doc-process-records.sh:933`：「照行号提示取那一行照收的」），派发闸 ⑥ 不列、直接放。

### S5-甲　真记录里同一轮有两行都带名字与「」原话：〔〕写转述、行号写错的那一行，P5 照收（量过，真树）

真记录 `records/2026-10-01-删除与并发事务.md` 第 1594、1595 行都写 m3-batch63-code-r1 与用户原话。第 1594 行说的是这一轮接着收尾、写判决：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/records/2026-10-01-删除与并发事务.md:1594`：「判决写完走提交前那几步」；第 1595 行才是两条腿停掉、不写判决（登记表现有那一行的出处）。

历史：用户说完第 1594 行那句（「63 打完 不要再打了」）、还没说第 1595 行那句时，主 agent 把「不要再打了」读成叫停，登记一行 `m3-batch63-code-r1	2026-10-11	用户 2026-10-11 叫停（records/2026-10-01-删除与并发事务.md 第 1594 行〔m3-batch63-code-r1 用户叫停，不再打〕）	不写判决`——〔〕里是转述，不是逐字照抄。树：r2、r1 各硬链接拷一份，记录截到第 1594 行（那一刻的样子）、登记表那一行换掉（`tools/s5_real_misread.sh`）。判叫停表，再拿派发闸 ⑥ 判「派实现员改那一轮快照里的 `verdict_store.rs`」：

`research/prompts/m3-defs-b9-r2-opus-model/out/s5-real-misread.txt:1`：「r2	叫停表：m3-batch63-code-r1	收	⑥ 派实现员退出=0」
`research/prompts/m3-defs-b9-r2-opus-model/out/s5-real-misread.txt:2`：「r1	叫停表：m3-batch63-code-r1	不收：第 8 行的出处 records/2026-10-01-删除与并发事务.md 里找不到〔m3-batch63-code-r1 用户叫停，不再打〕这段原话」

四句：
- 分不分辨臂：分。r1 不收、⑥ 拒；r2 收、⑥ 放。P5 带进来的。
- 看得到：看得到。判的那一刻〔〕里的字在那份文件里一处都找不到；P5 把「找不到」整个交给行号提示，不再要〔〕与提示那一行有任何共同的字。
- 满足的分句：「照规则做、结果错」：规则字面「〔〕里逐字照抄」在这一形上没有牙齿——转述的〔〕与逐字照抄的〔〕判法上一样收；结局是一个要写判决的轮被当成叫停，⑥ 不再挡实现员改它快照里的文件（用户定案「在跑的不动」那一类的保护失效）。
- 改法：P5 是来源。F-S5a（推的，被攻过零轮）：找不到时只在「〔〕与提示那一行去掉改名表里的新旧名、日期之后逐字相同」时取提示那一行，或者要〔〕在登记那天的版本里能找到（`git log -S` 取登记那次提交的父提交里那份文件）；后一种碰不到没进 git 的记录。

### S5-乙　专问：漂移、重复、改字三种日常改动各回到什么判定（量过，合成小树）

小树（`tools/s5_combos.py`）：登记那一刻〔〕原话「去向：hx-term-r1 两条腿停掉（第一条流的崩溃枚举没跑完）」在 `records/log.md` 第 10 行恰好一次。三种日常改动：漂移（前面的小节补一句，第 10 行往下挪一行）、重复（末尾追加补记，逐字抄了原话）、改字（照术语改名把 records/ 里的「第一条流」换成「新池新建文件流」，登记表在 `research/prompts/` 下不换）；单做与两两合做，各版自己的 `lib_abandoned_rounds.py` 判。

| 改动 | r2 | r1 |
|---|---|---|
| 漂移 | 收（原话现在在第 11 行） | 收 |
| 重复 | 收 | 不收（命中 2 次） |
| 改字 | 收 | 不收（找不到） |
| 漂移加重复 | 不收（命中第 11、13 行，提示第 10 行不在其中） | 不收 |
| 漂移加改字 | 不收（提示的第 10 行是开轮那一行，有名字、没有「」原话） | 不收 |
| 重复加改字 | 收 | 不收 |
| 漂移加改字，开轮那一行带用户「」原话 | 收（取的是开轮那一行） | 不收 |

`research/prompts/m3-defs-b9-r2-opus-model/out/s5-combos.txt:5`：「drift+dup	r2：不收：第 1 行的出处 records/log.md 里〔去向：hx-term-r1 两条腿停掉（第一条流的崩溃枚举没跑完）〕这段原话命中 2 次（第 11、13 行），行号提示的第 10 行不在其中」
`research/prompts/m3-defs-b9-r2-opus-model/out/s5-combos.txt:8`：「open-line-drift+reword	r2：收	r1：不收」

- 单做：P5 三种都修到（r1 第一轮 Z4 甲乙丙那三形）。
- 两两合做：漂移加重复、漂移加改字照旧不收，r1 同不收：改前也中，另立账。日常里两种常在同一批里出现（同一份调度记录前面补一句、后面追加补记，或补一句再跑一次术语改名），P5 的「行号只当提示」只在行号没动时帮得上。
- 第一轮攻方写的弱点（提示那一行恰好是另一句带这一轮名字与「」的话）：合成树上造出来了（最后一行，收的是开轮那一行）；真记录里也造得出来——S5-甲 的第 1594 行就是这样一行，且它说的是相反的事（写判决）。在合成树上这一次收的结局碰巧是对的（那一轮确实叫停了），收的理由是错的那一行；S5-甲 收的结局本身错。

## S6　doc-text：date-once 红行、sweep-term 输出打码，片段挪到私有值边上（P6）

没打中（P6 的范围内）。喂法（`tools/s6_mask_shapes.py`，从 r2 的 doc-text 抽出私有值那一份共用函数，拿编的词表直接喂三条通道：sweep-term 输出经 `shown_line`、date-once 红行经 `shown_path`、行文片段经 `snippet_bounds` 加 `shown_text`）九形里六形打码干净（`out/s6-mask-shapes.txt` 第 1–3、5–7 行）：逐份列的路径、`、` 连成一长串的「没扫的」清单、带「」的路径、权限报错里的路径、文件名里的主机名都打成〔文件名含私有值〕；剩下三行是下面两种形状（第 4、8、9 行）。

剩下两形是打码只认逐字整词（r1 的 `shown_text` 正文与 r2 逐字相同、只是行号不同，按代码推改前也中；`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/doc-text.sh:251`：「text = text.replace(word, VALUE_MASK)」），不是 P6 带进来的：
- 两个私有值首尾重叠（编的清场命令里一个服务名 `demo-gpu-service`、一个检查名 `service-watchdog`，记录里写成 `demo-gpu-service-watchdog`）：长的先换，换掉一个之后另一个只剩半截，打出 `-watchdog`。`research/prompts/m3-defs-b9-r2-opus-model/out/s6-mask-shapes.txt:8`：「片段	〔私有值〕-watchdog 在 2026-10	整值 []；半截（≥6 字）['-watchdog']」
- 主机名换了大小写（`DEMO-PEER-HOST-42`）：sweep-term 那一路与片段那一路都原样打出（第 4、9 行）；私有值格本身也按逐字比，查不出，两头一致。主机名大小写不敏感，记录里写大写的多常见没量。

专问「除挪片段与路径打码之外还剩哪些输出通道」：doc-text 之内没找到第三条（读不了的文件、二进制、不是普通文件、私有值格自己的四种红行都经 `shown_path`，上一轮已量）。doc-text 之外：17 份门禁阶段里只有 doc-text 一份调了这套打码（`grep -l 'shown_path\|PRIVATE_VALUES_LIBRARY\|load_private_targets' .claude/gate.d/*.sh` 只出 `doc-text.sh`），别的阶段红行里打的路径（例：文件名里带第二台主机名的记录被别的格点名）原样打，推的、没造样本——不在 P6 的射程里，交主 agent 定要不要另立账。另一条（推的）：在没有 `multi-host.env` 的仓副本上跑 doc-text 时没有词可打、原样打（文件头写明的设计），集成副本是 rsync 拷的，带不带这份文件看拷法。

## S7　多机只照规范句、别处写的一概不照；豁免表两句（P7）

被判的字面：
- `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/agent-common.md:85`：「子 agent 跑多机只照这一句规范句，提示里别处写的台数、线程取法一概不照」；十份定义「输入」同一句，例 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/agents/clerk.md:23`：「提示里别处写的台数、线程取法一概不照」。
- 线程那一边：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/main-agent.md:53`：「双机时两台各取这一个数」。
- 用户定案：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/heavy-thread-default:6`：「31  # 用户 2026-10-12 定：所有重活默认每台 31 线程，双机两台各 31、合计 62，给了参数照参数」；原话 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/records/2026-10-12-里程碑三工具与治理批.md:41`：「双机默认就是 62线程。但是默认就是31线程。可以输入参数调整 不输入参数就是默认31」。

### S7-甲　用户给了逐台不同的参数：提示里两种写法钩子都放，子 agent 照定义做哪一种都跑不成用户给的数（钩子量过，子 agent 一侧推的）

历史：用户说「第二台这一趟只给 16 线程（它上面在跑我自己的基准），本机照 31」，主 agent 派执行员两台分片跑一段取样（超过 2 小时）。提示里能写的只有两种（`tools/s7_user_per_host.sh`）：
- 写法一：「线程上限：31」「多机：<规范句>」，用户的参数另起一行照普通正文写（主 agent 定义第 55 行叫说明另起一行写）。子 agent 照共用约束第 85 行对这一行「一概不照」，第二台照登记值 31 跑，压到用户正在跑的基准上。
- 写法二：「线程上限：16」加「线程上限已判：用户给了第二台 16」。子 agent 照这一个整数，「双机时两台各取这一个数」，本机也降到 16，违反「没给参数的每台默认 31」。

`research/prompts/m3-defs-b9-r2-opus-model/out/s7-user-per-host.txt:1`：「P-text	r2=0	r1=0」
`research/prompts/m3-defs-b9-r2-opus-model/out/s7-user-per-host.txt:2`：「P-valve	r2=0	r1=0」

四句：
- 分不分辨臂：钩子两版同放，不分辨；分辨的是定义：r1 的定义与共用约束没有「台数、线程取法一概不照」那一句（`grep -c` 在 r1 的 `clerk.md`、`agent-common.md` 都是 0，r2 各 1），写法一在 r1 下子 agent 照不照没写，在 r2 下写明不照。P7 的文字带进来的。
- 看得到：钩子看得到那一行，但 ⑭ 只判「多机：」一行的写法，没有逐台参数的出路；`.claude/hooks/` 下没有「多机已判」一类的放行行（grep 0 处）。
- 满足的分句：「照定义做却违反用户定案」——用户定案的「给了参数照参数」在逐台不同的参数上无路可走。
- 改法：P7 是来源。F-S7a（推的，被攻过零轮）：⑭ 加一行放行「多机已判：<理由>」，子 agent 只在有这一行时照提示里逐台写的参数，没有就照规范句；或者在多机配置里逐台写线程数（与 `multi-host.env` 的「每台可选配」同一处），提示照旧只写规范句。

### S7-乙　专问：定义说「一概不照」之后，钩子判不到的那一半由谁判

钩子判的是提示文字的形状（⑭ 一行逐字等于规范句、⑮ 一个整数）。判不到的那一半是子 agent 实际怎么起：起几台、每台几线程、是不是经 `capped.sh N` 起、N 是不是提示那一个数。现查（grep，量过）：`.claude/hooks/` 里提到 `capped.sh` 的六份（`bash-command-detector.sh`、`heavy-test-guard.sh`、`lib_heavy_tests.py`、`lib_shell_words.py`、`write-guard.sh`、`lib_workspace_cargo_lock.py`）都只把它当包装剥开、往里判重型测试与内存包装（`write-guard.sh`、`bash-command-detector.sh` 只在拒绝说明的出路里写到它），没有一处拿派发提示的 N 去核命令里的数；没有一处判「多机」。所以那一半没有机械判法，只剩子 agent 自守与主 agent 读交回（推的）。另一处缺口（量过）：P7 的那一句只进了共用约束与十份定义，主 agent 定义里没有（`grep -o '[^；。]*多机[^；。]*一概不照[^；。]*'` 在 r2 `main-agent.md` 0 处、`agent-common.md` 1 处）；主 agent 定义第 55 行仍叫「说明另起一行写」，主 agent 照它把台数、线程的说明写进下一行，不知道子 agent 会一概不照——正文「一」表 S7 一行把 `.claude/main-agent.md` 列进「在哪」，冻结副本里没有这一句。

豁免表那两句（「基准那一版没有这张表时不比；重钉提交进基准之后不再红」）：文字与判法一致，第一轮 Z7-甲 已量，本腿没造新历史（见「没打中的形状」）。

## 正文「三」各格专问（汇总，细节在各节）

| 格 | 答 |
|---|---|
| S1 | 不一致的写法：列表项里开的围栏关在顶格、`<details>` 里有空行（这一批带进来）；引用块里关的那一行漏 `>`、关的那一行多缩进 4 格或用制表符、开四关三（改前也中）；定义写「照 CommonMark」而缩进代码块、没闭合的围栏照判（字面不一致）；链接引用定义、setext 标题、表格两版一致。退路由 PATH 上没装库的 `python3`（venv 一类）触发；50 份现有提示误伤 0 份，因为 0 份有围栏块 |
| S3 | 函数里直接赋同顶层判、经函数输出赋点名（误红）；source 进来的文件里给字面数看不见（漏）；赋两次照「每一处都是登记位」判；条件赋一支字面点名、`:=` 补默认漏；另有 `${SINGLEFS_THREAD_CAP:-$变量}`、除以一个变量两形漏，`2>/dev/null` 一形误红 |
| S5 | 单做三种都收；漂移加重复、漂移加改字不收（改前也中）；第一轮写的弱点在合成树与真记录里都造得出，真记录那一形（S5-甲）收的结局本身错 |
| S6 | doc-text 之内没找到第三条；打码只认逐字整词那两形改前也中；doc-text 之外 16 份阶段不打码（推的） |
| S7 | 那一半没有钩子或门禁判；主 agent 定义缺那一句；逐台不同的用户参数无路可走（S7-甲） |

## 我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 改什么 | 修的格 | 在别的格上 | 量过 / 推的 |
|---|---|---|---|---|
| F-S1a | ⑭ ⑮ 的标签行，在 CommonMark 认法（照抄 r2 的 `without_code_blocks`）与宽的逐行认法（反引号、波浪线都认，去掉行首 `>` 与任意缩进，往后第一行同一种字符三个以上当关块）之间遮不遮不一致，就拒，说明叫把围栏写规整 | S1-甲（C1 C3 H1 由放变拒）、S1-丙（C2 C4 C5 C6 由放变拒）；第一轮 T2 T3 照拒 | 第一轮修到的 O0–O3（反引号、波浪线、嵌套列表、引用块里的示例）照放；代价：正规的四个反引号套三个反引号写法（O5）由放变拒 | 量过，原型 `out/s1-ambiguity-prototype.txt`（没改钩子、没跑钩子自证与上一题 195 个标签） |
| F-S1b | 定义、共用约束、主 agent 定义把「代码块（照 CommonMark）」写成「闭合了的围栏块（照 CommonMark 认）」 | S1-乙 | 碰不到别的格 | 推的 |
| F-S3a | `registry_sourced_variables` 只认右边整条是登记位读法（`"$(heavy_thread_default)"`，可带 ` 2>/dev/null`；`"$(default_heavy_threads)"`；`"${SINGLEFS_THREAD_CAP}"`），且那个变量在文件里没出现在 `${变量:-…}`、`${变量:=…}` 里（`tools/s3_patch.py`） | S3-甲四形由漏变点名；S3-乙 `2>/dev/null` 一形由误红变绿 | 经函数输出那一形照旧误红；格的六份样本（var-source 两份、per-line 两份、green、red）点名数与 r2 相同；真树照绿 | 量过 `out/s3-samples.txt` patched 段、`out/s3-fixtures-and-real-tree.txt` 第 8–14 行（抽出的判法，没起格、没核样本的 want 行） |
| F-S5a | 原话找不到时，只在〔〕与提示那一行去掉改名表里的新旧名与日期之后逐字相同才取提示那一行；或要〔〕在登记那次提交的父提交里那份记录里找得到 | S5-甲 | S5-乙 的漂移加改字仍不收（另立账那一类）；后一种碰不到没进 git 的记录 | 推的 |
| F-S7a | ⑭ 加放行行「多机已判：<理由>」，有它时子 agent 照提示里逐台写的参数；或逐台线程数写进多机配置的每台可选配；主 agent 定义补「多机只照规范句、别处一概不照」那一句 | S7-甲、S7-乙后一半 | 碰不到别的格 | 推的 |

S1-甲 与 S3-甲、S5-甲 都是这一批的改法本身带进来的；按跑前条款「一格判站不住 ⇒ 改法进下一批，改完开第三轮」。S1-丙、S3-乙、S5-乙的合做两形、S6 两形、`\r\n` 那一形改前也中，照「打中也打中第一轮冻结副本 ⇒ 另立一笔账」。

## 要主 agent 跑的命令

门禁格子 agent 起不了。造树脚本与预计在 `research/prompts/m3-defs-b9-r2-opus-model/gate-samples/`（`make_trees.sh` 在给的冻结副本上造两棵，`要主agent跑.txt` 是逐条命令与预计，各在 r2、r1 上各跑一次）：
- heavy-thread-defaults（S3-甲乙）：r2 预计退 1 只点名 3 处，不点名 S3-甲那四份；r1 预计点名 8 处。
- abandoned-rounds（S5-甲）：r2 预计退 0、成功句另列「照行号提示取那一行照收的 1 行」；r1 预计退 1、点名「找不到〔m3-batch63-code-r1 用户叫停，不再打〕这段原话」。

主 agent 复核 S1-甲 可以直接在冻结副本拷贝上喂 `--check-prompt`：提示文件在复跑时由 `tools/s1_fence_feed.py` 写进 `<WORK>/s1/C1-list-fence-closed-at-column-0.txt`、`C3-details-fold-with-blank-line.txt`，`tools/s1_hosts_and_fallback.sh` 写 `<WORK>/s1h/H1-…`、`H2-…`。

## 没打中的形状

- S1：第一轮 Z1-甲那四形（波浪线块里露出三个反引号、括注那一形、列表项里没闭合的围栏、HTML 注释）r2 都拒，P1 修到（原型里 T1 照常判、T2 T3 照拒）；链接引用定义、setext 标题、表格单元（两版一致）；markdown-it 抛异常让整道闸放行（412,474 份随机拼的输入加两份万层嵌套，一次没抛，草稿量的）；50 份现有派发提示在退路下判定变了 0 份。没试：`<pre>`、`<script>` 这类到收尾标签才结束的 HTML 块；行内 HTML 跨行；提示整份放进列表项里的写法。
- S3：两处赋值都从登记位取（不点名，对）；条件赋一支字面（点名，对）；格的六份样本与真树在 r2 与 F-S3a 下点名数相同。没试：`read`、`printf -v`、`mapfile`、`for 变量 in` 这类不写 `名=` 的赋值（按第 2031 行与 SHELL_ANY_ASSIGNMENT 推，都看不见，同 S3-甲 乙那一类）；python、rust 一侧（P3 只动 shell）。
- S5：真记录 `doc-consolidate.py migrate` 一次（第 1594、1595 行所在那份记录改前改后 1606 行、同日日期删 0、合并 0，这一份上 date-once 的出路不挪行，草稿量的）；单做三种日常改动（都收，P5 修到）。没试：用户叫停之后又说接着跑、主 agent 就地改那一行而没删登记行（按第 232 行推，找不到而提示那一行仍有名字与「」，照收，改前不收；没造样本）。
- S6：九形里六形打码干净（`out/s6-mask-shapes.txt` 第 1–3、5–7 行），值切半截那一形 P6 修到。没试：值里带空白（`split_words` 按 shell 切开，切开之后短于 8 个字的那一段照设计不判）；doc-text 之外的阶段没造样本。
- S7：⑭ ⑮ 对两种写法两版都放，不分辨；豁免表两句没造新历史：第一轮 Z7-甲 已量，基准取上游 merge-base 时重钉在推上去之前一直红（「交用户」那一步在），推上去之后才不红，与 P7 写的一致。
- S2、S4 不在本腿射程，没攻。第一轮两腿与上一题三轮腿报告里有的历史没再交：`:=` 直接写线程变量（第一轮 Z2-丙）、`${1:-8}` 位置参数（上一题第三轮）、规范句下一行写现算（第一轮 Z6-乙）都没重交；本腿交的是它们与 P3、P7 合起来的新形（S3-甲 `:=` 经中间变量、S7-甲 用户给的逐台参数）。

## 这条腿自己的限度

- 「读者眼里」那一侧（S1-甲 里子 agent 读到 16、S7-甲 里子 agent 照哪一行）是推的：没派真 agent。CommonMark 那一侧是 markdown-it-py 3.0.0 的解析（本机 `python3-markdown-it`），与钩子用的同一个库。
- F-S1a 只是独立原型：照抄 r2 的认法与宽的逐行认法比，没接进钩子，没跑派发闸自证与上一题第二轮辩方 195 个标签；F-S3a 改的是格里那一个函数，只抽出格的判法跑了本腿样本、格的六份样本（只比点名数，没比 `expect` 的 want 行）与真树，没起 `--check`、没跑 code-tooling 样本档。
- S5-甲 的树把真记录截到第 1594 行，模拟那一刻；第 1595 行那句之后主 agent 会不会回头改登记行，是用户与主 agent 的动作，本腿没把它固定：放开扫的话，改成第 1595 行逐字照抄就两版都收（这才是对的登记），不改则 r2 一直收着第 1594 行。
- `\r\n` 经 stdin 入口那一形、markdown-it 随机输入、真记录 migrate 一次、现有提示有没有围栏块，都是草稿量的，没进模型目录（命令在本报告对应处）。
- 50 份现有提示是工作根下此刻的 `prompt.md`，不在仓里、会变；合计行在 `corpus/s1-corpus-run.txt`。
- 开工看负载：别的会话有 `cargo test`（checker 档、带覆盖率的那几条）在跑，没有 `qemu-system`、`vm-bench`、`e152`、`fio`；本腿只起 python 与 bash 判法，都加了 `nice -n 19`，没编译，线程上限 31 没用上。复跑经内存包装两趟各约 2.5–3 分钟挂钟，用户态 6 秒与 17 秒，余下是排队。
- 禁读：没读禁读清单里的辩方腿报告、模型目录与草稿目录，没读冻结副本根下 `multi-host.env` 的值（两份冻结副本拷贝里删掉了那一条链接，私有值样本全是编的）。共用约束与 `.claude/rules/three-way-inference.md`、`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 读的是主工作区那一份：定义「开工先读」点名、派发提示写「做法照主工作区的共用约束」；其中 `three-way-inference.md` 与本腿自己的定义 `three-way-attack.md`（作为本腿的系统提示进来）也是被判文件，与禁读清单「主工作区里被判的…规则的字面」相抵，照定义读了；报告里被判的字面一律取自冻结副本。

## 没做什么

- 没起门禁格（`--check`）、没跑任何 `--selftest`、没跑 gate.sh、没编译、没跑重型测试。
- 没判辩方那几格（P1–P7 对第一轮两腿每一条修没修到、改过头），没替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没攻 S2、S4。
- 草稿目录 `~/.local/share/singlefs/work/m3-defs-b9-r2-opus/` 里的两份冻结副本硬链接拷贝、复跑的 `rerun-*`、各组 `run-*`、试改树 `patched-s3/` 与 venv 交回前删掉；留下 `prompt.md`、`model/`（与仓里模型目录同一份）、`model-sha256.txt`。

## 交回前核

`python3 scripts/agents/cite-check.py research/prompts/m3-defs-b9-r2-opus-output.md --root /home/fy5090/code/singlefs` 末行：

```
  ✓ 核了 45 处引文、0 处裸行号，对上 45 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）
```
