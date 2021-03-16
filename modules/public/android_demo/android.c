#include <stdlib.h> // malloc
#include <stddef.h> // size_t

__attribute__((import_module("host")))
void* dlopen(const char *);
__attribute__((import_module("host")))
void* dlsym(void*, const char *);
__attribute__((import_module("host")))
char* dlerror();

// Android API
__attribute__((import_module("host")))
void Log(const char* message);
__attribute__((import_module("host")))
void RegisterOnTick(void (*callback)());
// TextView
__attribute__((import_module("host")))
void* CreateTextView(const char* text);
__attribute__((import_module("host")))
void ModifyTextView(void* textview_handle, const char* text);
__attribute__((import_module("host")))
void RemoveTextView(void* textview_handle);
// Button
__attribute__((import_module("host")))
void* CreateButton(const char* label);
__attribute__((import_module("host")))
int RegisterOnClick(void* button_handle, void (*callback)());
// Bitmap
__attribute__((import_module("host")))
void* CreateBitmap(int width, int height);
__attribute__((import_module("host")))
void ModifyBitmap(void* bitmap_handle, int x, int y, int color);
__attribute__((import_module("host")))
void BitmapSetPosition(void* bitmap_handle, int left, int top);
__attribute__((import_module("host")))
void BitmapSetZIndex(void* bitmap_handle, int z_index);
// Canvas
__attribute__((import_module("host")))
void* CreateCanvas(int width, int height);
__attribute__((import_module("host")))
void CanvasAddBitmap(void* canvas_handle, void* bitmap_handle);
__attribute__((import_module("host")))
void CanvasRedraw(void* canvas_handle);

// alloc function export is required by our runtime to be able to allocate memory
__attribute__((used))
void* alloc(size_t size) {
    void* p = malloc(size);
    return p;
}

const int WIDTH = 500;
const int HEIGHT = 500;

// global so they can be accessed in callbacks
void* output = 0;
void* canvas = 0;
void* bmp = 0;
unsigned int img[WIDTH][HEIGHT] = { 0 };

int test_error(void* handle) {
    if (!handle) {
        ModifyTextView(output, dlerror());
        return 0;
    }
    return 1;
}

int load_and_link(const char* module, const char* symbol, void** module_handle, void** symbol_handle) {
    if (!*module_handle) {
        *module_handle = dlopen(module);
        if (!test_error(*module_handle)) return 0;
    }
    if (!*symbol_handle) {
        *symbol_handle = dlsym(*module_handle, symbol);
        if (!test_error(*symbol_handle)) return 0;
    }
    return 1;
}

void redraw() {
    for (int x = 0; x < WIDTH; ++x) {
        for (int y = 0; y < HEIGHT; ++y) {
            ModifyBitmap(bmp, x, y, img[y][x]);
        }
    }
    CanvasRedraw(canvas);
}

void* reset_canvas_handle = 0;
int (*reset_canvas)(unsigned int* img, int width, int height) = 0;
void load_canvas_callback() {
    if (load_and_link("android_canvas", "setup", &reset_canvas_handle, (void**)&reset_canvas))
        reset_canvas((unsigned int*)img, WIDTH, HEIGHT);
    redraw();
}

void* invert_canvas_handle = 0;
int (*invert_canvas)(unsigned int* img, int width, int height) = 0;
void invert_callback() {
    if (load_and_link("android_bmp_invert", "invert", &invert_canvas_handle, (void**)&invert_canvas))
        invert_canvas((unsigned int*)img, WIDTH, HEIGHT);
    redraw();
}

void* grayscale_canvas_handle = 0;
int (*grayscale_canvas)(unsigned int* img, int width, int height) = 0;
void grayscale_callback() {
    if (load_and_link("android_bmp_grayscale", "grayscale", &grayscale_canvas_handle, (void**)&grayscale_canvas))
        grayscale_canvas((unsigned int*)img, WIDTH, HEIGHT);
    redraw();
}

void* chatbot_handle = 0;
int (*chatbot_write_message)(void* textview_handle) = 0;
void chatbot_callback() {
    if (load_and_link("android_chatbot", "write_message", &chatbot_handle, (void**)&chatbot_write_message))
        chatbot_write_message(output);
}

void error_callback() {
    void* handle = dlopen("nonexistant_module");
    test_error(handle);
}

__attribute__((used))
__attribute__((export_name("_start")))
int init() {
    output = CreateTextView("Hello world!");

    void* chatbot_button = CreateButton("Say something");
    RegisterOnClick(chatbot_button, &chatbot_callback);

    void* error_button = CreateButton("Error demo");
    RegisterOnClick(error_button, &error_callback);

    canvas = CreateCanvas(WIDTH, HEIGHT);
    bmp = CreateBitmap(WIDTH, HEIGHT);
    BitmapSetPosition(bmp, 0, 0);
    BitmapSetZIndex(bmp, 0);
    CanvasAddBitmap(canvas, bmp);

    void* load_canvas_button = CreateButton("Load image");
    RegisterOnClick(load_canvas_button, &load_canvas_callback);

    void* invert_button = CreateButton("Invert");
    RegisterOnClick(invert_button, &invert_callback);

    void* grayscale_button = CreateButton("Grayscale");
    RegisterOnClick(grayscale_button, &grayscale_callback);

    redraw();
    return 0;
}
