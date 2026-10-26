# agents 回归测试

在仓根运行：

```bash
python3 -B -m unittest discover -s scripts/agents/tests -v
```

覆盖快照路径与执行权限、备份独立性和并发清单、引文解析、统计参数、看门狗文件消失，以及提醒的并发登记和停止。测试使用临时目录、临时 Git 仓和自行启动的进程，结束后清理。

脚本原有的 `--selftest` 仍可运行；新增回归用例放在本目录。
