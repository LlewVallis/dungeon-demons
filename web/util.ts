export function all(...callbacks: (() => void)[]) {
  const errors = [];

  for (const callback of callbacks) {
    try {
      callback();
    } catch (err) {
      errors.push(err);
    }
  }

  if (errors.length > 1) {
    throw new AggregateError(errors);
  } else if (errors.length === 1) {
    throw errors[0];
  }
}
