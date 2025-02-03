#![allow(
    non_camel_case_types,
    non_snake_case,
    dead_code,
    non_upper_case_globals,
    clippy::enum_clike_unportable_variant,
    clippy::mixed_case_hex_literals
)]

use crate::rbtree_amd::rbtree_key_t;

#[derive(Debug, Clone)]
pub struct rbtree_node_s<'a> {
    pub key: rbtree_key_t,
    pub left: &'a rbtree_node_t,
    pub right: &'a rbtree_node_t,
    pub parent: &'a rbtree_node_t,
    pub color: u8,
    pub data: u8,
}

pub type rbtree_node_t = rbtree_node_s<'static>;

#[derive(Debug, Clone)]
pub struct rbtree_s<'a> {
    pub root: Option<&'a rbtree_node_t>,
    pub sentinel: Option<rbtree_node_t>,
}

pub type rbtree_t = rbtree_s<'static>;
