extern crate piston;
extern crate opengl_graphics;
extern crate graphics;

extern crate glutin_window;

use opengl_graphics::{GlGraphics, OpenGL};
use graphics::{Context, Graphics};
use std::collections::HashMap;
use piston::window::{AdvancedWindow, Window, WindowSettings};
use piston::input::*;
use piston::event_loop::*;

use glutin_window::GlutinWindow as AppWindow;
use std::f64::consts::PI;
use crate::{FunctionBox, Container, Connector, ConnectorDirection};
use std::cmp::max;
use self::graphics::{Rectangle, color, CircleArc, line_from_to};
use self::graphics::rectangle::{square, Border};
use self::graphics::types::{Color, Radius};
use self::piston::{Position, Size};
use self::graphics::math::Vec2d;
use std::cell::RefCell;
use std::ptr;
use std::ptr::eq;
use petgraph::graph::{Node, NodeIndex};
use std::ops::IndexMut;
use vecmath::{vec2_add, vec2_sub};
use crate::ConnectorDirection::Output;
use std::mem::replace;


pub type PosF = Vec2d;
type OurGraphics = GlGraphics;

struct DrawCtx<'a> {
    g: &'a mut GlGraphics,
    c: &'a Context,
    window: &'a dyn Window,
}

#[derive(Debug)]
struct State {
    mouse_position: PosF,
    mouse_button1_pressed: bool,
    mouse_delta: PosF,
    window_size: Size,

    container: Container,
    dragged_function_box: Option<(NodeIndex, PosF)>,
    dragged_connector: Option<(NodeIndex, Connector, PosF)>,
}

pub fn ui_main() {
    let opengl = OpenGL::V3_2;
    let mut window: AppWindow = WindowSettings::new("piston-example-user_input", [1024, 768])
        .exit_on_esc(true).graphics_api(opengl).build().unwrap();

    let ref mut gl = GlGraphics::new(opengl);

    let mut state = State {
        container: Container::new(),
        mouse_button1_pressed: false,
        mouse_position: [0., 0.],
        mouse_delta: [0., 0.],
        window_size: Size { width: 0., height: 0. },
        dragged_function_box: None,
        dragged_connector: None,
    };
    let and_box = state.container.add(FunctionBox::new("and", [50., 50.], vec!["i1".into(), "i2".into()], vec!["and".into()]));
    let one_box = state.container.add(FunctionBox::new("1", [50., 200.], vec![], vec!["1".into()]));
    state.container.connect(one_box, Connector::new_output("1".into()), and_box, Connector::new_input("i1".into()));

    let mut mouse_position = state.mouse_position;
    let mut mouse_delta = state.mouse_delta;
    let mut mouse_button1_pressed = state.mouse_button1_pressed;
    let mut window_size = state.window_size;

    let mut events = Events::new(EventSettings::new().lazy(true));
    while let Some(e) = events.next(&mut window) {
        if let Some(Button::Mouse(button)) = e.press_args() {
            println!("Pressed mouse button '{:?}'", button);
            if let MouseButton::Left = button {
                mouse_button1_pressed = true;
            }
        }
        if let Some(Button::Keyboard(key)) = e.press_args() {
            println!("Pressed keyboard key '{:?}'", key);
        };
        if let Some(args) = e.button_args() {
            println!("Scancode {:?}", args.scancode);
        }
        if let Some(button) = e.release_args() {
            match button {
                Button::Keyboard(key) => println!("Released keyboard key '{:?}'", key),
                Button::Mouse(button) => {
                    println!("Released mouse button '{:?}'", button);
                    if let MouseButton::Left = button {
                        mouse_button1_pressed = false;
                    }
                }
                Button::Controller(button) => println!("Released controller button '{:?}'", button),
                Button::Hat(hat) => println!("Released controller hat `{:?}`", hat),
            }
        };
        e.mouse_cursor(|pos| {
            println!("Mouse moved '{} {}'", pos[0], pos[1]);
            mouse_position = pos;
        });
        e.mouse_relative(|d| {
            println!("Relative mouse moved '{} {}'", d[0], d[1]);
            mouse_delta = d;
        });
        e.resize(|args| {
            println!("Resized '{}, {}'", args.window_size[0], args.window_size[1]);
            window_size = args.draw_size.into();
        });
        if let Some(cursor) = e.cursor_args() {
            if cursor { println!("Mouse entered"); } else { println!("Mouse left"); }
        };
        if let Some(args) = e.render_args() {
            // println!("Render {}", args.ext_dt);

            state.window_size = window_size;
            state.mouse_position = mouse_position;
            state.mouse_button1_pressed = mouse_button1_pressed;
            state.mouse_delta = mouse_delta;

            gl.draw(args.viewport(), |c, g| {
                graphics::clear(rgba(178, 190, 195, 1.0), g);
                let mut ctx = DrawCtx {
                    g,
                    c: &c,
                    window: &window,
                };
                draw(&mut state, &mut ctx);
            },
            );
        }
        if let Some(_args) = e.idle_args() {}
        if let Some(_args) = e.update_args() {}
    }
}

fn rgba(r: i32, g: i32, b: i32, a: f32) -> Color {
    [r as f32 / 255., g as f32 / 255., b as f32 / 255., a]
}


fn draw(
    state: &mut State,
    ctx: &mut DrawCtx,
) {
    if !state.mouse_button1_pressed {
        state.dragged_function_box = None;
        state.dragged_connector = None;
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

        let mut draw = FunctionBoxDraw::new(&state.container.graph[i]);

        if state.mouse_button1_pressed {
            if let Some((fb, hpos)) = state.dragged_function_box {
                if i == fb {
                    draw.highlighted = true;
                }
            } else if let Some((fb, conn, hpos)) = state.dragged_connector.clone() {
                if i == fb {
                    line_from_to(color::BLACK, 1., draw.connector_position(&conn), state.mouse_position, ctx.c.transform, ctx.g);
                    draw.highlighted_connector = Some(conn);
                } else if let Some(FunctionBoxCollideDesc::Connector(connector)) =
                draw.collide(state.mouse_position) {
                    draw.highlighted_connector = Some(connector);
                }
            } else {
                let origin = vec2_sub(state.mouse_position, draw.function_box.position);
                match draw.collide(state.mouse_position) {
                    Some(FunctionBoxCollideDesc::FunctionBox) => {
                        state.dragged_function_box = Some((i, origin));
                    }
                    Some(FunctionBoxCollideDesc::Connector(connector)) => {
                        state.dragged_connector = Some((i, connector, origin));
                    }
                    _ => {}
                }
            }
        } else {
            state.dragged_function_box = None;
        };

        draw.draw(ctx)
    })
}

struct FunctionBoxDraw<'a> {
    function_box: &'a FunctionBox,
    rect: [f64; 4],
    padding: f64,
    connector_radius: f64,
    connector_margin: f64,
    highlighted: bool,
    highlighted_connector: Option<Connector>,
}

impl<'a> FunctionBoxDraw<'a> {
    fn new(function_box: &'a FunctionBox) -> Self {
        let padding = 20.;
        let connector_radius = 6.5;
        let connector_margin = 8.;

        let height = 2. * padding + 40.;
        let width = 2. * padding - connector_margin + ((connector_margin + connector_radius * 2.) * max(function_box.outputs.len(), function_box.inputs.len()) as f64);
        let rect = [function_box.position[0], function_box.position[1], width, height];

        FunctionBoxDraw {
            function_box,
            rect,
            padding,
            connector_radius,
            connector_margin,
            highlighted: false,
            highlighted_connector: None,
        }
    }

    fn connector_position(&self, connector: &Connector) -> PosF {
        let cs = if matches!(connector.direction, ConnectorDirection::Input) { &self.function_box.inputs } else { &self.function_box.outputs };
        let i = cs.iter().position(|x| x == connector).unwrap();
        self.connector_position_idx(i, connector.direction)
    }

    fn connector_position_idx(&self, connector_idx: usize, connector_direction: ConnectorDirection) -> PosF {
        let i = connector_idx as f64;

        [
            self.rect[0] + self.padding + (i * self.connector_margin + i * self.connector_radius * 2. + self.connector_radius),
            self.rect[1] + (self.rect[3] * (if matches!(connector_direction, ConnectorDirection::Input) { 1. } else { 0. }))
        ]
    }
}

trait Collide {
    type CollideDesc;
    fn collide(&self, point: PosF) -> Option<Self::CollideDesc>;
}

trait Draw {
    fn draw(&self, ctx: &mut DrawCtx);
}

enum FunctionBoxCollideDesc {
    FunctionBox,
    Connector(Connector),
}

impl Collide for FunctionBoxDraw<'_> {
    type CollideDesc = FunctionBoxCollideDesc;

    fn collide(&self, point: [f64; 2]) -> Option<FunctionBoxCollideDesc> {
        let [x1, y1] = point;

        if let Some((_i, conn)) =
        self.function_box.outputs.iter().enumerate()
            .chain(self.function_box.inputs.iter().enumerate())
            .find(|(i, x)| {
                let [x2, y2] = self.connector_position_idx(*i, x.direction);

                f64::sqrt((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)) <= self.connector_radius
            }) {
            Some(FunctionBoxCollideDesc::Connector(conn.clone()))
        } else if x1 >= self.rect[0]
            && x1 <= self.rect[0] + self.rect[2]
            && y1 >= self.rect[1]
            && y1 <= self.rect[1] + self.rect[3] {
            Some(FunctionBoxCollideDesc::FunctionBox)
        } else {
            None
        }
    }
}

impl Draw for FunctionBoxDraw<'_> {
    fn draw(&self, ctx: &mut DrawCtx) {
        let bg_color = if self.highlighted { rgba(253, 203, 110, 1.0) } else { rgba(9, 132, 227, 1.0) };
        let mut rectangle = Rectangle::new_round_border(bg_color, 3., 0.);
        rectangle = rectangle.color(bg_color);
        rectangle.draw_tri(self.rect, &Default::default(), ctx.c.transform, ctx.g);

        let fb = self.function_box;
        // draw connectors
        fb.outputs.iter().enumerate()
            .chain(fb.inputs.iter().enumerate())
            .for_each(|(i, c)| {
                let highlighted = self.highlighted_connector.as_ref().map_or(false, |x| { *x == *c });
                draw_arc_centered(self.connector_position_idx(i, c.direction),
                                  self.connector_radius, rgba(99, 110, 114, 1.0), highlighted, ctx);
            });
    }
}

fn draw_arc_centered(center: PosF, circle_radius: Radius, color: Color, highlighted: bool, ctx: &mut DrawCtx) {
    CircleArc::new(color, circle_radius / if highlighted { 1. } else { 2. }, 0., 2. * PI)
        .draw_tri([center[0] - circle_radius, center[1] - circle_radius, circle_radius * 2., circle_radius * 2.], &Default::default(), ctx.c.transform, ctx.g);
}