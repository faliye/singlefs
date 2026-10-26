#!/usr/bin/env bash
# 样本：交给第二台时 --memory-cap 写死
bash scripts/hosts/multi-host-run.sh --peer --memory-cap 6G --label sample -- true
