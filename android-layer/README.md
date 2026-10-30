# android-layer/ (Lay-Android)

Runtime de compatibilidade com aplicativos Android (APK release e debug, NDK/JNI, Qt for Android).

Planejado:
- `runtime/`: ART, Bionic e framework reduzido em namespaces Linux
- `binder/`: configuracao do Binder e servicemanager
- `window-bridge/`: superficies Android -> Wayland
- `input-bridge/`: touch, teclado, gamepad, sensores
- `hal-bridge/`: camera, audio, GPU, GPS
- `adbd/`: `lay-adbd`, para depurar pelo Android Studio e Qt Creator

Ver `docs/ANDROID_COMPAT.md`.
