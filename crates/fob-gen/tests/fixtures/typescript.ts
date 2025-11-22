// TypeScript features
interface User {
  name: string;
  age: number;
}

type Status = 'active' | 'inactive';

function processUser<T extends User>(user: T): T {
  return user;
}

export { User, Status, processUser };
