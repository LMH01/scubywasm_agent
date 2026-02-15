use std::{cmp::Ordering, collections::HashMap, os::raw::c_char};

use bindings::ActionFlags_ACTION_NONE;
use config::Config;

mod bindings;
mod config;
mod logging;

#[derive(Default)]
pub struct Context {
    config: Config,
    seed: u32,
    world_state: WorldState,
    /// stores ships that are owned by this agent that did not yet receive instructions on what to do next
    own_ships_to_action: Vec<Ship>,
    /// Agent id of this agent, is set the first time make_aktion is called
    own_agent_id: Option<u32>,
}

#[unsafe(no_mangle)]
pub extern "C" fn init_agent(n_agents: u32, agent_multiplicity: u32, seed: u32) -> Box<Context> {
    //null::<Context>() as *mut Context
    let context = Context {
        config: Config::default(),
        seed,
        world_state: WorldState::default(),
        own_ships_to_action: Vec::new(),
        own_agent_id: None,
    };

    Box::new(context)
}

#[unsafe(no_mangle)]
pub extern "C" fn free_context(ctx: &mut Context) {
    *ctx = Context::default();
}

#[unsafe(no_mangle)]
pub extern "C" fn set_config_parameter(
    ctx: &mut Context,
    param: bindings::ConfigParameter,
    value: f32,
) {
    ctx.config.update(param, value)
}

#[derive(Default)]
struct WorldState {
    /// stores all ships on the playfield
    ships: HashMap<u32, Vec<Ship>>,
    shots: HashMap<u32, Vec<Shot>>,
    agents: Vec<Agent>,
}

#[derive(Default, Clone)]
struct Ship {
    hp: i32,
    pos_x: f32,
    pos_y: f32,
    heading: f32,
}

#[derive(Default)]
struct Shot {
    lifetime: i32,
    pos_x: f32,
    pos_y: f32,
    heading: f32,
}

#[derive(Default)]
struct Agent {
    agent_id: u32,
    score: i32,
}

#[unsafe(no_mangle)]
pub extern "C" fn clear_world_state(ctx: &mut Context) {
    ctx.world_state = WorldState::default();
}

#[unsafe(no_mangle)]
pub extern "C" fn update_ship(
    ctx: &mut Context,
    agent_id: u32,
    hp: i32,
    pos_x: f32,
    pos_y: f32,
    heading: f32,
) {
    let ship = Ship {
        hp,
        pos_x,
        pos_y,
        heading: (90.0 - heading).to_radians(),
    };
    if let Some(ships) = ctx.world_state.ships.get_mut(&agent_id) {
        ships.push(ship.clone());
    } else {
        ctx.world_state.ships.insert(agent_id, vec![ship.clone()]);
    }
    // check if ship to update belongs to this agent
    if let Some(own_agent_id) = ctx.own_agent_id {
        if own_agent_id == agent_id {
            ctx.own_ships_to_action.push(ship)
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn update_shot(
    ctx: &mut Context,
    agent_id: u32,
    lifetime: i32,
    pos_x: f32,
    pos_y: f32,
    heading: f32,
) {
    let shot = Shot {
        lifetime,
        pos_x,
        pos_y,
        heading,
    };
    if let Some(shots) = ctx.world_state.shots.get_mut(&agent_id) {
        shots.push(shot);
    } else {
        ctx.world_state.shots.insert(agent_id, vec![shot]);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn update_score(ctx: &mut Context, agent_id: u32, score: i32) {
    ctx.world_state.agents.push(Agent { agent_id, score })
}

#[derive(Default)]
struct Action {
    enable_thrusters: bool,
    turn_direction: Option<TurnDirection>,
    fire: bool,
}

impl Into<u32> for Action {
    fn into(self) -> u32 {
        let enable_thrusters = if self.enable_thrusters {
            bindings::ActionFlags_ACTION_THRUST
        } else {
            0
        };
        let turn_direction = self.turn_direction.map_or(0, |f| f.into());
        let fire = if self.fire {
            bindings::ActionFlags_ACTION_FIRE
        } else {
            0
        };
        enable_thrusters | turn_direction | fire
    }
}

enum TurnDirection {
    Left,
    Right,
}

impl Into<u32> for TurnDirection {
    fn into(self) -> u32 {
        match self {
            Self::Left => bindings::ActionFlags_ACTION_TURN_LEFT,
            Self::Right => bindings::ActionFlags_ACTION_TURN_RIGHT,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn make_action(ctx: &mut Context, own_agent_id: u32, tick: u32) -> u32 {
    if ctx.own_agent_id.is_none() {
        ctx.own_agent_id = Some(own_agent_id);
    }
    let world_state = &ctx.world_state;
    let mut action = Action::default();
    let current_ship_to_action = match ctx.own_ships_to_action.pop() {
        Some(ship) => ship,
        None => {
            // no ship found for which an action could be calculated, so we do nothing
            // should only be run on first action because own ships are not yet initialized
            return bindings::ActionFlags_ACTION_NONE;
        }
    };
    // TODO implement detection of shots to try and evade them

    // acquire target
    let mut target: Option<(f32, Ship)> = None;
    for (agent_id, ships) in &world_state.ships {
        if *agent_id == own_agent_id {
            // we don't want to lock an allied ship as target
            continue;
        }
        // found ships of enemy, determine what ship is closest, choose that as target
        for ship in ships {
            let x1 = ship.pos_x;
            let y1 = ship.pos_y;
            let x2 = current_ship_to_action.pos_x;
            let y2 = current_ship_to_action.pos_y;
            let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt().abs();
            if let Some((current_min_distance, ship)) = &target {
                // only update target when distance is closer
                if current_min_distance > &distance {
                    // closer target found, update target
                    target = Some((distance, ship.clone()));
                }
            } else {
                // no target yet, set target
                target = Some((distance, ship.clone()));
            }
        }
    }
    let (_, target) = match target {
        Some(target) => target,
        None => {
            // no target found, so game *should* be won already
            return action.into();
        }
    };

    // target found, determine action

    // find direction in which the target is
    let x1 = target.pos_x;
    let y1 = target.pos_y;
    let x2 = current_ship_to_action.pos_x;
    let y2 = current_ship_to_action.pos_y;

    let target_angle = (y1 - y2).atan2(x1 - x2);
    let current_angle = current_ship_to_action.heading;

    // Smallest signed angle difference (-pi .. pi)
    let mut angle_diff = target_angle - current_angle;

    // Normalize to [-pi, pi]
    while angle_diff > std::f32::consts::PI {
        angle_diff -= 2.0 * std::f32::consts::PI;
    }
    while angle_diff < -std::f32::consts::PI {
        angle_diff += 2.0 * std::f32::consts::PI;
    }

    let movement = if angle_diff.abs() < 0.01 {
        None
    } else if angle_diff > 0.0 {
        Some(TurnDirection::Left)
    } else {
        Some(TurnDirection::Right)
    };

    log!(
        "Agent: {own_agent_id}, Current position: [{},{}], Target direction: {}, Current direction: {}",
        current_ship_to_action.pos_x,
        current_ship_to_action.pos_y,
        90.0 - target_angle.to_degrees(),
        90.0 - current_angle.to_degrees()
    );

    action.turn_direction = movement;
    action.enable_thrusters = true;
    action.into()

    //if (ctx.seed & 1) == 1 {
    //    bindings::ActionFlags_ACTION_THRUST
    //} else {
    //    bindings::ActionFlags_ACTION_FIRE
    //}
}
