# Satisfactory World Generator
program to simulate the random game mode's randomized node distribution introduced in 1.2

## Usage

**You can try it [HERE](https://konsl.github.io/satisfactory-world-generator/)**

![screenshot](screenshot.png)
Look in [Releases](https://github.com/Konsl/satisfactory-world-generator/releases/latest) to use the tool locally

## Inaccuracies

**NOTE**: if you created your world on build `480321` (first 1.2 exp build), this program's output will **not** match your world
(because that version was missing a limestone node, which was fixed in the following release)

if you notice that the output of this program does not match what you see in game, open an issue containing the following:

- game version: version code (`vx.x.x.x`), branch (`main` / `experimental`), **build number** (e.g. `481836`, should be visible somewhere in game, maybe title screen? if you dont know, at least provide the release date of the patch)
- version of this tool you used (git commit hash / git tag / version number / release date / download date)
- randomization settings (seed, mode, purity mode)
- a save file created with these parameters
- description of the mismatch between game and this tool (if not obvious)
- (preferrably) a screenshot of the generated node distribution

## Build Instructions - **ONLY FOR EXPERTS**

- Clone this repo
- You need to provide `src/default-world.json`, which describes the resources present in the default world and can be generated from the game assets using `scripts/`
- You need to provide `src/world-outline.json`, which describes the outline of the map and is used as a background graphic.
  This was created manually by masking + redrawing and then tracing the in-game map image. Just put an empty array.
- Run `cargo run` to start the tool
