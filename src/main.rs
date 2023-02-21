use std::f32::consts::PI;

use cgmath::{Quaternion, Rotation3};
use playground::{
    renderer::{
        render::Renderer, CameraFollowComponent, GeometryComponent, ModelComponent, ModelResource,
    },
    systems::{input::InputSystem, movement::MovementSystem, System, WASDControllerComponent, camera::CameraSystem},
    *,
};
use winit::event::KeyboardInput;
fn main() {
    pollster::block_on(run()).expect("Error");
}

pub async fn run() -> anyhow::Result<()> {
    let window = window::Window::new();
    let mut world = World::new();
    let mut renderer = Renderer::new(&window).await;
    let mut input = InputSystem::new();
    let mut movement = MovementSystem::new();
    let window_size = window.window.inner_size();
    let aspect = window_size.width as f32 / window_size.height as f32;
    let mut camera = CameraSystem::new(aspect);
    let resource = renderer
        .create_model_resource(String::from("./assets/racecar.obj"))
        .await;
    let model_index = world.create_resource::<ModelResource>(resource);

    world.register_component::<ModelComponent>();
    world.register_component::<GeometryComponent>();
    world.register_component::<WASDControllerComponent>();
    world.register_component::<CameraFollowComponent>();

    let entity = world.spawn();
    let model_component = ModelComponent::new(model_index);

    world.add_component(entity, model_component);
    world.add_component(entity, WASDControllerComponent::default());
    world.add_component(
        entity,
        GeometryComponent::new(None, None, Some(cgmath::Vector3::unit_x() * -1.0)),
    );
    world.add_component(entity, CameraFollowComponent {});

    let entity = world.spawn();
    let model_component = ModelComponent::new(model_index);

    world.add_component(entity, model_component);
    world.add_component(
        entity,
        GeometryComponent::new(Some([5.0, 0.0, 5.0].into()), None, None),
    );
    // let resource = renderer.create_model_resource(String::from("./assets/frog.obj")).await;
    // let model_index = world.create_resource::<ModelResource>(resource);

    // let entity = world.spawn();
    // let model_component = ModelComponent::new(model_index);

    // world.add_component(entity, model_component);
    // world.add_component(entity, GeometryComponent::new(Some([5.0, 0.0, 5.0].into()), Some(Quaternion::from_angle_y(cgmath::Rad(PI)))));

    window.run(move |event| match event {
        window::Event::Redraw => {
            renderer.draw(&mut world, camera.view_proj());
        }
        window::Event::Resize { width, height } => {
            renderer.resize(width, height);
            camera.resize(width, height);
        }
        window::Event::Loop { delta_time } => {
            input.run(&mut world, delta_time);
            movement.run(&mut world, delta_time);
            camera.run(&mut world, delta_time);
        }
        window::Event::Keyboard {
            key:
                KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
        } => {
            camera.process_keyboard(keycode, state);
            input.process_keyboard(keycode, state);
        }
        _ => (),
    });
}
