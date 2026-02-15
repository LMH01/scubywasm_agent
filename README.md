# Scubywasm agent

## TODOs

- [ ] update target selection to not reset by each turn. Ideally the agent saves the currently assigned target per ship
- [X] `HashMap<u33, Vec<Ship>>` does not work as expected, one agent id is assigned to exactly one shot and one ship. So I have to change the logic that stores if a ship is in my team. For that I could add a Vec to the state that stored agent ids of agents that this team control.
- [X] find out why movement does not work correctly. ships should move thorward the acquired target (they don't they move erratic)
- [X] find out how to use debug logging
