# 模型：照 make_filesystem.rs fixed_structure_extents 的式子（系统配置槽 i 在 i*槽距、宽 4096；区域 r 起点 = 64*16384 + r*3*1048576，长 S*槽距；journal 环从 1024*16384 起）
SLOT=16384; BASE=64*SLOT; STEP=3*1048576; J=1024*SLOT
def extents(sp,S):
    e=[("sys0",0,4096),("sys1",sp,4096)]
    e+=[(f"reg{r}",BASE+r*STEP,S*sp) for r in range(3)]
    e+=[("journal",J,10**12)]  # 环长不限，只看起点
    return e
def ov(a,b): return a[1]<b[1]+b[2] and b[1]<a[1]+a[2]
def first_overlap(e,keep=None):
    es=[x for x in e if keep is None or x[0] in keep]
    for i,a in enumerate(es):
        for b in es[i+1:]:
            if ov(a,b): return (a[0],b[0])
    return None
print("io_min  S  all-pairs(现行)  same-disk-regions(disk0: sys,reg0,reg2,journal)")
for name,io in [("4K",4096),("64K",65536),("128K",131072),("256K",262144),("384K",393216),("512K",524288),("1M",1<<20),("2M",2<<20)]:
    sp=-(-4096//io)*io
    for S in (4,8,16):
        e=extents(sp,S)
        a=first_overlap(e)
        d0=first_overlap(e,{"sys0","sys1","reg0","reg2","journal"})
        d1=first_overlap(e,{"sys0","sys1","reg1","journal"})
        print(f"{name:5} S={S:2} sp={sp:8} all={a}  disk0={d0}  disk1={d1}")
