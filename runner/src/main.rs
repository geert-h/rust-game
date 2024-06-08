mod controller;
mod model;
mod view;

#[macro_use]
extern crate glium;

use std::fmt::Display;
use glium::{Frame, Surface, VertexBuffer};
use winit::event_loop::EventLoop;
use winit::event::{KeyEvent, MouseButton};
use winit::keyboard::{Key, NamedKey, SmolStr};
use winit::event::Event::WindowEvent;
use gust_core::data::mesh::*;
use gust_core::parsers::wavefront_object_parser;
use gust_core::data::vertex::Vertex;
fn main() {

    let mut position = [0.0, 0.0, 5.0];
    let mut direction = [0.0, 0.0, -1.0];

    let frac_shader_string = include_str!("../../resources/shaders/frac.glsl");
    let vert_shader_string = include_str!("../../resources/shaders/vert.glsl");

    let wavefront_object = wavefront_object_parser::parse_wavefront_object("C:\\Users\\Geert\\source\\repos\\Personal\\gust\\resources\\assets\\objects\\monkey.obj");
    let object = from_wavefront_object(wavefront_object);

    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build()
        .expect("Event loop building");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Gust")
        .build(&event_loop);

    let mut t: f32 = 0.0;

    let flattened_triangles: Vec<Vertex> = object.triangles
        .iter()
        .flat_map(|triangle| triangle
            .iter()
            .cloned()
        )
        .collect();

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let vertex_buffer = VertexBuffer::new(&display, &flattened_triangles).unwrap();

    event_loop
        .run(move |event, window_target| {
            match event {
                WindowEvent { event: window_event, .. } => match window_event {
                    winit::event::WindowEvent::KeyboardInput { event : KeyEvent { logical_key : key, ..}, .. } => new_handle_inputs(&mut direction, &mut position, key),
                    winit::event::WindowEvent::CloseRequested => window_target.exit(),
                    winit::event::WindowEvent::Resized(window_size) => {
                        display.resize(window_size.into());
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        t += 0.02;

                        let program = glium::Program::from_source(
                            &display,
                            vert_shader_string,
                            frac_shader_string,
                            None,
                        )
                            .unwrap();

                        let params = glium::DrawParameters {
                            depth: glium::Depth {
                                test: glium::draw_parameters::DepthTest::IfLess,
                                write: true,
                                ..Default::default()
                            },
                            backface_culling:
                            glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                            ..Default::default()
                        };

                        let mut target = display.draw();
                        target.clear_color_and_depth((0.3, 0.3, 0.4, 1.0), 1.0);
                        target
                            .draw(
                                &vertex_buffer,
                                &indices,
                                &program,
                                &get_uniforms(position, direction, t, &target),
                                &params,
                            )
                            .unwrap();

                        target.finish().unwrap();
                    },
                    winit::event::WindowEvent::CursorMoved { position, .. } => {
                        println!("Cursor moved to {:?}", position);
                    },
                    _ => (),
                },
                winit::event::Event::AboutToWait => {
                    window.request_redraw();
                },
                _ => (),
            };
        })
        .unwrap();
}

fn get_uniforms(position: [f32; 3], direction : [f32; 3], t: f32, target: &Frame) -> impl glium::uniforms::Uniforms {
    let light = [1.4, 0.4, -0.7f32];

    let view = view_matrix(&position, &direction, &[0.0, 1.0, 0.0]);

    let perspective = {
        let (width, height) = target.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;

        let fov: f32 = std::f32::consts::PI / 3.0;
        let z_far = 1024.0;
        let z_near = 0.1;

        let f = 1.0 / (fov / 2.0).tan();

        [
            [f * aspect_ratio, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (z_far + z_near) / (z_far - z_near), 1.0],
            [0.0, 0.0, -(2.0 * z_far * z_near) / (z_far - z_near), 0.0],
        ]
    };

    uniform! {
        perspective: perspective,
        model: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32]
        ],
        u_light: light,
        view : view,
    }
}

fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [
        up[1] * f[2] - up[2] * f[1],
        up[2] * f[0] - up[0] * f[2],
        up[0] * f[1] - up[1] * f[0],
    ];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [
        f[1] * s_norm[2] - f[2] * s_norm[1],
        f[2] * s_norm[0] - f[0] * s_norm[2],
        f[0] * s_norm[1] - f[1] * s_norm[0],
    ];

    let p = [
        -position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
        -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
        -position[0] * f[0] - position[1] * f[1] - position[2] * f[2],
    ];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}

// fn handle_mouse_input(
//     mouse_btn: MouseButton,
//     x: i32,
//     y: i32,
//     game: &mut model::game::Game,
// ) {
//     if mouse_btn == MouseButton::Left {
//         game.player.x = x - (game.player.width / 2) as i32;
//         game.player.y = y - (game.player.height / 2) as i32;
//     }
// }
//
fn new_handle_inputs(
    direction : &mut [f32; 3],
    position : &mut [f32; 3],
    key: Key,
) {
    match key {
        Key::Character(key_value) if key_value == smol_str::SmolStr::from("w") => {
            position[0] += direction[0] * 0.1;
            position[1] += direction[1] * 0.1;
            position[2] += direction[2] * 0.1;
        }
        Key::Character(key_value) if key_value == smol_str::SmolStr::from("s") => {
            position[0] -= direction[0] * 0.1;
            position[1] -= direction[1] * 0.1;
            position[2] -= direction[2] * 0.1;
        }
        Key::Character(key_value) if key_value == smol_str::SmolStr::from("a") => {
            position[0] -= direction[2] * 0.1;
            position[1] -= direction[0] * 0.1;
            position[2] -= direction[1] * 0.1;
        }
        Key::Character(key_value) if key_value == smol_str::SmolStr::from("d") => {
            position[0] += direction[2] * 0.1;
            position[1] += direction[0] * 0.1;
            position[2] += direction[1] * 0.1;
        }
        _ => (),
    }
}