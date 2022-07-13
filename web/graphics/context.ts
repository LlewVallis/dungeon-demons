import Vec2 from "../math/vec2";

export default class Context {
  readonly gl: WebGL2RenderingContext;
  readonly canvas: HTMLCanvasElement;

  constructor() {
    this.canvas = document.getElementById("game-canvas") as HTMLCanvasElement;

    this.gl = this.canvas.getContext("webgl2", {
      antialias: false,
      powerPreference: "high-performance",
    })!!;

    const gl = this.gl;

    gl.clearColor(56 / 255, 39 / 255, 45 / 255, 1);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    gl.enable(gl.BLEND);

    if (gl.isContextLost()) {
      console.error("Graphics initialized with lost context");
    }
  }

  resize() {
    const canvas = this.canvas;

    const rect = canvas.getBoundingClientRect();

    const dpr = window.devicePixelRatio;
    let width = Math.round(rect.width * dpr);
    const height = Math.round(rect.height * dpr);

    const windowWidth = Math.round(window.innerWidth * dpr);

    if (windowWidth < width) {
      width = windowWidth;
    }

    if (canvas.width !== width || canvas.height !== height) {
      canvas.width = width;
      canvas.height = height;
    }
  }

  get size(): Vec2 {
    return new Vec2(this.canvas.width, this.canvas.height);
  }
}
