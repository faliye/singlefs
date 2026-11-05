#!/usr/bin/env bash
# 复用三问的闭包外改动：alpha.py 末尾加一个没人调的函数（不带缺省值、不带装饰器，import 时不收进闭包），五条的键都不变
set -euo pipefail
printf '\n\ndef unused_note():\n    return "闭包外的改动"\n' >> scripts/gate/alpha.py
