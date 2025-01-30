use amdgpu_drm_sys::bindings::amdgpu_device;
use std::sync::Mutex;

/* The VMs from DRM render nodes are used by KFD for the lifetime of
 * the process. Therefore we have to keep using the same FDs for the
 * lifetime of the process, even when we close and reopen KFD. There
 * are up to 128 render nodes that we cache in this array.
 */
pub const DRM_FIRST_RENDER_NODE: usize = 128;
pub const DRM_LAST_RENDER_NODE: usize = 255;

#[derive(Debug, Clone)]
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

pub fn hsakmt_fmm_global_get() -> HsaKmtFmmGlobal {
    HSA_KMT_FMM_GLOBAL.lock().unwrap().clone()
}

pub fn hsakmt_fmm_global_print() {
    println!("{:?}", HSA_KMT_FMM_GLOBAL.lock().unwrap());
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
