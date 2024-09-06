I took blue-dev's code for the original arena latency slider mod and modified it so that it only works with local online. I also added in options to change the game's framerate to get even lower input delay. You can see the [original readme](./ORIGINAL_README.md) to get details as to how exactly the mod changes the online latency.

## Installation

1) Make sure you have the latest [skyline](https://github.com/skyline-dev/skyline/releases) installed on yuzu:
```
%yuzu_folder%/sdmc/atmosphere/contents/01006A800016E000/exefs/
                                                          ├── subsdk9 
                                                          └── main.npdm
```
2) Download the latest `liblocal_latency_slider.nro` from the [releases](https://github.com/saad-script/local-latency-slider/releases) page.
Place it in the corresponding folder. This is how it should look:
```
%yuzu_folder%/sdmc/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/liblocal_latency_slider.nro
```

## Usage

- This mod is intended for use with the yuzu emulator. Some functionality like the framerate options may not work on a real switch or other emulators.
- This mod will only work in local online modes. It is intended to be used with yuzu's LDN capabilities to provide an online experience that is very close to the feel of playing offline (and sometimes even identical to offline!)
- Make sure that yuzu's speed limit is set to 100%. Make sure the previous 120 fps cheat is disabled/removed.
- Due to a current yuzu bug, make sure to connect to LDN **after** launching smash. Otherwise yuzu will crash when loading the game.
- As a QOL feature, new smash LDN rooms will be set to have 2 players by default. (You can increase/decrease this yourself)
- The top of the character select screen will display your selected framerate and delay. It will also show the ping of the room (determined by the average ping of all opponent players). Red = Unstable Connection, Yellow = Playable Connection, Green = Stable Connection.
- Under each player you can see their selected framerate and delay. You can also see the ping of each individual players (see bullet point below).
- In the character select screen, you can do any of the following:
  - Use DPAD left/right to change the online delay set by the game.
  - Use DPAD up/down to change the target framerate. 120 fps removes 3f of delay, 180 fps removes 4 frames of delay, and 240 fps removes 4.5 frames of delay.
  - Press X to toggle VSYNC on/off (only available when playing on 60 fps). This is useful if you dont have a PC capable of reaching higher framerates like 120 fps, but still want to enjoy the lower delay. This can remove upto 3 frames of delay.
  - Press the right stick to view the individual pings of opponent players. (Displayed under each connected player)

## Troubleshooting & FAQ

- "I can't enter the local online menu"
  - Ensure that you have the latest skyline is installed. Previous versions had a bug that did not allow you to enter to local wireless menu.
- "Yuzu crashes when launching the game"
  - Make sure to connect to LDN **after** launching smash. If the issue still persists you may have to delete your shaders.
- "I set the online delay to a low value but the game still feels very slow and stuttery"
  - Setting the delay too low will cause that. Set the delay based on your proximity to your opponent. Also, both of you should be wired to ethernet on a good network/IPS. You should never set the delay lower than 2f, as that has never worked even on computers wired directly to the same network.
- "The game seems to speeds up randomly"
  - Make sure your computer is actually capable of reaching the target framerate. If you have the target framerate set to 120 fps, your PC should be able to maintain a solid 120 fps in game. Also make sure that yuzu's speed limit is set to 100% and that you are not using the previous 120 fps cheat.
- "There are random stutters that occur in game"
  - Shader stutters will happen the first time you do a move/action in yuzu. Yuzu will compile and store these shaders so the next time you do the move, it won't stutter. The more you play the less these stutter will happen. It is recommended that you dedicate a game where you and your opponent will hit each other with all of your moves, so in subsequent games these stutters won't occur.

Check out the [NA](https://discord.gg/jE9hTsmbjD) or [EU](https://discord.gg/yuzu-smash-meet-up-1051577844318339172) yuzu smash discord for more information and also matchmaking!


## Known Issues
- If a players yuzu crashes, or disconnects unexpectedly, the room ping and opponent info (framerate, delay, ping) may not show the correct values or may show under the wrong person. To fix this, the room host will have to exit the room by backing out into the local online menu and create a room again.
- If 2 or more players are playing on the same yuzu, the opponent info (framerate, delay, ping) may show under the wrong person or not show at all.
- If you notice any other issues, you can report them here or in the discord servers mentioned above.
- If you know how to solve these issues, contributions are always welcome.
