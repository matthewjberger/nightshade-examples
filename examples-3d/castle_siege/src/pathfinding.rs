use nightshade::prelude::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::castle;

pub const NODE_WELL: usize = 0;
pub const NODE_ARMORY: usize = 1;
pub const NODE_HEALING: usize = 2;
pub const NODE_REPAIR_PILE: usize = 3;
pub const NODE_GATE: usize = 4;
pub const NODE_RIVER: usize = 5;
pub const NODE_ARCHER_NW: usize = 6;
pub const NODE_ARCHER_NE: usize = 7;
pub const NODE_ARCHER_SW: usize = 8;
pub const NODE_ARCHER_SE: usize = 9;
pub const NODE_CENTER: usize = 10;
pub const NODE_NORTH_CENTER: usize = 11;
pub const NODE_SOUTH_CENTER: usize = 12;
pub const NODE_EAST_CENTER: usize = 13;
pub const NODE_WEST_CENTER: usize = 14;
pub const NODE_NW_COURTYARD: usize = 15;
pub const NODE_NE_COURTYARD: usize = 16;
pub const NODE_SW_COURTYARD: usize = 17;
pub const NODE_SE_COURTYARD: usize = 18;
pub const NODE_BACK_GATE: usize = 19;
pub const NODE_COUNT: usize = 20;

#[derive(Clone, Debug)]
pub struct WaypointGraph {
    pub positions: Vec<Vec3>,
    pub edges: Vec<Vec<(usize, f32)>>,
    pub blocked_edges: Vec<(usize, usize)>,
}

impl Default for WaypointGraph {
    fn default() -> Self {
        let mut graph = Self {
            positions: Vec::with_capacity(NODE_COUNT),
            edges: vec![Vec::new(); NODE_COUNT],
            blocked_edges: Vec::new(),
        };
        graph.build();
        graph
    }
}

impl WaypointGraph {
    fn build(&mut self) {
        self.positions = vec![
            castle::WELL_POS,
            castle::ARMORY_POS,
            castle::HEALING_POS,
            castle::REPAIR_PILE_POS,
            nalgebra_glm::vec3(0.0, 0.0, 13.0),
            castle::RIVER_POS,
            nalgebra_glm::vec3(-12.0, 0.0, -12.0),
            nalgebra_glm::vec3(12.0, 0.0, -12.0),
            nalgebra_glm::vec3(-12.0, 0.0, 12.0),
            nalgebra_glm::vec3(12.0, 0.0, 12.0),
            nalgebra_glm::vec3(0.0, 0.0, 0.0),
            nalgebra_glm::vec3(0.0, 0.0, -8.0),
            nalgebra_glm::vec3(0.0, 0.0, 8.0),
            nalgebra_glm::vec3(8.0, 0.0, 0.0),
            nalgebra_glm::vec3(-8.0, 0.0, 0.0),
            nalgebra_glm::vec3(-7.0, 0.0, -7.0),
            nalgebra_glm::vec3(7.0, 0.0, -7.0),
            nalgebra_glm::vec3(-7.0, 0.0, 7.0),
            nalgebra_glm::vec3(7.0, 0.0, 7.0),
            nalgebra_glm::vec3(0.0, 0.0, -15.5),
        ];

        let connections: &[(usize, usize)] = &[
            (NODE_CENTER, NODE_NORTH_CENTER),
            (NODE_CENTER, NODE_SOUTH_CENTER),
            (NODE_CENTER, NODE_EAST_CENTER),
            (NODE_CENTER, NODE_WEST_CENTER),
            (NODE_CENTER, NODE_WELL),
            (NODE_NORTH_CENTER, NODE_NW_COURTYARD),
            (NODE_NORTH_CENTER, NODE_NE_COURTYARD),
            (NODE_NORTH_CENTER, NODE_WELL),
            (NODE_SOUTH_CENTER, NODE_SW_COURTYARD),
            (NODE_SOUTH_CENTER, NODE_SE_COURTYARD),
            (NODE_SOUTH_CENTER, NODE_REPAIR_PILE),
            (NODE_SOUTH_CENTER, NODE_GATE),
            (NODE_EAST_CENTER, NODE_NE_COURTYARD),
            (NODE_EAST_CENTER, NODE_SE_COURTYARD),
            (NODE_EAST_CENTER, NODE_HEALING),
            (NODE_WEST_CENTER, NODE_NW_COURTYARD),
            (NODE_WEST_CENTER, NODE_SW_COURTYARD),
            (NODE_NW_COURTYARD, NODE_ARMORY),
            (NODE_NW_COURTYARD, NODE_ARCHER_NW),
            (NODE_NE_COURTYARD, NODE_ARCHER_NE),
            (NODE_NE_COURTYARD, NODE_HEALING),
            (NODE_SW_COURTYARD, NODE_ARCHER_SW),
            (NODE_SE_COURTYARD, NODE_ARCHER_SE),
            (NODE_SE_COURTYARD, NODE_GATE),
            (NODE_SW_COURTYARD, NODE_GATE),
            (NODE_BACK_GATE, NODE_NORTH_CENTER),
            (NODE_BACK_GATE, NODE_RIVER),
        ];

        for &(from, to) in connections {
            let distance = nalgebra_glm::distance(&self.positions[from], &self.positions[to]);
            self.edges[from].push((to, distance));
            self.edges[to].push((from, distance));
        }
    }

    pub fn is_edge_blocked(&self, from: usize, to: usize) -> bool {
        self.blocked_edges.iter().any(|&(edge_from, edge_to)| {
            (edge_from == from && edge_to == to) || (edge_from == to && edge_to == from)
        })
    }

    pub fn block_edge(&mut self, from: usize, to: usize) {
        if !self.is_edge_blocked(from, to) {
            self.blocked_edges.push((from, to));
        }
    }

    pub fn unblock_edge(&mut self, from: usize, to: usize) {
        self.blocked_edges.retain(|&(edge_from, edge_to)| {
            !((edge_from == from && edge_to == to) || (edge_from == to && edge_to == from))
        });
    }

    pub fn nearest_node(&self, position: &Vec3) -> usize {
        let mut best = 0;
        let mut best_dist = f32::MAX;
        for (index, node_pos) in self.positions.iter().enumerate() {
            let dist = nalgebra_glm::distance(position, node_pos);
            if dist < best_dist {
                best_dist = dist;
                best = index;
            }
        }
        best
    }

    pub fn find_path(&self, from: usize, to: usize) -> Option<Vec<usize>> {
        if from == to {
            return Some(vec![to]);
        }

        let mut open = BinaryHeap::new();
        let mut came_from = [usize::MAX; NODE_COUNT];
        let mut g_score = [f32::MAX; NODE_COUNT];

        g_score[from] = 0.0;
        let h = nalgebra_glm::distance(&self.positions[from], &self.positions[to]);
        open.push(PathNode {
            node: from,
            f_score: h,
        });

        while let Some(current) = open.pop() {
            if current.node == to {
                let mut path = Vec::new();
                let mut node = to;
                while node != from {
                    path.push(node);
                    node = came_from[node];
                }
                path.push(from);
                path.reverse();
                return Some(path);
            }

            for &(neighbor, edge_cost) in &self.edges[current.node] {
                if self.is_edge_blocked(current.node, neighbor) {
                    continue;
                }

                let tentative_g = g_score[current.node] + edge_cost;
                if tentative_g < g_score[neighbor] {
                    came_from[neighbor] = current.node;
                    g_score[neighbor] = tentative_g;
                    let h = nalgebra_glm::distance(&self.positions[neighbor], &self.positions[to]);
                    open.push(PathNode {
                        node: neighbor,
                        f_score: tentative_g + h,
                    });
                }
            }
        }

        None
    }
}

#[derive(Clone, Debug)]
struct PathNode {
    node: usize,
    f_score: f32,
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_score.to_bits() == other.f_score.to_bits()
    }
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_score
            .partial_cmp(&self.f_score)
            .unwrap_or(Ordering::Equal)
    }
}
