# 独立小模型：照 verdict_store.rs import_the_planned_rows_in_batches_of 的循环次序，数 28 行 / 7 批。
# 行数规则（put_conclusion_rows_into_the_batch，verdict_store.rs:934）：块 1 + 违例数 + 计数向量 1 + 块属性 1 + 恢复字(有则 1)
def rows(viol, recovery): return 1 + viol + 1 + 1 + (1 if recovery else 0)
def run(limit):
    # 批的“字节”用行数×1 代替：任何 >=1 行的批在 limit=1 时都满；limit=inf 永不满
    pending=0; batches=[]
    def flush_if_full():
        nonlocal pending
        if pending>=1 and limit==1:   # size_in_bytes 在刚 put 之后 >= 1 字节
            batches.append(pending); pending=0
    # 计划次序：F1(Gpu,Cpu,Obs) F2(...) F3(...)
    # A 里有：F1 CPU（与 B 相同）。B 里有：F1 Gpu, F1 Cpu, F2 Gpu, F2 Obs, F3 Gpu
    registered_in_A=2  # "" -> 0, "red-f" -> 1
    new_vectors=0
    items=[("F1","Gpu"),("F1","Cpu"),("F1","Obs"),("F2","Gpu"),("F2","Cpu"),("F2","Obs"),("F3","Gpu"),("F3","Cpu"),("F3","Obs")]
    inB={("F1","Gpu"):(1,True,0),("F1","Cpu"):(1,True,0),("F2","Gpu"):(1,True,1),("F2","Obs"):(1,False,1),("F3","Gpu"):(1,True,1)}
    newvec={("F2","Gpu"):1,("F3","Gpu"):1}   # F2 Obs 用 F2 Gpu 已登记的同一种
    written=ident=missing=0
    for it in items:
        if it not in inB: missing+=1; continue
        v,rec,_=inB[it]
        if it==("F1","Cpu"): ident+=1; continue
        r=rows(v,rec)+2*newvec.get(it,0)
        pending+=r; written+=1; flush_if_full()
    paths=2; pending+=paths
    verifier=0; facts=0
    for blk in ("F1","F2","F3"):
        for dig in ("CPU","GPU"):
            if dig=="CPU" and blk in ("F1","F2"):
                pending+=1; verifier+=1; flush_if_full()
    facts=1; pending+=1; flush_if_full()
    if pending: batches.append(pending)
    return written,ident,missing,paths,verifier,facts,sum(batches),len(batches),batches
print("1 字节一批:",run(1))
print("64MiB:",run(10**9))
