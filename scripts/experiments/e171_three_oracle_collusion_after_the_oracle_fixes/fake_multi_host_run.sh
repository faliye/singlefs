#!/usr/bin/env bash
# 假 multi-host-run.sh：rows_orchestrator.py 照真的那样用 bash 起它；换成 python 那一份（同一个进程号，信号直达）
exec python3 -B "$(dirname "${BASH_SOURCE[0]}")/fake_multi_host_run.py" "$@"
