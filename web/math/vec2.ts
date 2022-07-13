export default class Vec2 {
  constructor(readonly x: number, readonly y: number) {}

  mul(n: number): Vec2 {
    return new Vec2(this.x * n, this.y * n);
  }

  div(n: number): Vec2 {
    return new Vec2(this.x / n, this.y / n);
  }

  length(): number {
    return Math.sqrt(this.x ** 2 + this.y ** 2);
  }

  normalize(): Vec2 {
    const length = this.length();
    return new Vec2(this.x / length, this.y / length);
  }

  static add(left: Vec2, right: Vec2): Vec2 {
    return new Vec2(left.x + right.x, left.y + right.y);
  }

  static sub(left: Vec2, right: Vec2): Vec2 {
    return new Vec2(left.x - right.x, left.y - right.y);
  }

  static mul(left: Vec2, right: Vec2): Vec2 {
    return new Vec2(left.x * right.x, left.y * right.y);
  }

  static div(left: Vec2, right: Vec2): Vec2 {
    return new Vec2(left.x / right.x, left.y / right.y);
  }
}
