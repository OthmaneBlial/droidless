#import "macos-font.h"
#include <math.h>

int droidless_font_metrics(uint32_t family, uint32_t style, float size, float *metrics) {
    if (!metrics || family > 2 || style > 3 || !isfinite(size) || size < 0 || size > 4096) return 0;
    @autoreleasepool {
        if (size == 0) { metrics[0] = metrics[1] = metrics[2] = 0; return 1; }
        NSFont *font = droidlessFont(family, style, size);
        if (!font) return 0;
        // Android y grows downward: ascent is negative, descent is positive.
        metrics[0] = -font.ascender;
        metrics[1] = -font.descender;
        metrics[2] = font.leading;
        return isfinite(metrics[0]) && isfinite(metrics[1]) && isfinite(metrics[2]);
    }
}
