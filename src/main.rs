#![feature(phase)]
extern crate cgmath;
extern crate gfx;

#[phase(plugin)]
extern crate gfx_macros;
extern crate glfw;
extern crate nice_glfw;
extern crate native;
extern crate time;
extern crate net;

use cgmath::FixedArray;
use cgmath::{Matrix, Point3, Vector3};
use cgmath::{Transform, AffineMatrix3};
use cgmath::Matrix4;
use gfx::{Device, DeviceHelper, ToSlice};
use glfw::Context;
use std::io::net::ip::SocketAddr;

#[vertex_format]
struct Vertex {
    #[as_float]
    #[name = "a_Pos"]
    pos: [i8, ..3],

    #[as_float]
    #[name = "a_TexCoord"]
    tex_coord: [u8, ..2],
}

#[shader_param(CubeBatch)]
struct Params {
    #[name = "u_Transform"]
    transform: [[f32, ..4], ..4],

    #[name = "t_Color"]
    color: gfx::shade::TextureParam,
}

static VERTEX_SRC: gfx::ShaderSource<'static> = shaders! {
GLSL_120: b"
    #version 120

    attribute vec3 a_Pos;
    attribute vec2 a_TexCoord;
    varying vec2 v_TexCoord;

    uniform mat4 u_Transform;

    void main() {
        v_TexCoord = a_TexCoord;
        gl_Position = u_Transform * vec4(a_Pos, 1.0);
    }
"
GLSL_150: b"
    #version 150 core

    in vec3 a_Pos;
    in vec2 a_TexCoord;
    out vec2 v_TexCoord;

    uniform mat4 u_Transform;

    void main() {
        v_TexCoord = a_TexCoord;
        gl_Position = u_Transform * vec4(a_Pos, 1.0);
    }
"
};

static FRAGMENT_SRC: gfx::ShaderSource<'static> = shaders! {
GLSL_120: b"
    #version 120

    varying vec2 v_TexCoord;
    uniform sampler2D t_Color;

    void main() {
        vec4 tex = texture2D(t_Color, v_TexCoord);
        float blend = dot(v_TexCoord-vec2(0.5,0.5), v_TexCoord-vec2(0.5,0.5));
        gl_FragColor = mix(tex, vec4(0.0,0.0,0.0,0.0), blend*1.0);
    }
"
GLSL_150: b"
    #version 150 core

    in vec2 v_TexCoord;
    out vec4 o_Color;

    uniform sampler2D t_Color;
    void main() {
        vec4 tex = texture(t_Color, v_TexCoord);
        float blend = dot(v_TexCoord-vec2(0.5,0.5), v_TexCoord-vec2(0.5,0.5));
        o_Color = mix(tex, vec4(0.0,0.0,0.0,0.0), blend*1.0);
    }
"
};

fn main() {
    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (window, events) = nice_glfw::WindowBuilder::new(&glfw)
        .try_modern_context_hints()
        .size(800, 600)
        .create()
        .expect("Failed to create GLFW window.");

    window.make_current();
    glfw.set_error_callback(glfw::FAIL_ON_ERRORS);
    window.set_size_polling(true);
    window.set_key_polling(true);

    let (w, h) = window.get_framebuffer_size();
    let mut frame  = gfx::Frame::new(w as u16, h as u16);

    let mut device = gfx::GlDevice::new(|s| glfw.get_proc_address(s));

    let vertex_data = vec![
        Vertex { pos: [-1, -1, 1], tex_coord: [0, 0] },
        Vertex { pos: [ 1, -1, 1], tex_coord: [1, 0] },
        Vertex { pos: [ 1,  1, 1], tex_coord: [1, 1] },
        Vertex { pos: [-1,  1, 1], tex_coord: [0, 1] },

        Vertex { pos: [ 1,  1,-1], tex_coord: [0, 0] },
        Vertex { pos: [-1,  1,-1], tex_coord: [1, 0] },
        Vertex { pos: [-1, -1,-1], tex_coord: [1, 1] },
        Vertex { pos: [ 1, -1,-1], tex_coord: [0, 1] },

        Vertex { pos: [ 1, -1,-1], tex_coord: [0, 0] },
        Vertex { pos: [ 1,  1,-1], tex_coord: [1, 0] },
        Vertex { pos: [ 1,  1, 1], tex_coord: [1, 1] },
        Vertex { pos: [ 1, -1, 1], tex_coord: [0, 1] },

        Vertex { pos: [-1,  1, 1], tex_coord: [0, 0] },
        Vertex { pos: [-1, -1, 1], tex_coord: [1, 0] },
        Vertex { pos: [-1, -1,-1], tex_coord: [1, 1] },
        Vertex { pos: [-1,  1,-1], tex_coord: [0, 1] },

        Vertex { pos: [-1,  1,-1], tex_coord: [0, 0] },
        Vertex { pos: [ 1,  1,-1], tex_coord: [1, 0] },
        Vertex { pos: [ 1,  1, 1], tex_coord: [1, 1] },
        Vertex { pos: [-1,  1, 1], tex_coord: [0, 1] },

        Vertex { pos: [ 1, -1, 1], tex_coord: [0, 0] },
        Vertex { pos: [-1, -1, 1], tex_coord: [1, 0] },
        Vertex { pos: [-1, -1,-1], tex_coord: [1, 1] },
        Vertex { pos: [ 1, -1,-1], tex_coord: [0, 1] },
    ];

    let mesh = device.create_mesh(vertex_data.as_slice());

    let index_data: Vec<u8> = vec![
        0, 1, 2, 2, 3, 0,
        4, 5, 6, 6, 7, 4,
        8, 9, 10, 10, 11, 8,
        12, 13, 14, 14, 16, 12,
        16, 17, 18, 18, 19, 16,
        20, 21, 22, 22, 23, 20,
    ];

    let slice = device
        .create_buffer_static::<u8>(index_data.as_slice())
        .to_slice(gfx::TriangleList);

    let texture_info = gfx::tex::TextureInfo {
        width: 1,
        height: 1,
        depth: 1,
        levels: 1,
        kind: gfx::tex::Texture2D,
        format: gfx::tex::RGBA8,
    };

    let image_info = texture_info.to_image_info();
    let texture = device.create_texture(texture_info).unwrap();
    device.update_texture(&texture, &image_info,
                          vec![0x20u8, 0x0Au8, 0xC0u8, 0x00u8].as_slice()).unwrap();
    let sampler = device.create_sampler(
        gfx::tex::SamplerInfo::new(gfx::tex::Bilinear,
                                   gfx::tex::Clamp)
    );

    let program = device.link_program(VERTEX_SRC.clone(), FRAGMENT_SRC.clone()).unwrap();
    let state = gfx::DrawState::new().depth(gfx::state::LessEqual, true);
    let mut graphics = gfx::Graphics::new(device);
    let batch: CubeBatch = graphics.make_batch(&program, &mesh, slice, &state).unwrap();

    let view: AffineMatrix3<f32> = Transform::look_at(
        &Point3::new(1.5f32, -5.0, 3.0),
        &Point3::new(0f32, 0.0, 0.0),
        &Vector3::unit_z(),
    );

    let mut trans = Matrix4::<f32>::from_translation(&Vector3::new(0f32, 0f32, 0f32));

    let aspect = w as f32 / h as f32;
    let mut proj = cgmath::perspective(cgmath::deg(45.0f32), aspect, 1.0, 40.0);

    let viewm = view.mat;

    let mut tmp = viewm.mul_m(&trans);
    let mut data = Params {
        transform: proj.mul_m(&tmp).into_fixed(),
        color: (texture, Some(sampler)),
    };

    let clear_data = gfx::ClearData {
        color: [0.3, 0.3, 0.3, 1.0],
        depth: 1.0,
        stencil: 0,
    };

    let mut x: f32 = 0f32;
    let mut y: f32 = 0f32;

    let addr: SocketAddr = from_str("127.0.0.1:34000").expect("Invalid IP or Port");
    let mut conn = net::Conn::new_client(addr);

    while !window.should_close() {
        glfw.poll_events();
        let mut cubeMoved = false;
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::SizeEvent(w, h) => {
                    frame.width = w as u16;
                    frame.height = h as u16;
                    let aspect = w as f32 / h as f32;
                    proj = cgmath::perspective(cgmath::deg(45.0f32), aspect, 1.0, 40.0);
                    trans = Matrix4::<f32>::from_translation(&Vector3::new(x, 0f32, y));
                    tmp = viewm.mul_m(&trans);
                    data.transform = proj.mul_m(&tmp).into_fixed();
                }
                glfw::KeyEvent(glfw::KeyEscape, _, glfw::Press, _) =>
                    window.set_should_close(true),
                glfw::KeyEvent(glfw::KeyUp, _, glfw::Press, _) => {
                    y = y+1f32;
                    trans = Matrix4::<f32>::from_translation(&Vector3::new(x, 0f32, y));
                    tmp = viewm.mul_m(&trans);
                    data.transform = proj.mul_m(&tmp).into_fixed();
                    cubeMoved = true;
                }
                glfw::KeyEvent(glfw::KeyDown, _, glfw::Press, _) => {
                    y = y-1f32;
                    trans = Matrix4::<f32>::from_translation(&Vector3::new(x, 0f32, y));
                    tmp = viewm.mul_m(&trans);
                    data.transform = proj.mul_m(&tmp).into_fixed();
                    cubeMoved = true;
                }
                glfw::KeyEvent(glfw::KeyRight, _, glfw::Press, _) => {
                    x = x+1f32;
                    trans = Matrix4::<f32>::from_translation(&Vector3::new(x, 0f32, y));
                    tmp = viewm.mul_m(&trans);
                    data.transform = proj.mul_m(&tmp).into_fixed();
                    cubeMoved = true;
                }
                glfw::KeyEvent(glfw::KeyLeft, _, glfw::Press, _) => {
                    x = x-1f32;
                    trans = Matrix4::<f32>::from_translation(&Vector3::new(x, 0f32, y));
                    tmp = viewm.mul_m(&trans);
                    data.transform = proj.mul_m(&tmp).into_fixed();
                    cubeMoved = true;
                }

                _ => {},
            }
        }

        graphics.clear(clear_data, gfx::COLOR | gfx::DEPTH, &frame);
        graphics.draw(&batch, &data, &frame);
        graphics.end_frame();

        window.swap_buffers();

        if cubeMoved {
            println!("Sending cube moved");
            conn.send_move_cube(x, y, 0f32);
        }
    }

}

