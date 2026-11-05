#!/usr/bin/env python3
"""样本：自证开跑先调客户端的 selftest_service_problem（只在临时 prune-server 上跑）；这一格该在起它之前就判红，它不该被起。"""
import sys

print("  ✗ 样本：这一条不该被起（runner 表里裸写，这一格该先判红）")
sys.exit(1)
