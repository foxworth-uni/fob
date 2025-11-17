// TypeScript features
interface User {
    name: string;
    age: number;
}

type Status = 'active' | 'inactive';

function processUser<T extends User>(user: T): T {
    return user;
}

const users: User[] = [
    { name: "Alice", age: 30 },
    { name: "Bob", age: 25 }
];

export { User, Status, processUser };

