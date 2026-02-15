#[derive(Default)]
pub struct Config {
    pub ship_max_turn_rate: f32,
    pub ship_max_velocity: f32,
    pub ship_hit_radius: f32,
    pub shot_velocity: f32,
    pub shot_lifetime: f32,
}

impl Config {
    pub fn update(&mut self, id: u32, value: f32) {
        match id {
            0 => self.ship_max_turn_rate = value,
            1 => self.ship_max_velocity = value,
            2 => self.ship_hit_radius = value,
            3 => self.shot_velocity = value,
            4 => self.shot_lifetime = value,
            _ => (),
        }
    }
}
