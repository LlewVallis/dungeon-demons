import Context from "./context";
import Program from "./program";
import Mesh from "./mesh";
import Mat3 from "../math/mat3";
import Vec2 from "../math/vec2";
import DrawBuffer from "./drawBuffer";
import Texture from "./texture";
import Vec3 from "../math/vec3";
import { all } from "../util";

function loadTexture(): Promise<HTMLImageElement> {
  return new Promise((resolve) => {
    const image = new Image();
    image.onload = () => resolve(image);
    image.src = require("/assets/sprites.png");
  });
}

const textureImage = await loadTexture();

class State {
  readonly context = new Context();

  readonly entityShader = new Program(
    this.context,
    require("./shaders/entity.vert"),
    require("./shaders/entity.frag")
  );

  readonly vignetteShader = new Program(
    this.context,
    require("./shaders/vignette.vert"),
    require("./shaders/vignette.frag")
  );

  readonly quad = Mesh.createQuad(this.context, 0);

  readonly texture = new Texture(this.context, textureImage);

  dispose(): void {
    all(
      () => this.texture.dispose(),
      () => this.quad.dispose(),
      () => this.entityShader.dispose(),
      () => this.vignetteShader.dispose()
    );
  }
}

export default class Graphics {
  private state = new State();

  constructor() {
    this.canvas.addEventListener("webglcontextrestored", this.restoredListener);
  }

  private readonly restoredListener = () => {
    console.info("Resetting graphics");
    this.state = new State();
  };

  get aspectRatio(): number {
    const size = this.state.context.size;
    return size.x / size.y;
  }

  get canvas(): HTMLCanvasElement {
    return this.state.context.canvas;
  }

  isBroken(): boolean {
    return this.state.context.gl.isContextLost();
  }

  draw(buffer: DrawBuffer): void {
    const view = buffer.readMatrix();
    const vignetteColor = buffer.readVec3();
    const vignetteScale = buffer.readFloat();

    this.state.context.resize();

    this.prepareViewport();

    this.state.quad.bind();

    this.drawEntities(buffer, view);
    this.drawVignette(vignetteColor, vignetteScale);
  }

  private drawEntities(buffer: DrawBuffer, view: Mat3): void {
    const state = this.state;
    const gl = state.context.gl;

    state.entityShader.bind();
    state.texture.bind(state.entityShader.getUniformLocation("sampler"));

    this.bindViewMatrix(view);

    const transformLocation =
      state.entityShader.getUniformLocation("transform");
    const textureTransformLocation =
      state.entityShader.getUniformLocation("textureTransform");

    for (let i = 0; i < buffer.size; i++) {
      Program.uniformMat3(
        state.context,
        transformLocation,
        buffer.readMatrix()
      );

      Program.uniformMat3(
        state.context,
        textureTransformLocation,
        buffer.readMatrix()
      );

      gl.drawArrays(gl.TRIANGLES, 0, 6);
    }
  }

  private drawVignette(color: Vec3, scale: number): void {
    const state = this.state;
    const gl = state.context.gl;

    state.vignetteShader.bind();

    const colorLocation =
      state.vignetteShader.getUniformLocation("vignetteColor");
    Program.uniformVec3(state.context, colorLocation, color);
    const scaleLocation =
      state.vignetteShader.getUniformLocation("vignetteScale");
    Program.uniformFloat(state.context, scaleLocation, scale);

    gl.drawArrays(gl.TRIANGLES, 0, 6);
  }

  private prepareViewport(): void {
    const gl = this.state.context.gl;
    gl.viewport(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
  }

  private bindViewMatrix(base: Mat3): void {
    const screenSize = this.state.context.size;
    const screenScaling = new Vec2(screenSize.y / screenSize.x, 1);
    const screenTransform = Mat3.scale(screenScaling);

    Program.uniformMat3(
      this.state.context,
      this.state.entityShader.getUniformLocation("view"),
      Mat3.mul(screenTransform, base)
    );
  }

  dispose() {
    all(
      () => this.state.dispose(),
      () =>
        this.canvas.removeEventListener(
          "webglcontextrestored",
          this.restoredListener
        )
    );
  }
}
