// Component check: real AppKit panels and controls, without a physical-input claim.
#import "../crates/droidless-cli/src/macos.m"
#include <assert.h>
typedef struct { unsigned clicks, closes; size_t handle; } DialogCheck;
static int dialogCheck(void *opaque, uint32_t kind, size_t handle, const char *text, const NativeMotion *motion) {
    (void)text; (void)motion;
    DialogCheck *check = opaque;
    if (kind == 1) { check->clicks++; check->handle = handle; }
    if (kind == 11) { check->closes++; check->handle = handle; }
    return 2;
}
int main(void) {
    @autoreleasepool {
        DialogCheck check = {0};
        void *opaque = dl_open("Activity", 320, 240, &check, dialogCheck, 0);
        DroidlessHost *owner = (__bridge DroidlessHost *)opaque;
        dl_dialog_begin_frame(opaque);
        void *first = dl_dialog(opaque, 17, 240, 160);
        DroidlessHost *lower = (__bridge DroidlessHost *)first;
        dl_begin(first, "Guest dialog", 0, 0);
        NativeView node = {0}; node.handle = 23; node.click_target = 23;
        node.kind = node.enabled = 1; node.alpha = 1; node.width = 200; node.height = 40;
        node.text_size = 18; node.text = "Guest confirmation";
        dl_view(first, &node); dl_end(first); dl_dialog_end_frame(opaque);
        assert([lower.window isKindOfClass:[NSPanel class]] && lower.window.visible);
        assert(lower.window.parentWindow == owner.window && owner.activeDialog == lower);
        NSButton *button = (NSButton *)lower.views[@23];
        [button performClick:nil];
        assert(check.clicks == 1 && check.handle == 23);
        [lower.window performClose:nil];
        assert(check.closes == 1 && check.handle == 17 && lower.window.visible && owner.running);
        dl_dialog_begin_frame(opaque);
        assert(dl_dialog(opaque, 17, 200, 120) == first);
        void *second = dl_dialog(opaque, 19, 180, 100);
        DroidlessHost *upper = (__bridge DroidlessHost *)second;
        dl_begin(second, "Upper dialog", 0, 0); dl_end(second); dl_dialog_end_frame(opaque);
        assert(owner.dialogs.count == 2 && owner.activeDialog == upper);
        [button performClick:nil];
        assert(check.clicks == 1); // Native controls below the top modal surface cannot dispatch.
        assert(NSEqualSizes(lower.window.contentView.bounds.size, NSMakeSize(200,120)));
        [owner.window performClose:nil];
        assert(check.closes == 2 && check.handle == 19 && owner.running);
        dl_dialog_begin_frame(opaque); dl_dialog(opaque, 17, 200, 120); dl_dialog_end_frame(opaque);
        assert(owner.dialogs.count == 1 && !upper.window.visible && !upper.running);
        assert(owner.activeDialog == lower);
        dl_dialog_begin_frame(opaque); dl_dialog_end_frame(opaque);
        assert(owner.dialogs.count == 0 && !lower.window.visible && !owner.activeDialog);
        assert(owner.window.visible && owner.running);
        dl_destroy(opaque);
        puts("Native dialogs: separate NSPanels, guest control callback, close routing, nesting, resize and retirement passed.");
    }
}
