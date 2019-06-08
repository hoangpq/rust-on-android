package com.node.sample;

import android.annotation.SuppressLint;
import android.content.Context;
import android.content.Intent;
import android.content.SharedPreferences;
import android.content.pm.PackageInfo;
import android.content.pm.PackageManager;
import android.content.res.AssetManager;
import android.graphics.Color;
import android.os.AsyncTask;
import android.os.Bundle;
import android.support.v7.app.AppCompatActivity;
import android.util.Log;
import android.view.Gravity;
import android.widget.ArrayAdapter;
import android.widget.Button;
import android.widget.ListView;
import android.widget.TextView;
import android.widget.Toast;

import com.node.util.ScriptUtils;
import com.node.v8.V8Context;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.net.URL;
import java.util.ArrayList;
import java.util.concurrent.atomic.AtomicBoolean;

import static com.node.util.RestUtil.fetch;

public class MainActivity extends AppCompatActivity {

    static {
        System.loadLibrary("native-lib");
        System.loadLibrary("node");
    }

    public native int startNodeWithArguments(String[] arguments);

    public native void initVM(Observable callbackObj);

    public native void releaseVM();

    public native String getUtf8String();

    AtomicBoolean _startedNodeAlready = new AtomicBoolean(false);
    private ListView listView;
    private ArrayList dataList;
    private ArrayAdapter adapter;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        getWindow().getDecorView().setBackgroundColor(Color.parseColor("#ffeef7f0"));

        final Button buttonVersions = findViewById(R.id.btVersions);
        final Button btnImageProcessing = findViewById(R.id.btImageProcessing);
        final TextView txtMessage = findViewById(R.id.txtMessage);
        final Button evalScriptButton = findViewById(R.id.evalScriptBtn);

        txtMessage.setText(getUtf8String());
        listView = findViewById(R.id.listView);
        dataList = new ArrayList();

        adapter = new ArrayAdapter(this, android.R.layout.simple_list_item_1, dataList);
        listView.setAdapter(adapter);

        // Init VM
        _initVM();

        evalScriptButton.setOnClickListener(view -> {
            new Thread(() -> {
                try {
                    fetch("http://127.0.0.1:8080");
                } catch (Exception e) {
                    Log.d("Java", e.getMessage());
                }
            }).start();
        });

        initNodeJS();

        // Generate image activity
        btnImageProcessing.setOnClickListener(view -> startActivity(
                new Intent(MainActivity.this, GenerateImageActivity.class)));

        buttonVersions.setOnClickListener(v -> {
            adapter.notifyDataSetChanged();
            requestApi();
        });
    }

    public void updateListView(String msg) {
        runOnUiThread(() -> {
            dataList.add(msg);
            adapter.notifyDataSetChanged();
        });
    }

    public void onNodeServerLoaded() {
        new Thread(() -> {
            try {
                V8Context context_ = V8Context.create();

                ScriptUtils.require(getApplicationContext(), context_, R.raw.core);
                ScriptUtils.require(getApplicationContext(), context_, R.raw.user);
                ScriptUtils.require(getApplicationContext(), context_, R.raw.model);

                /*ScriptUtils.bulkEval(context_,
                        "$timeout(function() { $log('$timeout 7s'); }, 7e3);",
                        "$timeout(function() { $log('$timeout 10s'); }, 1e4);");
                context_.eval("createUser('Vampire')");*/

            } catch (Exception e) {
                e.printStackTrace();
            }
        }).start();

        new Thread(V8Context::initEventLoop).start();

    }

    private void _initVM() {
        // toast watcher
        initVM(new Observable() {
            @Override
            public void subscribe(String arg) {
                runOnUiThread(() -> {
                    Toast toast = Toast.makeText(getApplicationContext(), arg, Toast.LENGTH_SHORT);
                    TextView tv = toast.getView().findViewById(android.R.id.message);
                    if (tv != null) {
                        tv.setGravity(Gravity.CENTER);
                    }
                    toast.show();
                });
            }
        });
    }

    private void initNodeJS() {
        if (!_startedNodeAlready.get()) {
            new Thread(() -> {
                try {
                    //The path where we expect the node project to be at runtime.
                    String nodeDir = getApplicationContext()
                            .getFilesDir().getAbsolutePath() + "/deps";
                    if (wasAPKUpdated()) {
                        //Recursively delete any existing deps.
                        File nodeDirReference = new File(nodeDir);
                        if (nodeDirReference.exists()) {
                            deleteFolderRecursively(new File(nodeDir));
                        }
                        //Copy the node project from assets into the application's data path.
                        copyAssetFolder(getApplicationContext()
                                .getAssets(), "deps", nodeDir);

                        saveLastUpdateTime();
                    }
                    String[] args = {"node", "--expose_gc", nodeDir + "/main.js"};
                    startNodeWithArguments(args);
                } catch (Exception e) {
                    e.printStackTrace();
                }
            }).start();
        }
    }

    @SuppressLint("StaticFieldLeak")
    private void requestApi() {
        new AsyncTask<Void, Void, String>() {
            @Override
            protected String doInBackground(Void... params) {
                StringBuilder nodeResponse = new StringBuilder();
                try {
                    URL localNodeServer = new URL("http://localhost:3000/");
                    BufferedReader in = new BufferedReader(
                            new InputStreamReader(localNodeServer.openStream()));
                    String inputLine;
                    while ((inputLine = in.readLine()) != null)
                        nodeResponse.append(inputLine);
                    in.close();
                } catch (Exception ex) {
                    nodeResponse = new StringBuilder(ex.toString());
                }
                return nodeResponse.toString();
            }

            @Override
            protected void onPostExecute(String result) {
                // textViewVersions.setText(result);
            }
        }.execute();
    }

    private boolean wasAPKUpdated() {
        SharedPreferences prefs = getApplicationContext().getSharedPreferences(
                "PREFS", Context.MODE_PRIVATE);
        long previousLastUpdateTime = prefs.getLong("LastUpdateTime", 0);
        long lastUpdateTime = 1;
        try {
            PackageInfo packageInfo = getApplicationContext().getPackageManager().getPackageInfo(
                    getApplicationContext().getPackageName(), 0);
            lastUpdateTime = packageInfo.lastUpdateTime;
        } catch (PackageManager.NameNotFoundException e) {
            e.printStackTrace();
        }
        return (lastUpdateTime != previousLastUpdateTime);
    }

    private void saveLastUpdateTime() {
        long lastUpdateTime = 1;
        try {
            PackageInfo packageInfo = getApplicationContext().getPackageManager().getPackageInfo(
                    getApplicationContext().getPackageName(), 0);
            lastUpdateTime = packageInfo.lastUpdateTime;
        } catch (PackageManager.NameNotFoundException e) {
            e.printStackTrace();
        }
        SharedPreferences prefs = getApplicationContext().getSharedPreferences(
                "PREFS", Context.MODE_PRIVATE);
        SharedPreferences.Editor editor = prefs.edit();
        editor.putLong("LastUpdateTime", lastUpdateTime);
        editor.apply();
    }

    private static boolean deleteFolderRecursively(File file) {
        try {
            boolean res = true;
            for (File childFile : file.listFiles()) {
                if (childFile.isDirectory()) {
                    res &= deleteFolderRecursively(childFile);
                } else {
                    res &= childFile.delete();
                }
            }
            res &= file.delete();
            return res;
        } catch (Exception e) {
            e.printStackTrace();
            return false;
        }
    }

    private static boolean copyAssetFolder(AssetManager assetManager,
                                           String fromAssetPath, String toPath) {
        try {
            String[] files = assetManager.list(fromAssetPath);
            boolean res = true;

            assert files != null;
            if (files.length == 0) {
                //If it's a file, it won't have any assets "inside" it.
                res = copyAsset(assetManager,
                        fromAssetPath,
                        toPath);
            } else {
                if (new File(toPath).mkdirs()) {
                    for (String file : files)
                        res &= copyAssetFolder(assetManager,
                                fromAssetPath + "/" + file,
                                toPath + "/" + file);
                }
            }
            return res;
        } catch (Exception e) {
            e.printStackTrace();
            return false;
        }
    }

    private static boolean copyAsset(AssetManager assetManager,
                                     String fromAssetPath, String toPath) {
        InputStream in;
        OutputStream out;
        try {
            in = assetManager.open(fromAssetPath);
            if (new File(toPath).createNewFile()) {
                out = new FileOutputStream(toPath);
                copyFile(in, out);
                in.close();
                out.flush();
                out.close();
            }
            return true;
        } catch (Exception e) {
            e.printStackTrace();
            return false;
        }
    }

    private static void copyFile(InputStream in, OutputStream out) throws IOException {
        byte[] buffer = new byte[1024];
        int read;
        while ((read = in.read(buffer)) != -1) {
            out.write(buffer, 0, read);
        }
    }

    @Override
    protected void onDestroy() {
        releaseVM();
        super.onDestroy();
    }

}
