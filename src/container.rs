use std::ops::IndexMut;

use petgraph::{Direction, Graph};
use petgraph::graph::NodeIndex;

use crate::connector::ConnectorDirection;
use crate::function_box::FunctionBox;
use crate::function_box_draw::output_input_pair;

pub type FunctionBoxRef = NodeIndex<u32>;
pub type ConnectorRef = usize;
pub type FBGraph = Graph<FunctionBox, Vec<(ConnectorRef, ConnectorRef)>>;

#[derive(Debug)]
pub struct Container {
    pub graph: FBGraph
}


impl Container {
    pub(crate) fn new() -> Container {
        Container {
            graph: Graph::new()
        }
    }

    pub(crate) fn add(&mut self, function_box: FunctionBox) -> FunctionBoxRef {
        self.graph.add_node(function_box)
    }

    pub fn can_connect(&self, c1: (FunctionBoxRef, ConnectorRef), c2: (FunctionBoxRef, ConnectorRef)) -> bool {
        if let Some(((output_ref, output_connector), (input_ref, input_connector))) = output_input_pair(&self.graph, c1, c2) {
            assert!(matches!(self.graph[output_ref].connectors[output_connector].direction, ConnectorDirection::Output), "wrong direction {}", output_connector);
            assert!(matches!(self.graph[input_ref].connectors[input_connector].direction, ConnectorDirection::Input), "wrong direction {}", input_connector);
            self.graph.edges_directed(input_ref, Direction::Incoming).into_iter()
                .find(|x| {
                    x.weight().iter().find(|(outp, inp)| { *inp == input_connector }).is_some()
                }).is_none()
        } else {
            false
        }
    }

    pub fn connect(&mut self, output: (FunctionBoxRef, ConnectorRef), input: (FunctionBoxRef, ConnectorRef)) {
        let ((output_ref, output_connector), (input_ref, input_connector)) = (output, input);

        assert!(matches!(self.graph[output_ref].connectors[output_connector].direction, ConnectorDirection::Output), "wrong direction {}", output_connector);
        assert!(matches!(self.graph[input_ref].connectors[input_connector].direction, ConnectorDirection::Input), "wrong direction {}", input_connector);
        assert!(self.can_connect(output, input), "not connectable {} -> {}", output_connector, input_connector);

        let edge_ref = self.graph.find_edge(output_ref, input_ref)
            .unwrap_or_else(|| { self.graph.add_edge(output_ref, input_ref, Vec::new()) });
        let vec = self.graph.index_mut(edge_ref);

        let new_edge = (output_connector, input_connector);
        if !vec.contains(&new_edge) {
            vec.push(new_edge)
        }
    }

    pub(crate) fn disconnect(&mut self, connector: (FunctionBoxRef, ConnectorRef)) {
        if let ConnectorDirection::Input = self.graph[connector.0].connectors[connector.1].direction {
            self.graph[connector.0].connectors[connector.1].state = false;
            let mut neighbors = self.graph.neighbors_directed(connector.0, Direction::Incoming)
                .detach();
            while let Some(n) = neighbors.next_edge(&self.graph) {
                self.graph.index_mut(n).retain(|&(out, inp)|{
                    inp != connector.1
                });
                if self.graph[n].is_empty() {
                    self.graph.remove_edge(n);
                }
            }
        }
    }
}
