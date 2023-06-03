# Cells

Cells is a singleplayer game inspired by agar.io written in Rust. <br>
Every cell/entity can have a mix of differents capabilities. <br>

It's been optimized so that 1 million+ entities can be in game at the same time, but full FPS is not guaranteed with this amount of entities!

![img](https://github.com/psincf/Cells/assets/44228825/6078fc04-6b42-4922-95ad-e5917e2beefb)
![img2](https://github.com/psincf/Cells/assets/44228825/2980304f-172f-4269-bbe9-3f8038f2b7aa)

https://github.com/psincf/Cells/assets/44228825/b41b6b56-fcb8-4d3c-b5ec-6fbfacd86b7e

## Status
It's a demo/experiment. <br>
It is no more actively developed. <br>
Some hacky code here and there!

## How to run

```
git clone https://github.com/psincf/Cells
cd Cells
cargo run --release
```


## Features
* In-game Editor
* Multiples examples maps

## Controls
### Ingame
* Mouse: moving
* Space: split
* W: throw cell
* R: revive
* Right CTRL: Enter/Leave Debug Mode

### Debug Mode
* R: Revive/reset mass of local cells to minimum
* E: Local cells can merge
* Right click: Teleport
* Z: Augment Mass
* Up: add 1 computing thread
* Down: remove 1 computing thread 
* +: add cells speed
* -: remove cells speed
* \*: Multiply cells speed by 2
* /: Divide cells speed by 2

## Engine detail
* Multithreaded. Amount of threads can be changed in real time.
* Struct Of Arrays optimization
* Cells can have multiple different properties/capabilities
* Custom `threadpool` crate
* Custom multithreaded physics engine
* Custom reflection crate written with procedural macro ( `#[derive(Reflect)]` )
* Custom automatic inspection crate for reflected struct with both Imgui and Egui ( inside `reflect` crate). <br>
  Used for Editor mode.
* Custom profiling crate (`benchmark`). Real time profiling data can be seen in Debug mode
* Separate threads for Graphics and Update -> Synchronization through a `quintuple buffer` <br>
  The quintuple buffer allow the retrieval of the last 2 game updates on each frame. A timer mesuring the elapsed time between 2 game updates is constantly updating.
  Thanks to this, interpolation can be done and the game can be at full FPS even if the Update thread is lagging.
* Custom `Slab` implementation
* WASM scripting (a bit hacky and limited)
