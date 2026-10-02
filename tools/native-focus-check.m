// xcrun clang -fobjc-arc -Wall -Wextra -Werror -framework AppKit -framework QuartzCore
// tools/native-focus-check.m -o /tmp/droidless-focus-check && /tmp/droidless-focus-check
#import "../crates/droidless-cli/src/macos.m"
#include <assert.h>

typedef struct { unsigned requests; size_t handle; int result; } FocusCheck;
static int focusCheck(void *opaque,uint32_t kind,size_t handle,const char *text,const NativeMotion *motion) {
    (void)text; (void)motion;
    FocusCheck *check=opaque;
    if(kind==10) { check->requests++; check->handle=handle; return check->result; }
    return 1;
}
int main(void) {
    @autoreleasepool {
        FocusCheck check={0,0,2};
        void *opaque=dl_open("Native focus contract",240,120,&check,focusCheck,0);
        DroidlessHost *host=(__bridge DroidlessHost *)opaque;
        NativeView node={0}; node.kind=3; node.enabled=node.editable=1; node.alpha=1;
        node.width=220; node.height=36; node.text_size=20; node.text="Editable text";
        dl_begin(opaque,"Native focus contract",1,0);
        node.handle=1; dl_view(opaque,&node);
        node.handle=2; node.y=40; dl_view(opaque,&node);
        NSTextField *first=(NSTextField *)host.views[@1], *second=(NSTextField *)host.views[@2];
        [host.window makeFirstResponder:first];
        assert(check.requests==0 && !first.currentEditor);
        dl_end(opaque);
        first.nextKeyView=second;
        assert([host.window makeFirstResponder:first]);
        assert(check.requests==1 && check.handle==1 && first.currentEditor);
        NSTextView *editor=(NSTextView *)first.currentEditor;
        editor.selectedRange=NSMakeRange(2,4);
        dl_begin(opaque,"Native focus contract",1,0);
        node.handle=1; node.y=0; dl_view(opaque,&node);
        node.handle=2; node.y=40; dl_view(opaque,&node); dl_end(opaque);
        assert(check.requests==1 && first.currentEditor==editor && NSEqualRanges(editor.selectedRange,NSMakeRange(2,4)));
        [host.window selectKeyViewFollowingView:first];
        assert(check.requests==2 && check.handle==2 && second.currentEditor && !first.currentEditor);
        [host.window makeFirstResponder:nil];
        check.result=1; [host.window makeFirstResponder:first];
        assert(check.requests==3 && check.handle==1 && !first.currentEditor && host.running);
        [host.window makeFirstResponder:nil];
        check.result=0; [host.window makeFirstResponder:first];
        assert(check.requests==4 && !first.currentEditor && !host.running);
        dl_destroy(opaque);
        puts("Native editor focus: first responder, key traversal, retained selection, rejection and failure passed.");
    }
}
