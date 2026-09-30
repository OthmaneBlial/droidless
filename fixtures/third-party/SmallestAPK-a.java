package c.k;

import android.app.Activity;
import android.os.Bundle;
import android.widget.TextView;

public class a extends Activity
{
    @Override
    public void onCreate(Bundle savedInstanceState)
    {
        super.onCreate(savedInstanceState);
        TextView t = new TextView(this);
        t.setText(android.R.string.ok);
        setContentView(t);
    }
}
