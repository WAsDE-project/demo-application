__attribute__((used))
int setup(unsigned int* img, int width, int height) {
    int mid_width = width/2;
    int mid_height = height/2;
    int thickness_width = 0.2*width;
    int thickness_height = 0.2*height;
    for (int x = 0; x < width; ++x) {
        for (int y = 0; y < height; ++y) {
            if ((x >= mid_width-thickness_width && x <= mid_width+thickness_width) || (y >= mid_height-thickness_height && y <= mid_height+thickness_height))
                img[x+(width*y)] = 0xFF000000;
            else
        	    img[x+(width*y)] = 0xFFFFFFFF;
        }
    }
    return 1;
}
