## 九、`check.sh` 与登记给我的门禁阶段

副本上最后一版代码跑的 `nice -n 19 bash .claude/scripts/check.sh`（`logs/check-sh-final.log`，19:27:52 → 19:28:47，退出码 0），末尾原样：

```
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests singlefs_harness

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
```

四道全过（`grep -E '✓|✗'`）：`✓ 格式通过`、`✓ clippy 通过`、`✓ 构建通过`、`✓ 单测通过`。16:20 那一趟（`logs/check-sh.log`）也是绿的，但跑的途中我改了代码（改名、重写一条测试），不算数。

登记给 implementation-writer 的阶段只有 `53-format-const-placeholders.sh`，在副本上跑，末行原样 `  ✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位：D23（journal 的角色与格式） 已定项 19；C323（镜像大小全仓没有条款））`，退出码 0。

另跑过、不归我的（都在副本上）：
- `naming-lint.sh`：第一次红 2 处（`a_worker_thread_panicked` 里的「a」算单字母），已改名 `some_worker_thread_panicked`；最后一版上重跑，原样 `  ✓ 命名纪律通过：查了 192 个 .rs 文件、35971 个声明的名字`。
- 门禁 59 号的判法，只拿追加的 8 行跑（`gate59-root/` 里的表只留注释与这 8 行，`logs/gate59-new-rows-final.log`）：末行 `  ✓ crates 变异表复跑：8 条变异各自红在点名的测试上（原文都恰好命中一次）`，退出码 0。整张表 133 行的锚点预扫：每行六段、原文在文件里都恰好命中 1 次（Python 照 59 号的判法扫，零条不合）；整表没真跑。
- 补过的 54 号整道在副本上跑了一次（没设 `SINGLEFS_LAYER0_THREADS`），结果见第七节末尾的补记。
