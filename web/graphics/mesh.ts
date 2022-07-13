import Context from "./context";
import Buffer from "./buffer";
import { all } from "../util";

export default class Mesh {
  private readonly gl: WebGL2RenderingContext;
  private readonly vao: WebGLVertexArrayObject;
  private readonly buffers: Buffer[] = [];

  constructor(context: Context) {
    this.gl = context.gl;
    const gl = this.gl;

    this.vao = gl.createVertexArray()!!;
  }

  static createQuad(context: Context, location: number): Mesh {
    const positions = new Float32Array([
      -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5,
    ]);

    const buffer = new Buffer(context, positions);
    const mesh = new Mesh(context);
    mesh.addAttribute(location, buffer, context.gl.FLOAT, 2);

    return mesh;
  }

  addAttribute(
    location: number,
    buffer: Buffer,
    type: GLenum,
    size: number
  ): void {
    const gl = this.gl;

    buffer.bind();
    gl.bindVertexArray(this.vao);
    gl.enableVertexAttribArray(location);
    gl.vertexAttribPointer(location, size, type, false, 0, 0);

    this.buffers.push(buffer);
  }

  bind() {
    const gl = this.gl;
    gl.bindVertexArray(this.vao);
  }

  dispose(): void {
    const bufferDisposes = this.buffers.map((buffer) => () => buffer.dispose());

    all(...bufferDisposes, () => this.gl.deleteVertexArray(this.vao));
  }
}
