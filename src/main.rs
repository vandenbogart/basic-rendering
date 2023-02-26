use playground::{
    renderer::{
        render::Renderer, CameraFollowComponent, GeometryComponent, ModelComponent, ModelResource,
    },
    systems::{
        camera::CameraSystem, input::InputSystem, movement::MovementSystem, ClickMoveComponent,
        System,
    },
    *,
};
use winit::event::KeyboardInput;
fn main() {
    pollster::block_on(run()).expect("Error");
}

pub async fn run() -> anyhow::Result<()> {
    let window = window::Window::new();

    let renderer = Renderer::new(&window).await;
    let mut input = InputSystem::new();
    let mut movement = MovementSystem::new();
    let window_size = window.window.inner_size();
    let mut camera_system = CameraSystem::new(window_size.width as f32, window_size.height as f32);

    let mut world = World::new(renderer);

    world.register_component::<ModelComponent>();
    world.register_component::<GeometryComponent>();
    world.register_component::<ClickMoveComponent>();
    world.register_component::<CameraFollowComponent>();

    let player_model = world
        .create_resource::<ModelResource>("./assets/cylinder.obj")
        .await;
    let floor_model = world
        .create_resource::<ModelResource>("./assets/floor.obj")
        .await;

    let entity = world.spawn();
    let model_component = ModelComponent::new(player_model);

    world.add_component(entity, model_component);

    world.add_component(
        entity,
        GeometryComponent::new(None, None, Some(cgmath::Vector3::unit_x() * -1.0)),
    );
    world.add_component(entity, CameraFollowComponent {});
    world.add_component(
        entity,
        ClickMoveComponent::new(40.0),
    );

    let entity = world.spawn();
    let model_component = ModelComponent::new(floor_model);
    world.add_component(entity, model_component);
    world.add_component(
        entity,
        GeometryComponent::new(Some([0.0, 0.0, 0.0].into()), None, None),
    );
    // let resource = renderer.create_model_resource(String::from("./assets/frog.obj")).await;
    // let model_index = world.create_resource::<ModelResource>(resource);

    // let entity = world.spawn();
    // let model_component = ModelComponent::new(model_index);

    // world.add_component(entity, model_component);
    // world.add_component(entity, GeometryComponent::new(Some([5.0, 0.0, 5.0].into()), Some(Quaternion::from_angle_y(cgmath::Rad(PI)))));

    window.run(move |event| match event {
        window::Event::Redraw => {
            pollster::block_on(world.draw(camera_system.view_proj()));
        }
        window::Event::Resize { width, height } => {
            world.resize(width, height);
            camera_system.resize(width, height);
        }
        window::Event::Loop { delta_time, elapsed } => {
            camera_system.run(&mut world, delta_time);
            input.run(&mut world, delta_time);
            movement.run(&mut world, delta_time);
        }
        window::Event::CursorInput { state, button } => {
            input.process_mouse_click(state, button);
        }
        window::Event::CursorMove { x, y, modifiers } => {
            input.process_mouse_move(x, y, modifiers);
        }
        window::Event::Keyboard {
            key:
                KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
        } => {
            camera_system.process_keyboard(keycode, state);
            input.process_keyboard(keycode, state);
        }
        _ => (),
    });
}
