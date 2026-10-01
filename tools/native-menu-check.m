// Run from the repo: xcrun clang -fobjc-arc -Wall -Wextra -Werror -framework AppKit
// -framework QuartzCore tools/native-menu-check.m -o /tmp/droidless-menu-check && /tmp/droidless-menu-check
#import "../crates/droidless-cli/src/macos.m"
#include <assert.h>

typedef struct { void *host; unsigned prepares, selections; size_t selected; } MenuCheck;
static int menuCheck(void *opaque, uint32_t kind, size_t handle, const char *text, const NativeMotion *motion) {
    (void)text; (void)motion;
    MenuCheck *check = opaque;
    if (kind == 8) {
        check->prepares++;
        dl_menu_clear(check->host);
        char title[] = "Choisir café";
        dl_menu_item(check->host, 17, title, 1, 1);
        title[0] = 'X'; // Native titles must outlive the borrowed UTF-8 buffer.
        dl_menu_item(check->host, 23, "Disabled", 0, 0);
    } else if (kind == 9) {
        check->selections++;
        check->selected = handle;
    }
    return 1;
}
int main(void) {
    @autoreleasepool {
        MenuCheck check = {0};
        check.host = dl_open("Native menu contract", 128, 128, &check, menuCheck, 0);
        assert(check.host);
        DroidlessHost *host = (__bridge DroidlessHost *)check.host;
        assert(host.options.delegate == nil);
        dl_end(check.host);
        assert(host.options.delegate == host && !host.options.autoenablesItems);
        [host.options.delegate menuNeedsUpdate:host.options];
        assert(check.prepares > 0 && host.options.numberOfItems == 2);
        NSMenuItem *first = [host.options itemAtIndex:0];
        assert([first.title isEqualToString:@"Choisir café"] && first.enabled);
        assert(first.state == NSControlStateValueOn && first.tag == 17);
        NSMenuItem *second = [host.options itemAtIndex:1];
        assert(!second.enabled && second.state == NSControlStateValueOff && second.tag == 23);
        [host.options performActionForItemAtIndex:0];
        assert(check.selections == 1 && check.selected == 17);
        unsigned before = check.prepares;
        [host.options.delegate menuNeedsUpdate:host.options];
        assert(check.prepares > before && host.options.numberOfItems == 2);
        dl_menu_clear(check.host);
        assert(host.options.numberOfItems == 0);
        dl_destroy(check.host);
        assert(host.options.delegate == nil && !host.window.visible);
        assert([NSApp.mainMenu indexOfItemWithSubmenu:host.options] == -1);
        puts("Native menu: preparation, copied UTF-8, enabled/checked state, action dispatch and close passed.");
    }
    return 0;
}
