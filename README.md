# rslox - A simple scripting language

---
## Learning log

### Output buffering [06-01-25]
In many programming languages, data is temporarily stored in a location called a `buffer` before begin send to the console. This buffer holds the data until it's ready to be displayed

But this data is only displayed when one of these conditions is met
- The `buffer` is full
- A newline `\n` is entered
- **(Rust)** `flush` is explicitly called