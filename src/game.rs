use opengl_graphics::{GlGraphics, GlyphCache};
use piston::{Size, Window};
use graphics::{Context, line_from_to, color};
use std::ops::{IndexMut, Index};
use vecmath::vec2_sub;
use graphics::math::Vec2d;
use petgraph::{Direction, Graph};
use std::fmt::{Display, Formatter};
use core::fmt;
use crate::game::ConnectorDirection::{Output, Input};
use crate::function_box_draw::{FunctionBoxCollideDesc, FunctionBoxDraw, output_input_pair};
use petgraph::prelude::EdgeRef;
use std::borrow::{Borrow, BorrowMut};
use petgraph::graph::{DefaultIx, NodeIndex};

pub type PosF = Vec2d;
type OurGraphics = GlGraphics;
pub type FunctionBoxRef = NodeIndex<u32>;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectorDirection {
    Input,
    Output,
}

#[derive(Debug, Clone)]
pub struct Connector {
    pub name: String,
    pub direction: ConnectorDirection,
    pub idx: usize,
    pub state: bool,
}

impl Connector {
    fn new(name: String, direction: ConnectorDirection, idx: usize) -> Connector {
        Connector {
            name,
            direction,
            idx,
            state: false,
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
        self.idx == other.idx && self.direction == other.direction
    }
}

#[derive(Debug)]
pub struct FunctionBox {
    pub name: String,
    pub inputs: Vec<Connector>,
    pub outputs: Vec<Connector>,

    pub position: PosF,
    pub generation: usize, // increased in every tick to avoid infinite recursion in circles
}

impl FunctionBox {
    pub fn get_input_connector(&self, name: &str) -> &Connector {
        self.inputs.iter().find(|x| { x.name == name }).unwrap()
    }
    pub fn get_output_connector(&self, name: &str) -> &Connector {
        self.outputs.iter().find(|x| { x.name == name }).unwrap()
    }

    pub(crate) fn new(name: &str, position: PosF, inputs: Vec<String>, outputs: Vec<String>) -> FunctionBox {
        FunctionBox {
            name: name.into(),
            inputs: inputs.into_iter().enumerate().map(|(i, n)| Connector::new(n, ConnectorDirection::Input, i)).collect(),
            outputs: outputs.into_iter().enumerate().map(|(i, n)| Connector::new(n, ConnectorDirection::Output, i)).collect(),
            position,
            generation: 0,
        }
    }

    fn connectors(&self) -> impl Iterator<Item=&Connector> {
        self.inputs.iter().chain(self.outputs.iter())
    }
}

pub type ConnectorRef =  usize;

#[derive(Debug)]
pub struct Container {
    pub graph: Graph<FunctionBox, Vec<(ConnectorRef, ConnectorRef)>>
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
        if let Some(((output_ref, output_connector), (input_ref, input_connector))) = output_input_pair(c1, c2) {
            assert!(self.graph[output_ref].outputs.contains(&output_connector), "unknown output {}", output_connector);
            assert!(self.graph[input_ref].inputs.contains(&input_connector), "unknown input {}", input_connector);
            assert!(matches!(output_connector.direction, ConnectorDirection::Output), "wrong direction {}", output_connector);
            assert!(matches!(input_connector.direction, ConnectorDirection::Input), "wrong direction {}", input_connector);
            self.graph.edges_directed(input_ref, Direction::Incoming).into_iter()
                .find(|x| {
                    x.weight().iter().find(|(outp, inp)| { inp == input_connector }).is_some()
                }).is_none()
        } else {
            false
        }
    }

    pub fn connect(&mut self, output: (FunctionBoxRef, &Connector), input: (FunctionBoxRef, &Connector)) {
        let ((output_ref, output_connector), (input_ref, input_connector)) = (output, input);

        assert!(self.graph[output_ref].outputs.contains(&output_connector), "unknown output {}", output_connector);
        assert!(self.graph[input_ref].inputs.contains(&input_connector), "unknown input {}", input_connector);
        assert!(matches!(output_connector.direction, ConnectorDirection::Output), "wrong direction {}", output_connector);
        assert!(matches!(input_connector.direction, ConnectorDirection::Input), "wrong direction {}", input_connector);
        assert!(self.can_connect(output, input), "not connectable {} -> {}", output_connector, input_connector);

        let edge_ref = self.graph.find_edge(output_ref, input_ref).unwrap_or(self.graph.add_edge(output_ref, input_ref, Vec::new()));
        let vec = self.graph.index_mut(edge_ref);

        let new_edge = (output_connector.idx, input_connector.idx);
        if !vec.contains(&new_edge) {
            vec.push(new_edge)
        }
    }
}


#[derive(Debug)]
pub enum EntityKind {
    FunctionBox,
    Connector,
}

#[derive(Debug)]
pub struct State {
    pub mouse_position: PosF,
    pub mouse_button1_pressed: bool,
    pub mouse_delta: PosF,
    pub window_size: Size,

    pub container: Container,
    pub dragged_entity_kind: Option<EntityKind>,
    pub dragged_function_box: Option<(FunctionBoxRef, PosF)>,
    pub dragged_connector: Option<(FunctionBoxRef, Connector, PosF)>,
    pub dragged_connector_target: Option<(FunctionBoxRef, Connector, PosF)>,
}

pub trait Update {
    fn update(&mut self, state: &State);
}

pub trait Collide {
    type CollideDesc;
    fn collide(&self, point: PosF) -> Option<Self::CollideDesc>;
}

pub trait Draw {
    fn draw(&self, ctx: &mut DrawCtx);
}

pub struct DrawCtx<'a, 'b> {
    pub g: &'a mut GlGraphics,
    pub c: &'a Context,
    pub window: &'a dyn Window,
    pub font_normal: &'a mut GlyphCache<'b>,
}

pub(crate) fn update_fb_states(
    state: &mut State,
) {
    let generation = 1 + state.container.graph.raw_nodes().first().map_or(0, |x| x.weight.generation);

    state.container.graph.node_indices().for_each(|x| {
        if state.container.graph[x].generation < generation {
            calculate_and_set_state(&mut state.container, x);
            state.container.graph.index_mut(x).generation = generation;
        }
    });

    state.container.graph.node_indices().for_each(|i| {
        assert_eq!(state.container.graph[i].generation, generation)
    });
}


pub(crate) fn draw(
    state: &mut State,
    ctx: &mut DrawCtx,
) {
    if !state.mouse_button1_pressed {
        if let (Some((fb1, c1, _)), Some((fb2, c2, _))) = (&state.dragged_connector, &state.dragged_connector_target) {
            let (output, input) = output_input_pair((*fb1, c1), (*fb2, c2)).unwrap();
            println!("Connect {:?} to {:?}", output, input);
            state.container.connect(output, input);
        }

        state.dragged_function_box = None;
        state.dragged_connector = None;
        state.dragged_connector_target = None;
        state.dragged_entity_kind = None;
    }

    state.container.graph.node_indices().for_each(|i| {
        if state.mouse_button1_pressed {
            if let Some((fb, hpos)) = state.dragged_function_box {
                if i == fb {
                    let pos = &mut state.container.graph.index_mut(i).position;
                    *pos = vec2_sub(state.mouse_position, hpos);
                    println!("Position {:?}", pos);
                }
            }
        }
    });

    state.dragged_connector_target = None;
    state.container.graph.node_indices().for_each(|i| {
        let draw = FunctionBoxDraw::new(&state.container.graph[i], i);

        if state.mouse_button1_pressed {
            let origin = vec2_sub(state.mouse_position, state.container.graph[i].position);

            match (&state.dragged_entity_kind, draw.collide(state.mouse_position)) {
                (None, Some(FunctionBoxCollideDesc::FunctionBox)) => {
                    if let None = state.dragged_function_box {
                        state.dragged_entity_kind = Some(EntityKind::FunctionBox);
                        state.dragged_function_box = Some((i, origin));
                    }
                }
                (None, Some(FunctionBoxCollideDesc::Connector(connector))) => {
                    state.dragged_entity_kind = Some(EntityKind::Connector);
                    state.dragged_connector = Some((i, connector, origin));
                }
                (Some(EntityKind::Connector), Some(FunctionBoxCollideDesc::Connector(connector))) => {
                    let option = state.dragged_connector.as_ref().unwrap();
                    if state.container.can_connect((option.0, &option.1), (i, &connector)) {
                        state.dragged_connector_target = Some((i, connector, origin));
                    }
                }
                _ => {}
            }
        }

        let mut draw = FunctionBoxDraw::new(&state.container.graph[i], i);

        draw.update(state);
        draw.draw(ctx);

        state.container.graph.edges_directed(i, Direction::Outgoing)
            .for_each(|e| {
                e.weight().iter().for_each(|(c1, c2)| {
                    let d2 = FunctionBoxDraw::new(&state.container.graph[e.target()], e.target());

                    draw.draw_connection_line(state.container.graph[e.source()].outputs[ c1], d2.connector_position(state.container.graph[e.target()].inputs[c2]), ctx)
                })
            });

        if let Some((i2, c, o)) = &state.dragged_connector {
            if i == *i2 {
                draw.draw_connection_line(c, state.mouse_position, ctx);
            }
        }
    });
}

fn calculate_and_set_state(graph: &mut Container, x: NodeIndex) {
    let result_state: bool;
    {
        let x1: &FunctionBox = graph.graph.index(x);
        match x1.name.as_str() {
            "nand" => {
                let mut result = true;
                let mut count = 0;
                graph.graph.edges_directed(x, Direction::Incoming)
                    .for_each(|x| {
                        let inc = &x.weight().first().unwrap().1;
                        result = result && inc.state;
                        count += 1;
                    });
                if count < 2 {
                    result = false
                }
                result_state = !result;
            }
            "1" => {
                result_state = true;
            }
            "0" => {
                result_state = false;
            }
            "output_toggle" => {
                unimplemented!()
                //result_state = x1.state;
            }
            _ => panic!("Unknown function {:?}", x1.name)
        }
    }

    let mut neighbors = graph.graph.neighbors_directed(x, Direction::Outgoing)
        .detach();
    while let Some(e) = neighbors
        .next_edge(&graph.graph) {
        graph.graph.edge_weight_mut(e).unwrap()
            .iter_mut().for_each(|y| {
            y.0.state = result_state;
            y.1.state = result_state;
        })
    }
}

