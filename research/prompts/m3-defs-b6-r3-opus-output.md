# m3-defs-b6-r3 攻方腿（Opus）报告（2026-10-13）

攻击面：正文「一」表 Y1、Y2、Y3；第三轮分工（「已定」项撑不撑得住），只攻前两轮站住的形态（第一轮 V2、V4，第二轮 X5，第二轮辩方判「修到」的条目）。
记号：r3 = 第三轮冻结副本 `~/.local/share/singlefs/work/m3-defs-b6-r3/repo/`；r2、r1 = 前两轮冻结副本；b3 = 零轮之前那一版 `~/.local/share/singlefs/work/m3-defs-b3-r2/repo/`；fix = r3 拷贝上打了本腿三份试改（`patches/`）的那一份。
钩子与判法只在草稿目录的拷贝上跑（冻结副本只读）：派发闸喂 `--check-prompt` 与 stdin 的 PreToolUse JSON，重型测试闸喂 Bash 工具的 hook JSON，门禁格只抽格里的 python 原样跑，`--check` 交主 agent（`gate-samples/要主agent跑.txt`）。没跑任何 `--selftest`，没编译，没起重型测试。

## 复跑

整条经内存包装（上限 2G），分两趟（第二趟单趟约 2 分钟），工作目录随意，`--work` 给新的空目录：

```
cd /home/fy5090/code/singlefs
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b6-r3-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-defs-b6-r3/repo --work <新目录1>
RERUN_PARTS=def bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b6-r3-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-defs-b6-r3/repo --work <新目录2>
```

r2、r1、b3 默认取 `--frozen` 同级的 `m3-defs-b6-r2/repo`、`m3-defs-b6-r1/repo`、`m3-defs-b3-r2/repo`（`R2_FROZEN`、`R1_FROZEN`、`B3_FROZEN` 可改）；四份先拷进 `--work`（不拷 target、.git、research/results 与 multi-host.env）。第二趟（def）拿第二轮辩方模型 `research/prompts/m3-defs-b6-r2-sonnet-model/` 的矩阵、以 r3 当 after、r2 当 before 重跑，只留汇总。
腿从仓里的模型目录最后复跑一次：第一趟 20 份 `result=same`、0 份 differs，挂钟 36 秒；第二趟 `def-summary.txt` same，挂钟约 2 分钟（`time` 量的）。

模型目录 `research/prompts/m3-defs-b6-r3-opus-model/` 每个文件的 sha256（61 份）：

```
0c92936ba1d7ee909877e746c68d29d4d0d8b1fbfdd2f53de630ced246d92af7  ./feeds/y2htd/sample/.claude/heavy-thread-default
3231f3efc1498c0997387135c149fabc31abc2723638cf78d3c1f57e5e235bd3  ./feeds/y2htd/sample/crates/demo/src/harness_wrapper.rs
f02b3a2a24d2d6be771c762bb36d3ffb31a0436f9f06246c2d4678bd0dc0fc64  ./feeds/y2htd/sample/crates/demo/src/lib.rs
fc9bd82d72ad7e1a9e60696dee085622d62c348b1757c28d59db6023c72001ee  ./feeds/y2htd/sample/crates/demo/src/project_wrapper.rs
4c5450ba8c8910f0a8017b89b31213d281ee5c3263be1ac055b6560a5a6341f0  ./feeds/y2htd/sample/crates/demo/src/tests.rs
78d1273421a56d2672895fbf75e83db7e7716597a2760c936d748a39ba9b0dd2  ./feeds/y2htd/sample/crates/demo/src/use_alias.rs
be9f49020cf18a169d9da06101abe8646cd79816db651a881cca9f286ff5a7a2  ./feeds/y2htd/sample/crates/demo/src/vars_iter.rs
fd713452409eba8d805a59edecaa9a4b8cbf4527d8ff1e2c21ac6200aed3bb16  ./feeds/y2htd/sample/scripts/experiments/alias_env.py
ad131d17d5c0d7235b67aeaf67153e2ea292bd3ccac0cde6fead8f29274d3d60  ./feeds/y2htd/sample/scripts/experiments/baseline_read.py
1c40b302488c91bdf83b38aae90cdd3ea9ebf221398a8185bd0f6a271a470b10  ./feeds/y2htd/sample/scripts/experiments/copied_env.py
0307030d3f491c74a9f51b8a7051d0b6c38d0ec1d831293257f5895496f921e5  ./feeds/y2htd/sample/scripts/experiments/wrap_helper.py
15b2f086fa9206e31521fd1f5cdfcbd5902f7829eb0cbcb0422446f965984b6e  ./feeds/y2htd/sample/scripts/hosts/and_default.sh
251406dc05980a697055ac3b494348d21d6ed1917accb52c1c92ccc7a454b874  ./feeds/y2htd/sample/scripts/hosts/argparse_default.py
77314fde542c805dcdad14d16707944fdde4e8bfe8bc90e863a498b0d8004b73  ./feeds/y2htd/sample/scripts/hosts/capped.sh
f20e419bc820a3a96069a10960e9175a9db2d7e2e54e6bf30bdd586ba3658f13  ./feeds/y2htd/sample/scripts/hosts/env_grep_read.sh
7080f5c4b4c5fe06ddd2b348b0aa33192ab9a7b5f37edd418d8f8efa2e0121e4  ./feeds/y2htd/sample/scripts/hosts/env_sed_read.sh
daff66fe20eb64f1c0b46df188343237f5d1b7873f392680fa513615e0b8c2d9  ./feeds/y2htd/sample/scripts/hosts/heavy_threads.py
13a8b1fa23b4e143b5e134c85a53a4e7d4f9f6db35ef0b843a1b07f88ae84a9d  ./feeds/y2htd/sample/scripts/hosts/if_default.sh
cc1e2511fee8890a7468c09dcc71f779168fcbe258ec65f173f736c01072392b  ./feeds/y2htd/sample/scripts/hosts/lower_name.py
1700a97d6657ea443ea7088278e0ba8480de3d25d1f3eb1415caf3865757c27e  ./feeds/y2htd/sample/scripts/hosts/or_default.sh
0be1d7c41556e569b2adab35331af6fa902fcb761e2854865b02bf7791aa49c0  ./feeds/y2htd/sample/scripts/hosts/positional_default.sh
83584509c8107ec0c4004d7f87386eac6358cb796bdc028c6257e6eca2fcda65  ./feeds/y2htd/sample/scripts/hosts/printenv_read.sh
59996d6f6af6418f5b926bc5b66eecaa9a92839f867a8ab8bd9e256ca71e1cf1  ./feeds/y2htd/sample/scripts/hosts/proc_environ.py
a0daed5b08813f2c3a70acbd198bd08b3e2f99c12bfe91ec18174e6b55908499  ./feeds/y3/multi-host.env
a619bc5ee4432588bc22a81b435f4d01b14e3f90361f87075dfb0169bf3ddc24  ./feeds/y3/.private-values-hostname
13a0467ff8a604ac016fbce95596d1c3c0f609a1111976da442d0749790a5661  ./feeds/y3/setup.sh
69073d84124ce7af089e1698069509d73d43e0b63788cf8bbb02bcb20ea8fbd3  ./gate-samples/make_drift_trees.sh
a7d5222efad365c853489efd249c6f0ed0e9bfeb3f9b4464907ce07a3c951cc3  ./gate-samples/要主agent跑.txt
49bf1c791fdffb86867b3617906588ce921a307cfdb93ff23f4ceea18281b238  ./out/def-summary.txt
a34a0f6617598a77b61ac457633f282ab658b543bc1d72d597231500ba7c766d  ./out/y1-drift-b3.txt
0297acde145a048914de5fd47c61e677ee9523f1e6812977fb9a4a64e3368570  ./out/y1-drift-r1.txt
0297acde145a048914de5fd47c61e677ee9523f1e6812977fb9a4a64e3368570  ./out/y1-drift-r2.txt
0297acde145a048914de5fd47c61e677ee9523f1e6812977fb9a4a64e3368570  ./out/y1-drift-r3.txt
1747748c90385154c89c14d59cb2d0c4434fa44efd1885324544ded44addf034  ./out/y1-fence.txt
400aa32fda80fc1aceb3ae76e047f93ee9e98389eb7897f7686fc545344ba0e5  ./out/y1-stdin.txt
34aca39042de94affff95ea946b5fa2935a36e9ad82a8568668a538bde60a239  ./out/y2-allow-fix.txt
34aca39042de94affff95ea946b5fa2935a36e9ad82a8568668a538bde60a239  ./out/y2-allow-r3.txt
2ca26df9d45aeda71e5869eb6feb1c9a525208507cee7bb6f4557d0947924a07  ./out/y2-env-b3.txt
76dcb04329e56dedb0527eed874da6fa8e8d025eaf23e7819d4511b8561b449f  ./out/y2-env-fix.txt
951ef2d4bc3ade04fbc81d565ae23b488a522ec07351b1fc7ad8ece9828a21b9  ./out/y2-env-r1.txt
c8afbc500af141e7fe8b1221e6aef5d911e050bbfeed7a12181779c37d96e9e7  ./out/y2-env-r2.txt
c8afbc500af141e7fe8b1221e6aef5d911e050bbfeed7a12181779c37d96e9e7  ./out/y2-env-r3.txt
f89ad9f0cfa8cc362458dd468d1504482c5265b8d8138db965c7b4e27945dc77  ./out/y2-htd-b3.txt
d50048a71d81c8cd60ce3e6efae54b21c39878a5b7f1405da12046631e232ab9  ./out/y2-htd-r2.txt
04edfb32d6d35c82a4e5f60ddcb882294ec4ba0a0f1902fe10691eb17d2df1f1  ./out/y2-htd-r3.txt
4edbc235838d65a0b2612ad98d4d51d3eabc7d90141f5a93deaef0e44c586e18  ./out/y3-git-blob.txt
4a27c6bb4fc52944cb233ca03d02a52ac4a12bbbc9d7bae06191b297c7d92390  ./out/y3-symlink-dangling.txt
c06a6072c6de6079853bbb74d9816ee6c96cd8ef485b33368fa39e00e1388fd9  ./out/y3-symlink-mount.txt
eee6e804b2a49730e467da33880986e1a21363569b48e86694260c7189573f56  ./out/y3-unreadable.txt
b810dbe0384d16a22c912e387616d2de9b3a11239b6be27d12ac451a304da96b  ./patches/doc-text.sh.diff
304f52fba1a510c92768a5a11d49a543115a7f1562bc87b30b30d042feb90764  ./patches/lib_shell_words.py.diff
51a4d0cf5a20d6e07f278781b98a5afc822602d302f87a494ca461208018a110  ./patches/runner-dispatch-guard.sh.diff
0b296ffd6fd3971cb26ddf301a421f9d64b6b7f6a2cc54841e85b71691808aec  ./tools/rerun.sh
d0c0b8a638ea079ca932bedb7e0484aad3e6acad1567ba2cf2e1a9934809bbcb  ./tools/run_htd_cell.py
c9d57797d9e38f9296c25c5931f13163d46d31f6e12c710fcc9cf345e6e53ba3  ./tools/run_pv_cell3.py
ceb395e7d57f007a0f68c56efd428409d2479516d3356299fb079a7127d2a4be  ./tools/y1_drift_b3.py
03fc5793c02c16bc374afdd160132f8df5fffa59dea2bff2134e4181c1d8483c  ./tools/y1_drift.sh
909df5f130046dadbf3021448192be46f596533b35445d368598c2b491014c97  ./tools/y1_fence_feed.py
ef8f7a07a16b261fb6422c1d82c8f88dbf8df769e2d73e21963d257e6dfffa8f  ./tools/y1_stdin_feed.py
5d2efd40c808876736c9e1b8547ea41377c554f9fa8e860234b69028ba0298f1  ./tools/y2_allow_feed.sh
1f211636f96c405dcf80f2f0a0f2fc002cb018a689c7e0ee2224c1b3588704cb  ./tools/y2_env_feed.sh
```

## 各格判定一览

「量过」= 在拷贝上真跑了钩子入口或抽出的判法，原样输出在模型目录 `out/`；「推的」= 按代码推，没跑。「r2 / r1 / b3」三列是同一段历史喂那一版的结果。

| 格 | 判定 | 打中的那一步（一句） | r2 / r1 / b3 | 这一批带进来的？ | 量过 / 推的 |
|---|---|---|---|---|---|
| Y1-甲 | 打中（定义说拒而钩子不拒；第一轮 V1-乙、X6 那几句又回到「线程上限：」「多机：」那一行本身） | 提示里有一行以三个反引号起头又在同一行闭合的行内代码（```cargo test …``` 写在行首），钩子把它当围栏开头，一直遮到下一个真代码块的开头；夹在中间的真「线程上限：16」「线程上限：31（机器忙，按负载降到 16）」「多机：每台线程按此刻空闲现算…」不判，读者眼里代码块里的那行示例 31 / 规范句反倒被钩子当成声明 | 拒 / 拒 / 拒 | 是（第二轮改法 M1、M6 加的「代码块里的行不判」） | 量过 `out/y1-fence.txt` F2 F3 H2 H3、`out/y1-stdin.txt`（stdin 入口同判） |
| Y1-乙 | 打中（钩子拒而定义没写；零轮形态 M1，只记一行） | 同一种错位，反方向：读者眼里在代码块里的示例「线程上限：31（本机）」被钩子判，拒 | 拒 / 放 / 放 | 是（M1 没修全第二轮 X1-乙） | 量过 F4 |
| Y1-丙 | 打中（照规则做、结果错） | 叫停登记的出处照规则写「records/log.md 第 3 行」，登记时收；之后在那份调度记录前面照常加一节（插一行），同一张表、同一个出处，⑥ 与门禁同读的库判「不收」：已叫停的轮重新算开着、门禁 abandoned-rounds 红 | 中 / 中 / 中 | 否（零轮之前就这样） | 量过 `out/y1-drift-*.txt`（四版） |
| Y1 修着没有 | 修着 | 第二轮辩方的派发闸矩阵（195 个标签）以 r3 当 after 重跑：r3 与 r2 不同的只有 fm-X6 的 11 行，都是 r3 拒、r2 放（M6 的本意）；没有一行 r3 放而 r2 拒 | — | — | 量过 `out/def-summary.txt` 第 1–5 行 |
| Y2-甲 | 打中（定义说拒而钩子不拒） | `env -C .. env -C singlefs-checker-tier cargo test`（两层 env 各换一次目录，第二层相对第一层）：GNU env 落到 checker 档包，库把几层 env 当一层、最后一个 -C 按起点目录解，落到 harness 底下一个不存在的目录，放；`env -S '-C .. env -C …'`、`env -C .. -S 'env -C …'` 同 | 放 / 放 / 放 | 否（零轮之前就这样） | 量过 `out/y2-env-*.txt` N1 N1b N2 N16 |
| Y2-乙 | 打中（从未被看过） | 并行度从参数取、没给参数写死 8：`jobs=${1:-8}; cargo build -j "$jobs"`；与 `env \| sed` / `env \| grep \| cut` 取线程变量、小写的变量名、读 `/proc/self/environ`，heavy-thread-defaults 一处都不点名 | 同漏（r2、b3 量过；r1 没喂） | 否 | 量过 `out/y2-htd-*.txt` |
| Y2 修着没有 | 修着 | 辩方 48 例 env 矩阵 r3 全判对；第一轮 V3 九种与 `lib.rs:8` r3 全点名；辩方 v5-chdir 11 例 r3 与 r2 逐行同 | — | — | 量过 `out/def-summary.txt` 第 6–20 行 |
| Y3-甲 | 打中（照规则做、结果错） | 产物目录里放一个符号链接，链接目标是第二台的仓路径（坏链接）或以第二台主机名命名的挂载点下的文件：private-values 绿；`git add` 进仓的是链接文字本身，带私有值 | 绿 / 绿 / 绿 | 否 | 量过 `out/y3-symlink-*.txt`、`out/y3-git-blob.txt` |
| Y3-乙 | 打中（输出漏值） | 从第二台 tar 过来的产物里有一份读不了的文件（权限 000）：格崩在 `open`，python 回溯把整条原路径（含私有值）打进门禁输出 | 同 / 同 / 同 | 否 | 量过 `out/y3-unreadable.txt`（第二轮攻方列为推的、没造） |
| Y3 X5 打码本身 | 没打中 | 打码函数在本腿样本上没漏值；Y3-甲乙都不经打码那条路 | — | — | 量过（y3 三份样本） |

## Y1　⑮ ⑭ ⑥ 修到的条目还修着没有

### 修着的（量过）

第二轮辩方模型的派发闸矩阵、stdin 入口与叫停轮矩阵以 r3 当 after、r2 当 before 原样重跑（`RERUN_PARTS=def`）：

`research/prompts/m3-defs-b6-r3-opus-model/out/def-summary.txt:5`：「fm-X6.txt 行=31 r3=2,r2=0:11」
`research/prompts/m3-defs-b6-r3-opus-model/out/def-summary.txt:4`：「fm-X1.txt 行=52」

（fm-V1、fm-V2、fm-V8、fm-X1 四份 r3 与 r2 逐行同判，第 1–4 行只有行数；fm-X6 的 11 行是 M6 把「规范句；说明」收成拒。）真表五行 r3 全收（第 22–26 行）。第一轮 V1、第二轮辩方列的括注、尖括号、「线程上限说明」那几形，第一轮 V8 乙 M1–M7 那几行，原样喂 r3 判定都没回退。

### Y1-甲　行内的三个反引号被当成围栏开头，真的那一行被遮掉（量过）

被判的字面：
- 「三个反引号围起来的代码块里的行 ⑮ 不判」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/main-agent.md:53`），同一行「机器忙时也固定这个数，不看忙闲」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/main-agent.md:53`）。
- 子 agent 一侧：「那一个整数，提示里别处写的线程数一概不照」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/agent-common.md:61`）。
- 钩子认围栏：「CODE_FENCE_OPEN = re.compile(r"^[ \t]*(`{3,})")」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/hooks/runner-dispatch-guard.sh:1019`）——行首（任意缩进）三个反引号就算开头，后面接什么都不看；关的那一行要「至少同样多个反引号、后面只有空白」。CommonMark 里反引号围栏的语言名里不许再有反引号，一行里的 ```…``` 是行内代码，不开块；缩进超过 3 格也不是围栏。钩子与读者在这一行上分道，之后每一对围栏都错开一位。

历史（investigator，2 小时内；草稿目录与报告在腿的草稿目录下）。主 agent 看到机器忙，写了 16，提示正文里顺手贴了上一次的起法（行内代码写在行首）与上一次派发的原文（代码块）：

````
开工先读 `.claude/agent-common.md`。
重型测试：不跑
草稿目录：<草稿>/
报告：<草稿>/report.md
活：在 crates/singlefs-harness 里复现一个 harness 测试的失败，交回最小复现。
现象：harness 的一个测试偶发失败。
预计挂钟：2 小时内
```cargo test -p singlefs-harness --lib``` 是上一次的起法，照它起。
线程上限：16
上一次派发的原文：
```
线程上限：31
```
````

钩子：第 8 行当开头，第 11 行（真代码块的开头）当关，8–11 行换成空行；第 12 行「线程上限：31」露出来当声明，第 13 行当一个没闭合的开头照原样留着。只看到 31，放。读者（与 markdown 渲染）：第 8 行是行内代码，第 9 行「线程上限：16」是正文里唯一的那一行，11–13 是代码块里的示例。r2 不去代码块，两行都判，16 拒。

`research/prompts/m3-defs-b6-r3-opus-model/out/y1-fence.txt:5`：「F3-fence-shift-16	investigator	r3=0	r2=2	r1=2	b3=2	fix=2」
`research/prompts/m3-defs-b6-r3-opus-model/out/y1-fence.txt:4`：「F2-fence-shift-bracket-load	investigator	r3=0	r2=2	r1=2	b3=2	fix=2」

F2 把第 9 行换成第一轮 V1-乙 的原话「线程上限：31（机器忙，按负载降到 16）」，同样放。stdin 入口（钩子真正被调的那一条路，`DISPATCH_GUARD_STATE_ROOT` 指到草稿目录）同判：

`research/prompts/m3-defs-b6-r3-opus-model/out/y1-stdin.txt:5`：「r3 退出=0 -」

⑭ 同一种错位（多机一行，超过 2 小时）：第 9 行写「多机：每台线程按此刻空闲现算，机器忙时按负载降」或「多机：台数取 --emit-hosts，每台线程取登记值的一半」，代码块里贴骨架那一句规范句；M6 新加的「同一标签写两行就拒」也看不到被遮的那一行：

`research/prompts/m3-defs-b6-r3-opus-model/out/y1-fence.txt:9`：「H2-fence-shift-live	investigator	r3=0	r2=2	r1=2	b3=2	fix=2」
`research/prompts/m3-defs-b6-r3-opus-model/out/y1-fence.txt:10`：「H3-fence-shift-half	investigator	r3=0	r2=2	r1=2	b3=2	fix=2」

四句：
- 分不分辨臂：分。r2、r1、b3 都拒，r3 放；是第二轮改法 M1（⑮）与 M6 一并加的「代码块里的行不判」带进来的。
- 被判的系统看不看得到：看得到。分辨两种读法的东西（第 8 行同一行里有闭合的反引号）就在提示文字里。
- 满足判据的哪一分句：「定义说拒而钩子不拒」——读者眼里那一行是 16（或带按负载降的括注），定义要它等于登记值且不接括注；钩子放。
- 跑前条款的改法：两轮改法里没有一条碰这一格（M1 正是它的来源）。

改法 F-Y1（只在我的模型上量过、被攻过零轮）：围栏认法照 CommonMark——开头行至多缩进 3 格、反引号围栏的语言名里不许有反引号；关的那一行同样至多缩进 3 格。补丁 `patches/runner-dispatch-guard.sh.diff`（两行）。fix 一列：F2 F3 H2 H3 全拒，F4 放；第二轮辩方 195 个标签 fix 与 r3 逐行同判（草稿里量的，命令同 def 一趟、`--frozen` 换成 fix，不进复跑）。

### Y1-乙（零轮形态 M1，只记一行）

同一错位反过来：正文里已有「线程上限：31」，之后一行行内代码、再一个代码块里贴示例「线程上限：31（本机）」，钩子判那行示例、拒（`out/y1-fence.txt` 第 6 行 r3=2、r1=0、b3=0）。M1 想修的第二轮 X1-乙（代码块示例被拒）在这一形上没修到；F-Y1 同时修它（fix=0）。

### Y1-丙　叫停登记的出处按行号引，出处那份调度记录之后照常改动，登记失效（量过）

被判的字面：「登记行里写出处，写成 `records/<文件>.md 第 N 行`（或 `records/<文件>.md:N`）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/rules/three-way-inference.md:121`）；库只读那一行：「行里没有这一轮的名字 {round_name}（那不是为这一轮说的叫停原话）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/hooks/lib_abandoned_rounds.py:189`）。登记行里不存原话，规则与定义里也没有一句「出处那一行之后不许移动」。

历史（`tools/y1_drift.sh`）：用户在调度记录里叫停 dr-r1，主 agent 照规则登记「dr-r1 / 2026-10-13 / 用户叫停；出处 records/log.md 第 3 行 / 并进别处」，库判收（⑥ 不再挡这一轮的快照，门禁 abandoned-rounds 绿）。之后照常往这份调度记录里加一节（第 2 行之后插一行小节标题；门禁 date-once 格的出路 `doc-consolidate.py migrate` 并小节、删重复日期，按它的说明推同样会挪行，推的、没跑），同一张表、同一个出处：

`research/prompts/m3-defs-b6-r3-opus-model/out/y1-drift-r3.txt:2`：「dr-r1	收」
`research/prompts/m3-defs-b6-r3-opus-model/out/y1-drift-r3.txt:4`：「dr-r1	不收：第 1 行的出处 records/log.md 第 3 行里没有这一轮的名字 dr-r1（那不是为这一轮说的叫停原话）」

r2、r1 同（`out/y1-drift-r2.txt`、`out/y1-drift-r1.txt`）；零轮之前那一版判法在派发闸脚本里（`abandoned_round_problem`），抽出来对同一棵树判，同样插行前收、插行后不收：

`research/prompts/m3-defs-b6-r3-opus-model/out/y1-drift-b3.txt:2`：「--- 插行后：dr-r1 不收：第 1 行的出处 records/log.md 第 3 行里没有这一轮的名字 dr-r1（那不是为这一轮说的叫停原话）」

后果：已叫停的轮重新算开着，派写回员与实现员改它快照里的文件又被 ⑥ 挡；门禁 abandoned-rounds 红，红的原因指到「那不是为这一轮说的叫停原话」，而登记那天是对的。插的行落在出处那一行之后不触发；落在之前（或并掉之前的小节）就触发。

四句：
- 分不分辨臂：不分（r1、r2、r3、b3 同中）。病根是各版共用的前提「出处按行号引、登记里不存原话」。
- 看不看得到：判的那一刻看不到登记时第 N 行写的是什么（表里没存），分不出「行挪了」与「指错了行」——判别子观测不到，要改的是登记的写法，不是判法的严宽。
- 分句：「照规则做、结果错」。
- 跑前条款的改法：两轮改法（L4、M4）都只动「别的轮名」与旧行名单，碰不到这一格。另立一笔账（零轮之前也中）。

改法 F-Y1c（推的，没实现）：登记行把那句「」原话也抄进第三列，判法按「这一轮的名字 + 那句原话」在出处那份文件里找，行号只当提示；找到的行不是第 N 行的报一句「出处行号已挪到第 M 行」而不判红。

## Y2　重型测试闸的 env 拆法与 heavy-thread-defaults

### 修着的（量过）

`research/prompts/m3-defs-b6-r3-opus-model/out/def-summary.txt:6`：「v5-env-matrix [after:拒✓] 30」
`research/prompts/m3-defs-b6-r3-opus-model/out/def-summary.txt:7`：「v5-env-matrix [after:放✓] 18」
`research/prompts/m3-defs-b6-r3-opus-model/out/def-summary.txt:20`：「v5-chdir r3 与 r2 逐行同」

辩方 48 例 r3 一例不错（`env -iC`、`env -S`、缩写长选项、`-S` 里嵌 `env -C` 都拒）；第一轮 V3 九种写法与 `lib.rs:8` r3 各点名一次（第 10–19 行）。仓副本那一支的拒绝说明在 r3 改成指到 `implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」那一节（M3，零轮形态，不开攻）；那一节在 r3 里在（第 45 行起），写了提交带 commit、用户要求带 user-request 并先弹窗。

### Y2-甲　两层 env 各换一次目录：库把几层 env 当一层（量过）

被判的字面：「命令里用 `cd`、`env -C`、`env --chdir=` 换了目录的，都按换到的目录判」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/agent-common.md:81`）。库的做法：「换的目录（最后一个，GNU env 取最后一个）交在 chdir 里，由调用方按当前目录解开」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/hooks/lib_shell_words.py:591`），实现是每遇到一个 -C 就「chdir = option.chdir」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/hooks/lib_shell_words.py:624`）。同一个 env 里几个 -C 取最后一个是对的；几层 env 串着时每一层的 -C 相对上一层已经换到的目录，库没有叠，仍按起点目录解最后那一个。

历史（implementation-writer 在 `crates/singlefs-harness` 里起，经内存包装）：

```
bash ../../scripts/hosts/run-with-memory-cap.sh 2G env -C .. env -C singlefs-checker-tier cargo test
```

GNU env 9.4 真跑（把 `cargo test` 换成 `pwd`，在 r3 拷贝里现跑）打出的是 `…/crates/singlefs-checker-tier`；库把它解成 `crates/singlefs-harness/singlefs-checker-tier`（不存在），当 harness 目录里的 cargo test，放：

`research/prompts/m3-defs-b6-r3-opus-model/out/y2-env-r3.txt:2`：「N1-nested-env-C	rc=0」
`research/prompts/m3-defs-b6-r3-opus-model/out/y2-env-r3.txt:1`：「N0-control-env-C	rc=2」

同一根上的三形也放：`env -C ../.. env -C crates/singlefs-checker-tier cargo test`（N1b）、`env -S '-C .. env -C singlefs-checker-tier cargo test'`（N2，shebang 一类写法）、`env -C .. -S 'env -C singlefs-checker-tier cargo test'`（N16）。r2、r1、b3 四形同放（`out/y2-env-r2.txt`、`-r1`、`-b3` 第 2–4、19 行）。同一张喂法里的对照：`cd .. && env -C singlefs-checker-tier`、`env -C .. sh -c 'cd singlefs-checker-tier && …'`、中间夹 `capped.sh` 的两层 env、`env -C <ck>/tests`、`cargo t`、`+stable`、`--manifest-path`、`nice` / `timeout` 接在 env 后面，r3 都拒（第 5–18 行）。

四句：
- 分不分辨臂：不分（r1、r2、r3、b3 同放）；病根是共用的「几层 env 前缀并成一层、只留最后一个 -C」。第二轮辩方的 48 例与第二轮攻方 E2–E14 只有一层 -C（`env -- env -C <绝对>`、`-S 'env -C <ck>'` 都是一个 -C），没测到叠加。
- 看不看得到：看得到，两层 -C 都在命令文字里。
- 分句：「定义说拒而钩子不拒」（共用约束第 81 行「都按换到的目录判」）。
- 改法：L3 只拆 `-iC`、`-S`，碰不到这一格。另立一笔账（零轮之前也中）。

改法 F-Y2（只在我的模型上量过、被攻过零轮）：每遇到一个新前缀先记下前面几层已经换到的目录，这一层的 -C 是相对路径就接在它后面解（同一层里几个 -C 仍取最后一个；绝对路径、`~`、带变量的照旧）。补丁 `patches/lib_shell_words.py.diff`（两行）。fix 一列：N1 N1b N2 N16 全拒（`out/y2-env-fix.txt`），落回 harness 的两层 env 与同一层两个 -C 仍放（`out/y2-allow-fix.txt` 三行 rc=0）；辩方 48 例 fix 全判对、辩方 v5-chdir 11 例 fix 与 r3 同（草稿里量的，不进复跑）。

### Y2-乙　heavy-thread-defaults：参数默认值写死、从 env 输出里取线程变量（量过，从未被看过）

被判的字面：「门禁 code-tooling 的 heavy-thread-defaults 格判 scripts/、.claude/gate.d/、.claude/hooks/、crates/ 里不经这里自己定并行度默认值的写法」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/heavy-thread-default:5`）；同一份文件第 6 行写用户的定案「给了参数照参数」，没给参数就该取登记值。格认线程变量只认名字里带 THREAD / JOBS / WORKERS 的全大写词：「THREAD_VARIABLE = r"[A-Z0-9_]*(?:THREADS?|THREAD_CAP|JOBS|WORKERS)[A-Z0-9_]*"」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/code-tooling.sh:1992`）。

样本 `feeds/y2htd/sample/`（第二轮辩方 v3 样本原样，加本腿六份），用第二轮辩方的 `run_htd_cell.py` 抽格里的 python 在样本上原样跑：

| 文件 | 写法 | r3 | r2 | b3 |
|---|---|---|---|---|
| `scripts/hosts/positional_default.sh` | `jobs=${1:-8}` 后 `cargo build -j "$jobs"`（给了参数照参数，没给写死 8） | 不点名 | 不点名 | 不点名 |
| `scripts/hosts/env_sed_read.sh` | `threads=$(env \| sed -n 's/^SAMPLE_ENVSED_THREADS=//p')`，`${threads:-8}` | 不点名 | 不点名 | 不点名 |
| `scripts/hosts/env_grep_read.sh` | `env \| grep ^SAMPLE_ENVGREP_JOBS= \| cut -d= -f2`，`make -j "${jobs:-8}"` | 不点名 | 不点名 | 不点名 |
| `scripts/hosts/lower_name.py` | `os.environ.get("sample_lower_workers", "8")` | 不点名 | 不点名 | 不点名 |
| `scripts/hosts/proc_environ.py` | 读 `/proc/self/environ`，按 `b"SAMPLE_PROC_THREADS="` 前缀找，没有回退 8 | 不点名 | 不点名 | 不点名 |
| `scripts/hosts/argparse_default.py`（对照） | `add_argument("--jobs", type=int, default=8)` | 点名 | 点名 | 点名 |

`research/prompts/m3-defs-b6-r3-opus-model/out/y2-htd-r3.txt:13`：「scripts/hosts/argparse_default.py:4  写死的线程数默认」

（r3 那一份 18 处里没有上表前五份；`out/y2-htd-r2.txt`、`out/y2-htd-b3.txt` 同。r1 没喂。）`${1:-8}` 这一形最自然：shell 脚本取第一个参数、没给就写死默认值是常见写法；r3 真树里 `${N:-数}` 形现有 4 处，都不是线程数（`scripts/experiments/e6-units.sh` 的轮数、`scripts/hosts/cache-keepalive.sh` 的秒数与两处退出码）——今天没有真泄漏，新脚本照这一形写不会被点名。

四句：分不分辨臂——不分（b3 同漏）；看不看得到——看得到（`-j "$jobs"` 与 `${1:-8}` 都在文件里，格已经认 `-j` 一类的选项名，只是不认小写变量的默认值）；分句——「定义说拒而钩子不拒」（heavy-thread-default 第 5 行的射程）；改法——L2、M2 只放宽认「读自己的线程变量」与 `|| X=$var`，碰不到参数默认值与 `env | sed` 取法。另立一笔账。改法 F-Y2b（推的）：shell 里被 `-j`、`--jobs`、`--threads`、`-P`、`--test-threads` 用到的变量，往回找它的赋值，`${N:-数}` / `${X:-数}` 的字面默认判「写死的线程数默认」，不看变量名大小写。

## Y3　X5 private-values（整串打码）

样本：`feeds/y3/setup.sh <case>` 在空目录里建 git 仓、提交一版干净记录，假配置照第一轮攻方样本（`multi-host.env` 与 `.private-values-hostname` 都是编的，假第二台主机名 sample-peer-box-17、假第二台仓路径 /srv/sample-private/peer-tree），再造这一次改动；`tools/run_pv_cell3.py` 抽格里的 python（r3 连同抽出去的共用库 PRIVATE_VALUES_LIBRARY）在样本里原样跑，改动范围取 git 的未跟踪加已改动（带 `-c core.quotepath=false`）。

### X5 本身

整串打码在三份样本上没漏：凡是走到打码那一步的路径都打成「〔文件名含私有值〕」（fix 一列的红行见 `out/y3-unreadable.txt` 第 22–26 行）。下面两条都不经打码那一步。

### Y3-甲　符号链接：进仓的是链接文字，格比的是链接指到的东西（量过）

被判的字面：「改动范围里每个存在的文件的路径也比同一批长词（二进制的也比），命中就红」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/doc-text.sh:52`）；格先跳过「if not os.path.isfile(path) or os.path.realpath(path) == configuration_real:」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/doc-text.sh:562`）——`isfile` 跟着链接走，坏链接整份跳过（连路径都不比）；好链接读的是目标文件的内容（`open` 跟着走）。git 对符号链接存的是链接文字（模式 120000），格一处都不比它。「管不到的」一句（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/doc-text.sh:60`）只列换了写法的值、用户原话里的别称、commit message、不足 8 个字的值，没有这一种。

历史：执行员在产物目录里放一个指到第二台产物的链接，图找起来方便：
- symlink-dangling：`research/results/peer-latest -> /srv/sample-private/peer-tree/research/results/e999`（第二台的仓路径，本机不存在）；
- symlink-mount：`research/results/peer-e999.out -> <挂载根>/sample-peer-box-17/results/e999.out`（第二台的目录挂在本机、挂载点以它的主机名命名，目标存在、内容干净）。

两种 r3 都绿：

`research/prompts/m3-defs-b6-r3-opus-model/out/y3-symlink-mount.txt:2`：「✓ 查了这次改动里 2 份文本文件新加的 2 行：对着 multi-host.env 的 4 个长词（8 个字以上）与本机主机名判了，没有一处命中」

照绿提交，进仓的内容（同一份样本里 `git add -A` 后 `git cat-file -p :<路径>`）：

`research/prompts/m3-defs-b6-r3-opus-model/out/y3-git-blob.txt:2`：「research/results/peer-latest 模式=120000 进仓的内容=/srv/sample-private/peer-tree/research/results/e999」
`research/prompts/m3-defs-b6-r3-opus-model/out/y3-git-blob.txt:4`：「research/results/peer-e999.out 模式=120000 进仓的内容=<WORK>/y3/mnt-symlink-mount/sample-peer-box-17/results/e999.out」

r2、r1、b3 同绿（`out/y3-symlink-*.txt` 每份第 4、8、12、16 行 rc=0）。

四句：分不分辨臂——不分（四版同绿，零轮之前也中）；看不看得到——看得到（`os.path.islink` 与 `os.readlink` 都在手边，格只是没问）；分句——「照规则做、结果错」：照共用约束「私有主机信息不进仓」那一条（`.claude/agent-common.md` 第 47 行）提交前跑格、格绿，值进了仓；改法——L5、M5 只动打码，碰不到。另立一笔账。

### Y3-乙　读不了的文件：异常回溯把原路径打出来（量过）

被判的字面：「输出：只报文件、行号与键名」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/doc-text.sh:53`）；格读文件头那一步没有接异常：「head = handle.read(HEAD_BYTES)」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/doc-text.sh:568`）的前一行 `with open(path, "rb")` 抛 PermissionError 时格整个崩掉。

历史：照第一轮 V7 把第二台的产物目录 tar 回来（`tar x` 保留权限位），里面有一份权限 000 的文件（root 起的装置写的、或第二台上就是 0600 而属主不同，推的）。提交前跑格：

`research/prompts/m3-defs-b6-r3-opus-model/out/y3-unreadable.txt:4`：「PermissionError: [Errno 13] Permission denied: 'research/results/from-peer/srv/sample-private/peer-tree/research/results/e999.out'」

r2、r1、b3 同（第 9、14、19 行）。这一份 git 自己也读不了、进不了仓；漏的是门禁输出，照「贴原样输出」贴进进度记录就带进仓。第二轮攻方把这一条列为推的、没造（`research/prompts/m3-defs-b6-r2-opus-output.md` 的「还有没有别的通道」一节），这里造出来量过。

四句：不分（四版同）；看得到（异常就在格自己手里）；分句——格自己的输出约定「不打印值」被破；改法——L5、M5 碰不到。另立一笔账。

改法 F-Y3（只在我的模型上量过、被攻过零轮；补丁 `patches/doc-text.sh.diff`）：先判 `os.path.islink`，链接比路径与链接文字、不跟进去；读文件头包一层 `except OSError`，读不了的列进新的一类红行、路径经 `shown_path` 打码。fix 一列：两种链接都红、点名「链接目标」与键名；读不了的那份红、路径打码（`out/y3-symlink-*.txt` 与 `out/y3-unreadable.txt` 的 fix 段）。只在本腿三份样本上跑过，doc-text 的样本档没跑（登记的自证，子 agent 不起）。

## 我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 改什么 | 修的格 | 在别的打中格上 | 量过 / 推的 |
|---|---|---|---|---|
| F-Y1 | 派发闸认代码块照 CommonMark：围栏行至多缩进 3 格，反引号围栏的语言名里不许有反引号（`patches/runner-dispatch-guard.sh.diff`） | Y1-甲（F2 F3 H2 H3 由放变拒）、Y1-乙（F4 由拒变放） | 碰不到 Y1-丙、Y2、Y3 | 量过：`out/y1-fence.txt` fix 一列；辩方 195 个标签 fix 与 r3 同判（草稿量的） |
| F-Y1c | 叫停登记行抄进那句原话，判法按「轮名 + 原话」在出处文件里找，行号只当提示 | Y1-丙 | 碰不到别的格 | 推的（没实现） |
| F-Y2 | 几层 env 前缀的 -C 逐层相对上一层解（`patches/lib_shell_words.py.diff`） | Y2-甲（N1 N1b N2 N16 由放变拒） | 碰不到 Y2-乙（那是门禁格）、C6（目录写在变量里，照旧罩不住） | 量过：`out/y2-env-fix.txt`、`out/y2-allow-fix.txt`；辩方 48 例 fix 全对（草稿量的） |
| F-Y2b | heavy-thread-defaults 认被 `-j` / `--jobs` / `--threads` / `-P` 用到的 shell 变量的 `${N:-数}` 字面默认，不看名字大小写 | Y2-乙 前三行（positional、env_sed、env_grep） | 碰不到 `lower_name.py`、`proc_environ.py`（那两份没有 `-j`，要另认「读到环境里的值又没回退」） | 推的（没实现） |
| F-Y3 | private-values 先判符号链接（比路径与链接文字、不跟进去），读文件包 `except OSError`、读不了的列成红行并打码（`patches/doc-text.sh.diff`） | Y3-甲、Y3-乙 | 兄弟三格（clock-times、prime-marks、session-names）读同一份文件时同样会崩，没改、没量 | 量过：`out/y3-*.txt` fix 段；doc-text 样本档没跑 |

## 要主 agent 跑的命令

门禁格子 agent 起不了。完整写法、造树脚本与预计在 `research/prompts/m3-defs-b6-r3-opus-model/gate-samples/要主agent跑.txt`（第三轮冻结副本的拷贝里起，前缀照主 agent 跑门禁格的做法）：

1. abandoned-rounds：`bash research/prompts/m3-defs-b6-r3-opus-model/gate-samples/make_drift_trees.sh <T>` 造 before / after 两棵；`bash <拷贝>/.claude/gate.d/doc-process-records.sh --check abandoned-rounds <T>/before` 预计退 0，`<T>/after` 预计退 1、点名 dr-r1「第 3 行里没有这一轮的名字」。零轮之前那一版拷贝上预计同样。
2. private-values：三份样本各 `setup.sh <case>` 后 `bash <拷贝>/.claude/gate.d/doc-text.sh --check private-values <样本>`。预计 symlink-dangling、symlink-mount 退 0（没点名链接），unreadable 退 1 且输出是带原路径的 PermissionError 回溯。跑完先 `chmod -R u+rwX` 再删。
3. heavy-thread-defaults：`cp -a research/prompts/m3-defs-b6-r3-opus-model/feeds/y2htd/sample <T>/htd` 后 `--check heavy-thread-defaults <T>/htd`。预计退 1，点名 argparse_default.py，不点名 positional_default.sh、env_sed_read.sh、env_grep_read.sh、lower_name.py、proc_environ.py。

预计都是按格的代码与抽出来跑的判法推的，没起过格；抽出来跑与格本身的差别：改动范围由 `changed-paths.sh` 取、基准取法不同（第一轮攻方报告同一节已写）。

## 顺手看到的零轮形态（M 改法，只记一行，不判）

- M1：Y1-乙（代码块示例行在错位时照判，X1-乙 没修全），量过 F4。
- M6：说明另起一行照普通正文写之后，第一轮 V8-乙 的「登记值的一半」「按此刻余量少给」挪到「多机：」下一行照放、子 agent 照样读得到——与第二轮 X1-甲 同形；推的（r3 钩子自证里「长活:规范句之后另起一行写普通说明放行」那一格写明放行），没另喂。
- M2：r3 排除表 89 行（r2 66 行）；拿 r2 的表当基准，r3 的格点名「排除表涨了」（第二轮辩方 `x2-base-growth` 那一步在草稿里重跑看到，没进本腿 out/）。

## 没打中的形状

- Y1 ⑮ ⑭：错位之外，第二轮辩方 195 个标签原样喂 r3 无一回退（`out/def-summary.txt`）；没试波浪线围栏（定义只写三个反引号，钩子判它不算与定义相抵）。
- Y1 ⑥：只试了出处行号漂移一形；名单名占用、别的轮名全集随时间变（仓里认得的非 -r 形轮名 310 个里只有 1 个，第二轮辩方数的）没造历史。
- Y2 env：`cd .. && env -C <相对>`、`env -C .. sh -c 'cd … && cargo test'`、`env -C <ck> bash <包装> 2G cargo test`、中间夹 `capped.sh` 的两层 env、`-C <ck>/tests`、`-C <ck>/tests/..`、env 之后接 `nice -n 19` / `timeout 600`、`cargo t`、`cargo +stable test`、`--manifest-path`、`--ch=`、`env -C <ck> bash -c 'cargo test'`，r3 都拒（`out/y2-env-r3.txt` 第 5–18 行）；GNU env 9.4 没有 `-a`，那一形不存在。
- Y2 heavy-thread-defaults：`argparse` 的 `default=8` 点名（对照）；第一轮 V3 九种与 `lib.rs:8` 点名。
- Y3：整串打码本身在三份样本上不漏值；文件名带引号、制表符而被 git 转义的路径（`changed-paths.sh` 文件头写明管不到）没造；二进制里的值（文件头写明不扫、列名）不算。

## 这条腿自己的限度

- 子 agent 拿到 Y1-甲 那份提示会照哪一行起重活，是推的：没派真 agent。钩子判定量过（`--check-prompt` 与 stdin 入口两条路）。
- Y1-甲的触发要提示里出现「行首三个反引号、同一行闭合」的行内代码，再接一个真代码块；这种写法多常见没量。Y3-甲 的链接、Y3-乙 的权限 000 文件在本项目里多常见，同样是推的；量过的只是「出现了就绿 / 就漏」。
- 门禁格都只抽格里的 python 在小树上跑，没起 `--check`；三条交主 agent。
- 四份冻结副本都拷进草稿目录再跑，冻结副本本身没碰（开工核 r3 快照 33 份非 OK 0 行）；拷贝里的 python 都设了 `PYTHONDONTWRITEBYTECODE=1` 或 `-B`。
- Y2-乙 r1 没喂；Y1-丙 零轮之前那一版是把派发闸里的判法函数抽出来调的（那一版没有共用库），不是起整个钩子。
- 试改三份只在本腿的模型与第二轮辩方的喂法上量过；没跑三份被改脚本登记的自证与 doc-text 样本档（子 agent 不起，副本里改过的脚本闸也不放）。
- 第二轮辩方模型（`research/prompts/m3-defs-b6-r2-sonnet-model/`）在 def 一趟里原样起，r3 当 after；它自己的 `rerun_compare` 对不上是因为 after 换了，本腿只取汇总。
- 开工看负载：别的会话有 `cargo test`（崩溃放量、harness），没有 `qemu-system`、`vm-bench`、`e152`、`fio`；本腿只起 python 与 bash 判法，都加了 `nice -n 19`，线程上限 31 没用上。
- 没读主工作区里被判的文件与禁读清单（调度记录、实现员自报、这一轮正推腿的报告与模型目录）。开工先读的共用约束与两份规则读的是第三轮冻结副本那一份。用到主工作区的工具：`run-with-memory-cap.sh`、`replace-once.py`、`cite-check.py`、`three-way-rerun-template.sh`（不在被判的 33 份里）。

## 没做什么

- 没起门禁格（`--check`）、没跑任何 `--selftest`、没跑 gate.sh、没编译、没跑重型测试。
- 没判正推那几格（Y4、Y5、Y6），没替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没为 M1–M7 开攻（顺手看到的各记一行）。
- 交回前删掉草稿目录里的四份冻结副本拷贝、试改拷贝与各趟 `--work`（都在 `~/.local/share/singlefs/work/m3-defs-b6-r3-opus/` 下），留下的只有喂法输出与日志，交回里列。
