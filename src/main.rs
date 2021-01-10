use petgraph::{Graph, Direction};
use petgraph::graph::NodeIndex;
use std::ops::IndexMut;
use piston::Position;
use crate::ui::PosF;

mod ui;

#[derive(Debug)]
struct Connector {
    name: String
}

impl Connector {
    fn new(name: &str) -> Connector {
        Connector {
            name: name.into()
        }
    }
}

impl PartialEq for Connector {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Debug)]
struct FunctionBox {
    name: String,
    inputs: Vec<String>,
    outputs: Vec<String>,

    position: PosF,
}

impl FunctionBox {
    fn new(name: &str, position: PosF, inputs: Vec<String>, outputs: Vec<String>) -> FunctionBox {
        FunctionBox {
            name: name.into(), inputs, outputs, position
        }
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

    fn connect(&mut self, output_ref: FunctionBoxRef, output_connector: String, input_ref: FunctionBoxRef, input_connector: String,) {
        assert!(self.graph[output_ref].outputs.contains(&output_connector), "unknown output {}", output_connector);
        assert!(self.graph[input_ref].inputs.contains(&input_connector), "unknown input {}", input_connector);
        assert!(self.graph.first_edge(input_ref, Direction::Incoming).is_none(), "input {} already connected", input_connector);

        let edge_ref = self.graph.find_edge(output_ref, input_ref).unwrap_or(self.graph.add_edge(output_ref, input_ref, Vec::new()));
        let vec = self.graph.index_mut(edge_ref);

        let new_edge = (Connector::new(&output_connector), Connector::new(&input_connector));
        if !vec.contains(&new_edge) {
            vec.push(new_edge)
        }
    }
}

fn main() {
    ui::ui_main();
}
