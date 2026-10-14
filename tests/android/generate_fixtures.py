#!/usr/bin/env python3
"""Build real signed binary APK archives using Android SDK tools (no Gradle/network).
Fixtures are our own MIT-licensed app; no third-party application/code is included.
ANDROID_JAR and D8_JAR may override the SDK paths. Requires aapt2, javac, java,
keytool, apksigner and clang/lld on PATH. Run from the repository root.
"""
import os, pathlib, subprocess, tempfile, zipfile
ROOT = pathlib.Path(__file__).resolve().parents[2]
SDK = pathlib.Path(os.environ.get('ANDROID_HOME', '/home/oof/Android/Sdk'))
ANDROID = os.environ.get('ANDROID_JAR', str(SDK / 'platforms/android-36/android.jar'))
D8 = os.environ.get('D8_JAR', '/opt/android-sdk/build-tools/37.0.0/lib/d8.jar')

def run(*args):
    subprocess.run(list(map(str, args)), check=True)

def write(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)

with tempfile.TemporaryDirectory(prefix='apkana-fixture-') as temp:
    w = pathlib.Path(temp)
    key = w/'test.jks'
    run('keytool', '-genkeypair', '-keystore', key, '-storepass', 'android', '-keypass', 'android', '-alias', 'test', '-keyalg', 'RSA', '-validity', '10000', '-dname', 'CN=Apkana Integration Fixture')
    package = 'dev.apkana.mergefixture'
    xmlns = 'xmlns:android="http://schemas.android.com/apk/res/android"'
    write(w/'base.xml', f'''<manifest {xmlns} package="{package}" android:versionCode="1" android:versionName="1" android:requiredSplitTypes="base__abi">
    <uses-sdk android:minSdkVersion="23" android:targetSdkVersion="35"/>
    <application android:label="@string/title" android:extractNativeLibs="false">
    <activity android:name="{package}.MainActivity" android:exported="true"><intent-filter><action android:name="android.intent.action.MAIN"/><category android:name="android.intent.category.LAUNCHER"/></intent-filter></activity>
    </application></manifest>''')
    write(w/'res/values/strings.xml', '<resources><string name="title">Merge fixture</string><string name="message">base-default</string></resources>')
    write(w/'res/values-fr/strings.xml', '<resources><string name="message">bonjour-feature</string></resources>')
    write(w/'res/drawable-hdpi/probe.xml', '<shape xmlns:android="http://schemas.android.com/apk/res/android" android:shape="rectangle"><solid android:color="#ff0000"/></shape>')
    write(w/'res/drawable-xhdpi/probe.xml', '<shape xmlns:android="http://schemas.android.com/apk/res/android" android:shape="rectangle"><solid android:color="#00ff00"/></shape>')
    run('aapt2', 'compile', '--dir', w/'res', '-o', w/'compiled.zip')
    run('aapt2', 'link', '-o', w/'base.apk', '--manifest', w/'base.xml', '-I', ANDROID, '--java', w/'java', '--split', f'{w}/fr.apk:fr', '--split', f'{w}/density.apk:hdpi,xhdpi', w/'compiled.zip')
    # A feature's own resource package exercises global-string remapping and
    # importing a package absent from the base table.
    write(w/'feature.xml', f'''<manifest {xmlns} package="{package}" split="camera" android:isFeatureSplit="true" android:versionCode="1">
    <uses-sdk android:minSdkVersion="23" android:targetSdkVersion="35"/>
    <uses-permission android:name="android.permission.INTERNET"/>
    <application android:hasCode="true"><activity android:name="{package}.FeatureActivity" android:exported="true"/><service android:name="{package}.FeatureService" android:exported="false"/><receiver android:name="{package}.FeatureReceiver" android:exported="false"/><provider android:name="{package}.FeatureProvider" android:authorities="{package}.provider" android:exported="false"/></application></manifest>''')
    write(w/'feature-res/values/strings.xml', '<resources><string name="feature_message">feature-resource-ok</string></resources>')
    run('aapt2', 'compile', '--dir', w/'feature-res', '-o', w/'feature.zip')
    run('aapt2', 'link', '-o', w/'feature.apk', '--package-id', '0x80', '--manifest', w/'feature.xml', '-I', ANDROID, '--java', w/'feature-java', '--custom-package', package+'.feature', w/'feature.zip')
    write(w/'java/dev/apkana/mergefixture/MainActivity.java', '''package dev.apkana.mergefixture;
public class MainActivity extends android.app.Activity {
 public void onCreate(android.os.Bundle b) { super.onCreate(b); android.content.res.Configuration c = new android.content.res.Configuration(getResources().getConfiguration()); c.setLocale(java.util.Locale.FRENCH); String text = createConfigurationContext(c).getString(R.string.message); if (!text.equals("bonjour-feature")) throw new AssertionError(text); getResources().getDrawable(R.drawable.probe, null); startActivity(new android.content.Intent(this, FeatureActivity.class)); }
}''')
    write(w/'feature-java/dev/apkana/mergefixture/FeatureActivity.java', '''package dev.apkana.mergefixture;
public class FeatureActivity extends android.app.Activity {
 private static native int probe();
 public void onCreate(android.os.Bundle b) { super.onCreate(b); try { for (String name : new String[]{"base", "feature"}) { java.io.InputStream input = getAssets().open(name+".txt"); byte[] bytes = new byte[64]; int n = input.read(bytes); input.close(); if (!new String(bytes,0,n,"UTF-8").equals(name+"-asset")) throw new AssertionError("asset "+name); } } catch (java.io.IOException e) { throw new RuntimeException(e); } System.loadLibrary("probe"); if (probe()!=42) throw new AssertionError("native"); String resource = getString(dev.apkana.mergefixture.feature.R.string.feature_message); if (!resource.equals("feature-resource-ok")) throw new AssertionError(resource); getContentResolver().query(android.net.Uri.parse("content://dev.apkana.mergefixture.provider/probe"), null, null, null, null); startService(new android.content.Intent(this, FeatureService.class)); sendBroadcast(new android.content.Intent(this, FeatureReceiver.class)); android.widget.TextView v = new android.widget.TextView(this); v.setText("APKANA_MERGE_OK "+resource); setContentView(v); android.util.Log.i("ApkanaFixture", "APKANA_MERGE_OK"); }
}''')
    write(w/'feature-java/dev/apkana/mergefixture/FeatureService.java', '''package dev.apkana.mergefixture; public class FeatureService extends android.app.Service { public android.os.IBinder onBind(android.content.Intent i) {return null;} public int onStartCommand(android.content.Intent i,int f,int s){android.util.Log.i("ApkanaFixture","SERVICE_OK");stopSelf();return START_NOT_STICKY;} }''')
    write(w/'feature-java/dev/apkana/mergefixture/FeatureReceiver.java', '''package dev.apkana.mergefixture; public class FeatureReceiver extends android.content.BroadcastReceiver {public void onReceive(android.content.Context c,android.content.Intent i){android.util.Log.i("ApkanaFixture","RECEIVER_OK");}}''')
    write(w/'feature-java/dev/apkana/mergefixture/FeatureProvider.java', '''package dev.apkana.mergefixture; public class FeatureProvider extends android.content.ContentProvider {public boolean onCreate(){return true;} public android.database.Cursor query(android.net.Uri u,String[] p,String s,String[] a,String o){android.util.Log.i("ApkanaFixture","PROVIDER_OK");return null;}public String getType(android.net.Uri u){return null;}public android.net.Uri insert(android.net.Uri u,android.content.ContentValues v){return null;}public int delete(android.net.Uri u,String s,String[] a){return 0;}public int update(android.net.Uri u,android.content.ContentValues v,String s,String[] a){return 0;}}''')
    sources = list((w/'java').rglob('*.java')) + list((w/'feature-java').rglob('*.java'))
    run('javac', '-source', '8', '-target', '8', '-bootclasspath', ANDROID, '-d', w/'classes', *sources)
    for name in ['base', 'feature']:
        dexdir = w/(name+'-dex'); dexdir.mkdir()
        files = list((w/'classes').rglob('*.class'))
        files = [f for f in files if (f.name.startswith('Feature') or '/feature/' in str(f)) == (name == 'feature')]
        run('java', '-cp', D8, 'com.android.tools.r8.D8', '--lib', ANDROID, '--min-api', '23', '--output', dexdir, *files)
        with zipfile.ZipFile(w/(name+'.apk'), 'a') as z:
            z.write(dexdir/'classes.dex', 'classes.dex')
            z.writestr('assets/'+name+'.txt', name+'-asset')
    write(w/'abi.xml', f'<manifest {xmlns} package="{package}" split="config.x86_64" android:splitTypes="base__abi" android:versionCode="1"><uses-sdk android:minSdkVersion="23" android:targetSdkVersion="35"/><application android:hasCode="false"/></manifest>')
    run('aapt2', 'link', '-o', w/'abi.apk', '--manifest', w/'abi.xml', '-I', ANDROID)
    write(w/'probe.c', 'int Java_dev_apkana_mergefixture_FeatureActivity_probe(void *env, void *object) { return 42; }')
    run('clang', '--target=x86_64-linux-android23', '-fuse-ld=lld', '-nostdlib', '-shared', '-Wl,-soname,libprobe.so', '-o', w/'libprobe.so', w/'probe.c')
    with zipfile.ZipFile(w/'abi.apk', 'a') as z: z.write(w/'libprobe.so', 'lib/x86_64/libprobe.so')
    for name in ['base', 'feature', 'fr', 'density', 'abi']:
        run('zipalign', '-f', '-P', '16', '4', w/(name+'.apk'), w/(name+'-aligned.apk'))
        (w/(name+'-aligned.apk')).replace(w/(name+'.apk'))
        run('apksigner', 'sign', '--ks', key, '--ks-pass', 'pass:android', '--ks-key-alias', 'test', w/(name+'.apk'))
    dest = ROOT/'tests/fixtures'; dest.mkdir(exist_ok=True)
    # Deliberately misleading names exercise metadata-based base and split lookup.
    for ext in ['apks','xapk','apkm']:
        with zipfile.ZipFile(dest/('feature.'+ext), 'w', zipfile.ZIP_DEFLATED) as archive:
            for name, filename in [('base','misleading_config.fr.apk'), ('feature','base.apk'), ('fr','camera.apk'), ('density','whatever.apk'), ('abi','language.apk')]:
                archive.write(w/(name+'.apk'), filename)
    with zipfile.ZipFile(dest/'variants.apks', 'w', zipfile.ZIP_DEFLATED) as archive:
        archive.write(w/'base.apk', 'one.apk'); archive.write(w/'base.apk', 'two.apk')
