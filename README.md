# wrapper-small-pot
Small Powers of Tau Rust code wrapper to be used in browsers for participants' contributions. The crypto implementation used as library is [https://github.com/crate-crypto/small-powers-of-tau](https://github.com/crate-crypto/small-powers-of-tau).

&nbsp;

## Rust

### **Check**
To check that the wrapper code is written properly, run:

```  cargo check --target x86_64-unknown-linux-gnu ```

### **Build**
To build the wrapper code, run:

``` cargo build --target x86_64-unknown-linux-gnu ```

### **Run**
To build and run the wrapper code and test it using the code inside the `main.rs` file, run:

``` cargo run --release --target x86_64-unknown-linux-gnu ```

*Note:* In Ubuntu/Linux you can use target `x86_64-unknown-linux-gnu`, in Windows you can use `WINDOWS_TARGET_HERE`

&nbsp;


## Wasm

### **Build**
To get the files for integrate the code into Javascript, you can run:

``` wasm-pack build --target web -d wasm/pkg ```

### **Test**
To test that the wasm is called correctly in a web setting, you can run a HTTP server:

``` cd wasm ```
``` python3 server.py ```

In some cases, the `wasm-worker.js` might not run and not throw any error. This issue could be cause because the functions in `wasm.rs` where not binded correctly.