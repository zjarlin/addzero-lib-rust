#import <Cocoa/Cocoa.h>
#import <FinderSync/FinderSync.h>

static NSString * const AIOFinderExtensionID = @"site.addzero.drive.findersync";
static NSString * const AIOFinderExtensionName = @"AIODriveFinderSync";
static NSString * const AIOAppName = @"AIO Drive Finder.app";

static NSString *AIORunTool(NSString *launchPath, NSArray<NSString *> *arguments, int *status) {
    NSTask *task = [NSTask new];
    task.executableURL = [NSURL fileURLWithPath:launchPath];
    task.arguments = arguments;
    NSPipe *pipe = [NSPipe pipe];
    task.standardOutput = pipe;
    task.standardError = pipe;

    NSError *error = nil;
    if (![task launchAndReturnError:&error]) {
        if (status != NULL) {
            *status = -1;
        }
        return error.localizedDescription ?: @"launch failed";
    }

    [task waitUntilExit];
    if (status != NULL) {
        *status = task.terminationStatus;
    }
    NSData *data = [[pipe fileHandleForReading] readDataToEndOfFile];
    if (data.length == 0) {
        return @"";
    }
    NSString *text = [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
    return [text stringByTrimmingCharactersInSet:NSCharacterSet.whitespaceAndNewlineCharacterSet] ?: @"";
}

static BOOL AIORunToolExpectSuccess(NSString *launchPath,
                                    NSArray<NSString *> *arguments,
                                    NSMutableArray<NSString *> *notes,
                                    NSString *successNote,
                                    NSString *failurePrefix) {
    int status = 0;
    NSString *output = AIORunTool(launchPath, arguments, &status);
    if (status == 0) {
        if (successNote.length > 0) {
            [notes addObject:successNote];
        }
        return YES;
    }
    NSString *detail = output.length > 0 ? output : [NSString stringWithFormat:@"exit %d", status];
    [notes addObject:[NSString stringWithFormat:@"%@：%@", failurePrefix, detail]];
    return NO;
}

static NSString *AIOBundlePath(void) {
    return NSBundle.mainBundle.bundlePath;
}

static NSString *AIOExtensionPath(void) {
    return [AIOBundlePath() stringByAppendingPathComponent:
            [NSString stringWithFormat:@"Contents/PlugIns/%@.appex", AIOFinderExtensionName]];
}

static NSString *AIODriveBinaryPath(void) {
    return [AIOBundlePath() stringByAppendingPathComponent:@"Contents/MacOS/az-drive-app"];
}

static BOOL AIOInstallQuickActions(NSMutableArray<NSString *> *notes) {
    NSString *binary = AIODriveBinaryPath();
    if (![[NSFileManager defaultManager] isExecutableFileAtPath:binary]) {
        [notes addObject:@"内置 az-drive-app 不存在，无法安装 Finder 快速操作"];
        return NO;
    }
    return AIORunToolExpectSuccess(binary,
                                   @[@"install-macos-actions"],
                                   notes,
                                   @"已安装 Finder 右键快速操作",
                                   @"安装 Finder 右键快速操作失败");
}

static BOOL AIORegisterFinderExtension(NSMutableArray<NSString *> *notes) {
    NSString *extensionPath = AIOExtensionPath();
    if (![[NSFileManager defaultManager] fileExistsAtPath:extensionPath]) {
        [notes addObject:@"Finder 扩展缺失，无法注册右键菜单和状态图标"];
        return NO;
    }

    NSString *lsregister =
        @"/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister";
    AIORunTool(lsregister, @[@"-f", AIOBundlePath()], NULL);

    NSString *homeAppsPath = [NSHomeDirectory() stringByAppendingPathComponent:
                              [NSString stringWithFormat:@"Applications/%@/Contents/PlugIns/%@.appex",
                               AIOAppName,
                               AIOFinderExtensionName]];
    NSString *systemAppsPath = [@"/Applications" stringByAppendingPathComponent:
                                [NSString stringWithFormat:@"%@/Contents/PlugIns/%@.appex",
                                 AIOAppName,
                                 AIOFinderExtensionName]];
    AIORunTool(@"/usr/bin/pluginkit", @[@"-r", extensionPath], NULL);
    AIORunTool(@"/usr/bin/pluginkit", @[@"-r", homeAppsPath], NULL);
    AIORunTool(@"/usr/bin/pluginkit", @[@"-r", systemAppsPath], NULL);

    BOOL added = AIORunToolExpectSuccess(@"/usr/bin/pluginkit",
                                         @[@"-a", extensionPath],
                                         notes,
                                         @"已注册 Finder 扩展到系统插件数据库",
                                         @"注册 Finder 扩展失败");
    BOOL enabled = AIORunToolExpectSuccess(@"/usr/bin/pluginkit",
                                           @[@"-e", @"use", @"-i", AIOFinderExtensionID],
                                           notes,
                                           @"已请求启用 Finder 扩展",
                                           @"启用 Finder 扩展失败");
    return added && enabled;
}

static BOOL AIORestartFinder(NSMutableArray<NSString *> *notes) {
    int status = 0;
    NSString *output = AIORunTool(@"/usr/bin/killall", @[@"Finder"], &status);
    if (status == 0) {
        [notes addObject:@"已请求重启 Finder 以加载新菜单和状态图标"];
        return YES;
    }
    NSString *detail = output.length > 0 ? output : [NSString stringWithFormat:@"exit %d", status];
    [notes addObject:[NSString stringWithFormat:@"重启 Finder 失败：%@", detail]];
    return NO;
}

static void AIOOpenFullDiskAccessSettings(void) {
    NSArray<NSString *> *urls = @[
        @"x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension",
        @"x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles"
    ];
    for (NSString *candidate in urls) {
        NSURL *url = [NSURL URLWithString:candidate];
        if (url != nil && [NSWorkspace.sharedWorkspace openURL:url]) {
            break;
        }
    }
}

static void AIOOpenExtensionsSettings(void) {
    NSArray<NSString *> *urls = @[
        @"x-apple.systempreferences:com.apple.LoginItems-Settings.extension",
        @"x-apple.systempreferences:com.apple.ExtensionsPreferences"
    ];
    for (NSString *candidate in urls) {
        NSURL *url = [NSURL URLWithString:candidate];
        if (url != nil && [NSWorkspace.sharedWorkspace openURL:url]) {
            return;
        }
    }
    if (@available(macOS 10.14, *)) {
        [FIFinderSyncController showExtensionManagementInterface];
    }
}

static NSString *AIOInstallLocationNote(void) {
    NSString *bundlePath = AIOBundlePath();
    if ([bundlePath hasPrefix:@"/Applications/"] ||
        [bundlePath hasPrefix:[NSHomeDirectory() stringByAppendingPathComponent:@"Applications/"]]) {
        return @"当前 app 已位于 Applications 目录";
    }
    return @"建议先把 app 拖到 /Applications 或 ~/Applications 再打开";
}

@interface AppDelegate : NSObject <NSApplicationDelegate>
@end

@implementation AppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)notification {
    (void)notification;

    [NSApp activateIgnoringOtherApps:YES];

    NSMutableArray<NSString *> *notes = [NSMutableArray array];
    [notes addObject:AIOInstallLocationNote()];
    BOOL installedQuickActions = AIOInstallQuickActions(notes);
    BOOL registeredExtension = AIORegisterFinderExtension(notes);
    BOOL restartedFinder = NO;
    if (installedQuickActions || registeredExtension) {
        restartedFinder = AIORestartFinder(notes);
    }

    NSMutableArray<NSString *> *instructions = [NSMutableArray array];
    [instructions addObject:@"如果 Finder 里还没有 “AIO Drive 托管”，先点“打开扩展设置”；系统会打开设置页，你需要手动确认 AIO Drive Finder 已启用。"];
    [instructions addObject:@"macOS 不会自动弹完全磁盘访问授权；点“打开完全磁盘访问”后会打开‘隐私与安全性’，你需要手动把 AIO Drive Finder 加进去。"];
    [instructions addObject:@"然后回 Finder 里重新右键文件或目录测试。"];

    NSAlert *alert = [NSAlert new];
    alert.alertStyle = (installedQuickActions || registeredExtension) ? NSAlertStyleInformational : NSAlertStyleWarning;
    alert.messageText = (installedQuickActions || registeredExtension)
        ? @"AIO Drive 已完成本机 Finder 集成安装"
        : @"AIO Drive 没有完成 Finder 集成安装";

    NSString *status = [notes componentsJoinedByString:@"\n- "];
    NSString *guide = [instructions componentsJoinedByString:@"\n"];
    alert.informativeText = [NSString stringWithFormat:@"状态：\n- %@\n\n后续：\n%@",
                             status,
                             guide];

    [alert addButtonWithTitle:@"打开扩展设置"];
    [alert addButtonWithTitle:@"打开完全磁盘访问"];
    if (restartedFinder) {
        [alert addButtonWithTitle:@"完成"];
    } else {
        [alert addButtonWithTitle:@"退出"];
    }

    NSModalResponse response = [alert runModal];
    if (response == NSAlertFirstButtonReturn) {
        AIOOpenExtensionsSettings();
        [NSThread sleepForTimeInterval:1.0];
    } else if (response == NSAlertSecondButtonReturn) {
        AIOOpenFullDiskAccessSettings();
        [NSThread sleepForTimeInterval:1.0];
    }

    [NSApp terminate:nil];
}

@end

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        NSApplication *application = [NSApplication sharedApplication];
        AppDelegate *delegate = [AppDelegate new];
        application.delegate = delegate;
        return NSApplicationMain(argc, argv);
    }
}
