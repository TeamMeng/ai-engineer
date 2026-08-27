/// 决策树上的单个状态节点
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: usize,                    // 节点在 Arena 数组中的下标索引
    pub parent: Option<usize>,        // 父节点索引
    pub children: Vec<usize>,         // 子节点索引列表
    pub action: Option<String>,       // 尝试调用的工具名
    pub action_input: Option<String>, // 工具入参
    pub observation: Option<String>,  // 工具执行后的观测事实
    pub visits: usize,                // MCTS 访问探索次数 (N)
    pub total_value: f32,             // 累积价值得分 (W)
    pub depth: usize,                 // 树深度
}

#[derive(Debug, Default)]
pub struct SearchTree {
    pub nodes: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(id: usize, parent: Option<usize>, depth: usize) -> Self {
        Self {
            id,
            parent,
            children: Vec::new(),
            action: None,
            action_input: None,
            observation: None,
            visits: 0,
            total_value: 0.0_f32,
            depth,
        }
    }

    pub fn q_value(&self) -> f32 {
        if self.visits == 0 {
            0.0_f32
        } else {
            self.total_value / (self.visits as f32)
        }
    }
}

impl SearchTree {
    pub fn new() -> Self {
        let mut tree = Self { nodes: Vec::new() };
        tree.nodes.push(TreeNode::new(0, None, 0));
        tree
    }

    pub fn add_child(
        &mut self,
        parent_id: usize,
        action: String,
        action_input: String,
        observation: String,
    ) -> usize {
        let new_id = self.nodes.len();
        let depth = self.nodes[parent_id].depth + 1;
        let mut child = TreeNode::new(new_id, Some(parent_id), depth);

        child.action = Some(action);
        child.action_input = Some(action_input);
        child.observation = Some(observation);

        self.nodes.push(child);
        self.nodes[parent_id].children.push(new_id);

        new_id
    }
}
