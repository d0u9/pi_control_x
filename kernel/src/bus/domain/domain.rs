use std::fmt::Debug;
use std::any::Any;

use petgraph::graph::Graph;

use super::handler::*;
use super::traits::*;
use super::errors::DomainError;
use super::super::switch::*;
use super::super::wire::{Wire, Endpoint};
use super::super::address::Address;

enum Device {
    Switch(Box<dyn Any>),
    Test1,
    Test2
}

pub struct Domain {
    devices: Graph<Device, (), petgraph::Undirected>,
}

impl Domain {
    pub fn new() -> Self {
        Self {
            devices: Graph::new_undirected(),
        }
    }

    pub fn add_switch<T>(&mut self, name: &str) -> SwitchHandler
    where
        T: 'static + Clone + Debug
    {
        let switch = Switch::<T>::builder()
                .set_name(name)
                .done();

        let switch_id = switch.get_id();

        let device = Device::Switch(Box::new(switch));
        let node_id = self.devices.add_node(device);

        SwitchHandler::new(switch_id, node_id)
    }

    pub fn add_endpoint<T>(&mut self, switch: &SwitchHandler, addr: Address) -> Result<Endpoint<T>, DomainError>
    where
        T: 'static + Debug + Clone
    {
        let (ep0, ep1) = Wire::endpoints::<T>();
        let device = self.devices.node_weight_mut(switch.graph_id).ok_or(DomainError::InvalidHandler)?;

        match device {
            Device::Switch(switch) => {
                match switch.downcast_mut::<Switch<T>>() {
                    Some(switch) => {
                        switch.attach(addr, ep1)?;
                    }
                    _ => return Err(DomainError::InvalidHandler),
                }
            }
            _ => { }
        }


        Ok(ep0)
    }

    pub fn serve(self) {
        let Self {
            devices,
            ..
        } = self;
        let (device_nodes, _) = devices.into_nodes_edges();
        let _ = device_nodes.into_iter()
            .map(|node| node.weight)
            .map(|device| {
                match device {
                    Device::Switch(switch) => {  }
                    _ => { }
                }
            })
            .collect::<Vec<_>>();
    }
}
