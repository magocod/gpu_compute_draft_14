#![allow(
    non_camel_case_types,
    non_snake_case,
    dead_code,
    non_upper_case_globals,
    clippy::enum_clike_unportable_variant,
    clippy::mixed_case_hex_literals
)]

const ADDR_BIT: usize = 0;
const SIZE_BIT: usize = 1;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct rbtree_key_s {
    pub addr: u64,
    pub size: u64,
}

pub type rbtree_key_t = rbtree_key_s;

pub fn rbtree_key(addr: u64, size: u64) -> rbtree_key_t {
    rbtree_key_t { addr, size }
}
