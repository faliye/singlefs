//! m3-code-r1 攻方腿原型的共用件：点名第几次读坏的故障盘。
use std::cell::RefCell;
use std::rc::Rc;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_harness::memory_pool::SparseBlockDevice;

/// 一段读故障：哪块盘、[offset, offset + length) 与读区间有交集的那几次读里，第几次（从 1 数）读坏。
#[derive(Clone, Debug)]
pub struct FaultRange {
    pub device: DeviceIdentity,
    pub offset: u64,
    pub length: u64,
    pub failing_read_numbers: Vec<u64>,
    pub reads_so_far: u64,
}

#[derive(Debug, Default)]
pub struct Faults {
    pub ranges: Vec<FaultRange>,
}

pub struct FaultyDevice {
    pub identity: DeviceIdentity,
    pub inner: SparseBlockDevice,
    pub faults: Rc<RefCell<Faults>>,
}

impl BlockDevice for FaultyDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let end = offset.0 + buffer.len() as u64;
        let mut fail = false;
        for range in &mut self.faults.borrow_mut().ranges {
            if range.device == self.identity && offset.0 < range.offset + range.length && range.offset < end {
                range.reads_so_far += 1;
                if range.failing_read_numbers.contains(&range.reads_so_far) {
                    fail = true;
                }
            }
        }
        if fail {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("原型点名的这一次读坏")));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 一组稀疏盘套上同一份故障表。
pub fn with_faults(devices: Vec<(DeviceIdentity, SparseBlockDevice)>, ranges: Vec<FaultRange>) -> (Vec<(DeviceIdentity, FaultyDevice)>, Rc<RefCell<Faults>>) {
    let faults = Rc::new(RefCell::new(Faults { ranges }));
    let devices = devices
        .into_iter()
        .map(|(identity, inner)| (identity, FaultyDevice { identity, inner, faults: faults.clone() }))
        .collect();
    (devices, faults)
}

/// 给调用线程（与它之后 exec 的程序）装一个 seccomp 过滤器：`fallocate(2)` 一律由内核交回 `errno`，不进文件系统。
/// 别的系统调用照常。装上就卸不掉：只在专门起的线程里、或 exec 之前调。
///
/// # Panics
/// `prctl` 设不上（内核不许）。
#[allow(unsafe_code)]
pub fn make_fallocate_fail_with(errno: i32) {
    #[repr(C)]
    struct SockFilter {
        code: u16,
        jt: u8,
        jf: u8,
        k: u32,
    }
    #[repr(C)]
    struct SockFprog {
        len: u16,
        filter: *const SockFilter,
    }
    const LOAD_WORD_ABSOLUTE: u16 = 0x20;
    const JUMP_IF_EQUAL_CONSTANT: u16 = 0x15;
    const RETURN_CONSTANT: u16 = 0x06;
    const AUDIT_ARCH_X86_64: u32 = 0xc000_003e;
    const SECCOMP_RET_ERRNO: u32 = 0x0005_0000;
    const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
    let filter = [
        SockFilter { code: LOAD_WORD_ABSOLUTE, jt: 0, jf: 0, k: 4 },
        SockFilter { code: JUMP_IF_EQUAL_CONSTANT, jt: 0, jf: 3, k: AUDIT_ARCH_X86_64 },
        SockFilter { code: LOAD_WORD_ABSOLUTE, jt: 0, jf: 0, k: 0 },
        SockFilter { code: JUMP_IF_EQUAL_CONSTANT, jt: 0, jf: 1, k: u32::try_from(libc::SYS_fallocate).expect("小") },
        SockFilter { code: RETURN_CONSTANT, jt: 0, jf: 0, k: SECCOMP_RET_ERRNO | (u32::try_from(errno).expect("正") & 0xffff) },
        SockFilter { code: RETURN_CONSTANT, jt: 0, jf: 0, k: SECCOMP_RET_ALLOW },
    ];
    let program = SockFprog { len: u16::try_from(filter.len()).expect("小"), filter: filter.as_ptr() };
    // SAFETY: 原型代码；prctl 只读 program 指向的六条指令，调用返回之前它们都活着。
    unsafe {
        assert_eq!(libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0), 0, "PR_SET_NO_NEW_PRIVS");
        assert_eq!(
            libc::prctl(libc::PR_SET_SECCOMP, 2 as libc::c_ulong, &program as *const SockFprog as libc::c_ulong, 0, 0),
            0,
            "PR_SET_SECCOMP：{}",
            std::io::Error::last_os_error()
        );
    }
}

/// errno 名字到值（原型只认这几个）。
#[must_use]
pub fn errno_named(name: &str) -> i32 {
    match name {
        "EOPNOTSUPP" => libc::EOPNOTSUPP,
        "ENOSYS" => libc::ENOSYS,
        "ENOSPC" => libc::ENOSPC,
        "EINTR" => libc::EINTR,
        "EPERM" => libc::EPERM,
        "EIO" => libc::EIO,
        "EINVAL" => libc::EINVAL,
        other => panic!("原型不认 {other}"),
    }
}
