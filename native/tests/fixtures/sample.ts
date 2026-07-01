// TypeScript sample for tree-sitter parsing tests
function greet(name: string): string {
    const greeting = `Hello, ${name}!`;
    console.log(greeting);
    return greeting;
}

interface User {
    id: number;
    name: string;
    email?: string;
}

const users: User[] = [
    { id: 1, name: "Alice" },
    { id: 2, name: "Bob", email: "bob@test.com" },
];

export function findUser(id: number): User | undefined {
    return users.find(u => u.id === id);
}
