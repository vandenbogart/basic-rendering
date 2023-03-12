use std::path::Path;

use playground::{
    asset_manager::AssetManager,
    component_manager::ComponentManager,
    components::{model::Model, transform::Transform, walk_to::WalkTo, walkable_surface::WalkableSurface, click_move::ClickMove, click::Click},
    loaders::obj::load_model,
    renderer::render::Renderer,
    systems::{camera::CameraSystem, movement::MovementSystem, click::ClickSystem},
    world::World,
    *,
};
use winit::event::KeyboardInput;
fn main() {
    pollster::block_on(run()).expect("Error");
}

pub async fn run() -> anyhow::Result<()> {
    let window = window::Window::new();

    let renderer = Renderer::new(&window).await;

    let mut world = World::new();
    let size = window.window.inner_size();
    let mut camera_system = CameraSystem::new(size.width as f32, size.height as f32);
    let mut movement_system = MovementSystem::new();
    let mut click_system = ClickSystem::new();

    let mut cm = ComponentManager::new();
    cm.register_component::<Model>();
    cm.register_component::<Transform>();
    cm.register_component::<WalkTo>();
    cm.register_component::<WalkableSurface>();
    cm.register_component::<Click>();
    cm.register_component::<ClickMove>();

    let mut am = AssetManager::new();

    let player_model = load_model(Path::new("./assets/cylinder.obj")).await?;
    let asset_handle = am.create_asset(player_model);

    let player = world.spawn();
    let model = Model { asset_handle };
    let transform = Transform::new(None, None, None);
    cm.add_component(model, player);
    cm.add_component(transform, player);
    cm.add_component(ClickMove::new(98.0), player);

    let floor = world.spawn();
    let floor_model = load_model(Path::new("./assets/floor.obj")).await?;
    let floor_asset_handle = am.create_asset(floor_model);
    let floor_model = Model {
        asset_handle: floor_asset_handle,
    };
    let floor_transform = Transform::new(None, None, None);
    cm.add_component(floor_model, floor);
    cm.add_component(floor_transform, floor);
    cm.add_component(WalkableSurface {}, floor);

    //TODO: Create drawstatebuilder in renderer and build the drawstate
    window.run(move |event| match event {
        window::Event::Redraw => {
            let entities = world.get_entities();
            let mut dsb = renderer.get_draw_state_builder();
            for &entity in entities {
                let model = cm.get_component::<Model>(entity).unwrap();
                let transform = cm.get_component::<Transform>(entity).unwrap();
                let model_asset = am.get_asset(model.asset_handle).unwrap();
                dsb.add_model_instance(model_asset, transform)
            }
            dsb.set_view_proj(camera_system.view_proj());
            let draw_state = dsb.build();
            renderer.draw(draw_state);
        }
        window::Event::Resize { width, height } => {}
        window::Event::Loop {
            delta_time,
            elapsed: _,
        } => {
            use crate::systems::System;
            camera_system.run(&mut world, &mut cm, &am, delta_time);
            movement_system.run(&mut world, &mut cm, &am, delta_time);
            click_system.run(&mut world, &mut cm, &am, delta_time);
        }
        window::Event::CursorInput { state, button } => {
            click_system.process_click(state, button, &camera_system.camera);
        }
        window::Event::CursorMove {
            x,
            y,
            modifiers: _,
        } => {
            let norm_x = x / size.width as f32;
            let norm_y = y / size.height as f32;
            click_system.process_mousemove(norm_x, norm_y);
        }
        window::Event::Keyboard {
            key:
                KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
        } => camera_system.process_keyboard(keycode, state),
        _ => (),
    });
}
