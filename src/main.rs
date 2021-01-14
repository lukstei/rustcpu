use petgraph::{Graph, Direction};
use petgraph::graph::NodeIndex;
use std::ops::IndexMut;
use piston::Position;
use crate::ui::PosF;
use std::fmt::{Display, Formatter};
use std::fmt;
use crate::ConnectorDirection::{Output, Input};

mod ui;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ConnectorDirection {
    Input,
    Output,
}

#[derive(Debug, Clone)]
struct Connector {
    name: String,
    direction: ConnectorDirection,
}

impl Connector {
    fn new_output(name: String) -> Connector {
        Self::new(name, Output)
    }
    fn new_input(name: String) -> Connector {
        Self::new(name, Input)
    }
    fn new(name: String, direction: ConnectorDirection) -> Connector {
        Connector {
            name,
            direction,
        }
    }
}

impl Display for Connector {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.name)
    }
}

impl PartialEq for Connector {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.direction == other.direction
    }
}

#[derive(Debug)]
struct FunctionBox {
    name: String,
    inputs: Vec<Connector>,
    outputs: Vec<Connector>,

    position: PosF,
}

impl FunctionBox {
    fn new(name: &str, position: PosF, inputs: Vec<String>, outputs: Vec<String>) -> FunctionBox {
        FunctionBox {
            name: name.into(),
            inputs: inputs.into_iter().map(|n| Connector::new(n, ConnectorDirection::Input)).collect(),
            outputs: outputs.into_iter().map(|n| Connector::new(n, ConnectorDirection::Output)).collect(),
            position,
        }
    }

    fn connectors(&self) -> impl Iterator<Item=&Connector> {
        self.inputs.iter().chain(self.outputs.iter())
    }
}

type FunctionBoxRef = NodeIndex;

#[derive(Debug)]
struct Container {
    graph: Graph<FunctionBox, Vec<(Connector, Connector)>>
}

impl Container {
    fn new() -> Container {
        Container {
            graph: Graph::new()
        }
    }

    fn add(&mut self, function_box: FunctionBox) -> FunctionBoxRef {
        self.graph.add_node(function_box)
    }

    fn connect(&mut self, output_ref: FunctionBoxRef, output_connector: Connector, input_ref: FunctionBoxRef, input_connector: Connector) {
        assert!(self.graph[output_ref].outputs.contains(&output_connector), "unknown output {}", output_connector);
        assert!(self.graph[input_ref].inputs.contains(&input_connector), "unknown input {}", input_connector);
        assert!(matches!(output_connector.direction, ConnectorDirection::Output), "wrong direction {}", output_connector);
        assert!(matches!(input_connector.direction, ConnectorDirection::Input), "wrong direction {}", input_connector);
        assert!(self.graph.first_edge(input_ref, Direction::Incoming).is_none(), "input {} already connected", input_connector);

        let edge_ref = self.graph.find_edge(output_ref, input_ref).unwrap_or(self.graph.add_edge(output_ref, input_ref, Vec::new()));
        let vec = self.graph.index_mut(edge_ref);

        let new_edge = (output_connector, input_connector);
        if !vec.contains(&new_edge) {
            vec.push(new_edge)
        }
    }
}

fn main() {
    ui::ui_main();
}
