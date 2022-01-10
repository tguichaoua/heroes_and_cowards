# heroes_and_cowards

A simulation of heroes and cowards made in Rust with the game engine [`Bevy`](https://bevyengine.org/).

## 👾 The simulation

The simulation is a multi-agent system with two kind of agent: the heroes and the cowards.
Each agent choose two other agents, one friend and one foe.
Agent moves based on their kind:
- Heroes try to protected their friend from their foe.
- Coward try to flee from their foe, behind their friend.

## 🔧 The parameters

The following parameters can be changed: 
- the number of agent
- the proportion of heroes
- the size of the arena
- the view range of the agents
- the behaviour of the agents when they didn't see neither their friend nor their foe
