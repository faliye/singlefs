#!/usr/bin/env bash
# 往第二台起活的那一份的部分文件（钩子拆法：multi-host-run-<部分>.sh，自证在里面）也不扫
bash scripts/hosts/multi-host-run.sh --peer -- true
