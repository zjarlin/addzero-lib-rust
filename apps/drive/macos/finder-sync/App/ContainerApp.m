#import <Cocoa/Cocoa.h>
#import <FinderSync/FinderSync.h>

@interface AppDelegate : NSObject <NSApplicationDelegate>
@end

@implementation AppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)notification {
    (void)notification;
    if (@available(macOS 10.14, *)) {
        [FIFinderSyncController showExtensionManagementInterface];
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
