# Single Player Commands

Modern, minimal take on simo_415's Single Player Commands (b1.7.3 v2.11.1). Not a 1:1 port: only the core commands,
built on the loader's command API. Every command needs op, which singleplayer always has. Each also answers to
`/spc:<name>` when another mod owns the short name.

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

Fly and god persist per player (`player.setData`). Flight state reaches the client on the `fly` channel; client movement
runs in `TickPost`, the server clears fall distance.

## Status

- [x] commands, abilities, build and clippy clean
- [ ] live check on the test server (blocked: shared test server busy)
