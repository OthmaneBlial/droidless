package org.droidless.counter;

/** Portable float extrema and rounding contract, compiled to DEX and desktop Java. */
public final class MathContract {
    static boolean same(float value, float expected) {
        return expected != expected ? value != value
            : value == expected && (expected != 0f || 1f/value == 1f/expected);
    }
    public static int run() {
        float nan = 0f/0f, infinity = 1f/0f;
        float[][] cases = {
            {1f, 2f, 1f, 2f}, {2f, 1f, 1f, 2f}, {-3f, 2f, -3f, 2f},
            {4f, 4f, 4f, 4f}, {0f, -0f, -0f, 0f}, {-0f, 0f, -0f, 0f},
            {0f, 0f, 0f, 0f}, {-0f, -0f, -0f, -0f},
            {-infinity, infinity, -infinity, infinity}, {infinity, -infinity, -infinity, infinity},
            {nan, 1f, nan, nan}, {1f, nan, nan, nan}, {nan, nan, nan, nan},
            {1.4e-45f, 0f, 0f, 1.4e-45f}
        };
        for (float[] values : cases) {
            if (!same(Math.min(values[0], values[1]), values[2])
                    || !same(Math.max(values[0], values[1]), values[3])) return 0;
        }
        float[] rounds = {0f, -0f, 0.49999997f, 0.5f, -0.5f, -0.50000006f, 1.5f, -1.5f,
            nan, infinity, -infinity, 2147483520f, 2147483648f};
        int[] expected = {0, 0, 0, 1, 0, -1, 2, -1, 0, Integer.MAX_VALUE, Integer.MIN_VALUE, 2147483520, Integer.MAX_VALUE};
        for (int i = 0; i < rounds.length; i++) if (Math.round(rounds[i]) != expected[i]) return 0;
        return 1;
    }
    public static void main(String[] args) {
        if (run() != 1) throw new IllegalStateException("Math contract failed");
        System.out.println("Float extrema and rounding contract passed");
    }
}
