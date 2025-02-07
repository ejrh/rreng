use bevy::prelude::{Component, Entity, Reflect};
use ndarray::Array2;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
pub struct BlockId {
    pub row: usize,
    pub col: usize,
    pub level: usize,
}

#[derive(Debug, PartialEq)]
pub enum BlockKind {
    Pending,
    Populated(Entity),
    Invalid,
}

pub struct Entry {
    pub kind: BlockKind,
}

pub struct Level {
    entries: Array2<Entry>,
}

#[derive(Component)]
pub struct MeshTree {
    height: usize,
    width: usize,
    pub levels: Vec<Level>,
}

impl MeshTree {
    pub fn new(num_blocks: [usize; 2], max_level: usize) -> Self {
        let [height, width] = num_blocks;
        let girth = height.min(width);
        let max_level = ((girth as f32).log2().ceil() as usize).min(max_level);
        let max_block_size = 1 << max_level;

        let mut level_height = height.div_ceil(max_block_size) * max_block_size;
        let mut level_width = width.div_ceil(max_block_size) * max_block_size;
        let mut valid_height = height;
        let mut valid_width = width;

        let mut levels = Vec::new();
        for _ in 0..=max_level {
            levels.push(Level {
                entries: Array2::from_shape_fn((level_height, level_width), |(i, j)| {
                    let valid = i < valid_height && j < valid_width;
                    let kind = if valid { BlockKind::Pending } else { BlockKind::Invalid };
                    Entry { kind }
                })
            });

            level_height /= 2;
            level_width /= 2;
            valid_height /= 2;
            valid_width /= 2;
        }

        MeshTree {
            height,
            width,
            levels,
        }
    }

    pub fn parent(&self, block_id: BlockId) -> BlockId {
        BlockId {
            row: block_id.row / 2,
            col: block_id.col / 2,
            level: block_id.level + 1,
        }
    }

    pub fn children(&self, block_id: BlockId) -> Vec<BlockId> {
        if block_id.level == 0 { return Vec::new() }

        let inds = [0, 1, 2, 3];
        let children = inds.iter().map(|ind| BlockId {
            row: block_id.row * 2 + (ind >> 1) as usize,
            col: block_id.col * 2 + (ind & 1) as usize,
            level: block_id.level - 1,
        }).collect();

        children
    }

    pub fn ancestors(&self, block_id: BlockId) -> Vec<BlockId> {
        let mut results = Vec::new();

        let BlockId { mut row, mut col, .. } = block_id;
        for lvl in block_id.level + 1..self.levels.len() {
            row /= 2;
            col /= 2;
            results.push(BlockId { row, col, level: lvl });
        }

        results
    }

    pub fn descendants(&self, block_id: BlockId) -> Vec<BlockId> {
        if block_id.level == 0 { return Vec::new() }

        let mut results = Vec::new();

        let mut children = vec![block_id];
        while !children.is_empty() {
            results.extend(children.clone());
            let grandchildren = children.iter()
                .flat_map(|c| self.children(*c))
                .collect();
            children = grandchildren;
        }

        results
    }

    pub fn get_entry(&self, block_id: BlockId) -> &Entry {
        &self.levels[block_id.level].entries[(block_id.row, block_id.col)]
    }

    fn get_entry_mut(&mut self, block_id: BlockId) -> &mut Entry {
        &mut self.levels[block_id.level].entries[(block_id.row, block_id.col)]
    }

    pub fn set_mesh(&mut self, block_id: BlockId, kind: BlockKind) -> Option<Entity> {
        let entry = self.get_entry_mut(block_id);
        let old_kind = std::mem::replace(&mut entry.kind, kind);
        if let BlockKind::Populated(old_id) = old_kind {
            Some(old_id)
        } else { None }
    }

    pub fn valid(&self, block_id: BlockId) -> bool {
        !matches!(self.get_entry(block_id).kind, BlockKind::Invalid)
    }

    pub fn populated(&self, block_id: BlockId) -> bool {
        matches!(self.get_entry(block_id).kind, BlockKind::Populated(_))
    }

    pub fn walk(&self, visit: &mut impl FnMut(&MeshTree, BlockId) -> bool) {
        fn walk_block(tree: &MeshTree, block_id: BlockId,
                      visit: &mut impl FnMut(&MeshTree, BlockId) -> bool) {
            if visit(tree, block_id) {
                for child in tree.children(block_id) {
                    walk_block(tree, child, visit);
                }
            }
        }

        let lvl = self.levels.len() - 1;
        let roots = &self.levels[lvl];
        for ((i, j), _entry) in roots.entries.indexed_iter() {
            let block_id = BlockId { row: i, col: j, level: lvl };
            walk_block(self, block_id, visit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let tree = MeshTree::new([2, 2], 1);
        assert_eq!(tree.height, 2);
        assert_eq!(tree.width, 2);
        assert_eq!(tree.levels.len(), 2);
        assert_eq!(tree.levels[0].entries.dim(), (2, 2));
        assert_eq!(tree.levels[1].entries.dim(), (1, 1));
        assert_eq!(tree.levels[1].entries[(0, 0)].kind, BlockKind::Pending);
    }

    #[test]
    fn test_irregular() {
        let tree = MeshTree::new([3, 7], 2);
        assert_eq!(tree.height, 3);
        assert_eq!(tree.width, 7);
        assert_eq!(tree.levels.len(), 3);
        assert_eq!(tree.levels[0].entries.dim(), (4, 8));
        assert_eq!(tree.levels[1].entries.dim(), (2, 4));
        assert_eq!(tree.levels[2].entries.dim(), (1, 2));
        assert_eq!(tree.levels[0].entries[(3, 7)].kind, BlockKind::Invalid);
        assert_eq!(tree.levels[1].entries[(1, 3)].kind, BlockKind::Invalid);
        assert_eq!(tree.levels[2].entries[(0, 1)].kind, BlockKind::Invalid);
    }

    #[test]
    fn test_structure() {
        let tree = MeshTree::new([2, 2], 1);
        let root = BlockId { row: 0, col: 0, level: 1 };
        let leaves = [
            BlockId { row: 0, col: 0, level: 0 },
            BlockId { row: 0, col: 1, level: 0 },
            BlockId { row: 1, col: 0, level: 0 },
            BlockId { row: 1, col: 1, level: 0 },
        ];
        assert_eq!(tree.ancestors(root), vec![], "ancestors(root)");
        assert_eq!(tree.ancestors(leaves[0]), vec![root], "ancestors(child)");
        assert_eq!(tree.children(root), leaves, "children");

        let mut num_visited = 0;
        let nv = &mut num_visited;
        tree.walk(&mut |_, _| { *nv += 1; true });
        assert_eq!(num_visited, 5, "num_visited");
    }
}
