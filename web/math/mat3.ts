import Vec2 from "./vec2";

export default class Mat3 {
  constructor(
    readonly m11: number,
    readonly m21: number,
    readonly m31: number,
    readonly m12: number,
    readonly m22: number,
    readonly m32: number,
    readonly m13: number,
    readonly m23: number,
    readonly m33: number
  ) {}

  determinant(): number {
    return (
      this.m11 * (this.m22 * this.m33 - this.m32 * this.m23) -
      this.m12 * (this.m12 * this.m33 - this.m32 * this.m13) +
      this.m13 * (this.m12 * this.m23 - this.m22 * this.m13)
    );
  }

  inverse(): Mat3 {
    const x = 1 / this.determinant();

    return new Mat3(
      (this.m22 * this.m33 - this.m32 * this.m23) * x,
      (this.m31 * this.m23 - this.m21 * this.m33) * x,
      (this.m21 * this.m32 - this.m31 * this.m22) * x,

      (this.m32 * this.m13 - this.m12 * this.m33) * x,
      (this.m11 * this.m33 - this.m31 * this.m13) * x,
      (this.m31 * this.m12 - this.m11 * this.m32) * x,

      (this.m12 * this.m23 - this.m22 * this.m13) * x,
      (this.m21 * this.m13 - this.m11 * this.m23) * x,
      (this.m11 * this.m22 - this.m21 * this.m12) * x
    );
  }

  apply(v: Vec2): Vec2 {
    return new Vec2(
      this.m11 * v.x + this.m21 * v.y + this.m31,
      this.m12 * v.x + this.m22 * v.y + this.m32
    );
  }

  static identity = new Mat3(1, 0, 0, 0, 1, 0, 0, 0, 1);

  static mul(left: Mat3, right: Mat3): Mat3 {
    return new Mat3(
      left.m11 * right.m11 + left.m21 * right.m12 + left.m31 * right.m13,
      left.m11 * right.m21 + left.m21 * right.m22 + left.m31 * right.m23,
      left.m11 * right.m31 + left.m21 * right.m32 + left.m31 * right.m33,

      left.m12 * right.m11 + left.m22 * right.m12 + left.m32 * right.m13,
      left.m12 * right.m21 + left.m22 * right.m22 + left.m32 * right.m23,
      left.m12 * right.m31 + left.m22 * right.m32 + left.m32 * right.m33,

      left.m13 * right.m11 + left.m23 * right.m12 + left.m33 * right.m13,
      left.m13 * right.m21 + left.m23 * right.m22 + left.m33 * right.m23,
      left.m13 * right.m31 + left.m23 * right.m32 + left.m33 * right.m33
    );
  }

  static translation(vector: Vec2): Mat3 {
    return new Mat3(1, 0, vector.x, 0, 1, vector.y, 0, 0, 1);
  }

  static scale(vector: Vec2): Mat3 {
    return new Mat3(vector.x, 0, 0, 0, vector.y, 0, 0, 0, 1);
  }
}
