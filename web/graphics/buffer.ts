import Context from "./context";

export default class Buffer {
  private readonly gl: WebGL2RenderingContext;
  private readonly buffer: WebGLBuffer;

  constructor(context: Context, data: BufferSource) {
    this.gl = context.gl;
    const gl = this.gl;

    this.buffer = gl.createBuffer()!!;
    this.bind();

    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
  }

  bind() {
    const gl = this.gl;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.buffer);
  }

  dispose(): void {
    this.gl.deleteBuffer(this.buffer);
  }
}
