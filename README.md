# Miracle Octad Generator (Interactive GUI)

## Try It Online  
You can experiment with the latest version directly in your browser:  
👉 [Play with it here](https://pishleback.github.io/Miracle-Octad-Generator/)

## What Is It For?  
This project provides an interactive graphical interface for the [Miracle Octad Generator](https://en.wikipedia.org/wiki/Miracle_Octad_Generator), a tool in combinatorial design and group theory especially useful for exploring:  
- The [Steiner system](https://en.wikipedia.org/wiki/Steiner_system) $S(5, 8, 24)$
- The [extended binary Golay code](https://en.wikipedia.org/wiki/Binary_Golay_code) and the [perfect binary Golay code](https://en.wikipedia.org/wiki/Binary_Golay_code)  
- The [Mathieu group](https://en.wikipedia.org/wiki/Mathieu_group) $M_{24}$ and its subgroups  
- Other related mathematical structures  

## Run It Locally  

If you’d like to build and run the project on your own machine:  

1. Install the [Rust compiler](https://rustup.rs/).  
   - Verify your installation by running:  
     ```bash
     cargo --version
     ```
2. Clone this repository and open a terminal in the project’s root directory.  
3. Build and launch natively:
   ```bash
   cargo run --release
   ```
