#!/usr/bin/env python3
"""丢根槽叠加：随机历史里把 L（丢候选集里第 k 条非现行根的根槽）与回退、抬 F 的比重拉高，让「现行根 C 与回退根同时读不出」这类叠加出现。
python3 run_fault_double_loss.py > run_fault_double_loss.out"""
import os, sys
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import birth_floor_v2 as m
alpha = m.ALPHABETS["FAULT"] + [("L", k) for k in range(3)] * 6 + [a for a in m.ALPHABETS["FAULT"] if a[0] in "RFN"] * 3 + [("W",)] * 4
for seed in (31, 32, 33):
    s, wrapped, ex = m.random_histories(alpha, 3000, 60, seed, fut=False, fault=True)
    m.show(f"[随机·丢根槽叠加] 3000 段 × 至多 60 步，种子 {seed}", s, f"走过根环的历史 {wrapped} 段")
