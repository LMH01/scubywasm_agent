#ifndef SCUBYWASM_AGENT_H
#define SCUBYWASM_AGENT_H
 
#include <stdint.h>
 
#ifdef __cplusplus
extern "C"
{
#endif
 
enum ActionFlags : unsigned int
{
    ACTION_NONE = 0U,
 
    ACTION_THRUST = 1U,
 
    ACTION_TURN_LEFT = 2U,
 
    ACTION_TURN_RIGHT = 4U,
 
    ACTION_FIRE = 8U,
};
 
enum ConfigParameter : unsigned int
{
    CFG_SHIP_MAX_TURN_RATE = 0U,
 
    CFG_SHIP_MAX_VELOCITY = 1U,
 
    CFG_SHIP_HIT_RADIUS = 2U,
 
    CFG_SHOT_VELOCITY = 3U,
 
    CFG_SHOT_LIFETIME = 4U,
};
 
struct Context;
 
struct Context *
init_agent(uint32_t n_agents, uint32_t agent_multiplicity, uint32_t seed);
 
void free_context(struct Context *ctx);
 
void set_config_parameter(struct Context *ctx,
                          enum ConfigParameter param,
                          float value);
 
void clear_world_state(struct Context *ctx);
 
void update_ship(struct Context *ctx,
                 uint32_t agent_id,
                 int32_t hp,
                 float x,
                 float y,
                 float heading);
 
void update_shot(struct Context *ctx,
                 uint32_t agent_id,
                 int32_t lifetime,
                 float x,
                 float y,
                 float heading);
 
void update_score(struct Context *ctx, uint32_t agent_id, int32_t score);
 
uint32_t make_action(struct Context *ctx, uint32_t agent_id, uint32_t tick);
 
#ifdef __cplusplus
}
#endif
#endif // SCUBYWASM_AGENT_H
