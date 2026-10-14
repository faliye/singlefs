#!/usr/bin/env bash
run_device --workers "${SAMPLE_SHELL_WORKERS:?工作进程数}"
bash scripts/hosts/capped.sh default true
