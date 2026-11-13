# Music Discs

Adds the nine C418 music discs that Beta 1.9 Prerelease 2 introduced: 11, blocks, chirp, far, mall, mellohi, stal,
strad and ward. Each plays in a jukebox and shows "Now playing: C418 - <name>".

## Getting them

When a skeleton's arrow kills a creeper it drops one disc picked evenly from all eleven, 13 and cat included, as the
creeper does from Minecraft 1.0 on. Beta 1.9 Prerelease 2 itself only had them in creative mode. Gunpowder drops as
in beta.

## Blank discs

Craft a blank disc from four coal around a flint. Right click it to open the burn screen, paste a link with Ctrl+V and
press Burn. The server downloads the song, compresses it to Ogg Vorbis and turns the first blank disc in your inventory
into a Music Disc that plays that song in any jukebox. Put a burned disc back in the crafting grid to wipe it blank.

- Paste any song link or type a song name. YouTube links are burned as they are. Any other link (Spotify, Apple Music,
  SoundCloud and so on) is turned into its song name, and names are searched on YouTube, so every song comes from
  YouTube's stereo AAC track and burns the same way on every system.
- Nothing to install on Windows or Linux. The machine that owns the world (the server, or your own game in single
  player) uses `yt-dlp` from its path when present, otherwise it downloads the official yt-dlp build once, checks it
  against the release checksums, keeps it in the game cache and refreshes it weekly. The game decodes and compresses
  the audio itself, so no ffmpeg is needed. A download YouTube refuses is tried again up to three times.
- Windows and Linux players share servers freely: songs travel as the same Ogg file and play from the same moment.
- Limits: one disc burns at a time. Songs over 10 minutes or 20 MB are refused; change `burn.max_minutes` and
  `burn.max_megabytes` in the mod config.
- Songs are kept in the world's `media` folder. Clients download a song from the server the first time they hear it and
  keep it in the game cache. Everyone near the jukebox starts at the same moment, players who were still downloading
  join in where the song has got to.


- Music: the game's own `resources/streaming` folder already holds `11.mus`, `blocks.mus` and the rest, and the discs
  play those. Nothing to install.
- Art: Mojang's disc icons are not shipped. Put the Minecraft 1.0 client jar, named `1.0.jar`, in the game's
  `mods/discs_assets/` folder. The client cuts the nine icons out of its `gui/items.png` on the next start.

## Building

```
discs/build.sh
```
