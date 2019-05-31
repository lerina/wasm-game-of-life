import { Universe } from "wasm-game-of-life";

// let's get that <pre> element we just added and instantiate a new universe
const pre = document.getElementById("game-of-life-canvas");
const universe = Universe.new();

// The JavaScript runs in a requestAnimationFrame loop. 
// On each iteration, it draws the current universe to the <pre>, 
// and then calls Universe::tick
const renderLoop = () => {
  pre.textContent = universe.render();
  universe.tick();

  requestAnimationFrame(renderLoop);
};

// To start the rendering process, all we have to do is make the initial call 
// for the first iteration of the rendering loop
requestAnimationFrame(renderLoop);

