// Painting masks, independent mouse bounds, foregrounds, reuse and dialog surfaces.
#import "../crates/droidless-cli/src/macos.m"
#include <assert.h>
static int callback(void *context, uint32_t kind, size_t handle, const char *text, const NativeMotion *motion) {
    (void)context; (void)handle; (void)text; (void)motion; return kind==10 ? 2 : 1;
}
static NSView *hit(NSView *surface, CGFloat x, CGFloat y) {
    return [surface hitTest:[surface.superview convertPoint:NSMakePoint(x,y) fromView:surface]];
}
static void pixels(NSView *view) {
    unsigned char bytes[100*80*4] = {0};
    CGColorSpaceRef space = CGColorSpaceCreateDeviceRGB();
    CGContextRef context = CGBitmapContextCreate(bytes,100,80,8,100*4,space,kCGImageAlphaPremultipliedLast|kCGBitmapByteOrder32Big);
    assert(context); [view.layer renderInContext:context];
    // Quartz bitmap rows use the opposite Y direction to this flipped desktop surface.
    assert(bytes[((79-15)*100+25)*4+3]>200); // clipped blue paint survives inside
    assert(bytes[((79-15)*100+5)*4+3]==0 && bytes[((79-35)*100+25)*4+3]==0);
    CGContextRelease(context); CGColorSpaceRelease(space);
}
int main(void) {
    @autoreleasepool {
        void *opaque = dl_open("Clip contract",240,160,NULL,callback,0);
        DroidlessHost *host = (__bridge DroidlessHost *)opaque;
        NativeView node={0}; node.handle=11; node.enabled=1; node.alpha=1; node.has_clips=1;
        node.x=20; node.y=20; node.width=100; node.height=80; node.text_size=18; node.text="";
        node.has_background=1; node.background=0xff2244cc;
        float paint[]={40,30,30,20}, input[]={20,20,50,80};
        memcpy(node.paint_clip,paint,sizeof(paint)); memcpy(node.input_clip,input,sizeof(input));
        dl_begin(opaque,"Clip contract",0,0); dl_view(opaque,&node);
        dl_foreground(opaque,11,0xff2244cc,20,20,100,80,1); dl_end(opaque);
        NSView *view=host.views[@11], *overlay=host.foregrounds[@11], *surface=host.window.contentView;
        [host.window displayIfNeeded]; pixels(view); pixels(overlay);
        assert(hit(surface,25,25)==view); // padding painting clip does not block targeting
        assert(hit(surface,60,40)==view && hit(surface,80,40)!=view); // ancestor input bounds
        dl_begin(opaque,"Clip contract",0,0); node.has_clips=0; dl_view(opaque,&node);
        dl_foreground(opaque,11,0xff2244cc,20,20,100,80,1); dl_end(opaque);
        assert(host.views[@11]==view && !view.layer.mask && !overlay.layer.mask);
        assert(hit(surface,80,40)==view);
        // A real native button uses its own coordinate orientation and the same input scope.
        node.handle=12; node.kind=1; node.has_clips=1; node.text="Click";
        dl_begin(opaque,"Clip contract",0,0); dl_view(opaque,&node); dl_end(opaque);
        NSView *button=host.views[@12];
        assert([button isKindOfClass:NSButton.class]);
        NSRect expected=[button convertRect:NSMakeRect(40,30,30,20) fromView:surface];
        assert(NSEqualRects(CGPathGetPathBoundingBox(((CAShapeLayer *)button.layer.mask).path),expected));
        assert(hit(surface,25,25)==button && hit(surface,80,40)!=button);
        node.handle=13; node.kind=3; node.editable=1; node.text="Editable";
        dl_begin(opaque,"Clip contract",0,0); dl_view(opaque,&node); dl_end(opaque);
        NSTextField *field=(NSTextField *)host.views[@13];
        assert([host.window makeFirstResponder:field] && field.currentEditor);
        NSTextView *editor=(NSTextView *)field.currentEditor; editor.selectedRange=NSMakeRange(1,3);
        assert(hit(surface,25,25)==editor && hit(surface,80,40)!=editor);
        dl_begin(opaque,"Clip contract",0,0); node.paint_clip[2]=20; dl_view(opaque,&node); dl_end(opaque);
        assert(field.currentEditor==editor && NSEqualRanges(editor.selectedRange,NSMakeRange(1,3)));
        node.handle=12; node.kind=1; node.editable=0; node.text="Click";
        dl_dialog_begin_frame(opaque); void *panel=dl_dialog(opaque,99,240,160);
        DroidlessHost *dialog=(__bridge DroidlessHost *)panel;
        dl_begin(panel,"Dialog clip",0,0); dl_view(panel,&node); dl_end(panel); dl_dialog_end_frame(opaque);
        assert(hit(dialog.window.contentView,25,25)==dialog.views[@12]);
        assert(hit(dialog.window.contentView,80,40)!=dialog.views[@12]);
        dl_begin(opaque,"Clip contract",0,0); dl_end(opaque);
        assert(host.views.count==0 && host.foregrounds.count==0 && view.superview==nil);
        dl_destroy(opaque);
        puts("Native clipping: painted pixels, foreground masks, independent input bounds, retained editor selection, control reuse, button orientation and dialog surfaces passed.");
    }
}
