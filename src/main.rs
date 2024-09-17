// Uncomment these following global attributes to silence most warnings of "low" interest:
/*
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
*/
extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};
mod shader;

mod util;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  byte_size_of_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()


// == // Generate your VAO here
unsafe fn create_vao(vertices: &Vec<f32>, colors: &Vec<f32>, indices: &Vec<u32>) -> u32 {
    // VAO 
    let mut vao: u32 = 0;
    gl::GenVertexArrays(1, &mut vao); 
    gl::BindVertexArray(vao);

    // VBOs
    let mut vbos: Vec<u32> = vec![0; 2]; 
    gl::GenBuffers(2, vbos.as_mut_ptr());

    // Position
    gl::BindBuffer(gl::ARRAY_BUFFER, vbos[0]);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(&vertices),
        pointer_to_array(&vertices),
        gl::STATIC_DRAW,
    );
    gl::VertexAttribPointer(
        0,           
        3,            
        gl::FLOAT,     
        gl::FALSE,      
        3 * size_of::<f32>(),
        std::ptr::null() 
    );
    gl::EnableVertexAttribArray(0); 

    // Colors
    gl::BindBuffer(gl::ARRAY_BUFFER, vbos[1]);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(&colors),
        pointer_to_array(&colors),
        gl::STATIC_DRAW,
    );
    gl::VertexAttribPointer(
        1,
        4,
        gl::FLOAT,
        gl::FALSE,
        4 * size_of::<f32>(),
        std::ptr::null(),
    );
    gl::EnableVertexAttribArray(1);

    // Now rotate the colors
    //let chunks: Vec<_> = colors.chunks(12).collect();
    //let rotated_colors: Vec<f32> = chunks.iter()
    //    .cycle()
    //    .skip(1) // Rotate by 1
    //    .take(chunks.len())
    //    .flat_map(|chunk| chunk.iter()) // Flatten the chunks back into a single vector
    //    .copied()
    //    .collect();

    //gl::BufferSubData(
    //        gl::ARRAY_BUFFER,                 // Target buffer
    //        0,                                // Offset, start at the beginning of the buffer
    //        byte_size_of_array(&rotated_colors), // Size of the data to update
    //        pointer_to_array(&rotated_colors),   // Pointer to the rotated color data
    //    );



    // Now rotate the z-values of the vertices
    //let mut swapped_z_vertices: Vec<f32> = vec![];
    //let left_eye = vec![
    //        -0.4, 0.5, -0.7,
    //        -0.4, 0.1, -0.7,
    //        0.3, 0.4, -0.7,
    //];

    //let right_eye = vec![
    //    0.5, 0.7, -0.5,
    //    -0.5, -0.3, -0.5,
    //    0.5, -0.2, -0.5,
    //];

    //let mouth = vec![
    //    -0.1, 0.6, 0.0,
    //    0.2, 0.0, 0.0,
    //    0.5, 0.4, 0.0,
    //];
    //swapped_z_vertices.extend(left_eye);
    //swapped_z_vertices.extend(right_eye);
    //swapped_z_vertices.extend(mouth);

    //gl::BindBuffer(gl::ARRAY_BUFFER, vbos[0]);
    //gl::BufferSubData(
    //    gl::ARRAY_BUFFER,                 // Target buffer
    //    0,                                // Offset, start at the beginning of the buffer
    //    byte_size_of_array(&swapped_z_vertices), // Size of the data to update
    //    pointer_to_array(&swapped_z_vertices),   // Pointer to the modified vertex data
    //);
    
    // IBO 
    let mut ibo: u32 = 0;
    gl::GenBuffers(1, &mut ibo);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
    
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        byte_size_of_array(&indices),
        pointer_to_array(&indices),
        gl::STATIC_DRAW,
    );

    vao
}

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(INITIAL_SCREEN_W, INITIAL_SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // == // Set up your VAO around here

        let number_of_triangles = 3;

        let left_eye = vec![
            -0.4, 0.5, 0.0,
            -0.4, 0.1, 0.0,
            0.3, 0.4, 0.0,
        ];

        let left_eye_color: Vec<f32> = vec![
            1.0, 0.0, 0.0, 0.5, // Vertex 1: Red
            0.0, 1.0, 0.0, 0.5, // Vertex 2: Green
            0.0, 0.0, 1.0, 0.5, // Vertex 3: Blue
        ];

        let right_eye = vec![
            0.5, 0.7, -0.5,
            -0.5, -0.3, -0.5,
            0.5, -0.2, -0.5,
        ];

        let right_eye_color: Vec<f32> = vec![
            1.0, 1.0, 0.0, 0.5, // Vertex 1: Yellow
            1.0, 0.5, 0.0, 0.5, // Vertex 2: Orange
            0.5, 0.0, 1.0, 0.5, // Vertex 3: Purple
        ];

        let mouth = vec![
            -0.1, 0.6, -0.7,
            0.2, 0.0, -0.7,
            0.5, 0.4, -0.7,
        ];

        let mouth_color: Vec<f32> = vec![
            0.5, 0.5, 0.5, 0.5, // Vertex 1: Light Gray
            0.75, 0.75, 0.75, 0.5, // Vertex 2: Medium Gray
            0.9, 0.9, 0.9, 0.5, // Vertex 3: Dark Gray
        ];

        let mut vertex_array: Vec<f32> = vec![];
        let mut color_array: Vec<f32> = vec![];

        vertex_array.extend(left_eye);
        vertex_array.extend(right_eye);
        vertex_array.extend(mouth);

        color_array.extend(left_eye_color);
        color_array.extend(right_eye_color);
        color_array.extend(mouth_color);       
        let indices = (0..number_of_triangles*3).collect();
        let my_vao = unsafe { create_vao(&vertex_array,&color_array, &indices) };

        println!("vertex array: {:#?}", vertex_array);
        println!("vertex array: {:#?}", indices);


        // == // Set up your shaders here

        // Basic usage of shader helper:
        // The example code below creates a 'shader' object.
        // It which contains the field `.program_id` and the method `.activate()`.
        // The `.` in the path is relative to `Cargo.toml`.
        // This snippet is not enough to do the exercise, and will need to be modified (outside
        // of just using the correct path), but it only needs to be called once

        
        let simple_shader = unsafe{
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.frag")
                .attach_file("./shaders/simple.vert")
                .link()
        };
        unsafe{
            simple_shader.activate();
        }

        let uniform_location = unsafe {
            simple_shader.get_uniform_location("transformationmat")
        };


        let mut x_translation: f32 = 0.0;
        let mut y_translation: f32 = 0.0;
        let mut z_translation: f32 = 0.0;
        
        let mut horizontal_rot: f32 = 0.0;
        let mut vertical_rot: f32 = 0.0;

        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut previous_frame_time = first_frame_time;
        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
            previous_frame_time = now;

            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                    window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                    (*new_size).2 = false;
                    println!("Window was resized to {}x{}", new_size.0, new_size.1);
                    unsafe { gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32); }
                }
            }

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // The `VirtualKeyCode` enum is defined here:
                        //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html
                        VirtualKeyCode::W => {
                            z_translation += delta_time; 
                        }
                        VirtualKeyCode::S => {
                            z_translation -= delta_time;
                        }

                        VirtualKeyCode::D => {
                            x_translation -= delta_time;
                        }
                        VirtualKeyCode::A => {
                            x_translation += delta_time;
                        }
                        VirtualKeyCode::Space => {
                            y_translation -= delta_time;
                        }
                        VirtualKeyCode::LShift => {
                            y_translation += delta_time;
                        }

                        VirtualKeyCode::Up => {
                            vertical_rot -= delta_time;
                            vertical_rot = vertical_rot.clamp(-std::f32::consts::PI/3.0,std::f32::consts::PI/3.0);
                        }
                        VirtualKeyCode::Down => {
                            vertical_rot += delta_time;
                            vertical_rot = vertical_rot.clamp(-std::f32::consts::PI/3.0,std::f32::consts::PI/3.0);
                        }
                        VirtualKeyCode::Right => {
                            horizontal_rot += delta_time;
                        }
                        VirtualKeyCode::Left => {
                            horizontal_rot -= delta_time;
                        }

                        // default handler:
                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {

                // == // Optionally access the accumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            // == // Please compute camera transforms here (exercise 2 & 3)
            //let projection_mat: glm::Mat4 = glm::identity();
            let z_offset = -1.0;
            let translational_mat: glm::Mat4 = 
                glm::translation(&glm::vec3(x_translation, y_translation, z_translation + z_offset));

            let vertical_rot_matrix: glm::Mat4 =
                glm::rotation(vertical_rot, &glm::vec3(1.0,0.0,0.0));
            let horizontal_rot_matrix: glm::Mat4 =
                glm::rotation(horizontal_rot, &glm::vec3(0.0,1.0,0.0));
            let projection_mat: glm::Mat4 = 
                glm::perspective(
                    window_aspect_ratio, //aspect ration
                    1.3962634,// 80 degrees, vertical FOV
                    1.0,   //near
                    100.0,   //far
                    );

            let combined_transformation: glm::Mat4 = projection_mat * translational_mat * vertical_rot_matrix * horizontal_rot_matrix;

            unsafe {
                gl::UniformMatrix4fv(uniform_location, 1, gl::FALSE, combined_transformation.as_ptr());
            }

            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);


                // == // Issue the necessary gl:: commands to draw your scene here
                gl::BindVertexArray(my_vao);
                gl::DrawElements(gl::TRIANGLES,9,gl::UNSIGNED_INT,std::ptr::null());
            }

            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    });


    // == //
    // == // From here on down there are only internals.
    // == //


    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events are initially handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::Resized(physical_size), .. } => {
                println!("New window size received: {}x{}", physical_size.width, physical_size.height);
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => { *control_flow = ControlFlow::Exit; }
                    Q      => { *control_flow = ControlFlow::Exit; }
                    _      => { }
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => { }
        }
    });
}
