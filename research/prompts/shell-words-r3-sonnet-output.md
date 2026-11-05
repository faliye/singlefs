# shell-words-r3 正推腿甲报告（X1、X3）

日期 2026-10-28。腿：three-way-forward（Sonnet），格 X1、X3；X2 不归这条腿。
被判：冻结副本 `~/.local/share/singlefs/work/shell-words-r3-frozen/.claude/hooks/` 下五份钩子（sha256 与 `research/prompts/shell-words-r3-snapshot/files-sha256.txt` 逐份相同，下面「方法」一节贴了 `sha256sum -c` 式核对）。

## 〇、复跑命令与模型目录 sha256

复跑（模型目录 `research/prompts/shell-words-r3-sonnet-model/`，整条本该经内存包装；这一趟包装排不上，见「方法」第 7 点）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/shell-words-r3-sonnet-model/tools/rerun.sh \
  --repo <仓根> --frozen ~/.local/share/singlefs/work/shell-words-r3-frozen --work <新的草稿目录>
```

它在 `--work` 里搭四棵树、跑全部样本族与 X1 样本集、把汇总写进 `--work/out/`，并与模型目录 `out/` 里登记的十份汇总逐份比（`identical` 或 `DIFFERENT`）。
`tools/x1class.py` 与 `out/x1class.txt`（X1 判错条的归类计数）不在 rerun.sh 里，单独跑：`cd <草稿根> && python3 -I tools/x1class.py`。

模型目录每个文件的 sha256（`cd research/prompts/shell-words-r3-sonnet-model && find . -type f | sort | xargs sha256sum`）：

```
8d5da2bae90356912f16cd023b172459b5aea346f3f246c5fb02f2ab58e5493a  ./cases/cases-ansic.json
1e358fa46bac27e33ade9e8fe760f8b829a20e7cc239dcfa35959e671ecb2de0  ./cases/cases-brace.json
88e543d20e21c73bbedc323a469d6983decad73bff4f9b031a4adfd37f9c02e9  ./cases/cases-builtin.json
a41d368d442b5dadedf2f825e681ea2e0f1bdad80d8793120392a07a92649ea1  ./cases/cases-cpforms.json
bcf8279b557b57d0453810e28f06ecb2e2783a87aee25de64e911f25c543d7b2  ./cases/cases-empties.json
9558a12f06694fe976edd191c035da3a69b898acf4c6ee8475ea93e7f3f6e4f7  ./cases/cases-env.json
9b1c348d5e503c1cbfc34421f3e13827ed26c85b800904729e0d8faae260c47a  ./cases/cases-fncall.json
d35dcab0508eedd53f0b73de74d3196cc8dfe814bb081525257fcfda83445c1d  ./cases/cases-function.json
19617759d08acb680a36d14eebd28995cbe819bd6c572cfff77d66c4f0fd9919  ./cases/cases-fuzz2.json
9e2451dda1fa163441dff5a8d6bf75a90cd8f59d9869315e1b491bc85c498e4f  ./cases/cases-head1.json
1d419bf70c558dea294bf7eb19a3b8c267f0b057ba246c031f4b265f18cf2df9  ./cases/cases-heredoc.json
ca75293a1d922f6a0d8d73e1261b657ca1f8772562176e65777a075e0d4b2f5a  ./cases/cases-hwrite.json
988fbf7142db4a12803fa754d90800f08661670d4a16705b8cab9d8c0ee3ff60  ./cases/cases-key.json
a56fabdb383f54a2683139f66879ad9ba999293125f328e2625575f2efceddaa  ./cases/cases-kf.json
666db9677e7a3791091d9429806dfb8016e94e44941aedd9faf08cb913b2cac4  ./cases/cases-ln.json
7729ff2083795a1bb4cc693d4fac7ca1b10f058bb3f370d4a4b2dcfa59e2b649  ./cases/cases-r1-575-bare.json
e093a3b2e2d975ea188cee5ad86ca210569e5fa5013d0ce1726e1252eb396412  ./cases/cases-r1-575.json
b08e87b501f29d50ac78e2997a7f40c009d5ab2a1e8351f14a67a1b724eae151  ./cases/cases-redirect.json
d7f28fc2fbe61867d47d438307dff9081b95849a2e839e268b98442fb8f6b7cd  ./cases/cases-shell.json
a1096e9d19a7b768f609788994d81ca1738512617860ea48e6546e12e7fc7d53  ./cases/cases-x10.json
2393fb0e8b2d59ab0d3747c31e744292266d7ed386a9f811caa5b1b2e7eeef7a  ./cases/cases-x10-now.json
a45297d6ffad0028d5d1884cd731476bc3a4885136ad311f50e5f31e3e9228ce  ./cases/cases-x10-r1.json
cbed39247b61430bb98f4fc7414ec86c3d5c5f5154ef0a5076633e613466adef  ./cases/cases-x10-r2.json
8d62b03da6426c235746e4bbd9951d7dfd97e74608085eb352fcfd868646dd51  ./cases/cases-x1-V2-2.json
494aaf8dc19bd5e9d98426836486795fe298360b80fee58e5d5d7f2a45a53c58  ./cases/cases-x1-V2-3.json
bdff291626989844e8569b9034ff5954e1f5f05dd799c8ebd3b842bc4e458367  ./cases/cases-x1-V2-6.json
d51fc3bb0de2aafc0528e1b16f84d735ea9f7462f49058b63cc3a03bd4ccc983  ./cases/cases-x1-V3-7.json
4741627d5b43ea03cf8661df60afa82038e7b916206f7c73d8139fa71c04d4bd  ./cases/cases-x1-W1.json
718864f964c3938e033d4213b3fab19b7bd33df64a306f21687f65539d205791  ./cases/cases-x1-W2.json
1bd60908a7e287fc05e6703d851e5c7d2c761676cf0824d098ccaacf2fbd548a  ./cases/cases-x1-W3.json
ffa3555d66c0600c26c993c25c46da14669975434f998fc633fce4c62a2d8e51  ./cases/cases-x1-W4.json
f9beab1b509c591a25f5b9759be8aa770abd5d73f511417983879d005b53a154  ./cases/cases-x1-W5.json
7988d9fa84beeb8c486150ed56f9f7e4966dc9889d88e85729ff090cbc526d7b  ./cases/cases-x8.json
543d68d1ba3e3de20350c5f3c2a4516228ccb036ffbb15613a5cbe3952d84628  ./cases/cases-x9.json
c1e517b201f0176acc1f987429ed01b51b439fce0425011b25952aff81d93dd9  ./oracle/bin/cargo
03d0deb7e6dfde23bffc42d4263ddf049e0d8002269e6a284d040873419dd53d  ./oracle/bin/sudo
f4148e7eac2d64521c6923cbf1f6b1013ec4dccc09f7bca7ab55dc2fb4b5a765  ./oracle/bin/systemd-run
1f75b8a5c197ab41370af5eed158b4e84920cc74f5f838702585205507532f93  ./out/x1class.txt
3fad2e43a8af600dd8394c8a7e85dc8a4e77527f0a6bb40769d869d76ddc60b5  ./out/x1.txt
8d83facab2fcb26cb09ddf554df06b1804f242c7a3075cfcf5f22b972fcd0dda  ./out/x3brace.txt
efbc165be261c9aea6283eb42314c4ce36220647992105025d937600293e4191  ./out/x3chg.txt
16823b0bdf44f27cedc53726b655de986f24607cacc5e2cd4ca55510d28b06b2  ./out/x3fam.txt
5138528a45c1a94e82601c2b4b3ab167100b87b50554b3eec890ccc494b11a87  ./out/x3kf.txt
492546d280787fad6b17424cdefc5ba3677e5bb0bdc3d621d1c704319ce68c14  ./out/x3pre.txt
35d5d9d2df3629c7a24e5058ee428142a402dc9192686bc7744cd8b148b3f178  ./out/x3s2.txt
f2cda3eb7179719004ec8e762d504d6b3575dd0fe317f99fb2e018d882c16080  ./out/x3selftests.txt
1d00b46df04d81425b748b70875f3c26e0555cfac620b22b6bc7bb6f03a2c649  ./out/x3small.txt
6562dcd626c9a718bcc5268d6201cac1378066092c3a53381ae192618be87140  ./out/x8910.txt
192b54859ad35304e61c27885b3365096655bfea40efb765d3011be5e296b7d0  ./README.md
ea3bf4b4b4b312e437b5e01a3056d9dcc8d7a4d98669f265f6e73d4548618720  ./template/light.sh
8ce34da2d691a9ac0fbf28bd6b04d2df42a9c137b5fc36649a670c9a09547744  ./template/s01.sh
8ce34da2d691a9ac0fbf28bd6b04d2df42a9c137b5fc36649a670c9a09547744  ./template/s02.sh
434ab1edcfae67175d87ede041c738ef6e092e0e31aaf7d81fa63f8bf7142499  ./template/scripts/hosts/detached-run.sh
8ce34da2d691a9ac0fbf28bd6b04d2df42a9c137b5fc36649a670c9a09547744  ./template/s.sh
dc481623fc7f75983afa32c11af0c5de3c76b7beeb19b16c6e4d6c21c3c34175  ./tools/bracecmp.py
4644202e1e3b0c39641e6df96124dad406e5826590a9deda57ac1be32e9bd945  ./tools/feedhooks.py
7f3e821003e59b154ed64f134c02216554f882667b436b7d113190f5aff738a3  ./tools/mkx1_a.py
a05f7fbb45309b2dfc04b62e99aba5ae0d280210d325e44318ae2602b1ecba22  ./tools/mkx1_b.py
d18715b3ecc201e9ed8bc61c9da3150683be1ef47881e3a980d4d08db51aa9b8  ./tools/mkx1_c.py
4127f6cb3103e48518e61126ef290d67bf73401e1786e87e314415593a543c1c  ./tools/mkx1_d.py
dcb5890847bb98df14aed8d5ab673cc330adb5cce40d8746d95efc998d516dd4  ./tools/mkx1_e.py
68d6b06ae3994f9a3d924276c504a23c1dbf6d44eda4732e3f2537de4dc77814  ./tools/myoracle.py
e9aef60b6037bd5f8f9299c325b62f6b48d4651cf1a029289c6c2376b6739b89  ./tools/probe.py
fbfc7c19eb31d2b8ad1b905f8c1b2a4ff23024f877b1877d62763528d4cbde1e  ./tools/rerun.sh
719eed043ff1e1a94c731152abb5057c6a0992a2cc5dff781660e531a0e3bf63  ./tools/runfam.sh
757f836a8e6dc8acfdf57761a1546c894d13f2c021f11beedd676d760e91af78  ./tools/s2feed.py
0910f5f6e5787dbbfd963a0067d758fddcf7b4a11fa4d0d184c80944e5947666  ./tools/selftests.py
4bc81172dafa207d9ac9cde749c8d10dc2abb7d4cf939a2e66a65e7498f2647f  ./tools/show.py
07c845730967deffd10d07a8621f45e09e26ac96512265cc94015a03b486689b  ./tools/smallchk.py
dcb2d53d5fff4d177ec0bfdf3cead396779a2c8223be4bfa10d26c4a1f817725  ./tools/trans.py
f02bc2ece12b45c8408f3e82258ea8b3d5d0001dc2a6db13fc28f953e5227a63  ./tools/x1class.py
902caa2a890758ae9d98106984c3b4bfb8d38cff8c9bdc74dcd70f8175d3c451  ./tools/x1common.py
69eebb0baa3e02f70bbb29127d35d0f674124a6210d9fa0bff7f125c714b87d1  ./tools/x1sum.py
3bff2450550f24fc7a409875c22d3dba95742c92c0d619c89842e032594139cc  ./tools/x3chg.py
2f3e644774fe7ff5e5455fb14774315d83f6b03f68a1021a28908eae605a9d34  ./tools/x3fam.py
39846f07d9563f628ed84f2a60d15483d37b7841918ac1a308eb93abe6566b57  ./tools/x3kf.py
edc4e4cd9f3bbdda50878ae07cb7e87f140137145c996fdbb33ea5db80f86f15  ./tools/x3pre.py
6dfc55265264597b5e6369e99adcbdd1026e223598614db07c93387074b32027  ./tools/x3s2.py
cdecc2e9ee5dcdbf830d534d90d53ce2be426bd56ac9a13786a734e9f43510b1  ./tools/x8910.py
```

## 一、方法与口径

1. 四棵树（模型目录 `tools/rerun.sh` 搭）：`now` = 冻结副本五份钩子；`r2` = now 打回 `fix2.diff`（第二轮被判那一版）；`r1` = r2 再打回 `fix.diff`（第一轮被判那一版）；`pre` = r1 再打回 `r1.diff`（第一轮判决之前）。
   `sha256sum -c` 对三份快照清单：now 五份 OK（对 `shell-words-r3-snapshot/files-sha256.txt`）、r2 四份 OK、r1 四份 OK；pre 的四份与修补报告留的 `prer1/` 逐份同 sha（lib_shell_words.py d2ea46f65b37、bash-command-detector.sh 65c572d41083、heavy-test-guard.sh a0f5365f1ef0、lib_heavy_tests.py 642293b0477f）。
   pre 的 heavy-test-guard.sh 只打回了 `r1.diff` 十块里的七块（三块打不回），所以 pre 只当「第一轮之前的近似」，只用在 575 条的 pre 对照上。
2. 裁判 `tools/myoracle.py`（自己写，没用腿的 `oracle.py`、`judge.py`、`famcmp.py`）：每条样本 `bash -c` 放进新会话与新进程组（`start_new_session` 即 `setsid(2)`），PATH 前放假 cargo（只记参数）与假 `detached-run.sh`；
   「重活」= 日志里有 `cargo test|t` 带 `--all` 或 `--workspace`；「检出钩子的重活」= 日志里有 `detached-run start`。bash 报 `syntax error` 或 `unexpected` 的样本不进统计（单列）。20 秒超时；bash 退出后进程组还有活的最多等 0.8 秒，命令里有 `setsid`、`&`、`coproc`、`nohup` 的再等 0.6 秒让日志写完；然后对整组 `SIGKILL`。
   沙盒里放了假 `sudo`（丢选项后照跑）与假 `systemd-run`（同），因为本机真 sudo 要口令、真 systemd-run 连不上总线，这两样的命令在别的机器上是会跑的；这是与第二轮腿的裁判唯一有意不同的地方（影响见 X3 第 2 小节）。
3. 喂法 `tools/feedhooks.py`：真钩子入口，`bash <钩子>`，标准输入给 hook 的 JSON（`tool_name`、`cwd`、`tool_input.command`、子 agent 带 `agent_type`），`SINGLEFS_WORK_ROOT=/var/empty/swr`，不带 `SINGLEFS_HEAVY_TESTS`。一次约 0.1 秒，所以没走进程内喂法。
   X1 的主 agent 样本喂重型闸，clerk 样本喂检出钩子；hook 输入的 cwd 默认是草稿根的 `scratch/`（有 `s.sh`、`light.sh`、`scripts/hosts/detached-run.sh`），写范围与编译目录两组用主仓替身或 `trees/<树>` 当 cwd。
4. 样本：X3 用两条腿登记的族清单（第二轮腿的 `mkcases.py`、`fuzz2.py`、`fuzzhead.py`、第一轮 575 条，只取清单，已拷进 `cases/`）与第二轮腿乙的案例表 `cases_*.py`（只取案例数据）；X1 另自造九个样本集（`tools/mkx1_*.py`，共 4838 条）加第 8、9、10 条三组与 32 条关键反例。
5. 引导：本机 `/bin/sh` 是 dash，`env -i` 之后 PATH 清空、找不到 cargo。凡是靠这两点判出来的「误拒」，下面都标「环境」，不当打中。
6. 前两轮留到下一轮的清单（`>>`、`&>`、`>&`、`tee` 拷出、`bash -s < 文件`、`xargs env`、名单外启动器；cat 不看源文件在不在；检出钩子重定向目标花括号展开成几个词；函数体之外引号里花括号被当认不准的调用；`run_logged` 这类函数包装）落进去的只记归类，不算打中。
7. 内存包装：`bash scripts/hosts/run-with-memory-cap.sh 4G …` 第一趟排了 8 秒起跑，之后的几趟排在 slice 外（别的会话各预订 24G，slice 总上限 40.1G），排不上；从那一步起整条带 `ulimit -v 8G`（自证带 16G）与 `nice -n 19` 直接跑，写在 `out/cap-note.txt`。单条前台命令不超过 240 秒；超过的被工具转成后台、用 `proc.py wait` 等完。

## 二、X1　前两轮站住的判定，此刻撑不撑得住

判定列：撑得住 = 我的样本里没找到与真 bash 不一致且不在清单类里的写法；打中 = 找到了，命令与读数在下一节。读数全是「真 bash 应当 / 钩子退出码」，样本数以 `out/x1.txt`、`out/x1class.txt` 为准。

| 条 | 判定 | 样本与读数 |
|---|---|---|
| W1 花括号 | 撑得住（漏拒方向）；偏严 1 类打中 | 936 条（30 种负载写法 × 24 种外包法、10 种检出钩子的词 × 6 种外包法、序列 156 条）漏拒 0；误拒 62 = 引号里 23 + 反斜杠转义里 23（bash 不展开，钩子照展开，改前就有）+ `sh -c` 16（环境：dash 不展开） |
| W2 heredoc 进替换 | 打中（漏拒） | 400 条，140 条 bash 自己语法错不计；漏拒 114 = 喂 shell 的另四种写法 104（见第五节）+ 命令替换里 case 模式 `a)` 5 + 命令替换里注释带 `)` 5；`bash <<`、`sh <<`、`bash -s <<-` 在引号嵌套里全对 |
| W3 detached-run / 拷出再起 | 打中（漏拒 3 类）；主 agent 起法撑得住 | 260 条 + 二-3 样本集里的 `cat < 源`；主 agent 12 种起法 × 3 种前缀与 42 条包装起法一条没误拒；相对源、子目录 cwd、`cp`/`ln -s`/`install`/`mv`、变量里的脚本路径、`eval` 字面、`/usr/bin/time`、`flock`、`systemd-run --wait` 全对；漏拒 16 = 清单类 11（tee 8、`>>` 2、`script -qc` 1）+ `eval "$(echo …)"` 1（文件头写明判不到）+ 新的 `ln -s $PWD/…` 2、`X=bash; $X 脚本` 2；误拒 10 全是环境（dash 读不了 `set -o pipefail`）或我的子目录样本造法问题 |
| W4 env 赋值 | 撑得住；偏严 3 类打中（env 自己拒绝的写法） | 704 条漏拒 0；误拒 96 = `env -i`/`--ignore-environment` 32（环境）+ `env -0 命令` 16 + `env -u ''` 16 + `env -x`/`--foo` 32 |
| W5 time / builtin / function / bash 选项 / 双斜杠 | 打中（漏拒 1 类、误拒 1 类） | 937 条；`coproc` 5 条漏拒；`bash --version s.sh`、`bash --help s.sh` 2 条误拒；其余误拒是 `sh` 选项 56（环境）、`command -p` 5、`cd /` 之后的样本 6（沙盒造法问题） |
| 二-2 函数名拼写拆开的调用 | 撑得住（漏拒 0 / 424 条） | 误拒 114：命令词不是调函数（`command`、`builtin`、`exec`、`env`、`nice`、`bash -c`、`xargs`、通配）84、名字出现在数据里（`echo`、`printf`、heredoc）29、POSIX 形 `foo() {…}` 只定义 1 |
| 二-3 数字紧贴 `>` | 打中（漏拒 3 类、误拒 1 类） | 334 条；漏拒 29 = 清单类 13（`>>`/`&>`/`&>>`/`1>>`）+ 管道 tee 与 `exec 3>`…`>&3` 的清单类部分 + `x=1>` 2 + `1<>` 6 + `cat < 源 > 目的`、`{ cat 源; } > 目的`、`( cat 源 ) > 目的` + `||` 之后 1；误拒 `{v}>` 3（检出钩子，r1 起就有）；其余误拒 39 是 `||`、cat 找不到数字源（清单类） |
| 二-4 `bash -c` 之后的选项 | 撑得住 | `bash`、`/bin/bash`、`/usr/bin/env bash` × 37 种选项排列 × 有无参数 × 2 种钩子，没有一条与真 bash 不一致；`sh` 的 56 条是 dash（环境） |
| 二-5 `env =1` 之后再接选项 | 撑得住 | `env =1 -u X`、`env A=1 -u X`、`env -- =1`、`env -u X =1` 等在 704 条里全对 |
| 二-6 步长补零、ANSI-C、heredoc 写文件 fd | 撑得住 | 387 条漏拒 4（样本里 `#` 吃掉 heredoc 起头，造法问题）；误拒 26 全是多目标 `cp` 失败的造法问题（探针目录里 `s01.sh` 本来就在） |
| 二-7 case 模式词与 `[[ ]]` | 撑得住 | 456 条（54 条 bash 语法错不计）漏拒 0；误拒 58 = 没选中的分支（静态判不了）54 + 条件为假 2 + 裁判认不出的 `herd7`/`killall` 2 |
| 二-8 编译目录那一道的函数体 | 撑得住 | 424 条同一批调用写法，把 `cargo test --all` 换成 `cargo build`、主仓替身里喂：r1、r2 全拒，now 只放两条只定义的（`function foo {…}`、`function foo() {…}`）；漏拒 0 |
| 二-9 ⑫ 花括号 | 打中（偏严，1 条） | 59 条按文件头逐字核，不符 3 条：2 条是我写的应当值错，1 条是 `dd of=records/{a,b}.md`（文件头说命令参数上引号外的花括号逐个落点判，这里按「算不出」拒） |
| 二-10 ⑨ 从 cwd 起算 | 撑得住 | 38 条 `experiment-runner` 写 `crates/…/*.rs`：cwd 在主仓各层、仓外副本、不存在目录、`cd` 之后、绝对路径各换一遍，凡 `.rs` 的与文件头一致；其余 8 条不符是写范围那一道（⑫）按它自己的表拒，不是 ⑨ |

## 三、X1　打中的写法（原样命令、真 bash、钩子退出码）

退出码写成 `pre / r1 / r2 / now`（第一轮之前 / 第一轮被判 / 第二轮被判 / 此刻）；`\n` 是换行；「重型闸」= `heavy-test-guard.sh` 喂主 agent，「检出钩子」= `bash-command-detector.sh` 喂 clerk。原始输出在 `out/hits-probe.txt`（`tools/probe.py` 打的，贴时去掉了拒绝说明列）。

**A　命令替换里不加括号的 case 模式 `a)` 提前收尾（W2 同一机制，漏拒，静默：没有检出记录）**

| 命令（重型闸） | 真 bash | pre / r1 / r2 / now |
|---|---|---|
| `echo "$(case a in a) echo 1 ;; esac; bash <<'EOF'\ncargo test --all\nEOF\n)"` | 起了 `cargo test --all` | 0 / 0 / 0 / 0 |
| `echo "$(case a in a) cargo test --all;; esac)"` | 同 | 0 / 0 / 0 / 0 |
| `echo $(case a in a) cargo test --all;; esac)`、`: $(…)`、`true "$(…)"`、`x="$(…)"`、`$(…)` 单独成词 | 同 | 0 / 0 / 0 / 0 |
| `echo "$(echo a # c)\ncargo test --all\n)"`（注释里带 `)`） | 注释到行尾，`cargo test --all` 在替换里执行 | 0 / 0 / 0 / 0 |
| 对照：`echo "$(case a in (a) cargo test --all;; esac)"`、`x=$(case a in a) cargo test --all;; esac)`、`echo $(echo a; if true; then cargo test --all; fi)` | 同 | 2 / 2 / 2 / 2（判对） |

机制：`.claude/hooks/lib_shell_words.py` 的 `command_substitution_end` 逐字数括号深度，「elif character == ")":」（`.claude/hooks/lib_shell_words.py:696`）把 case 模式的 `)` 当成替换的收尾（这是读代码推的，读数是上表），后面的 `cargo test --all;; esac)` 落到替换外面当数据；也没有注释的处理。
归类：不在前两轮清单里；与 W2 判的「括号状态」同一处。前两轮两条腿的 `mkcases.py` 里，case 只出现在 `$(` 外面或 `x=$(` 的赋值写法里，都判对；`x=$(…)` 的赋值写法另走一条路，此刻也判对。
推翻它：把上面第二行在 now 树上喂出退出码 2。

**B　`coproc`（W5 同一类：命令前的关键字，漏拒，静默）**

| 命令（重型闸） | 真 bash | pre / r1 / r2 / now |
|---|---|---|
| `coproc cargo test --all` | 起了 `cargo test --all` | 0 / 0 / 0 / 0 |
| `coproc W { cargo test --all; }` | 同 | 0 / 0 / 0 / 0 |

机制：关键字表「COMMAND_KEYWORDS = {"if", "then", "else", "elif", "do", "while", "until", "!", "{", "}", "time"}」（`.claude/hooks/lib_shell_words.py:336`）没有 `coproc`。检出钩子 ③ 把 `coproc` 当「放出追踪」认，重型闸这一头不认。

**C　W3 / 二-3 的拷出再起（重型闸，漏拒；检出钩子同样）**

| 命令 | 真 bash | pre / r1 / r2 / now | 检出记录 |
|---|---|---|---|
| `ln -s $PWD/s.sh d.sh; bash d.sh` | d.sh 是指向 s.sh 的链接，起了 cargo | 0 / 0 / 0 / 0 | — |
| `cat < s.sh > d.sh; bash d.sh` | 拷成功，起了 cargo | 0 / 0 / 0 / 0 | 只记（「重型测试闸没判全」） |
| `{ cat s.sh; } > d.sh; bash d.sh`（`( cat s.sh ) > d.sh` 同） | 同 | 0 / 0 / 0 / 0 | 只记 |
| `cat s.sh x=1> d.sh ; bash d.sh` | cat 报 `x=1` 找不到，但 s.sh 仍写进 d.sh | 0 / 0 / 0 / 0 | — |
| 检出钩子 clerk：`X=bash; $X scripts/hosts/detached-run.sh start --label m -- sleep 1` | 调用了 `detached-run start` | 0 / 0 / 0 / 0 | — |
| 检出钩子 clerk（误拒）：`cat scripts/hosts/detached-run.sh {v}> d.sh ; bash d.sh start --label m -- sleep 1` | `{v}>` 把 fd 号放进变量 v，cat 照旧写标准输出、d.sh 是空的，没调用 | 0 / 2 / 2 / 2 | — |

文件头对「认不出」的说法：「认不出的写法（printf、echo、python 写文件、带选项的 cat、`>>` / `&>` 写出的 cat 拷贝、管道 `cat <<EOF | tee x.sh`、变量拼出的目标）照旧」（`.claude/hooks/heavy-test-guard.sh:176`）。`cat < 源`、组重定向、`$PWD` 开头的源、`x=1>` 不在这串里。
最后一行是 r1 起的回归（pre 是 0）；其余几行 pre 也是 0，不是前两轮修补带出来的。

**D　偏严（误拒），退出码四棵树都是 2**

| 命令（重型闸） | 真 bash | 说明 |
|---|---|---|
| `cargo test "--{all,x}"`、`cargo test --all\{,\}` | 引号与反斜杠里的花括号不展开，cargo 收到字面的 `--{all,x}`，没有 `--all` | 拒绝说明写「真起的那一条是 cargo test --all --x」，与 bash 不符（W1，pre 起就这样；漏拒方向 936 条没有一条） |
| `bash --version s.sh`、`bash --help s.sh` | 打印版本或帮助就退出，不读 s.sh | W5 的 bash 选项 |
| `env -0 cargo test --all`（env 退 125）、`env -u '' A=1 cargo test --all`（125）、`env -x cargo test --all`（125）、`env --foo cargo test --all`（125） | env 自己报错，cargo 不起 | W4；`env -i` 一类在本机也是不起（PATH 清空找不到 cargo），归环境 |
| `foo() { cargo test --all; }`（只定义） | 不跑函数体 | 二-2：`function foo {…}`、`function foo() {…}` 放，POSIX 的 `foo() {…}` 拒，文件头与实现只认带 `function` 的：「def drop_uncalled_function_bodies(tokens, text, stripped_heredocs):」（`.claude/hooks/lib_shell_words.py:1742`） |
| 检出钩子 clerk：`dd if=light.sh of=records/{a,b}.md` | `of=records/a.md of=records/b.md`，写 records/b.md，在范围里 | 二-9：文件头「命令参数上引号外的花括号由共用切词照 bash 展开成几个落点」（`.claude/hooks/bash-command-detector.sh:244`），`dd of=` 这一种按「落点算不出」拒 |

另：二-2 的偏严里，第二轮放行、此刻拒的数据写法（`function foo {…}; echo "fo"o`、`echo fo{o,}`、`printf '%s\n' 'a{,}' > t.txt`，共 8 条）是修补报告「函数体之外引号里的花括号被当认不准的调用」那一类的同形，不另算。

## 四、X3　修补报告「三」「四」的数，自己的判法重算

判法：第一节第 2、3 点（`myoracle.py` 当裁判，`feedhooks.py` 真入口喂 r1 / r2 / now 三棵树），没用 `famcmp.py`、`judge.py`、`oracle.py`。「报告」= `~/.local/share/singlefs/work/debt-tw-shell-words-fix2/result.md`，「登记」= 第二轮腿 `out/totals.txt`。原样输出在模型目录 `out/`（`x3fam.txt`、`x3chg.txt`、`x3pre.txt`、`x3kf.txt`、`x3s2.txt`、`x3brace.txt`、`x3small.txt`、`x3selftests.txt`）。

**4.1　各族迁移表（报告「三」正推腿甲）**：17 族 × 报告写的「此刻」格 + 没写的格视为与登记相同，我重算 r1→r2 与 r1→now。

| 族 | 登记（第二轮）/ 报告（此刻）/ 我 r1→r2 / 我 r1→now | 对上? |
|---|---|---|
| key、cpforms、heredoc、redirect、ln、shell、builtin、function、fncall、empties、hwrite、ansic、head1 | 每格都相同（报告「此刻」写出的格也相同：redirect 194/24/39/68/64/31，shell 570/8/53/38，fncall 468，hwrite 误拒→对 8，ansic 漏拒→对 23，head1 漏拒→对 778，语法错数相同） | 对上 |
| brace | 登记 对→对 181、对→误拒 2、漏拒→对 33；报告 177、6；我 r2：181、1、34；我 now：177、5、34 | 对不上 1 条 |
| env | 登记 对→误拒 9、漏拒→对 316；报告 对→对 161、对→误拒 3；我 r2：7、318；我 now：对→对 161、1、318 | 对不上 2 条 |
| r1-575 | 登记 对→对 440、误拒→误拒 61；报告 440、59；我 r2：450、51；我 now：450、49（漏拒→对 42、误拒→对 8、漏拒→漏拒 20 与报告相同） | 对不上 10 条 |
| fuzz2 | 登记 对→对 1912、语法错 300；报告误拒→对 143、误拒→误拒 112；我 r1→r2：1911、语法错 301；我 now：误拒→对 143、误拒→误拒 112 | 对不上 1 条（±1） |

对不上的原因（两边各怎么算）：
- brace 1、env 2、r1-575 10，共 13 条：样本含 `sudo`、`systemd-run`（如 `sudo {,} cargo test --all`、`sudo env =1 cargo test --all`、`systemd-run --user --wait bash scripts/hosts/detached-run.sh start …`、`sudo -n bash …`）。第二轮腿的裁判在本机真起 sudo 与 systemd-run，它们要口令或连不上总线，命令没跑，裁判判「放」，钩子拒就算误拒；我的裁判把这两样换成「丢选项后照跑」的替身，命令跑了，裁判判「拒」，同一个钩子读数算判对。我列了这 13 条的命令（`x3fam.py` 之外的核法在上一节「方法」2）。钩子读数两边相同，只是裁判不同。
- fuzz2 的 1 条：我头一趟裁判在 `setsid cargo test --all` 上与第二趟不同（`setsid` 分叉后日志写在我杀进程组之后），后来加了「含 `setsid`、`&`、`coproc`、`nohup` 的再等 0.6 秒」；加了之后 r1→r2 的 对→对 是 1911、语法错 301（登记 1912、300），差一条语法错的归类，没追到具体是哪一条（`time -p` 往 stderr 打耗时、第一行不同会改语法错归类），记为 ±1。

**4.2　逐条变了的表（报告「三」后一张表，r2→now）**：我按「r2 与 now 退出码不同」逐族数：

| 族 | 报告 | 我 | 对上? |
|---|---|---|---|
| fncall 漏拒→对 | 110 | 110 | 对上 |
| shell 误拒→对 | 216 | 216 | 对上 |
| env 误拒→对 | 6 | 6 | 对上 |
| hwrite 误拒→对 | 8 | 8 | 对上 |
| ansic 漏拒→对 | 19 | 19 | 对上 |
| head1 漏拒→对 | 125 | 125 | 对上 |
| r1-575 漏拒→对 1、误拒→对 2 | 1、2 | 1、2 | 对上 |
| redirect 漏拒→对 | 20 | 20 | 对上 |
| redirect 对→误拒 | 32 | 32（24 条重型闸、8 条检出钩子，全部 `&&` 接，16 条 cat 报源找不到、另 16 条 `2>` 吞了报错） | 对上 |
| brace 对→误拒 | 4 | 4（全是检出钩子：`> {,}`、`cat > {,}x`、`cat s.sh > d{,}.sh && bash d.sh`、`cat s.sh > {,}d.sh && bash d.sh`，bash 报 ambiguous redirect） | 对上 |
| fuzz2 对→误拒 | 1 | 1（那条 `function f() {…}` 加 `bash -c -- '…'` 的长命令；缩小的版本 `function f() { cargo test --all; }\nprintf '%s\n' 'echo a{,}' > ./t3.sh`：pre 2、r1 2、r2 0、now 2，真 bash 不起重活） | 对上 |

其余族（key、cpforms、heredoc、ln、builtin、function、empties）r2→now 的退出码没有变化。

**4.3　keyfeeds**：报告「不同 11 处」。我把第二轮腿 `keyfeeds.sh` 的 32 条命令（取 `one` 行的命令）喂 r1 / r2 / now：r2 与 now 退出码不同 11 条，与报告列的 11 条逐条同（`"fo"o`、`fo{o,}`、`01>`、`"2">`、`3>d.sh >&3`、`echo $'a\'b'; x=$(bash <<'EOF' …)` 六条 0→2；`bash -c --posix`、`bash -c -n`、`env =1 -u X`、`env A=1 -u X`、`cat <<'EOS' 2>d.sh` 五条 2→0）；r1 与 r2 不同 22 条。对上。

**4.4　small_checks**：内建名表 61、`compgen -b` 61、两边各多出 0；`RedirectOperator` 的 `copy.copy`、`copy.deepcopy`、`pickle` 三样在 r2 与 now 都是 `TypeError: RedirectOperator.__new__() missing 1 required positional argument: 'fd'`；bash 5.2.21、env (GNU coreutils) 9.4。与报告「与登记逐行相同」对上。

**4.5　正推腿乙 1943 条（报告「三」后半）**：用腿乙的案例表（`cases_*.py`）和自己的喂法（build-skeleton.sh 搭的草稿仓，钩子是冻结副本的拷贝）喂此刻，与登记的 `out/*.jsonl` 逐条比（重型闸退出码、检出钩子退出码、应当值）：此刻 1943、登记 1943、不同 37，与报告同。37 条逐条也同：O01–O07 的 imp、mai 各 7 + O30 的 9 个命令名 + O31 + O40 + O41 + O44 共 27 条（重型闸或检出钩子 2→0）、a.A81、h.Y15-mai 2 条（重型闸 2→0）、f.R11-ER、f.R12-ER 2 条（检出钩子 2→0）、g.U07-CL 1 条（检出钩子 0→2）、C12c 五格（重型闸 2→0）= 37。g.U42-CL、g.U44-CL 登记与此刻都是 (0, 0)，腿的应当值是检出钩子 2。对上。

**4.6　第一轮 575 条（报告「三」最后一段）**：报告「改前拒、改后放」13（第一轮判决之前→此刻），矩阵 166 / 168 / 13 / 228。用第一轮的探针目录形状（没有 `scripts/hosts`）喂 pre 与 now：pre→now `0→0` 166、`0→2` 168、`2→0` 13、`2→2` 228；pre→r2 为 166 / 168 / 12 / 229，与登记同。（我头一趟用了带 `scripts/hosts` 的探针目录，读数是 162 / 171 / 13 / 229，与对上的读数差 4 条；两种目录形状的差别是有没有 `scripts/hosts/detached-run.sh` 与 `sub/`，我没有追到具体是哪 4 条；换成第一轮的形状才对上。）
13 条逐条对真 bash：`env A=1 {,}cargo test --all`、`{,}cargo test --all`、`cargo{,} test --all`、`c{,}argo test --all`、`bash {,}scripts/hosts/detached-run.sh start …`、`bash --rcfile s.sh -c 'echo hi'`、`bash --init-file s.sh -c 'echo hi'`、`bash --rcfile s.sh light.sh`、`bash --rcfile s.sh -n light.sh`、`bash --rcfile s.sh` 十条 bash 不跑重活；`cp x scripts/hosts/y.sh{,}`、`cp x records/{,out}.md` 两条真 bash 报 `cp: target … No such file or directory`、什么都没写；`env FOO=1 -- cargo test --all` 真 bash 报 `env: '--': No such file or directory`。13 条放行都对。r1 树上这 13 条里 12 条已是 0、`env FOO=1 --` 是 2，即报告的「对→对 12、误拒→对 1」。对上。

**4.7　seq_diff / brace_diff2**：报告 seq_diff 腿的 tree 行「不同 shell_words 10、heavy_tests 10 → 不同 {}」。我没用腿的造词器，自己造：整数序列 1500 条（种子 3）在 r2 上 `shell_tokens` 与 `lib_heavy_tests.brace_sequence` 各 12 条与真 bash 不同、now 上 0 条；杂拌词 3000 条（种子 4）r2、now 都是 0 条；腿登记的 9 个具体词（`x{100..00..2}y`、`x{01..100..3}y`、`x{00..100..-2}y` 等）r2 上两个函数 9 条全不同、now 上 0。方向与数量级对上；样本不是同一批，10 与 12 不能逐数比。

**4.8　报告「四」14 份自证**：自己的驱动（每份放进自己的进程组，`ulimit -v 16G`、`nice 19`、超时杀整组）在仓根里跑：

| 自证 | 报告 | 我 |
|---|---|---|
| detector / heavylib / cargolock / runscope / twologs / handback / timer / hostres / sharedindex / detachedrun / agentwatch / lightguard / memorypeaks | 退出码 0；末行数字 heavylib 857、cargolock 54、runscope 102 格、twologs 49 格、handback 140、timer 65、hostres 9、sharedindex 11、detachedrun 60、lightguard 14、memorypeaks 6 | 退出码 0，末行数字逐个同；含 `✗` 的行数 detachedrun 4、agentwatch 1（报告同） |
| heavy-test-guard.sh | 退出码 1，红 2 格（读进去几份脚本 应当是 0，实际 1） | 退出码 1，红 2 格，两格的文字与报告逐字同 |

一次提醒：负载高时 agentwatch 那份在 220 秒超时（我头一趟 `rc=超时`），放宽到 600 秒后退 0；报告里的「ulimit、timeout 220」同样吃负载。
「两格在改前就有」我复核不了：把 r2 树里的 `heavy-test-guard.sh --selftest` 在草稿目录里起，重型闸拒（「副本里的脚本……内容不同」），按它的规矩不绕；报告自己也写了没在改前现跑。

## 五、新形态（这一轮冒出来的，不展开）

- 命令替换里不加括号的 case 模式 `a)` 与注释里的 `)`（本报告三-A）：重型闸静默漏拒，pre 起就这样。
- `coproc 命令`（三-B）：重型闸静默漏拒。
- `cat < 源 > 目的`、`{ cat 源; } > 目的`、`( cat 源 ) > 目的`、`ln -s $PWD/源 目的`、`cat 源 x=1> 目的`（三-C）：重型闸漏拒；前两种有「只记」检出记录，其余没有。
- `X=bash; $X scripts/hosts/detached-run.sh start …`（三-C）：检出钩子漏拒。
- 检出钩子对 `{v}>` 的误拒（三-C）：r1 起。
- `cat s.sh 1<> d.sh; bash d.sh`（`<>` 读写打开，同 `>>` 一类但不在清单字面里）6 条漏拒。
- 喂 shell 的 heredoc 另四种写法 `source /dev/stdin <<'EOF'`、`. /dev/stdin <<'EOF'`、`bash /dev/stdin <<'EOF'`、`bash -c 'cat | bash' <<'EOF'`：顶层与替换里都漏拒（104 条），归「名单外启动器」。
- 偏严：引号与反斜杠里的花括号照展开（三-D）、`bash --version/--help 脚本`、env 自己报错的选项、POSIX 形只定义不调用的函数、`dd of=…{a,b}`（三-D）。

## 六、前两轮清单的归类行

我的样本里落进清单的，只记归类，不算打中：`>>`/`&>`/`&>>`/`1>>` 拷出 13 条漏拒 + 4 条误拒（二-3 集）、`tee` 拷出 8 条（W3 集）、`>>` 2 条（W3 集）、`script -qc` 1 条（名单外启动器）；
cat 不看源文件在不在：二-3 集里源是 `1`、`2`、`"1"`、`a1`、`\1` 的 `&&` 接写法约 14 条误拒（族清单里的 32 条见 4.2）；`||` 接的 21 条误拒（静态判不了短路）；
检出钩子重定向目标花括号展开成几个词：4.2 的 4 条；函数体之外的花括号当认不准的调用：4.2 的 1 条与三-D 末句的 8 条同形；`run_logged` 函数包装：我没造（X2 的格）。

## 七、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| X1 W1 花括号 | 撑得住（漏拒 0 / 936） | 偏严：引号与转义里的花括号被展开（pre 起就有） |
| X1 W2 heredoc 进替换 | 打中 | 命令替换里不加括号的 case 模式 `a)`、注释里的 `)` 提前收尾，`bash <<EOF` 的 `cargo test --all` 漏拒，0/0/0/0 |
| X1 W3 detached-run 新认法 | 打中 | `ln -s $PWD/…`、`X=bash; $X 脚本`、`cat < 源`、组重定向漏拒；主 agent 的起法 78 条没误拒 |
| X1 W4 env 宽松赋值 | 撑得住 | 漏拒 0 / 704；env 自己报错的选项被偏严拒 |
| X1 W5 time / builtin / function / bash 选项 / `//` | 打中 | `coproc` 漏拒；`bash --version/--help 脚本` 误拒；r1→now 由对变误拒的 12 条全是 `sh -c -O/-o/-r`（dash 不认，环境） |
| X1 二-2 函数名拼写 | 撑得住 | 漏拒 0 / 424；POSIX 形 `foo() {…}` 与数据里的名字偏严 |
| X1 二-3 数字紧贴 `>` | 打中 | `x=1>` 漏拒、`{v}>` 误拒（检出钩子，r1 起）；`1<>` 为清单外 |
| X1 二-4 `bash -c` 后的选项 | 撑得住 | bash 三种起法全对；`sh` 是 dash（环境） |
| X1 二-5 `env =1` 后再接选项 | 撑得住 | 对 |
| X1 二-6 步长补零、ANSI-C、heredoc 写文件 | 撑得住 | 对 |
| X1 二-7 case 模式词与 `[[ ]]` | 撑得住 | 漏拒 0；只有没选中的分支（静态判不了） |
| X1 二-8 编译目录一道的函数体 | 撑得住 | 424 条漏拒 0，只放两条只定义的 |
| X1 二-9 ⑫ 花括号 | 打中（偏严 1 条） | `dd of=…{a,b}` 与文件头「逐个落点判」不符；其余 58 条与文件头同 |
| X1 二-10 ⑨ 从 cwd 起算 | 撑得住 | 38 条 `.rs` 的与文件头同 |
| X3 4.1 各族迁移表 | 17 族里 13 族逐格同，4 族共 14 条对不上 | 13 条是我的裁判把 sudo、systemd-run 换了替身，1 条 ±1 |
| X3 4.2 逐条变了的表 | 全部对上 | 110 / 216 / 6 / 8 / 19 / 125 / 1+2 / 20 / 32 / 4 / 1 |
| X3 4.3 keyfeeds 11 处 | 对上 | 六条 0→2、五条 2→0 逐条同 |
| X3 4.4 small_checks | 对上 | 61 / 61，三个 TypeError，版本同 |
| X3 4.5 腿乙 1943 条不同 37 | 对上 | 37 条逐条同 |
| X3 4.6 575 条 13 | 对上 | 166 / 168 / 13 / 228；13 条放行都对 |
| X3 4.7 seq_diff / brace_diff2 | 方向对上，数不可逐数比 | 我的样本 r2 上 12，now 上 0 |
| X3 4.8 14 份自证 | 对上 | 13 份退 0，heavy-test-guard 红 2 格同；「改前就有」复核不了 |

## 八、什么现象会推翻这些结论

- 「撑得住」（W1 漏拒方向、W4、二-2、二-4 至二-8、二-10）：任何一条同判法罩得到的写法，`tools/probe.py '<命令>'` 打出真 bash 起了 `cargo test --all`（或 `detached-run start`）而 now 一栏是 0，或真 bash 不起而 now 一栏是 2 且不在上面的清单类与环境类里，就推翻。我的样本是 4838 条加族清单，样本之外没有保证；这些格我只抽了一次样（单次观测的「没打中」不算数，需要第二条同立场腿印证）。
- 打中的四组（三-A 到三-D）：把那条命令在 now 树上喂出与我表里不同的退出码就推翻；A、B 在 `probe.py` 里一条命令就能复现。
- X3「对上」：主 agent 在入库装置上跑 `rerun.sh`，`x3fam`、`x3chg`、`x3pre`、`x3kf`、`x3brace`、`x3small` 六份不是 `identical` 就推翻（这六份在我这趟复跑里是 `identical`）。4.1 里「13 条对不上是裁判换替身」被推翻的条件：把 `oracle/bin/sudo`、`oracle/bin/systemd-run` 去掉后，那 13 条的读数回到报告里的数。
- X3「复核不了」：heavy-test-guard.sh 那两格红「改前就有」——要有一台允许在副本里起改前版自证的现场才能核。

## 九、没做什么

- X2（清单全不全）不归这条腿；X1 里落进前两轮清单的，只记了归类。
- 修补报告「二」的 17 个弄坏开关逐个复跑、「一」的换上记录、「五」的 69 个进程：不在 X3 点名的「三」「四」两节里，没复跑。主 agent 点的 `hook-swap.py --feed-only` 144 与 40 条没复跑。
- 没跑重型测试、没起 cargo、没碰 GPU、没做 git 写操作；`gate` 阶段与各类 lint 没跑。
- 第一轮之前（pre）那棵树 heavy-test-guard.sh 只打回了一部分（见第一节第 1 点），所以 575 条的 pre 对照只当近似。
- 没用内存包装跑完整趟（第一节第 7 点）；`rerun.sh` 的第一趟是整份跑完（退出码 0），之后发现自证与第二轮腿乙的步骤把 `SINGLEFS_WORK_ROOT=/var/empty/swr` 带进了环境（自证退 1，腿乙案例 95 条而不是 37 条），改了脚本；改后的脚本只做了续跑（`RERUN_REUSE_RES`，复用上一趟的 `res/`，重跑第 3 步起），没有再整份重跑（整份约 90 分钟，机器负载 150 上下）。
- 没读正推腿乙这一轮的任何文件（`ps` 里看到过它起的进程的命令行，里面带它的草稿目录名，没打开任何文件）。

## 十、复跑、进程核与清理

- 第一趟整份 `rerun.sh`（旧版脚本，约 90 分钟，机器负载 150 上下）：退出码 0。汇总十份与登记逐份比：`x3fam`、`x3chg`、`x3pre`、`x3kf`、`x3brace`、`x3small` 是 `identical`；`x1`、`x8910` 是 `DIFFERENT`，只差输出里的草稿目录路径；`x3s2` 是 `DIFFERENT`（腿乙案例不同 95 条，登记是 37）；`x3selftests` 是 `DIFFERENT`（detector、handback、agentwatch 三份退 1）。
  后两份是同一个原因：`rerun_steps` 开头 `export SINGLEFS_WORK_ROOT=/var/empty/swr`，自证与腿乙的案例继承了它，钩子把工作根当成不存在的目录——自证建临时目录时 `PermissionError: [Errno 13] Permission denied: '/var/empty'`，腿乙案例里写草稿目录、`rm -rf ~/.local/share/singlefs/work/*` 一类的判定翻了（多出来的 58 条）。
- 改脚本：去掉那条 `export`（`feedhooks.py` 喂钩子时自己给每次调用设）、汇总里草稿目录路径换成 `<work>`、进程核不把 `grep` 自己算进去、加 `RERUN_REUSE_RES=<上一趟的 --work>` 续跑（复用上一趟的 `res/`，重跑第 3 步起：腿乙案例、花括号、小检查、14 份自证、汇总）。
- 第二趟续跑（新脚本）：退出码 0，十份汇总全部 `identical`，`leftover_processes 0`。腿乙案例的读数是 37 条不同；另在不同位置的草稿目录里单跑一遍同样是 37，位置无关。
  退非 0 的件数：整份一趟 0、续跑一趟 0；改后的脚本没有再整份重跑（约 90 分钟）。
- 交回前 `ps` 核：命令行带本腿草稿目录 `~/.local/share/singlefs/work/shell-words-r3-sonnet/` 的进程 0 个（命令：`ps -u $(id -u) -o args | grep -E 'shell-words-r3-sonnet/' | grep -v -E 'grep|shell-words-r3-sonnet-s2' | wc -l`，输出 0）。这一趟里 `setsid cargo …`、`cat >(…)`、`bash <<` 这类样本的孙进程都在各自的进程组里被杀；中途 `ps` 里见过一个假 cargo 进程，几秒后自己退了，没有留下。
- 内存包装：见第一节第 7 点；自证与探针都带 `ulimit -v` 与 `nice`。
- 清理（删前 `du -sh`）：钩子四棵树 124M、腿乙草稿仓 244M、核位置用的第二份腿乙草稿仓 245M、两趟复跑的工作目录 399M 与 387M，主仓替身与探针目录 K 级；都已删。草稿目录 `~/.local/share/singlefs/work/shell-words-r3-sonnet/` 留 27M（样本结果 `res/`、`cases/`、`tools/`、日志 `keep/`），无仓副本、无编译目录。
