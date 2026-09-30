// AppKit only. No Android framework, browser, or emulator is linked here.
#import <AppKit/AppKit.h>
#import <QuartzCore/QuartzCore.h>
#include <stdint.h>

typedef int (*Callback)(void *, uint32_t, size_t, const char *);
typedef struct {
    size_t handle;
    uint32_t kind, enabled, editable, visible, foreground, background, has_background, gravity, key_listener;
    float x, y, width, height, text_size;
    const char *text;
} NativeView;

@interface FlippedView : NSView
@end
@implementation FlippedView
- (BOOL)isFlipped { return YES; }
@end

@interface DroidlessHost : NSObject <NSWindowDelegate, NSTextFieldDelegate>
@property NSWindow *window;
@property NSMutableDictionary<NSNumber *,NSView *> *views;
@property NSMutableSet<NSNumber *> *touched;
@property void *context;
@property Callback callback;
@property BOOL running;
@property size_t keyTarget;
- (void)clicked:(NSControl *)sender;
- (void)quit:(id)sender;
@end

@implementation DroidlessHost
- (void)quit:(id)sender { (void)sender; self.running = NO; }
- (void)clicked:(NSControl *)sender {
    if (!self.callback(self.context, 1, (size_t)sender.tag, NULL)) self.running = NO;
}
- (void)controlTextDidChange:(NSNotification *)notification {
    NSTextField *field = notification.object;
    if (!self.callback(self.context, 2, (size_t)field.tag, field.stringValue.UTF8String)) self.running = NO;
}
- (BOOL)windowShouldClose:(NSWindow *)sender {
    (void)sender;
    self.running = NO;
    return YES;
}
@end

static NSColor *color(uint32_t argb) {
    return [NSColor colorWithSRGBRed:((argb>>16)&255)/255.0 green:((argb>>8)&255)/255.0 blue:(argb&255)/255.0 alpha:((argb>>24)&255)/255.0];
}

void *dl_open(const char *title, float width, float height, void *context, Callback callback) {
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

void dl_begin(void *opaque, const char *title) {
    DroidlessHost *host = (__bridge DroidlessHost *)opaque;
    host.window.title = [NSString stringWithUTF8String:title];
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
        } else {view = [FlippedView new];}
        host.views[key] = view;
        [host.window.contentView addSubview:view];
    }
    view.frame = NSMakeRect(node->x,node->y,node->width,node->height);
    view.hidden = node->visible != 0;
    view.wantsLayer = YES;
    if (node->has_background) view.layer.backgroundColor = color(node->background).CGColor;
    NSString *text = [NSString stringWithUTF8String:node->text];
    if ([view isKindOfClass:[NSButton class]]) {
        NSButton *button = (NSButton *)view;
        button.tag = (NSInteger)node->handle;
        button.enabled = node->enabled != 0;
        button.font = [NSFont systemFontOfSize:node->text_size];
        button.title = text;
        if (node->has_background) {button.bordered = NO;button.layer.cornerRadius = 5;}
        button.attributedTitle = [[NSAttributedString alloc] initWithString:text attributes:@{NSForegroundColorAttributeName:color(node->foreground),NSFontAttributeName:button.font}];
        button.accessibilityLabel = text;
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
        field.accessibilityLabel = text;
    }
}
void dl_end(void *opaque) {
    DroidlessHost *host = (__bridge DroidlessHost *)opaque;
    for (NSNumber *key in [host.views.allKeys copy]) {
        if (![host.touched containsObject:key]) {[host.views[key] removeFromSuperview];[host.views removeObjectForKey:key];}
    }
}
void dl_run(void *opaque) {
    DroidlessHost *host = (__bridge DroidlessHost *)opaque;
    while (host.running && host.window.visible) {
        @autoreleasepool {
            if (!host.callback(host.context, 6, 0, NULL)) { host.running = NO; break; }
            NSEvent *event = [NSApp nextEventMatchingMask:NSEventMaskAny untilDate:[NSDate dateWithTimeIntervalSinceNow:0.05] inMode:NSDefaultRunLoopMode dequeue:YES];
            if (event) {
                int consumed = 0;
                if ((event.type == NSEventTypeKeyDown || event.type == NSEventTypeKeyUp) && event.keyCode == 53 && !(event.modifierFlags&NSEventModifierFlagCommand)) {
                    if (event.type == NSEventTypeKeyDown && !host.callback(host.context, 5, 0, NULL)) host.running = NO;
                    consumed = 1;
                } else if (host.keyTarget && (event.type == NSEventTypeKeyDown || event.type == NSEventTypeKeyUp) && !(event.modifierFlags&NSEventModifierFlagCommand)) {
                    int result = host.callback(host.context, event.type == NSEventTypeKeyDown ? 3 : 4, host.keyTarget, event.charactersIgnoringModifiers.UTF8String);
                    if (!result) host.running = NO;
                    consumed = result == 2;
                }
                if (!consumed) [NSApp sendEvent:event];
            }
            [NSApp updateWindows];
        }
    }
}
void dl_destroy(void *opaque) {
    DroidlessHost *host = (__bridge_transfer DroidlessHost *)opaque;
    [host.window close];
}
