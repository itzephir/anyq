#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>

@interface CGVirtualDisplayDescriptor : NSObject
@property unsigned int vendorID, productID, serialNum, serialNumber, maxPixelsWide, maxPixelsHigh;
@property NSString *name;
@property CGSize sizeInMillimeters;
@property CGPoint whitePoint, redPrimary, greenPrimary, bluePrimary;
@property dispatch_queue_t queue;
@end

@interface CGVirtualDisplayMode : NSObject
- (id)initWithWidth:(unsigned int)width height:(unsigned int)height refreshRate:(double)rate;
@end

@interface CGVirtualDisplaySettings : NSObject
@property NSArray *modes;
@property unsigned int hiDPI;
@end

@interface CGVirtualDisplay : NSObject
@property(readonly) unsigned int displayID;
- (id)initWithDescriptor:(CGVirtualDisplayDescriptor *)descriptor;
- (BOOL)applySettings:(CGVirtualDisplaySettings *)settings;
@end

static NSMutableArray *displays;

static NSArray *windows(pid_t pid) {
    NSArray *all = CFBridgingRelease(CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, kCGNullWindowID));
    NSMutableArray *result = [NSMutableArray array];
    for (NSDictionary *window in all) {
        if ([window[(id)kCGWindowOwnerPID] intValue] != pid) continue;
        NSDictionary *bounds = window[(id)kCGWindowBounds];
        [result addObject:@{ @"x":bounds[@"X"], @"y":bounds[@"Y"], @"width":bounds[@"Width"], @"height":bounds[@"Height"], @"layer":window[(id)kCGWindowLayer] }];
    }
    return result;
}

static void key(CGKeyCode code, CGEventFlags flags) {
    for (int down = 1; down >= 0; down--) {
        CGEventRef event = CGEventCreateKeyboardEvent(NULL, code, down);
        CGEventSetFlags(event, flags);
        CGEventPost(kCGHIDEventTap, event);
        CFRelease(event);
    }
}

static id perform(NSDictionary *command) {
    NSString *action = command[@"action"];
    if ([action isEqual:@"capabilities"]) return @{ @"input":@(CGPreflightPostEventAccess()), @"capture":@(CGPreflightScreenCaptureAccess()), @"screens":@(NSScreen.screens.count) };
    if ([action isEqual:@"windows"]) return windows([command[@"pid"] intValue]);
    if ([action isEqual:@"tray"]) {
        for (NSDictionary *window in windows([command[@"pid"] intValue])) {
            if ([window[@"layer"] intValue] >= 20 && [window[@"width"] intValue] <= 100 && [window[@"height"] intValue] <= 100) return window;
        }
        return [NSNull null];
    }
    if ([action isEqual:@"monitors"]) {
        NSMutableArray *result = [NSMutableArray array];
        double top = NSScreen.screens.firstObject.frame.size.height;
        for (NSScreen *screen in NSScreen.screens) {
            NSRect rect = screen.visibleFrame;
            [result addObject:@{ @"x":@(rect.origin.x), @"y":@(top - rect.origin.y - rect.size.height), @"width":@(rect.size.width), @"height":@(rect.size.height), @"scale":@(screen.backingScaleFactor) }];
        }
        return result;
    }
    if ([action isEqual:@"displays"]) {
        displays = [NSMutableArray array];
        for (NSDictionary *spec in command[@"displays"]) {
            unsigned int width = [spec[@"width"] unsignedIntValue];
            unsigned int height = [spec[@"height"] unsignedIntValue];
            unsigned int scale = [spec[@"scale"] unsignedIntValue];
            CGVirtualDisplayDescriptor *descriptor = [CGVirtualDisplayDescriptor new];
            descriptor.queue = dispatch_get_global_queue(QOS_CLASS_USER_INITIATED, 0);
            descriptor.name = @"anyq CI display";
            descriptor.vendorID = 505;
            descriptor.productID = 1;
            descriptor.serialNum = descriptor.serialNumber = (unsigned int)displays.count + 1;
            descriptor.maxPixelsWide = width * scale;
            descriptor.maxPixelsHigh = height * scale;
            descriptor.sizeInMillimeters = CGSizeMake(width * 0.25, height * 0.25);
            descriptor.whitePoint = CGPointMake(0.3125, 0.3291);
            descriptor.redPrimary = CGPointMake(0.6797, 0.3203);
            descriptor.greenPrimary = CGPointMake(0.2559, 0.6983);
            descriptor.bluePrimary = CGPointMake(0.1494, 0.0557);
            CGVirtualDisplay *display = [[CGVirtualDisplay alloc] initWithDescriptor:descriptor];
            NSCAssert(display != nil, @"Virtual display creation failed");
            CGVirtualDisplaySettings *settings = [CGVirtualDisplaySettings new];
            settings.hiDPI = scale == 2;
            settings.modes = @[[[CGVirtualDisplayMode alloc] initWithWidth:width height:height refreshRate:60]];
            NSCAssert([display applySettings:settings], @"Virtual display mode failed");
            [displays addObject:display];
        }
        CGDisplayConfigRef configuration;
        NSCAssert(CGBeginDisplayConfiguration(&configuration) == kCGErrorSuccess, @"Display configuration failed");
        NSUInteger index = 0;
        for (NSDictionary *spec in command[@"displays"]) {
            CGVirtualDisplay *display = displays[index++];
            NSCAssert(CGConfigureDisplayOrigin(configuration, display.displayID, [spec[@"x"] intValue], [spec[@"y"] intValue]) == kCGErrorSuccess, @"Display origin failed");
        }
        NSCAssert(CGCompleteDisplayConfiguration(configuration, kCGConfigureForSession) == kCGErrorSuccess, @"Display commit failed");
        return @(displays.count);
    }
    NSCAssert(CGPreflightPostEventAccess(), @"Native input requires Accessibility permission");
    if ([action isEqual:@"click"]) {
        CGPoint point = CGPointMake([command[@"x"] doubleValue], [command[@"y"] doubleValue]);
        BOOL right = [command[@"right"] boolValue];
        CGEventRef move = CGEventCreateMouseEvent(NULL, kCGEventMouseMoved, point, kCGMouseButtonLeft);
        CGEventPost(kCGHIDEventTap, move);
        CFRelease(move);
        for (int down = 1; down >= 0; down--) {
            CGEventType type = right ? (down ? kCGEventRightMouseDown : kCGEventRightMouseUp) : (down ? kCGEventLeftMouseDown : kCGEventLeftMouseUp);
            CGEventRef event = CGEventCreateMouseEvent(NULL, type, point, right ? kCGMouseButtonRight : kCGMouseButtonLeft);
            CGEventPost(kCGHIDEventTap, event);
            CFRelease(event);
        }
    } else if ([action isEqual:@"key"]) {
        key([command[@"code"] unsignedShortValue], [command[@"flags"] unsignedLongLongValue]);
    } else if ([action isEqual:@"type"]) {
        NSString *text = command[@"text"];
        unichar *characters = calloc(text.length, sizeof(unichar));
        [text getCharacters:characters range:NSMakeRange(0, text.length)];
        CGEventRef event = CGEventCreateKeyboardEvent(NULL, 0, true);
        CGEventKeyboardSetUnicodeString(event, text.length, characters);
        CGEventPost(kCGHIDEventTap, event);
        CFRelease(event);
        free(characters);
    } else {
        [NSException raise:@"Unknown action" format:@"%@", action];
    }
    return @YES;
}

int main(void) {
    @autoreleasepool {
        [NSApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyProhibited];
        char *line = NULL;
        size_t capacity = 0;
        while (getline(&line, &capacity, stdin) > 0) {
            @autoreleasepool {
                NSDictionary *reply;
                @try {
                    NSData *data = [[NSString stringWithUTF8String:line] dataUsingEncoding:NSUTF8StringEncoding];
                    NSDictionary *command = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];
                    reply = @{ @"ok":@YES, @"value":perform(command) };
                } @catch (NSException *exception) {
                    reply = @{ @"ok":@NO, @"error":exception.reason ?: @"Native operation failed" };
                }
                NSData *encoded = [NSJSONSerialization dataWithJSONObject:reply options:0 error:nil];
                puts([[NSString alloc] initWithData:encoded encoding:NSUTF8StringEncoding].UTF8String);
                fflush(stdout);
                [NSRunLoop.currentRunLoop runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.01]];
            }
        }
        free(line);
    }
}
