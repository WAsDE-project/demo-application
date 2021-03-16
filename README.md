# The PoC

This folder contains the proof of concept implementation of our system.

- The `android` folder contains the android studio project that implements the android application that utilizes our runtime and a repository to run Wasm applications.
- The `modules` folder implements a repository that hosts the modules and their metadata as well as performs module selection based on attributes.
- The `wasmtime_based_runtime` folder contains the runtime implementation. It supports several platforms and can be executed as a standalone application as well as used as a library by the android application.
- The `manager` folder is a dependency used by the runtime implementation.

Go through the "Prerequisite tooling" chapter in order and check that all of the tools are installed. Afterward, the 

## Prerequisite tooling and steps
### Android Studio and Other Android tools
1. Install Android SDK: https://medium.com/better-programming/install-android-studio-in-ubuntu-b8aed675849f
2. Install Android NDK: https://developer.android.com/studio/projects/install-ndk.

Add `export ANDROID_SDK_ROOT="$HOME/Android/Sdk"` to your `.bash_profile` so that the later tooling can find your sdk installation.

Make sure you have the 21.3.6528147 version of ndk downloaded or modify the paths in the appropriate build commands to point to the correct NDK.

adb must be in your $PATH and your phone must have debugging enabled. enabled. See [adb doc](https://developer.android.com/studio/command-line/adb.html) . `adb devices -l` must show your phone.

#### Android API levels
The instructions build the runtime for API level 29. This means that the Android phone or emulator must have that API level or higher. We have not tested building for lower API levels so it is possible that the lower API levels might also support this runtime. To change the API level the runtime is being built for, the `-D__ANDROID_API__=` flag in the ssl build command needs to be changed as well as the `wasmtime_based_runtime/.cargo/config` file needs to be changed to point to the correct api level linker.
The dinghy command in `wasmtime_based_runtime/bench_android.sh` also needs to be changed to the corresponding API level.

### Rust toolchain and targets
Follow instructions at <https://www.rust-lang.org/tools/install> to install the rust toolchain. If 
```
cargo --version
``` 
```
rustup --version
```
and
```
rustc --version
```
all work then the toolchain should be ready.

Run
```
rustup target install aarch64-unknown-linux-gnu
rustup target install aarch64-linux-android
```
to install the necessary rust targets for cross-compilation.

### Crosstools-ng and the Raspberry Pi toolchain
Install crosstools-ng from https://crosstool-ng.github.io/

Create a folder for the config
run 
```
ct-ng aarch64-rpi3-linux-gnu
```
This will create a config file in the folder it was run in. Open the created config file and change the libc version to match the one on your raspi.

As an example the Raspberry Pi had a GLIBC version 2.28. The config looked like this in the appropriate section:
```
#
# Options for glibc
#
CT_LIBC_GLIBC_PKG_KSYM="GLIBC"
CT_GLIBC_DIR_NAME="glibc"
CT_GLIBC_USE_GNU=y
CT_GLIBC_USE="GLIBC"
CT_GLIBC_PKG_NAME="glibc"
CT_GLIBC_SRC_RELEASE=y
CT_GLIBC_PATCH_ORDER="global"
# CT_GLIBC_V_2_29 is not set
CT_GLIBC_V_2_28=y
# CT_GLIBC_V_2_27 is not set
# CT_GLIBC_V_2_26 is not set
# CT_GLIBC_V_2_25 is not set
# CT_GLIBC_V_2_24 is not set
# CT_GLIBC_V_2_23 is not set
# CT_GLIBC_V_2_19 is not set
# CT_GLIBC_V_2_17 is not set
# CT_GLIBC_V_2_12_1 is not set
# CT_GLIBC_NO_VERSIONS is not set
CT_GLIBC_VERSION="2.28"
```

To create the toolchain run 
```
ct-ng build
```
A toolchain should be created in ~/x-tools/aarch64-rpi3-linux-gnu

### OpenSSL
Get the openssl source from https://www.openssl.org/source/openssl-1.1.1h.tar.gz and extract it into a folder. For these instruction we assume that the source folder was extracted to the home folder. Remember to change the paths if another folder was used.

Change the directory to inside the source folder and then create the installation folders for the Android and Raspberry Pi builds:
```
cd ~/openssl-1.1.1h/
mkdir android_install
mkdir rpi_install
```


For the Android build run
```
make clean

export ANDROID_NDK_HOME=~/Android/Sdk/ndk/21.3.6528147; export PATH=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$ANDROID_NDK_HOME/toolchains/aarch64-linux-android-4.9/prebuilt/linux-x86_64/bin:$PATH; ./Configure android-arm64 -D__ANDROID_API__=29 --prefix=$HOME/openssl-1.1.1h/android_install/

make

make install
```

For the Raspberry Pi build run:
```
make clean

./Configure linux-aarch64 --cross-compile-prefix=$HOME/x-tools/aarch64-rpi3-linux-gnu/bin/aarch64-rpi3-linux-gnu- --prefix=$HOME/openssl-1.1.1h/rpi_install/ --openssldir=$HOME/openssl-1.1.1h/rpi_install/ -static

make

make install
```

Now the builds for Android and Raspberry Pi should be in their respective directories.

### Dinghy
These instructions are based on
https://github.com/sonos/dinghy/blob/main/docs/android.md
and 
https://github.com/sonos/dinghy/blob/main/docs/ssh.md


Make sure you have the Rust toolchain, The raspi toolchain installed with Crosstools-ng and Android tools installed.

Run
```
cargo install cargo-dinghy
```
to install dinghy. Running 
```
cargo dinghy --version
```
should now output the version information for dinghy.

#### Setting up for Android
If the Android SDK and NDK are setup correctly running
```
cargo dinghy all-platforms
``` 
should output a list of platforms. Check that `auto-android-aarch64-api29` is available as we depend on API level 29.

If the API level is not available. Some instructions on changing the API level can be found in the "Android Studio and Other Android tools" section above however we cannot guarantee that the benchmarks will work.

#### Setting up for Raspberry Pi
Make sure you have the Raspberry Pi toolchain installed with crosstools-ng. Make sure that ssh is enable on the Raspberry Pi and that you can connect to it. Create `.dinghy.toml` file into your home directory. The contents of the file should be something like 
```toml
[platforms.64-bit-pi]
rustc_triple="aarch64-unknown-linux-gnu"
toolchain="/home/username/x-tools/aarch64-rpi3-linux-gnu"

[ssh_devices]
raspi = { hostname = "192.168.0.214", username="pi", platform="64-bit-pi" }
```
Change the toolchain path, hostname and username to correspond to your toolchain installation path, Raspberry Pi network address and Raspberry Pi username respectively.

Follow the instruction at https://www.raspberrypi.org/documentation/remote-access/ssh/passwordless.md to avoid having to write your password when using dinghy with your Raspberry Pi.

### Metadata setup

Replace all `insert_your_internal_ip_here` strings in the `JSON` metadata files with your actual ip. The metadata files are located in `modules/public`. `localhost` cannot be used as the address because the Android phone and the Raspberry Pi will also need to be able to locate your server.

You can check your ip by running 
```
ifconfig
```
One of the easier ways to insert your ip's if you have Visual Studio Code install is to open it in this folder, click on the magnifying glass icon on the top left and search for `insert_your_internal_ip_here` and replace with your ip.

## Running demo applications

Run the repository using the instructions in `modules/README.md`.

### Running the Android demo application
To build the Android runtime, first uncomment the line under `# Uncomment this for Android build.` in the `wasmtime_based_runtime/Cargo.toml` file.
Then run the following command
```
export OPENSSL_STATIC=1;export AARCH64_LINUX_ANDROID_OPENSSL_DIR=~/openssl-1.1.1h/install/;cargo dinghy -d android --platform auto-android-aarch64-api29 build --release
```
in the `wasmtime_based_runtime` folder. This should create a library file. The android project contains a symlink to this file so no further actions are needed.

Open the project in the `android` folder in Android Studio. Make sure you have an aarch64 based phone connected. It should show up as the active device next to the run button. The application is started with the green Run button in Android Studio.

The application shows different buttons that activate different modules. The modules are fetched and linked from the repository when required.

### Running the demo application on Linux
To build the runtime, first uncomment the line under `# Uncomment this for Desktop and Raspberry Pi build.` in the `wasmtime_based_runtime/Cargo.toml` file.

Then run the following command in `wasmtime_based_runtime` to build the runtime
```
cargo run --release -- ../modules/public/dynamic_linking/main.json
```

### Running the demo application on Raspberry Pi
To build the runtime, first uncomment the line under `# Uncomment this for Desktop and Raspberry Pi build.` in the `wasmtime_based_runtime/Cargo.toml` file.

Then run the following command in `wasmtime_based_runtime` to build the runtime
```
export AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_DIR=~/openssl-1.1.1h/install_rpi_real/;cargo dinghy -d raspi build --release
```

Then copy the binary from `wasmtime_based_runtime/aarch64-unknown-linux-gnu/release/runtime-binary` and the metadta from `modules/public/dynamic_linking/main.json` to the Raspberry Pi using `scp` for example.

On the Raspberry Pi run the following command to execute the demo application.
```
./runtime-binary main.json
```
