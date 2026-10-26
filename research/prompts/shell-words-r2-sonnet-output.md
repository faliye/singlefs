# shell-words-r2 正推腿甲报告（Sonnet）

日期：2026-10-26。轮：shell-words-r2（第二轮）。立场：正推腿甲，回答背景材料第三节的 V1、V2。

## 0　做法、复跑、指纹

做法：从 bash 与 GNU env 的真实行为推「这一条命令该判成什么」，再把修补前、修补后两棵树的钩子判定逐族比。三把尺：

- 真 bash 5.2.21 与 GNU env 9.4（coreutils）当裁判。每条命令在一份新拷的沙盒目录里用 `bash -c` 真跑一遍（虚存 1G、30 秒），PATH 前面放一个假 `cargo`（只把参数记进沙盒根的日志）和一个假 `scripts/hosts/detached-run.sh`（带签名行，只记参数）。重型闸（主 agent 不带前缀）的裁判结果：日志里有 `cargo test|t` 带 `--all` 或 `--workspace` 算「该拒」（rc=2），没有算「该放」（rc=0）；检出器（clerk）：日志里有 `detached-run start` 算「该拒」。
- 三棵树：「修补后」= 冻结副本里的四份钩子原样；「修补前」= 同四份把 `fix.diff` 反向打回；「第一轮判决之前」= 修补前再把 `r1.diff` 反向打回（`heavy-test-guard.sh` 有两处文件头注释打不回，不影响判定）。其余钩子库、脚本、定义、门禁表从仓里拷，三棵树一样。
- 喂法两种：进程内（`fam4.py` 把钩子里的 Python 载进来逐条喂）与真入口（`feed.sh`：`bash <钩子>`，标准输入给 hook 的 JSON）。真入口只用于 `keyfeeds.sh` 那 96 行；批量全走进程内。

每条结论后面写「什么现象会推翻它」。判对判错的记号：对 = 钩子的 rc 与裁判一致；误拒 = 钩子拒了、真 bash 不会起；漏拒 = 钩子放了、真 bash 会起。

裁判够不着的（报告里逐处标出，不当判错也不当判对）：

- 命令的静态形状与运行时不一致：`a || b`、`a && b` 的短路，`for i in {,}` 的空列表，`if false`。钩子是静态判，这类「误拒」不算钩子错。
- 沙盒里跑不成的外部程序：`sudo`、`systemd-run` 一类；`env -i` 清掉 PATH 后假 cargo 找不到。
- 写范围类（检出器判 clerk 的 `cp`、`sed -i`、`>` 写仓内文件）：真 bash 不起重活，裁判没有标记可看。这类只在「第一轮判决之前拒、修补后放」的 12 条里出现 2 条，下面 1.2 手工核。
- 第一轮腿的 `W1-deep`（`{a,b}` 乘 100 到 2000 层，真 bash 会展开成 2 的 100 次方份把内存吃光）不交真 bash，沿用第一轮腿写的期望。
- bash 自己报语法错、读不完的命令（命令一个字都没跑）不进统计，每族末行单列条数。

### 0.1　复跑命令

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/shell-words-r2-sonnet-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/shell-words-r2-frozen --work <新的草稿目录>
```

输出末尾的 `name=totals.txt_vs_model`、`name=flags.txt_vs_model`、`name=keyfeeds.txt_vs_model` 三行都应是 `identical`。机器负载高的时候（我跑的时候 loadavg 在 100 上下）整份约 7 到 8 分钟，内存撞不到 4G。我交回前在新 shell 里整份跑了一次，退出码 0，三行原样如下：

```
name=totals.txt_vs_model identical
name=flags.txt_vs_model identical
name=keyfeeds.txt_vs_model identical
```

定稿前我整份跑过十来次，改掉的毛病都在模型里：第一轮腿的 `W1-deep` 命令（`{a,b}` 乘 100 层）被真 bash 展开成 2 的 100 次方份，撞了 4G 被内存包装杀掉（退 250），现在标成不交真 bash；两个放后台的假 cargo 同时追加日志、一行被另一行插断，同一条命令前后两次的裁判不同，现在假 cargo 一行拼好一次写出，裁判里放后台 `&` 的命令多等 0.6 秒；`time` 关键字往 stderr 打的耗时随负载变，现在 `TIMEFORMAT` 置空；`keyfeeds.sh` 与错误行里的草稿目录路径先换成占位再截，不随草稿目录名长短变；GNU `cut -c` 其实按字节截，会把汉字截在中间，换成按字符截的 `trunc`。最后一次验证跑我特意把 `--work` 起了个很长的目录名，三行仍是 `identical`。探索期间零散的探针没经内存包装，只带 `nice -n 19`（单进程不到 60MB，最多 24 个并行）；数据不靠那些探针，都是 `rerun.sh` 里经 4G 包装跑出来的。

### 0.2　冻结副本与树的指纹（`rerun.sh` 打出的原样输出）

```
tree 089411539235d482f44f7e2182026b17ac3bbd4da981b27561db5b72f2f6e8a6  lib_shell_words.py
tree 88cc4caa538527a3cd6858d112fbfb8bd156fefff88bf730fd9aaad3b0536a7b  bash-command-detector.sh
tree a9b3e323ac31babf3aa24a6b0ffc0af0db994a66e253d42d020afdc84d0eb9a6  heavy-test-guard.sh
tree 995a38c47929f0834acea5cf5c1be2e73473d5b6af783a90a4e28d1215431982  lib_heavy_tests.py
tree-orig 0803f0343825ed7265171cb3d537c4a069a854440b45dc01dfcb170b61371c7e  lib_shell_words.py
tree-orig b9b2962f196cd6fb5d3d909b4d92d69d8ab1ff47615e23066e4bc752601809e1  bash-command-detector.sh
tree-orig febb12c7288864633c5dfc02ff8f97ad0990ec763cced95e613a77fe477e574d  heavy-test-guard.sh
tree-orig 642293b0477fe39fd9e6958e2ffa32259b5f67415cc30fe10964fe13f786e224  lib_heavy_tests.py
pre_r1 d2ea46f65b372cab6146cb140da04fc4e61e459bd2034c52a91315cec6a894ba  lib_shell_words.py
pre_r1 65c572d4108391293adf9cae7fbdc2f81fe7332df9130e2d6be02387057f2679  bash-command-detector.sh
pre_r1 a0f5365f1ef023dd157699bafc29c72a3cae4a3908f7ec6e84e4b47bea4bd043  heavy-test-guard.sh
```

冻结副本四份与 `research/prompts/shell-words-r2-snapshot/files-sha256.txt` 一致（修补后那四行与快照清单逐字节相同）；仓里 `.claude/hooks/` 下这四份此刻同一个 sha256，所以下面引的行号按仓路径读就是冻结副本的行。

### 0.3　模型目录每个文件的 sha256（`research/prompts/shell-words-r2-sonnet-model/`）

```
2b57e7148eba5cd2d4217e3360342909f6bbb3787d5dcc00dcff225dcc39c689  families/fam-b4.json
23564eee2f18096a7d71a0c155b491dedaae1e7a90da30aad02b3baa14a7e38d  families/fam-w1.json
f4d31578f76df56a7e90b5dfe7132524b8433c08b1f885867848c0e5284187a3  families/fam-w1b.json
db5262145eab0e6aaffea1567a06eb09ec30cfa3109e361670d360fbd7562ccc  families/fam-w1c.json
740b5fe1c08003bbb8e21b122bd6aa529ef0be9b335e3c5e127ffb4f9444928d  families/fam-w1d.json
23d6c3f676c0d11d1a195c5fb5245038d2968fe31931eb7f1d5b8141edcb0b49  families/fam-w1e.json
56d76f9952bff7c630b5a9ac69bb7418c300fd2440df4fa40b4d955de93bb573  families/fam-w2.json
f2adf9df609706108ac0189edca46e006e8933f6b9a73bca25c6ea380b356df9  families/fam-w3.json
6ff8dbe7e61523b3f26e918286f84c1302e92e1fe4bd69a1aaec8f8fa9794d1d  families/fam-w3b.json
3ac73e06c7d8bd5ad38733605866343f70d81efe753380b6b88b6c01528eb2cc  families/fam-w3c.json
911c17f3365613981d769cdaf59107874be66022d68abe52140f3733a8c9fa85  families/fam-w3d.json
d61458ceaa040e4a32a9019a5185d6fb1bcc0ef83256babd411c43739679dffc  families/fam-w3e.json
04e3aafd6d9fff9f2defa9d6397400dc97992e57538aa7569e39f0851f1489d5  families/fam-w4.json
6e9bb784b0d3f81a121cc74eeff1b85151c159b63cd01f6484c40c2383192a10  families/fam-w5.json
870afaa7b673307a7c67afb3231e9da554354002824004f44050728a8dd93d7a  families/fam-w5b.json
333cdc3ee0036f61ba19ac8fda26de500ed52f1cbf01dce8505696d69cfa69c9  oracle/bin/cargo
97d0d1d1381c34636436acac781eae96f478fb9805b3bb42222ef73c6674c9db  out/brace-diff-tree-orig-seed1.txt
6a16c088e12b7d559b907fa457fae163e41d1eb228fe74e79406c2aaacab7db9  out/flags.txt
f19ce14d148e25ff6988d3cf377e06af2101d761b4f32be79ff0dbeb3424fc23  out/keyfeeds.txt
62b056a0ac235c5a5c4e6ae07e3f9a96816f0faa13b742b7d58012677538562e  out/perf-reference-not-compared.txt
47100ef87d20b5d6fea73891aa71e640f4925d58cee4e806440306a21992103c  out/seq-diff-tree-lines.txt
7c6c1d7bab9133339c476425a0f118b6c3dcce656455936ebadc9ebda3996b17  out/totals.txt
ea3bf4b4b4b312e437b5e01a3056d9dcc8d7a4d98669f265f6e73d4548618720  template/light.sh
8ce34da2d691a9ac0fbf28bd6b04d2df42a9c137b5fc36649a670c9a09547744  template/s.sh
8ce34da2d691a9ac0fbf28bd6b04d2df42a9c137b5fc36649a670c9a09547744  template/s01.sh
8ce34da2d691a9ac0fbf28bd6b04d2df42a9c137b5fc36649a670c9a09547744  template/s02.sh
31dc49dc74df73a9d7661f9607885cceb791623207c424a1f3517cd3aa2ae5e7  template/scripts/hosts/detached-run.sh
5019afa634f2768388cf2ea53c87c7382d800609e7191a184b5e5c9245ede25d  tools/brace_diff2.py
7273b2ada130c22f81f677a6b04c1745d91048c87e2bf466f8163dc510845905  tools/fam4.py
e65aa057b6f3aeb61f9517442b173c8f4c0b31df1e668ebae61e6b0019465673  tools/feed.sh
ece2f3ed64cd18f20104d14535b79d67ceb3f24a933187b6e3b9890c9111ce9d  tools/fuzz2.py
196fa4e809b2306f6045c64f5dbb89c1962bdcb321f3ede51b0b6bd0bacde199  tools/fuzzhead.py
e33ab101f844e40687db5401548637b214f0c9a5bedae7a400002826e5a266e3  tools/judge.py
5d62ab06d5c66bfffbf32e254d3cac3d018f2e691d6b63c32d7b7d24c5c5c1a9  tools/keyfeeds.sh
1db5a7489005d19e760daf21459fc26abe6b519e243533baa368f8eed942eae5  tools/matrix.py
211d1429a0a66a0fade08723c53fa2dba6a9b30dd76cc362daa71ef531855021  tools/mk2to0.py
ac831030327cd792ca3031216920daa368e73e6f1cd176ef7f68db80288df021  tools/mkcases.py
26d033386e117b8df6cc419adb14f3d0c11c90d4968fe2f6a91b0e6109614893  tools/oracle.py
7763b16b3f9e2333260d801fa3fe3cbb6f42dc36a03eb8f8dc361ec675be8e28  tools/perf.py
fa1a37bc78ca0294c99a434c1c50961ac128b5c2aee9a8a4bf7d95ef23ba5192  tools/pjudge.sh
98e32b9d1e0f826b16dff885e66ad26f3029befb2047ca13fa0376fb8f249ecf  tools/q.sh
be2e9ac0644e224fde646d520eb90e08b82172abddb6397a248f4134997a5f21  tools/rerun.sh
2e3d54db954edd1b4c68ade83a0e532df52d1630c6b1fff06818b9521c7d0428  tools/runfams.sh
bdaa0d12dac45cc120a935af1e078b4330c7ea9dddaa6a14819c489e20e96014  tools/seq_diff.py
ded4ad138b893ad6088f344f56deeac01cd95b883958a1e6284554c232054de0  tools/small_checks.py
```

## 1　V1　修补修得对不对

### 1.1　第一轮五格的反例，修补后逐条核

结论：判对。第一轮判决第三节五格点名的反例（W1 花括号空份、W2 双引号里的 `$(` 收不了尾、W3 `2>` 与 `ln -s` 相对源、W4 `env =1`、W5 `bash -c --` 与三处偏严），加上 W5 里原来判对的 `time -p`、`--rcfile`，共 21 条，真 bash 当裁判，修补后 21 条全判对；修补前 7 条对、10 条漏拒、4 条误拒。

喂法：`rerun.sh` 的 `key` 族（进程内、真 bash 裁判），与 `keyfeeds.sh`（真钩子入口，下面贴它前 51 行）。

| # | 命令（重型闸、主 agent；有标注的是检出器 clerk） | 真 bash | 修补前 | 修补后 |
|---|---|---|---|---|
| 1 | `{cargo,} test --all` | 起 | 漏拒 | 对 |
| 2 | `{,} cargo test --all` | 起 | 漏拒 | 对 |
| 3 | `cargo {,test} --all` | 起 | 漏拒 | 对 |
| 4 | `FOO="$(pwd)" bash <<'EOF'` 里 `cargo test --all` | 起 | 漏拒 | 对 |
| 5 | `cd "$(pwd)" && bash <<'EOF'` 里 `cargo test --all` | 起 | 漏拒 | 对 |
| 6 | `x=$(bash <<'EOF'` 里 `cargo test --all`，下一行 `)` | 起 | 对 | 对 |
| 7 | `cat s.sh > d1.sh 2>/dev/null && bash d1.sh` | 起 | 漏拒 | 对 |
| 8 | `cat s.sh > d2.sh && bash d2.sh` | 起 | 对 | 对 |
| 9 | clerk：`cat …/detached-run.sh > /tmp/shw-dr-b.sh 2>/dev/null && bash /tmp/shw-dr-b.sh start …` | 起 | 漏拒 | 对 |
| 10 | clerk：同上，没有 `2>/dev/null` | 起 | 对 | 对 |
| 11 | `env =1 cargo test --all` | 起 | 漏拒 | 对 |
| 12 | `bash -c -- 'cargo test --all'` | 起 | 漏拒 | 对 |
| 13 | `ln -s ../s.sh sub/l.sh && bash sub/l.sh` | 起 | 漏拒 | 对 |
| 14 | `ln -s s.sh sub/l2.sh && bash sub/l2.sh`（链接悬空） | 不起 | 误拒 | 对 |
| 15 | `bash -s -- s.sh` | 不起 | 误拒 | 对 |
| 16 | `builtin cargo test --all` | 不起 | 误拒 | 对 |
| 17 | `function f { cargo test --all; }` | 不起 | 误拒 | 对 |
| 18 | `function f { cargo test --all; }; f` | 起 | 对 | 对 |
| 19 | `time -p cargo test --all` | 起 | 对 | 对 |
| 20 | `bash --rcfile light.sh s.sh` | 起 | 对 | 对 |
| 21 | `bash -c 'cargo test --all' arg0` | 起 | 对 | 对 |

`cd "$(git rev-parse --show-toplevel)" && …` 在沙盒里不是 git 仓，`cd ""` 失败后 `&&` 短路，裁判没法起；`key` 族换成同形状的 `cd "$(pwd)"`，真入口 `keyfeeds.sh` 仍是原写法（见下）。

`key` 族的原样输出（`out/p-key.txt` 的拼接：12 块各打一行统计，最后一行是合计）：

```
guard/main|裁判=2|前=0(漏拒)|后=2(对)  <==|'{cargo,} test --all'
guard/main|裁判=0|前=2(误拒)|后=0(对)  <==|'bash -s -- s.sh'
统计 (修补前判定,修补后判定): {'漏拒->对': 1, '误拒->对': 1} 语法错 0
guard/main|裁判=2|前=0(漏拒)|后=2(对)  <==|'{,} cargo test --all'
guard/main|裁判=0|前=2(误拒)|后=0(对)  <==|'builtin cargo test --all'
统计 (修补前判定,修补后判定): {'漏拒->对': 1, '误拒->对': 1} 语法错 0
guard/main|裁判=2|前=0(漏拒)|后=2(对)  <==|'cargo {,test} --all'
guard/main|裁判=0|前=2(误拒)|后=0(对)  <==|'function f { cargo test --all; }'
统计 (修补前判定,修补后判定): {'漏拒->对': 1, '误拒->对': 1} 语法错 0
guard/main|裁判=2|前=0(漏拒)|后=2(对)  <==|'FOO="$(pwd)" bash <<\'EOF\'\ncargo test --all\nEOF'
guard/main|裁判=2|前=2(对)|后=2(对)|'function f { cargo test --all; }; f'
统计 (修补前判定,修补后判定): {'对->对': 1, '漏拒->对': 1} 语法错 0
guard/main|裁判=2|前=0(漏拒)|后=2(对)  <==|'cd "$(pwd)" && bash <<\'EOF\'\ncargo test --all\nEOF'
guard/main|裁判=2|前=2(对)|后=2(对)|'time -p cargo test --all'
统计 (修补前判定,修补后判定): {'对->对': 1, '漏拒->对': 1} 语法错 0
guard/main|裁判=2|前=2(对)|后=2(对)|"x=$(bash <<'EOF'\ncargo test --all\nEOF\n)"
guard/main|裁判=2|前=2(对)|后=2(对)|'bash --rcfile light.sh s.sh'
统计 (修补前判定,修补后判定): {'对->对': 2} 语法错 0
guard/main|裁判=2|前=0(漏拒)|后=2(对)  <==|'cat s.sh > d1.sh 2>/dev/null && bash d1.sh'
guard/main|裁判=2|前=2(对)|后=2(对)|"bash -c 'cargo test --all' arg0"
统计 (修补前判定,修补后判定): {'对->对': 1, '漏拒->对': 1} 语法错 0
guard/main|裁判=2|前=2(对)|后=2(对)|'cat s.sh > d2.sh && bash d2.sh'
det/clerk|裁判=2|前=0(漏拒)|后=2(对)  <==|'cat scripts/hosts/detached-run.sh > /tmp/shw-dr-b.sh 2>/dev/null && bash /tmp/shw-dr-b.sh start --label m -- sleep 1'
统计 (修补前判定,修补后判定): {'对->对': 1, '漏拒->对': 1} 语法错 0
guard/main|裁判=2|前=0(漏拒)|后=2(对)  <==|'env =1 cargo test --all'
det/clerk|裁判=2|前=2(对)|后=2(对)|'cat scripts/hosts/detached-run.sh > /tmp/shw-dr-a.sh && bash /tmp/shw-dr-a.sh start --label m -- sleep 1'
统计 (修补前判定,修补后判定): {'对->对': 1, '漏拒->对': 1} 语法错 0
guard/main|裁判=2|前=0(漏拒)|后=2(对)  <==|"bash -c -- 'cargo test --all'"
统计 (修补前判定,修补后判定): {'漏拒->对': 1} 语法错 0
guard/main|裁判=2|前=0(漏拒)|后=2(对)  <==|'ln -s ../s.sh sub/l.sh && bash sub/l.sh'
统计 (修补前判定,修补后判定): {'漏拒->对': 1} 语法错 0
guard/main|裁判=0|前=2(误拒)|后=0(对)  <==|'ln -s s.sh sub/l2.sh && bash sub/l2.sh'
统计 (修补前判定,修补后判定): {'误拒->对': 1} 语法错 0
合计 {'对->对': 7, '漏拒->对': 10, '误拒->对': 4} 语法错 0 共 21
```

`keyfeeds.sh` 前 51 行（第一轮判决点名的 17 条，真入口喂改前、改后两棵树，原样）：

```
命令（guard/main）：{cargo,} test --all
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
命令（guard/main）：{,} cargo test --all
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
命令（guard/main）：cargo {,test} --all
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
命令（guard/implementation-writer）：{bash,} scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：cargo test 用 -p 点名 singlefs-checker-tier（singlefs-checker-tier；当前目录或清单认不出、或点名的不是认得出的工作区成员，照点名的包判；checker 档，默认只在
命令（guard/main）：FOO="$(pwd)" bash <<'EOF'\ncargo test --all\nEOF
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
命令（guard/main）：cd "$(git rev-parse --show-toplevel)" && bash <<'EOF'\ncargo test --all\nEOF
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
命令（det/main）：cd "$(git rev-parse --show-toplevel)" && bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF
  改前：rc=0
  改后：rc=2 ✗ 前台的等待循环没有超时（认出的循环：until grep -q x f）：等的条件不成立就一直不返回，这一次调用卡在这里期间，发给你的消息也送不到
命令（guard/main）：x=$(bash <<'EOF'\ncargo test --all\nEOF\n)
  改前：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
  改后：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
命令（guard/main）：cat s.sh > d1.sh 2>/dev/null && bash d1.sh
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：脚本 <work>/scratch/d1.sh:4 里的 cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=comm
命令（guard/main）：cat s.sh > d2.sh && bash d2.sh
  改前：rc=2 ✗ 重型测试被拒：脚本 <work>/scratch/d2.sh:4 里的 cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=comm
  改后：rc=2 ✗ 重型测试被拒：脚本 <work>/scratch/d2.sh:4 里的 cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=comm
命令（det/clerk）：cat scripts/hosts/detached-run.sh > /tmp/shw-dr-b.sh 2>/dev/null && bash /tmp/shw-dr-b.sh start --label m -- sleep 1
  改前：rc=0
  改后：rc=2 ✗ clerk 用 scripts/hosts/detached-run.sh 起或停脱离会话的长跑（认出的调用：/tmp/shw-dr-b.sh（这条命令里前面拷出来的 scripts/hosts/detached-run.sh） sta
命令（det/clerk）：cat scripts/hosts/detached-run.sh > /tmp/shw-dr-a.sh && bash /tmp/shw-dr-a.sh start --label m -- sleep 1
  改前：rc=2 ✗ clerk 用 scripts/hosts/detached-run.sh 起或停脱离会话的长跑（认出的调用：/tmp/shw-dr-a.sh（这条命令里前面拷出来的 scripts/hosts/detached-run.sh） sta
  改后：rc=2 ✗ clerk 用 scripts/hosts/detached-run.sh 起或停脱离会话的长跑（认出的调用：/tmp/shw-dr-a.sh（这条命令里前面拷出来的 scripts/hosts/detached-run.sh） sta
命令（guard/main）：env =1 cargo test --all
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
命令（guard/main）：bash -c -- 'cargo test --all'
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
命令（guard/main）：ln -s ../s.sh sub/l.sh && bash sub/l.sh
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：脚本 <work>/scratch/sub/l.sh:4 里的 cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=c
命令（guard/main）：ln -s s.sh sub/l2.sh && bash sub/l2.sh
  改前：rc=2 ✗ 重型测试被拒：脚本 <work>/scratch/sub/l2.sh:4 里的 cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=
  改后：rc=0
命令（guard/main）：bash -s -- s.sh
  改前：rc=2 ✗ 重型测试被拒：脚本 <work>/scratch/s.sh:4 里的 cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commi
  改后：rc=0
```

什么现象会推翻它：上面 21 条里任何一条在修补后的钩子上换了 rc；或在不同于上面的沙盒（比如 `scripts/hosts/` 下有真脚本）里同一条命令的判定变了。`rerun.sh` 的 `name=keyfeeds.txt_vs_model` 一行会红。

### 1.2　W1　花括号：空份、引号空串、补零序列

结论：判对。两处改法（展开出的空串去掉、整数序列补零）的行为与真 bash 逐词对得上；带出一处残留（下面最后一段），修补前后一样错，不是修补带进的。

改法与代码依据：

- 引号外展开出的空串去掉、引号写出的空串留着：「return [text for text, quoted_empty in expanded if text or quoted_empty or words_break_is_set("brace-empties-kept")]」（`.claude/hooks/lib_shell_words.py:663`）。引号写出的空串在切词时占一个位置，花括号展开之后据它判这一份是不是引号写出的空串：「piece, quoted = QUOTED_EMPTY, None」（`.claude/hooks/lib_shell_words.py:576`）。
- 补零宽度：「return max([len(text) for text in (first, last) if (len(text) > 1 and text[0] == "0") or (len(text) > 2 and text[:2] == "-0")], default=0)」（`.claude/hooks/lib_shell_words.py:713`），`lib_heavy_tests.py` 共用它：「width = shell_words.brace_sequence_width(first, last)」（`.claude/hooks/lib_heavy_tests.py:3322`）。

逐词对照真 bash（一次性手工喂，`set -f; set -- 词; echo $#; printf '[%s]' "$@"` 对 `shell_tokens`；系统化的版本是 `brace_diff2.py`，见下）：

```
$ {cargo,} test --all | bash: 3 个词 [cargo][test][--all] | 修补前: 4 个词 [cargo][][test][--all] | 修补后: 3 个词 [cargo][test][--all]
$ {,} | bash: 0 个词 [] | 修补前: 2 个词 [][] | 修补后: 0 个词 
$ ''{,} | bash: 2 个词 [][] | 修补前: 2 个词 [][] | 修补后: 2 个词 [][]
$ {"",} | bash: 1 个词 [] | 修补前: 2 个词 [][] | 修补后: 1 个词 []
$ {a,}{,b} | bash: 3 个词 [a][ab][b] | 修补前: 4 个词 [a][ab][][b] | 修补后: 3 个词 [a][ab][b]
$ x{,}y | bash: 2 个词 [xy][xy] | 修补前: 2 个词 [xy][xy] | 修补后: 2 个词 [xy][xy]
$ {01..03} | bash: 3 个词 [01][02][03] | 修补前: 3 个词 [1][2][3] | 修补后: 3 个词 [01][02][03]
$ {-01..2} | bash: 4 个词 [-01][000][001][002] | 修补前: 4 个词 [-1][0][1][2] | 修补后: 4 个词 [-01][000][001][002]
$ {1..-03} | bash: 5 个词 [001][000][-01][-02][-03] | 修补前: 5 个词 [1][0][-1][-2][-3] | 修补后: 5 个词 [001][000][-01][-02][-03]
$ {0..00} | bash: 1 个词 [00] | 修补前: 1 个词 [0] | 修补后: 1 个词 [00]
$ {1..05..2} | bash: 3 个词 [01][03][05] | 修补前: 3 个词 [1][3][5] | 修补后: 3 个词 [01][03][05]
```

系统化对照（`rerun.sh` 的 `totals.txt` 原样，前五行）：

```
brace_diff2 tree-orig seed1 Counter({'别的': 547, '只差空串': 16}) 共 3000 条（去重、bash 能跑的）
seq_diff tree-orig 共 1500 条，不同： {'shell_words': 1158, 'heavy_tests': 10}
brace_diff2 tree seed1 Counter() 共 3000 条（去重、bash 能跑的）
seq_diff tree 共 1500 条，不同： {'shell_words': 10, 'heavy_tests': 10}
brace_diff2 tree seed2 Counter() 共 3000 条（去重、bash 能跑的）
```

`brace_diff2.py` 随机造词（花括号、逗号、点、补零数字、引号空串、转义），每词交真 bash 与 `shell_tokens` 逐词比：修补前树 3000 条里 563 条不同（547 条别的、16 条只差空串），修补后树两个种子各 3000 条 0 条不同。修补前不同的头几条（`out/brace-diff-tree-orig-seed1.txt` 原样前四行）：

```
Counter({'别的': 547, '只差空串': 16}) 共 3000 条（去重、bash 能跑的）
["别的", "{3..01}-01..\\{", ["03-01..{", "02-01..{", "01-01..{"], ["3-01..{", "2-01..{", "1-01..{"]]
["别的", "{3..01}", ["03", "02", "01"], ["3", "2", "1"]]
["别的", "{{3..01}-01{,a,}}", ["{03-01}", "{03-01a}", "{03-01}", "{02-01}", "{02-01a}", "{02-01}", "{01-01}", "{01-01a}", "{01-01}"], ["{3-01}", "{3-01a}", "{3-01}", "{2-01}", "{2-01a}", "{2-01}", "{1-01}", "{1-01a}", "{1-01}"]]
```

引号空串放在各个位置的 `empties` 族 260 条（`cargo '' test --all`、`'' cargo test --all`、`{'',cargo} test --all`、`env '' cargo test --all`……）：256 条两棵树都对，4 条两棵树都误拒（`cargo test --all > ''`，bash 报「No such file or directory」不起命令；修补前就这样）。`brace` 族 227 条：181 条两棵树都对，33 条漏拒变对，1 条误拒变对；剩下的 12 条里 2 条修补后误拒是裁判够不着的（`sudo {,} cargo …` 沙盒里 sudo 要密码，`env {-i,} cargo …` 清了 PATH），10 条两棵树都误拒（`cp s.sh s{01..100..50}.sh` 展开成三个操作数、cp 报错什么都不写，`>{,}` 在 bash 里是「ambiguous redirect」不起命令、`for i in {,}` 空列表不进循环、`case x in {x,y})` 与 `X={cargo,}; $X …` 是文件头写明的偏严）。

「改前拒、改后放」的 12 条（第一轮判决之前的树拒、修补后的树放；用第一轮腿的 575 条跑出来，迁移数 `0->0 166 / 0->2 168 / 2->0 12 / 2->2 229` 与背景材料第二节主 agent 转述的读数逐数相同）：交真 bash 逐条判，12 条全是该放（`r1-2to0 合计 {'对->对': 12}`）。其中检出器 clerk 写范围的 2 条（`cp x scripts/hosts/y.sh{,}`、`cp x records/{,out}.md`）真 bash 不起重活、裁判没有标记，手工在临时目录里跑（未入复跑）：

```
$ cp x scripts/hosts/y.sh{,}
cp: target 'scripts/hosts/y.sh': No such file or directory
rc=1
$ cp x records/{,out}.md
cp: target 'records/out.md': No such file or directory
rc=1
```

目标不是目录，什么都没写，放行是对的。

残留（修补前后一样错）：整数序列带步长、一端写成 0 起头、另一端没有 0 起头而写得更长时，bash 补到两端写法里较长的那个宽度，代码只算「写了 0 起头」的那几边。`seq_diff.py` 1500 条随机序列：修补前树 `shell_words` 1158 条不同、`heavy_tests` 10 条不同；修补后树两边各剩 10 条（`totals.txt` 第二行、第四行），都是这一类，头几条（`out/seq-diff-tree-lines.txt` 原样）：

```
shell_words 不同 x{100..00..2}y ['x100y', 'x098y', 'x096y', 'x094y'] ['x100y', 'x98y', 'x96y', 'x94y']
heavy_tests 不同 x{100..00..2}y ['x100y', 'x098y', 'x096y', 'x094y'] ['x100y', 'x98y', 'x96y', 'x94y']
shell_words 不同 x{01..100..3}y ['x001y', 'x004y', 'x007y', 'x010y'] ['x01y', 'x04y', 'x07y', 'x10y']
heavy_tests 不同 x{01..100..3}y ['x001y', 'x004y', 'x007y', 'x010y'] ['x01y', 'x04y', 'x07y', 'x10y']
shell_words 不同 x{00..100..-2}y ['x000y', 'x002y', 'x004y', 'x006y'] ['x00y', 'x02y', 'x04y', 'x06y']
heavy_tests 不同 x{00..100..-2}y ['x000y', 'x002y', 'x004y', 'x006y'] ['x00y', 'x02y', 'x04y', 'x06y']
```

不带步长的同类写法（`{01..100}`）份数超过 64，钩子照原样留一个词（文件头写明的上限，`.claude/hooks/lib_shell_words.py:644`「BRACE_EXPANSION_LIMIT = 64」），不会判成别的名字。带步长又不超过 64 份的写法可以走到命令层，手工喂（未入复跑），`s.sh` 里是 `cargo test --all` 的探针目录：

```
命令：cp s.sh s{01..100..100}.sh && bash s001.sh
  改前：rc=0 
  改后：rc=0 
真 bash（临时目录里 s.sh 只有一行 echo SCRIPT-RAN）：SCRIPT-RAN，ls 得 s001.sh s.sh
```

bash 建的是 `s001.sh` 并起了它，钩子算出来的名字是 `s01.sh`，认不出，放了（漏拒）。几乎没人这样写。

什么现象会推翻它：`brace_diff2.py` 在修补后树上找到一个与 bash 词表不同的词；或上面 12 条里有一条真 bash 其实会起重活。

### 1.3　W2　heredoc 挂在没收尾的替换里：`open_substitution_tail` 改走同一套扫描

结论：判对。双引号里收了尾的 `$(…)` 不再被当成还开着，`cd "$(git rev-parse --show-toplevel)" && bash <<EOF` 这一类顶层 heredoc 现在照样当喂给 bash 的命令判；随机造的 3000 条 heredoc 前缀里，修补后唯一的错全是 ANSI-C 引号 `$'…\'…'`，那是修补前后都不认的写法。

改法与代码依据：`open_substitution_tail` 现在用「text.startswith("$(", position) or text.startswith(("<(", ">("), position)」（`.claude/hooks/lib_shell_words.py:431`）找替换的开头，再调 `command_substitution_end`（`.claude/hooks/lib_shell_words.py:518`）、`double_quote_end`（`.claude/hooks/lib_shell_words.py:498`）、`backquote_end`（`.claude/hooks/lib_shell_words.py:482`），它们就是 `shell_tokens` 切词用的那几个；单引号里照 bash 不认转义：「end = text.find("'", position + 1)」（`.claude/hooks/lib_shell_words.py:422`）；词首的 `#` 起到行尾是注释：「elif character == "#" and at_word_start:」（`.claude/hooks/lib_shell_words.py:435`）。

对照（`rerun.sh` 各族合计行，原样；格式是「修补前判定->修补后判定: 条数」，`语法错` 是真 bash 自己报语法错、不进统计的条数）：

```
heredoc 合计 {'对->对': 270, '对->误拒': 5, '漏拒->对': 148, '漏拒->漏拒': 5} 语法错 7 共 435
ansic 合计 {'对->对': 12, '漏拒->对': 4, '漏拒->漏拒': 19} 语法错 0 共 35
head1 合计 {'对->对': 2187, '对->漏拒': 1, '漏拒->对': 654, '漏拒->漏拒': 124, '误拒->误拒': 30} 语法错 4 共 3000
```

- `heredoc` 族 435 条（7 条真 bash 语法错）：148 条漏拒变对，270 条两棵树都对。5 条「对->误拒」全是 `echo "$(echo a)" || bash <<EOF`：`echo` 成功、`||` 短路，bash 根本不起，钩子静态判成起，不算钩子错。5 条两棵树都漏拒，全是 `echo "$(echo a # b⏎)"; bash <<EOF`：替换里的注释换了行，`<<` 所在那一行开头是 `)"; bash `，文件头写明「heredoc 挂的命令从上一行续行过来时认不出它喂给谁」，修补前后一样。
- `head1` 随机族：`fuzzhead.py` 把 47 种前缀（各式引号、转义、反引号、算术、`${…}`、`<(`、注释、函数定义、case）随机拼 1 到 4 个，后面接 `bash <<'EOF'` 或 `sh <<EOF` 之类，或把它放进没收尾的 `$(`、`` ` ``、`<(` 里。3000 条，4 条真 bash 语法错：2187 条两棵树都对，654 条漏拒变对，124 条两棵树都漏拒，30 条两棵树都误拒，1 条前对后漏拒。124 条加那 1 条（`out/flags.txt` 里 `[head1]` 的 125 行）逐行解析命令文本，全部含 ANSI-C 引号 `$'a\'b'`；30 条误拒全是 `echo a\⏎bash <<EOF`（反斜杠换行把 `bash` 接在 `echo` 后面，heredoc 挂在 echo 上，bash 没起），也是文件头写明的续行限制。
- `ansic` 族 35 条（`$'a\'b'`、`'a\'` 这类单引号尾部带反斜杠的写法，后面接顶层或没收尾的替换里的 heredoc）：4 条漏拒变对，全是普通单引号 `echo 'a\'; x=$(bash <<EOF`（bash 里 `'a\'` 是一个收了尾的串；修补前的代码把引号里的反斜杠当转义，错把收尾引号吃掉）；19 条两棵树都漏拒，全是 `$'a\'b'`（ANSI-C 引号里 `\'` 是转义，这里按普通单引号配对，后面全错位）；12 条两棵树都对。

前对后漏拒的那 1 条（修补前碰巧对）：

```
echo $'a\'b'; echo `echo '"'`; echo $(echo "$(sh <<EOF⏎cargo test --all⏎EOF⏎)")
```

修补前的代码在单引号里也跳过反斜杠后的字符，所以把 `$'a\'b'` 配对对了，同一个写法放在最简单的位置（`echo $'a\'b'; bash <<'EOF'`）修补前后都是漏拒（`ansic` 族与 `keyfeeds.txt` 里 `echo $'a\'b'; x=$(bash <<'EOF'…` 那一条）。

长命令的挂钟（`out/perf-reference-not-compared.txt`，随机器负载变，只看量级）：

```
perf 3000 个收尾的双引号替换后接 bash heredoc: tree-orig rc=0 0.72s | tree rc=2 3.17s
perf 300 层没收尾的 $(: tree-orig rc=0 0.17s | tree rc=0 0.21s
perf 300 层没收尾的双引号替换: tree-orig rc=0 0.09s | tree rc=0 0.08s
perf 500 个没配对的双引号: tree-orig rc=2 0.19s | tree rc=2 0.21s
perf 20 个花括号序列相乘: tree-orig rc=0 0.07s | tree rc=0 0.07s
perf 引号空串 800 个: tree-orig rc=2 0.10s | tree rc=2 0.10s
perf 花括号空份 40 层: tree-orig rc=2 0.08s | tree rc=2 0.09s
```

3000 个收尾的双引号替换后接 heredoc：修补前 0.72 秒不拒（碰巧，把它当成没收尾），修补后 3.17 秒拒；扫描是线性的，没有卡死。

什么现象会推翻它：`fuzzhead.py` 在不含 `$'…\'…'` 的前缀里找到一条修补后判错的；或 `[head1]` 的 125 行里有一行不含 ANSI-C 引号。

### 1.4　W3　`2>` 不当改标准输出、`ln -s` 相对源从链接所在目录算

结论：判对（点名的两处都改对了，第一轮判决点名的 `cat … > 拷贝 2>/dev/null` 与 `ln -s ../s.sh sub/l.sh` 修补后都拒，悬空的 `ln -s s.sh sub/l2.sh` 放）；带出两处没改全的地方：heredoc 写文件那一处消费方仍把 `2> 文件` 当写了 heredoc 正文（误拒，修补前后一样），和修补自己带进的一类漏拒（数字 fd 的几种写法，列在 2.1，不在这里重复）。

改法与代码依据：

- 重定向前的 fd 号挂在运算符上：「operator = RedirectOperator(operator, word)」（`.claude/hooks/lib_shell_words.py:632`），挂的条件是「if operator[0] in "<>" and word is not None and word.isdigit():」（`.claude/hooks/lib_shell_words.py:630`）。
- 改标准输出的判法：「return operator in OUTPUT_REDIRECTS and operator != ">&" and getattr(operator, "fd", None) in (None, "1")」（`.claude/hooks/lib_shell_words.py:328`）。两处用它：重型闸 `cat 源 > 目的` 那一条「targets = [target for operator, target in command.output_redirections if operator in (">", ">|") and shell_words.redirects_standard_output(operator)]」（`.claude/hooks/heavy-test-guard.sh:618`），检出器同类「if operator in DETACHED_RUN_CAT_COPY_REDIRECTS and shell_words.redirects_standard_output(operator)]」（`.claude/hooks/bash-command-detector.sh:1118`）。
- `ln -s` 的相对源：`link_source_word`（`.claude/hooks/lib_shell_words.py:1113`）最后一句「return os.path.join(os.path.dirname(link), source)」（`.claude/hooks/lib_shell_words.py:1128`）；带 `-r`、绝对路径、`~`、`$` 的原样交回。

真 bash 的行为：`ln -s ../s.sh sub/l.sh` 存进链接的是字面 `../s.sh`，读它时从 `sub/` 往上走，指到 `s.sh`；`ln -s s.sh sub/l2.sh` 存的 `s.sh` 从 `sub/` 里找，没有，链接悬空，`bash sub/l2.sh` 起不来；`ln -sr` 由 ln 自己算成从链接所在目录出发的相对路径，源按当前目录写。`cat s.sh 2>err > d.sh` 里只有 `>` 改标准输出；`1>` 同；`3>` 改的是 3 号，不是标准输出。

各族合计（`rerun.sh` 的 `totals.txt` 原样）：

```
cpforms 合计 {'对->对': 49, '误拒->误拒': 1} 语法错 0 共 50
redirect 合计 {'对->对': 196, '对->漏拒': 15, '对->误拒': 7, '漏拒->对': 34, '漏拒->漏拒': 73, '误拒->对': 79, '误拒->误拒': 16} 语法错 0 共 420
ln 合计 {'对->对': 276, '对->误拒': 13, '漏拒->对': 80, '漏拒->漏拒': 21, '误拒->对': 56, '误拒->误拒': 84} 语法错 0 共 530
hwrite 合计 {'对->对': 12, '漏拒->漏拒': 2, '误拒->误拒': 8} 语法错 0 共 22
```

- `redirect` 族 420 条（`cat 源 <重定向> d.sh && bash d.sh`，重定向换了三十来种写法，`2>/dev/null` 放前放后，`2>e.sh` 与 `> d.sh` 谁在前）：修补点本身是 34 条漏拒变对（`2>` 排在 `>` 后面把拷贝目标顶掉）加 79 条误拒变对（`2> d.sh` 被当成拷到 d.sh）；196 条两棵树都对。7 条「对->误拒」都是裁判的运行时短路：`cat X 2 > d.sh …`、`cat X a2> d.sh …`、`cat X ''2> d.sh …` 里 `2`、`a2`、`''2` 是 cat 的参数，cat 找不到这个文件退 1，`&&` 之后的 bash 不跑，钩子静态判成起；还有 `cat X >d.sh 2>/dev/null || bash d.sh` 的 `||` 短路。15 条「对->漏拒」是修补带进的，列在 2.1。
- 73 条两棵树都漏拒，是拷出写法本来就不认的（与修补无关，给主 agent 备查，每类一条原样，真 bash 都会起 `d.sh`）：`cat s.sh >> d.sh && bash d.sh`、`cat s.sh &> d.sh && bash d.sh`、`cat s.sh >& d.sh && bash d.sh`、`cat > d.sh < s.sh && bash d.sh`（源走标准输入）、`cat s.sh | tee d.sh >/dev/null && bash d.sh`、`sed 's/x/x/' s.sh > d.sh 2>/dev/null && bash d.sh`、`exec 3>d.sh; cat s.sh >&3; bash d.sh`。`>&` 在 `redirects_standard_output` 里被整个排除，是因为 `>&2` 是复制文件描述符；但 `>& 文件名` 在 bash 里与 `&> 文件名` 一样，把标准输出和标准错误都写进文件。
- `ln` 族 530 条：检出器一侧的 84 条两棵树都误拒，是检出器按名字认 `detached-run.sh`，源不存在的链接（`ln -sr ../scripts/…`、硬链接的源写错）也拒，偏严，修补前后一样。重型闸一侧：13 条「对->误拒」全是 `ln -s ../s.sh sub/x/../l.sh`（沙盒里没有 `sub/x`，ln 失败；钩子把路径折成 `sub/l.sh`），21 条两棵树都漏拒，是 `ln -s $PWD/s.sh sub/l.sh`（变量不展开，文件头写明）和 `ln -st sub ../s.sh`（合写的 `-st` 不认）。
- `cpforms` 族 50 条（`cp`、`mv`、`install`、`ln` 的 `-t`、`-T`、多个源、`--` 各种写法；`copy_pairs` 被改写过，每条路径都走一遍）：49 条两棵树都对，1 条是 `cp -t -- sub …`（bash 报「target directory '--'」）两棵树都误拒。

没跟着改的那一处消费方：重型闸里「同一条命令里先由 heredoc 或 tee 写出文件、再执行它」认写出的文件时，把这一条命令上所有输出重定向的目标都当成写了它的输入：「for operator, target in command.output_redirections:」（`.claude/hooks/heavy-test-guard.sh:603`）、「targets.append((target, operator in (">>", "&>>")))」（`.claude/hooks/heavy-test-guard.sh:606`），不看 fd 号。`hwrite` 族 22 条：12 条对；8 条两棵树都误拒（`cat <<'EOS' 2>d.sh` 之后 `bash d.sh`：heredoc 进的是标准输入、`cat` 写到标准输出，`2>d.sh` 里只有标准错误，d.sh 是空的，bash 不起任何东西；`3>d.sh`、`4>d.sh >e.sh` 同理）；2 条两棵树都漏拒（`sh -c 'cat >d.sh' <<'EOS'` 与 `cat <<'EOS' | tee d.sh >/dev/null`，heredoc 经 sh -c 或管道写文件不认，原来就这样）。原样（`out/flags.txt` 的 `[hwrite]` 行，去掉族名前缀）：

```
guard/main|裁判=0|前=2(误拒)|后=2(误拒)  <==|"cat <<'EOS' > d.sh 2>e.sh\ncargo test --all\nEOS\nbash e.sh"
guard/main|裁判=0|前=2(误拒)|后=2(误拒)  <==|"cat <<'EOS' 2>e.sh > d.sh\ncargo test --all\nEOS\nbash e.sh"
guard/main|裁判=0|前=2(误拒)|后=2(误拒)  <==|"tee d.sh 2>e.sh <<'EOS' >/dev/null\ncargo test --all\nEOS\nbash e.sh"
guard/main|裁判=0|前=2(误拒)|后=2(误拒)  <==|"cat <<'EOS' 2>d.sh\ncargo test --all\nEOS\nbash d.sh"
guard/main|裁判=2|前=0(漏拒)|后=0(漏拒)  <==|"sh -c 'cat >d.sh' <<'EOS'\ncargo test --all\nEOS\nbash d.sh"
guard/main|裁判=2|前=0(漏拒)|后=0(漏拒)  <==|"cat <<'EOS' | tee d.sh >/dev/null\ncargo test --all\nEOS\nbash d.sh"
guard/main|裁判=0|前=2(误拒)|后=2(误拒)  <==|"cat <<'EOS' 3>d.sh\ncargo test --all\nEOS\nbash d.sh"
guard/main|裁判=0|前=2(误拒)|后=2(误拒)  <==|"cat <<'EOS' 4>d.sh >e.sh\ncargo test --all\nEOS\nbash d.sh"
guard/main|裁判=0|前=2(误拒)|后=2(误拒)  <==|"cat <<'EOS' >&2 2>d.sh\ncargo test --all\nEOS\nbash d.sh"|bash:cargo test --all
guard/main|裁判=0|前=2(误拒)|后=2(误拒)  <==|"cat 2>d.sh <<'EOS'\ncargo test --all\nEOS\nbash d.sh"
```

后果只是误拒（把 `2>` 的文件当脚本读、里面有 `cargo test --all` 就拒），没有漏拒；写 `cat <<EOF 2>err.log` 再去 `bash err.log` 的人几乎不存在，所以不急，但「fd 号挂在运算符上」这件事只改了拷贝那一处，这里是同一类根因的第二个消费方。

什么现象会推翻它：重型闸里 `cat s.sh > d1.sh 2>/dev/null && bash d1.sh` 或 `ln -s ../s.sh sub/l.sh && bash sub/l.sh` 换了 rc；或 `ln -s s.sh sub/l2.sh && bash sub/l2.sh` 又被拒（`keyfeeds.txt` 对应三条）。

### 1.5　W4　`env` 后面开头是 `=` 的词当赋值

结论：判对。GNU env 把带 `=` 的词（含开头就是 `=` 的）都当 NAME=VALUE 设进环境，不是命令词，修补照这个改。

代码依据：「elif (name == "env" and "=" in argument[1 if words_break_is_set("env-leading-equals-as-command") else 0:]」（`.claude/hooks/lib_shell_words.py:1001`）。

真 env（本机 GNU coreutils 9.4，原样）：

```
$ env =1 echo hi
hi
rc=0
$ env = echo hi
hi
rc=0
$ env -- =1 echo hi
hi
rc=0
$ env =1 -u X echo hi
env: ‘-u’: No such file or directory
rc=127
$ env A=1 -u X echo hi
env: ‘-u’: No such file or directory
rc=127
$ env -u =1 echo hi
env: cannot unset ‘=1’: Invalid argument
rc=125
$ env -C =1 echo hi
env: cannot change directory to '=1': No such file or directory
rc=125
```

`env` 族 494 条（`env`、`env A=1`、`env -u X`、`env --`、`env -C .`、`env -i PATH="$PATH"` 后面接 `=1`、`=`、`==`、`=a=b`、`"=1"`、`\=1`、`=""`、`A=`、`a-b=1`、`1a=1`……再接 `cargo test --all`，另有 `env -S '=1 …'`、嵌在 `time`、`nice`、`timeout`、`bash -c` 里的）：

```
env 合计 {'对->对': 155, '对->误拒': 9, '漏拒->对': 316, '漏拒->漏拒': 2, '误拒->误拒': 12} 语法错 0 共 494
```

316 条漏拒变对（修补点）；155 条两边都对；2 条两边都漏拒（`xargs env =1 cargo test --all`，`xargs` 起命令本来就不认）。9 条「对->误拒」：6 条是 `env =1 -u X …`、`env =1 -i …`、`env =1 -- …` 三种写法各在重型闸与检出器两边出一条（`=1` 之后的 `-u`、`-i`、`--` 在真 env 里是命令词，env 退 127，不起命令；这一类放在 2.1）；1 条 `env -iS '=1 cargo …'`（清了 PATH，沙盒里找不到假 cargo）；2 条 `sudo env =1 …`（重型闸、检出器各一，沙盒里 sudo 要密码）。12 条两边都误拒：`env -u =1 cargo …`、`env -C =1 cargo …`、`env --chdir==1 …`、`env --unset==1 …`、`env -u=1`、`env -C=1`（`=1` 是选项的值，env 报错退 125，不起命令；修补前就拒）。

同类的修补前就有：`env A=1 -u X cargo test --all` 修补前后都拒（`keyfeeds.txt` 里 `env A=1 -u X` 一条），真 env 退 127。`=1` 现在走同一条路，所以这一类在 `=1` 上也有了，不是新的一类。

什么现象会推翻它：`env =1 cargo test --all` 在修补后的钩子上放行；或者换一个 coreutils 版本后 `env =1 echo hi` 不再起命令。

### 1.6　W5　`bash -c` 之后接着认选项、`builtin` 后不是内建放行、只定义不调用的 `function` 放行、`bash -s --` 后的词是位置参数

结论：判对，带两处与 bash 对不上的细节：`-c` 之后的长选项 bash 其实报错（见下，修补后误拒）；`function` 放行的条件在函数名拼写拆开时漏拒（2.1）。

#### `bash -c` 之后接着认选项

代码依据：`shell_invocation`（`.claude/hooks/lib_shell_words.py:1017`）里「takes_code = True」（`.claude/hooks/lib_shell_words.py:1041`），循环走完选项之后取下一个词当代码。

真 bash（本机 5.2.21，手工喂，原样；`-s` 那一条是顺带的）：

```
$ bash -c -- 'echo hi'
hi
rc=0
$ bash -c -x 'echo hi'
+ echo hi
hi
rc=0
$ bash -c -o pipefail 'echo hi'
hi
rc=0
$ bash -c -O extglob 'echo hi'
hi
rc=0
$ bash -c --posix 'echo hi'
bash: --: invalid option
Usage:	bash [GNU long option] [option] ...
	bash [GNU long option] [option] script-file ...
rc=2
$ bash -c -n 'echo hi'
rc=0
$ bash -n -c 'echo hi'
rc=0
$ bash --posix -c 'echo hi'
hi
rc=0
$ bash -s -- foo <<< 'echo $1'
foo
rc=0
```

bash 的长选项必须写在 `-c` 之前，`-c` 之后只认单字母选项和 `-o`、`-O` 的值；`-n` 只查语法、什么都不起。

`shell` 族 946 条（`bash`、`sh`、`env bash` 加 45 种选项排列，代码后接 0 到 2 个位置参数，另有 `-s` 的各种写法）：

```
shell 合计 {'对->对': 390, '对->误拒': 188, '漏拒->对': 266, '漏拒->漏拒': 11, '误拒->对': 17, '误拒->误拒': 74} 语法错 0 共 946
```

266 条漏拒变对（`bash -c -- '…'`、`-c -x`、`-c -e --`、`-c -o pipefail`、`-c -O extglob` 之后的代码都认出来了）；390 条两边都对。188 条「对->误拒」逐条分过类：162 条是 `bash -c --长选项 '代码'`（`--noprofile`、`--posix`、`--norc`、`--rcfile 文件`、`--debugger`、`--help`、`--login`、`--version`；`sh -c --…` 同），18 条是 `bash -c -n '代码'`、`bash -cn '代码'`，8 条是 `sh -c -o pipefail …` / `sh -c +o pipefail …`（本机 `sh` 是 dash，不认 pipefail，报错不起；换成 bash 是该拒的，裁判局限，不算钩子错）。前两类：修补前把紧跟 `-c` 的那个词（长选项本身）当代码，碰巧不拒；修补后把长选项也当选项跳过，再取下一个词当代码，拒。bash 对前一类报错不起，对后一类只查语法不起，所以是误拒；写这种命令的人不存在，但文件头那句「-c 之后照 bash 接着认选项」（`.claude/hooks/lib_shell_words.py:14`）与 bash 不完全一致：bash 只在 `-c` 后认短选项。例（原样，`out/flags.txt` `[shell]` 行）：

```
guard/main|裁判=0|前=0(对)|后=2(误拒)  <==|"bash -c --noprofile 'cargo test --all'"|bash:bash: --: invalid option
guard/main|裁判=0|前=0(对)|后=2(误拒)  <==|"bash -c --posix 'cargo test --all'"|bash:bash: --: invalid option
guard/main|裁判=0|前=0(对)|后=2(误拒)  <==|"env bash -c -n 'cargo test --all'"
guard/main|裁判=0|前=0(对)|后=2(误拒)  <==|"env bash -c --noprofile 'cargo test --all'"|bash:bash: --: invalid option
guard/main|裁判=0|前=0(对)|后=2(误拒)  <==|"env bash -c --posix 'cargo test --all'"|bash:bash: --: invalid option
```

74 条两边都误拒：`bash -n -c '…'`、`bash -cn '…'`（`-n` 不起命令，修补前后都拒，文件头已写明 `bash -n 脚本` 的处理没有覆盖 `-n -c`）与 `bash +s s.sh`。11 条两边都漏拒：标准输入来自文件或管道里的 `cat`（`bash < s.sh`、`bash -s < s.sh`、`cat s.sh | bash -s`、`bash <(cat s.sh)`、`source <(cat s.sh)`）和 `bash -c -- "$(echo '…')"`（代码是命令替换，看不见），文件头写明判不了的，修补前后一样。

#### `bash -s -- 词`：后面的词是位置参数

代码依据：「reads_standard_input = reads_standard_input or (argument.startswith("-") and "s" in letters)」（`.claude/hooks/lib_shell_words.py:1037`）。真 bash：`-s` 从标准输入读命令，后面的词是 `$1…`，不是脚本。17 条误拒变对（`bash -s -- s.sh`、`bash -s s.sh`、`bash -xs -- s.sh`、`bash -O extglob -s s.sh`、`sh -s s.sh` 等）；`shell` 族里没有一条「前对后漏拒」，`-s` 后接命令替换、heredoc、here-string、`echo '…' | bash -s -- foo` 都照拒（`shell` 族里这几类全在两边都对的 390 条里）。

#### `builtin` 后面不是内建

代码依据：内建表「BASH_BUILTINS = frozenset(". : [ alias bg bind …」（`.claude/hooks/lib_shell_words.py:331`），放行那一句「return PrefixSkip(None, prefix_names, assignments, chdir)   # `builtin cargo …`」（`.claude/hooks/lib_shell_words.py:956`）。表与本机 `bash -c 'compgen -b'` 逐个比（`rerun.sh` 末尾 `small_checks`）：

```
small_checks name=builtin_table table=61 compgen_b=61 only_in_table=[] only_in_bash=[]
small_checks name=versions GNU bash, version 5.2.21(1)-release (x86_64-pc-linux-gnu) | env (GNU coreutils) 9.4
```

`builtin` 族 451 条（`builtin` 后面接 80 来个词，含每个内建、`cargo`、`time`、关键字，带 `--`，嵌 `command`、`exec`、`eval`、`source`、`builtin builtin`、替换、通配、引号、花括号）：

```
builtin 合计 {'对->对': 361, '漏拒->漏拒': 2, '误拒->对': 76, '误拒->误拒': 12} 语法错 0 共 451
```

76 条误拒变对，361 条两边都对，没有一条前对后漏拒。12 条两边都误拒：检出器一侧 `builtin source|.|kill|disown bash scripts/hosts/detached-run.sh …`（模板把整条命令当这几个内建的参数，bash 报错不起；`builtin source 脚本` 里的脚本钩子照读，偏严）、`X=cargo; builtin $X test --all`（`$` 起头的词不放，文件头写明）、`builtin command -p cargo …`（沙盒里 `command -p` 用默认 PATH 找不到假 cargo）和两条 `builtin cargo … && …`（`&&` 短路）。2 条两边都漏拒：`builtin trap 'cargo test --all' EXIT`（trap 的串不判，原来就这样）。

#### 只定义不调用的 `function`

见 2.2：这一类的放行范围我专门造了 588 + 2378 条去撞，发现一处漏拒（函数名拼写拆开），列在 2.1。

## 2　V2　修补带进的新错判

做法：把修补改到的每一处（花括号空份与补零、`open_substitution_tail`、fd 号挂运算符、`link_source_word`、`env` 的 `=`、`shell_invocation`、`builtin`、只定义不调用的 `function`、`copy_pairs`）各造一族命令，真 bash 当裁判，修补前后两棵树各判一遍，只看「前对后错」与「后仍错」；再加两份随机组合（`fuzz2.py` 的命令组合、`fuzzhead.py` 的 heredoc 前缀）撞没想到的地方。上面第 1 节各族的「前对后错」格在这里汇总。

### 2.1　修补之后判错、原来判对的写法

结论：判错（反例），共四类，全是冷门写法，没有一类出在第一轮判决点名的写法上。逐类给反例、真 bash 行为、代码依据、条数。

追加到 `keyfeeds.sh` 的真入口喂法（`out/keyfeeds.txt` 第 52 行起原样，改前 = 修补前，改后 = 修补后）：

```
命令（guard/main）：function foo { cargo test --all; }; "fo"o
  改前：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
  改后：rc=0
命令（guard/main）：function foo { cargo test --all; }; fo{o,}
  改前：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
  改后：rc=0
命令（guard/main）：function foo { cargo test --all; }; foo
  改前：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
  改后：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
命令（guard/main）：cat s.sh 01> d.sh && bash d.sh
  改前：rc=2 ✗ 重型测试被拒：脚本 <work>/scratch/d.sh:4 里的 cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commi
  改后：rc=0
命令（guard/main）：cat s.sh "2"> d.sh ; bash d.sh
  改前：rc=2 ✗ 重型测试被拒：脚本 <work>/scratch/d.sh:4 里的 cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commi
  改后：rc=0
命令（guard/main）：cat s.sh 3>d.sh >&3; bash d.sh
  改前：rc=2 ✗ 重型测试被拒：脚本 <work>/scratch/d.sh:4 里的 cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commi
  改后：rc=0
命令（guard/main）：bash -c --posix 'cargo test --all'
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
命令（guard/main）：bash -c -n 'cargo test --all'
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
命令（guard/main）：env =1 -u X cargo test --all
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
命令（guard/main）：env A=1 -u X cargo test --all
  改前：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
  改后：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
命令（guard/main）：cat <<'EOS' 2>d.sh\ncargo test --all\nEOS\nbash d.sh
  改前：rc=2 ✗ 重型测试被拒：脚本 <work>/scratch/d.sh:1 里的 cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commi
  改后：rc=2 ✗ 重型测试被拒：脚本 <work>/scratch/d.sh:1 里的 cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commi
命令（guard/main）：cp s.sh s{01..100..50}.sh && bash s001.sh
  改前：rc=0
  改后：rc=0
命令（guard/main）：echo $'a\'b'; x=$(bash <<'EOF'\ncargo test --all\nEOF\n)
  改前：rc=0
  改后：rc=0
命令（guard/main）：cat s.sh >> d.sh && bash d.sh
  改前：rc=0
  改后：rc=0
命令（guard/main）：cat s.sh &> d.sh && bash d.sh
  改前：rc=0
  改后：rc=0
```

#### 函数名拼写拆开的调用：漏拒（W5 新放行带进）

反例：`function foo { cargo test --all; }; "fo"o`、`… ; fo{o,}`。真 bash 把 `"fo"o` 与 `fo{o,}` 展开成 `foo`，调用函数，起 `cargo test --all`；修补前拒，修补后放。

代码依据：放行只在函数名在整段文本里字面只出现一次时成立：「if len(re.findall(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"(?![A-Za-z0-9_])", text)) != 1:」（`.claude/hooks/lib_shell_words.py:1479`）；函数体之外的词只查有没有 `$`、反引号、通配和「eval、source、trap」这类词：「if kind is None or (kind is False and (set("$`*?[") & set(value) or os.path.basename(value) in FUNCTION_CALLING_WORDS)):」（`.claude/hooks/lib_shell_words.py:1482`）。引号拆开、花括号拆开、反斜杠拆开的调用，字面里没有 `foo`，也没有被查的字符，放行。

数据（`fncall` 族，588 条：函数名 `foo`，49 种调用写法 × 6 种定义形状 × 重型闸与检出器两边）：

```
fncall 合计 {'对->对': 358, '对->漏拒': 110, '误拒->误拒': 120} 语法错 0 共 588
```

110 条前对后漏拒，重型闸 55 条、检出器 55 条（11 种调用写法 × 5 种定义形状；不带 `function` 关键字的 `foo() { … }` 一向不放行，所以那一种定义形状不漏），调用写法共 11 种：`fo{o,}`、`f{o,}o`、`{fo,}o`、`fo{o,o}`、`"fo"o`、`f""oo`、`f'o'o`、`'f'oo`、`fo''o`、`fo\o`、`f\⏎o\⏎o`（反斜杠换行续行）。原样（`out/flags.txt` 的 `[fncall]` 行前 10 条）：

```
guard/main|裁判=2|前=2(对)|后=0(漏拒)  <==|'function foo { cargo test --all; }; {fo,}o'
guard/main|裁判=2|前=2(对)|后=0(漏拒)  <==|'function foo { cargo test --all; }; fo{o,o}'
guard/main|裁判=2|前=2(对)|后=0(漏拒)  <==|"function foo { cargo test --all; }; fo''o"
guard/main|裁判=2|前=2(对)|后=0(漏拒)  <==|'function foo { cargo test --all; }; fo\\o'
guard/main|裁判=2|前=2(对)|后=0(漏拒)  <==|"function foo { cargo test --all; }; 'f'oo"
guard/main|裁判=2|前=2(对)|后=0(漏拒)  <==|'function foo() { cargo test --all; }; {fo,}o'
guard/main|裁判=2|前=2(对)|后=0(漏拒)  <==|'function foo() { cargo test --all; }; fo{o,o}'
guard/main|裁判=2|前=2(对)|后=0(漏拒)  <==|"function foo() { cargo test --all; }; fo''o"
guard/main|裁判=2|前=2(对)|后=0(漏拒)  <==|'function foo() { cargo test --all; }; fo\\o'
guard/main|裁判=2|前=2(对)|后=0(漏拒)  <==|"function foo() { cargo test --all; }; 'f'oo"
```

范围：别的写法都不漏——同一族里 `foo`、`"foo"`、`'foo'`、`\foo`、`$'foo'`、`${X:-foo}`、`$(echo foo)`、`` `echo foo` ``、`eval foo`、`command foo`、`time foo`、`foo &`、管道两侧、`FOO=1 foo` 等 38 种调用都照拒；`function` 族 2378 条（被调用的 61 种方式：`f`、`$(f)`、`export -f f; bash -c f`、`trap f EXIT`、`alias`、`set -a`、`PROMPT_COMMAND=f`、`command_not_found_handle`、`declare -fx` 等）没有一条前对后漏拒。意思是代码里「点名字才算」这条设计是立得住的，口子只在「名字不点出来，bash 展开后才变成它」。要利用它得有意把函数名拆开写；日常写法碰不到。

#### 数字紧贴 `>` 但不是 fd 的写法：漏拒（W3 新认法带进）

反例（真 bash 都会把 `s.sh` 拷进 `d.sh`、再起它；修补前拒，修补后放）：

- `cat s.sh 01> d.sh && bash d.sh`：bash 把 `01` 当 1 号 fd；代码只认字面 `"1"`（`.claude/hooks/lib_shell_words.py:328` 的 `in (None, "1")`）。
- `cat s.sh "2"> d.sh ; bash d.sh`、`cat s.sh \2> d.sh ; bash d.sh`、`cat s.sh ''2> d.sh ; bash d.sh`：引号或转义过的数字在 bash 里是 cat 的参数（cat 报「2: No such file」，但标准输出照样进 d.sh），不是 fd；代码看到词里是纯数字（「if operator[0] in "<>" and word is not None and word.isdigit():」，`.claude/hooks/lib_shell_words.py:630`）就当 fd 2，不算改标准输出；`''2` 则是切词多了一个占位字符后数字不再被当 fd，`cat` 带了两个源，「只一个源」的拷贝判法认不出。
- `cat s.sh 3>d.sh >&3; bash d.sh`：把 3 号复制到标准输出，d.sh 拿到拷贝；`>&` 整个被排除。

条数：`redirect` 族 420 条里 15 条前对后漏拒（重型闸 8、检出器 7），就是这五种写法各一条加变体；原样见上面 `keyfeeds.txt` 的三条和 `out/flags.txt` 的 `[redirect]` 行。成因：修补前代码是「把数字丢掉、剩下的算改标准输出」，对 `01>`、引号数字、复制 fd 碰巧都对。

#### `bash -c` 后面接长选项或 `-n`：误拒（W5 新认法带进）

见 1.6。180 条：162 条 `bash -c --posix '代码'` 一类（bash 报 `invalid option`），18 条 `bash -c -n '代码'`（只查语法）；另 8 条 dash 的 pipefail 是裁判局限。修补前把长选项本身当代码，碰巧放。

#### `env =1` 后面再接选项：误拒（W4 修补带进，同类修补前就有）

`env =1 -u X cargo test --all`、`env =1 -i cargo test --all`、`env =1 -- cargo test --all`：真 env 把 `=1` 之后的 `-u`、`-i`、`--` 当命令词，退 127、不起 cargo；钩子照选项继续解析，拒。6 条（三种写法 × 重型闸与检出器两边）。同一写法 `env A=1 -u X …` 修补前后都拒，所以这一类不是修补发明的，只是 `=1` 现在也走进去了。

#### 补充　一条随机撞出来的

`fuzzhead.py` 3000 条里 1 条前对后漏拒，命令含 `$'a\'b'` 后面接嵌套的没收尾替换里的 heredoc（1.3 末尾原样）。修补前碰巧对，是 ANSI-C 引号两棵树都不认的那一类的又一个位置。

什么现象会推翻它：函数名拼写拆开那一条 `function foo { cargo test --all; }; "fo"o` 在修补后的钩子上拒；数字紧贴 `>` 那一条 `cat s.sh 01> d.sh && bash d.sh` 在修补后的钩子上拒。

### 2.2　W5 新放行的三类，会不会把该拒的一起放了

结论：三类里有一类放掉了该拒的（`function`，函数名拼写拆开，见 2.1 的「函数名拼写拆开的调用」）；`builtin` 和 `bash -s --` 两类没放掉。

- `builtin`：放行条件是后面那个词不在内建表里、又不含 `$`、反引号、`*`、`?`、`[`。`builtin` 族 451 条没有一条前对后漏拒。族里专门造了「后面那个词在表里、而它自己再起命令」的写法（`builtin command cargo test --all`、`builtin exec …`、`builtin eval '…'`、`builtin source 脚本`、`builtin builtin command …`、`builtin cd . ; builtin …`）和「词要展开了才知道是什么」的写法（`builtin $(echo cargo) …`、`builtin ${X:-cargo} …`、`builtin car* …`、`builtin {echo,cargo} …`、`builtin cargo test "$(cargo test --all)"`），都照拒或照放得对。内建表与本机 bash 完全一致（上面 `small_checks` 一行：61 对 61，两边都没有多的）。表是 bash 5.2 的 `compgen -b`；`enable -f` 载进来的内建、bash 5.3 新增的内建不认，文件头已写明。
- `bash -s -- 词`：`shell` 族 946 条（`-s` 与 `-c`、`-n`、`-O`、`--rcfile`、`--`、`-` 的各种次序，heredoc、here-string、管道、命令替换各喂一遍）没有一条前对后漏拒。`bash -s` 后面的位置参数里带命令替换（`bash -s -- "$(cargo test --all)"`）照拒，命令替换是另判的。
- `function`：`fncall` 族 588 条加 `function` 族 2378 条（定义写法 4 种：`function f { …; }`、`function f() { …; }`、`f() { …; }`、换行写；被调用方式 61 种）。放行的口子只在「名字不点出来」（2.1「函数名拼写拆开的调用」）。同一族里两类偏严是文件头写明的：只在命令文本这一层放行，`bash -c '…'`、heredoc、命令替换里的函数定义照判（`fuzz2` 里 26 条「前对后误拒」基本都是这类：函数定义在 `bash -c`、heredoc 或 `$(…)` 里，函数体里是修补后才认得出的 `env =1`、`{cargo,}`、`env A=1 =2` 写法，真 bash 里函数没被调到）；`function` 族的 174 条「前对后误拒」全是函数体里写 `env =1 cargo test --all` 而被 `echo f`、`declare -f f`、`type f` 之类的写法点了名（名字出现不止一次就按偏严处理，函数体照判；修补前这一句 `env =1` 本来就认不出，所以显得「原来对」）。

### 2.3　这轮读到、但修补前后一样错的（不是修补带进的，给主 agent 备查）

| 类 | 例 | 真 bash | 两棵树 | 条数与出处 |
|---|---|---|---|---|
| ANSI-C 引号里的 `\'` | `echo $'a\'b'; bash <<'EOF'` 里 `cargo test --all` | 起 | 都漏拒 | `ansic` 族 19 条；`head1` 族 124 条（`[head1]` 的 125 行全含它） |
| 拷出再起的写法不认 | `cat s.sh >> d.sh`、`cat s.sh &> d.sh`、`cat s.sh >& d.sh`、`cat > d.sh < s.sh`、`cat s.sh \| tee d.sh >/dev/null`、`sed … s.sh > d.sh`、`exec 3>d.sh; cat s.sh >&3` 后面接 `bash d.sh` | 起 | 都漏拒 | `redirect` 族 73 条 |
| heredoc 写文件的消费方不看 fd 号 | `cat <<'EOS' 2>d.sh` 之后 `bash d.sh` | 不起 | 都误拒 | `hwrite` 族 8 条，1.4 |
| `ln` 的 `-st` 合写、`$PWD` | `ln -st sub ../s.sh && bash sub/s.sh`、`ln -s $PWD/s.sh sub/l.sh && bash sub/l.sh` | 起 | 都漏拒 | `ln` 族 21 条 |
| 带步长的补零序列宽度 | `cp s.sh s{01..100..100}.sh && bash s001.sh` | 起 | 都漏拒 | `seq_diff` 10 条 / 1500，1.2 |
| 标准输入来自文件或 `cat` 管道 | `bash -s < s.sh`、`cat s.sh \| bash -s`、`bash <(cat s.sh)`、`source <(cat s.sh)` | 起 | 都漏拒 | `shell` 族 11 条 |
| `xargs` 起的命令 | `xargs env =1 cargo test --all` | 起 | 都漏拒 | `env` 族 2 条 |
| `trap` 的串 | `builtin trap 'cargo test --all' EXIT` | 退出时起 | 都漏拒 | `builtin` 族 2 条 |
| 展开超过 64 份 | `echo {a,b}` 乘 100 层以上后接 `cargo test --all` | 第一轮腿给的期望是拒；真 bash 展开成 2 的 100 次方份会吃光内存，没交真 bash | 都放（超 64 份照原样留一个词，文件头写明） | `r1-575` 的 `W1-deep` 2 条 |
| 变量隐患 | `copy.copy`、`copy.deepcopy`、`pickle` 一个 `RedirectOperator` 都抛 `TypeError`（`__new__` 要两个参数） | 不涉及 | 钩子里没人拷贝这些词（`grep` 零命中），所以没触发 | `small_checks` 三行；以后谁在词上 `copy.copy` 就红 |

### 2.4　随机组合

- `fuzz2`（3000 条：带装饰的 `cargo test --all`、`bash -c` / `eval` 包装、命令替换、heredoc 喂 shell、先写脚本再起、`function` 定义与调用、控制结构；300 条真 bash 语法错不计）：

```
fuzz2 合计 {'对->对': 1912, '对->漏拒': 3, '对->误拒': 26, '漏拒->对': 315, '漏拒->漏拒': 189, '误拒->对': 144, '误拒->误拒': 111} 语法错 300 共 3000
head1 合计 {'对->对': 2187, '对->漏拒': 1, '漏拒->对': 654, '漏拒->漏拒': 124, '误拒->误拒': 30} 语法错 4 共 3000
```

  3 条前对后漏拒（`out/flags.txt` 的 `[fuzz2]` 行），我把三条各自的修补前拒绝理由（钩子 stderr 里最先出现的那条 ✗ 行）和真 bash 里假 cargo 收到的参数查了一遍：一条是 `bash -s -- s.sh <<-'EOF'` 里再套 `sh <<-'EOF'` 先 `printf` 写脚本、再 `ln -s` 到它、再起它，真 bash 收到 `cargo test --all --quiet`；修补前拒的理由是「脚本 `s.sh:4`」，即把 `-s` 后面的 `s.sh` 当脚本读了（`s.sh` 里本来就有 `cargo test --all`）。一条是 `diff <(builtin -- cargo test --all …)` 之后 `printf` 写出 `function f { builtin cd . ; cargo test --all }` 再 `sh ./t3.sh`，真 bash 里是 dash 把这一行当成命令 `function` 之后接 `cargo test --all }`，假 cargo 收到 `test --all }`；修补前拒的理由是命令文本里的 `builtin -- cargo test --all`，bash 其实不会起它。一条里有 `bash -s < sub/t2.sh`（标准输入来自文件，见 2.3 表里「标准输入来自文件或 `cat` 管道」那一行），真 bash 收到 `cargo test --all`；修补前拒的理由是「脚本 `sub/t2.sh:1`」，同样是把 `-s` 后面的词当脚本。三条的真正重活步骤（printf 写出脚本再起、标准输入来自文件）修补前后都不认，修补前的「对」是别的理由拒出来的。
  26 条前对后误拒：上面 2.2 说的嵌套层函数、`sh -c 'cargo test{,} --all'`（`sh` 是 dash，不做花括号展开，钩子照 bash 展）、`x=$(! cargo …)` 之类。
- `head1`（heredoc 前缀）：1.3。
- 第一轮腿的 575 条（`r1-575` 族，真 bash 裁判只限重型闸主 agent 与检出器 clerk 起 detached-run 的 430 条，别的沿用第一轮腿的期望）：

```
r1-575 合计 {'对->对': 440, '对->误拒': 3, '漏拒->对': 41, '漏拒->漏拒': 21, '误拒->对': 6, '误拒->误拒': 61} 语法错 3 共 575
r1_575_pre_vs_fixed matrix_rows 575
r1_575_pre_vs_fixed     166 0->0
r1_575_pre_vs_fixed     168 0->2
r1_575_pre_vs_fixed      12 2->0
r1_575_pre_vs_fixed     229 2->2
r1-2to0 合计 {'对->对': 12} 语法错 0 共 12
```

  3 条「对->误拒」都是 `cat scripts/hosts/detached-run.sh > /tmp/dr-….sh 2>/dev/null && bash /tmp/dr-….sh start … -- cargo test --all`：沙盒里的假 detached-run 不会真起它后面的 `cargo test --all`，裁判记 0；真的 detached-run 会起，所以这三条是修补让钩子拒对了。迁移数 `0->0 166 / 0->2 168 / 2->0 12 / 2->2 229` 是第一轮判决之前的树到修补后的树。

## 3　判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| V1　第一轮五格反例，修补后是否都判对 | 判对 | `key` 族 21 条与 `keyfeeds.sh` 第一轮那 17 条：修补后 21/21 判对（修补前 7 对、10 漏拒、4 误拒）；第一轮判决之前的树到修补后的树迁移 `0->0 166 / 0->2 168 / 2->0 12 / 2->2 229` 与背景材料读数逐数相同，12 条「改前拒、改后放」真 bash 逐条判都是该放 |
| V1　W1 花括号空份、引号空串、补零 | 判对（带残留） | 随机 6000 个词对真 bash 零差（修补前 563/3000）；残留：带步长、两端写法长短不一的补零序列（`{01..100..3}`），bash 补到较长一端，代码只算 0 起头的那端，1500 条里 10 条，修补前后一样，命令层面 `cp s.sh s{01..100..100}.sh && bash s001.sh` 手工喂两棵树都放 |
| V1　W2 双引号里的 `$(` 收不了尾 | 判对 | 435 条定向加 3000 条随机前缀，修补后的错全含 ANSI-C 引号 `$'…\'…'`（修补前后都不认，另有 1 条修补前碰巧对）；`cd "$(…)" && bash <<EOF` 一类顶层 heredoc 都认 |
| V1　W3 `2>` 与 `ln -s` 相对源 | 判对（带没改全的一处） | `cat … > 拷贝 2>/dev/null`、`ln -s ../s.sh sub/l.sh` 修补后拒、悬空的 `ln -s s.sh sub/l2.sh` 放；heredoc 写文件的那个消费方（`heavy-test-guard.sh:603`）仍不看 fd 号，8 条误拒，修补前后一样；拷出写法 `>>`、`&>`、`>&`、`tee` 等 73 条原来就不认 |
| V1　W4 `env =1` | 判对 | 316 条漏拒变对，真 env 把开头 `=` 的词也当赋值，行为一致 |
| V1　W5 `bash -c --`、`builtin`、`function`、`bash -s --` | 判对（带两处细节） | 266 条漏拒变对、17+76+196 条误拒变对；细节：bash 在 `-c` 之后只认短选项，`bash -c --posix '…'` 会报错不起，修补后误拒（162 条，另有 18 条 `-n`）；`function` 放行见 V2 |
| V1　合计 | 判对 | 第一轮五格的反例逐条判对；改法与 bash 对不上的三处都是冷门写法：补零宽度带步长、`-c` 后长选项、W3 消费方只改了一处 |
| V2　修补带进的新错判 | 判错（反例） | 四类，全是冷门写法：函数名拼写拆开的调用漏拒（`function foo {…}; "fo"o`，110 条）；数字紧贴 `>` 但不是 fd 的写法漏拒（`cat s.sh 01> d.sh`、`"2">`，15 条）；`bash -c` 后接长选项或 `-n` 误拒（180 条）；`env =1` 之后再接选项误拒（6 条，同类修补前就有）；另 1 条随机撞出的 ANSI-C 位置变化 |
| V2　W5 新放行三类会不会放掉该拒的 | 判错（一类） | `function` 放掉了拼写拆开的调用（上一行的函数名拼写拆开那一种）；`builtin`（451 条）与 `bash -s --`（946 条）没放掉一条 |

逐格的「判对 / 判错（反例）/ 没核成」：V1 判对；V2 判错（反例）；V3 不归这一条腿。没有「没核成」的格；没核的东西列在下一节。

## 4　没做什么、复核不了的、清理

没做什么：

- 不替主 agent 采纳或出判决，不判 V3（乙腿的格）。
- 没读禁读清单里的文件与目录（`shell-words-r2-sonnet-s2` 的产出与模型目录、两个草稿目录；修补那一件的报告与探针；`fix.diff` 读的是冻结副本里的那份）。
- 没跑重型测试、没起 cargo（所谓 cargo 是只记参数的假脚本）；没改冻结副本和主工作区的钩子，两棵树都是自己草稿目录里的拷贝。
- 没改 `crates/`、`scripts/`、`.claude/` 下任何文件；写的只有报告、模型目录和草稿目录。

复核不了、没核成：

- 检出器判 clerk 写范围（`cp`、`sed -i`、`>` 写仓内文件）的对错：真 bash 不起重活、裁判没有标记，只在「改前拒、改后放」的 12 条里手工核了 2 条，别的这类没交真 bash。
- 名单外的启动器（`systemd-run`、`flock`、`xargs` 之类）：不在这一轮的射程，沿用第一轮判决第六节的安排；这轮裁判里它们要么沙盒里没有、要么起的是别的程序，`r1-575` 族里这类命令沿用第一轮腿的期望。
- 背景材料第二节里「主 agent 现喂此刻的重型闸」的 rc 读数：我没有喂主工作区的钩子，用的是冻结副本里同一个 sha256 的钩子，`keyfeeds.txt` 里 `env =1 cargo test --all`、`{cargo,} test --all` 同样 rc=2（修补后），结论一致，但不是复核主 agent 那一次喂。
- 别的 shell（zsh、ksh）、bash 5.3、第二台：没试。`sh` 在这台上是 dash，`sh -c '…{,}…'` 不做花括号展开，裁判与钩子（按 bash 展）在这类上不同，已标。
- 这几处是手工喂的（未入复跑，上文已逐处标注，原样输出贴在报告里）：1.2 的一次性逐词对照、两条 `cp` 的目录局限、`cp s.sh s{01..100..100}.sh && bash s001.sh`；1.5 的真 env；1.6 的真 bash `-c` 选项；2.4 里 fuzz2 三条的修补前拒绝理由。

清理：草稿目录 `~/.local/share/singlefs/work/shell-words-r2-sonnet`（认领的那个）交回前 `du -sh` 共 307M，其中仓副本两棵树 `tree` 33M、`tree-orig` 33M，第一轮判决之前的树和它的草稿根 `pre-root` 34M，最后一次验证跑的工作目录 `verify-run-with-a-longer-directory-name-xyz` 129M（里面又是三棵树的拷贝加各族输出），检出记录 `det` 21M，各族输出 `out` 55M，命令清单 `cases`、`cases2` 共 3.6M，其余（`model` 草稿副本 304K、`scratch`、`oracle`、零散的日志与中间文件）不到 1M。这些都是可由 `rerun.sh` 重新铺出来的，模型目录里留着复跑要的全部东西与参照输出，所以全删，只留草稿目录里的 `.owner`、`.writer-lock`。删的结果见交回正文。
