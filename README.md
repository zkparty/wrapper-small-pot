# wrapper-small-pot
This wrapper create helper functions to integrate the [KZG sequencer crypto library](https://github.com/ethereum/kzg-ceremony-sequencer/tree/master/crypto) with a [web implementation](https://github.com/zkparty/trusted-setup-frontend) to contribute into the [KZG Ceremony](https://github.com/ethereum/kzg-ceremony).

It previously used [Small Powers of Tau](https://github.com/crate-crypto/small-powers-of-tau) but we decided to on with the same code base as the sequencer.

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

*Note:* In Ubuntu/Linux you can use target `x86_64-unknown-linux-gnu`, in Windows you can use `x86_64-pc-windows-msvc`


### **Test**
To build and run tests, run:

``` cargo test ```

&nbsp;


## Wasm

### **Build**
To get the files for integrate the code into Javascript, you can run:

``` wasm-pack build --target web -d wasm/pkg ```

### **Test**
To test that the wasm is called correctly in a web setting, you need to:

1. Have a `initialContribution.json` file in the `/wasm` directory. Check an example [here](https://github.com/ethereum/kzg-ceremony-specs)

2. Run a HTTP server: ` cd wasm ` && ` python3 server.py `

3. Go to [http://localhost:8000/]() and input some entropy. Open the devTools to check everything is ok.

In some cases, the `wasm-worker.js` might not run and not throw any error. This issue could be caused because the functions in `wasm.rs` where not binded correctly.


## Docker
A Docker image is available, providing an environment in which to build this code. 

The image, named `zkparty/wasm-pack-wrapper`, can be found in the hub at https://hub.docker.com/.

### Building the WASM package

The WASM package can, of course, be built in your local environment providing you the required Rust toolkit along with `wasm-pack`. The Docker image provides the necessary environment, and can be helpful in obtaining a reproducible build. 

To make the WASM package folder accessible, you need to map a volume from your local environment to `/root/wasm`.

Build the WASM package using a command similar to this: ```docker run -it --rm 
    -v "</local/path/to/wasm/output/>:/root/wasm/" 
    zkparty/wasm-pack-wrapper```

### Build the Docker image
To rebuild the Docker image after code changes:
`docker build . -t zkparty/wasm-pack-wrapper`.


