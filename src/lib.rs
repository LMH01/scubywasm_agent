use std::{cmp::Ordering, collections::HashSet, os::raw::c_char};

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
    /// Agent ids of ships that are in this team.
    own_agent_ids: HashSet<u32>,
}

#[unsafe(no_mangle)]
pub extern "C" fn init_agent(n_agents: u32, agent_multiplicity: u32, seed: u32) -> Box<Context> {
    //null::<Context>() as *mut Context
    let context = Context {
        config: Config::default(),
        seed,
        world_state: WorldState::default(),
        own_ships_to_action: Vec::new(),
        own_agent_ids: HashSet::new(),
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
    ships: Vec<Ship>,
    shots: Vec<Shot>,
    agents: Vec<Agent>,
}

#[derive(Default, Clone)]
struct Ship {
    hp: i32,
    pos_x: f32,
    pos_y: f32,
    heading: f32,
    friendly: bool,
}

#[derive(Default, Clone)]
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
    let mut ship = Ship {
        hp,
        pos_x,
        pos_y,
        heading: (90.0 - heading).to_radians(),
        friendly: false,
    };
    if ctx.own_agent_ids.contains(&agent_id) {
        ship.friendly = true;
        ctx.own_ships_to_action.push(ship.clone());
    }
    ctx.world_state.ships.push(ship);
}

#[unsafe(no_mangle)]
pub extern "C" fn update_shot(
    ctx: &mut Context,
    _agent_id: u32,
    lifetime: i32,
    pos_x: f32,
    pos_y: f32,
    heading: f32,
) {
    let shot = Shot {
        lifetime,
        pos_x,
        pos_y,
        heading: (90.0 - heading).to_radians(),
    };
    ctx.world_state.shots.push(shot)
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
    // add this agent id to own agents, is used on first make_action calls to let ctx know
    // what agents are controlled by this team
    ctx.own_agent_ids.insert(own_agent_id);

    let world_state = &ctx.world_state;
    let mut action = Action::default();
    let current_ship_to_action = match ctx.own_ships_to_action.pop() {
        Some(ship) => ship,
        None => {
            // no ship found for which an action could be calculated, so we do nothing
            // should only be run on first action because own ships are not yet initialized
            log!("Agent {own_agent_id}: did not find own ships");
            return bindings::ActionFlags_ACTION_NONE;
        }
    };

    // shot evasion logic

    // stores shots with the distance it is away from the ship
    let mut shots: Vec<(f32, Shot)> = Vec::new();
    for shot in &world_state.shots {
        // calculate distance between shot and ship
        let x1 = shot.pos_x;
        let y1 = shot.pos_y;
        let x2 = current_ship_to_action.pos_x;
        let y2 = current_ship_to_action.pos_y;
        let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt().abs();
        shots.push((distance, shot.clone()));
    }
    shots.sort_by(|a, b| match a.0.partial_cmp(&b.0) {
        Some(order) => order,
        None => Ordering::Equal,
    });
    // shot with lowest distance to ship is popped first
    shots.reverse();

    // iterate through shots and calculate if they would hit
    // first shot that is determined to hit the ship will be tried to be evaded
    for (distance, shot) in shots {
        // calculate if shot is in hit radius

        let x1 = current_ship_to_action.pos_x;
        let y1 = current_ship_to_action.pos_y;
        let x2 = shot.pos_x;
        let y2 = shot.pos_y;

        // target = my ship
        let target_angle = (y1 - y2).atan2(x1 - x2);
        // current angle between the shot and the ship
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

        // TODO this needs some work
        if angle_diff.abs() <= -179.0 || angle_diff.abs() >= 179.0 {
            // shot is probably not a danger for the ship
            continue;
        }

        // calculate if shot is in hit radius
        let lateral_distance_target = distance * angle_diff.tan().abs();
        let hit_radius = ctx.config.ship_hit_radius;

        // shot would hit ship
        if lateral_distance_target <= hit_radius {
            log!(
                "Agent: {own_agent_id}: evading shot {}/{}, angle_diff {}",
                shot.pos_x,
                shot.pos_y,
                angle_diff.to_degrees()
            );
            return bindings::ActionFlags_ACTION_NONE;
        }
    }

    // acquire target
    let mut target: Option<(f32, Ship)> = None;
    for ship in &world_state.ships {
        if ship.friendly {
            // we don't want to lock an allied ship as target
            continue;
        }
        // ship is enemy, so we can lock on to it
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
    let (distance, target) = match target {
        Some(target) => target,
        None => {
            // no target found, so game *should* be won already
            log!("Agent: {own_agent_id} - no target found");
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

    // check if we are locked on target, if yes, fire shot

    // calculate if shot is in hit radius
    let lateral_distance_target = distance * angle_diff.tan().abs();
    let hit_radius = ctx.config.ship_hit_radius;

    // fire if shot would it if target does not move
    if lateral_distance_target <= hit_radius {
        action.fire = true;
    }

    log!(
        "Agent: {own_agent_id}, Current position: [{},{}], Target direction: {}, Current direction: {}, Target in scope: {}",
        current_ship_to_action.pos_x,
        current_ship_to_action.pos_y,
        90.0 - target_angle.to_degrees(),
        90.0 - current_angle.to_degrees(),
        lateral_distance_target <= hit_radius
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
