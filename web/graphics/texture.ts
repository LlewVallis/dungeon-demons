import Context from "./context";
import Vec2 from "../math/vec2";

export default class Texture {
  private readonly gl: WebGL2RenderingContext;
  private readonly texture: WebGLTexture;

  private readonly size: Vec2;

  constructor(context: Context, image: HTMLImageElement) {
    this.gl = context.gl;
    const gl = this.gl;

    this.texture = gl.createTexture()!!;
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);

    this.size = new Vec2(image.width, image.height);
  }

  bind(location: WebGLUniformLocation) {
    this.gl.uniform1i(location, 0);
  }

  dispose() {
    this.gl.deleteTexture(this.texture);
  }
}
