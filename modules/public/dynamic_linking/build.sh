export DOCKER_EMSCR="docker run --rm -v $(pwd):/src -u $(id -u):$(id -g) emscripten/emsdk:2.0.10"

$DOCKER_EMSCR emcc main.c -o main.wasm -fPIC -Wl,--export-table,--growable-table,--export-all,--export=__stack_pointer -s ALLOW_MEMORY_GROWTH=1
$DOCKER_EMSCR emcc side.c -o side.wasm -s SIDE_MODULE=1

