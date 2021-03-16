package com.hy.wasmandroid;

import android.app.Activity;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;

import java.util.EnumMap;
import java.util.HashMap;
import java.util.Map;

public class GuiContext {
    private static native int JNIButtonPress(int id) throws Exception;

    private Map<ElementType, Integer> counters = new EnumMap<>(ElementType.class);
    private Map<Integer, TextView> textViews = new HashMap<>();
    private Map<Integer, Button> buttons = new HashMap<>();
    private Map<Integer, GameCanvas> canvases = new HashMap<>();
    private Map<Integer, GameCanvas.Sprite> sprites = new HashMap<>();
    private Activity activity;
    private LinearLayout.LayoutParams params;
    private LinearLayout linearLayout;

    public GuiContext(Activity activity) {
        this.activity = activity;
        this.params = new LinearLayout.LayoutParams(LinearLayout.LayoutParams.MATCH_PARENT, LinearLayout.LayoutParams.WRAP_CONTENT);
        this.linearLayout = new LinearLayout(activity);
        linearLayout.setOrientation(LinearLayout.VERTICAL);
        activity.addContentView(linearLayout, params);
    }

    int createTextView(String text) {
        // TODO: Have the id's generated in a way that reuses id's of removed textViews.
        Integer id = counters.merge(ElementType.TEXT_VIEW, 1, Integer::sum);
        TextView textView = new TextView(activity);
        textView.setText(text);
        textView.setLayoutParams(params);
        this.linearLayout.addView(textView);
        textViews.put(id, textView);
        return id;
    }

    void modifyTextView(int id, String text) {
        TextView textView = textViews.get(id);
        textView.setText(text);
    }

    void removeTextView(int id) {
        this.linearLayout.removeView(textViews.remove(id));
    }

    int createButton(String label) {
        System.out.println("CREATED BUTTON");
        // TODO: Check the todo in the createTextView method.
        Integer id = counters.merge(ElementType.BUTTON, 1, Integer::sum);
        Button button = new Button(activity);
        button.setText(label);
        button.setLayoutParams(params);
        button.setOnClickListener(v -> {
            System.out.println("Button "+ id +" pressed;");
            try {
                JNIButtonPress(id);
            } catch(Exception e) {
                e.printStackTrace();
            }
        });
        this.linearLayout.addView(button);
        buttons.put(id, button);
        return id;
    }

    int createCanvas(int width, int height) {
        // TODO: Check the todo in the createTextView method.
        Integer id = counters.merge(ElementType.CANVAS, 1, Integer::sum);
        GameCanvas canvas = new GameCanvas(activity, width, height);
        linearLayout.addView(canvas);
        canvases.put(id, canvas);
        return id;
    }

    int createBitmap(int width, int height) {
        // TODO: Check the todo in the createTextView method.
        Integer id = counters.merge(ElementType.BITMAP, 1, Integer::sum);
        GameCanvas.Sprite sprite = new GameCanvas.Sprite(id, width, height);
        sprites.put(id, sprite);
        return id;
    }

    int createText(String text, int color, int text_size) {
        Integer id = counters.merge(ElementType.BITMAP, 1, Integer::sum);
        GameCanvas.Sprite sprite = new GameCanvas.Sprite(id, text, color, text_size);
        sprites.put(id, sprite);
        return id;
    }

    void setText(int text_id, String text) {
        GameCanvas.Sprite sprite = sprites.get(text_id);
        sprite.text = text;
    }

    void modifyBitmap(int id, int x, int y, int color) {
        GameCanvas.Sprite sprite = sprites.get(id);
        sprite.bitmap.setPixel(x, y, color);
    }

    void bitmapSetPosition(int id, int left, int top) {
        GameCanvas.Sprite sprite = sprites.get(id);
        sprite.left = left;
        sprite.top = top;
    }

    void bitmapSetZIndex(int id, int zIndex) {
        GameCanvas.Sprite sprite = sprites.get(id);
        sprite.zIndex = zIndex;
    }

    void canvasAddBitmap(int canvasId, int bitmapId) {
        GameCanvas.Sprite sprite = sprites.get(bitmapId);
        GameCanvas canvas = canvases.get(canvasId);
        canvas.sprites.add(sprite);
    }

    void canvasRedraw(int canvasId) {
        GameCanvas canvas = canvases.get(canvasId);
        canvas.invalidate();
    }

    void canvasRemoveBitmap(int canvasId, int bitmapId) {
        GameCanvas.Sprite sprite = sprites.get(bitmapId);
        GameCanvas canvas = canvases.get(canvasId);
        canvas.sprites.remove(sprite);
    }

    void canvasDeleteBitmap(int bitmapId) {
        sprites.remove(bitmapId);
    }
}
