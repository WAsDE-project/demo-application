__attribute__((used))
int invert(unsigned int* img, int width, int height) {
    for (int x = 0; x < width; ++x) {
        for (int y = 0; y < height; ++y) {
            img[x+(width*y)] = img[x+(width*y)] ^ 0x00FFFFFF;
        }
    }
	return 1;
}
