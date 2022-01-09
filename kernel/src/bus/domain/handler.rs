use petgraph::graph::NodeIndex;

use super::super::types::DevId;

#[derive(Debug)]
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

pub struct RouterHandler {
    pub(super) switch_id: DevId,
    pub(super) graph_id: NodeIndex,
}

impl RouterHandler {
    pub fn new(switch_id: DevId, graph_id: NodeIndex) -> Self {
        Self {
            switch_id,
            graph_id,
        }
    }
}
