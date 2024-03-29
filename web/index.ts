import "./audio";

// Import order is important
import * as Wasm from "../pkg";

import Graphics from "./graphics";
import DrawBuffer from "./graphics/drawBuffer";
import Vec2 from "./math/vec2";
import { all } from "./util";
import isMobileDetected from "./detect-mobile";

const TAP_TIME = 150;
const MAX_TICK_DELTA = 0.05;
const MAX_TICKS_PER_FRAME = 2;
const PERF_REPORT_INTERVAL = 5;

const joystickElement = document.getElementById("joystick")!!;
const welcomeElement = document.getElementById("welcome")!!;
const playInstructionsElement = document.getElementById("play-instructions")!!;
const hudElement = document.getElementById("hud")!!;
const transitionScreenElement = document.getElementById("transition-screen")!!;
const roundElement = document.getElementById("round-display")!!;
const creditsElement = document.getElementById("credits-display")!!;
const interactHeadingElement = document.getElementById("interact-heading")!!;
const interactCaptionElement = document.getElementById("interact-caption")!!;
const ammoElement = document.getElementById("ammo-display")!!;

const isMobile = isMobileDetected();

const joystickImage = require("/assets/joystick.png");
if (isMobile) {
  joystickElement.style.backgroundImage = `url(${joystickImage})`;
  joystickElement.style.display = "";
  playInstructionsElement.innerText = "Use the joystick to play";
}

const startKeys = [
  "KeyW",
  "KeyA",
  "KeyS",
  "KeyD",
  "ArrowUp",
  "ArrowLeft",
  "ArrowRight",
  "ArrowDown",
];

const startKeyListener = (event: KeyboardEvent) => {
  if (!startKeys.includes(event.code)) {
    return;
  }

  startPlaying();
};

const startTouchListener = (event: TouchEvent) => {
  let found = false;
  for (const touch of event.touches) {
    if (isJoystickTouch(touch)) {
      found = true;
      break;
    }
  }

  if (!found) {
    return;
  }

  startPlaying();
};

function isJoystickTouch(touch: Touch): boolean {
  return touch.target === joystickElement;
}

function startPlaying() {
  removeStartListeners();
  welcomeElement.style.opacity = "0";
  hudElement.style.display = "";
  game.play();
}

function addStartListeners() {
  window.addEventListener("keydown", startKeyListener);
  window.addEventListener("touchstart", startTouchListener);
}

function removeStartListeners() {
  window.removeEventListener("keydown", startKeyListener);
  window.removeEventListener("touchstart", startTouchListener);
}

addStartListeners();

window.addEventListener("contextmenu", (e) => e.preventDefault());

window.addEventListener("touchstart", (e) => e.preventDefault(), {
  passive: false,
});
window.addEventListener("touchend", (e) => e.preventDefault(), {
  passive: false,
});
window.addEventListener("touchmove", (e) => e.preventDefault(), {
  passive: false,
});
window.addEventListener("touchcancel", (e) => e.preventDefault(), {
  passive: false,
});

let game: Game;

class Game {
  private readonly backend;

  private readonly graphics = new Graphics();

  private playing = false;

  private playTime: number | null = null;
  private lastTick = performance.now();

  private frameCounter = 0;
  private totalFrameTime = 0;
  private tickCounter = 0;
  private totalTickTime = 0;
  private maxLoopTime = 0;
  private cycles = 0;

  private loopTask: number = null as any;
  private readonly performanceReportTask: number;

  private mouseX = 0;
  private mouseY = 0;

  private joystickTouchStart: number | null = null;

  private transitioning = false;

  constructor() {
    const seed = Math.floor(Math.random() * Math.pow(2, 32));
    console.info(`Using seed ${seed}`);

    this.backend = new Wasm.Backend(seed, isMobile);

    this.addInputListeners();
    window.addEventListener("beforeunload", this.navigateListener);
    this.performanceReportTask = this.addPerformanceReporter();

    this.loop();
  }

  play(): void {
    this.playTime = performance.now();
    this.backend.enableHud();
    this.playing = true;
  }

  private readonly navigateListener = (event: Event) => {
    const playing =
      this.playTime !== null && performance.now() > this.playTime + 30_000;
    if (playing && !this.backend.isOver()) {
      event.preventDefault();
    }
  };

  private readonly mouseMoveListener = (event: MouseEvent) => {
    this.mouseX = event.clientX;
    this.mouseY = event.clientY;
  };

  private readonly mouseDownListener = (event: MouseEvent) => {
    if (event.button != 0) {
      return;
    }

    const mouse = this.mapMouseCoordinates(event.clientX, event.clientY);
    this.backend.mouseDown(mouse.x, mouse.y);
  };

  private readonly mouseUpListener = (event: MouseEvent) => {
    if (event.button != 0) {
      return;
    }

    const mouse = this.mapMouseCoordinates(event.clientX, event.clientY);
    this.backend.mouseUp(mouse.x, mouse.y);
  };

  private readonly keyDownListener = (event: KeyboardEvent) => {
    if (!event.repeat) {
      this.backend.keyDown(event.code);
    }
  };

  private readonly keyUpListener = (event: KeyboardEvent) => {
    this.backend.keyUp(event.code);
  };

  private readonly touchListener = (event: TouchEvent) => {
    if (event.type === "touchstart") {
      for (const touch of event.changedTouches) {
        if (!isJoystickTouch(touch)) {
          const mouse = this.mapMouseCoordinates(touch.clientX, touch.clientY);
          this.backend.mouseDown(mouse.x, mouse.y);
        } else {
          this.joystickTouchStart = performance.now();
        }
      }
    }

    if (event.type === "touchend" || event.type === "touchcancel") {
      for (const touch of event.changedTouches) {
        if (!isJoystickTouch(touch)) {
          const mouse = this.mapMouseCoordinates(touch.clientX, touch.clientY);
          this.backend.mouseUp(mouse.x, mouse.y);
        } else {
          if (this.joystickTouchStart !== null) {
            console.log(performance.now() - this.joystickTouchStart);
            if (performance.now() - this.joystickTouchStart <= TAP_TIME) {
              this.backend.joystickTap();
            }
          }

          this.joystickTouchStart = null;
        }
      }
    }

    this.backend.updateJoystick(0, 0);

    for (const touch of event.touches) {
      if (isJoystickTouch(touch)) {
        const offset = this.joystickOffset(touch);
        this.backend.updateJoystick(offset.x, offset.y);
      } else {
        this.mouseX = touch.clientX;
        this.mouseY = touch.clientY;
      }
    }
  };

  private joystickOffset(touch: Touch): Vec2 {
    const rect = joystickElement.getBoundingClientRect();
    const centerX = rect.x + rect.width / 2;
    const centerY = rect.y + rect.height / 2;

    const x = ((touch.clientX - centerX) / rect.width) * 2;
    const y = ((centerY - touch.clientY) / rect.height) * 2;

    const offset = new Vec2(x, y);

    if (offset.length() > 1) {
      return offset.normalize();
    } else {
      return offset;
    }
  }

  private mapMouseCoordinates(x: number, y: number): Vec2 {
    const canvas = this.graphics.canvas;
    const rect = canvas.getBoundingClientRect();

    const mappedX = ((x - rect.x) / rect.width) * 2.0 - 1.0;
    const mappedY = 1.0 - ((y - rect.y) / rect.height) * 2.0;
    const clampedX = Math.min(Math.max(mappedX, -1.0), 1.0);
    const clampedY = Math.min(Math.max(mappedY, -1.0), 1.0);

    return new Vec2(clampedX * this.graphics.aspectRatio, clampedY);
  }

  private addInputListeners(): void {
    document.addEventListener("mousemove", this.mouseMoveListener);
    document.addEventListener("mousedown", this.mouseDownListener);
    document.addEventListener("mouseup", this.mouseUpListener);
    document.addEventListener("keydown", this.keyDownListener);
    document.addEventListener("keyup", this.keyUpListener);
    document.addEventListener("touchstart", this.touchListener);
    document.addEventListener("touchend", this.touchListener);
    document.addEventListener("touchmove", this.touchListener);
    document.addEventListener("touchcancel", this.touchListener);
  }

  private removeInputListeners(): void {
    document.removeEventListener("mousemove", this.mouseMoveListener);
    document.removeEventListener("mousedown", this.mouseDownListener);
    document.removeEventListener("mouseup", this.mouseUpListener);
    document.removeEventListener("keydown", this.keyDownListener);
    document.removeEventListener("keyup", this.keyUpListener);
    document.removeEventListener("touchstart", this.touchListener);
    document.removeEventListener("touchend", this.touchListener);
    document.removeEventListener("touchmove", this.touchListener);
    document.removeEventListener("touchcancel", this.touchListener);
  }

  private addPerformanceReporter(): number {
    return setInterval(() => {
      const fps = (this.frameCounter / PERF_REPORT_INTERVAL).toFixed(1);
      const tps = (this.tickCounter / PERF_REPORT_INTERVAL).toFixed(1);

      const mspf = (this.totalFrameTime / this.frameCounter).toFixed(2);
      const mspt = (this.totalTickTime / this.tickCounter).toFixed(2);

      const mlt = this.maxLoopTime.toFixed(2);

      const entities = this.backend.entityCount();

      console.debug(
        `FPS: ${fps}, TPS: ${tps}, MSPF: ${mspf}, MSPT: ${mspt}, Max loop time: ${mlt}, Entities: ${entities}`
      );

      this.frameCounter = 0;
      this.totalFrameTime = 0;
      this.tickCounter = 0;
      this.totalTickTime = 0;
      this.maxLoopTime = 0;
    }, PERF_REPORT_INTERVAL * 1000) as unknown as number;
  }

  private loop(): void {
    this.loopTask = requestAnimationFrame(() => this.loop());

    try {
      const start = performance.now();

      const mouse = this.mapMouseCoordinates(this.mouseX, this.mouseY);
      this.backend.updateMouse(mouse.x, mouse.y);

      const delta = (start - this.lastTick) / 1000;
      const ticks = Math.ceil(delta / MAX_TICK_DELTA);

      const computedTicks = Math.min(ticks, MAX_TICKS_PER_FRAME);
      for (let i = 0; i < computedTicks; i++) {
        const tickDelta = Math.min(delta / computedTicks, MAX_TICK_DELTA);
        this.tick(tickDelta);
      }

      this.lastTick = start;
      this.draw();

      this.maxLoopTime = Math.max(this.maxLoopTime, performance.now() - start);

      if (this.cycles === 0) {
        const loadTime = Math.round(performance.now());
        console.debug(`Loaded in ${loadTime}ms`);
      }

      if (this.backend.isOver()) {
        this.handleGameOver();
      }
    } catch (error) {
      console.error("Exiting due to unhandled exception", error);
      this.dispose();
    }

    this.cycles++;
  }

  private handleGameOver(): void {
    if (this.transitioning) {
      return;
    }

    this.transitioning = true;

    transitionScreenElement.style.opacity = "1";

    setTimeout(() => {
      this.dispose();
      game = new Game();
      addStartListeners();
      transitionScreenElement.style.opacity = "0";
      hudElement.style.display = "none";
      welcomeElement.style.opacity = "";
    }, 5000);
  }

  private tick(delta: number): void {
    let start = performance.now();

    if (!document.hidden && this.playing) {
      this.backend.tick(delta);
    }

    this.tickCounter += 1;
    this.totalTickTime += performance.now() - start;
  }

  private draw(): void {
    let start = performance.now();

    this.backend.draw((ptr: number) => {
      const buffer = new DrawBuffer(ptr);

      try {
        this.graphics.draw(buffer);
      } catch (error) {
        console.error("Error displaying graphics", error);
      }
    }, this.graphics.aspectRatio);

    this.drawHud();

    this.frameCounter += 1;
    this.totalFrameTime += performance.now() - start;
  }

  private drawHud(): void {
    const round = this.backend.round() + 1;
    roundElement.innerText = `Round ${round}`;

    const credits = this.backend.credits();
    creditsElement.innerText = `Credits $${credits}`;

    const interactHeading = JSON.parse(this.backend.interactionLine());
    const interactCaption = JSON.parse(this.backend.interactionCaption());
    this.writeUiText(interactHeading, interactHeadingElement);
    this.writeUiText(interactCaption, interactCaptionElement);

    const currentAmmo = this.backend.currentAmmo();
    const maxAmmo = this.backend.maxAmmo();
    ammoElement.innerText = `${currentAmmo}|${maxAmmo}`;
  }

  private writeUiText(uiText: [string, string][], element: HTMLElement): void {
    element.innerHTML = "";

    for (const [text, color] of uiText) {
      const span = document.createElement("span");
      span.innerText = text;
      span.style.color = color;
      element.appendChild(span);
    }
  }

  private cancelLoop(): void {
    if (this.loopTask !== null) {
      cancelAnimationFrame(this.loopTask);
    }
  }

  private cancelPerformanceReporter(): void {
    if (this.performanceReportTask !== null) {
      clearInterval(this.performanceReportTask);
    }
  }

  dispose(): void {
    all(
      () => this.cancelLoop(),
      () => this.removeInputListeners(),
      () => window.removeEventListener("beforeunload", this.navigateListener),
      () => this.cancelPerformanceReporter(),
      () => this.backend.free(),
      () => this.graphics.dispose()
    );
  }
}

Wasm.start();
game = new Game();
