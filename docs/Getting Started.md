# CoccinelleForRust

## Getting Started

CoccinelleForRust(cfr) is a program transformation tool, which is the rust version of [coccinelle](https://coccinelle.lip6.fr/) . It uses the Semantic Patch Language(SmPL) to describe changes to a codebase and transform them accordingly.

For example, 

We have a rust project which deals with a robot with wheels. At first we could set only the speed of a wheel and each wheel could have a positive or negative speed showing its direction. 

```
struct Wheel {
    ...
}

impl Wheel {
    pub fn setSpeed(speed: i32) {
        ...
    }
}

```

At different points in the code wheelx.setSpeed(i32) is called. But due to recent change in API, it would be better to use a bool variable to set the direction and using usize for the magnitude. So the new definition would be.

```
struct Wheel {
    ...
}

impl Wheel {
    pub fn setSpeed(speed: usize, direction: bool) {
        ...
    }
}
```

We would have to change all occurences of setSpeed to incorporate a new parameter while keeping the magnitude of speed the same.

To do this in SmPL is simple. We will need something called metavariables. Metavariables are variables which can represent different parts of an Abstract Syntax Tree. Currently cfr supports expression, identifier, type, lifetime, and parameter metavariables. More metavariables are being added. To achieve the above task we would need two metavariables, one for refering to the wheel instance and the other for the speed. The SmPL would look something like this :-

```
@ rule1 @
expression wheel, speed;
@@

- x.setSpeed(y);
+ x.setSpeed(y, true);
```

This is not the correct patch, but we will come to that.

The @@s are used to seperate the declaration space from the modifier space.
The first line declares the name of a Rule. Each rule is applied on the whole source code before going on to the next rule. Rules can inherit metavariables from other previously declared rules. But that is a discussion for another section.

The second line declares two expression metavariables named wheel and speed. These variables can take on any valid expression: literals, objects, function calls etc.

The last two lines make the actual chnge we want. The minus(-) at the beginning of the line instructs Coccinelle to remove any matching line from rust file, and the plus(+) in the next line adds that line along with the information it collected in the previous line.

For the following code segment

```
x.setSpeed(100);
```

would be replaced by

```
x.setSpeed(100, true);
```

