# m3-defs-b9-r1 攻方腿（Opus）报告（2026-10-13）

攻击面：正文「一」表 Z1、Z2、Z4、Z5、Z6、Z7 与「三、要答的」各格专问；第一轮分工（过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱）。
记号：b9 = 这一轮冻结副本 `~/.local/share/singlefs/work/m3-defs-b9-r1/repo/`（after-b9-fixup）；r2 = M 形态之前那一版 `m3-defs-b6-r2/repo/`；r3 = N 形态之前那一版 `m3-defs-b6-r3/repo/`（r3 已带 M 形态，所以 M 形态的「之前」看 r2，N 形态与 F-Y3 的「之前」看 r3）；fix = b9 打了本腿三份试改（`patches/`）的那一份。
钩子与判法只在草稿目录的硬链接拷贝上跑（要改的文件先换成新 inode）：派发闸喂 `--check-prompt` 与 stdin 的 PreToolUse JSON；门禁格只抽格里的 python / 函数原样跑，`--check` 交主 agent。开工与收尾各核一次 b9 快照：`sha256sum -c` 非 OK 0 行，三份冻结副本里比派发提示新的文件 0 份。

## 复跑

整条经内存包装（上限 2G），分两趟（第二趟 z2 是真树，约 1–2 分钟），工作目录随意，`--work` 给工作根底下新的空目录：

```
cd /home/fy5090/code/singlefs
RERUN_PARTS="z1 z4 z5 z7" bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b9-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-defs-b9-r1/repo --work <新目录1>
RERUN_PARTS="z2" bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b9-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-defs-b9-r1/repo --work <新目录2>
```

r2、r3 默认取 `--frozen` 同级的 `m3-defs-b6-r2/repo`、`m3-defs-b6-r3/repo`（`R2_FROZEN`、`R3_FROZEN` 可改），都硬链接拷进 `--work`；末尾逐份打 `same` / `DIFF` / `新出`（路径换成 `<WORK>`、`<MODEL>`、`<FROZEN>` 再比）。腿从仓里的模型目录最后复跑一次：第一趟 12 份全 `same`，第二趟 5 份全 `same`，共 17 份 0 份 DIFF。z1 用到本机 python 的 markdown-it-py 3.0.0（CommonMark 参照实现，只用来给出「读者眼里」那一侧；F-Z1 试改也靠它）。门禁格五组要主 agent 跑：`gate-samples/要主agent跑.txt`。
模型目录交回之前整份换过一次：第一次拷进仓之后发现三份 bash 喂法用 `cut -c` 按字节截断、把 `out/` 里的中文切坏，改成按字截再整份重拷，上面两趟复跑是重拷之后的。

模型目录 `research/prompts/m3-defs-b9-r1-opus-model/` 每个文件的 sha256（48 份）：

```
0c92936ba1d7ee909877e746c68d29d4d0d8b1fbfdd2f53de630ced246d92af7  ./feeds/z2/sample/.claude/heavy-thread-default
3de0665968539362a4dd306f360ecbdd91767992dd0e58d322883c063170764c  ./feeds/z2/sample/.claude/heavy-thread-exempt
6811c6b8bdc11e994806c3dc0c3c1c2cc2254548bcebf0cb19600e07c6ff870f  ./feeds/z2/sample/scripts/experiment-apparatus/assign_default.sh
f303fced68981a6ba88bf934fb4214d67a06648c5ba05d57d274550eaaf3a77f  ./feeds/z2/sample/scripts/experiment-apparatus/capped_assign_default.sh
1833495fdbe3afb14e77a65c4329cd57a6ad01ea0308e3e0cbec2b0041b3935d  ./feeds/z2/sample/scripts/experiment-apparatus/capped_dash_default.sh
17aecfd3cb8a99c9c7c956486dcd7658f45f1d7adad33e5db5cda02b78627e3b  ./feeds/z2/sample/scripts/experiment-apparatus/if_one_line.sh
6b9862d76d1ff66fb138d588a9a4bd352b1e7dc7fa69717f1cfb1324defcc2c8  ./feeds/z2/sample/scripts/experiment-apparatus/printenv_fallback.sh
65655dfd1b6bd1f0e7423d8f0a371c29926a92651565ee9aae7b187931f5e2d3  ./feeds/z2/sample/scripts/experiment-apparatus/var_from_registry.sh
77314fde542c805dcdad14d16707944fdde4e8bfe8bc90e863a498b0d8004b73  ./feeds/z2/sample/scripts/hosts/capped.sh
daff66fe20eb64f1c0b46df188343237f5d1b7873f392680fa513615e0b8c2d9  ./feeds/z2/sample/scripts/hosts/heavy_threads.py
480b2179539b4c23137f7463660bb6b4543022f9705085f9cf357b8eb028fdb1  ./feeds/z2/sample/scripts/hosts/heavy-threads.sh
91d29fa2bef9db640dce0f3d41480ecd4fd7a7d4618dc21e056288c831c51f21  ./feeds/z5/multi-host.env
c4b6cd4ee3e52051012ef1bd426ab3130e82681ef1c9547c0a8cfba83773612e  ./feeds/z5/.private-values-hostname
3559b795bfcdb83c0d3b32a986ed2f44bf8b0bb5826718d3c465a628aebf9f33  ./feeds/z5/setup.sh
a18618bb260a2407b019f4ec5081bf79fd0d62c61de403e706ca900122cb2a98  ./gate-samples/make_trees.sh
5b50cd986f9e40880a4bf1434672bceca5de13473a2020c2e74dbb0e135379aa  ./gate-samples/要主agent跑.txt
4f3aa590416ceb31942d16275633c6ad713c3512104ce30e89271e2e7296a2d6  ./out/z1-commonmark.txt
05a1389e4711a5743cff3c244de87a75be5b2d4b0932268df7b9a5a0a87309c5  ./out/z1-fence.txt
3a01168b77f89d050fd23242aed42c027dcc495e4d04c2cd67f93966da0f594e  ./out/z1-fix.txt
6509c94c1ae01d094c5a179a73905bf21dae44b10b107d1127dc88b84a4577fd  ./out/z1-stdin.txt
7c84eb63aa0ec91e618fae58513c687c345ce7d8b4ce5918f588e49e1549c810  ./out/z2-anchors-b9.txt
a2e441dbbbe368d7512ee3ccc725d5a2a8b2fa61a839a20540306ffd1e4806a2  ./out/z2-real.txt
d585f6535a451628220bc7d4b39a3c143a284508029e312ff1c5d547fb6493c4  ./out/z2-shell-b9.txt
759ae0b599c2ee9c508d4a307439dd2f2bfd47b37fa75d572aff40a482a6b83d  ./out/z2-shell-r2.txt
d585f6535a451628220bc7d4b39a3c143a284508029e312ff1c5d547fb6493c4  ./out/z2-shell-r3.txt
e0beca1d9674ab7946f02eead9ef04a3c135a43534dfc1182b80589a0904b140  ./out/z4-drift.txt
becef57ebb9ddaa614a570c48fc8ef4762170a87700634142a787ed3dab5957d  ./out/z4-misc.txt
735bb0e4cfac10b30b7c95ed70e7a21d0786fcd06fd64eb537cfb27270cec8fb  ./out/z4-real.txt
af628cf0ac14ef2dc97cd20ad79fb6dbb5e6a87ee408a1c66464bc1593cafd29  ./out/z5-cells-b9.txt
9b1aef011dc37245a488a875936f7a3fc67e165bcad574331bf3e68b14b9f257  ./out/z5-cells-fix.txt
54ba0bf0c34e452521b04ebd87f1972d2c53ceee245896c770343b6665caf2ab  ./out/z5-cells-r2.txt
af628cf0ac14ef2dc97cd20ad79fb6dbb5e6a87ee408a1c66464bc1593cafd29  ./out/z5-cells-r3.txt
dace0ec5ef4828ca16a2777253b5bd0f01e24101809a5855c1831cb07528f442  ./out/z7.txt
703f1518c9097c5f38e546de84eb2588ff53da55cd40d5eb5cfb6b4039e1eab1  ./patches/doc-text.sh.diff
8e42101168b769bacd2bbdd866f9e61591dd7fd2730bd6237bb819d957e8898b  ./patches/lib_abandoned_rounds.py.diff
8e0222a01417a6d6d6ab8e4ca5a81ca7281d229bce7615ecf4c55b122e1c4c99  ./patches/runner-dispatch-guard.sh.diff
486d81468c02e682da287b4082eb64b8093e5a4053521274734de1be21e72c91  ./tools/make_fix_tree.sh
5ed7108f6ff1f65bfd0ea1ca613b850f947155fefa7aa211e1f84893e921330c  ./tools/rerun.sh
6588722de25b9d51e23997b9e1a8b403529e18ac692ba1cfe7d307e9cb349530  ./tools/z1_commonmark_view.py
0cbcd0d3289aee5209c7d7f6d6ac02ed3214468344a9510bc3a3bf9443addfa4  ./tools/z1_fence_feed.py
0f45b830a4a21745fe44b3183535daf8af2c3e03afc5965086528faa8aebb4db  ./tools/z1_stdin_feed.py
d3d5c3b64b901dc18f38f7e5f904220db7cd9ef39ef486e87cc4934bedb55046  ./tools/z2_htd_cell.py
8c5a5bef8fac9e5a37cc766151f26e7790899c02dbb488adbba6ce630df7431c  ./tools/z2_real_tree.sh
d7ff35e4a0f3aca0b39c0dccf29f293a9ed4f737ec60e1fd03b9643e0065af48  ./tools/z4_misc.sh
e47dc600b9b89bed0eda07e7a062862ef8d343e62e722a325f026e8a9f925062  ./tools/z4_real_dispatch.sh
813b3f2399fc4e5473c6491b1f47484753ffcba4e3187cf0005df46bacd76e2c  ./tools/z4_source_drift.sh
5e0ceeb0458f7214df073a7130cd8336798016f5f6689296e99d88eea177507a  ./tools/z5_cells.py
9f74b05344009c22b1e417e7b65aa856f8526bcad428456b04490885a930e96d  ./tools/z7_exempt_repin.sh
```

## 各格判定一览

「量过」= 在拷贝上真跑了钩子入口或抽出的判法，原样输出在模型目录 `out/`；「推的」= 按代码推，没跑。「之前那一版」一列是同一段历史喂 r2（M 形态）或 r3（N 形态、F-Y3）的结果。

| 格 | 判定 | 打中的那一步（一句） | 之前那一版 | 哪一版带进来 | 量过 / 推的 |
|---|---|---|---|---|---|
| Z1-甲 | 打中（定义说拒而钩子不拒） | 提示里有一个波浪线代码块、里面露出一行三个反引号（或列表项里没闭合的反引号围栏、或 HTML 注释里一行三个反引号），钩子把那一行当围栏开头，一直遮到下一个真代码块的开头：真的「线程上限：16」不判，代码块里的示例 31 被当声明，放 | r2 拒、r3 放 | M1（代码块不判）带进来，N1 没修到这几形 | 量过 `out/z1-fence.txt` 第 2–5 行、`out/z1-stdin.txt`、读者一侧 `out/z1-commonmark.txt` 第 14–17 行 |
| Z1-乙 | 打中（钩子拒而定义没写） | 嵌套列表里缩进 4 格的反引号代码块里的示例被判、拒 | r3 放、r2 拒 | N1（至多缩进 3 格，不认容器） | 量过 `out/z1-fence.txt` 第 8 行 |
| Z1-丙 | 打中（内部矛盾） | 照钩子缺行时的拒绝说明「要别的数…单起一行已判」只补已判行，放行；共用约束说只有已判行时子 agent 不起重活，实现员停下交回 | 钩子三版同判 | N3（共用约束加了停下那一句，钩子出路文字没跟着改） | 钩子一侧量过 `out/z1-fence.txt` 第 17–18 行；子 agent 一侧推的 |
| Z2-甲 | 打中（照规则做、结果错：没审过的一处被旧理由放行） | 已有排除行的文件里新加一处 available_parallelism，或把被钉的那一行改几个字读进新的线程变量，格照绿；89 行排除表里 17 行的锚串命中 2–5 条被判行 | r2 同绿 | 改前也中（M2 没修全） | 量过 `out/z2-real.txt` 第 2、4 行（对照第 3、5 行红）、`out/z2-anchors-b9.txt` |
| Z2-乙 | 打中（照格的出路做仍红） | 先从登记位取进变量、再 `\|\| X=$default_threads`，格点名「写死的线程数默认」 | r2 不点名 | M2 | 量过 `out/z2-shell-b9.txt` 第 6 行、`out/z2-shell-r2.txt` |
| Z2-丙 | 打中（从未被看过） | capped.sh 设的变量写 `: "${RUST_TEST_THREADS:=8}"`，不点名（同一变量写 `:-8` 点名） | 三版同 | 改前也中 | 量过 `out/z2-shell-*.txt` |
| Z4-甲 | 打中（照规则做、结果错） | 已收的叫停轮 m3-batch63-code-r1：出处那份调度记录追加一行补记、抄了登记用的原话，原话命中 2 次，不收；⑥ 拒派实现员改它快照里的文件，门禁 abandoned-rounds 红 | r3 收、⑥ 放 | N2 | 量过 `out/z4-real.txt` 第 2、4 行 |
| Z4-乙 | 打中（照规则做、结果错） | 登记之后照 `.claude/rules/path-moves.md` 跑术语改名 `sweep-term.py --apply`（records/ 照换，research/prompts/ 整个豁免），原话找不到，不收 | r3 收 | N2 | 量过 `out/z4-drift.txt` 第 2、5、8 行 |
| Z4-丙 | 打中（门禁格之间一收一拒） | 碰过那份记录之后照 date-once 格的出路 `doc-consolidate.py migrate` 删掉正文里重复标题的日期，原话找不到，不收 | r3 收 | N2 | 量过 `out/z4-drift.txt` 第 3、6、9 行 |
| Z4-丁 | 打中（内部矛盾，文字） | 派发闸与门禁 abandoned-rounds 格两份文件头写「不按 -r<数字> 形」，库按 -r 形拒：出处那一行提到仓里没材料的 ghost-r2，被拒 | r2 收 | M4（库改了，两份文件头没跟） | 量过 `out/z4-misc.txt` 第 1–3 行 |
| Z5-甲 | 打中（输出漏值） | date-once、term-renames 两格的红行原样打出含（编的）私有值的路径 | r2、r3 同 | 改前也中（M5「各格」只做了三格） | 量过 `out/z5-cells-b9.txt` 第 12、17 行 |
| Z5-乙 | 打中（输出漏值） | clock-times 的片段截在私有值中间，半截不是整词、打码不认，原样打出 17 个字 | r2、r3 同 | 改前也中（M5 打码只认整词） | 量过 `out/z5-cells-b9.txt` 第 3–4 行、`out/z5-cells-r2.txt` 第 3–4 行 |
| Z6-甲 | 打中（定义说拒而钩子不拒） | Z1-甲 同一形落在「多机：」：真「多机：每台线程按此刻空闲现算…」被遮，代码块里的规范句被当声明，放 | r2 拒、r3 放 | M1、M6（代码块不判） | 量过 `out/z1-fence.txt` 第 13 行 |
| Z6-乙 | 专问喂了（钩子不拒；子 agent 一侧没有对应的一句） | 规范句之后下一行写「每台线程按此刻空闲现算」，⑭ 三版都放，派执行员也放 | 三版同 | 改前也中 | 钩子量过 `out/z1-fence.txt` 第 14–15 行；子 agent 照不照推的 |
| Z7-甲 | 打中（文字与判法不一致） | 基准里还没有这张豁免表时，改被钉的入口并重钉 sha256，exempt-check 绿；stage-owners 写「按只缩不涨判红」 | r2 同绿 | 改前也中（M7 在这个窗口里不起作用） | 量过 `out/z7.txt` 第 2–3 行 |
| Z7-乙 | 打中（「新入口不进表」可绕） | 被钉的 E171 入口只有 20 行、`exec` 没被钉的编排脚本、配置从参数给：下一次修订改编排脚本、给新配置，同一个入口照起，exempt-check（带基准）与长活声明都绿 | r2 同 | 改前也中 | 量过 `out/z7.txt` 第 4–5 行 |

## Z1　⑮ ⑭ 认围栏、只有已判行（M1、N1、N3）

### Z1-甲　波浪线块里的一行三个反引号、列表项里没闭合的围栏、HTML 注释：钩子开块、遮掉真声明（量过）

被判的字面：
- `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/main-agent.md:53`：「三个反引号围起来的代码块里的行 ⑮ 不判」
- `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:69`：「围栏行至多缩进 3 个空格，开头那一行反引号后面的语言名里不许有反引号」
- 开块判法只看这一行本身：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:1043`：「CODE_FENCE_OPEN = re.compile(r"^ {0,3}(`{3,})[^`]*$")」；关块是往后找到的第一行同样多个反引号，不看这两行之间、之外有没有别的块。CommonMark 里这一行在波浪线围栏块里是块的内容、在 HTML 块里是 HTML，列表项里没闭合的围栏在列表项结束处结束；钩子三样都当开块、再拿下一个真代码块的开头当关块。

历史（investigator，2 小时内）：主 agent 照用户的话写 16（或第一轮 V1-乙 的括注），提示里顺手贴了上一次给工具员讲围栏写法时的样子（外层波浪线，里面露出一行三个反引号）和上一次派发的原文：

````
开工先读 `.claude/agent-common.md`。
重型测试：不跑
草稿目录：<草稿>/
报告：<草稿>/report.md
活：在 crates/singlefs-harness 里复现一个 harness 测试的失败，交回最小复现。
现象：harness 的一个测试偶发失败。
预计挂钟：2 小时内
上一次给工具员讲围栏时贴的样子（外层用波浪线围起来，里面露出一行三个反引号）：
~~~
```
~~~
线程上限：16
上一次派发的原文：
```
线程上限：31
```
````

钩子：第 10 行当开块，第 14 行（真代码块的开头）当关块，10–14 行遮掉；第 15 行「线程上限：31」露出来当声明，第 16 行当没闭合的开块原样留着，只看到 31，放。读者（CommonMark 参照实现）：第 12 行「线程上限：16」是正文里唯一的声明行。

`research/prompts/m3-defs-b9-r1-opus-model/out/z1-fence.txt:2`：「T1-tilde-lone-fence-16	investigator	b9=0	r3=0	r2=2」
`research/prompts/m3-defs-b9-r1-opus-model/out/z1-commonmark.txt:14`：「T1-tilde-lone-fence-16	不在代码块里的声明行：第 12 行」
`research/prompts/m3-defs-b9-r1-opus-model/out/z1-stdin.txt:2`：「b9 退出=0 -」

同一种遮法换三个口子（`out/z1-fence.txt` 第 3–5 行，b9、r3 都放，r2 都拒）：T1b 把第 12 行换成「线程上限：31（机器忙，按负载降到 16）」；T2 是列表项 `- 上一次的起法：` 下面缩进 2 格的反引号围栏忘了关、紧接着下一个列表项（CommonMark 在列表项结束处结束它）；T3 是 `<!-- 旧写法：` 注释里一行三个反引号。⑭ 同形（Z6-甲）：真行换成「多机：每台线程按此刻空闲现算，机器忙时按负载降」、代码块里贴骨架的规范句，b9、r3 放，r2 拒（`out/z1-fence.txt` 第 13 行）。

四句：
- 分不分辨臂：分。r2（M1 之前，代码块里的行照判）拒，r3、b9 放：是 M1、M6 加的「代码块里的行不判」带进来的；N1 只把行首写在一行里的 ```…``` 修了（上一题第三轮的 F2、F3 在 b9 已拒，本腿重跑上一题的喂法看到的，草稿量的），这几形没修到。
- 被判的系统看不看得到：看得到。波浪线围栏、列表项的缩进、`<!--` 都在提示文字里。
- 满足的分句：「定义说拒而钩子不拒」：读者眼里那一行是 16 或带按负载降的括注，定义要它等于登记值、不接括注；违反用户定案「默认 31、固定不看忙闲」。
- 改法：N1（只认缩进与语言名里的反引号）在这几格上照中；本腿 F-Z1（见「我提的改法」）在这几格上拒。

### Z1-乙　嵌套列表里的反引号代码块：示例行照判（量过）

```
线程上限：31
- 上一次：
  - 派发原文：
    ```
    线程上限：31（本机）
    ```
```

（外层这个围栏是本报告的，提示里只有里面那几行。）围栏缩进 4 格，N1 的「至多缩进 3 格」不认它是围栏，示例行「线程上限：31（本机）」被判、拒。CommonMark 里它是第二层列表项里的代码块。`research/prompts/m3-defs-b9-r1-opus-model/out/z1-fence.txt:8`：「O2-nested-list-fence-example	investigator	b9=2	r3=0	r2=2」。r3（任意缩进都开块）放，N1 带进来的；r2 代码块照判，拒。

四句：分辨（r3 放、b9 拒）；看得到（缩进与列表标记都在文字里）；分句「钩子拒而定义没写」（定义写三个反引号围起来的代码块里的行不判，这就是一个）；改法：N1 本身就是来源，F-Z1 放。另三形三版都拒、不分辨：波浪线代码块里的示例（定义只写三个反引号，照字面不算错，与文件头「认围栏照 CommonMark」不一致）、引用块里的反引号代码块、到文末没闭合的围栏（文件头写明照判）（`out/z1-fence.txt` 第 7、9、10 行）。

### Z1-丙　缺行的拒绝说明与「只有已判行不起重活」相抵（钩子量过，子 agent 推的）

派实现员没写「线程上限：」一行，钩子的出路（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:1160`）：「这件活确要别的数（用户给了参数、内存装不下），或确实不起重活，提示里单起一行」。主 agent 照这一句只补「线程上限已判：内存只装得下 16 份编译，这一次用 16」，三版都放（`out/z1-fence.txt` 第 17–18 行）。子 agent 一侧：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agent-common.md:61`：「的，不起重活，要起就停下交回，报告里写明要主 agent 补一行」；同一行还说「提示里别处写的线程数一概不照」，所以已判行里的 16 也不照。实现员要编译，照共用约束只能停下交回，这一次派发白跑；主 agent 定义第 53 行（「顶替了的派发子 agent 不起重活，要起就停下交回」）与共用约束一致，只有钩子的出路文字把「要别的数」与「不起重活」并成一句、只给已判行一条路。

四句：钩子一侧三版同判，不分辨；子 agent 停下那一句是 N3 加的，矛盾随 N3 出现（之前子 agent 拿不到 N，结局没写）；看得到（钩子判这一次时知道缺的是哪一行）；分句「照规则做、结果错」（照钩子出路做，用户给的 16 一条重活都没起）；改法：N3 碰不到钩子文字，F-Z1c（推的）：缺行的出路里「要别的数」那一半写成「写「线程上限：N」一行再加已判行」，只把「确实不起重活」留给单独的已判行。

### 专问

- CommonMark 之外的写法各怎么判（量过，`out/z1-fence.txt` 第 2–10 行）：波浪线围栏不认（里面的示例照判；里面露出的三个反引号反被当开块，Z1-甲）；缩进 4 格的代码块不认，里面的示例照判（与定义「三个反引号」字面一致）；列表里的围栏只在缩进不到 4 格时认（嵌套一层就不认，Z1-乙）；列表项里没闭合、靠容器结束的围栏，钩子拿后面的关块配对（Z1-甲 T2）；没闭合的围栏后面照判（文件头写明）。
- 只有已判行时「不起重活」谁判得到：钩子判不到（它只看提示，已判行一律放），定义写在共用约束第 61 行与主 agent 定义第 53 行，靠子 agent 自己守；Z1-丙 是钩子出路把主 agent 引到这条路上。

## Z2　heavy-thread-defaults 按被判行匹配、`|| X=$var`、`printenv`（M2）

### Z2-甲　锚串落在被判行里就放：锚串不唯一，新加的一处与改过的一行都被旧理由放行（量过，真树拷贝）

被判的字面：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/code-tooling.sh:1885`：「排除表按被判行匹配，不按文件并组：锚串要落在读它的那一行原文里才放那一行」；判法是子串：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/code-tooling.sh:2351`：「if any(anchor in line for anchor in anchors):」。锚串在那份文件里命中几条被判行不核（对照：术语改名的按行豁免要求 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/scripts/kb/sweep-term.py:26`：「它要在那份文件里恰好命中一行」）。

排除表 89 行逐行数（抽出格的判法在 b9 整棵上跑，只多打每行锚串命中几条被判行）：89 行都至少命中 1 条（专问「有没有锚串已经落不到被判行上」：没有，格绿、「每行都命中」）；17 行命中 2–5 条，其中 16 行的锚串就是 `available_parallelism`、`"nproc"`、`os.cpu_count() or 4` 这类同文件里随手就会再写一遍的串。

`research/prompts/m3-defs-b9-r1-opus-model/out/z2-anchors-b9.txt:66`：「ANCHOR	81	crates/singlefs-harness/tests/random_histories.rs	available_parallelism	命中被判行 2 条	[172, 1975]」
`research/prompts/m3-defs-b9-r1-opus-model/out/z2-anchors-b9.txt:50`：「ANCHOR	63	scripts/mutation/crates-mutation-rows.py	os.cpu_count() or 4	命中被判行 2 条	[1854, 2142]」

历史一（专问「同一锚串在一份文件里出现几次」）：实现员在 `random_histories.rs` 里照现成写法再加一个取样线程默认值 `fn sampling_workers_by_default() -> usize { std::thread::available_parallelism().map_or(1, usize::from) }`（没有上限、不经登记位）。排除表那一行（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/heavy-thread-exempt:81`：「crates/singlefs-harness/tests/random_histories.rs:available_parallelism」，理由「代码批：随机历史快档与取样活动的线程」）的锚串落在新的这一行里，格绿；同一个函数写进新文件红。

`research/prompts/m3-defs-b9-r1-opus-model/out/z2-real.txt:2`：「b9	insert-fn-exempted-file	  ✓ 每台默认重活线程数只经登记位取：登记值 31」
`research/prompts/m3-defs-b9-r1-opus-model/out/z2-real.txt:3`：「b9	insert-fn-new-file	  ✗ 1 处自己定并行度默认值的写法没经登记位取」

历史二（专问「行改了几个字会怎样」）：把被钉的那一行 `processor_count = os.cpu_count() or 4`（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/heavy-thread-exempt:63`：「scripts/mutation/crates-mutation-rows.py:os.cpu_count() or 4」）改成先读一个 capped.sh 不设、也不回退的新变量 `int(os.environ.get("GATE_MUTATION_TOTAL_WORKERS") or 0) or os.cpu_count() or 4`。「读自己的线程变量」这一种按 M2 并到它第一次被读的那一行上，那一行的锚串还在，格绿；同一行写进新文件红，点名三种。

`research/prompts/m3-defs-b9-r1-opus-model/out/z2-real.txt:4`：「b9	edit-exempted-line	  ✓ 每台默认重活线程数只经登记位取」

r2 两段历史同绿（`out/z2-real.txt` 第 7、9 行）。

四句：不分辨（r2 按文件并组也绿）；看得到（格自己算出了新的那一处是被判行，只是拿旧锚串放了）；分句「照规则做、结果错」：用户定案「豁免表不登记别人没审过的」，新加的一处、改过的一行由一行旧理由放行；改法：M2 碰到的是第二轮 X2-乙 那一种（新读的线程变量落在新的一行），锚串本身不唯一这一半碰不到。改前也中，另立账。

### Z2-乙　照格的出路从登记位取进变量再赋，被点名成钉数（量过）

样本 `feeds/z2/sample/scripts/experiment-apparatus/var_from_registry.sh`：`source …/heavy-threads.sh`、`default_threads="$(heavy_thread_default)"`、`[[ -n "${SAMPLE_SWEEP_THREADS:-}" ]] || SAMPLE_SWEEP_THREADS=$default_threads`。格的出路（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/code-tooling.sh:2389`）：「shell 先 source scripts/hosts/heavy-threads.sh 再 heavy_thread_default 取」，这份脚本就是这么取的；M2 把 `$变量` 一律认成钉数（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/code-tooling.sh:1877`：「即默认值先放进变量、先算好再赋」），不看变量是不是从登记位来的。b9、r3 点名「写死的线程数默认（|| / && 赋值）」（`out/z2-shell-b9.txt` 第 6 行），r2 不点名。

四句：分辨（r2 绿、b9 红，M2 带进来）；看得到（同一份文件上一行就是 `$(heavy_thread_default)`）；分句：文件头写到了 `$变量` 算钉数，不是「钩子拒而定义没写」，是「照出路做仍红」——出路剩下的只有把命令替换内联、或进排除表（排除表的理由要「确实不是重活」，这一处是重活）；改法：M2 自己是来源，F-Z2b（推的）：`$变量` 只在同一份文件里那个变量不是由 `heavy_thread_default`、`default_heavy_threads`、`$SINGLEFS_THREAD_CAP` 赋值时才算钉数。

### Z2-丙　`${X:=8}` 不点名（量过，从未被看过）

`: "${RUST_TEST_THREADS:=8}"`（capped.sh 清单里的变量，没经 capped.sh 起时取 8）三版都不点名；同一变量写 `${RUST_TEST_THREADS:-8}` 三版都点名（`out/z2-shell-*.txt`）。写死默认那一形只认 `:-`：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/code-tooling.sh:2015`：「thread_default_form("写死的线程数默认", r"\$\{" + THREAD_VARIABLE + r":-[0-9]+\}"」。单行 `if [[ -z … ]]; then X=8; fi` 与 `$(printenv X || echo 8)` 只在变量不在 capped.sh 清单里时经「读自己的线程变量」那一路点名，不点名成钉数。改前也中，另立账；改法 F-Z2c（推的）：那一形加 `:=`、`-`、`=` 三种参数展开。

## Z4　叫停登记的出处按原话找、恰好一次才收（M4、N2）

被判的字面：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/rules/three-way-inference.md:121`：「拿〔〕里的原话在那份文件里找，恰好命中一次才收」；库里命中两次以上即拒（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/lib_abandoned_rounds.py:217`：「if len(hits) > 1:」），找不到即拒。出处那份文件是 `records/` 下的调度记录：照常追加、照术语改名换字、照 date-once 删日期的那一类文件；登记表在 `research/prompts/`，术语改名整个不扫（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/term-rename-exempt:11`：「research/prompts/  # 冻结证据」）。

### Z4-甲　出处那份记录追加一行补记、抄了登记用的原话：命中两次，已叫停的轮重新开着（量过，真树）

这一版里已收的那一行：m3-batch63-code-r1，出处 `records/2026-10-01-删除与并发事务.md` 第 1595 行，原话 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/records/2026-10-01-删除与并发事务.md:1595`：「去向：m3-batch63-code-r1 两条腿当场停掉（没交回、不写判决）」。历史：主 agent 在同一份调度记录末尾记一句「补记：m3-batch63-code-r1 的叫停登记已收进 research/prompts/abandoned-rounds.tsv，出处抄的是第 1595 行「…」」（记下登记了什么是调度记录的本分，抄出处最省事）。之后叫停表的判法（门禁 abandoned-rounds 格与 ⑥ 同一份）不收，⑥ 拒派实现员改那一轮快照里的 `crates/singlefs-checker-tier/src/verdict_store.rs`：

`research/prompts/m3-defs-b9-r1-opus-model/out/z4-real.txt:2`：「这段原话命中 2 次（第 1595、1607 行），只许恰好一次」
`research/prompts/m3-defs-b9-r1-opus-model/out/z4-real.txt:2`：「⑥ 派实现员退出=2	✗ 派 implementation-writer 要改的文件落在还没判完的三方轮 m3-batch63-code-r1 的开工快照里」
`research/prompts/m3-defs-b9-r1-opus-model/out/z4-real.txt:4`：「r3	追加之后	叫停表：m3-batch63-code-r1	收	⑥ 派实现员退出=0」

拒绝说明给的出路「多抄几个字」对抄全了原话的补记无效（补记里就是同一段字）；要改的是登记表那一行，而红指到的是登记表，不是新加的那一行补记。合成小树上同形（`out/z4-drift.txt` 第 1 行），r3 两种写法都收（第 4、7 行）。

### Z4-乙　登记之后照规则改术语，records/ 照换、登记表不换：原话找不到（量过）

历史：2026-10-12 用户叫停 hx-term-r1，记录那一行写「去向：hx-term-r1 两条腿停掉（第一条流的崩溃枚举没跑完）」，登记表照抄这段原话；2026-10-13 用户定「第一条流 → 新池新建文件流」（`.claude/kb/term-renames.md` 那一节，这一轮冻结副本的表里还没有这一行，样本照主工作区那一次补一行），主 agent 照 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/rules/path-moves.md@改一个全仓术语：正文之外还有五处会红`：「历史类文件同样换名」跑 `sweep-term.py --apply`：records/ 那一行换了 2 处，登记表在豁免目录里没动。

`research/prompts/m3-defs-b9-r1-opus-model/out/z4-drift.txt:2`：「b9	excerpt	term-sweep	改动前：hx-term-r1	收 	改动后：hx-term-r1	不收：第 1 行的出处 records/log.md 里找不到」
`research/prompts/m3-defs-b9-r1-opus-model/out/z4-drift.txt:5`：「r3	line	term-sweep	改动前：hx-term-r1	收 	改动后：hx-term-r1	收」

那条规则说正文之外「五处会红」，没有这一处；`--apply` 还把用户「」里的原话一起换了（样本里「第一条流那一轮先停掉」成了「新池新建文件流那一轮先停掉」），另记一笔，不在这一题。

### Z4-丙　date-once 格的出路删掉出处那一行的日期：两道门禁格一收一拒（量过）

历史：记录那一行写「- hx-date-r1 停掉（用户 2026-10-12 说「先停掉那一轮不写判决」）」，登记表抄「hx-date-r1 停掉（用户 2026-10-12 说「先停掉那一轮不写判决」）」。同一批里碰过这份记录，date-once 格红，照它的出路（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/doc-text.sh:275`：「python3 scripts/kb/doc-consolidate.py migrate <那一份> --out <临时文件> 看它删了哪些、并了哪些，照着改」）删掉正文里重复标题的日期，那一行成了「（用户说「…」）」，abandoned-rounds 格随之红：

`research/prompts/m3-defs-b9-r1-opus-model/out/z4-drift.txt:3`：「b9	excerpt	date-once	改动前：hx-date-r1	收 	改动后：hx-date-r1	不收：第 1 行的出处 records/log.md 里找不到」
`research/prompts/m3-defs-b9-r1-opus-model/out/z4-drift.txt:6`：「r3	line	date-once	改动前：hx-date-r1	收 	改动后：hx-date-r1	收」

Z4-甲乙丙四句：
- 分不分辨臂：分。r3（按行号读）三段历史都收，b9 都不收：N2 带进来的。上一题第三轮 Y1-丙（前面插行、行号漂移）N2 修到了（`out/z4-misc.txt` 第 9–11 行：b9 收、r3 与 r2 不收）；N2 换来的是另一组：原话命中几次、原话还在不在。
- 看得到：看得到。判的那一刻文件就在手边；行号提示那一行仍是登记时那一句（甲丙行号没动，乙行号没动、字改了），库没拿行号提示去分辨。
- 分句：「照规则做、结果错」：照调度记录的写法追加、照 path-moves.md 改术语、照 date-once 的出路改日期，结局是用户已叫停的轮重新算开着，⑥ 挡写回员与实现员，门禁 abandoned-rounds 红。
- 改法：N2 是来源；F-Z4（量过、被攻过零轮）在这三格上都收，Y1-丙 照收，插行又改字照拒（`out/z4-drift.txt` 第 10–12 行、`out/z4-real.txt` 第 6 行、`out/z4-misc.txt` 第 12、16 行）。

### Z4-丁　两份文件头仍写「不按 -r<数字> 形」，库按 -r 形拒（量过）

`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:39`：「不按 -r<数字> 形」；`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/doc-process-records.sh:103`：「不按 -r<数字> 形」；库与规则按 M4 写成两样都拒：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/lib_abandoned_rounds.py:19`：「形如 <名>-r<数字> 的写法不看仓里有没有它的材料」。出处那一行写「改并进 ghost-r2 那一题（ghost-r2 还没开轮、仓里没有材料）」，b9、r3 拒，r2 收（`out/z4-misc.txt` 第 1–3 行）。四句：分辨（r2 收，M4 带进来）；看得到；分句「钩子拒而（钩子自己的文件头）写的是收」；改法：两行文件头照库改，文字一处。

### 专问

- 「恰好一次」在记录被追加、改写之后稳不稳：不稳，上面三条各是一种日常改动；按行号读的旧判法在这三种上都稳，在插行上不稳；两头都只看一样，F-Z4 两样都看。
- 旧行名单与表：名单四个名字，表里只剩 smalldata-ttime-r1 一行；三个没有行的名字照样免出处，新登记一行 c196-r1 不写出处，b9 收（`out/z4-misc.txt` 第 17 行，r2、r3 同）。名单留着靠的是样本：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/doc-process-records.sh:180`：「登记撂下那一行是库里旧行名单里的 c364-r3、没有出处，不要求」。名单与表之间没有检查绑定（上一题判决 Y5 第 10 条已立账），轮名重用这一路够不着（轮名带题号，推的），只记一行。

## Z5　doc-text 打码（M5、F-Y3）

正文 Z5 一行说「doc-text 各格打印路径都经 private-values 那一份打码」；格的文件头写的只有三格：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/doc-text.sh:60`：「三格打路径时调同一份，不各抄一份」。样本 `feeds/z5/`（假配置，值都是编的）：从第二台 tar 过来的说明文件、以第二台主机名起名的记录、从第二台进度日志抄来的一句。

### Z5-甲　date-once、term-renames 两格原样打路径（量过）

date-once 红行直接 echo 路径（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/doc-text.sh:467`：「echo "  ✗ $path：正文里重复所属日期」）；term-renames 格把 sweep-term 的输出原样放出来（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/doc-text.sh:421`：「if python3 "$SWEEP_TERM" --check "$ROOT"; then」），sweep-term 逐份打相对路径（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/scripts/kb/sweep-term.py:429`：「print("      %-72s %d 处" % (relative, count))」）。同一棵样本上 private-values 自己的红行把这两条路径打成「〔文件名含私有值〕」：

`research/prompts/m3-defs-b9-r1-opus-model/out/z5-cells-b9.txt:12`：「✗ records/2026-10-13-demo-peer-host-42-对拍.md：正文里重复所属日期 1 处」
`research/prompts/m3-defs-b9-r1-opus-model/out/z5-cells-b9.txt:17`：「research/results/from-peer/srv/demo-private/peer-checkout/notes.md       1 处」

r2、r3 同（`out/z5-cells-r2.txt`、`out/z5-cells-r3.txt` 第 12、17 行）。四句：不分辨（改前也中）；看得到（同一道里打码函数就在手边，private-values 格已经对同一批路径打了码）；分句「照规则做、结果错」：提交前照规矩跑 doc-text、照「贴原样输出」把红行贴进进度记录，第二台的主机名与仓路径就进了仓（用户定案「私有信息不进仓」）；改法：M5、F-Y3 都只动三格与 private-values 本身，碰不到这两格。

### Z5-乙　clock-times 的片段截在私有值中间，半截原样打出（量过）

片段取命中位置前 20 个字、后 10 个字（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/doc-text.sh:368`：「findings.append((relative_path, line_number, pattern_name, line[max(0, position - 20):anchor + 10].strip()))」），再按整词打码；截剩的半截不是整词，打码不认：

`research/prompts/m3-defs-b9-r1-opus-model/out/z5-cells-b9.txt:3`：「ISO 时间戳  …ate/peer-checkout 上 2026-10-13T10:31:05 收尾。…」
`research/prompts/m3-defs-b9-r1-opus-model/out/z5-cells-b9.txt:4`：「〔私有值〕 上 2026-10-13T11:02:00 开跑」

假仓路径 31 个字，打出后 17 个字。对照行（整个主机名落在片段里）b9 打码、r2 原样（`/home/fy5090/code/singlefs/research/prompts/m3-defs-b9-r1-opus-model/out/z5-cells-r2.txt:4`：「demo-peer-host-42 上 2026-10-13T11:02:00 开跑」）。四句：截半截这一形不分辨（三版都打出 17 个字），整词那一形 M5 修到了；看得到；分句：格自己的输出约定被破（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/doc-text.sh:55`：「输出：只报文件、行号与键名」）；改法：M5 只认整词，碰不到；F-Z5（量过、被攻过零轮）把片段两头挪到私有值边上再打码，两行都成「〔私有值〕」（`out/z5-cells-fix.txt` 第 3–4 行）。

专问「链接、读不了的文件之外还有没有不经打码的通道」：有，就是上面两条；没再找到第三条（试过的见「没打中的形状」）。

## Z6　⑭ 多机一行逐字等于规范句、说明另起一行（M6）

- Z6-甲：见 Z1-甲 最后一段（波浪线块遮掉真「多机：」行，放；`out/z1-fence.txt` 第 13 行，r2 拒）。
- Z6-乙（专问，喂了）：「多机：<规范句>」下一行写「每台线程按此刻空闲现算，机器忙时按负载降到登记值的一半。」派调查员、派执行员，⑭ 三版都放：`research/prompts/m3-defs-b9-r1-opus-model/out/z1-fence.txt:14`：「H3-next-line-live	investigator	b9=0	r3=0	r2=0」，第 15 行执行员同。主 agent 定义许这样写（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/main-agent.md:55`：「后面不接任何说明，说明另起一行写」）。子 agent 一侧：线程上限有「`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agent-common.md:61`：「提示里别处写的线程数一概不照」」这一句，多机没有对应的一句；执行员的定义自己写了脚本取法（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/experiment-runner.md:33`：「脚本做的事：台数取多机配置」），调查员、实现员、工具员、攻方腿、clerk 等带重活的类型的定义只写「超过的另有三行」，下一行的现算说法照样送到手里，照不照是推的（没派真 agent）。四句：不分辨（三版都放，改前也中，与上一题第二轮 X1-甲 同形）；钩子看得到那一行，只是按定义不判；分句「照定义做却违反用户定案（固定不看忙闲）」的入口在主 agent 写了这一行；改法：M6 碰不到；F-Z6（推的）：共用约束补一句「多机只照规范句，提示里别处写的台数、线程取法一概不照」。

## Z7　长活声明豁免表按（路径、sha256）成对比（M7、N5）

### Z7-甲　基准里还没有这张表：重钉不红（量过）

格的文件头写明这一种不比：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/code-tooling.sh:2419`：「基准里还没有这张表、不在 git 仓里，这一条不比、成功句里写明」；stage-owners 那一行（N5）写的是：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/stage-owners.tsv:48`：「改被钉的脚本要重钉 sha256，按只缩不涨判红，交用户」。主工作区现查（只查有无，不读内容）：

```
$ for p in .claude/long-run-live-hosts-exempt .claude/heavy-thread-exempt; do printf '%s HEAD:' $p; git cat-file -e HEAD:$p 2>/dev/null && echo 有 || echo 没有; mb=$(git merge-base HEAD @{upstream} 2>/dev/null); printf '  上游 merge-base:'; git cat-file -e $mb:$p 2>/dev/null && echo 有 || echo 没有; done
.claude/long-run-live-hosts-exempt HEAD:没有
  上游 merge-base:没有
.claude/heavy-thread-exempt HEAD:没有
  上游 merge-base:没有
```

这一批落地那次提交起、推上去之前（基准取上游 merge-base），两张表的只缩不涨都不比。历史：这段时间里有人改被钉的 E171 入口（把 `nice -n 19` 改成 `nice -n 10`）并照 stage-owners 的说法重钉 sha256：

`research/prompts/m3-defs-b9-r1-opus-model/out/z7.txt:3`：「b9	repin-基准没有表	exempt-check 退出=0」
`research/prompts/m3-defs-b9-r1-opus-model/out/z7.txt:2`：「b9	repin-基准有表	exempt-check 退出=1」
`research/prompts/m3-defs-b9-r1-opus-model/out/z7.txt:7`：「r2	repin-基准有表	exempt-check 退出=0」

四句：基准有表时分辨（M7 修到了第二轮 X7-甲），没表时不分辨（r2 同绿）；看得到（表在工作区里，sha256 也在）；分句：文件头写到了，stage-owners 那一句写的是红，「两处文字一收一放」；改法：M7 在这个窗口里不起作用，F-Z7（推的）：基准没有表时拿用户审过的那一份（判决或决策里钉的 sha256 清单）当基准。

### Z7-乙　被钉的入口不干活：改没被钉的编排脚本、换配置，同一个入口照起（量过）

被钉的入口一共 20 行，配置从参数给、正文只 exec 编排脚本：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/run_segment_one_revision_11_multi_host.sh:15`：「CONFIG=${1:-$RESEARCH/revision_11_placeholders.json}」，同一份第 20 行：「exec nice -n 19 python3 -B "$RESEARCH/segment_one_units_revision_11.py" run --config "$CONFIG" --repo "$REPO"」。历史：E171 下一次修订不写新入口（主 agent 定义第 55 行「新脚本不往表里加」），改 `segment_one_units_revision_11.py`、放一份 `revision_13_placeholders.json`，照旧 `run_segment_one_revision_11_multi_host.sh revision_13_placeholders.json`：

`research/prompts/m3-defs-b9-r1-opus-model/out/z7.txt:4`：「b9	reuse-entry-基准有表	exempt-check 退出=0」
`research/prompts/m3-defs-b9-r1-opus-model/out/z7.txt:5`：「b9	reuse-entry-被起脚本	script 退出=0」

四句：不分辨（r2 同，第 9–10 行）；看不到：sha256 只钉入口一份，按现算干活的是编排脚本，判别子不在被钉的那份里；分句「照规则做、结果错」：新修订照样按现算起、表没涨；改法：M7 碰不到，F-Z7b（推的）：表里一行钉入口与它 exec / source 的整组文件，或 detached-run 起被豁免的入口时只认登记时的那份配置。

专问「（路径、sha256）成对比在基准那一版没有这张表时做什么」：不比，成功句写「基准那一版里还没有这张表」（`out/z7.txt` 第 3 行），即 Z7-甲。

## 正文「三」的三问（逐格）

| 格 | 过度外推与越位 | 内部矛盾与循环依赖 | 脆不脆（换一个最自然的写法） |
|---|---|---|---|
| Z1 | 文件头说「认围栏照 CommonMark」，实际只照了缩进与语言名两条：波浪线、容器、HTML 块都不认（Z1-甲乙） | 缺行的拒绝说明与共用约束「只有已判行不起重活」相抵（Z1-丙） | 脆：给工具员讲围栏时贴一段波浪线代码块就遮掉真声明 |
| Z2 | 「按被判行匹配」写成了逐行的保护，锚串是子串、不要求唯一，17 行一行罩几条（Z2-甲） | 出路叫从登记位取，从登记位取进变量再赋又被认成钉数（Z2-乙） | 中：同文件里再写一处 available_parallelism 即被吞 |
| Z4 | 「行号只当提示」之后行号一点不用，原话一改一重复就整行作废 | 三道规矩（追加记录、术语改名、date-once 出路）各自照做，都让 abandoned-rounds 与 ⑥ 翻回「开着」；两份文件头与库对 -r 形一收一拒（Z4-甲乙丙丁） | 最脆：三种日常改动各一下就打穿，都是 N2 带进来的 |
| Z5 | 「各格」只做了三格（Z5-甲） | 无 | 中：路径里带私有值的记录碰上 date-once 或旧术语就漏 |
| Z6 | 无 | 线程上限有「别处写的一概不照」，多机没有（Z6-乙） | 中：下一行写现算三版都放（改前也中） |
| Z7 | 「成对比」在第一次落地的窗口里不比（Z7-甲）；钉入口管不到干活的编排脚本（Z7-乙） | stage-owners 写判红、文件头写不比（Z7-甲） | 中：没表的窗口里重钉即过 |

七格里最脆弱的是 Z4（N2）：三种都不是攻击性的写法，是照别的规矩做的日常改动；第二是 Z1（M1、N1）：四种写法遮掉真声明，N1 的认法只修了上一题报的那一形。Z3 不在本腿射程，没攻。

## 我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 改什么 | 修的格 | 在别的打中格上 | 量过 / 推的 |
|---|---|---|---|---|
| F-Z1 | `without_code_blocks` 改用 CommonMark 参照实现（markdown-it-py 的 commonmark 预设）认围栏：反引号与波浪线、列表与引用块里的都认，只遮闭合了的围栏，没闭合的照判；载不进那个库时一行都不遮（退回 M1 之前整份照判）（`patches/runner-dispatch-guard.sh.diff`） | Z1-甲（T1 T1b T2 T3 由放变拒）、Z6-甲（H2 由放变拒）、Z1-乙与波浪线、引用块的示例（O1 O2 O3 由拒变放）；上一题 Y1 的 F2 F3 H2 H3 照拒、F4 照放 | 碰不到 Z1-丙、Z6-乙；第二台没有 markdown-it-py 时退回整份照判（代码块里的示例照拒）；定义「三个反引号围起来的」要改成「代码块（照 CommonMark）」 | 量过 `out/z1-fix.txt`；上一题 Y1 喂法草稿量过；派发闸自证没跑、第二轮辩方 195 个标签没重跑 |
| F-Z1c | 缺行时的出路把「要别的数」写成「写「线程上限：N」再加已判行」，单独的已判行只留给「确实不起重活」 | Z1-丙 | 碰不到别的格 | 推的 |
| F-Z2a | 排除表每行的锚串在那份文件里只许命中恰好一条被判行（与术语改名按行豁免同一判据），多了判红 | Z2-甲 | 现有 17 行要改锚串（量过的数，`out/z2-anchors-b9.txt`）；改被钉那一行时锚串还在照样放（Z2-甲历史二），这一半要再加「锚串以外的部分也不许变」或钉整行 | 推的 |
| F-Z2b | `\|\| X=$var` 只在同一份文件里 var 不是由 `heavy_thread_default`、`default_heavy_threads`、`$SINGLEFS_THREAD_CAP` 赋值时算钉数 | Z2-乙 | 碰不到 Z2-甲丙 | 推的 |
| F-Z2c | 写死默认那一形加 `:=`、`-`、`=` 三种参数展开 | Z2-丙 | 碰不到别的 | 推的 |
| F-Z4 | 原话命中几处而行号提示正落在其中一处：取那一行；原话找不到而行号提示那一行还有这一轮的名字：取那一行，往下照判名字、别的轮名与「」原话（`patches/lib_abandoned_rounds.py.diff`） | Z4-甲乙丙；上一题 Y1-丙 照收 | 碰不到 Z4-丁（文件头文字）；弱点：漂移之后行号提示那一行恰好是另一句带这一轮名字与「」的话（例：开轮那一行带了用户原话）会被当出处收（推的，没造） | 量过 `out/z4-drift.txt` 第 10–12 行、`out/z4-real.txt` 第 6 行、`out/z4-misc.txt` 第 12、16 行 |
| F-Z5 | clock-times / prime-marks / session-names 的片段两头落在私有值里就挪到值的边上，整值进片段再打码（`patches/doc-text.sh.diff`） | Z5-乙 | 碰不到 Z5-甲 | 量过 `out/z5-cells-fix.txt` 第 3–4 行；doc-text 样本档没跑 |
| F-Z5b | date-once 的红行与 term-renames 转出的 sweep-term 输出，路径都经 `shown_path` | Z5-甲 | 碰不到别的 | 推的 |
| F-Z6 | 共用约束补一句：多机只照规范句，提示里别处写的台数、线程取法一概不照 | Z6-乙 | 碰不到 Z6-甲 | 推的 |
| F-Z7 | 基准里没有豁免表时，拿用户审过的那一份 sha256 清单当基准；表里一行钉入口与它 exec / source 的整组文件 | Z7-甲、Z7-乙 | 碰不到别的 | 推的 |

## 要主 agent 跑的命令

门禁格子 agent 起不了。造树脚本与预计在 `research/prompts/m3-defs-b9-r1-opus-model/gate-samples/`（`make_trees.sh` 造七棵，`要主agent跑.txt` 是逐条命令与预计）：abandoned-rounds（Z4-甲 真树追加一行，预计红、点名命中 2 次）；heavy-thread-defaults 三棵（Z2-甲两棵预计绿、Z2 样本预计红 5 处且不点名 `capped_assign_default.sh`）；doc-text 四格一棵（Z5，预计 clock-times 片段带半截假值、date-once 与 term-renames 原样打假路径）；long-run-header-exempt 两棵 git 仓（Z7-甲，基准有表预计红、基准没表预计绿）。预计都是抽出格里的判法在本腿树上跑出来的，没起过格本身；抽出跑与格本身的差别：改动范围与基准由格自己经 `scripts/gate/changed-paths.sh` 取。

## 没打中的形状

- Z1 ⑮ ⑭：控制组 T0（正文写 16，三版都拒）、H0 H1；引用块里转述用户的「多机：」一行（三版都拒：它是第一个「多机：」行，被当声明判不合，`out/z1-fence.txt` 第 16 行）；没闭合的围栏到文末（三版都拒，文件头写明照判）；缩进 4 格的代码块示例（三版都拒，与定义「三个反引号」字面一致）。上一题第三轮 Y1 的 F2 F3 H2 H3 在 b9 都拒（N1 修到，草稿量的）。没试：制表符缩进的围栏、超过三个波浪线的围栏里套波浪线、`\r` 换行。
- Z2：printenv 读、`$(printenv X || echo 8)` 点名（M2 修到）；新文件里的新读、新文件里改过的那一行点名（对照组）；89 行排除表没有一行的锚串落空。没试 rust 一侧的 `unwrap_or` 换写法、python 的 `os.environ.setdefault`。
- Z4：前面插行漂移（上一题 Y1-丙）b9 收，修到；没材料的 -r 形当别的轮名（M4 本意，库与规则一致，只文件头不一致）；路径搬迁（path-moves.md 要求 research/prompts/ 也改路径，登记表与记录同改，推的不会断，没喂）。
- Z5：私有值整个落进片段时 b9 打码（对照行）；private-values 自己的红行、文件名红行、符号链接、读不了的文件都打码（F-Y3 修到，本腿样本没造读不了的那一形，上一题第三轮量过）。没找到第三条通道：`sweep-term --apply` 的输出不在门禁里；date-once 的 doc-consolidate 输出只打行数。
- Z6：同一标签两行拒（M6 本意）；规范句后面接「；…」拒（控制组未单列，上一题量过）。
- Z7：基准有表时重钉判红（M7 修到）；路径改名（同一脚本 git mv、sha256 不变）按「基准里没有的路径」判涨，r2 同，没单列。

## 这条腿自己的限度

- 子 agent 拿到 Z1-甲、Z1-丙、Z6-乙 那几份提示会照哪一行起重活，是推的：没派真 agent。钩子判定量过（`--check-prompt` 与 stdin 入口两条路）；「读者眼里」那一侧是 markdown-it-py 的解析，不是 LLM 的读法。
- Z4 三段历史在合成小树上量过、甲另在真树（这一版已收的 m3-batch63-code-r1 与它真的快照）上量过；乙丙两种日常改动在真记录里多常见没量。乙用的改名行是照主工作区 2026-10-13 那一次补进样本的，这一轮冻结副本的表里还没有。
- Z2-甲 的 17 行是这一版的数；新加一处被吞的概率没量。
- Z7-甲 的窗口靠主工作区 `git cat-file -e` 的有无判断（没读内容）；这一批推上去之后窗口就关了。
- 门禁格都只抽格里的 python 或 shell 函数在小树上跑，没起 `--check`。三份试改只在本腿的模型与上一题第三轮的 Y1 喂法上量过；没跑三份被改脚本登记的自证、doc-text 样本档、第二轮辩方 195 个标签。
- 开工看负载：别的会话有 `cargo test` / `cargo clippy` / 变异在跑，没有 `qemu-system`、`vm-bench`、`e152`、`fio`；本腿只起 python 与 bash 判法，都加了 `nice -n 19`，没编译，线程上限 31 没用上。复跑经内存包装时排过一次队（9 秒）。
- 没读主工作区里被判的文件与禁读清单（正推腿报告、模型目录、草稿目录，冻结副本根下 `multi-host.env` 的值：三份冻结副本拷贝都不带这个文件，私有值样本全是编的）。共用约束读的是主工作区那一份：定义「开工先读 `.claude/agent-common.md`」与禁读清单「主工作区里被判的定义…（只读冻结副本里的）」相抵，开工时照定义读了，读完才对上禁读清单（这一条交主 agent 定以后怎么写）；规则读的是冻结副本那一份，之后核被判字面一律用冻结副本。用到主工作区的工具：`run-with-memory-cap.sh`、`cite-check.py`、`three-way-rerun-template.sh`、上一题第三轮攻方模型的 `y1_fence_feed.py`（只读、只在草稿里跑）。

## 没做什么

- 没起门禁格（`--check`）、没跑任何 `--selftest`、没跑 gate.sh、没编译、没跑重型测试。
- 没判正推那几格（Z1、Z3、Z4、Z6、Z7 的文字一致性逐句核），没替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没攻 Z3。
- 草稿目录 `~/.local/share/singlefs/work/m3-defs-b9-r1-opus/` 里的三份冻结副本拷贝、试改树与各趟 `--work` 交回前删掉；留下 `prompt.md`、`model/`（与仓里模型目录同一份）、`model-sha256.txt` 与 `work/` 下的喂法输出。

## 交回前核

`python3 scripts/agents/cite-check.py research/prompts/m3-defs-b9-r1-opus-output.md --root /home/fy5090/code/singlefs` 末行：

```
  ✓ 核了 57 处引文，对上 57 处，没判 0 处（逐处列在上面）
```
