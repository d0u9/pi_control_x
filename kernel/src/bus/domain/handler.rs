use petgraph::graph::NodeIndex;

use super::super::types::DevId;

pub struct SwitchHandler {
    pub(super) switch_id: DevId,
    pub(super) graph_id: NodeIndex,
}

impl SwitchHandler {
    pub fn new(switch_id: DevId, graph_id: NodeIndex) -> Self {
        Self {
            switch_id,
            graph_id,
        }
    }

}

