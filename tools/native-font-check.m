// xcrun clang -fobjc-arc -Wall -Wextra -Werror -framework AppKit -framework QuartzCore
// tools/native-font-check.m -o /tmp/droidless-font-check && /tmp/droidless-font-check
#import "../crates/droidless-cli/src/macos.m"
#import "../crates/droidless-runtime/native/macos-text.m"
#include <assert.h>
static int fontCheck(void *context,uint32_t kind,size_t handle,const char *text,const NativeMotion *motion) {
    (void)context; (void)kind; (void)handle; (void)text; (void)motion; return 1;
}
int main(void) {
    @autoreleasepool {
        void *opaque=dl_open("Native font contract",240,100,NULL,fontCheck,0);
        DroidlessHost *host=(__bridge DroidlessHost *)opaque;
        NativeView node={0}; node.enabled=1; node.alpha=1; node.width=220; node.height=36;
        node.text_size=20; node.text="Font contract";
        for(uint32_t family=0;family<3;family++) for(uint32_t style=0;style<4;style++) {
            node.font_family=family; node.font_style=style;
            for(uint32_t kind=1;kind<=3;kind++) {
                node.handle=kind; node.kind=kind; node.editable=kind==3;
                dl_view(opaque,&node);
                NSFont *font=[(NSControl *)host.views[@(kind)] font]; assert(font && font.pointSize==20);
                NSFontTraitMask traits=[[NSFontManager sharedFontManager] traitsOfFont:font];
                assert(((traits&NSBoldFontMask)!=0)==((style&1)!=0));
                assert(((traits&NSItalicFontMask)!=0)==((style&2)!=0));
                if(family==2) assert(font.isFixedPitch);
                if(family==1) assert([font.familyName rangeOfString:@"Times"].location!=NSNotFound);
                float metrics[3]; assert(droidless_font_metrics(family,style,20,metrics));
                assert(fabs(metrics[0]+font.ascender)<0.001 && fabs(metrics[1]+font.descender)<0.001);
                assert(fabs(metrics[2]-font.leading)<0.001);
            }
        }
        node.text="abcd\u2026"; node.description="abcdefghij";
        for(uint32_t kind=1;kind<=2;kind++) {
            node.handle=kind; node.kind=kind; node.editable=0;
            dl_view(opaque,&node);
            NSControl *control=(NSControl *)host.views[@(kind)];
            NSString *painted=kind==1 ? [(NSButton *)control title] : [(NSTextField *)control stringValue];
            assert([painted isEqualToString:@"abcd\u2026"]);
            assert([control.accessibilityLabel isEqualToString:@"abcdefghij"]);
            node.text="full replacement"; node.description=NULL;
            dl_view(opaque,&node);
            assert([control.accessibilityLabel isEqualToString:@"full replacement"]);
            node.text="abcd\u2026"; node.description="abcdefghij";
        }
        float zero[3]; assert(droidless_font_metrics(0,0,0,zero));
        assert(zero[0]==0 && zero[1]==0 && zero[2]==0);
        assert(!droidless_font_metrics(3,0,20,zero) && !droidless_font_metrics(0,4,20,zero));
        assert(!droidless_font_metrics(0,0,NAN,zero) && !droidless_font_metrics(0,0,-1,zero));
        dl_destroy(opaque);
        puts("Native fonts: three families/four styles, matching metrics and shortened text with full accessibility labels passed.");
    }
}
