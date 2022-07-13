import Mat3 from "../math/mat3";
import Vec2 from "../math/vec2";
import Vec3 from "../math/vec3";

import { memory } from "../../pkg/index_bg.wasm";

export default class DrawBuffer {
  private readonly data = new DataView(memory.buffer);
  readonly size: number;

  constructor(private ptr: number) {
    this.size = this.readUint();
  }

  readUint(): number {
    const value = this.data.getUint32(this.ptr, true);
    this.ptr += 4;
    return value;
  }

  readFloat(): number {
    const value = this.data.getFloat32(this.ptr, true);
    this.ptr += 4;
    return value;
  }

  readMatrix(): Mat3 {
    return new Mat3(
      this.readFloat(),
      this.readFloat(),
      this.readFloat(),
      this.readFloat(),
      this.readFloat(),
      this.readFloat(),
      this.readFloat(),
      this.readFloat(),
      this.readFloat()
    );
  }

  readVec2(): Vec2 {
    return new Vec2(this.readFloat(), this.readFloat());
  }

  readVec3(): Vec3 {
    return new Vec3(this.readFloat(), this.readFloat(), this.readFloat());
  }
}
