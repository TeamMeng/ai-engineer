use crate::agent::lats::types::{SearchTree, TreeNode};

const UCB_EXPLORATION_CONSTANT: f32 = 1.414_f32;

pub fn ucb_score(node: &TreeNode, parent_visits: usize) -> f32 {
    if node.visits == 0 {
        return f32::INFINITY;
    }

    let exploiation = node.q_value();
    let exploration = UCB_EXPLORATION_CONSTANT
        * (((parent_visits as f32).ln() + 1.0_f32) / (node.visits as f32)).sqrt();

    exploiation + exploration
}

pub fn select_node(tree: &SearchTree) -> usize {
    let mut current_id = 0;

    while !tree.nodes[current_id].children.is_empty() {
        let parent_visits = tree.nodes[current_id].visits;

        let unvisited = tree.nodes[current_id]
            .children
            .iter()
            .copied()
            .find(|&child_id| tree.nodes[child_id].visits == 0);

        if let Some(child_id) = unvisited {
            return child_id;
        }

        let best_child = tree.nodes[current_id]
            .children
            .iter()
            .max_by(|&&a, &&b| {
                let ucb_a = ucb_score(&tree.nodes[a], parent_visits);
                let ucb_b = ucb_score(&tree.nodes[b], parent_visits);
                ucb_a
                    .partial_cmp(&ucb_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied();

        match best_child {
            Some(next_id) => current_id = next_id,
            None => break,
        }
    }

    current_id
}

pub fn backpropagate(tree: &mut SearchTree, leaf_id: usize, value: f32) {
    let mut curr = Some(leaf_id);

    while let Some(node_id) = curr {
        if let Some(node) = tree.nodes.get_mut(node_id) {
            node.visits += 1;
            node.total_value += value;
            curr = node.parent
        } else {
            break;
        }
    }
}

pub fn get_best_path(tree: &SearchTree) -> Vec<usize> {
    let mut path = Vec::new();
    let mut curr = 0;

    while !tree.nodes[curr].children.is_empty() {
        let best_child = tree.nodes[curr]
            .children
            .iter()
            .max_by_key(|&&child_id| tree.nodes[child_id].visits)
            .copied();

        if let Some(child_id) = best_child {
            path.push(child_id);
            curr = child_id
        } else {
            break;
        }
    }

    path
}
