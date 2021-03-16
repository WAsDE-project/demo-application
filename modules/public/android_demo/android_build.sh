export DOCKER_EMSCR="docker run --rm -v $(pwd):/src -u $(id -u):$(id -g) emscripten/emsdk:2.0.10"

$DOCKER_EMSCR emcc android.c -o android.wasm -fPIC -Wl,--export-table,--growable-table,--no-entry,--export=__stack_pointer -s ALLOW_MEMORY_GROWTH=1 -s ERROR_ON_UNDEFINED_SYMBOLS=0

$DOCKER_EMSCR emcc android_bmp_grayscale.c -o android_bmp_grayscale.wasm -s SIDE_MODULE=1
$DOCKER_EMSCR emcc android_bmp_invert.c -o android_bmp_invert.wasm -s SIDE_MODULE=1
$DOCKER_EMSCR emcc android_bmp_threshold.c -o android_bmp_threshold.wasm -s SIDE_MODULE=1
$DOCKER_EMSCR emcc android_canvas.c -o android_canvas.wasm -s SIDE_MODULE=1
$DOCKER_EMSCR emcc android_canvas_real.c -o android_canvas_real.wasm -s SIDE_MODULE=1 -Wno-trigraphs
$DOCKER_EMSCR emcc android_chatbot.c -o android_chatbot.wasm -s SIDE_MODULE=1
