#import <Cocoa/Cocoa.h>
#import <FinderSync/FinderSync.h>
#include <pwd.h>
#include <unistd.h>

static NSString * const AIOLogFileName = @"az-drive-finder-sync.log";
static NSString * const AIOHostedBadgeID = @"hosted";
static NSString * const AIOBusyBadgeID = @"busy";
static NSString * const AIOErrorBadgeID = @"error";
extern int NSExtensionMain(int argc, char **argv);

@interface FinderSync : FIFinderSync
@property(nonatomic, copy) NSArray<NSURL *> *commandURLs;
@end

static NSImage *AIOHostedBadgeImage(void);
static NSImage *AIOBadgeImage(NSString *symbolName, NSString *fallbackName, NSString *description);

static NSURL *AIOURLForPath(NSString *path, BOOL isDirectory) {
    return [NSURL fileURLWithPath:path isDirectory:isDirectory];
}

static NSString *AIORealHomeDirectory(void) {
    struct passwd *entry = getpwuid(getuid());
    if (entry != NULL && entry->pw_dir != NULL) {
        return [NSString stringWithUTF8String:entry->pw_dir];
    }
    return NSHomeDirectory();
}

static NSSet<NSURL *> *AIOManagedDirectoryURLs(void) {
    NSMutableSet<NSURL *> *urls = [NSMutableSet set];
    NSString *home = AIORealHomeDirectory();
    if (home.length > 0) {
        [urls addObject:AIOURLForPath(home, YES)];
    }
    [urls addObject:AIOURLForPath(@"/Volumes", YES)];
    return urls;
}

static NSString *AIOLogPath(void) {
    NSString *logs = [AIORealHomeDirectory() stringByAppendingPathComponent:@"Library/Logs"];
    return [logs stringByAppendingPathComponent:AIOLogFileName];
}

static NSArray<NSString *> *AIOStateFileCandidates(void) {
    NSMutableArray<NSString *> *paths = [NSMutableArray array];
    NSString *override = [NSProcessInfo processInfo].environment[@"AZ_DRIVE_STATE"];
    if (override.length > 0) {
        [paths addObject:override];
    }
    NSString *home = AIORealHomeDirectory();
    [paths addObject:[home stringByAppendingPathComponent:@"Library/Application Support/addzero/drive/state.json"]];
    [paths addObject:[home stringByAppendingPathComponent:@".config/addzero/drive/state.json"]];
    return paths;
}

static void AIOAppendLog(NSString *message) {
    NSLog(@"AIO Drive FinderSync: %@", message);
    NSString *logs = [AIOLogPath() stringByDeletingLastPathComponent];
    [[NSFileManager defaultManager] createDirectoryAtPath:logs
                              withIntermediateDirectories:YES
                                               attributes:nil
                                                    error:nil];
    NSString *line = [NSString stringWithFormat:@"%@ %@\n", [NSDate date], message];
    NSData *data = [line dataUsingEncoding:NSUTF8StringEncoding];
    if (![[NSFileManager defaultManager] fileExistsAtPath:AIOLogPath()]) {
        [data writeToFile:AIOLogPath() atomically:YES];
        return;
    }
    NSFileHandle *handle = [NSFileHandle fileHandleForWritingAtPath:AIOLogPath()];
    if (handle == nil) {
        return;
    }
    @try {
        [handle seekToEndOfFile];
        [handle writeData:data];
    } @finally {
        [handle closeFile];
    }
}

static NSString *AIOAppleScriptString(NSString *value) {
    NSString *escaped = [value stringByReplacingOccurrencesOfString:@"\\" withString:@"\\\\"];
    escaped = [escaped stringByReplacingOccurrencesOfString:@"\"" withString:@"\\\""];
    return [NSString stringWithFormat:@"\"%@\"", escaped];
}

static void AIONotify(NSString *message) {
    NSTask *task = [NSTask new];
    task.executableURL = [NSURL fileURLWithPath:@"/usr/bin/osascript"];
    NSString *script = [NSString stringWithFormat:@"display notification %@ with title \"AIO Drive\"",
                        AIOAppleScriptString(message)];
    task.arguments = @[@"-e", script];
    [task launchAndReturnError:nil];
}

static NSString *AIODriveBinaryPath(void) {
    NSString *override = [NSProcessInfo processInfo].environment[@"AZ_DRIVE_BINARY"];
    if (override.length > 0) {
        return override;
    }
    NSURL *bundleURL = [NSBundle mainBundle].bundleURL;
    NSURL *contentsURL = [[bundleURL URLByDeletingLastPathComponent] URLByDeletingLastPathComponent];
    return [[contentsURL URLByAppendingPathComponent:@"MacOS/az-drive-app"] path];
}

static NSString *AIONormalizedPath(NSString *path) {
    return path.stringByStandardizingPath;
}

static BOOL AIOPathContainsOrEquals(NSString *container, NSString *candidate) {
    NSString *left = AIONormalizedPath(container);
    NSString *right = AIONormalizedPath(candidate);
    if ([left isEqualToString:right]) {
        return YES;
    }
    NSString *prefix = [left hasSuffix:@"/"] ? left : [left stringByAppendingString:@"/"];
    return [right hasPrefix:prefix];
}

static NSArray<NSString *> *AIOStatePathsForKey(NSString *key) {
    NSFileManager *fileManager = [NSFileManager defaultManager];
    for (NSString *path in AIOStateFileCandidates()) {
        if (![fileManager fileExistsAtPath:path]) {
            continue;
        }
        NSData *data = [NSData dataWithContentsOfFile:path];
        if (data.length == 0) {
            continue;
        }
        NSError *error = nil;
        id json = [NSJSONSerialization JSONObjectWithData:data options:0 error:&error];
        if (error != nil || ![json isKindOfClass:NSDictionary.class]) {
            AIOAppendLog([NSString stringWithFormat:@"failed to read local state %@: %@", path, error]);
            return @[];
        }
        NSArray *items = ((NSDictionary *)json)[key];
        if (![items isKindOfClass:NSArray.class]) {
            return @[];
        }
        NSMutableArray<NSString *> *paths = [NSMutableArray array];
        for (id item in items) {
            if (![item isKindOfClass:NSDictionary.class]) {
                continue;
            }
            NSString *localPath = ((NSDictionary *)item)[@"local_path"];
            if ([localPath isKindOfClass:NSString.class] && localPath.length > 0) {
                [paths addObject:AIONormalizedPath(localPath)];
            }
        }
        return paths;
    }
    return @[];
}

static NSArray<NSString *> *AIOHostedPaths(void) {
    return AIOStatePathsForKey(@"hosted");
}

static NSArray<NSString *> *AIOHostedRootPaths(void) {
    return AIOStatePathsForKey(@"hosted_roots");
}

static BOOL AIOPathIsHosted(NSString *path) {
    NSString *candidate = AIONormalizedPath(path);
    for (NSString *hostedRoot in AIOHostedRootPaths()) {
        if (AIOPathContainsOrEquals(hostedRoot, candidate)) {
            return YES;
        }
    }
    for (NSString *hosted in AIOHostedPaths()) {
        if ([candidate isEqualToString:hosted]) {
            return YES;
        }
    }
    return NO;
}

static BOOL AIOURLsContainPath(NSArray<NSURL *> *urls, NSString *path) {
    NSString *candidate = AIONormalizedPath(path);
    for (NSURL *url in urls) {
        if ([AIONormalizedPath(url.path) isEqualToString:candidate]) {
            return YES;
        }
    }
    return NO;
}

static NSString *AIOURLListDescription(NSArray<NSURL *> *urls) {
    NSMutableArray<NSString *> *paths = [NSMutableArray array];
    for (NSURL *url in urls) {
        if (url.path.length > 0) {
            [paths addObject:url.path];
        }
    }
    return [paths componentsJoinedByString:@", "];
}

static NSArray<NSURL *> *AIOContextURLs(FIMenuKind menuKind) {
    FIFinderSyncController *controller = [FIFinderSyncController defaultController];
    NSArray<NSURL *> *selected = controller.selectedItemURLs;
    NSURL *targeted = controller.targetedURL;
    if (menuKind == FIMenuKindContextualMenuForItems && selected.count > 0) {
        AIOAppendLog([NSString stringWithFormat:@"context kind=%ld targeted=%@ selected=[%@] using=selected",
                      (long)menuKind,
                      targeted.path,
                      AIOURLListDescription(selected)]);
        return selected;
    }
    if (targeted != nil) {
        AIOAppendLog([NSString stringWithFormat:@"context kind=%ld targeted=%@ selected=[%@] using=targeted",
                      (long)menuKind,
                      targeted.path,
                      AIOURLListDescription(selected)]);
        return @[targeted];
    }
    if (selected.count > 0) {
        AIOAppendLog([NSString stringWithFormat:@"context kind=%ld targeted=(null) selected=[%@] using=selected",
                      (long)menuKind,
                      AIOURLListDescription(selected)]);
        return selected;
    }
    AIOAppendLog([NSString stringWithFormat:@"context kind=%ld targeted=(null) selected=[] using=empty",
                  (long)menuKind]);
    return @[];
}

@implementation FinderSync

- (instancetype)init {
    self = [super init];
    if (self != nil) {
        NSSet<NSURL *> *directories = AIOManagedDirectoryURLs();
        FIFinderSyncController *controller = [FIFinderSyncController defaultController];
        controller.directoryURLs = directories;
        [controller setBadgeImage:AIOHostedBadgeImage()
                            label:@"AIO Drive 已托管"
               forBadgeIdentifier:AIOHostedBadgeID];
        [controller setBadgeImage:AIOBadgeImage(@"arrow.triangle.2.circlepath.icloud", NSImageNameRefreshTemplate, @"AIO Drive syncing")
                            label:@"AIO Drive 同步中"
               forBadgeIdentifier:AIOBusyBadgeID];
        [controller setBadgeImage:AIOBadgeImage(@"exclamationmark.icloud", NSImageNameCaution, @"AIO Drive error")
                            label:@"AIO Drive 错误"
               forBadgeIdentifier:AIOErrorBadgeID];
        AIOAppendLog([NSString stringWithFormat:@"Finder Sync extension initialized with directories: %@",
                      directories]);
    }
    return self;
}

static NSImage *AIOHostedBadgeImage(void) {
    NSImage *image = [[NSImage alloc] initWithSize:NSMakeSize(20.0, 20.0)];
    [image lockFocus];

    [[NSColor colorWithSRGBRed:0.05 green:0.72 blue:0.28 alpha:1.0] setFill];
    NSBezierPath *circle = [NSBezierPath bezierPathWithOvalInRect:NSMakeRect(1.0, 1.0, 18.0, 18.0)];
    [circle fill];

    [[NSColor whiteColor] setStroke];
    NSBezierPath *check = [NSBezierPath bezierPath];
    check.lineWidth = 2.6;
    check.lineCapStyle = NSLineCapStyleRound;
    check.lineJoinStyle = NSLineJoinStyleRound;
    [check moveToPoint:NSMakePoint(4.7, 10.1)];
    [check lineToPoint:NSMakePoint(8.2, 6.6)];
    [check lineToPoint:NSMakePoint(14.3, 13.4)];
    [check stroke];

    NSBezierPath *arrow = [NSBezierPath bezierPath];
    arrow.lineWidth = 1.6;
    arrow.lineCapStyle = NSLineCapStyleRound;
    [arrow moveToPoint:NSMakePoint(13.8, 5.9)];
    [arrow curveToPoint:NSMakePoint(15.8, 9.1)
          controlPoint1:NSMakePoint(15.4, 6.6)
          controlPoint2:NSMakePoint(16.1, 7.8)];
    [arrow stroke];
    NSBezierPath *head = [NSBezierPath bezierPath];
    [head moveToPoint:NSMakePoint(15.9, 9.0)];
    [head lineToPoint:NSMakePoint(17.0, 6.5)];
    [head lineToPoint:NSMakePoint(14.4, 7.0)];
    [head closePath];
    [[NSColor whiteColor] setFill];
    [head fill];

    [image unlockFocus];
    image.template = NO;
    return image;
}

static NSImage *AIOBadgeImage(NSString *symbolName, NSString *fallbackName, NSString *description) {
    NSImage *image = nil;
    if (@available(macOS 11.0, *)) {
        image = [NSImage imageWithSystemSymbolName:symbolName
                         accessibilityDescription:description];
    }
    if (image == nil) {
        image = [NSImage imageNamed:fallbackName];
    }
    image.template = NO;
    return image;
}

- (NSMenu *)menuForMenuKind:(FIMenuKind)menuKind {
    if (menuKind != FIMenuKindContextualMenuForItems &&
        menuKind != FIMenuKindContextualMenuForContainer &&
        menuKind != FIMenuKindContextualMenuForSidebar) {
        return nil;
    }

    NSMenu *menu = [[NSMenu alloc] initWithTitle:@"AIO Drive"];
    NSArray<NSURL *> *urls = AIOContextURLs(menuKind);
    self.commandURLs = urls;
    __block BOOL hasHosted = NO;
    __block BOOL hasUnhosted = NO;
    [urls enumerateObjectsUsingBlock:^(NSURL *url, NSUInteger index, BOOL *stop) {
        (void)index;
        BOOL hosted = AIOPathIsHosted(url.path);
        hasHosted = hasHosted || hosted;
        hasUnhosted = hasUnhosted || !hosted;
        if (hasHosted && hasUnhosted) {
            *stop = YES;
        }
    }];
    AIOAppendLog([NSString stringWithFormat:@"menu kind=%ld urls=[%@] hasHosted=%@ hasUnhosted=%@",
                  (long)menuKind,
                  AIOURLListDescription(urls),
                  hasHosted ? @"YES" : @"NO",
                  hasUnhosted ? @"YES" : @"NO"]);

    if (hasUnhosted) {
        NSMenuItem *host = [[NSMenuItem alloc] initWithTitle:@"AIO Drive 托管"
                                                      action:@selector(hostSelected:)
                                               keyEquivalent:@""];
        host.target = self;
        [menu addItem:host];
    }

    if (hasHosted) {
        NSMenuItem *unhost = [[NSMenuItem alloc] initWithTitle:@"AIO Drive 取消托管"
                                                        action:@selector(unhostSelected:)
                                                 keyEquivalent:@""];
        unhost.target = self;
        [menu addItem:unhost];
    }
    if (menu.itemArray.count == 0) {
        return nil;
    }
    return menu;
}

- (void)requestBadgeIdentifierForURL:(NSURL *)url {
    NSString *badge = AIOPathIsHosted(url.path) ? AIOHostedBadgeID : @"";
    [[FIFinderSyncController defaultController] setBadgeIdentifier:badge forURL:url];
}

- (void)hostSelected:(id)sender {
    (void)sender;
    [self runDriveCommand:@"host" label:@"托管"];
}

- (void)unhostSelected:(id)sender {
    (void)sender;
    [self runDriveCommand:@"unhost" label:@"取消托管"];
}

- (void)runDriveCommand:(NSString *)command label:(NSString *)label {
    NSArray<NSURL *> *urls = self.commandURLs.count > 0 ? self.commandURLs : AIOContextURLs(FIMenuKindContextualMenuForItems);
    if (urls.count == 0) {
        AIONotify(@"Finder 没有传入选中文件");
        AIOAppendLog([NSString stringWithFormat:@"%@ skipped: no selected URL", label]);
        return;
    }

    NSString *binary = AIODriveBinaryPath();
    if (![[NSFileManager defaultManager] isExecutableFileAtPath:binary]) {
        AIONotify(@"AIO Drive CLI 不存在或不可执行");
        AIOAppendLog([NSString stringWithFormat:@"%@ failed: missing executable %@", label, binary]);
        return;
    }

    NSString *home = AIORealHomeDirectory();
    if ([command isEqualToString:@"host"]) {
        NSMutableArray<NSURL *> *allowed = [NSMutableArray array];
        for (NSURL *url in urls) {
            if ([AIONormalizedPath(url.path) isEqualToString:AIONormalizedPath(home)]) {
                AIOAppendLog(@"托管 skipped: refusing to host the whole home directory from Finder");
                AIONotify(@"请托管具体文件或子目录，不要直接托管整个家目录");
            } else {
                [allowed addObject:url];
            }
        }
        urls = allowed;
        if (urls.count == 0) {
            return;
        }
    }

    for (NSURL *url in urls) {
        [[FIFinderSyncController defaultController] setBadgeIdentifier:AIOBusyBadgeID forURL:url];
    }
    AIONotify([NSString stringWithFormat:@"%@开始", label]);

    dispatch_async(dispatch_get_global_queue(QOS_CLASS_UTILITY, 0), ^{
        __block BOOL failed = NO;
        [urls enumerateObjectsUsingBlock:^(NSURL *url, NSUInteger index, BOOL *stop) {
            (void)index;
            (void)stop;
            NSTask *task = [NSTask new];
            task.executableURL = [NSURL fileURLWithPath:binary];
            task.arguments = @[command, url.path];
            NSMutableDictionary<NSString *, NSString *> *environment =
                [NSProcessInfo.processInfo.environment mutableCopy];
            environment[@"HOME"] = AIORealHomeDirectory();
            task.environment = environment;
            NSPipe *pipe = [NSPipe pipe];
            task.standardOutput = pipe;
            task.standardError = pipe;
            NSError *launchError = nil;
            AIOAppendLog([NSString stringWithFormat:@"%@ %@ %@", label, command, url.path]);
            if (![task launchAndReturnError:&launchError]) {
                failed = YES;
                AIOAppendLog([NSString stringWithFormat:@"%@ launch failed: %@", label, launchError]);
                return;
            }
            [task waitUntilExit];
            NSData *output = [[pipe fileHandleForReading] readDataToEndOfFile];
            if (output.length > 0) {
                NSString *text = [[NSString alloc] initWithData:output encoding:NSUTF8StringEncoding];
                if (text.length > 0) {
                    AIOAppendLog(text);
                }
            }
            if (task.terminationStatus != 0) {
                failed = YES;
                AIOAppendLog([NSString stringWithFormat:@"%@ failed with status %d",
                              label,
                              task.terminationStatus]);
            }
        }];
        dispatch_async(dispatch_get_main_queue(), ^{
            NSString *badge = failed ? AIOErrorBadgeID : ([command isEqualToString:@"host"] ? AIOHostedBadgeID : @"");
            for (NSURL *url in urls) {
                [[FIFinderSyncController defaultController] setBadgeIdentifier:badge forURL:url];
            }
        });
        AIONotify(failed ? [NSString stringWithFormat:@"%@失败，查看 ~/Library/Logs/%@", label, AIOLogFileName]
                         : [NSString stringWithFormat:@"%@完成", label]);
    });
}

@end

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        return NSExtensionMain(argc, (char **)argv);
    }
}
