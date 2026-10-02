// Shared by text measurement and actual AppKit controls. No Android code.
#pragma once
#import <AppKit/AppKit.h>
#include <stdint.h>

static NSFont *droidlessFont(uint32_t family, uint32_t style, float size) {
    NSFont *font;
    switch (family) {
        case 1: font = [NSFont fontWithName:@"Times" size:size]; break;
        case 2: font = [NSFont monospacedSystemFontOfSize:size weight:NSFontWeightRegular]; break;
        default: font = [NSFont systemFontOfSize:size]; break;
    }
    if (!font) font = [NSFont systemFontOfSize:size];
    NSFontTraitMask traits = 0;
    if (style & 1) traits |= NSBoldFontMask;
    if (style & 2) traits |= NSItalicFontMask;
    if (traits) font = [[NSFontManager sharedFontManager] convertFont:font toHaveTrait:traits];
    return font;
}
