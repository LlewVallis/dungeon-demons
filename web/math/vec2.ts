export default class Vec2 {
  constructor(readonly x: number, readonly y: number) {}

  mul(n: number): Vec2 {
    return new Vec2(this.x * n, this.y * n);
  }

  div(n: number): Vec2 {
    return new Vec2(this.x / n, this.y / n);
  }

  normalizeOrZero(): Vec2 {
    if (this.x === 0 || this.y === 0) {
      return this;
    } else {
      const length = Math.sqrt(this.x ** 2 + this.y ** 2);
      return new Vec2(this.x / length, this.y / length);
    }
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
