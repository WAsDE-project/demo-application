__attribute__((used))
int grayscale(unsigned int* img, int width, int height) {
    for (int x = 0; x < width; ++x) {
        for (int y = 0; y < height; ++y) {
            int r = (img[x+(width*y)] & 0x00FF0000)>>16;
            int g = (img[x+(width*y)] & 0x0000FF00)>>8;
            int b = img[x+(width*y)] & 0x000000FF;
            int color = (r+g+b)/3;
            int pixel = 0xFF000000 + (color<<16) + (color<<8) + (color);
            img[x+(width*y)] = pixel;
        }
    }
	return 1;
}
