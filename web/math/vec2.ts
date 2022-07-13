export default class Vec2 {
  constructor(readonly x: number, readonly y: number) {}

  mul(n: number): Vec2 {
    return new Vec2(this.x * n, this.y * n);
  }

  div(n: number): Vec2 {
    return new Vec2(this.x / n, this.y / n);
  }

  inverse(): Vec2 {
    return new Vec2(1 / this.x, 1 / this.y);
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
