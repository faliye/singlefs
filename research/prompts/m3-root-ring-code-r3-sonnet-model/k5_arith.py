# K5：点名项上限、叶罩、派活状态数的算式（纯算术；常量取自冻结副本，行号见报告）
print('(4096-487)//56 =', (4096-487)//56, ' 487+64*56 =', 487+64*56, ' 487+65*56 =', 487+65*56, '(>4096)')
print('(16384-163)//112 =', (16384-163)//112)
w=290064
print('scratch bytes per state =', w*4, ' 2147483644//that =', 2147483644//(w*4), ' 12GiB//that =', (12*1024**3)//(w*4))
segs=[2,2,1,2,2,1,2,24,2,1,2,24,2,1,2,2,18,2,1,2,16,2,1,2,16,2,1,2,24,2,1,2,24,2,1,2,28,2,1,2,28,2,1,2,28,2,1,2,28,2,1,2,28,2,1,2,2,16,2,1,2,16,2,1,2,28,2,1,2,2,16,2,1,2,16,2,1,2]
print('segments',len(segs),'size-2 segments',segs.count(2),' = 19 record segments + 23 system-configuration pairs ->',19+23)
print('system configuration writes: acquire x2, raise-F x1, unmount x1, rotation after each of 19 publishes ->', 2*(2+1+1+19))
