# K5：从冻结副本 crash_enumeration_fixed_script_stream.rs 第 445-450 行的段序列算 481 / 两态闭式 / 三态全量。
import re,sys
src=open('/tmp/claude-1000/m3-root-ring-code-r3/tree/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs',encoding='utf-8').read().split('\n')
txt='\n'.join(src[444:451])  # 第 445-451 行（0 基 444..450）
nums=[int(x) for x in re.findall(r'\d+',txt.split('vec![')[1].split(']')[0])]
print('segments',len(nums),'writes',sum(nums))
two=1+sum(2**n-1 for n in nums)
print('two_state_closed_form',two)
# 三态：46 次系统配置槽写，两两自成 23 个 2 写段，每段 3^2-1 代替 2^2-1，多 5
two_segs=sum(1 for n in nums if n==2)
print('segments_of_size_2',two_segs)
print('three_state_if_all_size2_are_sysconf_pairs',two+5*two_segs)
print('three_state_if_23_pairs',two+5*23)
# 到 D 一段闭式
upto_d=nums[:36]
print('to_D_segments',len(upto_d),'writes',sum(upto_d),'closed',1+sum(2**n-1 for n in upto_d))
