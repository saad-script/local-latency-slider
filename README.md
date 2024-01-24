I took blue-dev's code for the [original arena latency slider mod](https://github.com/blu-dev/arena-latency-slider) and modified it so that it only works with local online. I also added in options to change the game's framerate to get even lower input delay. You can see the readme on the original repo to get details as to how exactly the mod changes the online latency.

## Installation

1) Make sure you have the latest [skyline](https://github.com/skyline-dev/skyline/releases) installed on yuzu.
2) For skyline to work with LDN, download [this](https://drive.google.com/file/d/1f_idi29L7Poxg0Cljbi4oz9ubpukmdXY/view) and replace the `subsdk9` file in
```
%yuzu_folder%/sdmc/atmosphere/contents/01006A800016E000/exefs/subsdk9
```
3) Download the latest `liblocal_latency_slider.nro` from the [releases](https://github.com/saad-script/local-latency-slider/releases) page.
Place it in the corresponding folder. This is how it should look:
```
%yuzu_folder%/sdmc/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/liblocal_latency_slider.nro
```

## Usage

- This mod is intended for use with the [yuzu](https://yuzu-emu.org/) emulator. Some functionality like the framerate options may not work on a real switch or other emulators.
- This mod will only work in local online modes. It is intended to be used with yuzu's LDN capabilities to provide an online experience that is very close to the feel of playing offline (and sometimes even identical to offline!)
- Make sure that yuzu's speed limit is set to 100%. Make sure the previous 120 fps cheat is disabled/removed.
- Due to a current yuzu bug, make sure to connect to LDN **after** launching smash. Otherwise yuzu will crash when loading the game.
- In the character select screen, you can do any of the following:
  - Use DPAD left/right to change the online delay set by the game.
  - Use DPAD up/down to change the target framerate. 120 fps removes 3f of delay, and 240 fps removes 4.5 frames of delay.
  - Press X to toggle VSYNC on/off (only available when playing on 60 fps). This is useful if you dont have a PC capable of reaching higher framerates like 120 fps, but still want to enjoy the lower delay. This can remove upto 3 frames of delay.

## Troubleshooting & FAQ

- "I can't enter the local online menu"
  - Make sure skyline is installed to the correct location and also that you installed the LDN fix in step 2 of the installation instructions.
- "Yuzu crashes when launching the game"
  - Make sure to connect to LDN **after** launching smash. If the issue still persists you may have to delete your shaders.
- "I set the online delay to a low value but the game still feels very slow and stuttery"
  - Setting the delay too low will cause that. Set the delay based on your proximity to your opponent. Also, both of you should be wired to ethernet on a good network/IPS. You should never set the delay lower than 2f, as that has never worked even on computers wired directly to the same network.
- "The game seems to speeds up randomly"
  - Make sure your computer is actually capable of reaching the target framerate. If you have the target framerate set to 120 fps, your PC should be able to maintain a solid 120 fps in game. Also make sure that yuzu's speed limit is set to 100% and that you are not using the previous 120 fps cheat.
- "There are random stutters that occur in game"
  - Shader stutters will happen the first time you do a move/action in yuzu. Yuzu will compile and store these shaders so the next time you do the move, it won't stutter. The more you play the less these stutter will happen. It is recommended that you dedicate a game where you and your opponent will hit each other with all of your moves, so in subsequent games these stutters won't occur.

Check out the [NA](https://discord.gg/jE9hTsmbjD) or [EU](https://discord.gg/yuzu-smash-meet-up-1051577844318339172) yuzu smash discord for more information and also matchmaking!
