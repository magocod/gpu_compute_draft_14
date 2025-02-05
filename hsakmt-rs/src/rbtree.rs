#![allow(
    non_camel_case_types,
    non_snake_case,
    dead_code,
    non_upper_case_globals,
    clippy::enum_clike_unportable_variant,
    clippy::mixed_case_hex_literals
)]

use crate::rbtree_amd::{rbtree_key_s, rbtree_key_t};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct rbtree_node_s<'a> {
    pub key: rbtree_key_t,
    pub left: Option<&'a rbtree_node_t<'a>>,
    pub right: Option<&'a rbtree_node_t<'a>>,
    pub parent: Option<&'a rbtree_node_t<'a>>,
    pub color: u8,
    pub data: u8,
}

impl Default for rbtree_node_s<'_> {
    fn default() -> Self {
        Self {
            key: rbtree_key_s { addr: 0, size: 0 },
            left: None,
            right: None,
            parent: None,
            color: 0,
            data: 0,
        }
    }
}

pub type rbtree_node_t<'a> = rbtree_node_s<'a>;

#[derive(Debug)]
pub struct rbtree_s<'a> {
    pub root: Option<&'a mut rbtree_node_t<'a>>,
    pub sentinel: Option<rbtree_node_t<'a>>,
}

pub type rbtree_t<'a> = rbtree_s<'a>;

// #define rbt_red(node)			((node)->color = 1)
// #define rbt_black(node)			((node)->color = 0)
// #define rbt_is_red(node)		((node)->color)
// #define rbt_is_black(node)		(!rbt_is_red(node))
// #define rbt_copy_color(n1, n2)		(n1->color = n2->color)
//
// /* a sentinel must be black */
//
// #define rbtree_sentinel_init(node)	rbt_black(node)}

// #define rbtree_init(tree)				\
// rbtree_sentinel_init(&(tree)->sentinel);	\
// (tree)->root = &(tree)->sentinel;

pub fn rbt_black(node: &mut rbtree_node_t) {
    node.color = 0;
}

pub fn rbtree_sentinel_init(node: &mut rbtree_node_t) {
    rbt_black(node);
}

pub fn rbtree_init(tree: &mut rbtree_t) {
    match &mut tree.sentinel {
        None => {
            // panic!("rbtree_init sentinel = None");
            let mut sentinel = rbtree_node_t::default();
            rbtree_sentinel_init(&mut sentinel);
            tree.sentinel = Some(sentinel);
        }
        Some(sentinel) => rbtree_sentinel_init(sentinel),
    }
}

pub fn hsakmt_rbtree_insert<'a>(tree: &'a mut rbtree_s<'a>, node: &'a mut rbtree_node_t<'a>) {
    // rbtree_node_t  **root, *temp, *sentinel;

    /* a binary tree insert */

    let root = match &mut tree.root {
        Some(r) => r,
        None => panic!("tree.root none"),
    };
    let sentinel = match &mut tree.sentinel {
        None => panic!("tree.sentinel none"),
        Some(s) => s,
    };

    // let k = root.key;
    // let k2 = &sentinel.key;

    if root.key.eq(&sentinel.key) {
        node.parent = None;
        node.left = Some(sentinel);
        node.right = Some(sentinel);
        rbt_black(node);

        tree.root = Some(node);

        return;
    }

    // hsakmt_rbtree_insert_value(*root, node, sentinel);
    //
    // /* re-balance tree */
    //
    // while (node != *root && rbt_is_red(node->parent)) {
    //
    // 	if (node->parent == node->parent->parent->left) {
    // 		temp = node->parent->parent->right;
    //
    // 		if (rbt_is_red(temp)) {
    // 			rbt_black(node->parent);
    // 			rbt_black(temp);
    // 			rbt_red(node->parent->parent);
    // 			node = node->parent->parent;
    //
    // 		} else {
    // 			if (node == node->parent->right) {
    // 				node = node->parent;
    // 				rbtree_left_rotate(root, sentinel, node);
    // 			}
    //
    // 			rbt_black(node->parent);
    // 			rbt_red(node->parent->parent);
    // 			rbtree_right_rotate(root, sentinel, node->parent->parent);
    // 		}
    //
    // 	} else {
    // 		temp = node->parent->parent->left;
    //
    // 		if (rbt_is_red(temp)) {
    // 			rbt_black(node->parent);
    // 			rbt_black(temp);
    // 			rbt_red(node->parent->parent);
    // 			node = node->parent->parent;
    //
    // 		} else {
    // 			if (node == node->parent->left) {
    // 				node = node->parent;
    // 				rbtree_right_rotate(root, sentinel, node);
    // 			}
    //
    // 			rbt_black(node->parent);
    // 			rbt_red(node->parent->parent);
    // 			rbtree_left_rotate(root, sentinel, node->parent->parent);
    // 		}
    // 	}
    // }

    rbt_black(root);
}
