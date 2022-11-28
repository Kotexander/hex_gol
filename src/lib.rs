use winit::event::ElementState;
use winit::event::Event;
use winit::event::KeyboardInput;
use winit::event::VirtualKeyCode;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

mod game;
use game::*;
mod renderer;
use renderer::*;

const WHITE: [f32; 3] = [1.0, 1.0, 1.0];

struct Renderer {
    gfx: Graphics,
    hex_instanced: InstancedMesh,
}

#[cfg_attr(target_os = "android", ndk_glue::main())]
pub fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Hex Game of Life")
        .build(&event_loop)
        .unwrap();

    let mut game = HexGOL::new(35);
    game.randomize();

    let mut app = None;

    let fps = std::time::Duration::from_secs_f32(1.0 / 15.0);
    let mut timer = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::Resumed => {
            let size = window.inner_size();
            let size = [size.width, size.height];
            let gfx = pollster::block_on(Graphics::new(size, &window));
            let hex = MeshBuilder::new_hexagon([0.0, 0.0], 1.0).build(gfx.context());
            let hex_instanced = InstancedMesh::new(hex, gfx.context(), &[]);

            app = Some(Renderer { gfx, hex_instanced });
        }
        Event::Suspended => {
            app = None;
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            if let Some(Renderer { gfx, hex_instanced }) = &mut app {
                let mut instances = vec![];
                for (hex, cell) in game.iter() {
                    if *cell {
                        instances.push(Instance::new(
                            HexFract::from(*hex).transform(1.0),
                            [1.0, 1.0],
                            WHITE,
                        ));
                    }
                }
                hex_instanced.update(gfx.context(), &instances);

                let mut render_pass = gfx.start_frame();
                hex_instanced.draw(&mut render_pass);

                drop(render_pass);
                gfx.end_frame();
            }
        }
        Event::MainEventsCleared => {
            if let Some(Renderer { gfx, .. }) = &mut app {
                gfx.update();
            }
            if timer.elapsed() > fps {
                timer = std::time::Instant::now();
                game.update();
            }
            // RedrawRequested will only trigger once, unless we manually request it.
            window.request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                if let Some(Renderer { gfx, .. }) = &mut app {
                    let size = [physical_size.width, physical_size.height];
                    gfx.resize(size);
                }
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                if let Some(Renderer { gfx, .. }) = &mut app {
                    // new_inner_size is &&mut so w have to dereference it twice
                    let size = [new_inner_size.width, new_inner_size.height];
                    gfx.resize(size);
                }
            }
            _ => {}
        },
        _ => {}
    });
}
