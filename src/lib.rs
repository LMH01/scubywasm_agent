mod bindings;

#[repr(C)]
pub struct Context {
    n_agents: u32,
    agent_multiplicity: u32,
    seed: u32,
    tick_counter: u32,
}

#[no_mangle]
pub extern "C" fn init_agent(
    n_agents: u32,
    agent_multiplicity: u32,
    seed: u32,
) -> *mut Context {
    let ctx = Context {
        n_agents,
        agent_multiplicity,
        seed,
        tick_counter: 0,
    };

    Box::into_raw(Box::new(ctx))
}

#[no_mangle]
pub extern "C" fn free_context(ctx: *mut Context) {
    if ctx.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(ctx));
    }
}

fn get_ctx<'a>(ctx: *mut Context) -> Option<&'a mut Context> {
    if ctx.is_null() {
        None
    } else {
        Some(unsafe { &mut *ctx })
    }
}

#[no_mangle]
pub extern "C" fn clear_world_state(ctx: *mut Context) {
    if let Some(ctx) = get_ctx(ctx) {
        ctx.tick_counter = 0;
    }
}

#[no_mangle]
pub extern "C" fn set_config_parameter(
    ctx: *mut Context,
    param: bindings::ConfigParameter,
    value: f32,
) {
    if let Some(ctx) = get_ctx(ctx) {
        match param {
            bindings::ConfigParameter_CFG_SHIP_MAX_TURN_RATE => {
                println!("Max turn rate: {}", value);
            }
            bindings::ConfigParameter_CFG_SHIP_MAX_VELOCITY => {
                println!("Max velocity: {}", value);
            }
            _ => {}
        }
    }
}

#[no_mangle]
pub extern "C" fn make_action(
    ctx: *mut Context,
    agent_id: u32,
    tick: u32,
) -> u32 {
    if let Some(ctx) = get_ctx(ctx) {
        ctx.tick_counter += 1;

        bindings::ActionFlags_ACTION_THRUST
            | bindings::ActionFlags_ACTION_TURN_LEFT
            | bindings::ActionFlags_ACTION_FIRE
    } else {
        bindings::ActionFlags_ACTION_NONE
    }
}
