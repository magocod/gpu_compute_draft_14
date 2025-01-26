use amdgpu_drm_sys::bindings::{amdgpu_device, amdgpu_device_initialize};
use libc::{open, EACCES, EINVAL, ENOENT, EPERM, O_CLOEXEC, O_RDWR};
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::sync::Mutex;
/* The VMs from DRM render nodes are used by KFD for the lifetime of
 * the process. Therefore we have to keep using the same FDs for the
 * lifetime of the process, even when we close and reopen KFD. There
 * are up to 128 render nodes that we cache in this array.
 */
pub const DRM_FIRST_RENDER_NODE: usize = 128;
pub const DRM_LAST_RENDER_NODE: usize = 255;

#[derive(Debug)]
pub struct HsaKmtFmmGlobal {
    pub drm_render_fds: [i32; DRM_LAST_RENDER_NODE + 1 - DRM_FIRST_RENDER_NODE],
    pub amdgpu_handle: [amdgpu_device; DRM_LAST_RENDER_NODE + 1 - DRM_FIRST_RENDER_NODE],
}

impl HsaKmtFmmGlobal {
    pub fn drm_render_fd_set(&mut self, index: usize, fd: i32) {
        self.drm_render_fds[index] = fd;
    }

    pub fn amdgpu_handle_set(&mut self, index: usize, dev: amdgpu_device) {
        self.amdgpu_handle[index] = dev;
    }
}

static HSA_KMT_FMM_GLOBAL: Mutex<HsaKmtFmmGlobal> = Mutex::new(HsaKmtFmmGlobal {
    drm_render_fds: [0; DRM_LAST_RENDER_NODE + 1 - DRM_FIRST_RENDER_NODE],
    amdgpu_handle: [amdgpu_device { _unused: [] };
        DRM_LAST_RENDER_NODE + 1 - DRM_FIRST_RENDER_NODE],
});

pub fn hsakmt_fmm_global_print() {
    println!("{:?}", HSA_KMT_FMM_GLOBAL.lock().unwrap());
}

/// ...
///
///
/// # Safety
///
/// TODO safety function explain
pub unsafe fn hsakmt_open_drm_render_device(minor: i32) -> i32 {
    let mut globals = HSA_KMT_FMM_GLOBAL.lock().unwrap();

    if minor < DRM_FIRST_RENDER_NODE as i32 || minor > DRM_LAST_RENDER_NODE as i32 {
        println!(
            "DRM render minor {} out of range [{}, {}]\n",
            minor, DRM_FIRST_RENDER_NODE, DRM_LAST_RENDER_NODE
        );
        return -EINVAL;
    }

    let index = (minor - DRM_FIRST_RENDER_NODE as i32) as usize;

    /* If the render node was already opened, keep using the same FD */
    if globals.drm_render_fds[index] != 0 {
        return globals.drm_render_fds[index];
    }

    let path = format!("/dev/dri/renderD{}", minor);
    let path_cs = CString::new(path.as_str()).unwrap();

    // let fd = File::open(&path).unwrap();
    // println!("File fd {:?}", fd);

    let fd = open(path_cs.as_ptr(), O_RDWR | O_CLOEXEC);

    println!("hsakmt_open_drm_render_device fd {fd}");

    if fd < 0 {
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap();

        if errno != ENOENT && errno != EPERM {
            println!("Failed to open {:?} {:?}", path, errno);
            if errno == EACCES {
                println!("Check user is in \"video\" group")
            }
        }
        return -errno;
    }

    globals.drm_render_fds[index] = fd;

    let mut device_handle: MaybeUninit<amdgpu_device> = MaybeUninit::uninit();
    let mut major_drm: MaybeUninit<u32> = MaybeUninit::zeroed();
    let mut minor_drm: MaybeUninit<u32> = MaybeUninit::zeroed();

    let ret = amdgpu_device_initialize(
        fd,
        major_drm.as_mut_ptr(),
        minor_drm.as_mut_ptr(),
        &mut device_handle.as_mut_ptr(),
    );
    if ret != 0 {
        panic!("amdgpu_device_initialize failed");
    }

    fd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_globals() {
        let mut drm_render_fds = HSA_KMT_FMM_GLOBAL.lock().unwrap();
        println!("{:?}", drm_render_fds);
        drm_render_fds.drm_render_fd_set(0, 32);
        drm_render_fds.amdgpu_handle_set(0, amdgpu_device { _unused: [] });
        println!("{:?}", drm_render_fds);

        // TODO assert
    }
}
