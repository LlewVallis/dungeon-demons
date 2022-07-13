import Context from "./context";
import Mat3 from "../math/mat3";
import Vec2 from "../math/vec2";
import Vec3 from "../math/vec3";
import { all } from "../util";

export default class Program {
  private readonly gl: WebGL2RenderingContext;
  private readonly program: WebGLProgram;
  private readonly vert: WebGLShader;
  private readonly frag: WebGLShader;

  private readonly uniformLocations: Record<string, WebGLUniformLocation> = {};

  constructor(context: Context, vertSource: string, fragSource: string) {
    this.gl = context.gl;
    const gl = this.gl;

    this.vert = this.createShader(gl.VERTEX_SHADER, vertSource);
    this.frag = this.createShader(gl.FRAGMENT_SHADER, fragSource);
    this.program = this.createProgram();
  }

  getUniformLocation(uniform: string): WebGLUniformLocation {
    if (!(uniform in this.uniformLocations)) {
      this.uniformLocations[uniform] = this.gl.getUniformLocation(
        this.program,
        uniform
      )!!;
    }

    return this.uniformLocations[uniform];
  }

  bind(): void {
    this.gl.useProgram(this.program);
  }

  static uniformMat3(
    context: Context,
    location: WebGLUniformLocation,
    matrix: Mat3
  ): void {
    const buffer = new Float32Array(3 * 3);
    buffer[0] = matrix.m11;
    buffer[1] = matrix.m12;
    buffer[2] = matrix.m13;
    buffer[3] = matrix.m21;
    buffer[4] = matrix.m22;
    buffer[5] = matrix.m23;
    buffer[6] = matrix.m31;
    buffer[7] = matrix.m32;
    buffer[8] = matrix.m33;

    context.gl.uniformMatrix3fv(location, false, buffer);
  }

  static uniformVec2(
    context: Context,
    location: WebGLUniformLocation,
    vector: Vec2
  ): void {
    context.gl.uniform2f(location, vector.x, vector.y);
  }

  static uniformVec3(
    context: Context,
    location: WebGLUniformLocation,
    vector: Vec3
  ): void {
    context.gl.uniform3f(location, vector.x, vector.y, vector.z);
  }

  static uniformFloat(
    context: Context,
    location: WebGLUniformLocation,
    float: number
  ): void {
    context.gl.uniform1f(location, float);
  }

  private createShader(type: GLenum, source: string): WebGLShader {
    const gl = this.gl;

    const shader = gl.createShader(type)!!;
    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    const status = gl.getShaderParameter(shader, gl.COMPILE_STATUS);
    if (!status) {
      const log = gl.getShaderInfoLog(shader);
      console.error(`Shader error: ${log}`);
    }

    return shader;
  }

  private createProgram(): WebGLProgram {
    const gl = this.gl;

    const program = gl.createProgram()!!;
    gl.attachShader(program, this.vert);
    gl.attachShader(program, this.frag);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    if (!status) {
      const log = gl.getProgramInfoLog(program);
      console.error(`Shader program error: ${log}`);
    }

    return program;
  }

  dispose(): void {
    const gl = this.gl;
    all(
      () => gl.deleteProgram(this.program),
      () => gl.deleteShader(this.vert),
      () => gl.deleteShader(this.frag)
    );
  }
}
