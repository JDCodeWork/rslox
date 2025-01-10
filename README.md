# rslox - A simple scripting language

---
## Learning log

### Dynamic types [10-01-25]
Rust as an innovate language compared to many others by providing a safe and efficient way to manage memory through concepts like **ownership**, **borrowing**, **lifetimes**, and the combined management of the **stack** and the **heap** 

For example, in Rust, it's possible to handle dynamic data flexible and securely using structures like the following:
```rust
pub struct Token { 
    literal: Box<dyn any::Any>,
}
```

In this case:
- **`Box`** allows the data to be stored on the **heap**, ensured that the Rust automatically handles its allocation and deallocation
- **`dyn`** indicates that the specific type of the value is not know at the compile time.
- **`any::Any`** allows the `literal` field to store values of any time that implements the `Any` trait.

### Output buffering [06-01-25]
In many programming languages, data is temporarily stored in a location called a `buffer` before begin send to the console. This buffer holds the data until it's ready to be displayed

But this data is only displayed when one of these conditions is met
- The `buffer` is full
- A newline `\n` is entered
- **(Rust)** `flush` is explicitly called