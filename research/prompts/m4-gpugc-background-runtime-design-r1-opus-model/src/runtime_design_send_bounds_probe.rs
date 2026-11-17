//! 运行库设计轮攻方腿的探针（只在仓副本里编）：「一把锁加周期线程」要把会话与盘移进线程，这几个类型过不过 `Send`。编不过就是答案。
//! 录制盘（`RecordingBlockDevice`）不在这里：它带 `SharedStream(Rc<RefCell<…>>)`，读源码就知道过不了。
use singlefs_core::block_device::{DirectInputOutputBlockDevice, FileBackedBlockDevice};
use singlefs_core::mounted_session::{MountedSession, MountedSessionWithPendingChanges};
use singlefs_verification::memory_pool::SparseBlockDevice;

fn must_be_send<T: Send>() {}

#[test]
fn the_session_and_the_devices_can_move_into_a_thread() {
    must_be_send::<MountedSession>();
    must_be_send::<MountedSessionWithPendingChanges>();
    must_be_send::<FileBackedBlockDevice>();
    must_be_send::<DirectInputOutputBlockDevice>();
    must_be_send::<SparseBlockDevice>();
    println!("name=send_bounds session=send pending_session=send file_device=send direct_device=send sparse_device=send");
}
