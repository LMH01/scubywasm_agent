use std::ptr::null;

mod bindings;

pub struct Context {
    seed: u32,
}

#[unsafe(no_mangle)]
pub extern "C" fn init_agent(n_agents: u32, agent_multiplicity: u32, seed: u32) -> Box<Context> {
    //null::<Context>() as *mut Context
    let context = Context {
        seed,
    };

    Box::new(context)
}

#[unsafe(no_mangle)]
pub extern "C" fn free_context(ctx: *mut Context) {}

#[unsafe(no_mangle)]
pub extern "C" fn set_config_parameter(
    ctx: *mut Context,
    param: bindings::ConfigParameter,
    value: f32,
) {
}

#[unsafe(no_mangle)]
pub extern "C" fn clear_world_state(ctx: *mut Context) {}

#[unsafe(no_mangle)]
pub extern "C" fn update_ship(
    ctx: *mut Context,
    agent_id: u32,
    hp: i32,
    x: f32,
    y: f32,
    heading: f32,
) {
}

#[unsafe(no_mangle)]
pub extern "C" fn update_shot(
    ctx: *mut Context,
    agent_id: u32,
    lifetime: i32,
    x: f32,
    y: f32,
    heading: f32,
) {
}
    
#[unsafe(no_mangle)]
pub extern "C" fn update_score(ctx: *mut Context, agent_id: u32, score: i32) {

}

#[unsafe(no_mangle)]
pub extern "C" fn make_action(ctx: &mut Context, agent_id: u32, tick: u32) -> u32 {
    //if let Some(ctx) = get_ctx(ctx) {
    //    ctx.tick_counter += 1;
    //    bindings::ActionFlags_ACTION_THRUST

    //    //bindings::ActionFlags_ACTION_THRUST
    //    //    | bindings::ActionFlags_ACTION_TURN_LEFT
    //    //    | bindings::ActionFlags_ACTION_FIRE
    //} else {
    //    bindings::ActionFlags_ACTION_NONE
    //}
    if (ctx.seed & 1) == 1 {
        bindings::ActionFlags_ACTION_THRUST
    } else {
        bindings::ActionFlags_ACTION_FIRE
    }
}
