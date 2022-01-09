use std::pin::Pin;
use std::convert::From;
use std::fmt::Debug;
use std::future::Future;

use petgraph::graph::Graph;

use super::handler::*;
use super::traits::*;
use super::errors::DomainError;
use super::super::router::*;
use super::super::switch::*;
use super::super::wire::{Wire, Endpoint};
use super::super::address::Address;

#[derive(Debug)]
enum Device {
    Switch(Box<dyn SwitchDev>),
    Router(Box<dyn RouterDev>),
    Endpoint(Address),
}

impl Device {
    fn switch_mut(&mut self) -> Option<&mut Box<dyn SwitchDev>> {
        match self {
            Device::Switch(ref mut switch) => Some(switch),
            _ => { None },
        }
    }

    fn router_mut(&mut self) -> Option<&mut Box<dyn RouterDev>> {
        match self {
            Device::Router(ref mut router) => Some(router),
            _ => None,
        }
    }
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
        T: 'static + Clone + Debug + Send
    {
        let switch = Switch::<T>::builder()
                .set_name(name)
                .set_mode(SwitchMode::Broadcast)
                .unwrap()
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
        let device = self.devices.node_weight_mut(switch.graph_id).ok_or(DomainError::InvalidHandler)?;
        let (ep0, ep1) = Wire::endpoints::<T>();

        match device {
            Device::Switch(switch) => {
                match switch.as_any_mut().downcast_mut::<Switch<T>>() {
                    Some(switch) => {
                        switch.attach(addr.clone(), ep1)?;
                    }
                    _ => return Err(DomainError::TypeMismatch),
                }
            }
            _ => {
                    return Err(DomainError::TypeMismatch);
            }
        }

        let device = Device::Endpoint(addr);
        let node_id = self.devices.add_node(device);
        let _edge = self.devices.add_edge(node_id, switch.graph_id, ());

        Ok(ep0)
    }

    pub fn join_switches<U, V>(&mut self, switch0_handler: &SwitchHandler, switch1_handler: &SwitchHandler, name: &str) -> Result<(), DomainError>
    where
        U: 'static + Clone + Debug + From<V> + Send,
        V: 'static + Clone + Debug + From<U> + Send,
    {
        let (ep0s, ep0r) = Wire::endpoints::<U>();
        let (ep1s, ep1r) = Wire::endpoints::<V>();

        let node0_id = {
            let switch0 = self.devices
                .node_weight_mut(switch0_handler.graph_id)
                .ok_or(DomainError::InvalidHandler)?
                .switch_mut()
                .ok_or(DomainError::HandlerIsNotSwitch)?
                .as_any_mut()
                .downcast_mut::<Switch<U>>()
                .ok_or(DomainError::TypeMismatch)?;


            let addr = Address::new(name);
            switch0.attach_router(addr.clone(), ep0s)?;

            let device = Device::Endpoint(addr);
            let node_id = self.devices.add_node(device);
            let _edge = self.devices.add_edge(node_id, switch0_handler.graph_id, ());
            node_id
        };

        let node1_id = {
            let switch1 = self.devices
                .node_weight_mut(switch1_handler.graph_id)
                .ok_or(DomainError::InvalidHandler)?
                .switch_mut()
                .ok_or(DomainError::HandlerIsNotSwitch)?
                .as_any_mut()
                .downcast_mut::<Switch<V>>()
                .ok_or(DomainError::TypeMismatch)?;


            let addr = Address::new(name);
            switch1.attach_router(addr.clone(), ep1s)?;

            let device = Device::Endpoint(addr);
            let node_id = self.devices.add_node(device);
            let _edge = self.devices.add_edge(node_id, switch1_handler.graph_id, ());
            node_id
        };

        let router = Router::<U, V>::builder()
            .set_name(name)
            .set_endpoint0(ep0r)
            .set_endpoint1(ep1r)
            .done()?;

        let router_id = router.get_id();

        let router = Device::Router(Box::new(router));
        let node_id = self.devices.add_node(router);

        let _edge = self.devices.add_edge(node0_id, node_id, ());
        let _edge = self.devices.add_edge(node1_id, node_id, ());

        RouterHandler::new(router_id, node_id);
        Ok(())
    }

    pub fn draw(&self)  {
        use petgraph::dot::{Dot, Config};
        println!("{:?}", Dot::with_config(&self.devices, &[Config::EdgeNoLabel]));
    }

    pub fn done(self) -> DomainServer {
        let Self {
            devices,
            ..
        } = self;

        let (device_nodes, _) = devices.into_nodes_edges();
        let pollers = device_nodes.into_iter()
            .map(|node| node.weight)
            .filter_map(|device| {
                match device {
                    Device::Switch(switch) => {
                        Some(switch.get_poller())
                    }
                    Device::Router(router) => {
                        Some(router.get_poller())
                    }
                    _ => {
                        None
                    }
                }
            })
            .collect::<Vec<_>>();

        DomainServer {
            pollers,
        }
    }
}

pub struct DomainServer {
    pollers: Vec<Pin<Box<dyn Future<Output = ()> + Send>>>,
}

impl DomainServer {
    pub async fn serve(self, shutdown: impl Future<Output = ()>) {
        tokio::select! {
            _ = self.inner_poll() => {},
            _ = shutdown => {},
        }
    }

    pub async fn inner_poll(self) {
        let pollers = self.pollers;

        futures::future::select_all(pollers).await;
    }
}
