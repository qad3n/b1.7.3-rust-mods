# Single Player Commands

Modern, minimal take on simo_415's Single Player Commands (b1.7.3 v2.11.1). Not a 1:1 port: only the core commands,
built on the loader's command API. Every command needs op, which singleplayer always has. Each also answers to
`/spc:<name>` when another mod owns the short name. `/give`, `/time` and `/tp` exist only in single player, a dedicated
server keeps its vanilla ones. With mods loaded, `T` opens chat in single player and slash lines run here.

| Command | Does |
|---|---|
| `/help` | lists the commands |
| `/give <item> [count] [damage]` | item by name (`diamond`, `minecraft:diamond`, `modid:item`) or numeric id, split into stacks |
| `/fly` | toggles flight: jump rises, sneak sinks, no fall damage |
| `/god` | toggles invulnerability (cancels every attack on the player) |
| `/heal` | full health, puts out fire |
| `/repair` | resets the held item's damage |
| `/clear` | empties the main inventory, armor stays |
| `/time <day\|noon\|night\|midnight\|ticks>` | moves forward to that time of day |
| `/tp <x> <y> <z>` | teleport, `~` for relative |
| `/summon <entity> [count]` | spawns at the player |
| `/locate <structure> [radius]` | nearest structure of a kind from the loader's structure registry (`dungeon`, `aether:silver_dungeon`, `twilightforest:lich_tower`), 64 chunks unless given |
| `/effect <effect> [level] [seconds]`, `/effect remove <effect>` | a mod status effect on the player such as `aether:inebriation`, level 1 for 30 seconds unless given (loader API 1.22) |

Fly and god persist per player (`player.setData`). Flight state reaches the client on the `fly` channel; client movement
runs in `TickPost`, the server clears fall distance.

## Status

- [x] commands, abilities, build and clippy clean
- [ ] live check on the test server (blocked: shared test server busy)
