// Modern decorator (TC39 Stage 3) example
function logged(value: any, context: ClassMethodDecoratorContext) {
  const methodName = String(context.name);

  function replacementMethod(this: any, ...args: any[]) {
    console.log(`Calling ${methodName}`);
    const result = value.call(this, ...args);
    console.log(`Called ${methodName}`);
    return result;
  }

  return replacementMethod;
}

class MyClass {
  @logged
  greet(name: string) {
    return `Hello, ${name}!`;
  }
}

export { MyClass };
