package com.hy.wasmandroid;

import androidx.annotation.Keep;
import androidx.appcompat.app.AppCompatActivity;

import android.content.res.AssetManager;
import android.os.Bundle;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.ByteBuffer;
import java.util.Scanner;

public class MainActivity extends AppCompatActivity {

    static {
        System.loadLibrary("runtime");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        try {
            Wasm wasm = new Wasm(this);
            wasm.initWASM();
            wasm.runWASM();

        } catch (Exception e) {
            e.printStackTrace();
        }
    }

}
