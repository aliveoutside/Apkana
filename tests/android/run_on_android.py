#!/usr/bin/env python3
"""Compare the original split set and merged APK on an explicit Android target.
Installs only dev.apkana.mergefixture; never clears global logs or changes another app.
"""
import argparse, os, pathlib, subprocess, tempfile, time, zipfile
parser = argparse.ArgumentParser()
parser.add_argument('--serial', required=True, help='ADB serial of a dedicated test target')
args = parser.parse_args()
root = pathlib.Path(__file__).resolve().parents[2]
package = 'dev.apkana.mergefixture'

def adb(*words):
    try:
        return subprocess.check_output(['adb', '-s', args.serial, *words], text=True, stderr=subprocess.STDOUT)
    except subprocess.CalledProcessError as e:
        raise RuntimeError(e.output) from e

def remove_previous_fixture():
    # Source and merged APKs have different public test signing keys.
    if 'package:'+package in adb('shell', 'pm', 'list', 'packages', package):
        print(adb('uninstall', package))

def launch_and_assert(label):
    print(adb('shell', 'am', 'start', '-W', '-n', package+'/.MainActivity'))
    deadline = time.monotonic()+20
    required = ['APKANA_MERGE_OK','PROVIDER_OK','SERVICE_OK','RECEIVER_OK']
    while time.monotonic() < deadline:
        pid = adb('shell', 'pidof', package).strip()
        if not pid:
            raise RuntimeError('fixture exited during startup')
        logs = adb('logcat', '-d', '--pid='+pid, '-s', 'ApkanaFixture:I', 'AndroidRuntime:E')
        if all(marker in logs for marker in required):
            activity = adb('shell', 'dumpsys', 'activity', 'activities')
            if package+'/.FeatureActivity' not in activity:
                raise RuntimeError('feature Activity is not running')
            print(logs)
            print('PASS '+label+': Activity/Service/Receiver/Provider, language/DPI resources, assets and JNI')
            return
        time.sleep(.5)
    raise RuntimeError('missing expected fixture markers: '+logs)

with tempfile.TemporaryDirectory(prefix='apkana-android-') as tmp:
    output = str(pathlib.Path(tmp)/'merged.apk')
    env = dict(os.environ, APKANA_ANDROID_TEST_OUTPUT=output)
    subprocess.run(['cargo', 'test', '--locked', '--test', 'merge_archives', 'real_archives', '--', '--ignored'], cwd=root, env=env, check=True)
    inputs = []
    with zipfile.ZipFile(root/'tests/fixtures/feature.apks') as archive:
        for i, name in enumerate(archive.namelist()):
            path = pathlib.Path(tmp)/f'input-{i}.apk'
            path.write_bytes(archive.read(name))
            inputs.append(str(path))
    remove_previous_fixture()
    print(adb('install-multiple', *inputs))
    launch_and_assert('original split set')
    remove_previous_fixture()
    print(adb('install', output))
    launch_and_assert('merged standalone APK')
