# V3 的算术：每条重判的单价、重判条数的两种读法、溢出上界、编译次数、基线秒数相加。输入都是别的脚本的输出常数（写在这里，来源注释）。
mix_full=(241*72.4+39*0.9)/280   # res.txt: caught full mean 72.4 (241 条)；unviable 0.9 (39 条，Sonnet r1 表)
mix_T=(241*22.8+39*0.9)/280      # res.txt: T-only mean 22.8 = 增量编译均值 + 那条测试在基线里的耗时
viable=241/280
print('s/mutant steady state: full %.1f  T-first %.1f ; viable share %.3f'%(mix_full,mix_T,viable))
print('baseline seven steps sum (baseline.tsv lines 2-8): %.1f s'%(23.5+278.5+95.7+1.5+936.0+43.5+47.3))
for k,A,B in ((1,5,368),(5,24,1516),(20,96,3364)):
    sp=0.228*(5091-B)
    print('k=%d A=%d B=%d spill<=%.0f | hours full/T-first: A %.2f/%.2f  B %.1f/%.1f  spill %.1f/%.1f  B+spill %.1f/%.1f | builds A %.0f B %.0f spill %.0f'%(
      k,A,B,sp,A*mix_full/3600,A*mix_T/3600,B*mix_full/3600,B*mix_T/3600,sp*mix_full/3600,sp*mix_T/3600,(B+sp)*mix_full/3600,(B+sp)*mix_T/3600,A*viable,B*viable,sp*viable))
for m in (1.0,1.5,2.0,3.0):
    print('coverage pass if multiplier %.1f (假设，没量): %.2f h ; plus the baseline run %.2f h'%(m,1426.0*m/3600,1426.0/3600))
print('real-commit median: 2157 mutants -> %.1f h full / %.1f h T-first'%(2157*mix_full/3600,2157*mix_T/3600))
print('all 5091 mutants: %.1f h full / %.1f h T-first'%(5091*mix_full/3600,5091*mix_T/3600))
print('B2: 170+2 mutants always run: %.2f h full / %.2f h T-first per commit'%(172*mix_full/3600,172*mix_T/3600))
