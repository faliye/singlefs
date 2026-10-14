# m3-defs-b6-r2 攻方腿（Opus）报告（2026-10-13）

攻击面：正文「一」表 X1、X2、X3、X5、X6、X7 与「三、要答的」各格专问；第二轮分工（第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方）。
对象：这一轮冻结副本 `~/.local/share/singlefs/work/m3-defs-b6-r2/repo/`（下称 r2）；每条也喂第一轮冻结副本 `~/.local/share/singlefs/work/m3-defs-b6-r1/repo/`（下称 r1）；X2 另喂零轮之前那一份 `~/.local/share/singlefs/work/m3-defs-b3-r2/repo/`（下称 b3）。都只在草稿目录的拷贝上跑；钩子喂 JSON 或 `--check-prompt`，门禁格只抽格里的 python 原样跑，`--check` 交主 agent。

## 复跑

整条经内存包装（上限 2G；不起 cargo、不起重型测试、不跑门禁阶段）：

```
R1_FROZEN=$HOME/.local/share/singlefs/work/m3-defs-b6-r1/repo B3_FROZEN=$HOME/.local/share/singlefs/work/m3-defs-b3-r2/repo \
  bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b6-r2-opus-model/tools/rerun.sh \
  --repo <仓根> --frozen $HOME/.local/share/singlefs/work/m3-defs-b6-r2/repo --work <工作根底下新的空目录>
```

`--work` 要在工作根（`~/.local/share/singlefs/work`）底下：X3 仓副本那一支靠它。末尾逐份打 `same` / `DIFF`（路径换成 `<WORK>`、`<MODEL>`、`<REPO>` 再比）。腿从仓里的模型目录最后复跑一次：26 份全 `same`、0 份 `DIFF`，挂钟 95 秒（`time` 量的）。`x5-prefix-probe.txt` 读冻结副本根下真的 `multi-host.env`，只打个数。门禁格四条要主 agent 跑：`gate-samples/要主agent跑.txt`。

模型目录 `research/prompts/m3-defs-b6-r2-opus-model/` 每个文件的 sha256（67 份）：

```
c89903b36783daeff83f5d010838ecc97e491f31deacd34af8a18de4a449af9a  ./feeds/x1/base-investigator.txt
08460735fd6ae2a628c84a69358df74bb83918481178f55f9491ccaa77f1d62c  ./feeds/x1/base-over2h.txt
de13dae8ab42e876b86d8c1d9ad7e1a21e9084ed631f9b272a4295c8763f6bf0  ./feeds/x1/variants.tsv
308e00027d5366e5d515a8ec8081c265748c5db6747e5943e57d03db68c11936  ./feeds/x2/new_device_control.rs
0c92936ba1d7ee909877e746c68d29d4d0d8b1fbfdd2f53de630ced246d92af7  ./feeds/x2/sample/.claude/heavy-thread-default
fb2422805995c4b8b2927777692ce67d6d08ef05a62d67f3cfe664b2f3be6eba  ./feeds/x2/sample/.claude/heavy-thread-exempt
156acb6415b0b9a663b4a93d67b1b4e4af5277021309c2dd81b31b6c95475c47  ./feeds/x2/sample/crates/demo/src/end_tests.rs
65a524a7e80339254c58f95c7f953e2290c5fdb84e6ae80467363f7c5dd7eca2  ./feeds/x2/sample/crates/demo/src/mid_tests.rs
7a7acbc5cb684cbcab3f92ec51feefc459a147c5accb524c3f9f9f95266652d8  ./feeds/x2/sample/scripts/experiments/absorbed.py
02a3b6c070dab452d331036970b4209ce1a7b291648bba299fe43e73e5c5d952  ./feeds/x2/sample/scripts/experiments/fstring_name.py
037dfa9388792441d72e0370925edd018110a14d7fcc0ab03498bde6776cd8c0  ./feeds/x2/sample/scripts/experiments/heavy_named.py
80db92d4302d311d042f53313ae93ede1a921101434c1aa3f9ec83b1aad39a2f  ./feeds/x2/sample/scripts/experiments/lanes_read.py
587fe2d4a0d6ca4d740a7c2603c0b3aa27750167d1da166575e9d3ce36e0ccef  ./feeds/x2/sample/scripts/experiments/not_absorbed.py
e45743d9e4f5e617ec42fc0254b4c6d661e118ab09a1b0f0d97e46d4461525c0  ./feeds/x2/sample/scripts/experiments/plain_read.py
77314fde542c805dcdad14d16707944fdde4e8bfe8bc90e863a498b0d8004b73  ./feeds/x2/sample/scripts/hosts/capped.sh
daff66fe20eb64f1c0b46df188343237f5d1b7873f392680fa513615e0b8c2d9  ./feeds/x2/sample/scripts/hosts/heavy_threads.py
98fbb7058ba7ef6ed1da1ee4658e8f0b89c981afedb42448c85d0c1af9a7eb78  ./feeds/x2/sample/scripts/hosts/or_arith_default.sh
99dde0afbd7cbba032ac522b9e45e6d8f505cdcf29cbe9b07cd7da132a19fb8a  ./feeds/x2/sample/scripts/hosts/or_literal_default.sh
c1c6a8906544f706c429bfa4fe4e2540527331da6a2ac1a763a0e2a6ac46d0e5  ./feeds/x2/sample/scripts/hosts/or_var_default.sh
a0daed5b08813f2c3a70acbd198bd08b3e2f99c12bfe91ec18174e6b55908499  ./feeds/x5/pv-mirror-siblings/multi-host.env
a619bc5ee4432588bc22a81b435f4d01b14e3f90361f87075dfb0169bf3ddc24  ./feeds/x5/pv-mirror-siblings/.private-values-hostname
3798142227a0ec1f957a34fae6d304671afb40636948b2c1ad78c93a60dfedca  ./feeds/x5/pv-mirror-siblings/setup.sh
95098bf9e393fa18b3a50c19c222e02a73cbd6b9f220f3b02278d7a05fdf534d  ./feeds/x6/header-scripts.txt
81b75b72a0a5b1244b60f34aa5a89715bc48062750cced9f8873cdbae1310651  ./feeds/x6/scripts/h0_canonical.sh
fc980a47ef9657278b2d63f6d20ad464a6838e9ade5b3bee0154ec7a2df285e6  ./feeds/x6/scripts/h1_note_live.sh
074e317419ee98400c21165c7f4771a6e4eeee9c7cf702c87d14a77e9b3bd462  ./feeds/x6/scripts/h2_wrapped.sh
8ef0744e31f7e5b86293fb7eeeaf60cda7dee909c8a3d482b36be878f4c457a8  ./feeds/x6/scripts/h3_note_eid.sh
6c376d74c3298a5068f660a2bb7d18a889540018df9d20bde3f72b17b035a0fa  ./feeds/x6/variants.tsv
ebcd3363e489b164dd1d0ff8b5fae25e9d70b83dbc5c427ddf38a4428726de5a  ./feeds/x7/r1-format/setup.sh
661b7d02ba2a1789e5827afef6ac85f76b79bee06afe9cc9c7fe48abb60151ec  ./feeds/x7/repin/setup.sh
dbdfc56e8742f6a6d44cd414c8811928655dafe2f411f7c63522f20b7819e2b6  ./gate-samples/要主agent跑.txt
67190bf07880b7a446fbb78e4e896ad77364a26e348e73ec61f18e5e703b8d7b  ./out/x1-r1-investigator.txt
b5cafb44a036aaabefdbe16d74389098cc6e209bf34475a5788c32079d39c5a9  ./out/x1-r2-investigator.txt
6bc4d43d3ad1bbe624ff0475b6b1489fca6e7f29de62eadffbe28db9b05a89ed  ./out/x2-absorb-control-r1.txt
24b48d80c5b2e068447273c9a7545a9b173c513ac1b15b83c998c8e57d08f608  ./out/x2-absorb-control-r2.txt
5b9200e3ac6d44aeef3ea984668a9d7742ee1cd4d2e3f2f7998291657dcfeae4  ./out/x2-absorb-r1.txt
5b9200e3ac6d44aeef3ea984668a9d7742ee1cd4d2e3f2f7998291657dcfeae4  ./out/x2-absorb-r2.txt
4d95581fa9e54cc50a9589b3b5f2356557c24a0c8d7c965cbfdc1afa9c1133ad  ./out/x2-noexempt-r1.head
a758ef162b45bd26d5dc256143b933269905e6f8e871eb40da8b09306a8e4184  ./out/x2-noexempt-r1.keys
4d95581fa9e54cc50a9589b3b5f2356557c24a0c8d7c965cbfdc1afa9c1133ad  ./out/x2-noexempt-r2.head
b21b87466440f62ab447f0358a925aa247a1a3c09bf919d0057f8fec9244defa  ./out/x2-noexempt-r2.keys
a5b79bdc8ac81788c44f2286399278af189260bb8afe7b6bcac89ae5e89dca4e  ./out/x2-sample-b3.txt
029362802058fd1f49e88aa8812674dcf827a8179e983f58db357abce83d15f1  ./out/x2-sample-r1.txt
fec9cfba6ef17c37ae28084314a61e2be9a91b97bb344a971ed1ab7aacc0de83  ./out/x2-sample-r2.txt
dc75f852f68d7a5efe29ae45213e8d9a857355b2fcd700d1e4889e284e8115b5  ./out/x3-copy-r1.txt
7392e3653a8634780d4fdaa2cddadb2f7ad52cf5310bbd1861d2c638029acde5  ./out/x3-copy-r2.txt
2eb7a9d64ecd1b941fa7bfd174b8009b4e3e82a445f9b9c9c39f3c5b0b84ec84  ./out/x3-env-r1.txt
12205434fac15b9362b10e8bbe7badfebff064c6009c747528249ec726b53d36  ./out/x3-env-r2.txt
558e6813455b8df59f1e7da2e97e30b73d699ab848de52a6e5bd1f944b21c362  ./out/x5-prefix-probe.txt
62bfda60ffdbad59550c1a79c51f9e48851a55753af0ebecd29392baffc30dd2  ./out/x5-siblings.txt
b69a24f7440c9361a2c538a8553aef0e42f17146e12a962cb2a7d422af7edf4c  ./out/x6-detached-check.txt
b3c745862f285fa35b64d66b5f6f8713c76acc556a7a37b25c4584af4e68aefb  ./out/x6-headers.txt
b84752b766da75ea7a37adf67bfd2753ae5f20075358d3d0450a7759959ffa00  ./out/x6-hosts-r1.txt
f58371bce282e34cac6b97c129060beef533be6dba69b4e1a13a679a2f3692f2  ./out/x6-hosts-r2.txt
2145d62ae0e956174b46c2550290160397fa2e50f8baeedbeaf66c0ec78c8b3e  ./out/x6-script-headers.txt
55a81b8b395305a6dcb26e166b49a22e555c5613793012c37bd0d89515d712a4  ./out/x7-r1-format.txt
4f0afc5368936aa7e8069c1a87b13b1d10a4ae4b7cc7b8a4cadc6575360e25e8  ./out/x7-repin.txt
9d777857462c41af66187de589400ff9bbb3364a4ddac68705501a5075382ca3  ./tools/pv_prefix_probe.py
449acffa87d2a3c7c0555de32137d757a1bd826aca4d179023e7c0c648a36c27  ./tools/rerun.sh
94646c94102cdfa4483a888399b582880fb7a18a17ab7571832b9f08398e2849  ./tools/run_htd_cell.py
341c53331a57ac0d4bef47400e351453022c60505aa30faa0b20d0a48d573859  ./tools/run_pv_cell.py
38224126dab7c704ee9112753fc70f14d2c07c5d5db43839546d0ff75933c0ef  ./tools/run_scan_form.py
8748347eef3d97e8bd805eea16802876745f720cd1778c229978fb61b330769f  ./tools/x1_feed.sh
582c5ca285aa8a09134502b6056ab5d2887df9943354ff7e3eea1f7f6e25a3e4  ./tools/x2_absorb_insert.sh
320061e6ac1112f681b918e8e7370ff67bb298db42e934ebd51fc936d2db897e  ./tools/x3_copy_feed.sh
729a903d32a0ef43f08fceb19869d932da3d792c5e31fb7ecbc3086ea1d8ab13  ./tools/x3_env_feed.sh
afdbb351a46a464f26356b4d962b9ed24445c096afdc35902367d834fd15f1d4  ./tools/x6_headers.py
```

## 各格判定一览

「量过」= 在冻结副本拷贝上真跑了钩子入口或抽出的判法函数，原样输出在模型目录 `out/`；「推的」= 按代码推，没跑。「r1」一列是同一段历史喂第一轮冻结副本的结果。

| 格 | 判定 | 打中的那一步（一句） | r1 | 这一批带进来的？ | 量过 / 推的 |
|---|---|---|---|---|---|
| X1-甲 | 打中（定义说拒而钩子不拒；L1 碰不到第一轮 V1-乙） | 第一轮 V1-乙 那几句（按负载降、对折、第二台 16、`capped.sh 16`）挪到新开的「线程上限说明：」一行，⑮ 照放 | 同样放 | 否（钩子判定两版一样）；定义把这一行写成正当出口是这一批的 | 量过 `out/x1-*-investigator.txt` H1–H4 |
| X1-乙 | 打中（钩子拒而定义没写，多拒） | 提示里代码块的喂法示例行「线程上限：31（本机）」也被当成那一行拒 | 放 | 是 | 量过 H9 |
| X2-甲 | 打中（L2 碰不到同一形的变体） | `\|\| X=$fallback`、`\|\| X=$((4 * 2))`（没设就给默认值）不点名；L2 只认右边是字面数字 | 同样漏 | 否（零轮带进来，L2 没修全；b3 点名） | 量过 `out/x2-sample-*.txt` |
| X2-乙 | 打中（豁免行吞新读法） | 照第一轮 V3 的历史在 `random_histories.rs` 里照现成写法加一个 capped.sh 不设的线程变量：r2 判绿；同一行写在新文件里才红 | 绿（新文件里也绿） | 否；L2 修了新文件那一半，已豁免的 45 份文件里照吞 | 量过 `out/x2-absorb-*.txt`（真树拷贝） |
| X2-丙 | 打中（从未被看过） | 语句里有含 `heavy_threads` 的变量名就算「回退到登记值」；f 串拼的变量名；`_LANES` / `_PARALLELISM`；文件中段的 `#[cfg(test)] mod tests { }` 之后 | 同样漏 | 否（b3 也漏） | 量过 |
| X3-甲 | 打中（定义说拒而钩子不拒） | `PKG=singlefs-checker-tier env -S 'cargo test -p ${PKG}'`、`D=… env -S '-C ${D} cargo test'`：GNU env 在 -S 里展开 `${…}`，库照原文留着，放行 | 同样放 | 否（r1 根本不拆 -S）；L3 的拆法没修到这一形 | 量过 `out/x3-env-*.txt` E1–E1d |
| X3-乙 | 打中（从未被看过） | `export PKG=singlefs-checker-tier; cargo test -p "$PKG"`：-p 的值写在变量里，放行，文件头「看不见的」一段没列这一种 | 同样放 | 否 | 量过 E1e、E1f |
| X3-丙 | 打中（定义之间矛盾） | L3 的拒绝说明让主 agent 直接带 `user-request` 前缀跑改过的自证；`implementation-workflow.md` 写 user-request「先弹窗问用户」，同一道闸给主 agent 的拒绝说明也写「提交之外要跑先弹窗问用户」 | r1 说明不带前缀 | 是 | 量过 `out/x3-copy-*.txt` |
| X5-甲 | 打中（别的通道漏值） | 同一道 doc-text 里 prime-marks、session-names 两格绿着也把二进制文件的路径原样打出，clock-times 红行原样打路径；第一轮 V7 的 tar 镜像历史照样把值打进输出 | 同样打 | 否 | 量过 `out/x5-siblings.txt`（假值样本） |
| X5-乙 | 没打中 | 整串比对的误报：真配置 8 个长词没有一个是本机 `$HOME`、工作根、仓根的子串；private-values 自己的打码在样本上不漏 | — | — | 量过 `out/x5-prefix-probe.txt` |
| X6-甲 | 打中（定义说拒而钩子不拒，这一批带进来的） | 规范句后接「；按此刻空闲现算每台槽数与线程」「；机器忙时按负载降」「；除以此刻同时在跑的重活数」「；第二台十六线程」，派发闸 ⑭ 与 `detached-run.sh long-run-check` 都放；新脚本文件头老实写现算也过，不用进豁免表 | 拒（N1、N4、N5、N13、h1） | 是 | 量过 `out/x6-*.txt` |
| X6-乙 | 打中（钩子拒、正当写法被挡） | 说明里写 `E171 修订 11`、`layer0`、`sha256`、半角分号、文件头规范句折成两行，都拒 | 放 | 是 | 量过 N7–N9、N11、h2、h3 |
| X7-甲 | 打中（只缩不涨没比 sha256） | 改了豁免表里脚本的正文、把表里 sha256 换成新的（行数、路径不变），exempt-check 判绿「与基准那一版比没涨」 | r1 不重钉也绿 | 否（r1 更宽）；L7 收窄了，重钉这一条路没关 | 量过 `out/x7-*.txt` |
| X7-乙 | 没打中 | 执行位变：sha256 不变、照放（不算错）；CRLF 行尾：sha256 变、判红（多拒）；E171 下一次修订：照 X6-甲 写规范句加现算说明就过 | — | — | 量过 |

三问的答：

- 第一轮的修补本身：L1、L6 把第一轮按词表拒的内容整段挪进「不判」或「只判数字」的说明里（X1-甲、X6-甲），第一轮判对的「拒现算」那一半在 ⑭ 上丢了（X6-甲 r1 拒、r2 放）；L3 的出路与另一份规则相抵（X3-丙）；L7 的 sha256 钉没有绑到只缩不涨上（X7-甲）。
- 产生结论的方法：L2、L3 的修补都只对着第一轮攻方样本的字面形验（`|| X=8`、`env -S 'cargo test -p singlefs-checker-tier'`），各自往旁边一步（右边是变量、串里是 `${VAR}`）就漏（X2-甲、X3-甲）；L2 新文件判红、同一行写进已有豁免行的文件判绿（X2-乙）。
- 从未被看过的地方：同一道 doc-text 的兄弟格打路径（X5-甲）；回退认法按子串认 `heavy_threads`（X2-丙）；-p 的值写在变量里（X3-乙）。

## X1　⑮ 线程上限只认整数、说明另起一行

被判的字面：/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/main-agent.md:53：「机器忙时也固定这个数，不看忙闲」；同一行：「用户给了参数、或内存装不下要用别的数的，另起一行」；钩子文件头 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/runner-dispatch-guard.sh:63`：「要说明的另起一行写」；子 agent 一侧 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/agent-common.md:61`：「只管 capped.sh default 与脚本内部的取法，子 agent 照派发提示的 N」。

喂法：`tools/x1_feed.sh <仓> investigator feeds/x1/variants.tsv <输出>`，每格 = `feeds/x1/base-investigator.txt` 加一段，经派发闸 `--check-prompt`（与 stdin 入口同一份 judge_dispatch）判。r2 与 r1 各一遍，原样 `out/x1-r2-investigator.txt`、`out/x1-r1-investigator.txt`。

### X1-甲　第一轮 V1-乙 的内容挪进「线程上限说明：」一行，照放（量过）

历史：机器忙，主 agent 想让子 agent 少吃线程，照 main-agent.md 第 53 行「N 只写一个十进制整数……要加说明另起一行」写：

```
线程上限：31
线程上限说明：机器忙时按负载降
```

（H2 写「第二台用 16」，H3「小机器对折」，H4「起重活时一律写 bash scripts/hosts/capped.sh 16」。）四格 r2、r1 都放，原样：

`research/prompts/m3-defs-b6-r2-opus-model/out/x1-r2-investigator.txt:8`：「H1-note-load rc=0 ✓ 派发闸照这份提示放行（investigator，查了 9 行提示）」
`research/prompts/m3-defs-b6-r2-opus-model/out/x1-r2-investigator.txt:11`：「H4-note-capped16 rc=0 ✓ 派发闸照这份提示放行（investigator，查了 9 行提示）」
`research/prompts/m3-defs-b6-r2-opus-model/out/x1-r1-investigator.txt:8`：「H1-note-load rc=0 ✓ 派发闸照这份提示放行（investigator，查了 9 行提示）」

对照：同样的话写进括注「线程上限：31（机器忙时按负载降）」r2 拒（钩子自检的格「线程:括注里写按负载降拒」），r1 放（第一轮 V1-乙）。L1 只关了括注那一个口，同一句话往下挪一行照放；「线程上限说明：」这一行是定义新写进去的正当出口（九份定义「输入」一节与 main-agent.md 第 53 行同一句），钩子按设计不判它。

四句：分辨臂——不分：r2、r1 钩子判定一样，差别只在定义把这一行写成了出口；看得到——看得到，说明行原文就在提示里；分句——H2、H4 是「定义说拒而钩子不拒」：main-agent.md 第 53 行要别的数就写已判行，H4 不写已判行、把 16 写进说明行；H1、H3 的后果要看子 agent 照不照说明行改 N——共用约束第 61 行写子 agent 照「线程上限：N」，第 85 行写说明行「那一行不判」，没写子 agent 不照它做（推的，没派真 agent）；改法——L1 在 H1–H4 上不起作用（第一轮 V1-乙 那几格照中）。按跑前条款「打中也打中第一轮冻结副本」另立账；交出去时写明「L1 对第一轮 V1-乙 不起作用，只是换了一行」。

### X1-乙　代码块里的示例行也当「线程上限：」一行判（量过）

历史：主 agent 派一条腿去攻 ⑮（就是这一题），在提示的喂法示例里放一个代码块，代码块里一行「线程上限：31（本机）」。r2 拒，r1 放：

`research/prompts/m3-defs-b6-r2-opus-model/out/x1-r2-investigator.txt:16`：「H9-codeblock rc=2 ✗ 派 investigator 的提示写了」
`research/prompts/m3-defs-b6-r2-opus-model/out/x1-r1-investigator.txt:16`：「H9-codeblock rc=0 ✓ 派发闸照这份提示放行（investigator，查了 12 行提示）」

四句：分辨臂——分（这一批带进来的）；看得到——看得到（代码围栏就在提示里），判法按行首判、不看围栏；分句——「钩子拒而定义没写」：定义只说「「线程上限：」一行」，没说代码块里的示例也算；结局是多拒，改写示例（行首加别的字）就过，不违反用户定案。记一笔，代价一次被拒。

### 专问

- 正当的说明有没有地方写：有，「线程上限说明：」一行，钩子不判（P1、H1–H4 放）。这一行同时是 X1-甲 的出口。
- 「线程上限说明：」会不会被当成第二个「线程上限」：钩子不会（判法要「线程上限」后紧跟冒号，P1 放、H7「只写说明行不写线程上限」按缺行拒）。「线程上限（第二台）：16」「线程上限 第二台：16」同样不被认成那一行（H5、H6 两版都放），与上一题攻过的「第二台线程上限：16」同形，不再交。
- 子 agent 一侧：共用约束只说 N 照「线程上限：N」，没说说明行里的数与做法子 agent 不照（推的）。

## X2　heavy-thread-defaults 退回宽判法

喂法：`tools/run_htd_cell.py <门禁根> <被判树>` 从门禁根的 `code-tooling.sh` 抽出这一格的 python 与 `CODE_WITHOUT_TEXT_LIBRARY`，在被判树里原样跑（基准 none）。

### 专问：第一轮之前那批误判回来了几处、排除表兜不兜得住、只缩不涨

真树（r2、r1 各拷一份 scripts/、crates/、.claude/gate.d、.claude/hooks 与登记位，排除表换成空的）上两版都报「✗ 94 处」（`out/x2-noexempt-*.head`）。按「路径 + 被判行原文」比（`out/x2-noexempt-*.keys`），r2 比 r1 多两行、少两行：多的是 `scripts/agents/dispatch-prompt.py` 的 `if "THREAD_LIMIT_REQUIRED_AGENTS" in long_run_table.group(2):`（字符串比较，宽判法带回来的那一类，回来 1 处）与 `scripts/experiments/replay.sh` 的 `[[ "$REPLAY_JOBS" -lt 1 ]] && REPLAY_JOBS=1`（L2 新认的 `&& X=N` 认到了夹下限，误判 1 处）；少的两行是 `runner-dispatch-guard.sh` 里改了原文的两行。名字元组、字典实参这两类在真树上没有新出现。排除表 r1→r2 删两行陈旧锚串、加两行（就是上面两处），行数不变，真树上带排除表判绿（草稿里那一次，原样没进模型目录：「✓ 每台默认重活线程数只经登记位取：登记值 31；扫了 832 份……可疑写法 94 处全在排除表里（表 66 行，每行都命中）」）。只缩不涨：仓的 HEAD 里还没有 `.claude/heavy-thread-exempt`，这一批第一次提交时基准里没有表，这一条按格的写法不比（成功句写「只缩不涨这一条没比」），这一批进多少行都不红。

### X2-甲　`|| X=$fallback`、`|| X=$((4 * 2))` 不点名（量过）

被判的字面：L2 认的钉数只认右边是字面正整数，`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/code-tooling.sh:1997`：「thread_default_form(CONDITIONAL_ASSIGNMENT_FORM, r"(?:\|\||&&)」（整行的右边是 `=[\"']?[1-9][0-9]*[\"']?`）；`|| X=…` 的左边同时被认成文件自己赋值的局部变量，读环境的那一处跟着不算。

历史：照第一轮 V3 的 `or_default.sh`，默认值先放进一个变量：

```
fallback=8
[[ -n "${SAMPLE_OR_VAR_JOBS:-}" ]] || SAMPLE_OR_VAR_JOBS=$fallback
make -j "$SAMPLE_OR_VAR_JOBS"
```

样本 `feeds/x2/sample/scripts/hosts/or_var_default.sh`、`or_arith_default.sh`（右边 `$((4 * 2))`）、对照 `or_literal_default.sh`（右边 `8`）。r2 只点名对照：

`research/prompts/m3-defs-b6-r2-opus-model/out/x2-sample-r2.txt:7`：「scripts/hosts/or_literal_default.sh:2 写死的线程数默认（|| / && 赋值）」

r1 三份都不点名；b3 三份都点名：

`research/prompts/m3-defs-b6-r2-opus-model/out/x2-sample-b3.txt:9`：「scripts/hosts/or_var_default.sh:3 读自己的线程变量 SAMPLE_OR_VAR_JOBS」

四句：分辨臂——分：b3 点名、r1 与 r2 不点名（零轮带进来，L2 只修回字面数字那一形）；看得到——看得到，`${SAMPLE_OR_VAR_JOBS:-}` 就在同一行；分句——「定义说拒而门禁不拒」：格的文件头说「没设就给默认值的写法」认成钉数，共用约束第 61 行要装置自己的线程变量不超过 N；改法——L2 在这两格上不起作用；跑前条款上 r1 也中，另立账，但它是第一轮 V3 那一形往旁边一步，交出去时写「L2 只对字面数字起作用」。

### X2-乙　已有豁免行的文件里新加的读法被吞（量过，真树拷贝）

被判的字面：`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/code-tooling.sh:2293`：「已有被判行的文件：每一行都加上这一种，排除表按那几行的锚串照放行」；文件头同一句 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/code-tooling.sh:1881`：「该文件已有的被判行各加上这一种」。`random_histories.rs` 的被判行由排除表这一行放行：`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/heavy-thread-exempt:78`：「crates/singlefs-harness/tests/random_histories.rs:available_parallelism # 代码批：随机历史快档与取样活动的线程」。

历史：就是第一轮 V3 的历史（实现员照仓里现成的写法 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/crates/singlefs-harness/tests/random_histories.rs:1959`：「fn count_from_environment(name: &str, default: u64)」加一个新并行度），只是写在 `random_histories.rs` 自己里面：在 `SINGLEFS_RANDOM_HISTORY_SEEDS` 那一行下面插一行 `let sampling_workers = count_from_environment("SINGLEFS_RANDOM_HISTORY_SAMPLING_WORKERS", 8);`（`tools/x2_absorb_insert.sh`）。这个变量不在 capped.sh 清单里，`capped.sh 31` 到不了它，没设取 8。

`research/prompts/m3-defs-b6-r2-opus-model/out/x2-absorb-r2.txt:1`：「✓ 每台默认重活线程数只经登记位取：登记值 31；扫了 832 份」

对照：同一行写进新文件 `crates/singlefs-harness/tests/new_device_control.rs`，r2 红：

`research/prompts/m3-defs-b6-r2-opus-model/out/x2-absorb-control-r2.txt:2`：「crates/singlefs-harness/tests/new_device_control.rs:2 读自己的线程变量 SINGLEFS_RANDOM_HISTORY_SAMPLING_WORKERS」

r1 两样都绿（`out/x2-absorb-r1.txt`、`out/x2-absorb-control-r1.txt`）。排除表 66 行落在 45 份文件上（其中 crates/ 11 份、scripts/experiments/ 14 份；`grep -v '^\s*#' .claude/heavy-thread-exempt | cut -d: -f1 | sort -u` 数的），这 45 份里新加的任何一处「读自己的线程变量」都这样被原有的豁免行吞掉，豁免行的理由（「代码批：随机历史快档与取样活动的线程」）说的不是新加的那一处。

四句：分辨臂——新文件那一半分（r2 红、r1 绿），已豁免文件这一半不分（两版都绿）；看得到——看得到：格自己算出了这一种，只是并到别的行上让锚串放行；分句——「照规则做、结果错」：照格的出路，排除表的每一行只为它写明理由的那一处放行，用户定案「豁免表不登记别人没审过的」在这里被一行旧理由顶着放了没人审过的新读法；改法——L2 碰不到这一格；我提的 F-X2（推的）：「读自己的线程变量」这一种不往已有被判行上并，单独落在点名它的那一行，排除表要另写一行才放。

### X2-丙　从未被看过的四种漏（量过）

样本同上（`feeds/x2/sample`），r2、r1、b3 三版都不点名（`out/x2-sample-*.txt`）：

- `heavy_named.py`：`heavy_threads = int(os.environ.get("SAMPLE_HEAVY_NAMED_THREADS", "16"))`。回退的认法按子串认，`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/code-tooling.sh:2016`：「CAP_OR_REGISTRY = re.compile(r"SINGLEFS_THREAD_CAP|heavy_thread_default|default_heavy_threads|heavy-threads\.sh|heavy_threads|capped\.sh")」，变量名里带 `heavy_threads` 就算「回退到登记值」。这个仓里「heavy threads」就是行话，起这个变量名很自然。
- `fstring_name.py`：`os.environ.get(f"E{EXPERIMENT}_THREADS", "8")`。python 去字面量时 f 串的中段一律抹掉（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/code-tooling.sh:2088`：「middle is not None and token.type == middle」），名字认不出。
- `lanes_read.py`：`SAMPLE_LANES`、`SAMPLE_PARALLELISM`。线程变量名只认 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/code-tooling.sh:1973`：「THREAD_VARIABLE = r"[A-Z0-9_]*(?:THREADS?|THREAD_CAP|JOBS|WORKERS)[A-Z0-9_]*"」，而同一格认 `--lanes` 的 default 是钉数（同一份文件第 1974 行的 THREAD_DEFAULT_KEPT_LITERAL 与「写死的线程数默认」那一形都认 `--lanes`），两处口径不一。
- `mid_tests.rs`：文件开头是 `#[cfg(test)] mod tests { … }`，后面的生产函数 `std::env::var("SAMPLE_AFTER_TESTS_WORKERS").expect(…)`。判法见到行首 `#[cfg(test)]` 接 mod 就 `break`，整份后面都不扫（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/code-tooling.sh:2263`：「code.startswith("#[cfg(test)]")」）；L2 只把外置的 `mod tests;` 改成不跳（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/code-tooling.sh:1878`：「只是声明，它后面的代码照扫」）。对照 `end_tests.rs`（测试模块在末尾）点名。

四句：分辨臂——不分（三版都漏）；看得到——都看得到；分句——「定义说拒而门禁不拒」（格的文件头「判不了的」一段只列了跨语句回退、shell 多行 if 等，没列这四种）；改法——L2 碰不到；跑前条款上另立账。

## X3　重型测试闸：env 拆法与仓副本那一支的出路

喂法：`tools/x3_env_feed.sh <仓>` 把 implementation-writer 在 `crates/singlefs-harness` 里起的命令（都经 `run-with-memory-cap.sh 2G`）做成 Bash 工具的 hook JSON 喂 `heavy-test-guard.sh` 入口；`tools/x3_copy_feed.sh <仓> <草稿>` 在工作根底下造一份改过 `write-guard.sh` 的仓副本，子 agent 与主 agent 各喂一遍。原样 `out/x3-env-*.txt`、`out/x3-copy-*.txt`。

### 专问：env 拆法的边界（量过）

| 格 | 写法（都在 harness 目录里起） | r2 | r1 |
|---|---|---|---|
| E2 | `env -- cargo test -p singlefs-checker-tier` | 拒 | 拒 |
| E3 | `env -u FOO cargo test -p singlefs-checker-tier` | 拒 | 拒 |
| E4 | `env --unset=FOO --chdir=../singlefs-checker-tier cargo test` | 拒 | 拒 |
| E5 | `env --spl='cargo test -p singlefs-checker-tier'`（缩写的长选项） | 拒 | 放 |
| E6 | `env -S 'env -C ../singlefs-checker-tier cargo test'`（-S 里再嵌 env -C） | 拒 | 放 |
| E8–E12 | `-S` 里用 `\_` 隔词、串里带引号、串后面还有参数、`-vS`、串里带 `#` 注释再接参数 | 拒 | 放 |
| E13 | `env -uC ../singlefs-checker-tier cargo test`（getopt 把 C 当 -u 的值，真 env 去执行那个目录、起不来） | 放（对） | 放 |
| E14 | `env --default-signal=PIPE -C ../singlefs-checker-tier cargo test` | 拒 | 拒 |

L3 在第一轮 C2、C8 那几形与这几种边界上都修到了。

### X3-甲　-S 串里的 `${VAR}`：GNU env 展开，闸照原文留着（量过）

被判的字面：共用约束 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/agent-common.md:81`：「写在它们后面不放过也不绕过」；库的拆词 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_shell_words.py:523`：「`$NAME`、`${NAME}` 不展开（照原文留着）」。本机 GNU env 9.4 的 -S 只认 `${VARNAME}` 并展开它（`PKG=singlefs-checker-tier env -S 'echo cargo test -p ${PKG}'` 打出 `cargo test -p singlefs-checker-tier`；`D=/tmp env -S '-C ${D} pwd'` 打出 `/tmp`，都是这一次现跑的）。

历史：实现员想把包名抽成变量，写 `PKG=singlefs-checker-tier bash ../../scripts/hosts/run-with-memory-cap.sh 2G env -S 'cargo test -p ${PKG}'`。r2、r1 都放：

`research/prompts/m3-defs-b6-r2-opus-model/out/x3-env-r2.txt:2`：「E1-prefix-var-in-S rc=0」
`research/prompts/m3-defs-b6-r2-opus-model/out/x3-env-r2.txt:5`：「E1d-prefix-var-chdir-in-S rc=0」

（E1b 先 export、E1c 只把包名后半截写成 `${TIER}` 同样放。）

四句：分辨臂——不分（r1 不拆 -S、一样放）；看得到——看得到：赋值与 -S 串在同一条命令文本里，库已经收了命令前的赋值（`PrefixSkip.assignments`），只是拆词时不拿它展开；分句——「定义说拒而钩子不拒」（第 81 行「写在它们后面不放过也不绕过」）；改法——L3 在这一格上不起作用（L3 照第一轮 C8 的字面串验过）；另立账。F-X3a（推的）：拆 -S 串时 `${NAME}` 用这段命令文本里已知的赋值与 export 展开，展开不了的按认不出判拒。

### X3-乙　-p 的值写在变量里（量过，从未被看过）

`export PKG=singlefs-checker-tier; … cargo test -p "$PKG"`（E1f）与 `PKG=…; … cargo test -p "$PKG"`（E1e）两版都放：

`research/prompts/m3-defs-b6-r2-opus-model/out/x3-env-r2.txt:7`：「E1f-export-shell-var-p rc=0」

闸文件头「看不见的」一段列了「变量里拼出来的命令与脚本路径」「目录写在变量里」，没列「-p 的值写在变量里」；共用约束第 81 行写「-p / --package 点名它的也判」。分辨臂不分（两版都放）；看得到——看得到（同一条命令文本里有 `PKG=` 的定义，闸对命令词已经会往定义里找 checker 档测试目标名）；分句「定义说拒而钩子不拒」；另立账。

### X3-丙　L3 的出路：主 agent 直接带 user-request，与「先弹窗问用户」相抵（量过）

被判的字面：L3 改的拒绝说明，`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_heavy_tests.py:1447`：「加在原来那条命令（解释器、路径、--selftest 照旧）的最前面」（前一段是「由主 agent 在集成副本里跑，前缀 SINGLEFS_HEAVY_TESTS=user-request」），共用约束同一句 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/agent-common.md:81`：「前缀、在集成副本（补丁打上之后的树）里起」；而 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/rules/implementation-workflow.md:53`：「任务确实要跑时主 agent 先弹窗问用户」（那一行是「用户要求」一档，命令带 user-request）。同一道闸给主 agent 的拒绝说明：

`research/prompts/m3-defs-b6-r2-opus-model/out/x3-copy-r2.txt:7`：「→ 照抄（提交流程里跑；用户要求时把 commit 换成 user-request，提交之外要跑先弹窗问用户）」

历史：tooling-writer 在仓副本里改了 `write-guard.sh`，跑它的自证被拒（K1），照拒绝说明把命令交回。主 agent 照这段拒绝说明在集成副本里起 `SINGLEFS_HEAVY_TESTS=user-request bash <副本>/.claude/hooks/write-guard.sh --selftest`：放（K3 rc=0），没有弹窗、用户也没要求。主 agent 若不带前缀先试（K2），闸给的出路又是「提交之外要跑先弹窗问用户」。同一条命令，经子 agent 转来的说明是「带 user-request 直接跑」，主 agent 自己撞上的说明是「先弹窗」。r1 的子 agent 拒绝说明只写「由主 agent 在集成副本里跑」，没给前缀（`out/x3-copy-r1.txt` 第 1 段），这一句是这一批带进来的。另外例子 `SINGLEFS_HEAVY_TESTS=user-request .claude/hooks/write-guard.sh --selftest` 去掉了 `bash`，与同一句「解释器……照旧」不一致（对 `.py` 的库要有执行位与 shebang 才起得来，推的）。

四句：分辨臂——分（这一批带进来的）；看得到——闸看不到用户问没问过（判别子观测不到），能改的是文字；分句——「定义之间矛盾」：共用约束第 81 行与 implementation-workflow.md 第 53 行对 user-request 前缀的用法不一，一处许主 agent 自己判、一处要先问用户；后果关「仓副本自证收回放宽」那一条定案：主 agent 照第 81 行做，改过的自证在集成副本里不问用户就跑了；改法——F-X3c（推的）：集成副本里跑改过的自证另给一个前缀值（例如 `=integration-selftest`，闸只在工作根底下副本里、单独 `--selftest` 时认它），user-request 留给用户要求的那一档；或者两处都写「先弹窗」。

## X5　private-values 整串比对

喂法：`tools/run_pv_cell.py <门禁根> <样本 git 仓>`（第一轮攻方的工具，抽格里的 python 原样跑，改动范围取 git 的未跟踪加已改动）；`tools/run_scan_form.py <门禁根> <样本> <形态键>` 抽同一道 doc-text 里 prime-marks / clock-times / session-names 三格共用的 scan_form python 原样跑。样本 `feeds/x5/pv-mirror-siblings/`：第一轮 V7 那一段历史（从第二台 tar 一个产物目录过来，第二台仓路径整段成了本地路径的一部分），假配置照第一轮样本（`PEER_REPOSITORY_DIRECTORY=/srv/sample-private/peer-tree`，编的），目录里多一份带 NUL 的 `e999.img` 与一份带 ISO 时间戳的 `e999.log`。原样 `out/x5-siblings.txt`。

### X5-甲　同一道 doc-text 的兄弟格原样打路径（量过）

被判的字面：这一道的输出约定只写在 private-values 一格（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/doc-text.sh:51`：「红行里的路径、文件名那一类红行都这样打」）；兄弟三格的射程是全仓（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/doc-text.sh:22`：「射程是全仓，冻结证据」），成功句之后逐个列二进制文件（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/doc-text.sh:224`：「二进制：{path}」）。

r2 private-values 把路径打码（L5 修到了）：

`research/prompts/m3-defs-b6-r2-opus-model/out/x5-siblings.txt:3`：「research/results/〔文件名含私有值〕/research/results/e999.img 键名：PEER_REPOSITORY_DIRECTORY（值不打印）」

同一次的 clock-times（红）与 prime-marks（绿）把同一条路径原样打出：

`research/prompts/m3-defs-b6-r2-opus-model/out/x5-siblings.txt:10`：「research/results/from-peer/srv/sample-private/peer-tree/research/results/e999.log:1 ISO 时间戳」
`research/prompts/m3-defs-b6-r2-opus-model/out/x5-siblings.txt:22`：「二进制：research/results/from-peer/srv/sample-private/peer-tree/research/results/e999.img」

（session-names 第 28 行同样；r1 四格同形，第 39、51、57 行。）

历史：执行员照第一轮 V7 把第二台的产物目录 tar 回来，提交前主 agent 跑 doc-text 这一道（或整轮门禁），把输出照「贴原样输出」贴进进度记录。private-values 一格打了码，紧挨着的 prime-marks、session-names 两格即使判绿也各打一行「二进制：」带整条路径，clock-times 红行再打一次——值照样进了仓。目录里只要有一份前 8 KiB 带 NUL 的文件（磁盘镜像、压缩包、核心转储），不需要别的条件。

四句：分辨臂——不分（r1 同样打；第一轮判决只看了 private-values 一格）；看得到——看得到：三格读的是同一份改动范围，私有值词表在 private-values 一格手里，兄弟格没用；分句——「照规则做、结果错」：违反「私有信息不进仓」；改法——L5 碰不到这一格（L5 只改 private-values 自己的红行）；另立账。F-X5（推的）：doc-text 这一道（与别的会打改动范围路径的格）打路径前统一过一遍 shown_path，或兄弟格的「二进制：」「不是普通文件」列表只报个数。

### 专问：整串比对的误报（量过、只数个数）

`tools/pv_prefix_probe.py` 照格的取词那一段原样取词，只打个数：

`research/prompts/m3-defs-b6-r2-opus-model/out/x5-prefix-probe.txt:1`：「取到的词 8 个（含本机主机名）」
`research/prompts/m3-defs-b6-r2-opus-model/out/x5-prefix-probe.txt:4`：「工作根：是它子串的词 0 个」

（`$HOME`、`$HOME/code`、仓根、仓根下 research/results 子目录同样 0 个。）真配置里没有一个长词是本机常见绝对路径的子串，这一格在本机不会把正常的工作根、仓根路径当私有值。私有值是路径前缀的情形（值是 `/srv/x`、路径里是 `/srv/x-old/`）会命中，命中的那一级目录整级打码，是多打、不漏。私有值是常见目录名（8 个字以上，如 `research`、`experiments`）时路径一侧几乎每份文件都红——真配置里没有这种词（上面的量与第一轮「相对路径命中 0 个」），没造历史。private-values 一格自己：打码区间从值开头那一级到结尾那一级、重叠并段，按代码推值不会从这一格的红行里漏；样本上 r2 的红行也没出现假值。

### 专问：还有没有别的通道

- 兄弟格（上面 X5-甲，量过）。
- private-values 一格自己的 `fail(f"取不到 {path} 相对基准 …")` 与读文件时的异常回溯（权限位 000 的文件 `open` 抛 PermissionError，回溯带原样路径）不经 shown_path（推的，没造：要 git diff 失败或文件读不了）。

## X6　规范句逐字、后接不含数字的说明

被判的字面：说明只判数字，`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_long_run.py:84`：「HOSTS_NOTE_DIGIT = re.compile(r"\d")」，文件头 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_long_run.py:21`：「说明里写的是什么钩子只判有没有数字」；定义要的做法 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/main-agent.md:55`：「每台线程取登记的每台默认重活线程数，不现算」；共用约束 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/agent-common.md:67`：「新脚本不往表里加」（指豁免表）。

喂法：派发提示一侧同 X1（基础提示 `feeds/x1/base-over2h.txt`，预计挂钟超过 2 小时、分片续跑齐，只换「多机：」一行，表 `feeds/x6/variants.tsv`），原样 `out/x6-hosts-*.txt`；脚本文件头一侧 `feeds/x6/scripts/` 四份样本经 `lib_long_run.py script` 与 `detached-run.sh long-run-check` 判，原样 `out/x6-script-headers.txt`、`out/x6-detached-check.txt`；仓里现有带 `# long-run-hosts:` 的 7 份脚本经 `judge_script_header` 判，原样 `out/x6-headers.txt`。

### X6-甲　说明里写现算，⑭ 与长活声明都放（量过，这一批带进来的）

| 格 | 规范句后面接的说明 | r2 | r1 |
|---|---|---|---|
| N1 | ；机器忙时按负载降 | 放 | 拒 |
| N4 | ；按此刻空闲现算每台槽数与线程 | 放 | 拒 |
| N5 | ；每台线程按 host-resources.py 的 available_threads 取 | 放 | 拒 |
| N13 | ；除以此刻同时在跑的重活数 | 放 | 拒 |
| N2 | ；第二台减半 | 放 | 放 |
| N3 | ；第二台十六线程，本机三十一线程 | 放 | 放 |
| N6 | ；机器忙时也固定这个数，不看忙闲（定义原话） | 放 | 拒（第一轮 V8-乙 的误伤，修到了） |

`research/prompts/m3-defs-b6-r2-opus-model/out/x6-hosts-r2.txt:6`：「N4-live-idle rc=0 ✓ 派发闸照这份提示放行」
`research/prompts/m3-defs-b6-r2-opus-model/out/x6-hosts-r1.txt:6`：「N4-live-idle rc=2 ✗ 派 investigator 的」

脚本文件头一侧：`h1_note_live.sh` 的文件头是规范句加「；按此刻空闲现算每台槽数与线程（编排照旧）」，不在豁免表里。r2 长活声明齐了，r1 拒：

`research/prompts/m3-defs-b6-r2-opus-model/out/x6-detached-check.txt:1`：「r2 h1_note_live detached-run long-run-check rc=0 ✓ 长活声明齐了」
`research/prompts/m3-defs-b6-r2-opus-model/out/x6-detached-check.txt:3`：「r1 h1_note_live detached-run long-run-check rc=5」

历史：E171 下一次修订（或别的新长活），实验员照修订 11 起现算编排。豁免表不许加新脚本（共用约束第 67 行），第一轮 V8-甲 那条「改头不改身」要在文件头撒谎；这一批之后连撒谎都不用：文件头照抄规范句，后面老实接一句「；按此刻空闲现算每台槽数与线程」，`detached-run.sh long-run-check`、`start`（同一个 judge_script_header）与派发闸 ⑭ 都放，跑起来按忙闲现算。派发提示一侧同理：主 agent 写「多机：<规范句>；机器忙时按负载降」派得出去，子 agent 读到同一行里先说「不现算」、再说「按负载降」。

四句：分辨臂——分：r1 拒、r2 放（L6 删了现算词表，这一半第一轮是判对的）；看得到——看得到：现算的话就写在同一行里，第一轮的词表正是按它拒的；分句——「定义说拒而钩子不拒」：main-agent.md 第 55 行「不现算」、共用约束第 67 行新脚本不进豁免表；结果违反用户定案「固定、不看忙闲」；钩子文件头自己写「说明里写的是什么钩子只判有没有数字」，对文件头与正文不一致（V8-甲）这一句是对的，对「文件头自己写明现算」这一种说不通——判别子看得到；改法——L6 是这一格的来处；F-X6（推的）：说明里除数字外，再拒第一轮那张现算词表里的词，「不看忙闲」「不现算」这类否定的固定句列成准写（第一轮 F2 的写法），或干脆不许接说明。

### X6-乙　「不含数字」挡掉的正当写法（量过）

r2 拒、r1 放：N7「；编排照 E171 修订 11 那一套」、N8「；分片按 layer0 状态切」、N9「；冻结层被 sha256 钉住，不改」、N11 半角分号「; 双机时两台各取这一个数」；脚本文件头 `h3_note_eid.sh`（说明里写 E171 修订 11）拒；`h2_wrapped.sh` 把规范句在「读，」之后折成两行 `# long-run-hosts:`（库把同一个键几行用空格接起来，接出来「读， 每台」多一个空格，不再逐字）拒。仓里自己的自证也把「；见第 2 节」当拒（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/scripts/hosts/host-dispatch.py:1306`：「；见第 2 节」）。实验号、`layer0`、`sha256` 是这个仓说明里最常见的词；结局是多拒，改写（「第一七一号实验」）就过，不违反用户定案，代价是说明写不清。N10「；第二台１６线程」两版都拒（全角数字，L6 写明的）。

### 专问：现有脚本与定义里的写法能不能照抄过闸

仓里现有 7 份带 `# long-run-hosts:` 的脚本：6 份 E171 入口都在豁免表里、按现算读，0 条问题；`detached-run.sh` 命中的是它自证里的样本行，文件头没有长活声明（`out/x6-headers.txt`，r2、r1 同）。没有一份现成脚本按规范句写，能照抄的只有骨架打的那一句（M0 放）与自证里的「；双机时两台各取这一个数」（M1 放）。照抄 E171 入口（第一轮 V8-甲 ①）照旧被拒。自证的取样方法：`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/scripts/hosts/detached-run.sh:1793`：「登记值除以此刻同时在跑的重活数」这一格把现算的话换掉规范句的后半截、判拒；同一句话接在「；」后面（N13）的那一形自证里没有，所以 X6-甲 在自证全绿时照中。

## X7　豁免表每行带 sha256

喂法：样本 git 仓（`feeds/x7/repin/setup.sh` 现场造）里豁免表登记一份按现算写文件头的 `scripts/live.sh`（带 sha256），提交为基准；之后改 `live.sh` 的正文（槽数从现算改成写死 8），把表里的 sha256 换成新文件的，行数与路径不变。经 `lib_long_run.py exempt-check <样本> --base-table <基准那一版>`（门禁格 long-run-header-exempt 调的就是它）判。原样 `out/x7-repin.txt`、`out/x7-r1-format.txt`。

### X7-甲　重钉 sha256 不算涨（量过）

被判的字面：只缩不涨只比路径，`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_long_run.py:87`：「基准那一版（可能还是不带 sha256 的旧写法）只取路径」；对不上时的出路 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_long_run.py:293`：「或交主 agent 问用户再改表里的 sha256」；共用约束点名的按内容哈希登记的豁免表只有两张，`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/agent-common.md:64`：「按内容哈希登记的豁免表」（括号里是 crates-review-exempt、agent-def-review-exempt，没有这一张）。

`research/prompts/m3-defs-b6-r2-opus-model/out/x7-repin.txt:2`：「✓ .claude/long-run-live-hosts-exempt 1 行：每行的脚本都在、登记的 sha256 都对得上」

（同一行末尾「与基准那一版比没涨」。）r1 的表不认带 sha256 的写法；按 r1 的写法（一行「路径  # 理由」）只改正文、表不动，r1 照绿：

`research/prompts/m3-defs-b6-r2-opus-model/out/x7-r1-format.txt:2`：「✓ .claude/long-run-live-hosts-exempt 1 行：每行的脚本都在」

历史：别的会话（或这一批的执行员）改了 E171 修订 11 的入口；集成时主 agent 跑 exempt-check 红在 sha256 对不上，照出路把表里的 sha256 换成新文件的——门禁格判绿、「没涨」，没有任何一格逼它先问用户，也没有一条规矩说这张表遇到别人没审过的改动不重钉（共用约束第 64 行只点名另两张表）。新加一行会红（只缩不涨），换一行的 sha256 不红，两条出路的门槛不一。

四句：分辨臂——分了一半：r1 不重钉都绿（更宽），r2 要重钉才绿；看得到——看得到：基准那一版这一行的 sha256 就在 `--base-table` 里，只是只取路径；判不了的是「用户问没问过」；分句——「照规则做、结果错」：照出路改 sha256，违反用户定案「豁免表不登记别人没审过的」，门禁不拦；改法——L7 收窄了（r1 改了正文也绿），重钉这一条路没关；F-X7（推的）：基准那一版带 sha256 的，只缩不涨按「路径 + sha256」比，换了 sha256 的行与新加的行同样判红、出路写交用户；共用约束第 64 行把这张表加进按内容哈希登记的豁免表。

### 专问：E171 下一次修订、换行符、执行位

- 执行位：`chmod -x` 之后 sha256 不变，照绿（`out/x7-repin.txt` 第 9–11 行）；豁免给的是内容，不算错。
- 换行符：整份换成 CRLF，sha256 变了、判红，出路照「撤回或交用户」（第 12–16 行）；结局是多拒。仓里没有 `.gitattributes` 一类会改行尾的设定时这一形只在人手改时出现（推的，没查第二台的 git 配置）。
- E171 下一次修订：表的文件头写「下一次修订的入口不进这张表，文件头按规范句写」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/long-run-live-hosts-exempt:5`：「E171 下一次修订的入口不进这张表，文件头按规范句写」）；照 X6-甲，新入口写规范句、后面接「；按此刻空闲现算每台槽数与线程」就过，固定 31 那一条没人兜。另外豁免只钉入口那一份，现算真正发生在入口 exec 的 `rows_orchestrator.py` 一类编排里，改编排不动入口时入口的 sha256 照对得上（推的，没造）。

## 我提的改法（只在我的模型上量过、被攻过零轮）

每一格都是「推的」：按代码推，没实现、没跑。

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| F-X1 「线程上限说明：」一行同样不许出现数字、÷、/、一半与第一轮那张现算词表的词（否定固定句「不看忙闲」「不现算」列成准写）；共用约束第 61 行加一句「说明行只给人看，子 agent 不照它改 N」 | X1-甲 H1–H4 | 推的 |
| F-X1b ⑮ 只认代码围栏之外的行首 | X1-乙 H9 | 推的 |
| F-X2a `\|\| X=…`、`&& X=…` 的右边不是字面数字的（变量、算术、命令替换），同一份文件里另有 `${X:-}` / `$X` 读它时，X 不算局部变量 | X2-甲 | 推的 |
| F-X2b 「读自己的线程变量」不往已有被判行上并，单独落在点名它的那一行 | X2-乙 | 推的 |
| F-X2c 回退认法改成认调用（`default_heavy_threads(`、`heavy_thread_default` 命令、`os.environ.get("SINGLEFS_THREAD_CAP"`），不按子串认变量名；f 串的字面中段留着给名字认法用；线程变量名加 `LANES`、`PARALLELISM`；`#[cfg(test)] mod x { }` 只跳到它配平的那个 `}` | X2-丙 | 推的 |
| F-X3a 拆 -S 串时 `${NAME}` 用这段命令文本里的赋值与 export 展开，展开不了的按认不出拒；-p 的值是没展开的变量时，定义里点得到 checker 档包名的照点名判 | X3-甲、X3-乙 | 推的 |
| F-X3c 集成副本里跑改过的自证另给一个前缀值，user-request 留给用户要求的那一档；或两处都写「先弹窗」 | X3-丙 | 推的 |
| F-X5 同一道 doc-text 打路径前统一过 shown_path，或兄弟格的「二进制：」「不是普通文件」列表只报个数 | X5-甲 | 推的 |
| F-X6 说明里再拒现算词（同 F-X1 的词表与准写句），或不许接说明 | X6-甲 | 推的 |
| F-X7 只缩不涨按「路径 + sha256」比；共用约束第 64 行把这张表列进按内容哈希登记的豁免表 | X7-甲 | 推的 |

F-X1 与 F-X6 共用一张词表，就是第一轮判决里撤掉的那一类东西：词表的误伤（第一轮 V1-甲、V8-乙）与这一轮的漏（X1-甲、X6-甲）是同一个取舍的两头，交用户定时两头的历史都在两轮腿报告里。

## 要主 agent 跑的门禁格

命令与预计结局逐条写在 `research/prompts/m3-defs-b6-r2-opus-model/gate-samples/要主agent跑.txt`，四条：

1. heavy-thread-defaults 喂 `feeds/x2/sample`（r2、r1 各一遍）：预计两版都退 1，r2 多点名 `or_literal_default.sh`；两版都不点名 X2-甲、X2-丙 那七份。
2. heavy-thread-defaults 喂冻结副本拷贝里插了一行的 `random_histories.rs`：预计两版都退 0；再放 `new_device_control.rs`：r2 退 1、r1 退 0（X2-乙）。
3. doc-text `--check private-values,clock-times,prime-marks,session-names` 喂 X5 样本：预计 private-values 退 1（r2 打码、r1 原样），clock-times 退 1 原样打路径，prime-marks、session-names 退 0 而「二进制：」行原样打路径（X5-甲）。
4. code-tooling `--check long-run-header-exempt` 喂 X7 重钉样本：预计退 0、「与基准那一版比没涨」（基准落在样本仓 HEAD 上是推的）。

## 没打中的形状

- X1：`- **线程上限**：31` 放（对）；`**线程上限：31**`、``线程上限：`31` ``、末尾全角空格、末尾句号都拒（两版同，多拒，代价一次被拒；`out/x1-*-investigator.txt` P2–P6）。`> 线程上限：16`（引用块里）两版都按第二个值拒（H10）。说明行写在线程上限行前面（H11）照放，同 X1-甲。
- X2：真树上宽判法只带回 1 处字符串比较（dispatch-prompt.py）、L2 的 `&& X=N` 误认 1 处夹下限（replay.sh），排除表各一行兜住，r1→r2 行数不变；名字元组、字典实参在真树上 0 处新出现。`${X-8}`、`${X:=8}`、`|| export X=8`、`|| { X=8; }` 按代码推都认（没造样本）。
- X3：`env --`、`-u`、带等号与缩写的长选项、`-S` 里嵌 `env -C`、`\_` 隔词、串后接参数、`-vS`、串里 `#` 注释（E2–E14）都拒，L3 修到了；九层以上嵌套的 -S（`MAXIMUM_NESTING = 8`）没造出能跑的串，不算。仓副本那一支：子 agent 带 `user-request` 前缀照拒（K5），主 agent 带 `commit` 或 `user-request` 都放（K3、K4）。
- X5：private-values 一格自己的红行在样本上没漏值；真配置的 8 个长词与本机常见绝对路径没有子串关系（只数个数）。
- X6：规范句本身、骨架打的那一句、自证里的「；双机时两台各取这一个数」照抄都过（M0、M1）；全角数字说明拒（N10，L6 写明的）。
- X7：执行位不影响判定（对）；CRLF 判红（多拒）。

## 这条腿自己的限度

- 门禁格只抽格里的 python 在小树上跑（基准 none 或样本仓 HEAD），没起 `--check`；`changed-paths.sh` 的基准取法与抽出来跑的差别见第一轮攻方报告同一节。四条交主 agent。
- 子 agent 会不会照「线程上限说明：」或「多机：」说明里的话改线程数，是推的，没派真 agent。
- X2-乙 的「45 份文件」是排除表的不同路径数，没逐份核每份里是不是真有装置会加新读法。
- X5 的异常回溯通道、X7 的编排不在钉里两条是推的。
- 重型测试不跑；没编译 Rust；没跑任何 `--selftest`；没读主工作区里被判的文件与禁读清单里的任何一份（`run-with-memory-cap.sh`、`replace-once.py`、`cite-check.py` 这三份工具用的是主工作区的，不在被判的 33 份里）。读了第一轮攻方报告与它的模型目录（为去重、拷两份工具 `run_htd_cell.py`、`run_pv_cell.py` 与假配置样本）。
- 线程上限 31 没用上：全程没起编译、测试、变异与产物，只起 python 与 bash 判法，都加了 `nice -n 19`。
- 草稿目录 `~/.local/share/singlefs/work/m3-defs-b6-r2-opus/` 里的仓副本（`r1/`、`r2/`、`lt-*`、`rr1/`、`rr2/`、`x3c-*`、`x5-run/`、`x7-*`）交回前删掉，见交回说明。
