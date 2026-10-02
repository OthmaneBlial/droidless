// Native component check for foreground stacking, alpha, input transparency and retirement.
#import "../crates/droidless-cli/src/macos.m"
#include <assert.h>
static int callback(void *context, uint32_t kind, size_t handle, const char *text, const NativeMotion *motion) {
    (void)context; (void)kind; (void)handle; (void)text; (void)motion; return 1;
}
int main(void) {
    @autoreleasepool {
        void *opaque = dl_open("Foreground contract", 240, 160, NULL, callback, 0);
        DroidlessHost *host = (__bridge DroidlessHost *)opaque;
        NativeView node = {0}; node.handle = 11; node.kind = node.enabled = 1; node.alpha = 1;
        node.width = 100; node.height = 50; node.text_size = 18; node.text = "Child control";
        dl_begin(opaque, "Foreground contract", 0, 0); dl_view(opaque, &node);
        dl_foreground(opaque, 9, 0x80224466, 0, 0, 240, 160, .75); dl_end(opaque);
        NSView *overlay = host.foregrounds[@9], *child = host.views[@11];
        assert(host.window.contentView.subviews.lastObject == overlay);
        assert([overlay hitTest:NSMakePoint(10,10)] == nil);
        assert(fabs(overlay.alphaValue-.75)<.001);
        NSColor *paint = [NSColor colorWithCGColor:overlay.layer.backgroundColor];
        assert(fabs(paint.alphaComponent-128.0/255.0)<.001);
        NSPoint parentPoint = [host.window.contentView.superview convertPoint:NSMakePoint(10,10) fromView:host.window.contentView];
        assert([host.window.contentView hitTest:parentPoint] == child);
        dl_begin(opaque, "Foreground contract", 0, 0); dl_view(opaque, &node);
        dl_foreground(opaque, 9, 0x80446688, 0, 0, 200, 120, .5); dl_end(opaque);
        assert(host.foregrounds[@9] == overlay && NSEqualSizes(overlay.bounds.size,NSMakeSize(200,120)));
        assert(host.window.contentView.subviews.lastObject == overlay);
        dl_begin(opaque, "Foreground contract", 0, 0); dl_view(opaque, &node); dl_end(opaque);
        assert(host.foregrounds.count == 0 && overlay.superview == nil && child.superview != nil);
        dl_destroy(opaque);
        puts("Native foreground: paint above child controls, ARGB/ancestor alpha, input transparency, reuse and clearing passed.");
    }
}
