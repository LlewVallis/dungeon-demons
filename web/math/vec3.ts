export default class Vec3 {
  constructor(readonly x: number, readonly y: number, readonly z: number) {}

  mul(n: number): Vec3 {
    return new Vec3(this.x * n, this.y * n, this.z * n);
  }

  div(n: number): Vec3 {
    return new Vec3(this.x / n, this.y / n, this.z / n);
  }

  static add(left: Vec3, right: Vec3): Vec3 {
    return new Vec3(left.x + right.x, left.y + right.y, left.z + right.z);
  }

  static sub(left: Vec3, right: Vec3): Vec3 {
    return new Vec3(left.x - right.x, left.y - right.y, left.z - right.z);
  }

  static mul(left: Vec3, right: Vec3): Vec3 {
    return new Vec3(left.x * right.x, left.y * right.y, left.z * right.z);
  }

  static div(left: Vec3, right: Vec3): Vec3 {
    return new Vec3(left.x / right.x, left.y / right.y, left.z / right.z);
  }
}
