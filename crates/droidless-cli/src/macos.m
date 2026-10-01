// AppKit only. No Android framework, browser, or emulator is linked here.
#import <AppKit/AppKit.h>
#import <QuartzCore/QuartzCore.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

typedef struct { uint64_t time; int32_t action; float x, y; } NativeMotion;
typedef int (*Callback)(void *, uint32_t, size_t, const char *, const NativeMotion *);
typedef struct {
    size_t handle;
    uint32_t kind, enabled, editable, visible, foreground, background, has_background, gravity, key_listener;
    int32_t image_scale;
    float x, y, width, height, text_size, alpha, padding[4];
    const char *text;
    const char *description;
    size_t click_target;
    const uint8_t *image;
    size_t image_len;
} NativeView;

@interface FlippedView : NSView
@end
@implementation FlippedView
- (BOOL)isFlipped { return YES; }
@end

@interface DroidlessImage : NSImageView
@property int32_t scaleType;
@property NSEdgeInsets contentPadding;
@end
@implementation DroidlessImage
- (BOOL)isFlipped { return YES; }
- (void)drawRect:(NSRect)dirty {
    (void)dirty;
    NSImage *image = self.image;
    if (!image || image.size.width <= 0 || image.size.height <= 0) return;
    NSEdgeInsets padding = self.contentPadding;
    NSRect content = NSMakeRect(self.bounds.origin.x+padding.left, self.bounds.origin.y+padding.top,
        self.bounds.size.width-padding.left-padding.right, self.bounds.size.height-padding.top-padding.bottom);
    if (content.size.width <= 0 || content.size.height <= 0) return;
    CGFloat scale = MIN(content.size.width / image.size.width, content.size.height / image.size.height);
    if (self.scaleType == 5) scale = 1;
    if (self.scaleType == 6) scale = MAX(content.size.width / image.size.width, content.size.height / image.size.height);
    if (self.scaleType == 7) scale = MIN(1, scale);
    NSSize size = self.scaleType == 1 ? content.size : NSMakeSize(image.size.width * scale, image.size.height * scale);
    CGFloat alignment = self.scaleType == 2 ? 0 : (self.scaleType == 4 ? 1 : .5);
    NSRect target = NSMakeRect(content.origin.x + (content.size.width - size.width) * alignment,
                              content.origin.y + (content.size.height - size.height) * alignment, size.width, size.height);
    [NSGraphicsContext saveGraphicsState];
    [[NSBezierPath bezierPathWithRect:content] addClip];
    [image drawInRect:target fromRect:NSZeroRect operation:NSCompositingOperationSourceOver fraction:1 respectFlipped:YES hints:nil];
    [NSGraphicsContext restoreGraphicsState];
}
@end

@interface DroidlessClick : NSClickGestureRecognizer
@property size_t handle;
@end
@implementation DroidlessClick
@end

@interface DroidlessHost : NSObject <NSWindowDelegate, NSTextFieldDelegate>
@property NSWindow *window;
@property NSMutableDictionary<NSNumber *,NSView *> *views;
@property NSMutableSet<NSNumber *> *touched;
@property void *context;
@property Callback callback;
@property BOOL running;
@property size_t keyTarget;
@property BOOL touchEnabled;
@property BOOL touchTracking;
@property double clockOffset;
@property NativeMotion lastMotion;
- (void)clicked:(NSControl *)sender;
- (void)cellClicked:(DroidlessClick *)sender;
- (void)quit:(id)sender;
@end

@implementation DroidlessHost
- (void)quit:(id)sender { (void)sender; self.running = NO; }
- (void)clicked:(NSControl *)sender {
    if (!self.callback(self.context, 1, (size_t)sender.tag, NULL, NULL)) self.running = NO;
}
- (void)cellClicked:(DroidlessClick *)sender {
    if (!self.callback(self.context, 1, sender.handle, NULL, NULL)) self.running = NO;
}
- (void)controlTextDidChange:(NSNotification *)notification {
    NSTextField *field = notification.object;
    if (!self.callback(self.context, 2, (size_t)field.tag, field.stringValue.UTF8String, NULL)) self.running = NO;
}
- (BOOL)windowShouldClose:(NSWindow *)sender {
    (void)sender;
    self.running = NO;
    return YES;
}
- (void)windowDidResignKey:(NSNotification *)notification {
    (void)notification;
    if (self.touchTracking && self.running) {
        self.touchTracking = NO;
        NativeMotion motion = self.lastMotion;
        motion.action = 3;
        motion.time = (uint64_t)(MAX(0.0, NSProcessInfo.processInfo.systemUptime + self.clockOffset) * 1000.0);
        if (!self.callback(self.context, 7, 0, NULL, &motion)) self.running = NO;
    }
}
@end

static NSColor *color(uint32_t argb) {
    return [NSColor colorWithSRGBRed:((argb>>16)&255)/255.0 green:((argb>>8)&255)/255.0 blue:(argb&255)/255.0 alpha:((argb>>24)&255)/255.0];
}

void *dl_open(const char *title, float width, float height, void *context, Callback callback, uint64_t uptime) {
    [NSApplication sharedApplication];
    [NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];
    NSMenu *menu = [NSMenu new];
    NSMenuItem *appItem = [NSMenuItem new];
    [menu addItem:appItem];
    NSMenu *appMenu = [[NSMenu alloc] initWithTitle:@"DROIDLESS"];
    NSMenuItem *quit = [appMenu addItemWithTitle:@"Quit DROIDLESS" action:@selector(quit:) keyEquivalent:@"q"];
    appItem.submenu = appMenu;
    NSMenuItem *editItem = [[NSMenuItem alloc] initWithTitle:@"Edit" action:NULL keyEquivalent:@""];
    NSMenu *editMenu = [[NSMenu alloc] initWithTitle:@"Edit"];
    [editMenu addItemWithTitle:@"Cut" action:@selector(cut:) keyEquivalent:@"x"];
    [editMenu addItemWithTitle:@"Copy" action:@selector(copy:) keyEquivalent:@"c"];
    [editMenu addItemWithTitle:@"Paste" action:@selector(paste:) keyEquivalent:@"v"];
    [editMenu addItemWithTitle:@"Select All" action:@selector(selectAll:) keyEquivalent:@"a"];
    editItem.submenu = editMenu;
    [menu addItem:editItem];
    NSApp.mainMenu = menu;
    DroidlessHost *host = [DroidlessHost new];
    quit.target = host;
    host.context = context;
    host.callback = callback;
    host.clockOffset = uptime / 1000.0 - NSProcessInfo.processInfo.systemUptime;
    host.running = YES;
    host.views = [NSMutableDictionary new];
    host.touched = [NSMutableSet new];
    host.window = [[NSWindow alloc] initWithContentRect:NSMakeRect(0,0,width,height)
        styleMask:NSWindowStyleMaskTitled|NSWindowStyleMaskClosable|NSWindowStyleMaskMiniaturizable
        backing:NSBackingStoreBuffered defer:NO];
    host.window.title = [NSString stringWithUTF8String:title];
    host.window.releasedWhenClosed = NO;
    host.window.delegate = host;
    host.window.contentView = [[FlippedView alloc] initWithFrame:NSMakeRect(0,0,width,height)];
    [host.window center];
    [host.window makeKeyAndOrderFront:nil];
    [NSApp activateIgnoringOtherApps:YES];
    [NSApp finishLaunching];
    if (getenv("DROIDLESS_NATIVE_TRACE")) fprintf(stderr, "native window %ld visible=%d frame=%s\n", (long)host.window.windowNumber, host.window.visible, NSStringFromRect(host.window.frame).UTF8String);
    return (__bridge_retained void *)host;
}

void dl_begin(void *opaque, const char *title, uint32_t touchEnabled, uint32_t touchActive) {
    DroidlessHost *host = (__bridge DroidlessHost *)opaque;
    host.window.title = [NSString stringWithUTF8String:title];
    host.touchEnabled = touchEnabled != 0;
    if (!host.touchEnabled || !touchActive) host.touchTracking = NO;
    [host.touched removeAllObjects];
    host.keyTarget = 0;
}
void dl_view(void *opaque, const NativeView *node) {
    DroidlessHost *host = (__bridge DroidlessHost *)opaque;
    NSNumber *key = @(node->handle);
    [host.touched addObject:key];
    if (node->key_listener && !host.keyTarget) host.keyTarget = node->handle;
    NSView *view = host.views[key];
    if (!view) {
        if (node->kind == 1) {
            NSButton *button = [NSButton buttonWithTitle:@"" target:host action:@selector(clicked:)];
            button.bezelStyle = NSBezelStyleRegularSquare;
            button.buttonType = NSButtonTypeMomentaryPushIn;
            view = button;
        } else if (node->kind == 2 || node->kind == 3) {
            NSTextField *text = [NSTextField new];
            text.delegate = host;
            text.bezeled = NO;
            text.drawsBackground = YES;
            view = text;
        } else if (node->kind == 4) {
            NSImageView *image = [DroidlessImage new];
            image.imageScaling = NSImageScaleProportionallyUpOrDown;
            view = image;
        } else {view = [FlippedView new];}
        host.views[key] = view;
        [host.window.contentView addSubview:view];
    }
    view.frame = NSMakeRect(node->x,node->y,node->width,node->height);
    view.hidden = node->visible != 0;
    view.alphaValue = node->alpha;
    view.wantsLayer = YES;
    view.layer.backgroundColor = node->has_background ? color(node->background).CGColor : NULL;
    NSString *text = [NSString stringWithUTF8String:node->text];
    NSString *description = node->description ? [NSString stringWithUTF8String:node->description] : nil;
    view.accessibilityLabel = description;
    NSData *imageData = node->image_len ? [NSData dataWithBytes:node->image length:node->image_len] : nil;
    if (node->image_len && getenv("DROIDLESS_NATIVE_TRACE")) fprintf(stderr, "native image handle=%zu bytes=%zu\n", node->handle, node->image_len);
    NSImage *image = imageData ? [[NSImage alloc] initWithData:imageData] : nil;
    if ([view isKindOfClass:[NSButton class]]) {
        NSButton *button = (NSButton *)view;
        button.tag = (NSInteger)(node->click_target ?: node->handle);
        button.enabled = node->enabled != 0;
        button.font = [NSFont systemFontOfSize:node->text_size];
        button.title = text;
        button.image = image;
        button.imagePosition = text.length ? NSImageLeading : NSImageOnly;
        button.bordered = !node->has_background;
        button.layer.cornerRadius = node->has_background ? 5 : 0;
        button.attributedTitle = [[NSAttributedString alloc] initWithString:text attributes:@{NSForegroundColorAttributeName:color(node->foreground),NSFontAttributeName:button.font}];
        button.accessibilityLabel = description ?: text;
    } else if ([view isKindOfClass:[NSTextField class]]) {
        NSTextField *field = (NSTextField *)view;
        field.tag = (NSInteger)node->handle;
        field.enabled = node->enabled != 0;
        field.editable = node->editable != 0;
        field.selectable = node->editable != 0;
        field.font = [NSFont systemFontOfSize:node->text_size];
        field.textColor = color(node->foreground);
        field.backgroundColor = node->has_background ? color(node->background) : NSColor.clearColor;
        field.alignment = (node->gravity&7)==5 ? NSTextAlignmentRight : ((node->gravity&7)==1 ? NSTextAlignmentCenter : NSTextAlignmentLeft);
        if (![field.stringValue isEqualToString:text]) field.stringValue = text;
        field.accessibilityLabel = description ?: text;
    } else if ([view isKindOfClass:[NSImageView class]]) {
        ((NSImageView *)view).image = image;
        DroidlessImage *nativeImage = (DroidlessImage *)view;
        nativeImage.scaleType = node->image_scale;
        nativeImage.contentPadding = NSEdgeInsetsMake(node->padding[1],node->padding[0],node->padding[3],node->padding[2]);
        [nativeImage setNeedsDisplay:YES];
    }
    if (![view isKindOfClass:[NSButton class]]) {
        DroidlessClick *click = nil;
        for (NSGestureRecognizer *recognizer in view.gestureRecognizers) {
            if ([recognizer isKindOfClass:[DroidlessClick class]]) {
                click = (DroidlessClick *)recognizer;
                break;
            }
        }
        if (node->click_target && !node->editable) {
            if (!click) {
                click = [[DroidlessClick alloc] initWithTarget:host action:@selector(cellClicked:)];
                [view addGestureRecognizer:click];
            }
            click.handle = node->click_target;
            click.enabled = node->enabled != 0;
        } else if (click) {
            [view removeGestureRecognizer:click];
        }
    }
}
void dl_end(void *opaque) {
    DroidlessHost *host = (__bridge DroidlessHost *)opaque;
    for (NSNumber *key in [host.views.allKeys copy]) {
        if (![host.touched containsObject:key]) {[host.views[key] removeFromSuperview];[host.views removeObjectForKey:key];}
    }
}
int dl_choose_directory(void *opaque, char **path) {
    DroidlessHost *host = (__bridge DroidlessHost *)opaque;
    *path = NULL;
    [host.window makeKeyAndOrderFront:nil];
    NSOpenPanel *panel = [NSOpenPanel openPanel];
    panel.title = @"Choose a folder";
    panel.message = @"Choose a folder to open in this app.";
    panel.canChooseDirectories = YES;
    panel.canChooseFiles = NO;
    panel.allowsMultipleSelection = NO;
    if ([panel runModal] != NSModalResponseOK) return 0;
    const char *selected = panel.URL.fileSystemRepresentation;
    if (!selected) return -1;
    *path = strdup(selected);
    return *path ? 1 : -1;
}
void dl_free_path(char *path) { free(path); }
void dl_run(void *opaque) {
    DroidlessHost *host = (__bridge DroidlessHost *)opaque;
    while (host.running && host.window.visible) {
        @autoreleasepool {
            if (!host.callback(host.context, 6, 0, NULL, NULL)) { host.running = NO; break; }
            NSEvent *event = [NSApp nextEventMatchingMask:NSEventMaskAny untilDate:[NSDate dateWithTimeIntervalSinceNow:0.05] inMode:NSDefaultRunLoopMode dequeue:YES];
            if (event) {
                int consumed = 0;
                if (getenv("DROIDLESS_NATIVE_TRACE") && (event.type == NSEventTypeKeyDown || event.type == NSEventTypeKeyUp))
                    fprintf(stderr, "native key type=%lu code=%u modifiers=%lu\n", (unsigned long)event.type, event.keyCode, (unsigned long)event.modifierFlags);
                if ((event.type == NSEventTypeKeyDown || event.type == NSEventTypeKeyUp) && event.keyCode == 53 && !(event.modifierFlags&NSEventModifierFlagCommand)) {
                    if (event.type == NSEventTypeKeyDown && !host.callback(host.context, 5, 0, NULL, NULL)) host.running = NO;
                    consumed = 1;
                } else if (host.keyTarget && (event.type == NSEventTypeKeyDown || event.type == NSEventTypeKeyUp) && !(event.modifierFlags&NSEventModifierFlagCommand)) {
                    int result = host.callback(host.context, event.type == NSEventTypeKeyDown ? 3 : 4, host.keyTarget, event.charactersIgnoringModifiers.UTF8String, NULL);
                    if (!result) host.running = NO;
                    consumed = result == 2;
                }
                if (event.window == host.window && (event.type == NSEventTypeLeftMouseDown || event.type == NSEventTypeLeftMouseDragged || event.type == NSEventTypeLeftMouseUp)) {
                    NSPoint point = [host.window.contentView convertPoint:event.locationInWindow fromView:nil];
                    if (event.type == NSEventTypeLeftMouseDown && host.touchEnabled && NSPointInRect(point, host.window.contentView.bounds)) {
                        // NSView hitTest expects its superview coordinates; guest MotionEvent stays in flipped content coordinates.
                        NSPoint hitPoint = [host.window.contentView.superview convertPoint:event.locationInWindow fromView:nil];
                        NSView *hit = [host.window.contentView hitTest:hitPoint];
                        // Keep AppKit's focus and text-selection behavior for editable controls.
                        if (![hit isKindOfClass:[NSTextField class]] || !((NSTextField *)hit).editable) host.touchTracking = YES;
                    }
                    if (host.touchTracking) {
                        NativeMotion motion = {(uint64_t)(MAX(0.0, event.timestamp + host.clockOffset) * 1000.0),
                            event.type == NSEventTypeLeftMouseDown ? 0 : event.type == NSEventTypeLeftMouseUp ? 1 : 2,
                            (float)point.x, (float)point.y};
                        host.lastMotion = motion;
                        if (getenv("DROIDLESS_NATIVE_TRACE")) fprintf(stderr, "native touch action=%d x=%.1f y=%.1f time=%llu\n", motion.action, motion.x, motion.y, (unsigned long long)motion.time);
                        if (event.type == NSEventTypeLeftMouseUp) host.touchTracking = NO;
                        if (!host.callback(host.context, 7, 0, NULL, &motion)) host.running = NO;
                        consumed = 1;
                    }
                }
                if (!consumed) [NSApp sendEvent:event];
            }
            [NSApp updateWindows];
        }
    }
}
void dl_destroy(void *opaque) {
    DroidlessHost *host = (__bridge_transfer DroidlessHost *)opaque;
    if (getenv("DROIDLESS_NATIVE_TRACE")) fprintf(stderr, "native exit running=%d visible=%d\n", host.running, host.window.visible);
    [host.window close];
}
