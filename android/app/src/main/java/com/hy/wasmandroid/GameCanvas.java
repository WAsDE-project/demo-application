package com.hy.wasmandroid;

import android.content.Context;
import android.graphics.Bitmap;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.view.View;
import android.view.ViewGroup;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

enum SpriteType {
    BITMAP,
    TEXT
}

public class GameCanvas extends View {
    public static class Sprite implements Comparable<Sprite> {
        private final int id;

        public int zIndex;
        public int left;
        public int top;
        public int color;
        public int text_size;
        public Paint paint;
        public Bitmap bitmap;
        public String text;
        public final SpriteType type;

        public Sprite(int id, int width, int height) {
            this.id = id;
            this.type = SpriteType.BITMAP;
            this.bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888);
        }

        public Sprite(int id, String text, int color, int text_size) {
            this.id = id;
            this.text = text;
            this.color = color;
            this.text_size = text_size;
            this.type = SpriteType.TEXT;
            Paint paint = new Paint();
            // TODO: Map colors to numbers.
            paint.setColor(Color.WHITE);
            paint.setTextSize(this.text_size);
            this.paint = paint;
        }

        @Override
        public int compareTo(Sprite other) {
            long indexA = ((long) zIndex << 32) + id;
            long indexB = ((long)other.zIndex << 32) + other.id;
            long result = indexA - indexB;
            if (result < 0) return -1;
            if (result > 0) return 1;
            return 0;
        }
    }

    public List<Sprite> sprites = new ArrayList<>();

    public GameCanvas(Context context, int width, int height) {
        super(context);
        setLayoutParams(new ViewGroup.LayoutParams(width, height));
    }

    public void RemoveSprite(int id) {
        sprites.removeIf(s -> s.id == id);
    }

    @Override
    protected void onDraw(Canvas canvas) {
        super.onDraw(canvas);
        canvas.drawColor(Color.BLACK);
        Collections.sort(sprites); // sort by z index
        sprites.forEach(s -> {
            if (s.type == SpriteType.BITMAP) {
                canvas.drawBitmap(s.bitmap, s.left, s.top, null);
            } else {
                canvas.drawText(s.text, s.left, s.top, s.paint);
            }
        });
    }
}
