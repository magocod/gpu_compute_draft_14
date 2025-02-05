#![allow(non_camel_case_types, dead_code, non_snake_case)]

use crate::fmm_globals::svm_aperture_type::{SVM_COHERENT, SVM_DEFAULT};
use crate::fmm_globals::{
    gpu_mem_find_by_gpu_id, gpu_mem_t, hsakmt_fmm_global_alignment_order_get,
    hsakmt_fmm_global_dgpu_shared_aperture_limit_get, hsakmt_fmm_global_get,
    hsakmt_fmm_global_gpu_mem_count_get, hsakmt_fmm_global_gpu_mem_set,
    hsakmt_fmm_global_svm_reserve_svm_get, hsakmt_fmm_global_svm_set, manageable_aperture_ops_t,
    manageable_aperture_t, svm_t, vm_object_t, DRM_FIRST_RENDER_NODE, DRM_LAST_RENDER_NODE,
    HSA_KMT_FMM_GLOBAL,
};
use crate::globals::{
    global_page_size, hsakmt_global_hsakmt_is_dgpu_get, hsakmt_global_is_svm_api_supported_set,
    hsakmt_kfd_fd_get,
};
use crate::hsakmttypes::HsakmtStatus::{HSAKMT_STATUS_ERROR, HSAKMT_STATUS_SUCCESS};
use crate::hsakmttypes::{
    HsaMemFlagSt, HsaMemFlagUnion, HsaMemFlags, HsakmtStatus, ALIGN_UP, GFX_VERSION_VEGA10,
    GPU_HUGE_PAGE_SIZE, HSA_ENGINE_ID, HSA_GET_GFX_VERSION_FULL,
};
use crate::kfd_ioctl::{
    kfd_ioctl_acquire_vm_args, kfd_ioctl_alloc_memory_of_gpu_args,
    kfd_ioctl_free_memory_of_gpu_args, kfd_ioctl_get_process_apertures_new_args,
    kfd_ioctl_set_memory_policy_args, kfd_process_device_apertures,
    KFD_IOC_ALLOC_MEM_FLAGS_COHERENT, KFD_IOC_ALLOC_MEM_FLAGS_EXT_COHERENT,
    KFD_IOC_ALLOC_MEM_FLAGS_MMIO_REMAP, KFD_IOC_ALLOC_MEM_FLAGS_NO_SUBSTITUTE,
    KFD_IOC_ALLOC_MEM_FLAGS_PUBLIC, KFD_IOC_ALLOC_MEM_FLAGS_USERPTR, KFD_IOC_ALLOC_MEM_FLAGS_VRAM,
    KFD_IOC_ALLOC_MEM_FLAGS_WRITABLE, KFD_IOC_CACHE_POLICY_COHERENT,
    KFD_IOC_CACHE_POLICY_NONCOHERENT,
};
use crate::libhsakmt::hsakmt_ioctl;
use crate::rbtree::{hsakmt_rbtree_insert, rbtree_init};
use crate::rbtree_amd::rbtree_key;
use crate::topology::{hsakmt_topology_setup_is_dgpu_param, HSA_KMT_TOPOLOGY_GLOBAL};
use amdgpu_drm_sys::bindings::{amdgpu_device, amdgpu_device_initialize};
use libc::{
    getenv, mmap, munmap, open, strcmp, strerror, EACCES, EINVAL, ENOENT, EPERM, MAP_ANONYMOUS,
    MAP_FAILED, MAP_FIXED, MAP_FIXED_NOREPLACE, MAP_NORESERVE, MAP_PRIVATE, MPOL_DEFAULT,
    O_CLOEXEC, O_RDWR, PROT_NONE,
};
use numa_sys::numaif_bindings::mbind;
use std::ffi::CString;
use std::mem::MaybeUninit;

pub unsafe fn hsakmt_open_drm_render_device(minor: i32) -> i32 {
    let mut globals = hsakmt_fmm_global_get();

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

// TODO complete fn get_vm_alignment
pub fn get_vm_alignment(device_id: u32) -> u32 {
    let _page_size = 0;

    let global_page_size = global_page_size();

    if device_id >= 0x6920 && device_id <= 0x6939 {
        /* Tonga */
        // page_size = TONGA_PAGE_SIZE;
    } else if device_id >= 0x9870 && device_id <= 0x9877 {
        /* Carrizo */
        // page_size = TONGA_PAGE_SIZE;
    } else {
        // println!("device_id no apply get_vm_alignment {}", device_id);
    }

    // MAX(PAGE_SIZE, page_size);
    // MAX tmp1 > tmp2 ? tmp1 : tmp2
    global_page_size as u32
}

/* Align size of a VM area
 *
 * Leave at least one guard page after every object to catch
 * out-of-bounds accesses with VM faults.
 */
pub fn vm_align_area_size(app: &manageable_aperture_t, size: u64) -> u64 {
    let global_page_size = global_page_size() as u64;
    size + (app.guard_pages as u64) * global_page_size
}

/*
 * returns allocated address or NULL. Assumes, that fmm_mutex is locked
 * on entry.
 */
pub fn reserved_aperture_allocate_aligned(
    app: &manageable_aperture_t,
    address: *mut std::os::raw::c_void,
    MemorySizeInBytes: &mut u64,
    align: &mut u64,
) -> *mut std::os::raw::c_void {
    let mut offset: u64 = 0;
    let orig_align = *align;

    let _start: *mut std::os::raw::c_void = std::ptr::null_mut();

    if *align < app.align {
        *align = app.align;
    }

    /* Align big buffers to the next power-of-2 up to huge page
     * size for flexible fragment size TLB optimizations
     */
    while *align < GPU_HUGE_PAGE_SIZE as u64 && MemorySizeInBytes >= &mut (*align << 1) {
        *align <<= 1;
    }

    /* If no specific alignment was requested, align the end of
     * buffers instead of the start. For fragment optimizations,
     * aligning the start or the end achieves the same effective
     * optimization. End alignment to the TLB cache line size is
     * needed as a workaround for TLB issues on some older GPUs.
     */
    let global_page_size = global_page_size();

    if orig_align <= global_page_size as u64 {
        offset = *align - (*MemorySizeInBytes & (*align - 1));
    }

    *MemorySizeInBytes = vm_align_area_size(app, *MemorySizeInBytes);

    /* Find a big enough "hole" in the address space */
    let _cur: *mut std::os::raw::c_void = std::ptr::null_mut();

    let _next = &app.vm_ranges;

    let start = if !address.is_null() {
        address
    } else {
        let r = ALIGN_UP(app.base as u64, *align) + offset;
        r as *mut std::os::raw::c_void
    };

    // while next {
    // if next.start > start && VOID_PTRS_SUB(next->start, start) >= MemorySizeInBytes) {
    //     break;
    // }
    //
    // cur = next;
    // next = next->next;
    // if (!address)
    // 	start = (void *)(ALIGN_UP((uint64_t)cur->end + 1, align) + offset);
    // }

    // if (!next && VOID_PTRS_SUB(app->limit, start) + 1 < MemorySizeInBytes)
    // 	/* No hole found and not enough space after the last area */
    // 	return NULL;
    //
    // if (cur && address && address < (void *)ALIGN_UP((uint64_t)cur->end + 1, align))
    // 	/* Required address is not free or overlaps */
    // 	return NULL;
    //
    // if (cur && VOID_PTR_ADD(cur->end, 1) == start) {
    // 	/* extend existing area */
    // 	cur->end = VOID_PTR_ADD(start, MemorySizeInBytes-1);
    // } else {
    // 	vm_area_t *new_area;
    // 	/* create a new area between cur and next */
    // 	new_area = vm_create_and_init_area(start,
    // 			VOID_PTR_ADD(start, (MemorySizeInBytes - 1)));
    // 	if (!new_area)
    // 		return NULL;
    // 	new_area->next = next;
    // 	new_area->prev = cur;
    // 	if (cur)
    // 		cur->next = new_area;
    // 	else
    // 		app->vm_ranges = new_area;
    // 	if (next)
    // 		next->prev = new_area;
    // }

    start
}

pub unsafe fn get_process_apertures(
    process_apertures: *mut kfd_process_device_apertures,
    num_of_nodes: &mut u32,
) -> HsakmtStatus {
    let kfd_process_device_apertures_ptr = process_apertures as *mut u64;

    let mut args_new = kfd_ioctl_get_process_apertures_new_args {
        kfd_process_device_apertures_ptr,
        num_of_nodes: *num_of_nodes,
        pad: 0,
    };

    let p_1 = ('K' as i32) << 8;
    let p_2 = ((std::mem::size_of::<kfd_ioctl_get_process_apertures_new_args>()) << (8 + 8)) as i32;
    let AMDKFD_IOC_GET_PROCESS_APERTURES_NEW = (((2 | 1) << ((8 + 8) + 14)) | p_1 | (0x14)) | p_2;

    let hsakmt_kfd_fd = hsakmt_kfd_fd_get();

    let ret = hsakmt_ioctl(
        hsakmt_kfd_fd,
        AMDKFD_IOC_GET_PROCESS_APERTURES_NEW as u64,
        &mut args_new as *mut _ as *mut std::os::raw::c_void,
    );

    if ret == -1 {
        println!(
            "hsakmt_kfd_fd {}, num_of_nodes {}",
            hsakmt_kfd_fd, num_of_nodes
        );
        panic!("hsakmt_ioctl failed {}", ret);
        // return HSAKMT_STATUS_ERROR
    }

    *num_of_nodes = args_new.num_of_nodes;
    HSAKMT_STATUS_SUCCESS
}

pub unsafe fn aperture_allocate_area(
    app: &manageable_aperture_t,
    address: *mut std::os::raw::c_void,
    MemorySizeInBytes: u64,
) -> *mut std::os::raw::c_void {
    let some_f = app
        .ops
        .allocate_area_aligned
        .expect("aperture_allocate_area not found");
    some_f(app, address, MemorySizeInBytes, app.align)
}

pub unsafe fn aperture_release_area(
    app: &manageable_aperture_t,
    address: *mut std::os::raw::c_void,
    MemorySizeInBytes: u64,
) {
    let some_f = app
        .ops
        .release_area
        .expect("aperture_release_area not found");
    some_f(app, address, MemorySizeInBytes);
}

pub unsafe fn acquire_vm(gpu_id: u32, fd: i32) -> HsakmtStatus {
    let mut args = kfd_ioctl_acquire_vm_args {
        gpu_id,
        drm_fd: fd as u32,
    };

    let hsakmt_kfd_fd = hsakmt_kfd_fd_get();

    let p_1 = ('K' as i32) << 8;
    let p_2 = (std::mem::size_of::<kfd_ioctl_acquire_vm_args>()) << (8 + 8);
    let AMDKFD_IOC_ACQUIRE_VM = ((1) << ((08 + 8) + 14)) | p_1 | (0x15) | p_2 as i32;

    // println!("acquiring VM for {} using {}", gpu_id, fd);
    let ret = hsakmt_ioctl(
        hsakmt_kfd_fd,
        AMDKFD_IOC_ACQUIRE_VM as u64,
        &mut args as *mut _ as *mut std::os::raw::c_void,
    );

    if ret > 0 {
        panic!("AMDKFD_IOC_ACQUIRE_VM failed {}", ret);
        // return HSAKMT_STATUS_ERROR;
    }

    HSAKMT_STATUS_SUCCESS
}

/* Managed SVM aperture limits: only reserve up to 40 bits (1TB, what
 * GFX8 supports). Need to find at least 4GB of usable address space.
 */
// #define SVM_RESERVATION_LIMIT ((1ULL << 40) - 1)
// #define SVM_MIN_VM_SIZE (4ULL << 30)
// #define IS_CANONICAL_ADDR(a) ((a) < (1ULL << 47))

pub const SVM_MIN_VM_SIZE: u64 = 4u64 << 30;

pub fn IS_CANONICAL_ADDR(gpuvm_limit: u64) -> bool {
    gpuvm_limit < (1u64 << 47)
}

/* Void pointer arithmetic (or remove -Wpointer-arith to allow void pointers arithmetic) */
// #define VOID_PTR_ADD32(ptr,n) (void*)((uint32_t*)(ptr) + n)/*ptr + offset*/
// #define VOID_PTR_ADD(ptr,n) (void*)((uint8_t*)(ptr) + n)/*ptr + offset*/
// #define VOID_PTR_SUB(ptr,n) (void*)((uint8_t*)(ptr) - n)/*ptr - offset*/
// #define VOID_PTRS_SUB(ptr1,ptr2) (uint64_t)((uint8_t*)(ptr1) - (uint8_t*)(ptr2)) /*ptr1 - ptr2*/
pub unsafe fn VOID_PTR_ADD(ptr: *mut std::os::raw::c_void, n: u64) -> *mut std::os::raw::c_void {
    let ptr_n = ptr as *mut u64;
    let r = ptr_n.add(n as usize);
    r as *mut std::os::raw::c_void
}

pub unsafe fn VOID_PTR_SUB(ptr: *mut std::os::raw::c_void, n: u64) -> *mut std::os::raw::c_void {
    let ptr_n = ptr as *mut u64;
    let r = ptr_n.sub(n as usize);
    r as *mut std::os::raw::c_void
}

pub unsafe fn VOID_PTRS_SUB(
    ptr_1: *mut std::os::raw::c_void,
    ptr_2: *mut std::os::raw::c_void,
) -> u64 {
    // let ptr_1_n = ptr_1 as *mut u8;
    // let ptr_2_n = ptr_2 as *mut u8;
    let ptr_1_n = ptr_1 as *mut u64;
    let ptr_2_n = ptr_2 as *mut u64;

    let r = ptr_1_n.sub(ptr_2_n as usize);

    // println!("VOID_PTRS_SUB p1 {} p2 {} - r: {}", ptr_1 as u8, ptr_2 as u8, r as usize);

    r as u64
}

pub unsafe fn hsakmt_mmap_allocate_aligned(
    prot: i32,
    flags: i32,
    size: u64,
    align: u64,
    guard_size: u64,
    aper_base: *mut std::os::raw::c_void,
    aper_limit: *mut std::os::raw::c_void,
) -> *mut std::os::raw::c_void {
    let page_size = global_page_size();

    let aligned_padded_size = size + guard_size * 2 + (align - page_size as u64);

    #[allow(clippy::zero_ptr)]
    /* Map memory PROT_NONE to alloc address space only */
    let mut addr = mmap(
        0 as *mut std::os::raw::c_void,
        aligned_padded_size as usize,
        PROT_NONE,
        flags,
        -1,
        0,
    );
    if addr == MAP_FAILED {
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap();
        println!("mmap failed: {:?}", strerror(errno));
        return std::ptr::null_mut();
    }

    /* Adjust for alignment and guard pages */
    // println!("size {}", size);

    let aligned_addr = ALIGN_UP((addr as u64) + guard_size, align) as *mut std::os::raw::c_void;
    let p = VOID_PTR_ADD(aligned_addr, size - 1);

    if (aligned_addr < aper_base || p > aper_limit) {
        println!(
            "mmap returned {:?}, out of range {:?} - {:?}",
            aligned_addr, aper_base, aper_limit
        );
        munmap(addr, aligned_padded_size as usize);
        return std::ptr::null_mut();
    }

    // let _r = VOID_PTRS_SUB(aligned_addr, addr);

    /* Unmap padding and guard pages */
    if aligned_addr > addr {
        munmap(addr, VOID_PTRS_SUB(aligned_addr, addr) as usize);
    }

    let aligned_end = VOID_PTR_ADD(aligned_addr, size);
    let mapping_end = VOID_PTR_ADD(addr, aligned_padded_size);
    if mapping_end > aligned_end {
        let r = VOID_PTRS_SUB(mapping_end, aligned_end) as usize;
        munmap(aligned_end, r);
    }

    // if prot == PROT_NONE {
    //     return aligned_addr;
    // }

    // /*  MAP_FIXED to the aligned address with required prot */
    // addr = mmap(aligned_addr, size as usize, prot, flags | MAP_FIXED, -1, 0);
    // if (addr == MAP_FAILED) {
    //     let errno  = std::io::Error::last_os_error().raw_os_error().unwrap();
    //
    //     println!("mmap failed: {:?}", strerror(errno));
    // 	return std::ptr::null_mut();
    // }

    addr
}

pub unsafe fn mmap_aperture_allocate_aligned(
    aper: &manageable_aperture_t,
    address: *mut std::os::raw::c_void,
    size: u64,
    mut align: u64,
) -> *mut std::os::raw::c_void {
    // std::ptr::null_mut()

    let page_size = global_page_size();
    let alignment_order = hsakmt_fmm_global_alignment_order_get();

    let alignment_size = page_size << alignment_order;
    let guard_size: u64 = 0;

    if !aper.is_cpu_accessible {
        println!("MMap Aperture must be CPU accessible\n");
        return std::ptr::null_mut();
    }

    if !address.is_null() {
        // #ifdef MAP_FIXED_NOREPLACE
        let addr = mmap(
            address,
            size as usize,
            PROT_NONE,
            MAP_ANONYMOUS | MAP_NORESERVE | MAP_PRIVATE | MAP_FIXED_NOREPLACE,
            -1,
            0,
        );
        // #endif
        if addr == MAP_FAILED {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap();
            println!("mmap failed: {:?}", strerror(errno));
            return std::ptr::null_mut();
        }

        return addr;
    }

    /* Align big buffers to the next power-of-2. By default, the max alignment
     * size is set to 2MB. This can be modified by the env variable
     * HSA_MAX_VA_ALIGN. This variable sets the order of the alignment size as
     * PAGE_SIZE * 2^HSA_MAX_VA_ALIGN. Setting HSA_MAX_VA_ALIGN = 18 (1GB),
     * improves the time for memory allocation and mapping. But it might lose
     * performance when GFX access it, specially for big allocations (>3GB).
     */
    while align < alignment_size as u64 && size >= (align << 1) {
        align <<= 1;
    }

    let page_size = global_page_size();

    /* Add padding to guarantee proper alignment and leave guard
     * pages on both sides
     */
    let guard_size = aper.guard_pages * page_size as u32;

    hsakmt_mmap_allocate_aligned(
        PROT_NONE,
        MAP_ANONYMOUS | MAP_NORESERVE | MAP_PRIVATE,
        size,
        align,
        guard_size as u64,
        aper.base,
        aper.limit,
    )
}

pub unsafe fn mmap_aperture_release(
    aper: &manageable_aperture_t,
    addr: *mut std::os::raw::c_void,
    size: u64,
) {
    if !aper.is_cpu_accessible {
        println!("MMap Aperture must be CPU accessible");
        return;
    }

    /* Reset NUMA policy */
    mbind(addr, size, MPOL_DEFAULT, std::ptr::null_mut(), 0, 0);

    /* Unmap memory */
    munmap(addr, size as usize);
}

pub unsafe fn init_mmap_apertures(
    base: u64,
    limit: u64,
    align: u32,
    guard_pages: u32,
) -> HsakmtStatus {
    let mut addr: *mut std::os::raw::c_void = std::ptr::null_mut();

    let page_size = global_page_size();

    let mut fmm_global = HSA_KMT_FMM_GLOBAL.lock().unwrap();

    if align > page_size as u32 {
        /* This should never happen. Alignment constraints
         * only apply to old GPUs that don't support 48-bit
         * virtual addresses.
         */
        println!("Falling back to reserved SVM apertures due to alignment constraints.");
        return HSAKMT_STATUS_ERROR;
    }

    let svm_default = SVM_DEFAULT as usize;

    /* Set up one SVM aperture */
    fmm_global.svm.apertures[svm_default].base = base as *mut std::os::raw::c_void;
    fmm_global.svm.apertures[svm_default].limit = limit as *mut std::os::raw::c_void;
    fmm_global.svm.apertures[svm_default].align = align as u64;
    fmm_global.svm.apertures[svm_default].guard_pages = guard_pages;
    fmm_global.svm.apertures[svm_default].is_cpu_accessible = true;
    fmm_global.svm.apertures[svm_default].ops = manageable_aperture_ops_t {
        allocate_area_aligned: Some(mmap_aperture_allocate_aligned),
        release_area: Some(mmap_aperture_release),
    };

    let svm_coherent = SVM_COHERENT as usize;

    fmm_global.svm.apertures[svm_coherent].base = std::ptr::null_mut();
    fmm_global.svm.apertures[svm_coherent].limit = std::ptr::null_mut();

    // let aperture = fmm_global.svm.apertures[svm_default].clone();
    let aperture = &fmm_global.svm.apertures[svm_default];

    // remove mutex lock
    // drop(fmm_global);

    /* Try to allocate one page. If it fails, we'll fall back to
     * managing our own reserved address range.
     */
    addr = aperture_allocate_area(&aperture, std::ptr::null_mut(), page_size as u64);
    if !addr.is_null() {
        aperture_release_area(&aperture, addr, page_size as u64);
        // let d = &fmm_global.svm.apertures[svm_default];

        // fmm_global.svm.dgpu_aperture = Some(d);
        // fmm_global.svm.dgpu_alt_aperture = Some(d);

        // println!("Initialized unreserved SVM apertures: {:?} - {:?}", aperture.base, aperture.limit);
    } else {
        println!("Failed to allocate unreserved SVM address space.");
        println!("Falling back to reserved SVM apertures.");
    }

    if !addr.is_null() {
        HSAKMT_STATUS_SUCCESS
    } else {
        HSAKMT_STATUS_ERROR
    }
}

pub unsafe fn init_svm_apertures(
    mut base: u64,
    mut limit: u64,
    align: u32,
    guard_pages: u32,
) -> HsakmtStatus {
    // let ADDR_INC = GPU_HUGE_PAGE_SIZE;

    // let mut found = false;

    // let mut addr: *mut std::os::raw::c_void = std::ptr::null_mut();
    // let mut ret_addr: *mut std::os::raw::c_void = std::ptr::null_mut();

    let dgpu_shared_aperture_limit = hsakmt_fmm_global_dgpu_shared_aperture_limit_get();

    /* If we already have an SVM aperture initialized (from a
     * parent process), keep using it
     */
    if !dgpu_shared_aperture_limit.is_null() {
        return HSAKMT_STATUS_SUCCESS;
    }

    /* Align base and limit to huge page size */
    base = ALIGN_UP(base, GPU_HUGE_PAGE_SIZE as u64);
    limit = ((limit + 1) & !(GPU_HUGE_PAGE_SIZE as u64 - 1)) - 1;

    /* If the limit is greater or equal 47-bits of address space,
     * it means we have GFXv9 or later GPUs only. We don't need
     * apertures to determine the MTYPE and the virtual address
     * space of the GPUs covers the full CPU address range (on
     * x86_64) or at least mmap is unlikely to run out of
     * addresses the GPUs can handle.
     */
    let reserve_svm = hsakmt_fmm_global_svm_reserve_svm_get();

    if limit >= ((1u64) << 47) - 1 && !reserve_svm {
        let status = init_mmap_apertures(base, limit, align, guard_pages);

        if status == HSAKMT_STATUS_SUCCESS {
            return status;
        }
        /* fall through: fall back to reserved address space */
    }

    // if (limit > SVM_RESERVATION_LIMIT) {
    //     limit = SVM_RESERVATION_LIMIT;
    // }
    if base >= limit {
        println!("No SVM range compatible with all GPU and software constraints\n");
        return HSAKMT_STATUS_ERROR;
    }

    panic!("TODO init_svm_apertures no complete");

    /* Try to reserve address space for SVM.
     *
     * Inner loop: try start addresses in huge-page increments up
     * to half the VM size we're trying to reserve
     *
     * Outer loop: reduce size of the allocation by factor 2 at a
     * time and print a warning for every reduction
     */

    // let mut len = limit - base + 1;
    //
    // loop {
    //     if !found && len >= SVM_MIN_VM_SIZE {
    //
    //         len = (len + 1) >> 1
    //     }
    //
    //     break;
    // }

    // for (len = limit - base + 1; !found && len >= SVM_MIN_VM_SIZE; len = (len + 1) >> 1) {
    // 	for (addr = (void *)base; (HSAuint64)addr + ((len + 1) >> 1) - 1 <= limit;
    // 	     addr = (void *)((HSAuint64)addr + ADDR_INC)) {
    // 		HSAuint64 top = MIN((HSAuint64)addr + len, limit+1);
    //
    // 		map_size = (top - (HSAuint64)addr) &
    // 			~(HSAuint64)(PAGE_SIZE - 1);
    // 		if (map_size < SVM_MIN_VM_SIZE)
    // 			break;
    //
    // 		ret_addr = reserve_address(addr, map_size);
    // 		if (!ret_addr)
    // 			break;
    // 		if ((HSAuint64)ret_addr + ((len + 1) >> 1) - 1 <= limit)
    // 			/* At least half the returned address
    // 			 * space is GPU addressable, we'll
    // 			 * take it
    // 			 */
    // 			break;
    // 		munmap(ret_addr, map_size);
    // 		ret_addr = NULL;
    // 	}
    // 	if (!ret_addr) {
    // 		pr_warn("Failed to reserve %uGB for SVM ...\n",
    // 			(unsigned int)(len >> 30));
    // 		continue;
    // 	}
    // 	if ((HSAuint64)ret_addr + SVM_MIN_VM_SIZE - 1 > limit) {
    // 		/* addressable size is less than the minimum */
    // 		pr_warn("Got %uGB for SVM at %p with only %dGB usable ...\n",
    // 			(unsigned int)(map_size >> 30), ret_addr,
    // 			(int)((limit - (HSAint64)ret_addr) >> 30));
    // 		munmap(ret_addr, map_size);
    // 		ret_addr = NULL;
    // 		continue;
    // 	} else {
    // 		found = true;
    // 		break;
    // 	}
    // }

    // if (!found) {
    // 	pr_err("Failed to reserve SVM address range. Giving up.\n");
    // 	return HSAKMT_STATUS_ERROR;
    // }
    //
    // base = (HSAuint64)ret_addr;
    // if (base + map_size - 1 > limit)
    // 	/* trim the tail that's not GPU-addressable */
    // 	munmap((void *)(limit + 1), base + map_size - 1 - limit);
    // else
    // 	limit = base + map_size - 1;
    //
    // /* init two apertures for non-coherent and coherent memory */
    // svm.apertures[SVM_DEFAULT].base  = dgpu_shared_aperture_base  = ret_addr;
    // svm.apertures[SVM_DEFAULT].limit = dgpu_shared_aperture_limit = (void *)limit;
    // svm.apertures[SVM_DEFAULT].align = align;
    // svm.apertures[SVM_DEFAULT].guard_pages = guard_pages;
    // svm.apertures[SVM_DEFAULT].is_cpu_accessible = true;
    // svm.apertures[SVM_DEFAULT].ops = &reserved_aperture_ops;
    //
    // /* Use the first 1/4 of the dGPU aperture as
    //  * alternate aperture for coherent access.
    //  * Base and size must be 64KB aligned.
    //  */
    // alt_base = (HSAuint64)svm.apertures[SVM_DEFAULT].base;
    // alt_size = (VOID_PTRS_SUB(svm.apertures[SVM_DEFAULT].limit,
    // 			  svm.apertures[SVM_DEFAULT].base) + 1) >> 2;
    // alt_base = (alt_base + 0xffff) & ~0xffffULL;
    // alt_size = (alt_size + 0xffff) & ~0xffffULL;
    // svm.apertures[SVM_COHERENT].base = (void *)alt_base;
    // svm.apertures[SVM_COHERENT].limit = (void *)(alt_base + alt_size - 1);
    // svm.apertures[SVM_COHERENT].align = align;
    // svm.apertures[SVM_COHERENT].guard_pages = guard_pages;
    // svm.apertures[SVM_COHERENT].is_cpu_accessible = true;
    // svm.apertures[SVM_COHERENT].ops = &reserved_aperture_ops;
    //
    // svm.apertures[SVM_DEFAULT].base = VOID_PTR_ADD(svm.apertures[SVM_COHERENT].limit, 1);
    //
    // pr_info("SVM alt (coherent): %12p - %12p\n",
    // 	svm.apertures[SVM_COHERENT].base, svm.apertures[SVM_COHERENT].limit);
    // pr_info("SVM (non-coherent): %12p - %12p\n",
    // 	svm.apertures[SVM_DEFAULT].base, svm.apertures[SVM_DEFAULT].limit);
    //
    // svm.dgpu_aperture = &svm.apertures[SVM_DEFAULT];
    // svm.dgpu_alt_aperture = &svm.apertures[SVM_COHERENT];

    // HSAKMT_STATUS_SUCCESS
}

pub unsafe fn fmm_set_memory_policy(
    gpu_id: u32,
    default_policy: i32,
    alt_policy: i32,
    alt_base: *mut u64,
    alt_size: u64,
) -> i32 {
    let mut args = kfd_ioctl_set_memory_policy_args {
        alternate_aperture_base: alt_base,
        alternate_aperture_size: alt_size,
        gpu_id,
        default_policy: default_policy as u32,
        alternate_policy: alt_policy as u32,
        pad: 0,
    };

    let hsakmt_kfd_fd = hsakmt_kfd_fd_get();

    let p_1 = (('K' as u64) << (0 + 8));
    let p_2 = ((std::mem::size_of::<kfd_ioctl_set_memory_policy_args>()) << ((0 + 8) + 8)) as u64;

    let AMDKFD_IOC_SET_MEMORY_POLICY = (((1) << (((0 + 8) + 8) + 14)) | p_1 | ((0x04) << 0) | p_2);

    hsakmt_ioctl(
        hsakmt_kfd_fd,
        AMDKFD_IOC_SET_MEMORY_POLICY,
        &mut args as *mut _ as *mut std::os::raw::c_void,
    )
}

pub fn fmm_init_rbtree(
    svm: &mut svm_t,
    cpuvm_aperture: &mut manageable_aperture_t,
    gpu_mem: &mut Vec<gpu_mem_t>,
) {
    // static int once;
    // int i = gpu_mem_count;
    // let mut i = hsakmt_fmm_global_gpu_mem_count_get();
    let svm_default = SVM_DEFAULT as usize;

    // if (once++ == 0) {
    rbtree_init(&mut svm.apertures[svm_default].tree);
    rbtree_init(&mut svm.apertures[svm_default].user_tree);
    rbtree_init(&mut svm.apertures[svm_default].tree);
    rbtree_init(&mut svm.apertures[svm_default].user_tree);
    rbtree_init(&mut cpuvm_aperture.tree);
    rbtree_init(&mut cpuvm_aperture.user_tree);
    // rbtree_init(&mem_handle_aperture.tree);
    // rbtree_init(&mem_handle_aperture.user_tree);
    // }

    // while i != 0 {
    // 	rbtree_init(&gpu_mem[i].scratch_physical.tree);
    // 	rbtree_init(&gpu_mem[i].scratch_physical.user_tree);
    // 	rbtree_init(&gpu_mem[i].gpuvm_aperture.tree);
    // 	rbtree_init(&gpu_mem[i].gpuvm_aperture.user_tree);
    //     i -= 1;
    // }

    for g_m in gpu_mem {
        rbtree_init(&mut g_m.scratch_physical.tree);
        rbtree_init(&mut g_m.scratch_physical.user_tree);
        rbtree_init(&mut g_m.gpuvm_aperture.tree);
        rbtree_init(&mut g_m.gpuvm_aperture.user_tree);
    }
}

pub fn init_mem_handle_aperture(_align: u32, _guard_pages: u32) -> bool {
    true
    // let found = false;
    //
    // /* init mem_handle_aperture for buffer handler management */
    // mem_handle_aperture.align = align;
    // mem_handle_aperture.guard_pages = guard_pages;
    // mem_handle_aperture.is_cpu_accessible = false;
    // mem_handle_aperture.ops = &reserved_aperture_ops;
    //
    // while (PORT_VPTR_TO_UINT64(mem_handle_aperture.base) < END_NON_CANONICAL_ADDR - 1) {
    //
    // 	found = true;
    // 	for (i = 0; i < gpu_mem_count; i++) {
    //
    // 		if (gpu_mem[i].lds_aperture.base &&
    // 			two_apertures_overlap(gpu_mem[i].lds_aperture.base, gpu_mem[i].lds_aperture.limit,
    // 								mem_handle_aperture.base, mem_handle_aperture.limit)) {
    // 				found = false;
    // 				break;
    // 		}
    //
    // 		if (gpu_mem[i].scratch_aperture.base &&
    // 			two_apertures_overlap(gpu_mem[i].scratch_aperture.base, gpu_mem[i].scratch_aperture.limit,
    // 								mem_handle_aperture.base, mem_handle_aperture.limit)){
    // 				found = false;
    // 				break;
    // 		}
    //
    // 		if (gpu_mem[i].gpuvm_aperture.base &&
    // 		   two_apertures_overlap(gpu_mem[i].gpuvm_aperture.base, gpu_mem[i].gpuvm_aperture.limit,
    // 								mem_handle_aperture.base, mem_handle_aperture.limit)){
    // 				found = false;
    // 				break;
    // 		}
    // 	}
    //
    // 	if (found) {
    // 		pr_info("mem_handle_aperture start %p, mem_handle_aperture limit %p\n",
    // 				mem_handle_aperture.base, mem_handle_aperture.limit);
    // 		return true;
    // 	} else {
    // 		/* increase base by 1UL<<47 to check next hole */
    // 		mem_handle_aperture.base =  VOID_PTR_ADD(mem_handle_aperture.base, (1UL << 47));
    // 		mem_handle_aperture.limit = VOID_PTR_ADD(mem_handle_aperture.base, (1ULL << 47));
    // 	}
    // }
    //
    // /* set invalid aperture if fail locating a hole for it */
    // mem_handle_aperture.base =  0;
    // mem_handle_aperture.limit = 0;
    //
    // false
}

pub unsafe fn hsakmt_topology_is_svm_needed(EngineId: &HSA_ENGINE_ID) -> bool {
    let hsakmt_is_dgpu = hsakmt_global_hsakmt_is_dgpu_get();

    if hsakmt_is_dgpu {
        return true;
    }

    if HSA_GET_GFX_VERSION_FULL(&EngineId.ui32) >= GFX_VERSION_VEGA10 as u32 {
        return true;
    }

    false
}

pub fn aperture_is_valid(
    app_base: *mut std::os::raw::c_void,
    app_limit: *mut std::os::raw::c_void,
) -> bool {
    if (!app_base.is_null() && !app_limit.is_null() && app_base < app_limit) {
        return true;
    }
    false
}

/* Wrapper functions to call aperture-specific VA management functions */
pub unsafe fn aperture_allocate_area_aligned(
    app: &manageable_aperture_t,
    address: *mut std::os::raw::c_void,
    MemorySizeInBytes: u64,
    align: u64,
) -> *mut std::os::raw::c_void {
    let some_f = app
        .ops
        .allocate_area_aligned
        .expect("aperture_allocate_area_aligned not found");

    let a = if align > 0 { align } else { app.align };

    some_f(app, address, MemorySizeInBytes, a)
}

pub unsafe fn fmm_translate_ioc_to_hsa_flags(ioc_flags: u32) -> HsaMemFlags {
    let mut mflags = HsaMemFlags {
        st: HsaMemFlagUnion {
            ui32: HsaMemFlagSt::default(),
        },
    };

    if (!(ioc_flags & KFD_IOC_ALLOC_MEM_FLAGS_WRITABLE as u32)) > 0 {
        mflags.st.ui32.ReadOnly = 1;
    }

    if (!(ioc_flags & KFD_IOC_ALLOC_MEM_FLAGS_COHERENT as u32)) > 0 {
        mflags.st.ui32.CoarseGrain = 1;
    }

    if (ioc_flags & KFD_IOC_ALLOC_MEM_FLAGS_EXT_COHERENT as u32) > 0 {
        mflags.st.ui32.ExtendedCoherent = 1;
    }

    if (ioc_flags & KFD_IOC_ALLOC_MEM_FLAGS_PUBLIC as u32) > 0 {
        mflags.st.ui32.HostAccess = 1;
    }

    mflags
}

pub fn vm_create_and_init_object(
    start: *mut std::os::raw::c_void,
    size: u64,
    handle: u64,
    mflags: HsaMemFlags,
) -> vm_object_t {
    let mut object = vm_object_t::default();

    // if (object) {
    object.start = start;
    object.userptr = std::ptr::null_mut();
    object.userptr_size = 0;
    object.size = size;
    object.handle = handle;
    object.registered_device_id_array_size = 0;
    object.mapped_device_id_array_size = 0;
    object.registered_device_id_array = std::ptr::null_mut();
    object.mapped_device_id_array = std::ptr::null_mut();
    object.registered_node_id_array = std::ptr::null_mut();
    object.mapped_node_id_array = std::ptr::null_mut();
    object.registration_count = 0;
    object.mapping_count = 0;
    object.mflags = mflags;
    object.metadata = std::ptr::null_mut();
    object.user_data = std::ptr::null_mut();
    object.is_imported_kfd_bo = false;
    object.node.key = rbtree_key(start as u64, size);
    object.user_node.key = rbtree_key(0, 0);
    // }

    object
}

/* returns 0 on success. Assumes, that fmm_mutex is locked on entry */
pub unsafe fn aperture_allocate_object(
    app: &manageable_aperture_t,
    new_address: *mut std::os::raw::c_void,
    handle: u64,
    MemorySizeInBytes: u64,
    mflags: HsaMemFlags,
) -> *mut vm_object_t {
    // let new_object: *mut vm_object_t = std::ptr::null_mut();

    /* Allocate new object */
    let mut new_object = vm_create_and_init_object(new_address, MemorySizeInBytes, handle, mflags);

    // if (!new_object) {
    //     println!("vm_create_and_init_object null");
    //     return std::ptr::null_mut();
    // }

    // hsakmt_rbtree_insert(&mut app.tree, &mut new_object.node);

    &mut new_object as *mut vm_object_t
}

/* After allocating the memory, return the vm_object created for this memory.
 * Return NULL if any failure.
 */
pub unsafe fn fmm_allocate_memory_object(
    gpu_id: u32,
    mem: *mut std::os::raw::c_void,
    MemorySizeInBytes: u64,
    aperture: &manageable_aperture_t,
    mmap_offset: &mut u64,
    ioc_flags: u32,
) -> *mut vm_object_t {
    let mut args = kfd_ioctl_alloc_memory_of_gpu_args {
        va_addr: std::ptr::null_mut(),
        size: 0,
        handle: 0,
        mmap_offset: 0,
        gpu_id,
        flags: 0,
    };
    let mut free_args = kfd_ioctl_free_memory_of_gpu_args { handle: 0 };

    // let vm_obj: *mut vm_object_t = std::ptr::null_mut();

    if mem.is_null() {
        return std::ptr::null_mut();
    }

    /* Allocate memory from amdkfd */
    args.gpu_id = gpu_id;
    args.size = MemorySizeInBytes;

    args.flags = ioc_flags | KFD_IOC_ALLOC_MEM_FLAGS_NO_SUBSTITUTE as u32;

    args.va_addr = mem as *mut u64;

    let hsakmt_is_dgpu = hsakmt_global_hsakmt_is_dgpu_get();

    let b = (ioc_flags & KFD_IOC_ALLOC_MEM_FLAGS_VRAM as u32);

    if !hsakmt_is_dgpu && b > 0 {
        args.va_addr = VOID_PTRS_SUB(mem, aperture.base) as *mut u64;
    }

    if (ioc_flags & KFD_IOC_ALLOC_MEM_FLAGS_USERPTR as u32) > 0 {
        args.mmap_offset = *mmap_offset;
    }

    /* if allocate vram-only, use an invalid VA */
    // if (aperture == &mem_handle_aperture) {
    //     args.va_addr = 0;
    // }

    let hsakmt_kfd_fd = hsakmt_kfd_fd_get();

    let p_1 = (('K' as u64) << (0 + 8));
    let p_2 = ((std::mem::size_of::<kfd_ioctl_alloc_memory_of_gpu_args>()) << ((0 + 8) + 8)) as u64;

    let AMDKFD_IOC_ALLOC_MEMORY_OF_GPU =
        (((2 | 1) << (((0 + 8) + 8) + 14)) | p_1 | ((0x16) << 0) | p_2);

    let r = hsakmt_ioctl(
        hsakmt_kfd_fd,
        AMDKFD_IOC_ALLOC_MEMORY_OF_GPU,
        &mut args as *mut _ as *mut std::os::raw::c_void,
    );

    if r > 0 {
        return std::ptr::null_mut();
    }

    let mflags = fmm_translate_ioc_to_hsa_flags(ioc_flags);

    /* Allocate object */
    let vm_obj = aperture_allocate_object(aperture, mem, args.handle, MemorySizeInBytes, mflags);

    if !vm_obj.is_null() {
        println!("aperture_allocate_object error");

        // free_args.handle = args.handle;
        // if (hsakmt_ioctl(hsakmt_kfd_fd, AMDKFD_IOC_FREE_MEMORY_OF_GPU, &free_args)) {
        //     pr_err("Failed to free GPU memory with handle: 0x%llx\n", free_args.handle);
        // }

        return std::ptr::null_mut();
    }

    if *mmap_offset > 0 {
        *mmap_offset = args.mmap_offset;
    }

    vm_obj
}

pub unsafe fn __fmm_allocate_device(
    gpu_id: u32,
    address: *mut std::os::raw::c_void,
    MemorySizeInBytes: u64,
    aperture: &manageable_aperture_t,
    mmap_offset: &mut u64,
    ioc_flags: u32,
    alignment: u64,
    vm_obj: *mut *mut vm_object_t,
) -> *mut std::os::raw::c_void {
    let mut mem: *mut std::os::raw::c_void = std::ptr::null_mut();
    let obj: *mut vm_object_t = std::ptr::null_mut();

    /* Check that aperture is properly initialized/supported */
    if !aperture_is_valid(aperture.base, aperture.limit) {
        return std::ptr::null_mut();
    }

    /* Allocate address space */
    let mut mem = aperture_allocate_area_aligned(aperture, address, MemorySizeInBytes, alignment);

    if mem.is_null() {
        println!("aperture_allocate_area_aligned is_null");
        return std::ptr::null_mut();
    }

    /*
     * Now that we have the area reserved, allocate memory in the device
     * itself
     */
    let obj = fmm_allocate_memory_object(
        gpu_id,
        mem,
        MemorySizeInBytes,
        aperture,
        mmap_offset,
        ioc_flags,
    );

    if !obj.is_null() {
        /*
         * allocation of memory in device failed.
         * Release region in aperture
         */
        aperture_release_area(aperture, mem, MemorySizeInBytes);

        /* Assign NULL to mem to indicate failure to calling function */
        mem = std::ptr::null_mut();
    }

    if !vm_obj.is_null() {
        *vm_obj = obj;
    }

    mem
}

pub unsafe fn map_mmio(
    node_id: u32,
    gpu_id: u32,
    mmap_fd: i32,
    svm: &svm_t,
) -> *mut std::os::raw::c_void {
    let aperture = svm.dgpu_alt_aperture_alt().unwrap();

    let mut vm_obj: *mut vm_object_t = std::ptr::null_mut();

    let mflags = HsaMemFlags {
        st: HsaMemFlagUnion { Value: 0 },
    };

    let mut mmap_offset: u64 = 0;

    /* Allocate physical memory and vm object*/
    let ioc_flags = KFD_IOC_ALLOC_MEM_FLAGS_MMIO_REMAP
        | KFD_IOC_ALLOC_MEM_FLAGS_WRITABLE
        | KFD_IOC_ALLOC_MEM_FLAGS_COHERENT;

    let page_size = global_page_size();

    let mem = __fmm_allocate_device(
        gpu_id,
        std::ptr::null_mut(),
        page_size as u64,
        aperture,
        &mut mmap_offset,
        ioc_flags as u32,
        0,
        &mut vm_obj,
    );

    if (mem.is_null() || vm_obj.is_null()) {
        return std::ptr::null_mut();
    }

    // mflags.Value = 0;
    // mflags.ui32.NonPaged = 1;
    // mflags.ui32.HostAccess = 1;
    // pthread_mutex_lock(&aperture->fmm_mutex);
    // vm_obj->mflags = mflags;
    // vm_obj->node_id = node_id;
    // pthread_mutex_unlock(&aperture->fmm_mutex);
    //
    // /* Map for CPU access*/
    // ret = mmap(mem, PAGE_SIZE,
    // 		 PROT_READ | PROT_WRITE,
    // 		 MAP_SHARED | MAP_FIXED, mmap_fd,
    // 		 mmap_offset);
    // if (ret == MAP_FAILED) {
    // 	__fmm_release(vm_obj, aperture);
    // 	return NULL;
    // }
    //
    // /* Map for GPU access*/
    // if (hsakmt_fmm_map_to_gpu(mem, PAGE_SIZE, NULL)) {
    // 	__fmm_release(vm_obj, aperture);
    // 	return NULL;
    // }

    mem
}

pub unsafe fn hsakmt_fmm_init_process_apertures(NumNodes: u32) -> HsakmtStatus {
    let topology_global = HSA_KMT_TOPOLOGY_GLOBAL.lock().unwrap();
    // let fmm_global = HSA_KMT_FMM_GLOBAL.lock().unwrap(); // error

    let mut svm = svm_t::default();

    let guardPages: u32 = 1;

    let zero_str = CString::new("0").unwrap();

    /* If HSA_DISABLE_CACHE is set to a non-0 value, disable caching */
    let env_str = CString::new("HSA_DISABLE_CACHE").unwrap();
    let disableCache = getenv(env_str.as_ptr());
    let b = !disableCache.is_null() && strcmp(disableCache, zero_str.as_ptr()) == 0;
    svm.disable_cache = b;

    /* If HSA_USERPTR_FOR_PAGED_MEM is not set or set to a non-0
     * value, enable userptr for all paged memory allocations
     */
    // let env_str = CString::new("HSA_USERPTR_FOR_PAGED_MEM").unwrap();
    // let pagedUserptr = getenv(env_str.as_ptr());
    // svm.userptr_for_paged_mem = !pagedUserptr.is_null() || strcmp(pagedUserptr, zero_str.as_ptr()) == 0;

    /* If HSA_CHECK_USERPTR is set to a non-0 value, check all userptrs
     * when they are registered
     */
    let env_str = CString::new("HSA_CHECK_USERPTR").unwrap();
    let checkUserptr = getenv(env_str.as_ptr());
    svm.check_userptr = !checkUserptr.is_null() && strcmp(checkUserptr, zero_str.as_ptr()) == 0;

    /* If HSA_RESERVE_SVM is set to a non-0 value,
     * enable packet capture and replay mode.
     */
    let env_str = CString::new("HSA_RESERVE_SVM").unwrap();
    let reserveSvm = getenv(env_str.as_ptr());
    svm.reserve_svm = !reserveSvm.is_null() && strcmp(reserveSvm, zero_str.as_ptr()) == 0;

    // let format_cs = CString::new("%u").unwrap();

    /* Specify number of guard pages for SVM apertures, default is 1 */
    // let env_str = CString::new("HSA_SVM_GUARD_PAGES").unwrap();
    // let guardPagesStr = getenv(env_str.as_ptr());
    // if !guardPagesStr.is_null() || sscanf(guardPagesStr, format_cs.as_ptr(), &guardPages) != 1 {
    //     guardPages = 1;
    // }

    /* Sets the max VA alignment order size during mapping. By default the order
     * size is set to 9(2MB)
     */
    // let env_str = CString::new("HSA_MAX_VA_ALIGN").unwrap();
    // let maxVaAlignStr = getenv(env_str.as_ptr());
    // if !maxVaAlignStr.is_null() || sscanf(maxVaAlignStr, format_cs.as_ptr(), &svm.alignment_order) != 1 {
    //     svm.alignment_order = 9;
    // }
    svm.alignment_order = 9;

    let mut gpu_mem: Vec<gpu_mem_t> = Vec::with_capacity(NumNodes as usize);

    /* Initialize gpu_mem[] from sysfs topology. Rest of the members are
     * set to 0 by calloc. This is necessary because this function
     * gets called before hsaKmtAcquireSystemProperties() is called.
     */

    #[allow(clippy::field_reassign_with_default)]
    for i in 0..NumNodes {
        let props = topology_global.hsakmt_topology_get_node_props(i);

        println!("node i {}", i);
        hsakmt_topology_setup_is_dgpu_param(props);

        /* Skip non-GPU nodes */
        if props.KFDGpuID > 0 {
            let fd = hsakmt_open_drm_render_device(props.DrmRenderMinor);
            if fd <= 0 {
                return HSAKMT_STATUS_ERROR;
            }

            let mut gpu_m = gpu_mem_t::default();

            gpu_m.drm_render_minor = props.DrmRenderMinor as u32;
            gpu_m.usable_peer_id_array.push(props.KFDGpuID);
            gpu_m.usable_peer_id_num = 1;

            gpu_m.EngineId.ui32.Major = props.EngineId.ui32.Major;
            gpu_m.EngineId.ui32.Minor = props.EngineId.ui32.Minor;
            gpu_m.EngineId.ui32.Stepping = props.EngineId.ui32.Stepping;

            gpu_m.drm_render_fd = fd;
            gpu_m.gpu_id = props.KFDGpuID;
            gpu_m.local_mem_size = props.LocalMemSize;
            gpu_m.device_id = props.DeviceId as u32;
            gpu_m.node_id = i;
            hsakmt_global_is_svm_api_supported_set(props.Capability.ui32.SVMAPISupported > 0);

            gpu_m.scratch_physical.align = global_page_size() as u64;
            gpu_m.scratch_physical.ops = manageable_aperture_ops_t {
                allocate_area_aligned: None,
                release_area: None,
            };

            gpu_m.gpuvm_aperture.align = get_vm_alignment(props.DeviceId as u32) as u64;
            gpu_m.gpuvm_aperture.guard_pages = guardPages;
            gpu_m.gpuvm_aperture.ops = manageable_aperture_ops_t {
                allocate_area_aligned: None,
                release_area: None,
            };

            gpu_mem.push(gpu_m);
        }
    }

    /* The ioctl will also return Number of Nodes if
     * args.kfd_process_device_apertures_ptr is set to NULL. This is not
     * required since Number of nodes is already known. Kernel will fill in
     * the apertures in kfd_process_device_apertures_ptr
     */
    let mut num_of_sysfs_nodes = topology_global.num_sysfs_nodes as u32;
    if num_of_sysfs_nodes < gpu_mem.len() as u32 {
        return HSAKMT_STATUS_ERROR;
    }

    let mut process_apertures =
        vec![kfd_process_device_apertures::default(); num_of_sysfs_nodes as usize];

    // let process_apertures = calloc(
    //     num_of_sysfs_nodes as usize,
    //     std::mem::size_of::<kfd_process_device_apertures>(),
    // ) as *mut kfd_process_device_apertures;
    //
    // if process_apertures.is_null() {
    //     return HSAKMT_STATUS_NO_MEMORY;
    // }

    // println!("process_apertures {:#?}", process_apertures);

    /* GPU Resource management can disable some of the GPU nodes.
     * The Kernel driver could be not aware of this.
     * Get from Kernel driver information of all the nodes and then filter it.
     */
    let ret = get_process_apertures(process_apertures.as_mut_ptr(), &mut num_of_sysfs_nodes);
    if ret != HSAKMT_STATUS_SUCCESS {
        return ret;
    }

    // let slice_process_apertures = std::ptr::slice_from_raw_parts_mut(process_apertures.as_mut_ptr(), num_of_sysfs_nodes as usize);
    // println!("num_of_sysfs_nodes {}", num_of_sysfs_nodes);
    //
    // let process_apertures_ref = &mut *(slice_process_apertures);
    // for p in process_apertures_ref {
    //     println!("{:#?}", p);
    // }

    // println!("process_apertures {:#?}", process_apertures);

    let mut svm_base: u64 = 0;
    let mut svm_limit: u64 = 0;
    let mut svm_alignment: u32 = 0;

    let mut all_gpu_id_array: Vec<u32> = Vec::with_capacity(gpu_mem.len());

    for i in 0..num_of_sysfs_nodes as usize {
        /* Map Kernel process device data node i <--> gpu_mem_id which
         * indexes into gpu_mem[] based on gpu_id
         */
        // let gpu_mem_id = gpu_mem_find_by_gpu_id(process_apertures[i].gpu_id);
        let mut gpu_mem_id: i32 = -1;

        for (i, gpu_m) in gpu_mem.iter().enumerate() {
            if gpu_m.gpu_id == process_apertures[i].gpu_id {
                gpu_mem_id = i as i32;
            }
        }

        // println!("gpu_mem_id i {} - {}", i, gpu_mem_id);

        if gpu_mem_id < 0 {
            continue;
        }

        let gpu_mem_id = gpu_mem_id as usize;

        all_gpu_id_array.push(gpu_mem_id as u32);

        /* Add this GPU to the usable_peer_id_arrays of all GPUs that
         * this GPU has an IO link to. This GPU can map memory
         * allocated on those GPUs.
         */
        let nodeId = gpu_mem[gpu_mem_id].node_id;
        let nodeProps = topology_global.hsakmt_topology_get_node_props(nodeId);

        assert!(nodeProps.NumIOLinks <= NumNodes);
        let linkProps = topology_global.hsakmt_topology_get_iolink_props(nodeId);

        for linkProp in linkProps {
            let to_gpu_mem_id = gpu_mem_find_by_gpu_id(linkProp.NodeTo);

            if to_gpu_mem_id < 0 {
                continue;
            }

            assert!(gpu_mem[to_gpu_mem_id as usize].usable_peer_id_num < NumNodes);
            let peer = gpu_mem[to_gpu_mem_id as usize].usable_peer_id_num;
            gpu_mem[to_gpu_mem_id as usize].usable_peer_id_num += 1;
            gpu_mem[to_gpu_mem_id as usize].usable_peer_id_array[peer as usize] =
                gpu_mem[gpu_mem_id].gpu_id;
        }

        gpu_mem[gpu_mem_id].lds_aperture.base =
            process_apertures[i].lds_base as *mut std::os::raw::c_void;
        gpu_mem[gpu_mem_id].lds_aperture.limit =
            process_apertures[i].lds_limit as *mut std::os::raw::c_void;

        gpu_mem[gpu_mem_id].scratch_aperture.base =
            process_apertures[i].scratch_base as *mut std::os::raw::c_void;
        gpu_mem[gpu_mem_id].scratch_aperture.limit =
            process_apertures[i].scratch_limit as *mut std::os::raw::c_void;

        if IS_CANONICAL_ADDR(process_apertures[i].gpuvm_limit) {
            let vm_alignment = get_vm_alignment(gpu_mem[gpu_mem_id].device_id);

            /* Set proper alignment for scratch backing aperture */
            gpu_mem[gpu_mem_id].scratch_physical.align = vm_alignment as u64;

            /* Non-canonical per-ASIC GPUVM aperture does
             * not exist on dGPUs in GPUVM64 address mode
             */
            gpu_mem[gpu_mem_id].gpuvm_aperture.base = std::ptr::null_mut();
            gpu_mem[gpu_mem_id].gpuvm_aperture.limit = std::ptr::null_mut();

            /* Update SVM aperture limits and alignment */
            if process_apertures[i].gpuvm_base > svm_base {
                svm_base = process_apertures[i].gpuvm_base;
            }
            if process_apertures[i].gpuvm_limit < svm_limit || svm_limit == 0 {
                svm_limit = process_apertures[i].gpuvm_limit;
            }
            if vm_alignment > svm_alignment {
                svm_alignment = vm_alignment;
            }
        } else {
            gpu_mem[gpu_mem_id].gpuvm_aperture.base =
                process_apertures[i].gpuvm_base as *mut std::os::raw::c_void;
            gpu_mem[gpu_mem_id].gpuvm_aperture.limit =
                process_apertures[i].gpuvm_limit as *mut std::os::raw::c_void;
            /* Reserve space at the start of the
             * aperture. After subtracting the base, we
             * don't want valid pointers to become NULL.
             */
            // aperture_allocate_area(
            //     &gpu_mem[gpu_mem_id].gpuvm_aperture,
            //     std::ptr::null_mut(),
            //     gpu_mem[gpu_mem_id].gpuvm_aperture.align);
        }

        /* Acquire the VM from the DRM render node for KFD use */
        let ret = acquire_vm(
            gpu_mem[gpu_mem_id].gpu_id,
            gpu_mem[gpu_mem_id].drm_render_fd,
        );
        if ret != HSAKMT_STATUS_SUCCESS {
            return ret;
        }
    }

    if svm_limit > 0 {
        /* At least one GPU uses GPUVM in canonical address
         * space. Set up SVM apertures shared by all such GPUs
         */
        let ret = init_svm_apertures(svm_base, svm_limit, svm_alignment, guardPages);
        if ret != HSAKMT_STATUS_SUCCESS {
            return ret;
        }

        for process_aperture in process_apertures.iter() {
            if !IS_CANONICAL_ADDR(process_aperture.gpuvm_limit) {
                continue;
            }

            /* Set memory policy to match the SVM apertures */
            // let alt_base = svm.dgpu_alt_aperture_get_mut().unwrap();
            let alt_base = &mut svm.apertures[SVM_DEFAULT as usize];

            let alt_size = VOID_PTRS_SUB(alt_base.limit, alt_base.base) + 1;

            let d_c = if svm.disable_cache {
                KFD_IOC_CACHE_POLICY_COHERENT
            } else {
                KFD_IOC_CACHE_POLICY_NONCOHERENT
            };

            let a_b = alt_base as *mut _ as *mut std::os::raw::c_void;

            let err = fmm_set_memory_policy(
                process_aperture.gpu_id,
                d_c as i32,
                KFD_IOC_CACHE_POLICY_COHERENT as i32,
                a_b as *mut u64,
                alt_size,
            );

            if err > 0 {
                println!(
                    "Failed to set mem policy for GPU {} {}",
                    process_aperture.gpu_id, err
                );
                return HSAKMT_STATUS_ERROR;
            }
        }
    }

    let mut cpuvm_aperture = manageable_aperture_t::default();

    let page_size = global_page_size();

    cpuvm_aperture.align = page_size as u64;
    cpuvm_aperture.limit = 0x7FFFFFFFFFFF as *mut std::os::raw::c_void; /* 2^47 - 1 */

    fmm_init_rbtree(&mut svm, &mut cpuvm_aperture, &mut gpu_mem);

    if (!init_mem_handle_aperture(page_size as u32, guardPages)) {
        println!("Failed to init mem_handle_aperture\n");
    }

    let hsakmt_kfd_fd = hsakmt_kfd_fd_get();

    for g_m in gpu_mem.iter_mut() {
        if !hsakmt_topology_is_svm_needed(&g_m.EngineId) {
            // println!("hsakmt_topology_is_svm_needed no");
            continue;
        }

        // println!("hsakmt_topology_is_svm_needed yes");

        g_m.mmio_aperture.base = map_mmio(g_m.node_id, g_m.gpu_id, hsakmt_kfd_fd, &svm);

        if !g_m.mmio_aperture.base.is_null() {
            let pt = (g_m.mmio_aperture.base as *mut u8).add((page_size - 1) as usize);
            let r = pt.add((page_size - 1) as usize);

            g_m.mmio_aperture.limit = r as *mut std::os::raw::c_void;
        } else {
            // println!("Failed to map remapped mmio page on gpu_mem {}", g_m.gpu_id);
            panic!("Failed to map remapped mmio page on gpu_mem {}", g_m.gpu_id);
        }
    }

    hsakmt_fmm_global_gpu_mem_set(gpu_mem);
    hsakmt_fmm_global_svm_set(svm);

    HSAKMT_STATUS_SUCCESS
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_globals() {
//         let mut drm_render_fds = HSA_KMT_FMM_GLOBAL.lock().unwrap();
//         println!("{:?}", drm_render_fds);
//         drm_render_fds.drm_render_fd_set(0, 32);
//         drm_render_fds.amdgpu_handle_set(0, amdgpu_device { _unused: [] });
//         println!("{:?}", drm_render_fds);
//
//         // TODO assert
//     }
// }
