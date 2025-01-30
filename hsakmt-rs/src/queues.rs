use crate::hsakmttypes::{
    GFX_VERSION_ALDEBARAN, GFX_VERSION_AQUA_VANJARAM, GFX_VERSION_ARCTURUS, GFX_VERSION_GFX1200,
    GFX_VERSION_GFX1201, GFX_VERSION_PLUM_BONITO, GFX_VERSION_WHEAT_NAS,
};

pub fn hsakmt_get_vgpr_size_per_cu(gfxv: u32) -> u32 {
    let mut vgpr_size = 0x40000;

    if (gfxv & !0xff) == GFX_VERSION_AQUA_VANJARAM as u32
        || gfxv == GFX_VERSION_ALDEBARAN as u32
        || gfxv == GFX_VERSION_ARCTURUS as u32
    {
        vgpr_size = 0x80000;
    } else if gfxv == GFX_VERSION_PLUM_BONITO as u32
        || gfxv == GFX_VERSION_WHEAT_NAS as u32
        || gfxv == GFX_VERSION_GFX1200 as u32
        || gfxv == GFX_VERSION_GFX1201 as u32
    {
        vgpr_size = 0x60000;
    }

    vgpr_size
}
