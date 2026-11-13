# Music Discs

Adds the nine C418 music discs that Beta 1.9 Prerelease 2 introduced: 11, blocks, chirp, far, mall, mellohi, stal,
strad and ward. Each plays in a jukebox and shows "Now playing: C418 - <name>".

## Getting them

When a skeleton's arrow kills a creeper it drops one disc picked evenly from all eleven, 13 and cat included, as the
creeper does from Minecraft 1.0 on. Beta 1.9 Prerelease 2 itself only had them in creative mode. Gunpowder drops as
in beta.

## Assets

- Music: the game's own `resources/streaming` folder already holds `11.mus`, `blocks.mus` and the rest, and the discs
  play those. Nothing to install.
- Art: Mojang's disc icons are not shipped. Put the Minecraft 1.0 client jar, named `1.0.jar`, in the game's
  `mods/discs_assets/` folder. The client cuts the nine icons out of its `gui/items.png` on the next start.

## Building

```
discs/build.sh
```
