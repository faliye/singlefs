// 样本：借主机内存额度排队，所在的库没有登记的入口罩着
pub fn take(room: &crate::Room) -> u64 { room.lease_host_memory_waiting(1 << 30) }
