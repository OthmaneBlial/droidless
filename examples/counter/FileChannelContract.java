package org.droidless.counter;

import android.app.Activity;
import java.io.Closeable;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
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

    public static void output(Activity context) throws IOException {
        File file = new File(context.getFilesDir(), "channel.bin");
        FileOutputStream output = new FileOutputStream(file);
        FileChannel target = output.getChannel();
        FileInputStream input = new FileInputStream("/proc/self/cmdline");
        FileChannel source = input.getChannel();
        long size = source.size();
        check(output instanceof Closeable && target.isOpen());
        check(target.transferFrom(source, 0, size) == size);
        check(source.position() == size && target.position() == 0 && target.size() == size);
        output.write(0xab);
        check(target.position() == 1 && target.size() == size);
        output.flush();
        output.close();
        check(!target.isOpen());
        input.close();

        FileInputStream saved = new FileInputStream(file);
        check(saved.read() == 0xab && saved.read() == 'r');
        FileOutputStream copy = new FileOutputStream(
                new File(context.getFilesDir(), "copy.bin"));
        FileChannel copyChannel = copy.getChannel();
        long copied = saved.getChannel().transferTo(0, saved.getChannel().size(), copyChannel);
        check(copied == size && saved.getChannel().position() == 2);
        check(copyChannel.position() == size);
        copyChannel.close();
        saved.close();

        FileOutputStream append = new FileOutputStream(file, true);
        append.write(new byte[] {(byte) 0xfe, 0x7f}, 0, 2);
        append.close();
        FileInputStream appended = new FileInputStream(file);
        appended.getChannel().position(size);
        check(appended.read() == 0xfe && appended.read() == 0x7f);
        appended.close();

        FileOutputStream abandoned = new FileOutputStream(file);
        abandoned.write(0x55);
        FileInputStream unchanged = new FileInputStream(file);
        check(unchanged.read() == 0xab);
        unchanged.close();

        try {
            new FileOutputStream("/proc/self/cmdline");
            check(false);
        } catch (java.io.FileNotFoundException expected) { }
    }
}
