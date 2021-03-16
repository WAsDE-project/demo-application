package com.hy.wasmandroid;

import android.content.res.AssetManager;

import androidx.annotation.Keep;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.Function;

public class Wasm {

    private static native void JNIInitializeRuntime(Wasm self, String cacheDir) throws Exception;
    private static native void JNIRunMainWASM() throws Exception;
    private static native void JNIOnTick() throws Exception;

    private MainActivity context;
    private GuiContext guiContext;

    Wasm(MainActivity context) {
        this.context = context;
        this.guiContext = new GuiContext(context);
    }

    public void initWASM() throws Exception {
        String cacheDir = context.getCacheDir().getAbsolutePath();
        JNIInitializeRuntime(this, cacheDir);


        new Thread(() -> {
            long MIN_TICK_MS = 500;
            long previousTime = System.currentTimeMillis();
            Object lock = new Object();
            AtomicBoolean ready = new AtomicBoolean(false);
            Runnable updater = () -> {
                try {
                    JNIOnTick();
                } catch (Exception e) {
                    e.printStackTrace();
                }
                synchronized(lock){
                    ready.set(true);
                    lock.notifyAll();
                }
            };
            while (true) {
                long now = System.currentTimeMillis();
                long passed = now - previousTime;

                this.context.runOnUiThread(updater);
                synchronized(lock){
                    while (!ready.get()) {
                        try {
                            lock.wait();
                        } catch (InterruptedException e) {
                            e.printStackTrace();
                        }
                    }
                    ready.set(false);
                }

                previousTime = now;
                long executionTime = System.currentTimeMillis() - now;
                if (executionTime < MIN_TICK_MS) {
                    try {
                        Thread.sleep(MIN_TICK_MS - executionTime);
                    } catch (InterruptedException e) {
                        e.printStackTrace();
                    }
                }
            }
        }).start();

    }

    public void runWASM() throws Exception {
        JNIRunMainWASM();
    }

    @Keep
    public int createTextView(String text) {
        return guiContext.createTextView(text);
    }

    @Keep
    public void removeTextView(int id) {
        guiContext.removeTextView(id);
    }

    @Keep
    public void modifyTextView(int id, String text) {
        guiContext.modifyTextView(id, text);
    }

    @Keep
    public int createButton(String label) {
        return guiContext.createButton(label);
    }

    @Keep
    public byte[] loadMetadata(String name) {
        AssetManager am = context.getAssets();
        try (InputStream inputStream = am.open(name)) {
            // Read file into byte array
            ByteArrayOutputStream baos = new ByteArrayOutputStream();
            int reads = inputStream.read();
            while(reads != -1) {
                baos.write(reads);
                reads = inputStream.read();
            }
            return baos.toByteArray();
        } catch (IOException ex) {
            System.out.println(ex);
        }
        return null;
    }

    @Keep
    public int createCanvas(int width, int height) {
        return guiContext.createCanvas(width, height);
    }
    @Keep
    public int createBitmap(int width, int height) {
        return guiContext.createBitmap(width, height);
    }
    @Keep
    public void modifyBitmap(int id, int x, int y, int color) {
        guiContext.modifyBitmap(id, x, y, color);
    }
    @Keep
    public void bitmapSetPosition(int bitmapId, int left, int top) {
        guiContext.bitmapSetPosition(bitmapId, left, top);
    }
    @Keep
    public void bitmapSetZIndex(int bitmapId, int zIndex) {
        guiContext.bitmapSetZIndex(bitmapId, zIndex);
    }
    @Keep
    public void canvasAddBitmap(int canvasId, int bitmapId) {
        guiContext.canvasAddBitmap(canvasId, bitmapId);
    }
    @Keep
    public void canvasRedraw(int canvasId) {
        guiContext.canvasRedraw(canvasId);
    }
    @Keep
    public void canvasRemoveBitmap(int canvasId, int bitmapId) {
        guiContext.canvasRemoveBitmap(canvasId, bitmapId);
    }
    @Keep
    public void canvasDeleteBitmap(int bitmapId) {
        guiContext.canvasDeleteBitmap(bitmapId);
    }
    @Keep
    public int createText(String text, int color, int text_size) {
        return guiContext.createText(text, color, text_size);
    }
    @Keep
    public void setText(int text_id, String text) {
        guiContext.setText(text_id, text);
    }
}

