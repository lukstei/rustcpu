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
use std::mem::replace;
use crate::game::{PosF, Container, FunctionBox, Connector, DrawCtx};
use crate::game;

pub fn rgba(r: i32, g: i32, b: i32, a: f32) -> Color {
    [r as f32 / 255., g as f32 / 255., b as f32 / 255., a]
}
pub fn ui_main() {
    let opengl = OpenGL::V3_2;
    let mut window: AppWindow = WindowSettings::new("piston-example-user_input", [1024, 768])
        .exit_on_esc(true).graphics_api(opengl).build().unwrap();

    let ref mut gl = GlGraphics::new(opengl);

    let mut state = crate::game::State {
        container: Container::new(),
        mouse_button1_pressed: false,
        mouse_position: [0., 0.],
        mouse_delta: [0., 0.],
        window_size: Size { width: 0., height: 0. },
        dragged_function_box: None,
        dragged_connector: None,
        dragged_connector_target: None,
        dragged_entity_kind: None
    };
    let and_box = state.container.add(FunctionBox::new("and", [50., 50.], vec!["i1".into(), "i2".into()], vec!["and".into()]));
    let one_box = state.container.add(FunctionBox::new("1", [50., 200.], vec![], vec!["1".into()]));
    let graph = &state.container.graph;
    state.container.connect(one_box, graph[one_box].get_output_connector("1").clone(), and_box, graph[and_box].get_input_connector("i1").clone());

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
                game::draw(&mut state, &mut ctx);
            },
            );
        }
        if let Some(_args) = e.idle_args() {}
        if let Some(_args) = e.update_args() {}
    }
}


pub fn draw_arc_centered(center: PosF, circle_radius: Radius, color: Color, highlighted: bool, ctx: &mut DrawCtx) {
    CircleArc::new(color, circle_radius / if highlighted { 1. } else { 2. }, 0., 2. * PI)
        .draw_tri([center[0] - circle_radius, center[1] - circle_radius, circle_radius * 2., circle_radius * 2.], &Default::default(), ctx.c.transform, ctx.g);
}