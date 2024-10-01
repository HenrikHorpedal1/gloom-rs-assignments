// Uncomment these following global attributes to silence most warnings of "low" interest:

#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]

extern crate nalgebra_glm as glm;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::{mem, os::raw::c_void, ptr};
mod mesh;
mod scene_graph;
mod shader;
mod toolbox;
mod util;

use glutin::event::{
    DeviceEvent,
    ElementState::{Pressed, Released},
    Event, KeyboardInput,
    VirtualKeyCode::{self, *},
    WindowEvent,
};
use glutin::event_loop::ControlFlow;
use scene_graph::SceneNode;

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
unsafe fn create_vao(
    vertices: &Vec<f32>,
    colors: &Vec<f32>,
    normals: &Vec<f32>,
    indices: &Vec<u32>,
) -> u32 {
    // VAO
    let mut vao: u32 = 0;
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);

    // VBOs
    let mut vbos: Vec<u32> = vec![0; 3];
    gl::GenBuffers(3, vbos.as_mut_ptr());

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
        std::ptr::null(),
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

    // Normals
    gl::BindBuffer(gl::ARRAY_BUFFER, vbos[2]);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(&normals),
        pointer_to_array(&normals),
        gl::STATIC_DRAW,
    );
    gl::VertexAttribPointer(
        2,
        3,
        gl::FLOAT,
        gl::FALSE,
        3 * size_of::<f32>(),
        std::ptr::null(),
    );
    gl::EnableVertexAttribArray(2);

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

unsafe fn draw_scene(
    node: &scene_graph::SceneNode,
    view_projection_matrix: &glm::Mat4,
    transformation_so_far: &glm::Mat4,
    uniform_mvp: i32,
    uniform_modelmat: i32,
) {
    let translate_back = glm::translation(&node.reference_point);
    let translate_to_reference = glm::translation(&(-node.reference_point));

    let translation_matrix = glm::translation(&node.position);

    let rotation_x_matrix = glm::rotation(node.rotation[0], &glm::vec3(1.0, 0.0, 0.0));
    let rotation_y_matrix = glm::rotation(node.rotation[1], &glm::vec3(0.0, 1.0, 0.0));
    let rotation_z_matrix = glm::rotation(node.rotation[2], &glm::vec3(0.0, 0.0, 1.0));

    let rotation_matrix = rotation_x_matrix * rotation_y_matrix * rotation_z_matrix; // The animation is going to look a little off, due to us so far using extrinsic euler angles instead of intrinsic angles. To partially mitigate this we suggest first applying Z rotation, then the Y rotation, then the X rotation.

    let scaling_matrix = glm::scaling(&node.scale); // allways unit scaling anyways

    let local_transformation = translation_matrix
        * translate_back
        * rotation_matrix
        * translate_to_reference           
        * scaling_matrix; 

    let accumulated_transformation = transformation_so_far * local_transformation;

    if node.index_count != -1 {
        unsafe {
            let mvp_matrix = view_projection_matrix * accumulated_transformation;

            gl::UniformMatrix4fv(uniform_mvp, 1, gl::FALSE, mvp_matrix.as_ptr());
            gl::UniformMatrix4fv(
                uniform_modelmat,
                1,
                gl::FALSE,
                accumulated_transformation.as_ptr(),
            );

            gl::BindVertexArray(node.vao_id);
            gl::DrawElements(
                gl::TRIANGLES,
                node.index_count,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }

    // Recurse
    for &child in &node.children {
        draw_scene(
            &*child,
            view_projection_matrix,
            &accumulated_transformation,
            uniform_mvp,
            uniform_modelmat,
        );
    }
}

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(
            INITIAL_SCREEN_W,
            INITIAL_SCREEN_H,
        ));
    let cb = glutin::ContextBuilder::new().with_vsync(true);
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
            // Print some diagnostics println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!(
                "GLSL\t: {}",
                util::get_gl_string(gl::SHADING_LANGUAGE_VERSION)
            );
        }
        // load meshes
        let lunar_terrain_path: &str = "./resources/lunarsurface.obj";
        let helicopter_path: &str = "./resources/helicopter.obj";
        let bomb_path: &str = "./resources/bomb.obj";
        let lunar_terrain_mesh = mesh::Terrain::load(lunar_terrain_path);
        let heilcopter = mesh::Helicopter::load(helicopter_path);
        let bomb_mesh = mesh::Bomb::load(bomb_path);

        

        // VAOs

        let mut lunar_surface_vao = unsafe {
            create_vao(
                &lunar_terrain_mesh.vertices,
                &lunar_terrain_mesh.colors,
                &lunar_terrain_mesh.normals,
                &lunar_terrain_mesh.indices,
            )
        };

        let heli_body_vao = unsafe {
            create_vao(
                &heilcopter.body.vertices,
                &heilcopter.body.colors,
                &heilcopter.body.normals,
                &heilcopter.body.indices,
            )
        };
        let heli_door_vao = unsafe {
            create_vao(
                &heilcopter.door.vertices,
                &heilcopter.door.colors,
                &heilcopter.door.normals,
                &heilcopter.door.indices,
            )
        };
        let heli_main_rotor_vao = unsafe {
            create_vao(
                &heilcopter.main_rotor.vertices,
                &heilcopter.main_rotor.colors,
                &heilcopter.main_rotor.normals,
                &heilcopter.main_rotor.indices,
            )
        };
        let heli_tail_rotor_vao = unsafe {
            create_vao(
                &heilcopter.tail_rotor.vertices,
                &heilcopter.tail_rotor.colors,
                &heilcopter.tail_rotor.normals,
                &heilcopter.tail_rotor.indices,
            )
        };
        let bomb_vao = unsafe {
            create_vao(
                &bomb_mesh.vertices,
                &bomb_mesh.colors,
                &bomb_mesh.normals,
                &bomb_mesh.indices,
            )};

        // Scene nodes
        let mut terrain_root_node = SceneNode::new();
        let mut terrain_node =
        SceneNode::from_vao(lunar_surface_vao, lunar_terrain_mesh.index_count);
        terrain_root_node.add_child(&terrain_node);


        // Multiple helicopters:
        let num_helicopters = 5;
        let mut heli_bodies = vec![];
        let mut heli_main_rotors = vec![];
        let mut heli_tail_rotors = vec![];
        let mut heli_doors = vec![];
        let mut bombs = vec![];

        for i in 0..num_helicopters {
            
            let mut heli_root_node = SceneNode::new();
            terrain_node.add_child(&heli_root_node);

            let mut body_node = SceneNode::from_vao(heli_body_vao, heilcopter.body.index_count);
            heli_root_node.add_child(&body_node);

            let mut door_node = SceneNode::from_vao(heli_door_vao, heilcopter.door.index_count);
            body_node.add_child(&door_node);

            let mut main_rotor_node =
                SceneNode::from_vao(heli_main_rotor_vao, heilcopter.main_rotor.index_count);
            body_node.add_child(&main_rotor_node);

            let mut tail_rotor_node =
                SceneNode::from_vao(heli_tail_rotor_vao, heilcopter.tail_rotor.index_count);
            body_node.add_child(&tail_rotor_node);

            let mut bomb_root = SceneNode::new();
            let mut bomb_node = 
            SceneNode::from_vao(bomb_vao,bomb_mesh.index_count);
            body_node.add_child(&bomb_root);
            bomb_root.add_child(&bomb_node);


            // Inital positions
            bomb_node.scale = glm::vec3(0.01,0.01,0.01); 
            
            tail_rotor_node.reference_point = glm::vec3(0.35, 2.3, 10.4);
            body_node.position = glm::vec3(0.0, 10.0, 0.0);
            body_node.rotation.y = 3.14;

            heli_bodies.push(body_node);
            heli_main_rotors.push(main_rotor_node);
            heli_tail_rotors.push(tail_rotor_node);
            heli_doors.push(door_node);
            bombs.push(bomb_node);
        }
        // == // Set up your shaders here
        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.frag")
                .attach_file("./shaders/simple.vert")
                .link()
        };
        unsafe {
            simple_shader.activate();
        }

        let uniform_mvp = unsafe { simple_shader.get_uniform_location("mvp") };
        let uniform_modelmat = unsafe { simple_shader.get_uniform_location("modelmat") };

        // Camera stuff
        let starting_position = glm::vec3(0.0, 0.0, 1.0);
        let mut camera_position = starting_position;
        let starting_horizontal_rot = 0.0;
        let starting_vertical_rot = 0.0;
        let mut horizontal_rot: f32 = starting_horizontal_rot;
        let mut vertical_rot: f32 = starting_vertical_rot;

        let mut open_door = false;
        let mut drop_bomb = false;
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
                    unsafe {
                        gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32);
                    }
                }
            }

            let forward = glm::vec3(
                horizontal_rot.sin() * vertical_rot.cos(),
                vertical_rot.sin(),
                -horizontal_rot.cos() * vertical_rot.cos(),
            );
            let right = glm::normalize(&glm::cross(&forward, &glm::vec3(0.0, 1.0, 0.0)));
            let up = glm::cross(&right, &forward);

            // Handle keyboard input
            let mut movement_direction = glm::vec3(0.0, 0.0, 0.0);

            
            let door_angle = 0;
            
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // The VirtualKeyCode enum is defined here:
                        // https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html

                        // Translations
                        VirtualKeyCode::W => movement_direction += forward,
                        VirtualKeyCode::S => movement_direction -= forward,
                        VirtualKeyCode::D => movement_direction += right,
                        VirtualKeyCode::A => movement_direction -= right,
                        VirtualKeyCode::Space => movement_direction += up,
                        VirtualKeyCode::LShift => movement_direction -= up,

                        // Rotations
                        VirtualKeyCode::Right => horizontal_rot += delta_time,
                        VirtualKeyCode::Left => horizontal_rot -= delta_time,
                        VirtualKeyCode::Up => {
                            vertical_rot += delta_time;
                            vertical_rot = vertical_rot
                                .clamp(-std::f32::consts::PI / 3.0, std::f32::consts::PI / 3.0);
                        }
                        VirtualKeyCode::Down => {
                            vertical_rot -= delta_time;
                            vertical_rot = vertical_rot
                                .clamp(-std::f32::consts::PI / 3.0, std::f32::consts::PI / 3.0);
                        }

                        // get back to starting point
                        VirtualKeyCode::R => {
                            camera_position = starting_position;
                            horizontal_rot = starting_horizontal_rot;
                            vertical_rot = starting_vertical_rot;
                        }

                        // open all doors
                        VirtualKeyCode::K => open_door = true,

                        // Drop a bomb
                        VirtualKeyCode::B => drop_bomb = true,
                        // default handler:
                        _ => {}
                    }
                }
            }

            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {
                // == // Optionally access the accumulated mouse movement between
                // == // frames here with delta.0 and delta.1

                *delta = (0.0, 0.0); // reset when done
            }

            if glm::length(&movement_direction) > 0.0 {
                movement_direction = glm::normalize(&movement_direction);
                let speed = 20.0;
                camera_position += movement_direction * delta_time * speed;
            }

            let view_matrix = glm::look_at(&camera_position, &(camera_position + forward), &up);

            let projection_mat: glm::Mat4 = glm::perspective(
                window_aspect_ratio, // aspect ratio
                1.3962634,           // 80 degrees, Fov
                1.0,                 // near
                1000.0,              // far
            );

            let view_projection_mat = projection_mat * view_matrix;
            let mut offset = 0.8;
            for i in 0..num_helicopters {

                // rotor movement
                let rotor_speed = 5.0;
                heli_main_rotors[i].rotation += glm::vec3(0.0, rotor_speed * delta_time, 0.0);
                heli_tail_rotors[i].rotation += glm::vec3(rotor_speed * delta_time, 0.0, 0.0);

                // Helicopter path
                let heading = toolbox::simple_heading_animation(elapsed + offset * i as f32);
                heli_bodies[i].position[0] = heading.x;
                heli_bodies[i].position[2] = heading.z;
                heli_bodies[i].rotation = glm::vec3(heading.pitch, heading.yaw, heading.roll);


                //doors
                if open_door {

                    let door_speed = 0.5;
                    heli_doors[i].position.z += door_speed * delta_time;
                    heli_doors[i].position.z = heli_doors[i].position.z.clamp(0.0,1.6);
                }

                //bombs
                if drop_bomb {
                    let drop_speed = 4.0;
                    bombs[i].position.y -= drop_speed * delta_time;
                    bombs[i].position.y = bombs[i].position.y.clamp(-15.0,1000.0);
                }
            }
           



            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
 
                draw_scene(
                    &terrain_node,
                    &view_projection_mat,
                    &glm::Mat4x4::identity(),
                    uniform_mvp,
                    uniform_modelmat,
                );
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
            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                println!(
                    "New window size received: {}x{}",
                    physical_size.width, physical_size.height
                );
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: key_state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        }
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    }
                    Q => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => {}
        }
    });
}
