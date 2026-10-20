# Add project specific ProGuard rules here.
# You can control the set of applied configuration files using the
# proguardFiles setting in build.gradle.
#
# For more details, see
#   http://developer.android.com/guide/developing/tools/proguard.html

# If your project uses WebView with JS, uncomment the following
# and specify the fully qualified class name to the JavaScript interface
# class:
#-keepclassmembers class fqcn.of.javascript.interface.for.webview {
#   public *;
#}

# Uncomment this to preserve the line number information for
# debugging stack traces.
#-keepattributes SourceFile,LineNumberTable

# If you keep the line number information, uncomment this to
# hide the original source file name.
#-renamesourcefileattribute SourceFile

# 原生提醒的类名用于 Tauri 反射和 JNI 符号匹配，发布压缩时保持稳定。
-keep class com.peroperochat.mobile.ReminderBridge { *; }
-keep class com.peroperochat.mobile.ReminderPlugin { *; }
-keep class com.peroperochat.mobile.ReminderSyncArgs { *; }
# 远程凭据插件通过 Tauri 反射调用，压缩后仍须保留参数与命令。
-keep class com.peroperochat.mobile.RemoteVaultPlugin { *; }
-keep class com.peroperochat.mobile.RemoteSecretArgs { *; }
# 无界面通知通过 JNI 回调，接收桥与回调方法不能被重命名。
-keep class com.peroperochat.mobile.RemoteReceiverBridge { *; }
-keep class com.peroperochat.mobile.RemoteReceiverPlugin { *; }
-keep class com.peroperochat.mobile.RemoteReceiverArgs { *; }
-keep interface com.peroperochat.mobile.RemoteReceiverCallback { *; }
-keepclassmembers class * implements com.peroperochat.mobile.RemoteReceiverCallback { *; }
# 文件保存依赖 Tauri 参数反射、Activity 回调和 JNI 模式守卫。
-keep class com.peroperochat.mobile.RemoteFilesPlugin { *; }
-keep class com.peroperochat.mobile.RemoteExportArgs { *; }
-keep class com.peroperochat.mobile.RemoteSaveArgs { *; }
-keep class com.peroperochat.mobile.RemoteCancelArgs { *; }
