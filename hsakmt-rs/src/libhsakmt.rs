use libc::{ioctl, EAGAIN, EBADF, EINTR};

/* Call ioctl, restarting if it is interrupted */

/// ...
///
///
/// # Safety
///
/// TODO safety function explain
#[allow(unused_assignments)]
pub unsafe fn hsakmt_ioctl(fd: i32, request: u64, arg: *mut std::os::raw::c_void) -> i32 {
    let mut ret = 0;
    let mut errno = 0;

    loop {
        ret = ioctl(fd, request, arg);

        errno = std::io::Error::last_os_error().raw_os_error().unwrap();

        if ret == -1 && (errno == EINTR || errno == EAGAIN) {
            continue;
        }

        break;
    }

    if ret == -1 && errno == EBADF {
        /* In case pthread_atfork didn't catch it, this will
         * make any subsequent hsaKmt calls fail in CHECK_KFD_OPEN.
         */
        println!("KFD file descriptor not valid in this process\n");
        // hsakmt_is_forked_child();
    }

    ret
}
