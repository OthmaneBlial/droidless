package org.droidless.counter;

import android.app.Activity;
import java.io.Closeable;
import java.io.FileInputStream;
import java.io.IOException;
import java.nio.channels.Channel;
import java.nio.channels.ClosedChannelException;
import java.nio.channels.FileChannel;
import java.nio.channels.ReadableByteChannel;
import java.nio.channels.WritableByteChannel;

public final class FileChannelContract {
    private static FileChannel retained;
    static void check(boolean value) {
        if (!value) throw new IllegalStateException("File channel contract failed");
    }
    public static FileChannel run(Activity context) throws IOException {
        FileInputStream input = new FileInputStream("/proc/self/cmdline");
        FileChannel channel = input.getChannel();
        check(input.getChannel() == channel && channel.isOpen());
        check(channel instanceof ReadableByteChannel && channel instanceof WritableByteChannel);
        check(channel instanceof Channel && channel instanceof Closeable);
        check(input instanceof Closeable);
        long size = context.getPackageName().length() + 1;
        check(channel.size() == size && channel.position() == 0);
        input.read();
        check(channel.position() == 1);
        input.skip(2);
        check(channel.position() == 3);
        check(channel.position(0) == channel && input.read() == 'o');
        channel.position(size + 17);
        check(channel.size() == size && input.read() == -1);
        check(input.available() == 0 && input.skip(100) == 0);
        try { channel.position(-1); check(false); }
        catch (IllegalArgumentException expected) { }
        check(channel.position() == size + 17);
        channel.position(1L << 40);
        check(channel.position() == (1L << 40) && input.read() == -1);
        input.close();
        check(!channel.isOpen() && input.getChannel() == channel);
        try { channel.size(); check(false); }
        catch (ClosedChannelException expected) { }
        try { channel.position(); check(false); }
        catch (IOException expected) { }
        try { channel.position(0); check(false); }
        catch (ClosedChannelException expected) { }
        channel.close();
        FileInputStream other = new FileInputStream("/proc/self/cmdline");
        other.getChannel().close();
        try { other.read(); check(false); }
        catch (IOException expected) { }
        other.close();
        retained = new FileInputStream("/proc/self/cmdline").getChannel();
        System.gc();
        check(retained.size() == size);
        return retained;
    }
    public static void release() { retained = null; }
}
