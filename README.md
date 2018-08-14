# katlang

A WIP golfing language.

Katlang is a conkatenative, interpreted, stack-based language. It attempts to utilize the implicit argument passing provided by the stack as much as possible. This means that things like mapping over lists (or stacks, as they are known in catlang) means pushing the elements to the stack one by one, running the callback block and collecting all new items that were left on the stack as the results.

See the [spec](spec.md) for more details.

```
> "Hello, ""Your name: "wR+
Interpreted as: [CreateString("Hello, "), CreateString("Your name: "), Write, ReadLine, Add]
Your name: World
Hello, World
```
